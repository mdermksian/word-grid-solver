// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Dictionary {
    words: HashSet<String>,
    prefixes: HashSet<String>,
}

impl Dictionary {
    pub fn from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut words = HashSet::new();
        let mut prefixes = HashSet::from([String::new()]);

        for line in reader.lines() {
            let word = line?.trim().to_ascii_lowercase();
            if word.is_empty() {
                continue;
            }

            for end in 1..=word.len() {
                prefixes.insert(word[..end].to_string());
            }
            words.insert(word);
        }

        Ok(Self { words, prefixes })
    }

    pub fn from_words(words: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        let mut dictionary = HashSet::new();
        let mut prefixes = HashSet::from([String::new()]);

        for word in words {
            let word = word.as_ref().trim().to_ascii_lowercase();
            if word.is_empty() {
                continue;
            }

            for end in 1..=word.len() {
                prefixes.insert(word[..end].to_string());
            }
            dictionary.insert(word);
        }

        Self {
            words: dictionary,
            prefixes,
        }
    }

    pub fn is_word_valid(&self, word: &str) -> bool {
        self.words.contains(&word.to_ascii_lowercase())
    }

    pub fn can_be_word(&self, prefix: &str) -> bool {
        self.prefixes.contains(&prefix.to_ascii_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::Dictionary;

    #[test]
    fn checks_exact_words_and_prefixes() {
        let dictionary = Dictionary::from_words(["cat", "cats", "dog"]);

        assert!(dictionary.is_word_valid("cat"));
        assert!(dictionary.is_word_valid("CATS"));
        assert!(!dictionary.is_word_valid("ca"));
        assert!(dictionary.can_be_word("ca"));
        assert!(dictionary.can_be_word(""));
        assert!(!dictionary.can_be_word("cow"));
    }
}
