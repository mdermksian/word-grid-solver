// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use std::time::Duration;

use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use word_grid_game_core::{CubeSet, GameRules, PlayMode, ScoreBand, ScoringTable};

use super::{BUTTON, GOLD, Preset, SELECTED, SelectedPreset, StartGameButton, text, value_row};
use crate::flow::Screen;
use crate::match_plugin::GameSet;

#[derive(Resource)]
pub(super) struct CustomGameDraft {
    pub grid_size: usize,
    seconds: String,
    endless: bool,
    minimum_length: String,
    points: Vec<String>,
}

impl Default for CustomGameDraft {
    fn default() -> Self {
        let rules = GameRules::normal(CubeSet::standard_new()).expect("standard rules are valid");
        let PlayMode::TimedRounds { duration } = rules.play_mode() else {
            unreachable!("standard games are timed")
        };
        Self {
            grid_size: rules.grid_size(),
            seconds: duration.as_secs().to_string(),
            endless: false,
            minimum_length: rules.minimum_word_length().to_string(),
            points: rules
                .scoring()
                .bands()
                .iter()
                .map(|band| band.points.to_string())
                .collect(),
        }
    }
}

impl CustomGameDraft {
    pub fn cube_set(&self) -> CubeSet {
        if self.grid_size == 5 {
            CubeSet::big()
        } else {
            CubeSet::standard_new()
        }
    }

    pub fn rules(&self) -> Result<GameRules, String> {
        if !matches!(self.grid_size, 4 | 5) {
            return Err("Choose a 4 x 4 or 5 x 5 grid.".into());
        }
        let minimum_length = self
            .minimum_length
            .parse::<usize>()
            .map_err(|_| "Minimum word length must be a whole number of at least 3.".to_string())?;
        let play_mode = if self.endless {
            PlayMode::Endless
        } else {
            let seconds = self.seconds.parse::<u64>().map_err(|_| {
                "Time limit must be a positive whole number of seconds.".to_string()
            })?;
            if seconds == 0 {
                return Err("Time limit must be greater than zero, or choose Endless.".into());
            }
            PlayMode::TimedRounds {
                duration: Duration::from_secs(seconds),
            }
        };
        let standard = ScoringTable::standard();
        let bands = standard
            .bands()
            .iter()
            .zip(&self.points)
            .map(|(band, points)| {
                points
                    .parse::<usize>()
                    .map(|points| ScoreBand {
                        minimum_length: band.minimum_length,
                        points,
                    })
                    .map_err(|_| {
                        "Scoring values must be whole numbers of zero or more.".to_string()
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let scoring = ScoringTable::new(bands).map_err(|error| error.to_string())?;
        GameRules::new(
            self.grid_size,
            minimum_length,
            play_mode,
            self.cube_set(),
            scoring,
        )
        .map_err(|error| error.to_string())
    }

    fn value(&self, field: NumberField) -> &str {
        match field {
            NumberField::Seconds => &self.seconds,
            NumberField::MinimumLength => &self.minimum_length,
            NumberField::Points(index) => &self.points[index],
        }
    }

    fn value_mut(&mut self, field: NumberField) -> &mut String {
        match field {
            NumberField::Seconds => &mut self.seconds,
            NumberField::MinimumLength => &mut self.minimum_length,
            NumberField::Points(index) => &mut self.points[index],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NumberField {
    Seconds,
    MinimumLength,
    Points(usize),
}

#[derive(Resource, Default)]
struct NumberFocus {
    field: Option<NumberField>,
    cursor: usize,
    select_all: bool,
}

#[derive(Component)]
struct NumericField(NumberField);

#[derive(Component)]
struct NumberValue(NumberField);

#[derive(Component)]
struct GridButton(usize);

#[derive(Component)]
struct TimingButton(bool);

#[derive(Component)]
struct ValidationMessage;

pub(super) struct CustomSetupPlugin;

impl Plugin for CustomSetupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CustomGameDraft>()
            .init_resource::<NumberFocus>()
            .add_systems(OnEnter(Screen::GameSetup), reset_custom)
            .add_systems(
                Update,
                (choose_options, focus_number, edit_number)
                    .chain()
                    .after(super::select_preset)
                    .before(super::start_game)
                    .in_set(GameSet::Input)
                    .run_if(in_state(Screen::GameSetup))
                    .run_if(custom_selected),
            )
            .add_systems(
                Update,
                (sync_controls, sync_start)
                    .after(super::refresh_rules)
                    .in_set(GameSet::Presentation)
                    .run_if(in_state(Screen::GameSetup)),
            );
    }
}

fn custom_selected(selection: Res<SelectedPreset>) -> bool {
    selection.0 == Preset::Custom
}

fn reset_custom(mut draft: ResMut<CustomGameDraft>, mut focus: ResMut<NumberFocus>) {
    *draft = CustomGameDraft::default();
    *focus = NumberFocus::default();
}

pub(super) fn spawn_editor(parent: &mut ChildSpawnerCommands, draft: &CustomGameDraft) {
    parent.spawn((
        text("", 18.0, Color::srgb(1.0, 0.55, 0.4)),
        ValidationMessage,
    ));
    value_row(parent, "Grid size (height x width)", |value| {
        for size in [4, 5] {
            value
                .spawn((
                    Button,
                    GridButton(size),
                    option_node(),
                    BackgroundColor(BUTTON),
                ))
                .with_children(|button| {
                    button.spawn(text(format!("{size} x {size}"), 20.0, Color::WHITE));
                });
        }
    });
    number_row(
        parent,
        "Minimum word length (at least 3)",
        NumberField::MinimumLength,
        draft,
    );
    value_row(parent, "Timing configuration", |value| {
        for (endless, label) in [(false, "Timed rounds"), (true, "Endless")] {
            value
                .spawn((
                    Button,
                    TimingButton(endless),
                    option_node(),
                    BackgroundColor(BUTTON),
                ))
                .with_children(|button| {
                    button.spawn(text(label, 20.0, Color::WHITE));
                });
        }
    });
    number_row(
        parent,
        "Time limit (whole seconds)",
        NumberField::Seconds,
        draft,
    );
    parent.spawn(text(
        "Endless rounds end only when you press End round.",
        16.0,
        GOLD,
    ));
    parent.spawn(text("Scoring table", 24.0, GOLD));
    parent.spawn(text(
        "Enter points for each word-length bucket. Zero is allowed.",
        16.0,
        Color::WHITE,
    ));
    let scoring = ScoringTable::standard();
    for index in 0..scoring.bands().len() {
        number_row(
            parent,
            &super::score_bucket_label(scoring.bands(), index),
            NumberField::Points(index),
            draft,
        );
    }
}

fn option_node() -> Node {
    Node {
        min_height: Val::Px(44.0),
        padding: UiRect::all(Val::Px(10.0)),
        align_items: AlignItems::Center,
        border_radius: BorderRadius::all(Val::Px(8.0)),
        ..default()
    }
}

fn number_row(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    field: NumberField,
    draft: &CustomGameDraft,
) {
    value_row(parent, label, |value| {
        value
            .spawn((
                Button,
                NumericField(field),
                Node {
                    width: Val::Px(160.0),
                    max_width: Val::Percent(100.0),
                    ..option_node()
                },
                BackgroundColor(Color::srgb(0.93, 0.88, 0.79)),
            ))
            .with_children(|input| {
                input.spawn((
                    text(draft.value(field), 20.0, Color::BLACK),
                    NumberValue(field),
                ));
            });
    });
}

fn choose_options(
    grids: Query<(&Interaction, &GridButton), Changed<Interaction>>,
    timings: Query<(&Interaction, &TimingButton), Changed<Interaction>>,
    mut draft: ResMut<CustomGameDraft>,
) {
    for (interaction, grid) in &grids {
        if *interaction == Interaction::Pressed && draft.grid_size != grid.0 {
            draft.grid_size = grid.0;
        }
    }
    for (interaction, timing) in &timings {
        if *interaction == Interaction::Pressed && draft.endless != timing.0 {
            draft.endless = timing.0;
        }
    }
}

type FocusButtons<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        Option<&'static NumericField>,
        Has<InteractionDisabled>,
    ),
    (With<Button>, Changed<Interaction>),
>;

fn focus_number(
    buttons: FocusButtons,
    draft: Res<CustomGameDraft>,
    mut focus: ResMut<NumberFocus>,
) {
    for (interaction, field, disabled) in &buttons {
        if *interaction != Interaction::Pressed || disabled {
            continue;
        }
        focus.field = field
            .map(|field| field.0)
            .filter(|field| *field != NumberField::Seconds || !draft.endless);
        focus.cursor = focus.field.map_or(0, |field| draft.value(field).len());
        focus.select_all = focus.field.is_some();
    }
    if draft.endless && focus.field == Some(NumberField::Seconds) {
        focus.field = None;
    }
}

fn edit_number(
    mut events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut draft: ResMut<CustomGameDraft>,
    mut focus: ResMut<NumberFocus>,
) {
    for event in events.read().filter(|event| event.state.is_pressed()) {
        if event.key_code == KeyCode::Tab {
            let mut fields = vec![NumberField::MinimumLength];
            if !draft.endless {
                fields.push(NumberField::Seconds);
            }
            fields.extend((0..draft.points.len()).map(NumberField::Points));
            let reverse = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
            let index = match focus
                .field
                .and_then(|field| fields.iter().position(|candidate| *candidate == field))
            {
                Some(index) if reverse => (index + fields.len() - 1) % fields.len(),
                Some(index) => (index + 1) % fields.len(),
                None if reverse => fields.len() - 1,
                None => 0,
            };
            focus.field = Some(fields[index]);
            focus.cursor = draft.value(fields[index]).len();
            focus.select_all = true;
            continue;
        }
        let Some(field) = focus.field else {
            continue;
        };
        if event.key_code == KeyCode::Enter || event.key_code == KeyCode::Escape {
            focus.field = None;
            continue;
        }
        let control = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
        if control {
            if event.key_code == KeyCode::KeyA {
                focus.select_all = true;
            }
            continue;
        }
        let value = draft.value_mut(field);
        focus.cursor = focus.cursor.min(value.len());
        match event.key_code {
            KeyCode::Home => {
                focus.cursor = 0;
                focus.select_all = false;
            }
            KeyCode::End => {
                focus.cursor = value.len();
                focus.select_all = false;
            }
            KeyCode::ArrowLeft => {
                focus.cursor = focus.cursor.saturating_sub(1);
                focus.select_all = false;
            }
            KeyCode::ArrowRight => {
                focus.cursor = (focus.cursor + 1).min(value.len());
                focus.select_all = false;
            }
            KeyCode::Backspace | KeyCode::Delete => {
                if focus.select_all {
                    value.clear();
                    focus.cursor = 0;
                } else if event.key_code == KeyCode::Backspace && focus.cursor > 0 {
                    focus.cursor -= 1;
                    value.remove(focus.cursor);
                } else if event.key_code == KeyCode::Delete && focus.cursor < value.len() {
                    value.remove(focus.cursor);
                }
                focus.select_all = false;
            }
            _ => {
                if let Some(input) = &event.text {
                    let digits = input
                        .chars()
                        .filter(char::is_ascii_digit)
                        .collect::<String>();
                    if !digits.is_empty() {
                        if focus.select_all {
                            value.clear();
                            focus.cursor = 0;
                            focus.select_all = false;
                        }
                        value.insert_str(focus.cursor, &digits);
                        focus.cursor += digits.len();
                    }
                }
            }
        }
    }
}

type NumericButtons<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static NumericField,
        &'static mut BackgroundColor,
        Has<InteractionDisabled>,
    ),
    Without<StartGameButton>,
>;

type OptionButtons<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut BackgroundColor,
        Option<&'static GridButton>,
        Option<&'static TimingButton>,
    ),
    (Without<NumericField>, Without<StartGameButton>),
>;

fn sync_start(
    draft: Res<CustomGameDraft>,
    selection: Res<SelectedPreset>,
    mut commands: Commands,
    mut starts: Query<
        (Entity, &mut BackgroundColor, Has<InteractionDisabled>),
        With<StartGameButton>,
    >,
) {
    let invalid = selection.0 == Preset::Custom && draft.rules().is_err();
    for (entity, mut background, disabled) in &mut starts {
        background.0 = if invalid { BUTTON } else { SELECTED };
        set_disabled(&mut commands, entity, disabled, invalid);
    }
}

fn sync_controls(
    draft: Res<CustomGameDraft>,
    selection: Res<SelectedPreset>,
    focus: Res<NumberFocus>,
    mut commands: Commands,
    mut texts: Query<(&mut Text, Option<&NumberValue>, Option<&ValidationMessage>)>,
    mut numeric: NumericButtons,
    mut options: OptionButtons,
) {
    let custom = selection.0 == Preset::Custom;
    let error = custom.then(|| draft.rules().err()).flatten();
    if !custom {
        return;
    }
    for (mut label, number, validation) in &mut texts {
        if validation.is_some() {
            label.0 = error.clone().unwrap_or_default();
        }
        if let Some(number) = number {
            let value = draft.value(number.0);
            label.0 = if focus.field == Some(number.0) {
                if focus.select_all {
                    format!("[{value}]")
                } else {
                    let cursor = focus.cursor.min(value.len());
                    format!("{}|{}", &value[..cursor], &value[cursor..])
                }
            } else {
                value.to_string()
            };
        }
    }
    for (entity, number, mut background, disabled) in &mut numeric {
        let inactive = number.0 == NumberField::Seconds && draft.endless;
        background.0 = if inactive {
            Color::srgb(0.45, 0.4, 0.35)
        } else if focus.field == Some(number.0) {
            GOLD
        } else {
            Color::srgb(0.93, 0.88, 0.79)
        };
        set_disabled(&mut commands, entity, disabled, inactive);
    }
    for (mut background, grid, timing) in &mut options {
        if let Some(grid) = grid {
            background.0 = if grid.0 == draft.grid_size {
                SELECTED
            } else {
                BUTTON
            };
        }
        if let Some(timing) = timing {
            background.0 = if timing.0 == draft.endless {
                SELECTED
            } else {
                BUTTON
            };
        }
    }
}

fn set_disabled(commands: &mut Commands, entity: Entity, current: bool, desired: bool) {
    if current != desired {
        if desired {
            commands.entity(entity).insert(InteractionDisabled);
        } else {
            commands.entity(entity).remove::<InteractionDisabled>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        content::{ContentCatalog, DictionaryAsset},
        flow::{FlowPlugin, RoundScreen},
        match_plugin::{ActiveMatch, LOCAL_PLAYER, MatchPlugin, PlayerIntent},
    };
    use bevy::input::{ButtonState, keyboard::Key, mouse::MouseWheel};
    use bevy::state::app::StatesPlugin;
    use word_grid_game_core::{Dictionary, MatchPhase};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(StatesPlugin)
            .insert_state(Screen::GameSetup)
            .insert_resource(Time::<()>::default())
            .init_resource::<Assets<DictionaryAsset>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<KeyboardInput>()
            .add_message::<MouseWheel>()
            .insert_resource(ContentCatalog {
                rules: GameRules::normal(CubeSet::standard_new()).unwrap(),
                dictionary: Handle::default(),
                cube_scene: Handle::default(),
            })
            .configure_sets(
                Update,
                (GameSet::Input, GameSet::Domain, GameSet::Presentation).chain(),
            )
            .add_plugins((FlowPlugin, MatchPlugin));
        app.update();
        click_label(&mut app, "Custom game");
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

    fn labeled_button(app: &mut App, label: &str) -> Entity {
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
        let button = labeled_button(app, label);
        click(app, button);
    }

    fn keyboard(app: &mut App, code: KeyCode, value: Option<&str>) {
        app.world_mut()
            .write_message(KeyboardInput {
                key_code: code,
                logical_key: value.map_or(
                    Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
                    |value| Key::Character(value.into()),
                ),
                state: ButtonState::Pressed,
                text: value.map(Into::into),
                repeat: false,
                window: Entity::PLACEHOLDER,
            })
            .unwrap();
        app.update();
    }

    fn enter_number(app: &mut App, field: NumberField, value: &str) {
        let entity = app
            .world_mut()
            .query::<(Entity, &NumericField)>()
            .iter(app.world())
            .find(|(_, number)| number.0 == field)
            .unwrap()
            .0;
        click(app, entity);
        keyboard(app, KeyCode::Digit0, Some(value));
    }

    fn intent(app: &mut App, intent: PlayerIntent) {
        app.world_mut().write_message(intent).unwrap();
        app.update();
        app.update();
    }

    #[test]
    fn custom_defaults_and_validation_follow_domain_rules() {
        let mut draft = CustomGameDraft::default();
        assert_eq!(
            draft.rules().unwrap(),
            GameRules::normal(CubeSet::standard_new()).unwrap()
        );
        draft.grid_size = 5;
        assert_eq!(draft.rules().unwrap().cube_set(), &CubeSet::big());
        draft.minimum_length = "2".into();
        assert!(draft.rules().is_err());
        draft.minimum_length = "4".into();
        for invalid in ["", "0", "-1", "1.5", "18446744073709551616"] {
            draft.seconds = invalid.into();
            assert!(draft.rules().is_err(), "accepted invalid time {invalid}");
        }
        draft.endless = true;
        assert_eq!(draft.rules().unwrap().play_mode(), PlayMode::Endless);
        for invalid in ["", "-1", "999999999999999999999999999999999999"] {
            draft.points[0] = invalid.into();
            assert!(draft.rules().is_err());
        }
        draft.points = vec!["0".into(); 5];
        assert!(
            draft
                .rules()
                .unwrap()
                .scoring()
                .bands()
                .iter()
                .all(|band| band.points == 0)
        );
        draft.grid_size = 6;
        assert!(draft.rules().is_err());
    }

    #[test]
    fn editor_launches_timed_custom_rules_and_applies_word_minimum_and_scores() {
        let mut app = setup_app();
        assert_eq!(
            app.world().resource::<CustomGameDraft>().rules().unwrap(),
            GameRules::normal(CubeSet::standard_new()).unwrap()
        );
        enter_number(&mut app, NumberField::Seconds, "1");
        enter_number(&mut app, NumberField::MinimumLength, "2");
        let start = labeled_button(&mut app, "Start game");
        assert!(app.world().get::<InteractionDisabled>(start).is_some());
        click(&mut app, start);
        assert!(!app.world().contains_resource::<ActiveMatch>());
        enter_number(&mut app, NumberField::MinimumLength, "4");
        for (index, points) in ["17", "9", "0", "42", "5"].into_iter().enumerate() {
            enter_number(&mut app, NumberField::Points(index), points);
        }
        click_label(&mut app, "5 x 5");
        click_label(&mut app, "Big game");
        click_label(&mut app, "Custom game");
        assert_eq!(app.world().resource::<CustomGameDraft>().seconds, "1");
        click_label(&mut app, "Start game");
        let rules = app.world().resource::<ActiveMatch>().0.rules();
        assert_eq!(rules.grid_size(), 5);
        assert_eq!(rules.cube_set(), &CubeSet::big());
        assert_eq!(rules.minimum_word_length(), 4);
        assert_eq!(
            rules.play_mode(),
            PlayMode::TimedRounds {
                duration: Duration::from_secs(1)
            }
        );
        assert_eq!(rules.scoring().score_length(4), 17);
        assert_eq!(
            rules
                .scoring()
                .bands()
                .iter()
                .map(|band| band.points)
                .collect::<Vec<_>>(),
            vec![17, 9, 0, 42, 5]
        );
        assert_eq!(
            rules
                .scoring()
                .bands()
                .iter()
                .map(|band| band.minimum_length)
                .collect::<Vec<_>>(),
            vec![3, 5, 6, 7, 8]
        );

        intent(&mut app, PlayerIntent::RollComplete);
        let paths = [vec![0, 1, 2], vec![0, 1, 2, 3]];
        let grid = app
            .world()
            .resource::<ActiveMatch>()
            .0
            .current_round()
            .unwrap()
            .board()
            .grid();
        let words = paths
            .iter()
            .map(|path| grid.word_for_path(path).unwrap())
            .collect::<Vec<_>>();
        let dictionary = app
            .world_mut()
            .resource_mut::<Assets<DictionaryAsset>>()
            .add(DictionaryAsset(Dictionary::from_words(words)));
        app.world_mut().resource_mut::<ContentCatalog>().dictionary = dictionary;
        for path in paths {
            intent(&mut app, PlayerIntent::SubmitPath(path));
        }
        let game = &app.world().resource::<ActiveMatch>().0;
        assert_eq!(
            game.current_round()
                .unwrap()
                .submissions(LOCAL_PLAYER)
                .len(),
            1
        );
        assert_eq!(game.current_score(LOCAL_PLAYER).unwrap(), 17);
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        app.update();
        app.update();
        assert_eq!(
            app.world().resource::<ActiveMatch>().0.phase(),
            MatchPhase::Review
        );
        assert_eq!(
            *app.world().resource::<State<RoundScreen>>().get(),
            RoundScreen::Review
        );
    }

    #[test]
    fn endless_custom_round_waits_for_manual_end_and_keeps_rules_for_next_round() {
        let mut app = setup_app();
        enter_number(&mut app, NumberField::Seconds, "0");
        click_label(&mut app, "Endless");
        let time = app
            .world_mut()
            .query::<(Entity, &NumericField)>()
            .iter(app.world())
            .find(|(_, field)| field.0 == NumberField::Seconds)
            .unwrap()
            .0;
        assert!(app.world().get::<InteractionDisabled>(time).is_some());
        click_label(&mut app, "Start game");
        let rules = app.world().resource::<ActiveMatch>().0.rules().clone();
        assert_eq!(rules.play_mode(), PlayMode::Endless);
        intent(&mut app, PlayerIntent::RollComplete);
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(86400));
        app.update();
        let game = &app.world().resource::<ActiveMatch>().0;
        assert_eq!(game.phase(), MatchPhase::Playing);
        assert_eq!(game.current_round().unwrap().remaining(), None);
        intent(&mut app, PlayerIntent::EndRound);
        assert_eq!(
            app.world().resource::<ActiveMatch>().0.phase(),
            MatchPhase::Review
        );
        intent(&mut app, PlayerIntent::StartNextRound);
        intent(&mut app, PlayerIntent::RollComplete);
        let game = &app.world().resource::<ActiveMatch>().0;
        assert_eq!(game.rules(), &rules);
        assert_eq!(game.phase(), MatchPhase::Playing);
        assert_eq!(game.current_round().unwrap().remaining(), None);
    }

    #[test]
    fn editing_supports_replacement_cursor_keys_and_tab_without_resetting_other_values() {
        let mut app = setup_app();
        enter_number(&mut app, NumberField::Seconds, "42");
        keyboard(&mut app, KeyCode::Home, None);
        keyboard(&mut app, KeyCode::Delete, None);
        assert_eq!(app.world().resource::<CustomGameDraft>().seconds, "2");
        keyboard(&mut app, KeyCode::Tab, None);
        assert_eq!(
            app.world().resource::<NumberFocus>().field,
            Some(NumberField::Points(0))
        );
        keyboard(&mut app, KeyCode::Digit0, Some("9"));
        assert_eq!(app.world().resource::<CustomGameDraft>().points[0], "9");
        assert_eq!(app.world().resource::<CustomGameDraft>().seconds, "2");
        click_label(&mut app, "Timed rounds");
        enter_number(&mut app, NumberField::Seconds, "0");
        click_label(&mut app, "Endless");
        let start = labeled_button(&mut app, "Start game");
        assert!(app.world().get::<InteractionDisabled>(start).is_none());
        click_label(&mut app, "Timed rounds");
        assert!(app.world().get::<InteractionDisabled>(start).is_some());
    }
}
