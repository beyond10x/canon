//! Adversary pass 1 on story:simulate-revise-step: command-line cases.
//!
//! `fixtures/investigation/judgement-marking.yaml` is the one repository protocol outside the six
//! `revise*` fixtures whose `canon simulate` output the revise step changes: its `draft_spec`
//! declares `revises: [spec]`, and before the revise step the walk printed a `cycle` at mixed
//! `spec_review` and `spec_test` results (exit 1). No test ran `canon simulate` on it. § Outcome 5
//! of the story says mixed results left by a declared revision are not reported (class c), so the
//! exact bytes and exit 0 are pinned here.

use std::path::PathBuf;
use std::process::{Command, Output};

fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn simulate(path: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(repository_root())
        .args(["simulate", "--path", path])
        .output()
        .expect("the canon binary runs")
}

/// story:simulate-revise-step § Outcome 5: mixed results on an artifact the protocol revises
/// through an admissible action are not reported. `draft_spec` has no precondition and revises
/// `spec`, which every unbound kind of the protocol is about, so every state reaches the empty
/// state and from it the outcome: exit 0, the summary line only, the same bytes twice.
#[test]
fn judgement_marking_has_no_finding_once_its_revision_is_walked() {
    let expected = "simulated: protocol `investigation.drafting` revision 1: 19 reachable \
                    states, 1 terminable (outcome `accepted` 1), 0 findings (0 stuck or cycle \
                    states)\n";
    let first = simulate("fixtures/investigation/judgement-marking.yaml");
    let second = simulate("fixtures/investigation/judgement-marking.yaml");
    assert_eq!(String::from_utf8_lossy(&first.stdout), expected);
    assert_eq!(String::from_utf8_lossy(&first.stderr), "");
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.status.code(), second.status.code());
}
