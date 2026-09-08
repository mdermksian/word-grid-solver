// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use bevy::prelude::*;
use word_grid_game_core::MatchPhase;

use crate::content::ContentCatalog;
use crate::match_plugin::{ActiveMatch, GameSet, PlayerIntent};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
#[states(scoped_entities)]
pub enum Screen {
    #[default]
    Loading,
    Menu,
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
struct NormalGameButton;

#[derive(Component)]
struct LoadingStatus;

pub struct FlowPlugin;

impl Plugin for FlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<Screen>()
            .add_sub_state::<RoundScreen>()
            .add_systems(OnEnter(Screen::Loading), setup_loading_screen)
            .add_systems(OnEnter(Screen::Menu), setup_menu_screen)
            .add_systems(
                Update,
                (
                    wait_for_content.run_if(in_state(Screen::Loading)),
                    start_normal_button
                        .in_set(GameSet::Input)
                        .run_if(in_state(Screen::Menu)),
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

fn setup_menu_screen(mut commands: Commands) {
    commands.spawn((Camera2d, DespawnOnExit(Screen::Menu)));
    commands
        .spawn((
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
            DespawnOnExit(Screen::Menu),
        ))
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
                .spawn((
                    Button,
                    NormalGameButton,
                    Node {
                        width: Val::Px(330.0),
                        height: Val::Px(76.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border_radius: BorderRadius::all(Val::Px(12.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.73, 0.28, 0.08)),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("START NORMAL - 3 MINUTES"),
                        TextFont {
                            font_size: FontSize::Px(22.0),
                            ..default()
                        },
                        TextColor::WHITE,
                    ));
                });
            parent.spawn((
                Text::new("4 x 4 - Standard New cubes - 3 letter minimum"),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::srgb(0.75, 0.67, 0.58)),
            ));
        });
}

fn start_normal_button(
    buttons: Query<&Interaction, (Changed<Interaction>, With<NormalGameButton>)>,
    mut intents: MessageWriter<PlayerIntent>,
) {
    if buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        intents.write(PlayerIntent::StartNormal);
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
