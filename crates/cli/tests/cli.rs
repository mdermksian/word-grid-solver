use std::path::PathBuf;
use std::process::Command;

fn fixture_path(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("tests/fixtures")
        .join(name)
        .display()
        .to_string()
}

#[test]
fn accepts_flags_and_prints_results() {
    let words = fixture_path("words.txt");
    let output = Command::new(env!("CARGO_BIN_EXE_word-grid-solver"))
        .args([
            "--size",
            "2",
            "--min-length",
            "3",
            "--dict",
            &words,
            "c",
            "a",
            "t",
            "s",
        ])
        .output()
        .expect("command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("--WORD GRID--"));
    assert!(stdout.contains("cast"));
    assert!(stdout.contains("Total number of words: 4"));
}

#[test]
fn rejects_grid_length_mismatch() {
    let words = fixture_path("words.txt");
    let output = Command::new(env!("CARGO_BIN_EXE_word-grid-solver"))
        .args([
            "--size",
            "2",
            "--min-length",
            "3",
            "--dict",
            &words,
            "c",
            "a",
            "t",
        ])
        .output()
        .expect("command should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("requires 4 cells"));
}

#[test]
fn reports_missing_dictionary() {
    let missing = fixture_path("missing.txt");
    let output = Command::new(env!("CARGO_BIN_EXE_word-grid-solver"))
        .args(["--size", "1", "--min-length", "1", "--dict", &missing, "a"])
        .output()
        .expect("command should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("could not read word list"));
}
