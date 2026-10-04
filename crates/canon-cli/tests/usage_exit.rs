//! Every command line `canon` does not understand exits `USAGE` (64), which the published
//! exit-status table describes as a usage error; `--help` and `--version` succeed
//! (story:review-hardening-w7). Exit 2 stays the unreadable-input status alone.

use std::process::{Command, Output};

use canon_cli::{EXIT_STATUSES, UNREADABLE, USAGE};

fn canon(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .args(args)
        .output()
        .expect("canon runs")
}

#[test]
fn every_usage_error_exits_the_usage_status() {
    for args in [
        vec![],
        vec!["no-such-command"],
        vec!["validate"],
        vec!["validate", "--path"],
        vec!["validate", "--path", "p.yaml", "--no-such-option"],
        vec!["evaluate", "--ir", "ir.json"],
        vec!["conform"],
        vec!["conform", "run", "--scenarios", "a", "--scenarios", "b"],
    ] {
        let run = canon(&args);
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert_eq!(
            run.status.code(),
            Some(i32::from(USAGE)),
            "{args:?}: {stderr}"
        );
        // clap's message, or, for a missing command, the help it prints instead.
        assert!(
            !stderr.is_empty() && run.stdout.is_empty(),
            "{args:?}: {stderr}"
        );
    }
}

#[test]
fn help_and_version_succeed_on_standard_output() {
    for args in [
        vec!["--help"],
        vec!["--version"],
        vec!["evaluate", "--help"],
    ] {
        let run = canon(&args);
        assert_eq!(run.status.code(), Some(0), "{args:?}");
        assert!(!run.stdout.is_empty(), "{args:?}");
    }
}

#[test]
fn the_table_gives_usage_and_unreadable_statuses_of_their_own() {
    let meaning = |status: u8| {
        EXIT_STATUSES
            .iter()
            .find(|(code, _)| *code == status)
            .map(|(_, meaning)| *meaning)
            .unwrap_or_else(|| panic!("status {status} is not in the table"))
    };
    assert_ne!(USAGE, UNREADABLE);
    assert!(meaning(USAGE).starts_with("Usage error"));
    assert!(meaning(UNREADABLE).starts_with("Unreadable"));
    let mut codes: Vec<u8> = EXIT_STATUSES.iter().map(|(code, _)| *code).collect();
    codes.dedup();
    assert_eq!(
        codes.len(),
        EXIT_STATUSES.len(),
        "each status is listed once"
    );
}
