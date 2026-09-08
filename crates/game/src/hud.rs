// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

use crate::StartupSet;
use crate::flow::{RoundScreen, Screen};
use crate::match_plugin::{ActiveMatch, GameSet, LOCAL_PLAYER, MatchNotice, PlayerIntent};

#[derive(Component)]
struct HudInput;

#[derive(Component)]
struct HudFeedback;

#[derive(Component)]
struct RoundTotal;

#[derive(Component)]
struct WordList;

#[derive(Resource, Default)]
pub(crate) struct InputDraft {
    pub path: Vec<usize>,
    pub typed: String,
    pub feedback: String,
}

type HudTextQueries<'w, 's> = (
    Query<'w, 's, &'static mut Text, With<HudInput>>,
    Query<'w, 's, &'static mut Text, With<HudFeedback>>,
    Query<'w, 's, &'static mut Text, With<RoundTotal>>,
);

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InputDraft {
            feedback: "Click adjacent dice; type a word; Enter submits.".into(),
            ..default()
        })
        .add_systems(Startup, setup_hud.in_set(StartupSet::Presentation))
        .add_systems(
            Update,
            keyboard_input
                .in_set(GameSet::Input)
                .run_if(in_state(RoundScreen::Playing)),
        )
        .add_systems(
            Update,
            (scroll_found_words, (consume_notices, refresh_hud).chain())
                .in_set(GameSet::Presentation)
                .run_if(in_state(Screen::Match)),
        );
    }
}

fn setup_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                top: Val::Px(16.0),
                width: Val::Px(300.0),
                height: Val::Px(560.0),
                padding: UiRect::all(Val::Px(16.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                border_radius: BorderRadius::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.06, 0.04, 0.88)),
            DespawnOnExit(Screen::Match),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("WORD GRID"),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor::WHITE,
            ));
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        min_height: Val::Px(42.0),
                        padding: UiRect::horizontal(Val::Px(12.0)),
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(Color::WHITE),
                ))
                .with_children(|input| {
                    input.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: FontSize::Px(22.0),
                            ..default()
                        },
                        TextColor::BLACK,
                        HudInput,
                    ));
                });
            parent.spawn((
                Text::new("Click dice or type a word, then press Enter."),
                TextFont {
                    font_size: FontSize::Px(15.0),
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.7, 0.5)),
                HudFeedback,
            ));
            parent.spawn((
                Text::new("FOUND WORDS"),
                TextFont {
                    font_size: FontSize::Px(17.0),
                    ..default()
                },
                TextColor::WHITE,
            ));
            parent.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                ScrollPosition::default(),
                WordList,
            ));
            parent.spawn((
                Text::new("ROUND TOTAL: 0"),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor::WHITE,
                RoundTotal,
            ));
        });
}

fn keyboard_input(
    mut events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut input: ResMut<InputDraft>,
    mut intents: MessageWriter<PlayerIntent>,
) {
    for event in events.read() {
        if !event.state.is_pressed() {
            continue;
        }
        if event.key_code == KeyCode::Backspace {
            delete_last_input(&mut input);
        } else if event.key_code == KeyCode::Enter {
            continue;
        } else if let Some(text) = &event.text {
            input.typed.push_str(text);
            input.path.clear();
        }
    }
    if keys.just_pressed(KeyCode::Escape) {
        input.path.clear();
        input.typed.clear();
    }
    if keys.just_pressed(KeyCode::Enter) {
        if input.typed.is_empty() {
            intents.write(PlayerIntent::SubmitPath(input.path.clone()));
        } else {
            intents.write(PlayerIntent::SubmitText(input.typed.clone()));
        }
        input.path.clear();
        input.typed.clear();
    }
}

fn delete_last_input(input: &mut InputDraft) {
    if input.typed.is_empty() {
        input.path.pop();
    } else {
        input.typed.pop();
    }
}

fn consume_notices(mut notices: MessageReader<MatchNotice>, mut input: ResMut<InputDraft>) {
    for notice in notices.read() {
        input.feedback = match notice {
            MatchNotice::SubmissionAccepted(submission) => format!(
                "Accepted {} (+{})",
                submission.word().to_uppercase(),
                submission.base_score()
            ),
            MatchNotice::SubmissionRejected(error) => error.clone(),
        };
    }
}

fn scroll_found_words(
    mut wheel_events: MessageReader<MouseWheel>,
    mut lists: Query<(&mut ScrollPosition, &ComputedNode), With<WordList>>,
) {
    let mut delta = 0.0;
    for event in wheel_events.read() {
        delta -= event.y
            * if event.unit == MouseScrollUnit::Line {
                24.0
            } else {
                1.0
            };
    }
    if delta == 0.0 {
        return;
    }

    for (mut position, computed) in &mut lists {
        let max_offset = (computed.content_size().y - computed.size().y).max(0.0)
            * computed.inverse_scale_factor();
        position.y = (position.y + delta).clamp(0.0, max_offset);
    }
}

fn refresh_hud(
    game: Res<ActiveMatch>,
    input: Res<InputDraft>,
    mut commands: Commands,
    mut texts: ParamSet<HudTextQueries>,
    word_list: Query<Entity, With<WordList>>,
) {
    let word = if input.typed.is_empty() {
        game.0
            .current_round()
            .and_then(|round| round.board().grid().word_for_path(&input.path).ok())
            .unwrap_or_default()
    } else {
        input.typed.clone()
    };
    if let Ok(mut text) = texts.p0().single_mut() {
        text.0 = word.to_uppercase();
    }
    if let Ok(mut text) = texts.p1().single_mut() {
        text.0 = input.feedback.clone();
    }
    if let Ok(mut text) = texts.p2().single_mut() {
        text.0 = format!(
            "ROUND TOTAL: {}",
            game.0.current_score(LOCAL_PLAYER).unwrap_or_default()
        );
    }

    if !game.is_changed() {
        return;
    }
    let Ok(list) = word_list.single() else {
        return;
    };
    commands.entity(list).despawn_related::<Children>();
    let mut words = game
        .0
        .current_round()
        .map(|round| round.submissions(LOCAL_PLAYER).to_vec())
        .unwrap_or_default();
    words.sort_by(|left, right| {
        right
            .word()
            .chars()
            .count()
            .cmp(&left.word().chars().count())
            .then_with(|| left.word().cmp(right.word()))
    });
    commands.entity(list).with_children(|parent| {
        for found in words {
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new(found.word().to_uppercase()),
                        TextFont {
                            font_size: FontSize::Px(18.0),
                            ..default()
                        },
                        TextColor::WHITE,
                    ));
                    row.spawn((
                        Text::new(found.base_score().to_string()),
                        TextFont {
                            font_size: FontSize::Px(18.0),
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.82, 0.35)),
                    ));
                });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{InputDraft, delete_last_input};

    #[test]
    fn backspace_removes_typed_text_before_a_selected_path() {
        let mut input = InputDraft {
            path: vec![1, 2],
            typed: "cat".into(),
            ..Default::default()
        };

        delete_last_input(&mut input);
        assert_eq!(input.typed, "ca");
        assert_eq!(input.path, vec![1, 2]);

        input.typed.clear();
        delete_last_input(&mut input);
        assert_eq!(input.path, vec![1]);
    }
}
