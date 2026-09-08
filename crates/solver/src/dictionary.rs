// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Dictionary {
    words: HashSet<String>,
    prefixes: HashSet<String>,
}

impl Dictionary {
    pub fn from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        fs::read_to_string(path).map(|text| Self::from_text(&text))
    }

    pub fn from_text(text: &str) -> Self {
        Self::from_words(text.lines())
    }

    pub fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        std::str::from_utf8(bytes)
            .map(Self::from_text)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }

    pub fn from_words(words: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        let mut dictionary = HashSet::new();
        let mut prefixes = HashSet::from([String::new()]);

        for word in words {
            let word = word.as_ref().trim().to_ascii_lowercase();
            if word.is_empty() {
                continue;
            }

            for end in word
                .char_indices()
                .map(|(index, _)| index)
                .skip(1)
                .chain(std::iter::once(word.len()))
            {
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

    #[test]
    fn constructs_from_text_and_rejects_invalid_utf8() {
        let dictionary = Dictionary::from_text("cat\nDOG\ncafé\n\n");
        assert!(dictionary.is_word_valid("cat"));
        assert!(dictionary.is_word_valid("dog"));
        assert!(dictionary.can_be_word("caf"));
        assert!(Dictionary::from_bytes(&[0xff]).is_err());
    }
}
