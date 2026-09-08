// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreBand {
    pub minimum_length: usize,
    pub points: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoringTable {
    bands: Vec<ScoreBand>,
}

impl ScoringTable {
    pub fn new(bands: impl IntoIterator<Item = ScoreBand>) -> Result<Self, ScoringTableError> {
        let bands: Vec<_> = bands.into_iter().collect();
        if bands.is_empty() {
            return Err(ScoringTableError::NoBands);
        }
        for (index, band) in bands.iter().enumerate() {
            if band.minimum_length == 0 {
                return Err(ScoringTableError::ZeroMinimumLength { index });
            }
            if index > 0 && bands[index - 1].minimum_length >= band.minimum_length {
                return Err(ScoringTableError::NotStrictlyIncreasing { index });
            }
        }
        Ok(Self { bands })
    }

    pub fn standard() -> Self {
        Self::new([
            ScoreBand {
                minimum_length: 3,
                points: 1,
            },
            ScoreBand {
                minimum_length: 5,
                points: 2,
            },
            ScoreBand {
                minimum_length: 6,
                points: 3,
            },
            ScoreBand {
                minimum_length: 7,
                points: 5,
            },
            ScoreBand {
                minimum_length: 8,
                points: 11,
            },
        ])
        .expect("standard scoring bands are valid")
    }

    pub fn bands(&self) -> &[ScoreBand] {
        &self.bands
    }

    pub fn score_word(&self, word: &str) -> usize {
        self.score_length(word.chars().count())
    }

    pub fn score_length(&self, length: usize) -> usize {
        self.bands
            .iter()
            .rev()
            .find(|band| length >= band.minimum_length)
            .map_or(0, |band| band.points)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScoringTableError {
    NoBands,
    ZeroMinimumLength { index: usize },
    NotStrictlyIncreasing { index: usize },
}

impl std::fmt::Display for ScoringTableError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoBands => write!(formatter, "scoring table must contain at least one band"),
            Self::ZeroMinimumLength { index } => {
                write!(formatter, "scoring band {index} has a zero minimum length")
            }
            Self::NotStrictlyIncreasing { index } => write!(
                formatter,
                "scoring band {index} is not ordered after the previous band"
            ),
        }
    }
}

impl std::error::Error for ScoringTableError {}

#[cfg(test)]
mod tests {
    use super::{ScoreBand, ScoringTable, ScoringTableError};

    #[test]
    fn standard_table_scores_every_boundary() {
        let scoring = ScoringTable::standard();
        assert_eq!(scoring.score_word("at"), 0);
        assert_eq!(scoring.score_word("cat"), 1);
        assert_eq!(scoring.score_word("cart"), 1);
        assert_eq!(scoring.score_word("crate"), 2);
        assert_eq!(scoring.score_word("crates"), 3);
        assert_eq!(scoring.score_word("closest"), 5);
        assert_eq!(scoring.score_word("longword"), 11);
    }

    #[test]
    fn rejects_unordered_bands() {
        assert_eq!(
            ScoringTable::new([
                ScoreBand {
                    minimum_length: 3,
                    points: 1,
                },
                ScoreBand {
                    minimum_length: 3,
                    points: 2,
                },
            ]),
            Err(ScoringTableError::NotStrictlyIncreasing { index: 1 })
        );
    }
}
