// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

mod dictionary;
mod grid;
mod output;
mod solver;

pub use dictionary::Dictionary;
pub use grid::{WordGrid, WordPathError};
pub use output::format_word_grid;
pub use solver::{FoundWord, GridSolver};
