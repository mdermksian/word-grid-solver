// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use bevy::{app::AppExit, prelude::*};
use word_grid_game_core::MatchPhase;

use crate::content::ContentCatalog;
use crate::match_plugin::{ActiveMatch, GameSet, PlayerIntent};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
#[states(scoped_entities)]
pub enum Screen {
    #[default]
    Loading,
    Menu,
    GameSetup,
    Match,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, SubStates)]
#[source(Screen = Screen::Match)]
#[states(scoped_entities)]
pub enum RoundScreen {
    #[default]
    Rolling,
    Playing,
    Review,
}

#[derive(Component)]
struct StartGameButton(PlayerIntent);

#[derive(Component)]
struct QuitButton;

#[derive(Component)]
struct MenuNavigationButton(Screen);

#[derive(Component)]
struct LoadingStatus;

pub struct FlowPlugin;

impl Plugin for FlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<Screen>()
            .add_sub_state::<RoundScreen>()
            .add_systems(OnEnter(Screen::Loading), setup_loading_screen)
            .add_systems(OnEnter(Screen::Menu), setup_menu_screen)
            .add_systems(OnEnter(Screen::GameSetup), setup_game_setup_screen)
            .add_systems(
                Update,
                (
                    wait_for_content.run_if(in_state(Screen::Loading)),
                    quit_button
                        .in_set(GameSet::Input)
                        .run_if(in_state(Screen::Menu)),
                    start_game_button
                        .in_set(GameSet::Input)
                        .run_if(in_state(Screen::GameSetup)),
                    navigate_menu
                        .in_set(GameSet::Input)
                        .run_if(in_state(Screen::Menu).or_else(in_state(Screen::GameSetup))),
                    project_match_phase.in_set(GameSet::Presentation),
                ),
            );
    }
}

fn setup_loading_screen(mut commands: Commands) {
    commands.spawn((Camera2d, DespawnOnExit(Screen::Loading)));
    commands.spawn((
        Text::new("LOADING WORD GRID..."),
        TextFont {
            font_size: FontSize::Px(34.0),
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.83, 0.58)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(45.0),
            ..default()
        },
        UiTransform::from_translation(Val2::px(-170.0, -20.0)),
        LoadingStatus,
        DespawnOnExit(Screen::Loading),
    ));
}

fn wait_for_content(
    catalog: Option<Res<ContentCatalog>>,
    asset_server: Res<AssetServer>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut status: Query<&mut Text, With<LoadingStatus>>,
) {
    let Some(catalog) = catalog else {
        return;
    };
    let dictionary_failed = asset_server.load_state(catalog.dictionary.id()).is_failed()
        || asset_server
            .recursive_dependency_load_state(catalog.dictionary.id())
            .is_failed();
    let cube_failed = asset_server.load_state(catalog.cube_scene.id()).is_failed()
        || asset_server
            .recursive_dependency_load_state(catalog.cube_scene.id())
            .is_failed();
    if dictionary_failed || cube_failed {
        if let Ok(mut text) = status.single_mut() {
            text.0 = "COULD NOT LOAD GAME ASSETS\nCheck the assets directory and restart.".into();
        }
        return;
    }
    if asset_server.is_loaded_with_dependencies(catalog.dictionary.id())
        && asset_server.is_loaded_with_dependencies(catalog.cube_scene.id())
    {
        next_screen.set(Screen::Menu);
    }
}

fn menu_root(screen: Screen) -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(24.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.12, 0.055, 0.025)),
        DespawnOnExit(screen),
    )
}

fn menu_button(height: f32) -> impl Bundle {
    (
        Button,
        Node {
            width: Val::Px(330.0),
            height: Val::Px(height),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.73, 0.28, 0.08)),
    )
}

fn button_label(label: &str) -> impl Bundle {
    (
        Text::new(label),
        TextFont {
            font_size: FontSize::Px(22.0),
            ..default()
        },
        TextColor::WHITE,
    )
}

fn setup_menu_screen(mut commands: Commands) {
    commands.spawn((Camera2d, DespawnOnExit(Screen::Menu)));
    commands
        .spawn(menu_root(Screen::Menu))
        .with_children(|parent| {
            parent.spawn((
                Text::new("WORD GRID"),
                TextFont {
                    font_size: FontSize::Px(58.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.86, 0.55)),
            ));
            parent
                .spawn((menu_button(76.0), MenuNavigationButton(Screen::GameSetup)))
                .with_children(|button| {
                    button.spawn(button_label("PLAY GAME"));
                });
            parent.spawn(menu_button(76.0)).with_children(|button| {
                button.spawn(button_label("SOLVE BOARD"));
            });
            parent
                .spawn((menu_button(56.0), QuitButton))
                .insert(BackgroundColor(Color::srgb(0.28, 0.18, 0.11)))
                .with_children(|button| {
                    button.spawn(button_label("QUIT"));
                });
        });
}

fn setup_game_setup_screen(mut commands: Commands) {
    commands.spawn((Camera2d, DespawnOnExit(Screen::GameSetup)));
    commands
        .spawn(menu_root(Screen::GameSetup))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PLAY GAME"),
                TextFont {
                    font_size: FontSize::Px(58.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.86, 0.55)),
            ));
            parent
                .spawn((
                    Button,
                    MenuNavigationButton(Screen::Menu),
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(16.0),
                        top: Val::Px(16.0),
                        padding: UiRect::axes(Val::Px(20.0), Val::Px(14.0)),
                        border_radius: BorderRadius::all(Val::Px(12.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.28, 0.18, 0.11)),
                ))
                .with_children(|button| {
                    button.spawn(button_label("BACK TO MAIN MENU"));
                });
            for (intent, label, description) in [
                (
                    PlayerIntent::StartNormal,
                    "START NORMAL - 3 MINUTES",
                    "4 x 4 - Standard New cubes - 3 letter minimum",
                ),
                (
                    PlayerIntent::StartBig,
                    "START BIG - 3 MINUTES",
                    "5 x 5 - Big cubes - 4 letter minimum",
                ),
            ] {
                parent
                    .spawn((menu_button(76.0), StartGameButton(intent)))
                    .with_children(|button| {
                        button.spawn(button_label(label));
                    });
                parent.spawn((
                    Text::new(description),
                    TextFont {
                        font_size: FontSize::Px(16.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.75, 0.67, 0.58)),
                ));
            }
        });
}

fn navigate_menu(
    buttons: Query<(&Interaction, &MenuNavigationButton), Changed<Interaction>>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    for (interaction, destination) in &buttons {
        if *interaction == Interaction::Pressed {
            next_screen.set(destination.0);
        }
    }
}

fn quit_button(
    buttons: Query<&Interaction, (Changed<Interaction>, With<QuitButton>)>,
    mut exit: MessageWriter<AppExit>,
) {
    if buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        exit.write(AppExit::Success);
    }
}

fn start_game_button(
    buttons: Query<(&Interaction, &StartGameButton), Changed<Interaction>>,
    mut intents: MessageWriter<PlayerIntent>,
) {
    for (interaction, button) in &buttons {
        if *interaction == Interaction::Pressed {
            intents.write(button.0.clone());
        }
    }
}

fn project_match_phase(
    game: Option<Res<ActiveMatch>>,
    screen: Res<State<Screen>>,
    round_screen: Option<Res<State<RoundScreen>>>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_round_screen: Option<ResMut<NextState<RoundScreen>>>,
) {
    let Some(game) = game else {
        if *screen.get() == Screen::Match {
            next_screen.set(Screen::Menu);
        }
        return;
    };

    if *screen.get() != Screen::Match {
        next_screen.set(Screen::Match);
        return;
    }

    let projected = projected_round_screen(game.0.phase());
    if round_screen.as_deref().map(State::get) != Some(&projected)
        && let Some(next_round_screen) = next_round_screen.as_mut()
    {
        next_round_screen.set(projected);
    }
}

fn projected_round_screen(phase: MatchPhase) -> RoundScreen {
    match phase {
        MatchPhase::Ready | MatchPhase::Rolling => RoundScreen::Rolling,
        MatchPhase::Playing => RoundScreen::Playing,
        MatchPhase::Review | MatchPhase::Complete => RoundScreen::Review,
    }
}

#[cfg(test)]
mod tests {
    use word_grid_game_core::MatchPhase;

    use super::{RoundScreen, Screen, projected_round_screen};

    #[test]
    fn initial_flow_loads_before_entering_a_match() {
        assert_eq!(Screen::default(), Screen::Loading);
        assert_eq!(RoundScreen::default(), RoundScreen::Rolling);
    }

    #[test]
    fn round_screen_is_only_a_projection_of_the_domain_phase() {
        assert_eq!(
            projected_round_screen(MatchPhase::Rolling),
            RoundScreen::Rolling
        );
        assert_eq!(
            projected_round_screen(MatchPhase::Playing),
            RoundScreen::Playing
        );
        assert_eq!(
            projected_round_screen(MatchPhase::Review),
            RoundScreen::Review
        );
        assert_eq!(
            projected_round_screen(MatchPhase::Complete),
            RoundScreen::Review
        );
    }
}
