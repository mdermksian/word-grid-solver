// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

pub fn score_word(word: &str) -> usize {
    match word.len() {
        0..=2 => 0,
        3 | 4 => 1,
        5 => 2,
        6 => 3,
        7 => 5,
        _ => 11,
    }
}

#[cfg(test)]
mod tests {
    use super::score_word;

    #[test]
    fn scores_each_length_boundary() {
        assert_eq!(score_word("at"), 0);
        assert_eq!(score_word("cat"), 1);
        assert_eq!(score_word("cart"), 1);
        assert_eq!(score_word("crate"), 2);
        assert_eq!(score_word("crates"), 3);
        assert_eq!(score_word("closest"), 5);
        assert_eq!(score_word("longword"), 11);
        assert_eq!(score_word("longwords"), 11);
    }
}
