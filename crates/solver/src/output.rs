// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use crate::grid::WordGrid;
use crate::solver::WordResult;

pub fn format_word_grid(grid: &WordGrid) -> String {
    let mut output = String::from("--WORD GRID--\n");

    for (index, cell) in grid.cells().iter().enumerate() {
        output.push_str(cell);
        if (index + 1) % grid.size() == 0 {
            output.push('\n');
        } else {
            output.push(' ');
        }
    }

    output.push_str("-------------");
    output
}

pub fn format_results(results: &[WordResult]) -> String {
    let mut output = String::from("Words found:\n");

    for result in results {
        output.push_str(&result.word);
        output.push('\t');
        if result.word.len() < 8 {
            output.push('\t');
        }
        output.push_str(&result.score.to_string());
        output.push('\n');
    }

    let total_score: usize = results.iter().map(|result| result.score).sum();
    output.push_str("------------------------------------\n");
    output.push_str(&format!(
        "Total number of words: {}, Total score: {}",
        results.len(),
        total_score
    ));
    output
}
