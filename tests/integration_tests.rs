use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_minimize_simple_json() {
    let mut cmd = Command::cargo_bin("qm-agent").unwrap();
    cmd.arg("minimize")
        .arg("-i")
        .arg(r#"{"minterms": [1, 3], "variables": 2}"#)
        .arg("-f")
        .arg("json");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("minimized_sop"));
}

#[test]
fn test_minimize_simple_format() {
    let mut cmd = Command::cargo_bin("qm-agent").unwrap();
    cmd.arg("minimize")
        .arg("-i")
        .arg("minimize minterms 1,3 with 2 variables");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Prime Implicants"));
}

#[test]
fn test_minimize_truth_table() {
    let mut cmd = Command::cargo_bin("qm-agent").unwrap();
    cmd.arg("minimize")
        .arg("-i")
        .arg("truth table: 1010");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Minimized Expression"));
}

#[test]
fn test_minimize_with_steps() {
    let mut cmd = Command::cargo_bin("qm-agent").unwrap();
    cmd.arg("minimize")
        .arg("-i")
        .arg("minimize minterms 1,3 with 2 variables")
        .arg("--show-steps");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Solution Steps"));
}

#[test]
fn test_minimize_table_format() {
    let mut cmd = Command::cargo_bin("qm-agent").unwrap();
    cmd.arg("minimize")
        .arg("-i")
        .arg("minimize minterms 1,3 with 2 variables")
        .arg("-f")
        .arg("table");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Truth Table"));
}

#[test]
fn test_minimize_from_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"{{"minterms": [0, 2], "variables": 2}}"#
    ).unwrap();

    let mut cmd = Command::cargo_bin("qm-agent").unwrap();
    cmd.arg("minimize")
        .arg("-i")
        .arg(temp_file.path().to_str().unwrap());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Minimized Expression"));
}

#[test]
fn test_examples_command() {
    let mut cmd = Command::cargo_bin("qm-agent").unwrap();
    cmd.arg("examples");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Usage Examples"));
}

#[test]
fn test_invalid_input() {
    let mut cmd = Command::cargo_bin("qm-agent").unwrap();
    cmd.arg("minimize")
        .arg("-i")
        .arg("invalid input format");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Could not parse input format"));
}

#[test]
fn test_help_message() {
    let mut cmd = Command::cargo_bin("qm-agent").unwrap();
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Quine-McCluskey Boolean minimization agent"));
}

#[test]
fn test_minimize_help() {
    let mut cmd = Command::cargo_bin("qm-agent").unwrap();
    cmd.arg("minimize").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Minimize a Boolean function"));
}