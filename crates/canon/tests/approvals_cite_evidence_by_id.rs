//! story:approvals-bound-to-evidence-content, documented form. An approval (a `canon-authority/1`
//! grant or a `canon-decisions/1` decision) names the evidence it rested on by record id only: no
//! supplied format carries a digest of a record's contents, and adding one is a change of a
//! versioned format, which waits for a design decision. So Canon checks the id, and whoever stores
//! the records keeps an id bound to one content. These tests pin that behaviour, so the day a
//! digest is bound they fail and are replaced, and hold that the module docs state the
//! requirement.

use std::path::PathBuf;

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, EvidenceRecord};

/// The words every place that states the requirement uses, compared without case and with runs of
/// whitespace as one space.
const REQUIREMENT: &str = "evidence ids must be immutable once an approval cites them";

fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("run through cargo"))
}

/// Read at run time, so a test binary reused from another build directory reads this tree.
fn fixture() -> ir::Ir {
    let path = root().join("../../fixtures/investigation/decision-grounds.yaml");
    let text = std::fs::read_to_string(&path).expect("the fixture reads");
    ir::compile(&model::parse(&text).expect("the fixture parses")).expect("the fixture compiles")
}

const CASE: &str = "format: canon-case/1\nid: G-1\nprotocol: grounds\nrevision: c2\n\
                    artifacts: {spec: {revision: r2}, up: {revision: u2}}\n";

const AUTHORITY: &str =
    "- {capability: x, decision: granted, principal: p, ground_evidence: [e1]}\n";

const DECISIONS: &str =
    "- {decision: d, outcome: o, principal: q, case_revision: c2, ground_evidence: [e1]}\n";

/// The record `e1` as it was when the approvals were taken.
const ORIGINAL: &str = "format: canon-evidence/1\nid: e1\nkind: k\nresult: pass\nsubject: spec\n\
                        subject_revision: r2\nobserved_at: 2026-11-22T10:00:00Z\n\
                        upstream_revisions: {up: u2}\nproducer: alice\n";

/// Other contents stored under the same id later: another result, another producer, another
/// observation instant, still applicable.
const REPLACED: &str = "format: canon-evidence/1\nid: e1\nkind: k\nresult: fail\nsubject: spec\n\
                        subject_revision: r2\nobserved_at: 2026-11-22T10:30:00Z\n\
                        upstream_revisions: {up: u2}\nproducer: mallory\n";

fn decide(evidence: &[EvidenceRecord]) -> model::Decision {
    let case = eval::read_case(CASE).expect("case reads");
    eval::evaluate_with(
        &fixture(),
        &case,
        evidence,
        Supplied {
            authority: Some(AUTHORITY),
            decisions: Some(DECISIONS),
            ..Supplied::default()
        },
    )
    .expect("decides")
}

fn record(text: &str) -> EvidenceRecord {
    eval::read_evidence(text).expect("evidence reads")
}

/// Replacing the contents stored under `e1` between two evaluations leaves both approvals holding:
/// the grant still makes `act` admissible, the decision still makes `o` legitimate, and the
/// decision document is the same, byte for byte. Without `e1` both lapse, so the ground is read.
#[test]
fn an_approval_holds_when_the_contents_under_its_evidence_id_are_replaced() {
    let original = record(ORIGINAL);
    let replaced = record(REPLACED);
    assert_ne!(
        original, replaced,
        "the two records differ in their contents"
    );
    assert_eq!(original.id, replaced.id, "and share their id");

    let before = decide(std::slice::from_ref(&original));
    let after = decide(std::slice::from_ref(&replaced));
    assert_eq!(
        before.actions.as_ref().expect("actions")["act"],
        serde_json::json!({"status": "admissible"})
    );
    assert_eq!(
        before.outcomes.as_ref().expect("outcomes")["o"],
        serde_json::json!({"status": "legitimate",
                           "decided_by": {"decision": "d", "principals": ["q"]}})
    );
    assert_eq!(
        eval::render(&after),
        eval::render(&before),
        "Canon checks an evidence ground by id only, so replaced contents change no byte"
    );

    let without = decide(&[]);
    assert_eq!(
        without.actions.expect("actions")["act"]["reasons"][0]["stale_grounds"],
        serde_json::json!([{"evidence": "e1", "present": false}]),
        "control: the grant rests on e1"
    );
    assert_ne!(
        without.outcomes.expect("outcomes")["o"]["status"],
        serde_json::json!("legitimate"),
        "control: the decision rests on e1"
    );
}

/// The words of a file's `//!` and `///` lines, lower case, whitespace runs as one space.
fn doc_text(relative: &str) -> String {
    let text = std::fs::read_to_string(root().join(relative)).expect("the source reads");
    let words: Vec<String> = text
        .lines()
        .filter_map(|line| {
            let line = line.trim_start();
            line.strip_prefix("//!")
                .or_else(|| line.strip_prefix("///"))
                .map(str::to_lowercase)
        })
        .collect();
    words
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The module docs of the grounds (both approval documents) and of the explicit decisions state
/// the requirement, so a reader of either learns who keeps an id bound to one content.
#[test]
fn the_module_docs_state_that_cited_evidence_ids_must_be_immutable() {
    for file in ["src/eval/grounds.rs", "src/eval/decisions.rs"] {
        assert!(
            doc_text(file).contains(REQUIREMENT),
            "{file} does not state `{REQUIREMENT}`"
        );
    }
}
