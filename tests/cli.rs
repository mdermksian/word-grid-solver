use std::process::Command;

#[test]
fn accepts_flags_and_prints_results() {
    let output = Command::new(env!("CARGO_BIN_EXE_word-grid-solver"))
        .args([
            "--size",
            "2",
            "--min-length",
            "3",
            "--dict",
            "tests/fixtures/words.txt",
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
    let output = Command::new(env!("CARGO_BIN_EXE_word-grid-solver"))
        .args([
            "--size",
            "2",
            "--min-length",
            "3",
            "--dict",
            "tests/fixtures/words.txt",
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
    let output = Command::new(env!("CARGO_BIN_EXE_word-grid-solver"))
        .args([
            "--size",
            "1",
            "--min-length",
            "1",
            "--dict",
            "tests/fixtures/missing.txt",
            "a",
        ])
        .output()
        .expect("command should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("could not read word list"));
}
