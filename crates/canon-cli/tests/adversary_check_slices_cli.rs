//! Adversary pass 1 for story:check-subject-slices (wave 2026-11-20-w2), against what `canon check`
//! tells its user about the bound.
//!
//! The story's scope names "the doc comment of `Check` and the generated page"
//! (`crates/canon-cli/src/lib.rs`, the `Check` doc comment, and `website/docs/reference/cli.md`).
//! Since this story the bound holds each subject's slice: `canon check` checks the 196608 states of
//! `fixtures/investigation/check/sliced/`. Its help still ends "A state space of more than 65536
//! states is refused.", which that fixture contradicts.

use std::path::PathBuf;
use std::process::{Command, Output};

fn canon(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .args(args)
        .output()
        .expect("canon runs")
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The help of `canon check` does not say a space above 65536 states is refused, since the check
/// it describes reports a 196608-state protocol.
#[test]
fn check_help_does_not_refuse_every_space_above_the_bound() {
    let sliced = root().join("fixtures/investigation/check/sliced/protocol.yaml");
    let properties = root().join("fixtures/investigation/check/sliced/properties.yaml");
    let run = canon(&[
        "check",
        "--path",
        sliced.to_str().expect("utf-8"),
        "--properties",
        properties.to_str().expect("utf-8"),
    ]);
    let stdout = String::from_utf8(run.stdout).expect("utf-8");
    assert!(
        stdout.contains("196608 states") && run.stderr.is_empty(),
        "{stdout}"
    );
    let help = String::from_utf8(canon(&["check", "--help"]).stdout).expect("utf-8");
    assert!(
        !help.contains("A state space of more than 65536 states is refused"),
        "`canon check --help` still says every space above the bound is refused:\n{help}"
    );
}
