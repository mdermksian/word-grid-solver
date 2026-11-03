// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use std::collections::HashMap;

use crate::dictionary::Dictionary;
use crate::grid::WordGrid;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundWord {
    pub word: String,
    pub path: Vec<usize>,
}

pub struct GridSolver {
    dictionary: Dictionary,
    min_length: usize,
}

impl GridSolver {
    pub fn new(dictionary: Dictionary, min_length: usize) -> Self {
        Self {
            dictionary,
            min_length,
        }
    }

    pub fn find_words(&self, grid: &WordGrid) -> Vec<FoundWord> {
        let mut found = HashMap::new();
        let mut visited = vec![false; grid.cells().len()];
        let mut path = Vec::new();

        for index in 0..grid.cells().len() {
            self.search(
                grid,
                index,
                &mut String::new(),
                &mut visited,
                &mut path,
                &mut found,
            );
        }

        let mut results: Vec<_> = found
            .into_iter()
            .map(|(word, path)| FoundWord { word, path })
            .collect();

        results.sort_by(|first, second| {
            second
                .word
                .chars()
                .count()
                .cmp(&first.word.chars().count())
                .then_with(|| first.word.cmp(&second.word))
        });
        results
    }

    fn search(
        &self,
        grid: &WordGrid,
        index: usize,
        prefix: &mut String,
        visited: &mut [bool],
        path: &mut Vec<usize>,
        found: &mut HashMap<String, Vec<usize>>,
    ) {
        if visited[index] {
            return;
        }

        let original_len = prefix.len();
        prefix.push_str(grid.cell(index));

        if !self.dictionary.can_be_word(prefix) {
            prefix.truncate(original_len);
            return;
        }

        visited[index] = true;
        path.push(index);

        if prefix.chars().count() >= self.min_length && self.dictionary.is_word_valid(prefix) {
            found.entry(prefix.clone()).or_insert_with(|| path.clone());
        }

        for neighbor in grid.neighbors(index) {
            self.search(grid, neighbor, prefix, visited, path, found);
        }

        path.pop();
        visited[index] = false;
        prefix.truncate(original_len);
    }
}

#[cfg(test)]
mod tests {
    use super::{FoundWord, GridSolver};
    use crate::{Dictionary, WordGrid};

    #[test]
    fn finds_unique_words_sorted_longest_first() {
        let dictionary = Dictionary::from_words(["cat", "cats", "cast", "sat", "at", "taco"]);
        let grid = WordGrid::new(
            2,
            ["c", "a", "t", "s"].into_iter().map(String::from).collect(),
        )
        .expect("valid grid");
        let solver = GridSolver::new(dictionary, 3);

        assert_eq!(
            solver.find_words(&grid),
            vec![
                FoundWord {
                    word: "cast".into(),
                    path: vec![0, 1, 3, 2],
                },
                FoundWord {
                    word: "cats".into(),
                    path: vec![0, 1, 2, 3],
                },
                FoundWord {
                    word: "cat".into(),
                    path: vec![0, 1, 2],
                },
                FoundWord {
                    word: "sat".into(),
                    path: vec![3, 1, 2],
                },
            ]
        );
    }

    #[test]
    fn does_not_reuse_cells_in_one_path() {
        let dictionary = Dictionary::from_words(["aaa"]);
        let grid = WordGrid::new(2, vec!["a".into(), "b".into(), "c".into(), "d".into()])
            .expect("valid grid");
        let solver = GridSolver::new(dictionary, 3);

        assert!(solver.find_words(&grid).is_empty());
    }
}
