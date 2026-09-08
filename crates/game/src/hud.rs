// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use std::time::Duration;

use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use word_grid_game_core::MatchPhase;

use crate::flow::{RoundScreen, Screen};
use crate::match_plugin::{ActiveMatch, GameSet, LOCAL_PLAYER, MatchNotice, PlayerIntent};

#[derive(Component)]
struct HudInput;

#[derive(Component)]
struct HudFeedback;

#[derive(Component)]
struct RoundTotal;

#[derive(Component)]
struct RoundTimer;

#[derive(Component)]
struct WordList;

#[derive(Component)]
struct ReviewTitle;

#[derive(Component)]
struct ReviewSummary;

#[derive(Component)]
struct NextRoundButton;

#[derive(Component)]
struct FinishMatchButton;

#[derive(Component)]
struct ReturnToMenuButton;

#[derive(Resource, Default)]
pub(crate) struct InputDraft {
    pub path: Vec<usize>,
    pub typed: String,
    pub feedback: String,
}

#[derive(Resource, Default, PartialEq, Eq)]
struct RenderedWordList {
    completed_rounds: usize,
    submissions: usize,
    active: bool,
}

type HudTextQueries<'w, 's> = (
    Query<'w, 's, &'static mut Text, With<HudInput>>,
    Query<'w, 's, &'static mut Text, With<HudFeedback>>,
    Query<'w, 's, &'static mut Text, With<RoundTotal>>,
    Query<'w, 's, &'static mut Text, With<RoundTimer>>,
);

type ReviewQueries<'w, 's> = (
    Query<'w, 's, &'static mut Text, With<ReviewTitle>>,
    Query<'w, 's, &'static mut Text, With<ReviewSummary>>,
    Query<'w, 's, &'static mut Node, With<NextRoundButton>>,
    Query<'w, 's, &'static mut Node, With<FinishMatchButton>>,
    Query<'w, 's, &'static mut Node, With<ReturnToMenuButton>>,
);

type ReviewButtonQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Interaction,
        Has<NextRoundButton>,
        Has<FinishMatchButton>,
        Has<ReturnToMenuButton>,
    ),
    (Changed<Interaction>, With<Button>),
>;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InputDraft {
            feedback: "Click adjacent dice; type a word; Enter submits.".into(),
            ..default()
        })
        .init_resource::<RenderedWordList>()
        .add_systems(OnEnter(Screen::Match), setup_hud)
        .add_systems(OnEnter(RoundScreen::Review), setup_review)
        .add_systems(
            Update,
            keyboard_input
                .in_set(GameSet::Input)
                .run_if(in_state(RoundScreen::Playing)),
        )
        .add_systems(
            Update,
            review_buttons
                .in_set(GameSet::Input)
                .run_if(in_state(RoundScreen::Review)),
        )
        .add_systems(
            Update,
            (
                scroll_found_words,
                (consume_notices, refresh_hud, refresh_review).chain(),
            )
                .in_set(GameSet::Presentation)
                .run_if(in_state(Screen::Match))
                .run_if(resource_exists::<ActiveMatch>),
        );
    }
}

fn setup_review(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                width: Val::Px(440.0),
                padding: UiRect::all(Val::Px(28.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(18.0),
                border_radius: BorderRadius::all(Val::Px(16.0)),
                ..default()
            },
            UiTransform::from_translation(Val2::px(-130.0, -170.0)),
            BackgroundColor(Color::srgba(0.08, 0.06, 0.04, 0.96)),
            DespawnOnExit(RoundScreen::Review),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("ROUND COMPLETE"),
                TextFont {
                    font_size: FontSize::Px(32.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.82, 0.35)),
                ReviewTitle,
            ));
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(21.0),
                    ..default()
                },
                TextColor::WHITE,
                ReviewSummary,
            ));
            parent
                .spawn((
                    Button,
                    NextRoundButton,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(54.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.73, 0.28, 0.08)),
                ))
                .with_children(|button| {
                    button.spawn((Text::new("PLAY ANOTHER ROUND"), TextColor::WHITE));
                });
            parent
                .spawn((
                    Button,
                    FinishMatchButton,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(54.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.28, 0.18, 0.11)),
                ))
                .with_children(|button| {
                    button.spawn((Text::new("FINISH MATCH"), TextColor::WHITE));
                });
            parent
                .spawn((
                    Button,
                    ReturnToMenuButton,
                    Node {
                        width: Val::Px(300.0),
                        height: Val::Px(54.0),
                        display: Display::None,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.73, 0.28, 0.08)),
                ))
                .with_children(|button| {
                    button.spawn((Text::new("RETURN TO MENU"), TextColor::WHITE));
                });
        });
}

fn review_buttons(interactions: ReviewButtonQuery, mut intents: MessageWriter<PlayerIntent>) {
    for (interaction, next_round, finish_match, return_to_menu) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if next_round {
            intents.write(PlayerIntent::StartNextRound);
        } else if finish_match {
            intents.write(PlayerIntent::FinishMatch);
        } else if return_to_menu {
            intents.write(PlayerIntent::LeaveMatch);
        }
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
            parent.spawn((
                Text::new("03:00"),
                TextFont {
                    font_size: FontSize::Px(36.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.82, 0.35)),
                RoundTimer,
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
        match notice {
            MatchNotice::RoundStarted => {
                input.path.clear();
                input.typed.clear();
                input.feedback = "Rolling the dice…".into();
            }
            MatchNotice::RoundBegan => {
                input.feedback = "Go! Click adjacent dice or type a word.".into();
            }
            MatchNotice::SubmissionAccepted(submission) => {
                input.feedback = format!(
                    "Accepted {} (+{})",
                    submission.word().to_uppercase(),
                    submission.base_score()
                );
            }
            MatchNotice::SubmissionRejected(error) => input.feedback.clone_from(error),
            MatchNotice::RoundFinished(_) => {
                input.path.clear();
                input.typed.clear();
                input.feedback = "Time! Review your round.".into();
            }
            MatchNotice::MatchFinished => {
                input.feedback = "Match complete.".into();
            }
            MatchNotice::MatchExited => {
                input.path.clear();
                input.typed.clear();
            }
        }
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
    mut rendered: ResMut<RenderedWordList>,
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
        let score = if game.0.current_round().is_some() {
            game.0.current_score(LOCAL_PLAYER).unwrap_or_default()
        } else {
            game.0
                .completed_rounds()
                .last()
                .and_then(|round| round.player(LOCAL_PLAYER))
                .map_or(0, |player| player.score())
        };
        text.0 = format!("ROUND TOTAL: {score}");
    }
    if let Ok(mut text) = texts.p3().single_mut() {
        text.0 = game.0.current_round().map_or_else(
            || "00:00".into(),
            |round| format_duration(round.remaining()),
        );
    }

    let mut words = if let Some(round) = game.0.current_round() {
        round
            .submissions(LOCAL_PLAYER)
            .iter()
            .cloned()
            .map(|submission| (submission, false))
            .collect::<Vec<_>>()
    } else {
        game.0
            .completed_rounds()
            .last()
            .and_then(|round| round.player(LOCAL_PLAYER))
            .map(|result| {
                result
                    .scored()
                    .iter()
                    .cloned()
                    .map(|submission| (submission, false))
                    .chain(
                        result
                            .canceled()
                            .iter()
                            .cloned()
                            .map(|submission| (submission, true)),
                    )
                    .collect()
            })
            .unwrap_or_default()
    };
    let version = RenderedWordList {
        completed_rounds: game.0.completed_rounds().len(),
        submissions: words.len(),
        active: game.0.current_round().is_some(),
    };
    if *rendered == version {
        return;
    }
    *rendered = version;
    let Ok(list) = word_list.single() else {
        return;
    };
    commands.entity(list).despawn_related::<Children>();
    words.sort_by(|(left, _), (right, _)| {
        right
            .word()
            .chars()
            .count()
            .cmp(&left.word().chars().count())
            .then_with(|| left.word().cmp(right.word()))
    });
    commands.entity(list).with_children(|parent| {
        for (found, canceled) in words {
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new(if canceled {
                            format!("{} (DUPLICATE)", found.word().to_uppercase())
                        } else {
                            found.word().to_uppercase()
                        }),
                        TextFont {
                            font_size: FontSize::Px(18.0),
                            ..default()
                        },
                        TextColor(if canceled {
                            Color::srgb(0.65, 0.6, 0.55)
                        } else {
                            Color::WHITE
                        }),
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

fn refresh_review(game: Res<ActiveMatch>, mut queries: ParamSet<ReviewQueries>) {
    let complete = game.0.phase() == MatchPhase::Complete;
    {
        let mut titles = queries.p0();
        let Ok(mut title) = titles.single_mut() else {
            return;
        };
        title.0 = if complete {
            "MATCH COMPLETE".into()
        } else {
            format!("ROUND {} COMPLETE", game.0.completed_rounds().len())
        };
    }

    let round_score = game
        .0
        .completed_rounds()
        .last()
        .and_then(|round| round.player(LOCAL_PLAYER))
        .map_or(0, |player| player.score());
    let word_count = game
        .0
        .completed_rounds()
        .last()
        .and_then(|round| round.player(LOCAL_PLAYER))
        .map_or(0, |player| player.scored().len());
    let total_score = game.0.total_score(LOCAL_PLAYER).unwrap_or_default();
    if let Ok(mut summary) = queries.p1().single_mut() {
        summary.0 = if complete {
            format!(
                "{} rounds played\nFINAL SCORE: {total_score}",
                game.0.completed_rounds().len()
            )
        } else {
            format!(
                "Words found: {word_count}\nRound score: {round_score}\nMatch total: {total_score}"
            )
        };
    }
    if let Ok(mut node) = queries.p2().single_mut() {
        node.display = if complete {
            Display::None
        } else {
            Display::Flex
        };
    }
    if let Ok(mut node) = queries.p3().single_mut() {
        node.display = if complete {
            Display::None
        } else {
            Display::Flex
        };
    }
    if let Ok(mut node) = queries.p4().single_mut() {
        node.display = if complete {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn format_duration(duration: Option<Duration>) -> String {
    let Some(duration) = duration else {
        return "∞".into();
    };
    let seconds = duration.as_secs_f32().ceil() as u64;
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{InputDraft, delete_last_input, format_duration};

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

    #[test]
    fn formats_round_time_by_rounding_partial_seconds_up() {
        assert_eq!(
            format_duration(Some(Duration::from_millis(179_001))),
            "03:00"
        );
        assert_eq!(format_duration(Some(Duration::from_secs(9))), "00:09");
        assert_eq!(format_duration(None), "∞");
    }
}
