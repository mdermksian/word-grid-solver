use rand::{Rng, RngExt};
use std::collections::HashSet;
use std::time::Duration;
use word_grid_solver::{Dictionary, WordGrid, WordPathError, score_word};

pub const STANDARD_NEW_DICE: [[&str; 6]; 16] = [
    ["A", "E", "A", "N", "E", "G"],
    ["A", "H", "S", "P", "C", "O"],
    ["A", "S", "P", "F", "F", "K"],
    ["O", "B", "J", "O", "A", "B"],
    ["I", "O", "T", "M", "U", "C"],
    ["R", "Y", "V", "D", "E", "L"],
    ["L", "R", "E", "I", "X", "D"],
    ["E", "I", "U", "N", "E", "S"],
    ["W", "N", "G", "E", "E", "H"],
    ["L", "N", "H", "N", "R", "Z"],
    ["T", "S", "T", "I", "Y", "D"],
    ["O", "W", "T", "O", "A", "T"],
    ["E", "R", "T", "T", "Y", "L"],
    ["T", "O", "E", "S", "S", "I"],
    ["T", "E", "R", "W", "H", "V"],
    ["N", "U", "I", "H", "M", "Qu"],
];

pub const STANDARD_OLD_DICE: [[&str; 6]; 16] = [
    ["A", "A", "C", "I", "O", "T"],
    ["A", "B", "I", "L", "T", "Y"],
    ["A", "B", "J", "M", "O", "Qu"],
    ["A", "C", "D", "E", "M", "P"],
    ["A", "C", "E", "L", "R", "S"],
    ["A", "D", "E", "N", "V", "Z"],
    ["A", "H", "M", "O", "R", "S"],
    ["B", "I", "F", "O", "R", "X"],
    ["D", "E", "N", "O", "S", "W"],
    ["D", "K", "N", "O", "T", "U"],
    ["E", "E", "F", "H", "I", "Y"],
    ["E", "G", "K", "L", "U", "Y"],
    ["E", "G", "I", "N", "T", "V"],
    ["E", "H", "I", "N", "P", "S"],
    ["E", "L", "P", "S", "T", "U"],
    ["G", "I", "L", "R", "U", "W"],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CubeSet {
    StandardNew,
    StandardOld,
}

impl CubeSet {
    pub fn dice(self) -> &'static [[&'static str; 6]; 16] {
        match self {
            Self::StandardNew => &STANDARD_NEW_DICE,
            Self::StandardOld => &STANDARD_OLD_DICE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameDefinition {
    pub grid_size: usize,
    pub minimum_word_length: usize,
    pub time_limit: Option<Duration>,
    pub cube_set: CubeSet,
}

impl GameDefinition {
    pub fn normal(cube_set: CubeSet) -> Self {
        Self {
            grid_size: 4,
            minimum_word_length: 3,
            time_limit: Some(Duration::from_secs(180)),
            cube_set,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoardRoll {
    pub grid: WordGrid,
    pub die_indices: Vec<usize>,
}

impl BoardRoll {
    pub fn roll(definition: &GameDefinition, rng: &mut impl Rng) -> Self {
        assert_eq!(
            definition.cube_set.dice().len(),
            definition.grid_size * definition.grid_size
        );
        let mut die_indices: Vec<_> = (0..definition.cube_set.dice().len()).collect();
        // Fisher-Yates keeps the generated board independent of its renderer.
        for index in (1..die_indices.len()).rev() {
            die_indices.swap(index, rng.random_range(0..=index));
        }
        let cells = die_indices
            .iter()
            .map(|&die| definition.cube_set.dice()[die][rng.random_range(0..6)].to_string())
            .collect();
        Self {
            grid: WordGrid::new(definition.grid_size, cells).expect("validated cube set"),
            die_indices,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedWord {
    pub word: String,
    pub path: Vec<usize>,
    pub score: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmissionError {
    InvalidPath(WordPathError),
    TooShort { actual: usize, minimum: usize },
    NotOnBoard,
    NotInDictionary,
    Duplicate,
}

impl std::fmt::Display for SubmissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPath(error) => error.fmt(f),
            Self::TooShort { actual, minimum } => {
                write!(f, "{actual} letters; minimum is {minimum}")
            }
            Self::NotOnBoard => write!(f, "that word cannot be traced on this board"),
            Self::NotInDictionary => write!(f, "that word is not in the dictionary"),
            Self::Duplicate => write!(f, "you already found that word"),
        }
    }
}

pub struct SinglePlayerSession {
    pub definition: GameDefinition,
    pub board: BoardRoll,
    pub round_words: Vec<AcceptedWord>,
    pub completed_round_scores: Vec<usize>,
    dictionary: Dictionary,
    found: HashSet<String>,
}

impl SinglePlayerSession {
    pub fn new(definition: GameDefinition, dictionary: Dictionary, rng: &mut impl Rng) -> Self {
        let board = BoardRoll::roll(&definition, rng);
        Self {
            definition,
            board,
            round_words: Vec::new(),
            completed_round_scores: Vec::new(),
            dictionary,
            found: HashSet::new(),
        }
    }
    pub fn round_score(&self) -> usize {
        self.round_words.iter().map(|word| word.score).sum()
    }
    pub fn total_score(&self) -> usize {
        self.completed_round_scores.iter().sum::<usize>() + self.round_score()
    }
    pub fn submit_path(&mut self, path: Vec<usize>) -> Result<AcceptedWord, SubmissionError> {
        let word = self
            .board
            .grid
            .word_for_path(&path)
            .map_err(SubmissionError::InvalidPath)?;
        let length = word.chars().count();
        if length < self.definition.minimum_word_length {
            return Err(SubmissionError::TooShort {
                actual: length,
                minimum: self.definition.minimum_word_length,
            });
        }
        if !self.dictionary.is_word_valid(&word) {
            return Err(SubmissionError::NotInDictionary);
        }
        if !self.found.insert(word.clone()) {
            return Err(SubmissionError::Duplicate);
        }
        let accepted = AcceptedWord {
            score: score_word(&word),
            word,
            path,
        };
        self.round_words.push(accepted.clone());
        Ok(accepted)
    }
    pub fn submit_text(&mut self, word: &str) -> Result<AcceptedWord, SubmissionError> {
        let word = word.trim().to_ascii_lowercase();
        let path = self
            .board
            .grid
            .find_path_for_word(&word)
            .ok_or(SubmissionError::NotOnBoard)?;
        self.submit_path(path)
    }
    pub fn new_round(&mut self, rng: &mut impl Rng) {
        self.completed_round_scores.push(self.round_score());
        self.board = BoardRoll::roll(&self.definition, rng);
        self.round_words.clear();
        self.found.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn normal_session_scores_and_carries_total_between_rounds() {
        let mut rng = rand::rngs::SmallRng::seed_from_u64(4);
        let mut game = SinglePlayerSession::new(
            GameDefinition::normal(CubeSet::StandardNew),
            Dictionary::from_words(["cat"]),
            &mut rng,
        );
        game.board.grid = WordGrid::new(
            4,
            ["c", "a", "t"]
                .into_iter()
                .chain(std::iter::repeat("x"))
                .take(16)
                .map(String::from)
                .collect(),
        )
        .unwrap();
        assert_eq!(game.submit_text("cat").unwrap().score, 1);
        assert_eq!(game.submit_text("cat"), Err(SubmissionError::Duplicate));
        game.new_round(&mut rng);
        assert_eq!(game.total_score(), 1);
        assert!(game.round_words.is_empty());
    }
}
