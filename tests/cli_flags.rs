use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn test_generate_completion() {
    let mut cmd = Command::cargo_bin("genlint").unwrap();
    cmd.args(["generate-completion", "bash"])
        .assert()
        .success();
}

#[test]
fn test_invalid_combination_long_line_and_max_length() {
    let mut cmd = Command::cargo_bin("genlint").unwrap();
    cmd.args(["--stdin", "--disable", "long-line", "--max-line-length", "120"])
        .write_stdin("test_invalid_combination_long_line_and_max_length")
        .assert()
        .failure()
        .stderr(contains("Cannot use --max-line-length"));
}

#[test]
fn test_invalid_combination_blank_and_max_consecutive_blank() {
    let mut cmd = Command::cargo_bin("genlint").unwrap();
    cmd.args(["--stdin", "--disable", "consecutive-blank", "--max-consecutive-blank", "2"])
        .write_stdin("test_invalid_combination_blank_and_max_consecutive_blank")
        .assert()
        .failure()
        .stderr(contains("Cannot use --max-consecutive-blank"));
}

#[test]
fn test_invalid_format_option() {
    let mut cmd = Command::cargo_bin("genlint").unwrap();
    cmd.args(["--stdin", "--format", "badformat"])
        .write_stdin("test_invalid_format_option")
        .assert()
        .failure()
        .stderr(contains("invalid value 'badformat' for '--format <FORMAT>'"));
}

#[test]
fn test_valid_input_file() {
    let mut cmd = Command::cargo_bin("genlint").unwrap();
    cmd.args([
        "--input",
        "tests/data/trailing_spaces.txt",
    ])
    .assert()
    .success()
    .stdout(contains("Trailing whitespaces or tabs").count(1));
}

#[test]
fn test_invalid_input_file() {
    let mut cmd = Command::cargo_bin("genlint").unwrap();
    cmd.args([
        "--input",
        "tests/data/non_exist.txt",
    ])
    .assert()
    .success()
    .stdout(contains(" ").count(0));
}

#[test]
fn test_disable_mix_indent() {
    let mut cmd = Command::cargo_bin("genlint").unwrap();
    cmd.args([
        "--disable",
        "mix-indent",
        "--stdin",
    ])
    .write_stdin("  \tMixed indentation line.\n")
    .assert()
    .success()
    .stdout(contains("Mixed tabs and whitespaces").count(0));
}

#[test]
fn test_mix_indent() {
    let mut cmd = Command::cargo_bin("genlint").unwrap();
    cmd.args([
        "--stdin",
    ])
    .write_stdin("  \tMixed indentation line.\n")
    .assert()
    .success()
    .stdout(contains("Mixed tabs and whitespaces").count(1));
}

#[test]
fn test_disable_trailing_space() {
    let mut cmd = Command::cargo_bin("genlint").unwrap();
    cmd.args([
        "--disable",
        "trailing-space",
        "--stdin",
    ])
    .write_stdin("Trailing whitespaces and tabs are here:  \t\t\n")
    .assert()
    .success()
    .stdout(contains("Trailing whitespaces or tabs").count(0));
}

#[test]
fn test_trailing_space() {
    let mut cmd = Command::cargo_bin("genlint").unwrap();
    cmd.args([
        "--stdin",
    ])
    .write_stdin("Trailing whitespaces and tabs are here:  \t\t\n")
    .assert()
    .success()
    .stdout(contains("Trailing whitespaces or tabs").count(1));
}

#[test]
fn test_disable_conflict_marker() {
    let mut cmd = Command::cargo_bin("genlint").unwrap();
    cmd.args([
        "--disable",
        "conflict-marker",
        "--stdin",
    ])
    .write_stdin("Here are\n<<<<<<< Head\nGit\n=======\nconflict\n>>>>>>> Remote\nmarkers\n")
    .assert()
    .success()
    .stdout(contains("Git conflict marker").count(0));
}

#[test]
fn test_conflict_marker() {
    let mut cmd = Command::cargo_bin("genlint").unwrap();
    cmd.args([
        "--stdin",
    ])
    .write_stdin("Here are\n<<<<<<< Head\nGit\n=======\nconflict\n>>>>>>> Remote\nmarkers\n")
    .assert()
    .success()
    .stdout(contains("Git conflict marker").count(3));
}

#[test]
fn test_disable_long_line() {
    let mut cmd = Command::cargo_bin("genlint").unwrap();
    cmd.args([
        "--disable",
        "long-line",
        "--stdin",
    ])
    .write_stdin(format!("This is a{} long line.\n", " very".repeat(30)))
    .assert()
    .success()
    .stdout(contains("Too long line").count(0));
}

#[test]
fn test_long_line() {
    let mut cmd = Command::cargo_bin("genlint").unwrap();
    cmd.args([
        "--stdin",
    ])
    .write_stdin(format!("This is a{} long line.\n", " very".repeat(30)))
    .assert()
    .success()
    .stdout(contains("Too long line").count(1));
}

#[test]
fn test_disable_control_char() {
    let mut cmd = Command::cargo_bin("genlint").unwrap();
    cmd.args([
        "--disable",
        "control-char",
        "--stdin",
    ])
    .write_stdin("Unexpected control character: \x15\n")
    .assert()
    .success()
    .stdout(contains("Line contains a control character").count(0));
}

#[test]
fn test_control_char() {
    let mut cmd = Command::cargo_bin("genlint").unwrap();
    cmd.args([
        "--stdin",
    ])
    .write_stdin("Unexpected control character: \x15\n")
    .assert()
    .success()
    .stdout(contains("Line contains a control character").count(1));
}
