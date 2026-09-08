// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use crate::grid::WordGrid;
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
