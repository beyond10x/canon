//! Adversary pass on story:artifact-revised-by, command-line contract (review of wave
//! 2026-11-21).
//!
//! The story's § Outcome 5 and 6 add a step to `canon simulate`, the external revise step written
//! `` protocol `q` revises `a` ``, offered for every artifact that declares `revised_by`, and a
//! summary phrase, `` W waiting on supplied evidence, expiry or a revision by another protocol ``;
//! § Scope lists the generated `website/docs/reference/cli.md` as changed at integration.
//! `ess/domains/cli.yaml` (`canon.cli.Simulate`) states both. The `canon simulate` help, from
//! which the CLI reference page is generated, lists every kind of step ("A step adds …, moves …,
//! adds …, grants or denies …, or takes one decision") and every kind of step a state is open by,
//! and names neither the external revise step nor `revised_by` nor the phrase: a reader of the
//! help or of the reference page cannot learn that the walk revises an artifact by itself, or what
//! the new summary means.

use std::process::Command;

fn help() -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_canon"))
        .args(["simulate", "--help"])
        .output()
        .expect("the canon binary runs");
    assert_eq!(output.status.code(), Some(0));
    String::from_utf8(output.stdout).expect("UTF-8 help")
}

/// The help of `canon simulate` names the step story:artifact-revised-by adds, the key that
/// declares it and the summary phrase it prints.
#[test]
fn simulate_help_names_the_external_revise_step_and_its_summary_phrase() {
    let help = help();
    let missing: Vec<&str> = ["revised_by", "a revision by another protocol"]
        .into_iter()
        .filter(|needle| !help.contains(needle))
        .collect();
    assert!(
        missing.is_empty(),
        "`canon simulate --help` does not mention {missing:?}:\n{help}"
    );
}
