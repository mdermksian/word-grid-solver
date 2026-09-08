// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michael Dermksian

use std::env;
use std::path::PathBuf;
use std::process;

use word_grid_game_core::ScoringTable;
use word_grid_solver::{Dictionary, FoundWord, GridSolver, WordGrid, format_word_grid};

#[derive(Debug, PartialEq, Eq)]
struct Config {
    size: usize,
    min_length: usize,
    dict_path: PathBuf,
    cells: Vec<String>,
}

fn main() {
    if let Err(err) = run(env::args().skip(1)) {
        eprintln!("Error: {err}");
        process::exit(1);
    }
}

fn run(args: impl IntoIterator<Item = String>) -> Result<(), String> {
    let config = parse_args(args)?;
    let dictionary = Dictionary::from_file(&config.dict_path).map_err(|err| {
        format!(
            "could not read word list '{}': {err}",
            config.dict_path.display()
        )
    })?;
    let grid = WordGrid::new(config.size, config.cells)?;
    let solver = GridSolver::new(dictionary, config.min_length);
    let results = solver.find_words(&grid);

    println!("{}", format_word_grid(&grid));
    println!("{}", format_results(&results, &ScoringTable::standard()));

    Ok(())
}

fn format_results(results: &[FoundWord], scoring: &ScoringTable) -> String {
    let mut output = String::from("Words found:\n");
    let mut total_score = 0;

    for result in results {
        let score = scoring.score_word(&result.word);
        total_score += score;
        output.push_str(&result.word);
        output.push('\t');
        if result.word.len() < 8 {
            output.push('\t');
        }
        output.push_str(&score.to_string());
        output.push('\n');
    }

    output.push_str("------------------------------------\n");
    output.push_str(&format!(
        "Total number of words: {}, Total score: {total_score}",
        results.len()
    ));
    output
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Config, String> {
    let mut size = None;
    let mut min_length = None;
    let mut dict_path = PathBuf::from("twl06.txt");
    let mut cells = Vec::new();
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" => return Err(usage()),
            "--size" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--size requires a value".to_string())?;
                size = Some(parse_positive_usize("--size", &value)?);
            }
            "--min-length" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--min-length requires a value".to_string())?;
                min_length = Some(parse_positive_usize("--min-length", &value)?);
            }
            "--dict" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--dict requires a path".to_string())?;
                dict_path = PathBuf::from(value);
            }
            _ if arg.starts_with('-') => {
                return Err(format!("unknown option '{arg}'\n\n{}", usage()));
            }
            _ => cells.push(arg),
        }
    }

    Ok(Config {
        size: size.ok_or_else(|| format!("--size is required\n\n{}", usage()))?,
        min_length: min_length.ok_or_else(|| format!("--min-length is required\n\n{}", usage()))?,
        dict_path,
        cells,
    })
}

fn parse_positive_usize(name: &str, value: &str) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| format!("{name} must be a positive integer"))?;

    if parsed == 0 {
        return Err(format!("{name} must be greater than 0"));
    }

    Ok(parsed)
}

fn usage() -> String {
    [
        "Usage:",
        "  word-grid-solver --size <N> --min-length <N> [--dict <PATH>] <CELL>...",
        "",
        "Example:",
        "  word-grid-solver --size 2 --min-length 3 c a t s",
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::{Config, parse_args};
    use std::path::PathBuf;

    #[test]
    fn parses_required_options_and_cells() {
        let config = parse_args([
            "--size".to_string(),
            "2".to_string(),
            "--min-length".to_string(),
            "3".to_string(),
            "--dict".to_string(),
            "words.txt".to_string(),
            "c".to_string(),
            "a".to_string(),
            "t".to_string(),
            "s".to_string(),
        ])
        .expect("valid args");

        assert_eq!(
            config,
            Config {
                size: 2,
                min_length: 3,
                dict_path: PathBuf::from("words.txt"),
                cells: vec!["c".into(), "a".into(), "t".into(), "s".into()],
            }
        );
    }

    #[test]
    fn rejects_invalid_size() {
        let err = parse_args([
            "--size".to_string(),
            "0".to_string(),
            "--min-length".to_string(),
            "3".to_string(),
            "a".to_string(),
        ])
        .expect_err("zero size should fail");

        assert_eq!(err, "--size must be greater than 0");
    }
}
