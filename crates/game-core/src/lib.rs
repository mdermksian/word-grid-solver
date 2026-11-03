// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

mod cube;
mod game_match;
mod roll;
mod rules;
mod scoring;

pub use cube::{Cube, CubeSet, CubeSetError};
pub use game_match::{
    Match, MatchError, MatchPhase, Player, PlayerId, PlayerRoundResult, Round, RoundResult,
    Submission, SubmissionError,
};
pub use roll::{BoardRoll, RolledCube};
pub use rules::{GameRules, GameRulesError, PlayMode};
pub use scoring::{ScoreBand, ScoringTable, ScoringTableError};
pub use word_grid_solver::{Dictionary, WordGrid, WordPathError};
