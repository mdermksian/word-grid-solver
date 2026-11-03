// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use rand::Rng;
use word_grid_solver::{Dictionary, WordPathError};

use crate::{BoardRoll, GameRules, PlayMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlayerId(u32);

impl PlayerId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    id: PlayerId,
    name: String,
}

impl Player {
    pub fn new(id: PlayerId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into().trim().to_string(),
        }
    }

    pub fn id(&self) -> PlayerId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchPhase {
    Ready,
    Rolling,
    Playing,
    Review,
    Complete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Submission {
    player_id: PlayerId,
    word: String,
    path: Vec<usize>,
    base_score: usize,
}

impl Submission {
    pub fn player_id(&self) -> PlayerId {
        self.player_id
    }

    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn path(&self) -> &[usize] {
        &self.path
    }

    pub fn base_score(&self) -> usize {
        self.base_score
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Round {
    board: BoardRoll,
    submissions: HashMap<PlayerId, Vec<Submission>>,
    found: HashMap<PlayerId, HashSet<String>>,
    remaining: Option<Duration>,
}

impl Round {
    pub fn board(&self) -> &BoardRoll {
        &self.board
    }

    pub fn submissions(&self, player_id: PlayerId) -> &[Submission] {
        self.submissions.get(&player_id).map_or(&[], Vec::as_slice)
    }

    pub fn remaining(&self) -> Option<Duration> {
        self.remaining
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerRoundResult {
    player_id: PlayerId,
    scored: Vec<Submission>,
    canceled: Vec<Submission>,
    score: usize,
}

impl PlayerRoundResult {
    pub fn player_id(&self) -> PlayerId {
        self.player_id
    }

    pub fn scored(&self) -> &[Submission] {
        &self.scored
    }

    pub fn canceled(&self) -> &[Submission] {
        &self.canceled
    }

    pub fn score(&self) -> usize {
        self.score
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoundResult {
    board: BoardRoll,
    players: Vec<PlayerRoundResult>,
}

impl RoundResult {
    pub fn board(&self) -> &BoardRoll {
        &self.board
    }

    pub fn players(&self) -> &[PlayerRoundResult] {
        &self.players
    }

    pub fn player(&self, player_id: PlayerId) -> Option<&PlayerRoundResult> {
        self.players
            .iter()
            .find(|result| result.player_id == player_id)
    }
}

#[derive(Debug)]
pub struct Match {
    rules: GameRules,
    players: Vec<Player>,
    phase: MatchPhase,
    current_round: Option<Round>,
    completed_rounds: Vec<RoundResult>,
}

impl Match {
    pub fn new(
        rules: GameRules,
        players: impl IntoIterator<Item = Player>,
    ) -> Result<Self, MatchError> {
        let players: Vec<_> = players.into_iter().collect();
        if players.is_empty() {
            return Err(MatchError::NoPlayers);
        }
        let mut ids = HashSet::with_capacity(players.len());
        for player in &players {
            if player.name.is_empty() {
                return Err(MatchError::EmptyPlayerName {
                    player_id: player.id,
                });
            }
            if !ids.insert(player.id) {
                return Err(MatchError::DuplicatePlayerId {
                    player_id: player.id,
                });
            }
        }
        Ok(Self {
            rules,
            players,
            phase: MatchPhase::Ready,
            current_round: None,
            completed_rounds: Vec::new(),
        })
    }

    pub fn single_player(rules: GameRules, name: impl Into<String>) -> Result<Self, MatchError> {
        Self::new(rules, [Player::new(PlayerId::new(0), name)])
    }

    pub fn rules(&self) -> &GameRules {
        &self.rules
    }

    pub fn players(&self) -> &[Player] {
        &self.players
    }

    pub fn phase(&self) -> MatchPhase {
        self.phase
    }

    pub fn current_round(&self) -> Option<&Round> {
        self.current_round.as_ref()
    }

    pub fn completed_rounds(&self) -> &[RoundResult] {
        &self.completed_rounds
    }

    pub fn start_round(&mut self, rng: &mut impl Rng) -> Result<&BoardRoll, MatchError> {
        if !matches!(self.phase, MatchPhase::Ready | MatchPhase::Review) {
            return Err(self.invalid_transition("start a round"));
        }
        let submissions = self
            .players
            .iter()
            .map(|player| (player.id, Vec::new()))
            .collect();
        let found = self
            .players
            .iter()
            .map(|player| (player.id, HashSet::new()))
            .collect();
        let remaining = match self.rules.play_mode() {
            PlayMode::TimedRounds { duration } => Some(duration),
            PlayMode::Endless => None,
        };
        self.current_round = Some(Round {
            board: BoardRoll::roll(&self.rules, rng),
            submissions,
            found,
            remaining,
        });
        self.phase = MatchPhase::Rolling;
        Ok(&self
            .current_round
            .as_ref()
            .expect("round was just inserted")
            .board)
    }

    pub fn begin_round(&mut self) -> Result<(), MatchError> {
        if self.phase != MatchPhase::Rolling {
            return Err(self.invalid_transition("begin accepting submissions"));
        }
        self.phase = MatchPhase::Playing;
        Ok(())
    }

    pub fn advance_time(&mut self, elapsed: Duration) -> Result<Option<RoundResult>, MatchError> {
        if self.phase != MatchPhase::Playing {
            return Err(self.invalid_transition("advance round time"));
        }
        let round = self
            .current_round
            .as_mut()
            .expect("Playing phase always has an active round");
        let Some(remaining) = round.remaining else {
            return Ok(None);
        };
        if elapsed < remaining {
            round.remaining = Some(remaining - elapsed);
            return Ok(None);
        }
        round.remaining = Some(Duration::ZERO);
        self.finish_round().map(Some)
    }

    pub fn submit_path(
        &mut self,
        player_id: PlayerId,
        path: Vec<usize>,
        dictionary: &Dictionary,
    ) -> Result<Submission, SubmissionError> {
        if self.phase != MatchPhase::Playing {
            return Err(SubmissionError::RoundInactive { phase: self.phase });
        }
        if !self.players.iter().any(|player| player.id == player_id) {
            return Err(SubmissionError::UnknownPlayer { player_id });
        }
        let round = self
            .current_round
            .as_mut()
            .expect("Playing phase always has an active round");
        let word = round
            .board
            .grid()
            .word_for_path(&path)
            .map_err(SubmissionError::InvalidPath)?;
        let length = word.chars().count();
        if length < self.rules.minimum_word_length() {
            return Err(SubmissionError::TooShort {
                actual: length,
                minimum: self.rules.minimum_word_length(),
            });
        }
        if !dictionary.is_word_valid(&word) {
            return Err(SubmissionError::NotInDictionary);
        }
        let player_found = round
            .found
            .get_mut(&player_id)
            .expect("known players have a found-word set");
        if !player_found.insert(word.clone()) {
            return Err(SubmissionError::Duplicate);
        }
        let submission = Submission {
            player_id,
            base_score: self.rules.scoring().score_word(&word),
            word,
            path,
        };
        round
            .submissions
            .get_mut(&player_id)
            .expect("known players have a submission list")
            .push(submission.clone());
        Ok(submission)
    }

    pub fn submit_text(
        &mut self,
        player_id: PlayerId,
        word: &str,
        dictionary: &Dictionary,
    ) -> Result<Submission, SubmissionError> {
        if self.phase != MatchPhase::Playing {
            return Err(SubmissionError::RoundInactive { phase: self.phase });
        }
        if !self.players.iter().any(|player| player.id == player_id) {
            return Err(SubmissionError::UnknownPlayer { player_id });
        }
        let path = self
            .current_round
            .as_ref()
            .expect("Playing phase always has an active round")
            .board
            .grid()
            .find_path_for_word(word)
            .ok_or(SubmissionError::NotOnBoard)?;
        self.submit_path(player_id, path, dictionary)
    }

    pub fn current_score(&self, player_id: PlayerId) -> Result<usize, MatchError> {
        if !self.players.iter().any(|player| player.id == player_id) {
            return Err(MatchError::UnknownPlayer { player_id });
        }
        Ok(self
            .current_round
            .as_ref()
            .map(|round| {
                round
                    .submissions(player_id)
                    .iter()
                    .map(Submission::base_score)
                    .sum()
            })
            .unwrap_or(0))
    }

    pub fn total_score(&self, player_id: PlayerId) -> Result<usize, MatchError> {
        if !self.players.iter().any(|player| player.id == player_id) {
            return Err(MatchError::UnknownPlayer { player_id });
        }
        Ok(self
            .completed_rounds
            .iter()
            .filter_map(|round| round.player(player_id))
            .map(PlayerRoundResult::score)
            .sum())
    }

    pub fn finish_round(&mut self) -> Result<RoundResult, MatchError> {
        if self.phase != MatchPhase::Playing {
            return Err(self.invalid_transition("finish the round"));
        }
        let round = self
            .current_round
            .take()
            .expect("Playing phase always has an active round");
        let mut counts = HashMap::<String, usize>::new();
        for submission in round.submissions.values().flatten() {
            *counts.entry(submission.word.clone()).or_default() += 1;
        }
        let players = self
            .players
            .iter()
            .map(|player| {
                let mut scored = Vec::new();
                let mut canceled = Vec::new();
                for submission in round.submissions(player.id).iter().cloned() {
                    if counts[submission.word()] > 1 {
                        canceled.push(submission);
                    } else {
                        scored.push(submission);
                    }
                }
                let score = scored.iter().map(Submission::base_score).sum();
                PlayerRoundResult {
                    player_id: player.id,
                    scored,
                    canceled,
                    score,
                }
            })
            .collect();
        let result = RoundResult {
            board: round.board,
            players,
        };
        self.completed_rounds.push(result.clone());
        self.phase = MatchPhase::Review;
        Ok(result)
    }

    pub fn finish_match(&mut self) -> Result<(), MatchError> {
        if self.phase != MatchPhase::Review {
            return Err(self.invalid_transition("finish the match"));
        }
        self.phase = MatchPhase::Complete;
        Ok(())
    }

    fn invalid_transition(&self, operation: &'static str) -> MatchError {
        MatchError::InvalidTransition {
            phase: self.phase,
            operation,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchError {
    NoPlayers,
    EmptyPlayerName {
        player_id: PlayerId,
    },
    DuplicatePlayerId {
        player_id: PlayerId,
    },
    UnknownPlayer {
        player_id: PlayerId,
    },
    InvalidTransition {
        phase: MatchPhase,
        operation: &'static str,
    },
}

impl std::fmt::Display for MatchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoPlayers => write!(formatter, "a match must have at least one player"),
            Self::EmptyPlayerName { player_id } => {
                write!(formatter, "player {} has an empty name", player_id.get())
            }
            Self::DuplicatePlayerId { player_id } => {
                write!(formatter, "player id {} is duplicated", player_id.get())
            }
            Self::UnknownPlayer { player_id } => {
                write!(formatter, "player {} is not in this match", player_id.get())
            }
            Self::InvalidTransition { phase, operation } => {
                write!(formatter, "cannot {operation} while match is {phase:?}")
            }
        }
    }
}

impl std::error::Error for MatchError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmissionError {
    RoundInactive { phase: MatchPhase },
    UnknownPlayer { player_id: PlayerId },
    InvalidPath(WordPathError),
    TooShort { actual: usize, minimum: usize },
    NotOnBoard,
    NotInDictionary,
    Duplicate,
}

impl std::fmt::Display for SubmissionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RoundInactive { phase } => {
                write!(
                    formatter,
                    "words cannot be submitted while match is {phase:?}"
                )
            }
            Self::UnknownPlayer { player_id } => {
                write!(formatter, "player {} is not in this match", player_id.get())
            }
            Self::InvalidPath(error) => error.fmt(formatter),
            Self::TooShort { actual, minimum } => {
                write!(formatter, "{actual} letters; minimum is {minimum}")
            }
            Self::NotOnBoard => write!(formatter, "that word cannot be traced on this board"),
            Self::NotInDictionary => write!(formatter, "that word is not in the dictionary"),
            Self::Duplicate => write!(formatter, "you already found that word"),
        }
    }
}

impl std::error::Error for SubmissionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPath(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use rand::SeedableRng;

    use crate::{
        Cube, CubeSet, Dictionary, GameRules, Match, MatchPhase, PlayMode, Player, PlayerId,
        ScoringTable, SubmissionError,
    };

    fn fixed_rules(play_mode: PlayMode) -> GameRules {
        let cubes = ["c", "a", "t", "s"].map(|letter| Cube::new([letter]).unwrap());
        GameRules::new(
            2,
            3,
            play_mode,
            CubeSet::new("test", cubes).unwrap(),
            ScoringTable::standard(),
        )
        .unwrap()
    }

    fn two_player_match() -> Match {
        Match::new(
            fixed_rules(PlayMode::Endless),
            [
                Player::new(PlayerId::new(1), "Ada"),
                Player::new(PlayerId::new(2), "Grace"),
            ],
        )
        .unwrap()
    }

    #[test]
    fn phase_guards_submission_and_timer_expiry_finishes_round() {
        let player = PlayerId::new(0);
        let mut game = Match::single_player(
            fixed_rules(PlayMode::TimedRounds {
                duration: Duration::from_secs(3),
            }),
            "Player",
        )
        .unwrap();
        let dictionary = Dictionary::from_words(["cat"]);
        assert!(matches!(
            game.submit_text(player, "cat", &dictionary),
            Err(SubmissionError::RoundInactive {
                phase: MatchPhase::Ready
            })
        ));

        let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
        game.start_round(&mut rng).unwrap();
        game.begin_round().unwrap();
        assert_eq!(game.advance_time(Duration::from_secs(2)).unwrap(), None);
        assert_eq!(
            game.current_round().unwrap().remaining(),
            Some(Duration::from_secs(1))
        );
        assert!(game.advance_time(Duration::from_secs(1)).unwrap().is_some());
        assert_eq!(game.phase(), MatchPhase::Review);
    }

    #[test]
    fn rejects_same_player_duplicate_and_reconciles_cross_player_words() {
        let mut game = two_player_match();
        let dictionary = Dictionary::from_words(["cat", "cats", "sat"]);
        let mut rng = rand::rngs::SmallRng::seed_from_u64(2);
        game.start_round(&mut rng).unwrap();
        game.begin_round().unwrap();

        let ada = PlayerId::new(1);
        let grace = PlayerId::new(2);
        game.submit_text(ada, "cat", &dictionary).unwrap();
        assert_eq!(
            game.submit_text(ada, "cat", &dictionary),
            Err(SubmissionError::Duplicate)
        );
        game.submit_text(grace, "cat", &dictionary).unwrap();
        game.submit_text(ada, "cats", &dictionary).unwrap();

        let result = game.finish_round().unwrap();
        assert_eq!(result.player(ada).unwrap().score(), 1);
        assert_eq!(result.player(ada).unwrap().canceled()[0].word(), "cat");
        assert_eq!(result.player(grace).unwrap().score(), 0);
        assert_eq!(result.player(grace).unwrap().canceled()[0].word(), "cat");
    }

    #[test]
    fn carries_player_totals_across_rounds() {
        let player = PlayerId::new(0);
        let mut game = Match::single_player(fixed_rules(PlayMode::Endless), "Player").unwrap();
        let dictionary = Dictionary::from_words(["cat"]);
        let mut rng = rand::rngs::SmallRng::seed_from_u64(4);

        for _ in 0..2 {
            game.start_round(&mut rng).unwrap();
            game.begin_round().unwrap();
            game.submit_text(player, "cat", &dictionary).unwrap();
            game.finish_round().unwrap();
        }
        assert_eq!(game.total_score(player).unwrap(), 2);
        game.finish_match().unwrap();
        assert_eq!(game.phase(), MatchPhase::Complete);
    }

    #[test]
    fn multi_letter_cube_counts_each_letter() {
        let cubes = ["qu", "i", "z", "x"].map(|letter| Cube::new([letter]).unwrap());
        let rules = GameRules::new(
            2,
            4,
            PlayMode::Endless,
            CubeSet::new("qu", cubes).unwrap(),
            ScoringTable::standard(),
        )
        .unwrap();
        let mut game = Match::single_player(rules, "Player").unwrap();
        let mut rng = rand::rngs::SmallRng::seed_from_u64(3);
        game.start_round(&mut rng).unwrap();
        game.begin_round().unwrap();
        let player = PlayerId::new(0);
        let grid = game.current_round().unwrap().board().grid().clone();
        let qu_index = grid.cells().iter().position(|cell| cell == "qu").unwrap();
        let mut path = vec![qu_index];
        path.extend(
            (0..grid.cells().len())
                .filter(|index| *index != qu_index)
                .take(2),
        );
        let word = grid.word_for_path(&path).unwrap();
        let dictionary = Dictionary::from_words([&word]);
        let submission = game.submit_path(player, path, &dictionary).unwrap();
        assert!(submission.word().chars().count() >= 4);
    }
}
