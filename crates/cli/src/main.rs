use std::env;
use std::path::PathBuf;
use std::process;

use word_grid_solver::{format_results, format_word_grid, Dictionary, GridSolver, WordGrid};

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
    println!("{}", format_results(&results));

    Ok(())
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
                return Err(format!("unknown option '{arg}'\n\n{}", usage()))
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
    use super::{parse_args, Config};
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
