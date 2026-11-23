//! Review of wave 2026-11-22-w12, story:approvals-bound-to-evidence-content (documented form).
//!
//! `eval/grounds.rs` states: "with other contents under the same id, still applicable, the entry
//! still applies and the decision is the same, byte for byte". The entry still applies; the
//! decision is the same only when no claim reads the replaced contents. `approvals_cite_evidence_
//! by_id.rs` pins the sentence on a fixture whose one claim reads no result (`{evidence: {kind:
//! k}}`). Here a claim reads the result: the grant resting on `e1` still applies, and the decision
//! differs.

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model;

const PROTOCOL: &str = "format: protocol/1\nprotocol: {id: grounds, revision: 1}\n\
     artifacts: {spec: {}}\nevidence_kinds: {k: {}}\n\
     claims:\n  c: {true_when: {evidence: {kind: k, result: pass}}}\n\
     actions:\n  act: {requires: [{capability: x}]}\n";

const CASE: &str =
    "format: canon-case/1\nid: G-1\nprotocol: grounds\nartifacts: {spec: {revision: r2}}\n";

const AUTHORITY: &str =
    "- {capability: x, decision: granted, principal: p, ground_evidence: [e1]}\n";

fn record(result: &str, producer: &str) -> String {
    format!(
        "format: canon-evidence/1\nid: e1\nkind: k\nresult: {result}\nsubject: spec\n\
         subject_revision: r2\nobserved_at: 2026-11-22T10:00:00Z\nproducer: {producer}\n"
    )
}

fn decide(text: &str) -> model::Decision {
    let ir = ir::compile(&model::parse(PROTOCOL).expect("parses")).expect("compiles");
    let case = eval::read_case(CASE).expect("case reads");
    let evidence = [eval::read_evidence(text).expect("record reads")];
    let supplied = Supplied {
        authority: Some(AUTHORITY),
        ..Supplied::default()
    };
    eval::evaluate_with(&ir, &case, &evidence, supplied).expect("decides")
}

#[test]
fn replaced_contents_under_a_cited_id_change_no_byte_of_the_decision_as_grounds_rs_states() {
    let before = decide(&record("pass", "alice"));
    let after = decide(&record("fail", "mallory"));
    let admissible = serde_json::json!({"status": "admissible"});
    assert_eq!(before.actions.as_ref().expect("actions")["act"], admissible);
    assert_eq!(
        after.actions.as_ref().expect("actions")["act"],
        admissible,
        "the grant on e1 still applies with the replaced contents"
    );
    assert_eq!(
        eval::render(&after),
        eval::render(&before),
        "grounds.rs: with other contents under the same id, still applicable, the entry still \
         applies and the decision is the same, byte for byte"
    );
}
