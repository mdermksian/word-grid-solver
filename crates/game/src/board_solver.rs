// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use std::collections::HashSet;

use bevy::input::{
    keyboard::KeyboardInput,
    mouse::{MouseScrollUnit, MouseWheel},
};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, futures::check_ready};
use bevy::ui::InteractionDisabled;
use word_grid_game_core::{CubeSet, GameRules};
use word_grid_solver::{FoundWord, GridSolver, WordGrid};

use crate::content::{ContentCatalog, DictionaryAsset};
use crate::flow::{MenuNavigationButton, Screen};
use crate::match_plugin::GameSet;
use crate::solutions::{SolutionWord, spawn_solution_words};

const GOLD: Color = Color::srgb(1.0, 0.86, 0.55);
const BUTTON: Color = Color::srgb(0.28, 0.18, 0.11);
const SELECTED: Color = Color::srgb(0.73, 0.28, 0.08);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Editing,
    Solving,
    Solved,
}

#[derive(Resource)]
struct SolverBoard {
    rules: GameRules,
    cells: Vec<String>,
    focus: Option<usize>,
    select_all: bool,
    stage: Stage,
    pending: Option<Task<Vec<FoundWord>>>,
    words: Vec<FoundWord>,
    highlight: Vec<usize>,
    error: String,
}

impl Default for SolverBoard {
    fn default() -> Self {
        Self::new(4)
    }
}

impl SolverBoard {
    fn new(size: usize) -> Self {
        let rules = if size == 5 {
            GameRules::big()
        } else {
            GameRules::normal(CubeSet::standard_new()).expect("standard rules are valid")
        };
        Self {
            cells: vec![String::new(); rules.grid_size().pow(2)],
            rules,
            focus: Some(0),
            select_all: true,
            stage: Stage::Editing,
            pending: None,
            words: Vec::new(),
            highlight: Vec::new(),
            error: String::new(),
        }
    }

    fn complete(&self) -> bool {
        self.cells
            .iter()
            .all(|cell| !cell.is_empty() && cell.chars().all(|letter| letter.is_ascii_alphabetic()))
    }

    fn move_focus(&mut self, reverse: bool) {
        self.focus = match (self.focus, reverse) {
            (Some(0), true) | (None, true) => Some(self.cells.len() - 1),
            (Some(index), true) => Some(index - 1),
            (Some(index), false) if index + 1 < self.cells.len() => Some(index + 1),
            (Some(_), false) => None,
            (None, false) => Some(0),
        };
        self.select_all = true;
    }
}

#[derive(Component, Clone, Copy)]
enum SolverAction {
    Size(usize),
    Cell(usize),
    Solve,
    Another,
}
#[derive(Component)]
struct TileLabel(usize);
#[derive(Component)]
struct PathOrder(usize);
#[derive(Component)]
struct GridContainer;
#[derive(Component)]
struct ResultsContainer;
#[derive(Component)]
struct SolverStatus;
#[derive(Component)]
struct ResultsHeading;

pub(crate) struct BoardSolverPlugin;

impl Plugin for BoardSolverPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SolverBoard>()
            .add_systems(
                OnEnter(Screen::SolveBoard),
                (reset_board, setup_screen).chain(),
            )
            .add_systems(OnExit(Screen::SolveBoard), reset_board)
            .add_systems(
                Update,
                (handle_buttons, keyboard_input, select_solution)
                    .chain()
                    .in_set(GameSet::Input)
                    .run_if(in_state(Screen::SolveBoard)),
            )
            .add_systems(
                Update,
                collect_solution
                    .in_set(GameSet::Domain)
                    .run_if(in_state(Screen::SolveBoard)),
            )
            .add_systems(
                Update,
                (render_grid, render_results, refresh_view, scroll_results)
                    .chain()
                    .in_set(GameSet::Presentation)
                    .run_if(in_state(Screen::SolveBoard)),
            );
    }
}

fn reset_board(mut board: ResMut<SolverBoard>) {
    *board = SolverBoard::default();
}

fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
        Node {
            flex_shrink: 0.0,
            ..default()
        },
    )
}

fn button_node() -> Node {
    Node {
        min_height: Val::Px(44.0),
        padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
        flex_shrink: 0.0,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border_radius: BorderRadius::all(Val::Px(8.0)),
        ..default()
    }
}

fn panel_node() -> Node {
    Node {
        flex_grow: 1.0,
        flex_basis: Val::Px(0.0),
        min_width: Val::Px(0.0),
        min_height: Val::Px(0.0),
        padding: UiRect::all(Val::Px(20.0)),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(16.0),
        border_radius: BorderRadius::all(Val::Px(16.0)),
        ..default()
    }
}

fn setup_screen(mut commands: Commands) {
    commands.spawn((Camera2d, DespawnOnExit(Screen::SolveBoard)));
    commands.spawn((
        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), padding: UiRect::all(Val::Px(24.0)),
            flex_direction: FlexDirection::Column, row_gap: Val::Px(24.0), ..default() },
        BackgroundColor(Color::srgb(0.12, 0.055, 0.025)), DespawnOnExit(Screen::SolveBoard),
    )).with_children(|root| {
        root.spawn(Node { height: Val::Px(80.0), flex_shrink: 0.0, align_items: AlignItems::Center,
            justify_content: JustifyContent::Center, ..default() }).with_children(|header| {
                header.spawn(text("Solve Board", 48.0, GOLD));
            });
        root.spawn((Button, MenuNavigationButton(Screen::Menu),
            Node { position_type: PositionType::Absolute, left: Val::Px(16.0), top: Val::Px(16.0), ..button_node() },
            BackgroundColor(BUTTON),
        )).with_children(|button| { button.spawn(text("BACK TO MAIN MENU", 18.0, Color::WHITE)); });
        root.spawn(Node { flex_grow: 1.0, min_height: Val::Px(0.0), column_gap: Val::Px(24.0), ..default() })
            .with_children(|body| {
                body.spawn((panel_node(), BackgroundColor(Color::srgb(0.18, 0.09, 0.045))))
                    .with_children(|left| {
                        left.spawn(text("Enter board", 28.0, GOLD));
                        left.spawn(Node { column_gap: Val::Px(12.0), flex_shrink: 0.0, ..default() }).with_children(|sizes| {
                            for size in [4, 5] {
                                sizes.spawn((Button, SolverAction::Size(size), button_node(), BackgroundColor(BUTTON)))
                                    .with_children(|button| { button.spawn(text(format!("{size} x {size}"), 22.0, Color::WHITE)); });
                            }
                        });
                        left.spawn(text("Enter letters, including multi-letter tiles such as QU.\nTab/Enter: next tile. Shift+Tab: previous. Click a tile to edit.", 16.0, Color::WHITE));
                        left.spawn(Node { width: Val::Percent(100.0), flex_grow: 1.0, flex_basis: Val::Px(0.0),
                            min_height: Val::Px(0.0), justify_content: JustifyContent::Center, ..default() }).with_children(|area| {
                                area.spawn((Node { height: Val::Percent(100.0), max_width: Val::Percent(100.0),
                                    aspect_ratio: Some(1.0), display: Display::Grid, row_gap: Val::Px(8.0), column_gap: Val::Px(8.0), ..default() }, GridContainer));
                            });
                        left.spawn((text("", 17.0, GOLD), SolverStatus));
                        left.spawn(Node { column_gap: Val::Px(12.0), flex_wrap: FlexWrap::Wrap, flex_shrink: 0.0, ..default() }).with_children(|actions| {
                            actions.spawn((Button, SolverAction::Solve, button_node(), BackgroundColor(SELECTED)))
                                .with_children(|button| { button.spawn(text("Solve grid", 22.0, Color::WHITE)); });
                            actions.spawn((Button, SolverAction::Another, button_node(), BackgroundColor(SELECTED)))
                                .with_children(|button| { button.spawn(text("Solve another board", 22.0, Color::WHITE)); });
                        });
                    });
                body.spawn((panel_node(), BackgroundColor(Color::srgba(0.08, 0.06, 0.04, 0.96))))
                    .with_children(|right| {
                        right.spawn((text("ALL SOLUTIONS", 28.0, GOLD), ResultsHeading));
                        right.spawn((Node { width: Val::Percent(100.0), flex_grow: 1.0, min_height: Val::Px(0.0),
                            flex_direction: FlexDirection::Column, row_gap: Val::Px(5.0), overflow: Overflow::scroll_y(), ..default() },
                            ScrollPosition::default(), ResultsContainer));
                    });
            });
    });
}

fn handle_buttons(
    buttons: Query<(&Interaction, &SolverAction, Has<InteractionDisabled>), Changed<Interaction>>,
    mut board: ResMut<SolverBoard>,
    catalog: Res<ContentCatalog>,
    dictionaries: Res<Assets<DictionaryAsset>>,
) {
    for (interaction, action, disabled) in &buttons {
        if *interaction != Interaction::Pressed || disabled {
            continue;
        }
        match *action {
            SolverAction::Size(size)
                if board.stage == Stage::Editing && size != board.rules.grid_size() =>
            {
                *board = SolverBoard::new(size);
            }
            SolverAction::Cell(index) if board.stage == Stage::Editing => {
                board.focus = Some(index);
                board.select_all = true;
            }
            SolverAction::Another if board.stage == Stage::Solved => {
                *board = SolverBoard::new(board.rules.grid_size());
            }
            SolverAction::Solve if board.stage == Stage::Editing && board.complete() => {
                let Some(dictionary) = dictionaries.get(&catalog.dictionary) else {
                    board.error = "Dictionary is still loading. Please try again.".into();
                    continue;
                };
                let Ok(grid) = WordGrid::new(board.rules.grid_size(), board.cells.clone()) else {
                    continue;
                };
                let dictionary = dictionary.0.clone();
                let minimum_length = board.rules.minimum_word_length();
                board.pending = Some(AsyncComputeTaskPool::get().spawn(async move {
                    GridSolver::new(dictionary, minimum_length).find_words(&grid)
                }));
                board.stage = Stage::Solving;
                board.focus = None;
                board.error.clear();
            }
            _ => {}
        }
    }
}

fn keyboard_input(
    mut events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut board: ResMut<SolverBoard>,
) {
    for event in events.read().filter(|event| event.state.is_pressed()) {
        if board.stage != Stage::Editing {
            continue;
        }
        if matches!(event.key_code, KeyCode::Tab | KeyCode::Enter) {
            let reverse = event.key_code == KeyCode::Tab
                && (keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight));
            board.move_focus(reverse);
            continue;
        }
        let Some(index) = board.focus else {
            continue;
        };
        if keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight) {
            if event.key_code == KeyCode::KeyA {
                board.select_all = true;
            }
            continue;
        }
        match event.key_code {
            KeyCode::ArrowLeft if index > 0 => {
                board.focus = Some(index - 1);
                board.select_all = true;
            }
            KeyCode::ArrowRight if index + 1 < board.cells.len() => {
                board.focus = Some(index + 1);
                board.select_all = true;
            }
            KeyCode::ArrowUp if index >= board.rules.grid_size() => {
                board.focus = Some(index - board.rules.grid_size());
                board.select_all = true;
            }
            KeyCode::ArrowDown if index + board.rules.grid_size() < board.cells.len() => {
                board.focus = Some(index + board.rules.grid_size());
                board.select_all = true;
            }
            KeyCode::Backspace => {
                if board.select_all {
                    board.cells[index].clear();
                } else if board.cells[index].is_empty() && index > 0 {
                    board.focus = Some(index - 1);
                    board.cells[index - 1].pop();
                } else {
                    board.cells[index].pop();
                }
                board.select_all = false;
                board.error.clear();
            }
            KeyCode::Delete => {
                board.cells[index].clear();
                board.select_all = false;
                board.error.clear();
            }
            _ => {
                if let Some(input) = &event.text {
                    let letters = input
                        .chars()
                        .filter(char::is_ascii_alphabetic)
                        .map(|letter| letter.to_ascii_lowercase())
                        .collect::<String>();
                    if !letters.is_empty() {
                        if board.select_all {
                            board.cells[index].clear();
                            board.select_all = false;
                        }
                        board.cells[index].push_str(&letters);
                        board.error.clear();
                    }
                }
            }
        }
    }
}

fn collect_solution(mut board: ResMut<SolverBoard>) {
    let ready = board.pending.as_mut().and_then(check_ready);
    if let Some(words) = ready {
        board.pending = None;
        board.words = words;
        board.stage = Stage::Solved;
    }
}

fn select_solution(
    buttons: Query<(&Interaction, &SolutionWord), Changed<Interaction>>,
    mut board: ResMut<SolverBoard>,
) {
    if board.stage != Stage::Solved {
        return;
    }
    for (interaction, word) in &buttons {
        if *interaction == Interaction::Pressed {
            board.highlight.clone_from(&word.path);
        }
    }
}

fn render_grid(
    mut commands: Commands,
    board: Res<SolverBoard>,
    mut grids: Query<(Entity, &mut Node), With<GridContainer>>,
    mut rendered: Local<Option<(Entity, usize)>>,
) {
    let Ok((grid, mut node)) = grids.single_mut() else {
        return;
    };
    let size = board.rules.grid_size();
    if *rendered == Some((grid, size)) {
        return;
    }
    *rendered = Some((grid, size));
    node.grid_template_columns = RepeatedGridTrack::flex(size as u16, 1.0);
    node.grid_template_rows = RepeatedGridTrack::flex(size as u16, 1.0);
    commands.entity(grid).despawn_related::<Children>();
    commands.entity(grid).with_children(|cells| {
        for index in 0..size.pow(2) {
            cells
                .spawn((
                    Button,
                    SolverAction::Cell(index),
                    Node {
                        min_width: Val::Px(0.0),
                        min_height: Val::Px(0.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BorderColor::all(BUTTON),
                    BackgroundColor(Color::srgb(0.93, 0.88, 0.79)),
                ))
                .with_children(|cell| {
                    cell.spawn((text("", 26.0, Color::BLACK), TileLabel(index)));
                    cell.spawn((text("", 13.0, Color::BLACK), PathOrder(index)))
                        .insert(Node {
                            position_type: PositionType::Absolute,
                            top: Val::Px(2.0),
                            left: Val::Px(4.0),
                            ..default()
                        });
                });
        }
    });
}

fn render_results(
    mut commands: Commands,
    board: Res<SolverBoard>,
    mut lists: Query<(Entity, &mut ScrollPosition), With<ResultsContainer>>,
    mut rendered: Local<Option<(Entity, Stage)>>,
) {
    let Ok((list, mut scroll)) = lists.single_mut() else {
        return;
    };
    if *rendered == Some((list, board.stage)) {
        return;
    }
    *rendered = Some((list, board.stage));
    scroll.0 = Vec2::ZERO;
    commands.entity(list).despawn_related::<Children>();
    commands
        .entity(list)
        .with_children(|content| match board.stage {
            Stage::Editing => {
                content.spawn(text(
                    "Complete the board, then choose Solve grid.",
                    20.0,
                    Color::WHITE,
                ));
            }
            Stage::Solving => {
                content.spawn(text("Solving...", 20.0, GOLD));
            }
            Stage::Solved if board.words.is_empty() => {
                content.spawn(text("No words found for this board.", 20.0, Color::WHITE));
            }
            Stage::Solved => spawn_solution_words(
                content,
                &board.words,
                board.rules.scoring(),
                &HashSet::new(),
            ),
        });
}

type SolverLabels<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Text,
        Option<&'static TileLabel>,
        Option<&'static PathOrder>,
        Has<SolverStatus>,
        Has<ResultsHeading>,
    ),
>;
type SolverButtons<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static SolverAction,
        &'static mut Node,
        &'static mut BackgroundColor,
        Option<&'static mut BorderColor>,
        Has<InteractionDisabled>,
    ),
>;

fn refresh_view(
    mut commands: Commands,
    board: Res<SolverBoard>,
    mut labels: SolverLabels,
    mut buttons: SolverButtons,
    mut words: Query<(&SolutionWord, &Interaction, &mut BackgroundColor), Without<SolverAction>>,
) {
    for (mut label, cell, order, status, heading) in &mut labels {
        if let Some(cell) = cell {
            label.0 = if board.cells[cell.0].is_empty() {
                "_".into()
            } else {
                board.cells[cell.0].to_ascii_uppercase()
            };
        }
        if let Some(order) = order {
            label.0 = board
                .highlight
                .iter()
                .position(|index| *index == order.0)
                .map_or_else(String::new, |index| (index + 1).to_string());
        }
        if status {
            label.0 = if !board.error.is_empty() {
                board.error.clone()
            } else {
                match board.stage {
                    Stage::Editing => format!(
                        "{} / {} tiles filled - {} letter minimum",
                        board.cells.iter().filter(|cell| !cell.is_empty()).count(),
                        board.cells.len(),
                        board.rules.minimum_word_length()
                    ),
                    Stage::Solving => "Solving...".into(),
                    Stage::Solved => "Click a word to highlight its numbered path.".into(),
                }
            };
        }
        if heading {
            label.0 = if board.stage == Stage::Solved {
                format!("ALL SOLUTIONS - {}", board.words.len())
            } else {
                "ALL SOLUTIONS".into()
            };
        }
    }
    for (entity, action, mut node, mut background, border, disabled) in &mut buttons {
        let inactive = match *action {
            SolverAction::Cell(index) => {
                background.0 = if board.highlight.contains(&index) {
                    GOLD
                } else {
                    Color::srgb(0.93, 0.88, 0.79)
                };
                if let Some(mut border) = border {
                    *border = BorderColor::all(if board.focus == Some(index) {
                        SELECTED
                    } else {
                        BUTTON
                    });
                }
                board.stage != Stage::Editing
            }
            SolverAction::Size(size) => {
                background.0 = if size == board.rules.grid_size() {
                    SELECTED
                } else {
                    BUTTON
                };
                board.stage != Stage::Editing
            }
            SolverAction::Solve => {
                node.display = if board.stage == Stage::Solved {
                    Display::None
                } else {
                    Display::Flex
                };
                let inactive = board.stage != Stage::Editing || !board.complete();
                background.0 = if inactive { BUTTON } else { SELECTED };
                inactive
            }
            SolverAction::Another => {
                node.display = if board.stage == Stage::Solved {
                    Display::Flex
                } else {
                    Display::None
                };
                board.stage != Stage::Solved
            }
        };
        if inactive != disabled {
            if inactive {
                commands.entity(entity).insert(InteractionDisabled);
            } else {
                commands.entity(entity).remove::<InteractionDisabled>();
            }
        }
    }
    for (word, interaction, mut background) in &mut words {
        background.0 = if board.highlight == word.path {
            SELECTED
        } else if *interaction == Interaction::Hovered {
            Color::srgb(0.42, 0.22, 0.1)
        } else {
            Color::srgb(0.22, 0.15, 0.1)
        };
    }
}

fn scroll_results(
    mut wheel: MessageReader<MouseWheel>,
    mut lists: Query<(&mut ScrollPosition, &ComputedNode), With<ResultsContainer>>,
) {
    let delta = wheel
        .read()
        .map(|event| {
            -event.y
                * if event.unit == MouseScrollUnit::Line {
                    24.0
                } else {
                    1.0
                }
        })
        .sum::<f32>();
    for (mut scroll, computed) in &mut lists {
        let max_offset = (computed.content_size().y - computed.size().y).max(0.0)
            * computed.inverse_scale_factor();
        scroll.y = (scroll.y + delta).clamp(0.0, max_offset);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flow::FlowPlugin;
    use bevy::{
        input::{ButtonState, keyboard::Key},
        state::app::StatesPlugin,
    };
    use std::time::{Duration, Instant};
    use word_grid_solver::Dictionary;

    fn solver_app() -> App {
        let mut dictionaries = Assets::<DictionaryAsset>::default();
        let dictionary = dictionaries.add(DictionaryAsset(Dictionary::from_words([
            "cat", "cats", "quit", "quits", "qat", "qats", "at",
        ])));
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), StatesPlugin))
            .insert_state(Screen::Menu)
            .insert_resource(dictionaries)
            .insert_resource(ContentCatalog {
                rules: GameRules::normal(CubeSet::standard_new()).unwrap(),
                dictionary,
                cube_scene: Handle::default(),
            })
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<KeyboardInput>()
            .add_message::<MouseWheel>()
            .configure_sets(
                Update,
                (GameSet::Input, GameSet::Domain, GameSet::Presentation).chain(),
            )
            .add_plugins(FlowPlugin);
        app.update();
        click_label(&mut app, "SOLVE BOARD");
        assert_eq!(
            *app.world().resource::<State<Screen>>().get(),
            Screen::SolveBoard
        );
        app
    }

    fn click(app: &mut App, entity: Entity) {
        for mut interaction in app
            .world_mut()
            .query::<&mut Interaction>()
            .iter_mut(app.world_mut())
        {
            *interaction = Interaction::None;
        }
        app.world_mut()
            .entity_mut(entity)
            .insert(Interaction::Pressed);
        app.update();
        app.update();
    }

    fn button(app: &mut App, label: &str) -> Entity {
        app.world_mut()
            .query_filtered::<(Entity, &Children), With<Button>>()
            .iter(app.world())
            .find(|(_, children)| {
                children.iter().any(|child| {
                    app.world()
                        .get::<Text>(child)
                        .is_some_and(|text| text.0 == label)
                })
            })
            .map(|(entity, _)| entity)
            .unwrap_or_else(|| panic!("missing button: {label}"))
    }

    fn click_label(app: &mut App, label: &str) {
        let entity = button(app, label);
        click(app, entity);
    }

    fn key(app: &mut App, key_code: KeyCode, value: Option<&str>) {
        app.world_mut()
            .write_message(KeyboardInput {
                key_code,
                logical_key: Key::Character(value.unwrap_or("").into()),
                state: ButtonState::Pressed,
                text: value.map(Into::into),
                repeat: false,
                window: Entity::PLACEHOLDER,
            })
            .unwrap();
        app.update();
    }

    fn fill(app: &mut App, cells: &[&str]) {
        for cell in cells {
            key(app, KeyCode::KeyA, Some(cell));
            key(app, KeyCode::Tab, None);
        }
    }

    fn wait_for_solution(app: &mut App) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while app.world().resource::<SolverBoard>().stage == Stage::Solving {
            assert!(Instant::now() < deadline, "solver did not finish");
            std::thread::sleep(Duration::from_millis(1));
            app.update();
        }
        assert!(app.world().resource::<SolverBoard>().stage == Stage::Solved);
    }

    #[test]
    fn explicit_multi_letter_entry_navigation_and_resize_keep_grid_valid() {
        let mut app = solver_app();
        let solve = button(&mut app, "Solve grid");
        assert!(app.world().get::<InteractionDisabled>(solve).is_some());
        click(&mut app, solve);
        assert!(app.world().resource::<SolverBoard>().pending.is_none());
        key(&mut app, KeyCode::Digit1, Some("123-!"));
        assert!(app.world().resource::<SolverBoard>().cells[0].is_empty());
        key(&mut app, KeyCode::KeyQ, Some("Q"));
        key(&mut app, KeyCode::KeyU, Some("u"));
        let board = app.world().resource::<SolverBoard>();
        assert_eq!(board.focus, Some(0));
        assert_eq!(board.cells[0], "qu");
        key(&mut app, KeyCode::Enter, None);
        assert_eq!(app.world().resource::<SolverBoard>().focus, Some(1));
        key(&mut app, KeyCode::KeyI, Some("i"));
        key(&mut app, KeyCode::ArrowLeft, None);
        key(&mut app, KeyCode::KeyQ, Some("q"));
        assert_eq!(app.world().resource::<SolverBoard>().cells[0], "q");
        click_label(&mut app, "5 x 5");
        let board = app.world().resource::<SolverBoard>();
        assert_eq!(board.cells.len(), 25);
        assert_eq!(board.focus, Some(0));
        assert!(board.cells.iter().all(String::is_empty));
        assert_eq!(
            app.world_mut()
                .query::<&TileLabel>()
                .iter(app.world())
                .count(),
            25
        );
        click_label(&mut app, "4 x 4");
        assert_eq!(
            app.world_mut()
                .query::<&TileLabel>()
                .iter(app.world())
                .count(),
            16
        );
        key(&mut app, KeyCode::KeyQ, Some("qu"));
        key(&mut app, KeyCode::Tab, None);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ShiftLeft);
        key(&mut app, KeyCode::Tab, None);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::ShiftLeft);
        assert_eq!(app.world().resource::<SolverBoard>().focus, Some(0));
    }

    #[test]
    fn solving_both_sizes_uses_game_defaults_shared_rows_and_highlights_paths() {
        for size in [4, 5] {
            let mut app = solver_app();
            if size == 5 {
                click_label(&mut app, "5 x 5");
            }
            let mut cells = vec!["x"; size * size];
            if size == 4 {
                cells[..4].copy_from_slice(&["qu", "i", "t", "s"]);
                cells[4..8].copy_from_slice(&["c", "a", "r", "d"]);
            } else {
                cells[..4].copy_from_slice(&["q", "a", "t", "s"]);
                cells[5..9].copy_from_slice(&["qu", "i", "t", "s"]);
            }
            fill(&mut app, &cells);
            assert!(app.world().resource::<SolverBoard>().complete());
            let solve = button(&mut app, "Solve grid");
            assert!(app.world().get::<InteractionDisabled>(solve).is_none());
            click(&mut app, solve);
            wait_for_solution(&mut app);
            let board = app.world().resource::<SolverBoard>();
            let expected = if size == 4 {
                vec!["quits", "cats", "quit", "cat"]
            } else {
                vec!["quits", "qats", "quit"]
            };
            assert_eq!(
                board
                    .words
                    .iter()
                    .map(|word| word.word.as_str())
                    .collect::<Vec<_>>(),
                expected
            );
            let grid = WordGrid::new(size, board.cells.clone()).unwrap();
            for word in &board.words {
                assert_eq!(grid.word_for_path(&word.path).unwrap(), word.word);
            }
            let first = board.words[0].clone();
            let word_row = button(&mut app, &first.word.to_ascii_uppercase());
            let row = app.world().get::<SolutionWord>(word_row).unwrap();
            assert_eq!(row.path, first.path);
            let child_text = app
                .world()
                .get::<Children>(word_row)
                .unwrap()
                .iter()
                .map(|child| app.world().get::<Text>(child).unwrap().0.clone())
                .collect::<Vec<_>>();
            assert_eq!(
                child_text[1],
                app.world()
                    .resource::<SolverBoard>()
                    .rules
                    .scoring()
                    .score_word(&first.word)
                    .to_string()
            );
            click(&mut app, word_row);
            assert_eq!(app.world().resource::<SolverBoard>().highlight, first.path);
            for (order, label) in app
                .world_mut()
                .query::<(&PathOrder, &Text)>()
                .iter(app.world())
            {
                if let Some(index) = first.path.iter().position(|cell| *cell == order.0) {
                    assert_eq!(label.0, (index + 1).to_string());
                } else {
                    assert!(label.0.is_empty());
                }
            }
            click_label(&mut app, "Solve another board");
            let board = app.world().resource::<SolverBoard>();
            assert!(board.stage == Stage::Editing);
            assert_eq!(board.rules.grid_size(), size);
            assert_eq!(board.focus, Some(0));
            assert!(board.cells.iter().all(String::is_empty));
            assert!(board.words.is_empty() && board.highlight.is_empty());
            assert_eq!(
                app.world_mut()
                    .query::<&SolutionWord>()
                    .iter(app.world())
                    .count(),
                0
            );
        }
    }

    #[test]
    fn zero_results_and_leaving_during_a_solve_do_not_leave_stale_state() {
        let mut app = solver_app();
        fill(&mut app, &["x"; 16]);
        click_label(&mut app, "Solve grid");
        wait_for_solution(&mut app);
        assert!(app.world().resource::<SolverBoard>().words.is_empty());
        assert!(
            app.world_mut()
                .query::<&Text>()
                .iter(app.world())
                .any(|text| text.0 == "No words found for this board.")
        );
        click_label(&mut app, "Solve another board");
        {
            let mut board = app.world_mut().resource_mut::<SolverBoard>();
            board.pending = Some(AsyncComputeTaskPool::get().spawn(std::future::pending()));
            board.stage = Stage::Solving;
        }
        app.update();
        key(&mut app, KeyCode::KeyA, Some("a"));
        assert!(
            app.world()
                .resource::<SolverBoard>()
                .cells
                .iter()
                .all(String::is_empty)
        );
        click_label(&mut app, "BACK TO MAIN MENU");
        assert_eq!(*app.world().resource::<State<Screen>>().get(), Screen::Menu);
        assert!(app.world().resource::<SolverBoard>().pending.is_none());
        click_label(&mut app, "SOLVE BOARD");
        let board = app.world().resource::<SolverBoard>();
        assert!(board.stage == Stage::Editing);
        assert_eq!(board.cells.len(), 16);
        assert!(board.cells.iter().all(String::is_empty));
    }
}
