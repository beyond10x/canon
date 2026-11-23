//! story:approvals-bound-to-evidence-content, documented form (review of wave 2026-11-22-w12).
//!
//! An evidence ground is checked by its id and by whether the record given under it is
//! applicable, not by the rest of its contents. So with other contents under a cited id the entry
//! still applies, and the decision changes only where a claim reads the replaced contents.
//! `approvals_cite_evidence_by_id.rs` pins the first half on a fixture whose claim reads no
//! result; here a claim reads the result, and the fields that decide applicability are shown to
//! decide the ground. The module docs of `eval/grounds.rs` are held to say both halves.

use std::path::PathBuf;

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model;

const AUTHORITY: &str =
    "- {capability: x, decision: granted, principal: p, ground_evidence: [e1]}\n";

fn protocol(claim: &str) -> ir::Ir {
    let source = format!(
        "format: protocol/1\nprotocol: {{id: grounds, revision: 1}}\n\
         artifacts: {{spec: {{}}}}\nevidence_kinds: {{k: {{}}}}\n\
         independence_dimensions: {{team: {{}}}}\n\
         claims:\n  c: {{true_when: {claim}}}\n\
         actions:\n  act: {{requires: [{{capability: x}}]}}\n"
    );
    ir::compile(&model::parse(&source).expect("parses")).expect("compiles")
}

const CASE: &str = "format: canon-case/1\nid: G-1\nprotocol: grounds\n\
                    artifacts: {spec: {revision: r2}}\nindependence: {alice: [team]}\n";

fn record(result: &str, producer: &str) -> String {
    format!(
        "format: canon-evidence/1\nid: e1\nkind: k\nresult: {result}\nsubject: spec\n\
         subject_revision: r2\nobserved_at: 2026-11-22T10:00:00Z\nproducer: {producer}\n"
    )
}

fn decide(ir: &ir::Ir, text: &str) -> model::Decision {
    let case = eval::read_case(CASE).expect("case reads");
    let evidence = [eval::read_evidence(text).expect("record reads")];
    let supplied = Supplied {
        authority: Some(AUTHORITY),
        ..Supplied::default()
    };
    eval::evaluate_with(ir, &case, &evidence, supplied).expect("decides")
}

fn claim(decision: &model::Decision) -> String {
    let rendered: serde_json::Value =
        serde_json::from_str(&eval::render(decision)).expect("the decision is JSON");
    rendered["claims"]["c"]["value"]
        .as_str()
        .expect("c has a value")
        .to_owned()
}

/// Replaced contents under `e1` (another result, another producer): the grant resting on `e1`
/// still makes `act` admissible, and the claim that reads the result changes, so the decision
/// does.
#[test]
fn replaced_contents_under_a_cited_id_keep_the_grant_and_change_a_claim_that_reads_them() {
    let ir = protocol("{evidence: {kind: k, result: pass}}");
    let before = decide(&ir, &record("pass", "alice"));
    let after = decide(&ir, &record("fail", "mallory"));
    let admissible = serde_json::json!({"status": "admissible"});
    assert_eq!(before.actions.as_ref().expect("actions")["act"], admissible);
    assert_eq!(
        after.actions.as_ref().expect("actions")["act"],
        admissible,
        "the grant on e1 still applies with the replaced contents"
    );
    assert_eq!(claim(&before), "true");
    assert_eq!(
        claim(&after),
        "false",
        "the claim reads the replaced result"
    );
    assert_ne!(eval::render(&after), eval::render(&before));
}

/// Where a claim demands an independent producer, the `producer` under `e1` decides whether the
/// record is applicable, so whether the ground holds: one the case does not list as independent
/// lists `e1` as `not_independent`, and the grant lapses.
#[test]
fn a_replaced_producer_a_claim_demands_independence_of_decides_the_ground() {
    let ir = protocol("{evidence: {kind: k, result: pass, independent_on: [team]}}");
    let before = decide(&ir, &record("pass", "alice"));
    let after = decide(&ir, &record("pass", "mallory"));
    assert_eq!(
        before.actions.as_ref().expect("actions")["act"],
        serde_json::json!({"status": "admissible"})
    );
    let act = &after.actions.as_ref().expect("actions")["act"];
    assert_ne!(act["status"], "admissible", "{act}");
    assert_eq!(
        act["reasons"][0]["stale_grounds"],
        serde_json::json!([{"evidence": "e1", "reason": "not_independent"}]),
        "{act}"
    );
}

/// The words of a file's `//!` lines, lower case, whitespace runs as one space.
fn module_docs(relative: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(root.join(relative)).expect("the source reads");
    let lines: Vec<String> = text
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("//!").map(str::to_lowercase))
        .collect();
    lines
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// `eval/grounds.rs` says both halves, and no longer that the decision stays the same byte for
/// byte.
#[test]
fn the_grounds_docs_say_the_entry_applies_and_only_a_claim_reading_the_contents_changes() {
    let docs = module_docs("src/eval/grounds.rs");
    for half in [
        "the entry still applies",
        "the decision changes only where a claim reads the replaced contents",
        "`subject_revision`, `observed_at` and `upstream_revisions`",
        "`producer`",
    ] {
        assert!(docs.contains(half), "grounds.rs does not say `{half}`");
    }
    assert!(
        !docs.contains("the decision is the same, byte for byte"),
        "grounds.rs still says replaced contents leave every byte of the decision"
    );
}
