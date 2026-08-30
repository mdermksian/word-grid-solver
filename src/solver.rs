use std::collections::HashSet;

use crate::dictionary::Dictionary;
use crate::grid::WordGrid;
use crate::scoring::score_word;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordResult {
    pub word: String,
    pub score: usize,
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

    pub fn find_words(&self, grid: &WordGrid) -> Vec<WordResult> {
        let mut found = HashSet::new();
        let mut visited = vec![false; grid.cells().len()];

        for index in 0..grid.cells().len() {
            self.search(grid, index, &mut String::new(), &mut visited, &mut found);
        }

        let mut results: Vec<_> = found
            .into_iter()
            .map(|word| {
                let score = score_word(&word);
                WordResult { word, score }
            })
            .collect();

        results.sort_by(|first, second| {
            second
                .word
                .len()
                .cmp(&first.word.len())
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
        found: &mut HashSet<String>,
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

        if prefix.len() >= self.min_length && self.dictionary.is_word_valid(prefix) {
            found.insert(prefix.clone());
        }

        for neighbor in grid.neighbors(index) {
            self.search(grid, neighbor, prefix, visited, found);
        }

        visited[index] = false;
        prefix.truncate(original_len);
    }
}

#[cfg(test)]
mod tests {
    use super::{GridSolver, WordResult};
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
                WordResult {
                    word: "cast".into(),
                    score: 1
                },
                WordResult {
                    word: "cats".into(),
                    score: 1
                },
                WordResult {
                    word: "cat".into(),
                    score: 1
                },
                WordResult {
                    word: "sat".into(),
                    score: 1
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
