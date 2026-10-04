//! Adversary case for story:action-admissibility (wave 2026-10-04-w7, pass 1): the published
//! reference against the evaluator. The unit made `--authority` read and the `actions` section
//! written; the generated pages still say neither happens.
//!
//! Sources of the pages: `crates/canon-cli/src/lib.rs` (the `--authority` help, cli.md),
//! `crates/canon/src/eval/mod.rs` module docs (evaluation.md), `crates/canon-docs/src/pages.rs`
//! (investigation-example.md).

use std::path::PathBuf;

use b10x_canon::eval::{self, Supplied};
use b10x_canon::{ir, model};

fn repository_root() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set"))
        .join("../..")
}

fn read(path: &str) -> String {
    std::fs::read_to_string(repository_root().join(path)).expect(path)
}

#[test]
fn the_reference_does_not_say_authority_is_unsupported_or_actions_unevaluated() {
    // The behaviour: a granted `--authority` is read, and the decision carries `actions`.
    let protocol = read("fixtures/investigation/action-admissibility.yaml");
    let ir = ir::compile(&model::parse(&protocol).expect("parses")).expect("compiles");
    let case = eval::read_case(
        "format: canon-case/1\nid: INV-18\nprotocol: investigation\nartifacts: {explanation: {revision: r1}}\n",
    )
    .expect("case reads");
    let supplied = Supplied {
        authority: Some("- {capability: finding.publish, decision: granted}\n"),
        ..Supplied::default()
    };
    let decision = eval::evaluate_with(&ir, &case, &[], supplied).expect("authority is read");
    assert!(decision.actions.is_some(), "the decision carries actions");

    // The pages.
    let mut stale = Vec::new();
    let cli = read("website/docs/reference/cli.md");
    for line in cli
        .lines()
        .filter(|line| line.contains("canon-authority/1"))
    {
        if line.contains("not supported yet") {
            stale.push(format!("cli.md: {}", line.trim()));
        }
    }
    let evaluation = read("website/docs/reference/evaluation.md");
    if evaluation.contains("`` `--authority` is not supported yet ``") {
        stale.push("evaluation.md: authority is refused as `unsupported-input`".to_owned());
    }
    if evaluation.contains("each section is absent") {
        stale.push("evaluation.md: each section is absent".to_owned());
    }
    let example = read("website/docs/reference/investigation-example.md");
    if example.contains("obligations, actions and outcomes are not") {
        stale.push(
            "investigation-example.md: actions are not evaluated (under a decision listing them)"
                .to_owned(),
        );
    }
    assert!(stale.is_empty(), "stale reference:\n{}", stale.join("\n"));
}
