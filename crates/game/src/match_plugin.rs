// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use bevy::prelude::*;
use word_grid_game_core::{Match, PlayerId, Submission};

use crate::StartupSet;
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
    SubmitPath(Vec<usize>),
    SubmitText(String),
}

#[derive(Debug, Clone, Message)]
pub enum MatchNotice {
    SubmissionAccepted(Submission),
    SubmissionRejected(String),
}

#[derive(Resource, Debug)]
pub(crate) struct ActiveMatch(pub Match);

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlayerIntent>()
            .add_message::<MatchNotice>()
            .add_systems(Startup, initialize_match.in_set(StartupSet::Match))
            .add_systems(
                Update,
                apply_player_intents
                    .in_set(GameSet::Domain)
                    .run_if(in_state(RoundScreen::Playing)),
            );
    }
}

fn initialize_match(mut commands: Commands, catalog: Res<ContentCatalog>) {
    let mut game = Match::single_player(catalog.rules.clone(), "Player")
        .expect("the built-in player is valid");
    let mut rng = rand::rng();
    game.start_round(&mut rng)
        .expect("a new match can start its first round");
    game.begin_round()
        .expect("the initial static presentation is ready immediately");
    commands.insert_resource(ActiveMatch(game));
}

fn apply_player_intents(
    mut intents: MessageReader<PlayerIntent>,
    mut notices: MessageWriter<MatchNotice>,
    mut game: ResMut<ActiveMatch>,
    catalog: Res<ContentCatalog>,
    dictionaries: Res<Assets<DictionaryAsset>>,
) {
    for intent in intents.read() {
        let Some(dictionary) = dictionaries.get(&catalog.dictionary) else {
            notices.write(MatchNotice::SubmissionRejected(
                "Dictionary is still loading.".into(),
            ));
            continue;
        };
        let result = match intent {
            PlayerIntent::SubmitPath(path) => {
                game.0
                    .submit_path(LOCAL_PLAYER, path.clone(), &dictionary.0)
            }
            PlayerIntent::SubmitText(word) => game.0.submit_text(LOCAL_PLAYER, word, &dictionary.0),
        };
        match result {
            Ok(submission) => {
                notices.write(MatchNotice::SubmissionAccepted(submission));
            }
            Err(error) => {
                notices.write(MatchNotice::SubmissionRejected(error.to_string()));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use rand::SeedableRng;
    use word_grid_game_core::{
        Cube, CubeSet, Dictionary, GameRules, Match, PlayMode, ScoringTable,
    };

    use super::{ActiveMatch, MatchNotice, PlayerIntent, apply_player_intents};
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
            .insert_resource(ContentCatalog { rules, dictionary })
            .insert_resource(ActiveMatch(game))
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
}
