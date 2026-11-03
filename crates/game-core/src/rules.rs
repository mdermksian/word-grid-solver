// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use std::time::Duration;

use crate::{CubeSet, ScoringTable};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayMode {
    TimedRounds { duration: Duration },
    Endless,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameRules {
    grid_size: usize,
    minimum_word_length: usize,
    play_mode: PlayMode,
    cube_set: CubeSet,
    scoring: ScoringTable,
}

impl GameRules {
    pub fn new(
        grid_size: usize,
        minimum_word_length: usize,
        play_mode: PlayMode,
        cube_set: CubeSet,
        scoring: ScoringTable,
    ) -> Result<Self, GameRulesError> {
        if grid_size == 0 {
            return Err(GameRulesError::ZeroGridSize);
        }
        if minimum_word_length < 3 {
            return Err(GameRulesError::MinimumWordLengthTooSmall {
                actual: minimum_word_length,
            });
        }
        let expected = grid_size
            .checked_mul(grid_size)
            .ok_or(GameRulesError::GridSizeOverflow)?;
        if cube_set.cubes().len() != expected {
            return Err(GameRulesError::CubeCountMismatch {
                actual: cube_set.cubes().len(),
                expected,
            });
        }
        if let PlayMode::TimedRounds { duration } = play_mode
            && duration.is_zero()
        {
            return Err(GameRulesError::ZeroRoundDuration);
        }
        Ok(Self {
            grid_size,
            minimum_word_length,
            play_mode,
            cube_set,
            scoring,
        })
    }

    pub fn normal(cube_set: CubeSet) -> Result<Self, GameRulesError> {
        Self::new(
            4,
            3,
            PlayMode::TimedRounds {
                duration: Duration::from_secs(180),
            },
            cube_set,
            ScoringTable::standard(),
        )
    }

    pub fn big() -> Self {
        Self::new(
            5,
            4,
            PlayMode::TimedRounds {
                duration: Duration::from_secs(180),
            },
            CubeSet::big(),
            ScoringTable::standard(),
        )
        .expect("built-in Big rules are valid")
    }

    pub fn grid_size(&self) -> usize {
        self.grid_size
    }

    pub fn minimum_word_length(&self) -> usize {
        self.minimum_word_length
    }

    pub fn play_mode(&self) -> PlayMode {
        self.play_mode
    }

    pub fn cube_set(&self) -> &CubeSet {
        &self.cube_set
    }

    pub fn scoring(&self) -> &ScoringTable {
        &self.scoring
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameRulesError {
    ZeroGridSize,
    GridSizeOverflow,
    MinimumWordLengthTooSmall { actual: usize },
    CubeCountMismatch { actual: usize, expected: usize },
    ZeroRoundDuration,
}

impl std::fmt::Display for GameRulesError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroGridSize => write!(formatter, "grid size must be greater than zero"),
            Self::GridSizeOverflow => write!(formatter, "grid size is too large"),
            Self::MinimumWordLengthTooSmall { actual } => write!(
                formatter,
                "minimum word length must be at least 3, but was {actual}"
            ),
            Self::CubeCountMismatch { actual, expected } => write!(
                formatter,
                "cube set has {actual} cubes, but the grid requires {expected}"
            ),
            Self::ZeroRoundDuration => {
                write!(formatter, "round duration must be greater than zero")
            }
        }
    }
}

impl std::error::Error for GameRulesError {}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{Cube, CubeSet, GameRules, GameRulesError, PlayMode, ScoringTable};

    #[test]
    fn rejects_cube_count_and_invalid_time() {
        let one_cube = CubeSet::new("tiny", [Cube::new(["a"]).unwrap()]).unwrap();
        assert_eq!(
            GameRules::new(
                2,
                3,
                PlayMode::Endless,
                one_cube.clone(),
                ScoringTable::standard(),
            ),
            Err(GameRulesError::CubeCountMismatch {
                actual: 1,
                expected: 4,
            })
        );
        assert_eq!(
            GameRules::new(
                1,
                3,
                PlayMode::TimedRounds {
                    duration: Duration::ZERO,
                },
                one_cube,
                ScoringTable::standard(),
            ),
            Err(GameRulesError::ZeroRoundDuration)
        );
    }
}
