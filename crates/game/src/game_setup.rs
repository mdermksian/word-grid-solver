// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use word_grid_game_core::{GameRules, PlayMode};

use crate::content::ContentCatalog;
use crate::flow::{MenuNavigationButton, Screen};
use crate::match_plugin::{GameSet, PlayerIntent};

const PANEL: Color = Color::srgb(0.18, 0.09, 0.045);
const BUTTON: Color = Color::srgb(0.28, 0.18, 0.11);
const SELECTED: Color = Color::srgb(0.73, 0.28, 0.08);
const GOLD: Color = Color::srgb(1.0, 0.86, 0.55);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Preset {
    #[default]
    Standard,
    Big,
}

impl Preset {
    fn rules(self, catalog: &ContentCatalog) -> GameRules {
        match self {
            Self::Standard => catalog.rules.clone(),
            Self::Big => GameRules::big(),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Standard => "Standard game",
            Self::Big => "Big game",
        }
    }

    fn intent(self) -> PlayerIntent {
        match self {
            Self::Standard => PlayerIntent::StartNormal,
            Self::Big => PlayerIntent::StartBig,
        }
    }
}

#[derive(Resource, Default)]
struct SelectedPreset(Preset);

#[derive(Component)]
struct PresetButton(Preset);

#[derive(Component)]
struct StartGameButton;

#[derive(Component)]
struct RulesContent;

pub(crate) struct GameSetupPlugin;

impl Plugin for GameSetupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedPreset>()
            .add_systems(OnEnter(Screen::GameSetup), setup_screen)
            .add_systems(
                Update,
                (select_preset, start_game)
                    .chain()
                    .in_set(GameSet::Input)
                    .run_if(in_state(Screen::GameSetup)),
            )
            .add_systems(
                Update,
                (refresh_rules, scroll_rules)
                    .chain()
                    .in_set(GameSet::Presentation)
                    .run_if(in_state(Screen::GameSetup)),
            );
    }
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
        width: Val::Percent(100.0),
        min_height: Val::Px(60.0),
        flex_shrink: 0.0,
        padding: UiRect::all(Val::Px(12.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        border_radius: BorderRadius::all(Val::Px(10.0)),
        ..default()
    }
}

fn setup_screen(mut commands: Commands, mut selection: ResMut<SelectedPreset>) {
    selection.0 = Preset::Standard;
    commands.spawn((Camera2d, DespawnOnExit(Screen::GameSetup)));
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(24.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(24.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.055, 0.025)),
            DespawnOnExit(Screen::GameSetup),
        ))
        .with_children(|root| {
            root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Px(80.0),
                flex_shrink: 0.0,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|header| {
                header.spawn(text("Game Setup", 48.0, GOLD));
            });
            root.spawn((
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
                BackgroundColor(BUTTON),
            ))
            .with_children(|back| {
                back.spawn(text("BACK TO MAIN MENU", 18.0, Color::WHITE));
            });
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                min_height: Val::Px(0.0),
                column_gap: Val::Px(24.0),
                ..default()
            })
            .with_children(|body| {
                body.spawn((
                    Node {
                        width: Val::Percent(25.0),
                        flex_shrink: 0.0,
                        padding: UiRect::all(Val::Px(20.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                        border_radius: BorderRadius::all(Val::Px(16.0)),
                        ..default()
                    },
                    BackgroundColor(PANEL),
                ))
                .with_children(|sidebar| {
                    sidebar.spawn(text("Game presets", 22.0, GOLD));
                    for preset in [Preset::Standard, Preset::Big] {
                        sidebar
                            .spawn((
                                Button,
                                PresetButton(preset),
                                button_node(),
                                BackgroundColor(BUTTON),
                            ))
                            .with_children(|button| {
                                button.spawn(text(preset.label(), 22.0, Color::WHITE));
                            });
                    }
                    sidebar
                        .spawn((
                            Button,
                            InteractionDisabled,
                            button_node(),
                            BackgroundColor(Color::srgb(0.22, 0.16, 0.12)),
                        ))
                        .with_children(|button| {
                            button.spawn(text("Custom game", 22.0, Color::srgb(0.5, 0.45, 0.4)));
                        });
                    sidebar.spawn(text(
                        "Custom games coming soon",
                        16.0,
                        Color::srgb(0.7, 0.6, 0.5),
                    ));
                    sidebar
                        .spawn((
                            Button,
                            StartGameButton,
                            Node {
                                margin: UiRect::top(Val::Auto),
                                ..button_node()
                            },
                            BackgroundColor(SELECTED),
                        ))
                        .with_children(|button| {
                            button.spawn(text("Start game", 24.0, Color::WHITE));
                        });
                });
                body.spawn((
                    Node {
                        flex_grow: 1.0,
                        flex_basis: Val::Px(0.0),
                        min_width: Val::Px(0.0),
                        min_height: Val::Px(0.0),
                        padding: UiRect::all(Val::Px(24.0)),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(16.0),
                        overflow: Overflow::scroll_y(),
                        border_radius: BorderRadius::all(Val::Px(16.0)),
                        ..default()
                    },
                    BackgroundColor(PANEL),
                    ScrollPosition::default(),
                    RulesContent,
                ));
            });
        });
}

fn select_preset(
    buttons: Query<(&Interaction, &PresetButton), Changed<Interaction>>,
    mut selection: ResMut<SelectedPreset>,
) {
    for (interaction, preset) in &buttons {
        if *interaction == Interaction::Pressed && selection.0 != preset.0 {
            selection.0 = preset.0;
        }
    }
}

fn start_game(
    buttons: Query<&Interaction, (Changed<Interaction>, With<StartGameButton>)>,
    selection: Res<SelectedPreset>,
    mut intents: MessageWriter<PlayerIntent>,
) {
    if buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        intents.write(selection.0.intent());
    }
}

fn refresh_rules(
    selection: Res<SelectedPreset>,
    catalog: Res<ContentCatalog>,
    mut commands: Commands,
    mut panels: Query<(Entity, &mut ScrollPosition), With<RulesContent>>,
    mut buttons: Query<(&PresetButton, &mut BackgroundColor)>,
) {
    if !selection.is_changed() && !catalog.is_changed() {
        return;
    }
    for (preset, mut background) in &mut buttons {
        background.0 = if preset.0 == selection.0 {
            SELECTED
        } else {
            BUTTON
        };
    }
    let Ok((panel, mut scroll)) = panels.single_mut() else {
        return;
    };
    scroll.0 = Vec2::ZERO;
    let rules = selection.0.rules(&catalog);
    commands.entity(panel).despawn_related::<Children>();
    commands.entity(panel).with_children(|content| {
        content.spawn(text(selection.0.label(), 32.0, GOLD));
        rule_row(
            content,
            "Grid size (height x width)",
            format!("{} x {}", rules.grid_size(), rules.grid_size()),
        );
        rule_row(
            content,
            "Minimum word length",
            format!("{} letters", rules.minimum_word_length()),
        );
        rule_row(
            content,
            "Timing configuration",
            timing_label(rules.play_mode()),
        );

        content.spawn(text("Scoring table", 24.0, GOLD));
        rule_row(content, "Word length", "Points");
        for (index, band) in rules.scoring().bands().iter().enumerate() {
            let length = match rules.scoring().bands().get(index + 1) {
                Some(next) if next.minimum_length == band.minimum_length + 1 => {
                    band.minimum_length.to_string()
                }
                Some(next) => format!("{}-{}", band.minimum_length, next.minimum_length - 1),
                None => format!("{}+", band.minimum_length),
            };
            rule_row(
                content,
                &format!("{length} letters"),
                band.points.to_string(),
            );
        }
        content.spawn(text(
            format!(
                "Only words with at least {} letters are accepted.",
                rules.minimum_word_length()
            ),
            16.0,
            Color::srgb(0.8, 0.72, 0.62),
        ));

        content.spawn(text("Configured cubes", 24.0, GOLD));
        rule_row(content, "Cube set", rules.cube_set().name());
        rule_row(
            content,
            "Dice count",
            rules.cube_set().cubes().len().to_string(),
        );
        rule_row(content, "Die", "Faces");
        for (index, cube) in rules.cube_set().cubes().iter().enumerate() {
            rule_row(content, &format!("{}", index + 1), cube.faces().join(" / "));
        }
    });
}

fn rule_row(parent: &mut ChildSpawnerCommands, label: &str, value: impl Into<String>) {
    parent
        .spawn(Node {
            width: Val::Percent(100.0),
            flex_shrink: 0.0,
            column_gap: Val::Px(20.0),
            padding: UiRect::vertical(Val::Px(4.0)),
            ..default()
        })
        .with_children(|row| {
            row.spawn(Node {
                width: Val::Percent(45.0),
                flex_shrink: 0.0,
                ..default()
            })
            .with_children(|cell| {
                cell.spawn(text(label, 20.0, Color::srgb(0.8, 0.72, 0.62)));
            });
            row.spawn(Node {
                flex_grow: 1.0,
                flex_basis: Val::Px(0.0),
                min_width: Val::Px(0.0),
                ..default()
            })
            .with_children(|cell| {
                cell.spawn(text(value, 20.0, Color::WHITE));
            });
        });
}

fn timing_label(mode: PlayMode) -> String {
    match mode {
        PlayMode::Endless => "Endless (no time limit)".into(),
        PlayMode::TimedRounds { duration } => format!(
            "Timed rounds - {} seconds per round",
            duration.as_secs_f64()
        ),
    }
}

fn scroll_rules(
    mut wheel: MessageReader<MouseWheel>,
    mut panels: Query<(&mut ScrollPosition, &ComputedNode), With<RulesContent>>,
) {
    let delta = wheel
        .read()
        .map(|event| {
            -event.y
                * if event.unit == MouseScrollUnit::Line {
                    32.0
                } else {
                    1.0
                }
        })
        .sum::<f32>();
    for (mut scroll, computed) in &mut panels {
        let max_offset = (computed.content_size().y - computed.size().y).max(0.0)
            * computed.inverse_scale_factor();
        scroll.y = (scroll.y + delta).clamp(0.0, max_offset);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::state::app::StatesPlugin;
    use std::time::Duration;
    use word_grid_game_core::{Cube, CubeSet, ScoreBand, ScoringTable};

    #[test]
    fn preview_reads_nonstandard_cubes_scoring_and_timing_from_rules() {
        let rules = GameRules::new(
            2,
            3,
            PlayMode::Endless,
            CubeSet::new(
                "Test cubes",
                ["A", "B", "C", "D"].map(|face| Cube::new([face, "Qu"]).unwrap()),
            )
            .unwrap(),
            ScoringTable::new([
                ScoreBand {
                    minimum_length: 3,
                    points: 7,
                },
                ScoreBand {
                    minimum_length: 6,
                    points: 12,
                },
            ])
            .unwrap(),
        )
        .unwrap();
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .insert_state(Screen::GameSetup)
            .insert_resource(ContentCatalog {
                rules,
                dictionary: Handle::default(),
                cube_scene: Handle::default(),
            })
            .add_message::<PlayerIntent>()
            .add_message::<MouseWheel>()
            .add_plugins(GameSetupPlugin);
        app.update();

        let rendered = app
            .world_mut()
            .query::<&Text>()
            .iter(app.world())
            .map(|text| text.0.as_str())
            .collect::<Vec<_>>();
        for expected in [
            "2 x 2",
            "3 letters",
            "Endless (no time limit)",
            "Test cubes",
            "4",
            "A / Qu",
            "D / Qu",
            "3-5 letters",
            "7",
            "6+ letters",
            "12",
        ] {
            assert!(
                rendered.contains(&expected),
                "missing rule detail: {expected}"
            );
        }
        assert_eq!(
            timing_label(PlayMode::TimedRounds {
                duration: Duration::from_millis(1500)
            }),
            "Timed rounds - 1.5 seconds per round"
        );
    }
}
