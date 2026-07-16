use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn reads_stdin_and_prints_ai_context() {
    let mut command = Command::cargo_bin("devtriage-cli").unwrap();
    command
        .write_stdin("TypeError: boom\n at run (src/a.ts:4:2)")
        .assert()
        .success()
        .stdout(predicate::str::contains("## Facts"))
        .stdout(predicate::str::contains("## Request"));
}

#[test]
fn json_mode_never_prints_the_original_secret() {
    let mut command = Command::cargo_bin("devtriage-cli").unwrap();
    command
        .arg("--json")
        .write_stdin("fatal token=hidden-value")
        .assert()
        .success()
        .stdout(predicate::str::contains("credential_redacted"))
        .stdout(predicate::str::contains("hidden-value").not());
}
