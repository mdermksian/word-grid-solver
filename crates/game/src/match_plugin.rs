// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use bevy::prelude::*;
use rand::SeedableRng;
use word_grid_game_core::{Match, MatchPhase, PlayerId, RoundResult, Submission};

use crate::content::{ContentCatalog, DictionaryAsset};
use crate::flow::RoundScreen;

pub(crate) const LOCAL_PLAYER: PlayerId = PlayerId::new(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum GameSet {
    Input,
    Domain,
    Presentation,
}

#[derive(Debug, Clone, Message)]
pub enum PlayerIntent {
    StartNormal,
    RollComplete,
    SubmitPath(Vec<usize>),
    SubmitText(String),
    EndRound,
    StartNextRound,
    FinishMatch,
    LeaveMatch,
}

#[derive(Debug, Clone, Message)]
pub enum MatchNotice {
    RoundStarted,
    RoundBegan,
    SubmissionAccepted(Submission),
    SubmissionRejected(String),
    RoundFinished(RoundResult),
    MatchFinished,
    MatchExited,
}

#[derive(Resource, Debug)]
pub(crate) struct ActiveMatch(pub Match);

#[derive(Resource)]
struct GameRng(rand::rngs::SmallRng);

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, app: &mut App) {
        let mut platform_rng = rand::rng();
        app.insert_resource(GameRng(rand::rngs::SmallRng::from_rng(&mut platform_rng)))
            .add_message::<PlayerIntent>()
            .add_message::<MatchNotice>()
            .add_systems(
                Update,
                (
                    apply_player_intents,
                    advance_round_clock.run_if(in_state(RoundScreen::Playing)),
                )
                    .chain()
                    .in_set(GameSet::Domain),
            );
    }
}

fn apply_player_intents(
    mut intents: MessageReader<PlayerIntent>,
    mut notices: MessageWriter<MatchNotice>,
    mut game: Option<ResMut<ActiveMatch>>,
    mut rng: ResMut<GameRng>,
    mut commands: Commands,
    catalog: Res<ContentCatalog>,
    dictionaries: Res<Assets<DictionaryAsset>>,
) {
    let mut match_exists = game.is_some();
    for intent in intents.read() {
        match intent {
            PlayerIntent::StartNormal => {
                if match_exists {
                    continue;
                }
                let mut new_match = Match::single_player(catalog.rules.clone(), "Player")
                    .expect("the built-in player is valid");
                new_match
                    .start_round(&mut rng.0)
                    .expect("a new match can start its first round");
                commands.insert_resource(ActiveMatch(new_match));
                match_exists = true;
                notices.write(MatchNotice::RoundStarted);
            }
            PlayerIntent::RollComplete => {
                let Some(game) = game.as_mut() else {
                    continue;
                };
                if let Err(error) = game.0.begin_round() {
                    notices.write(MatchNotice::SubmissionRejected(error.to_string()));
                } else {
                    notices.write(MatchNotice::RoundBegan);
                }
            }
            PlayerIntent::SubmitPath(path) => {
                let Some(game) = game.as_mut() else {
                    continue;
                };
                let Some(dictionary) = dictionaries.get(&catalog.dictionary) else {
                    notices.write(MatchNotice::SubmissionRejected(
                        "Dictionary is still loading.".into(),
                    ));
                    continue;
                };
                publish_submission(
                    game.0
                        .submit_path(LOCAL_PLAYER, path.clone(), &dictionary.0),
                    &mut notices,
                );
            }
            PlayerIntent::SubmitText(word) => {
                let Some(game) = game.as_mut() else {
                    continue;
                };
                let Some(dictionary) = dictionaries.get(&catalog.dictionary) else {
                    notices.write(MatchNotice::SubmissionRejected(
                        "Dictionary is still loading.".into(),
                    ));
                    continue;
                };
                publish_submission(
                    game.0.submit_text(LOCAL_PLAYER, word, &dictionary.0),
                    &mut notices,
                );
            }
            PlayerIntent::EndRound => {
                let Some(game) = game.as_mut() else {
                    continue;
                };
                match game.0.finish_round() {
                    Ok(result) => {
                        notices.write(MatchNotice::RoundFinished(result));
                    }
                    Err(error) => {
                        notices.write(MatchNotice::SubmissionRejected(error.to_string()));
                    }
                }
            }
            PlayerIntent::StartNextRound => {
                let Some(game) = game.as_mut() else {
                    continue;
                };
                if let Err(error) = game.0.start_round(&mut rng.0) {
                    notices.write(MatchNotice::SubmissionRejected(error.to_string()));
                } else {
                    notices.write(MatchNotice::RoundStarted);
                }
            }
            PlayerIntent::FinishMatch => {
                let Some(game) = game.as_mut() else {
                    continue;
                };
                if let Err(error) = game.0.finish_match() {
                    notices.write(MatchNotice::SubmissionRejected(error.to_string()));
                } else {
                    notices.write(MatchNotice::MatchFinished);
                }
            }
            PlayerIntent::LeaveMatch => {
                if match_exists {
                    commands.remove_resource::<ActiveMatch>();
                    match_exists = false;
                    notices.write(MatchNotice::MatchExited);
                }
            }
        }
    }
}

fn publish_submission(
    result: Result<Submission, word_grid_game_core::SubmissionError>,
    notices: &mut MessageWriter<MatchNotice>,
) {
    match result {
        Ok(submission) => {
            notices.write(MatchNotice::SubmissionAccepted(submission));
        }
        Err(error) => {
            notices.write(MatchNotice::SubmissionRejected(error.to_string()));
        }
    }
}

fn advance_round_clock(
    time: Res<Time>,
    mut game: Option<ResMut<ActiveMatch>>,
    mut notices: MessageWriter<MatchNotice>,
) {
    let Some(game) = game.as_mut() else {
        return;
    };
    if game.0.phase() != MatchPhase::Playing {
        return;
    }
    match game.0.advance_time(time.delta()) {
        Ok(Some(result)) => {
            notices.write(MatchNotice::RoundFinished(result));
        }
        Ok(None) => {}
        Err(error) => {
            notices.write(MatchNotice::SubmissionRejected(error.to_string()));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use rand::SeedableRng;
    use word_grid_game_core::{
        Cube, CubeSet, Dictionary, GameRules, Match, MatchPhase, PlayMode, ScoringTable,
    };

    use super::{
        ActiveMatch, GameRng, MatchNotice, PlayerIntent, advance_round_clock, apply_player_intents,
    };
    use crate::content::{ContentCatalog, DictionaryAsset};

    #[test]
    fn player_intent_mutates_match_and_emits_notice() {
        let cubes = ["c", "a", "t", "s"].map(|face| Cube::new([face]).unwrap());
        let rules = GameRules::new(
            2,
            3,
            PlayMode::Endless,
            CubeSet::new("test", cubes).unwrap(),
            ScoringTable::standard(),
        )
        .unwrap();
        let mut game = Match::single_player(rules.clone(), "Player").unwrap();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(8);
        game.start_round(&mut rng).unwrap();
        game.begin_round().unwrap();
        let path = vec![0, 1, 2];
        let word = game
            .current_round()
            .unwrap()
            .board()
            .grid()
            .word_for_path(&path)
            .unwrap();

        let mut dictionaries = Assets::<DictionaryAsset>::default();
        let dictionary = dictionaries.add(DictionaryAsset(Dictionary::from_words([word])));
        let mut app = App::new();
        app.add_message::<PlayerIntent>()
            .add_message::<MatchNotice>()
            .insert_resource(dictionaries)
            .insert_resource(ContentCatalog {
                rules,
                dictionary,
                cube_scene: Handle::default(),
            })
            .insert_resource(ActiveMatch(game))
            .insert_resource(GameRng(rand::rngs::SmallRng::seed_from_u64(9)))
            .add_systems(Update, apply_player_intents);

        app.world_mut()
            .write_message(PlayerIntent::SubmitPath(path))
            .unwrap();
        app.update();

        assert_eq!(
            app.world()
                .resource::<ActiveMatch>()
                .0
                .current_round()
                .unwrap()
                .submissions(super::LOCAL_PLAYER)
                .len(),
            1
        );
        let messages = app.world().resource::<Messages<MatchNotice>>();
        let mut cursor = messages.get_cursor();
        assert!(matches!(
            cursor.read(messages).next(),
            Some(MatchNotice::SubmissionAccepted(_))
        ));
    }

    #[test]
    fn bridge_supports_early_finish_and_timer_expiry_in_one_match() {
        let cubes = ["c", "a", "t", "s"].map(|face| Cube::new([face]).unwrap());
        let rules = GameRules::new(
            2,
            3,
            PlayMode::TimedRounds {
                duration: Duration::from_secs(1),
            },
            CubeSet::new("test", cubes).unwrap(),
            ScoringTable::standard(),
        )
        .unwrap();
        let letters = ["c", "a", "t", "s"];
        let words = (0..4)
            .flat_map(|first| {
                (0..4).flat_map(move |second| {
                    (0..4)
                        .filter(move |&third| first != second && first != third && second != third)
                        .map(move |third| {
                            format!("{}{}{}", letters[first], letters[second], letters[third])
                        })
                })
            })
            .collect::<Vec<_>>();
        let mut dictionaries = Assets::<DictionaryAsset>::default();
        let dictionary = dictionaries.add(DictionaryAsset(Dictionary::from_words(words)));

        let mut app = App::new();
        app.add_message::<PlayerIntent>()
            .add_message::<MatchNotice>()
            .insert_resource(Time::<()>::default())
            .insert_resource(dictionaries)
            .insert_resource(ContentCatalog {
                rules,
                dictionary,
                cube_scene: Handle::default(),
            })
            .insert_resource(GameRng(rand::rngs::SmallRng::seed_from_u64(22)))
            .add_systems(Update, (apply_player_intents, advance_round_clock).chain());

        app.world_mut()
            .write_message(PlayerIntent::StartNormal)
            .unwrap();
        app.update();
        for round_index in 0..2 {
            app.world_mut()
                .resource_mut::<Time<()>>()
                .advance_by(Duration::ZERO);
            app.world_mut()
                .write_message(PlayerIntent::RollComplete)
                .unwrap();
            app.update();
            app.world_mut()
                .write_message(PlayerIntent::SubmitPath(vec![0, 1, 2]))
                .unwrap();
            app.update();
            if round_index == 0 {
                assert_eq!(
                    app.world()
                        .resource::<ActiveMatch>()
                        .0
                        .current_round()
                        .and_then(|round| round.remaining()),
                    Some(Duration::from_secs(1))
                );
                app.world_mut()
                    .write_message(PlayerIntent::EndRound)
                    .unwrap();
                app.update();
            } else {
                app.world_mut()
                    .resource_mut::<Time<()>>()
                    .advance_by(Duration::from_secs(1));
                app.update();
            }

            assert_eq!(
                app.world().resource::<ActiveMatch>().0.phase(),
                MatchPhase::Review
            );

            if round_index == 0 {
                app.world_mut()
                    .resource_mut::<Time<()>>()
                    .advance_by(Duration::ZERO);
                app.world_mut()
                    .write_message(PlayerIntent::StartNextRound)
                    .unwrap();
                app.update();
            }
        }
        app.world_mut()
            .resource_mut::<Time<()>>()
            .advance_by(Duration::ZERO);
        app.world_mut()
            .write_message(PlayerIntent::FinishMatch)
            .unwrap();
        app.update();

        let game = &app.world().resource::<ActiveMatch>().0;
        assert_eq!(game.phase(), MatchPhase::Complete);
        assert_eq!(game.completed_rounds().len(), 2);
        assert_eq!(game.total_score(super::LOCAL_PLAYER).unwrap(), 2);
    }
}
