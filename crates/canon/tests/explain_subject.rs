//! The explanation of an evidence match that names a `subject` (story:explanation, after
//! story:subject-bound-evidence-match): the records listed under it are exactly the records the
//! match reads — of its kind and about that artifact — and a record of the kind about another
//! artifact is not listed, applied or otherwise.
//!
//! The protocol has two artifacts, `change` and `docs`, one evidence kind `test_result`, a claim
//! and an action precondition bound to `change`, and a control claim whose match names no subject
//! and so reads records about either artifact.

use b10x_canon::eval;
use b10x_canon::ir;
use b10x_canon::model::{self, EvidenceRecord};
use serde_json::{Value, json};

/// `change` is at `c2`, so a record bound to `c1` is excluded as `revision_mismatch`.
const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\n\
    artifacts: {change: {revision: c2}, docs: {revision: d1}}\n";

const PROTOCOL: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {change: {}, docs: {}}\n\
    evidence_kinds: {test_result: {}}\n\
    claims:\n\
    \x20\x20reviewed: {true_when: {evidence: {kind: test_result, result: pass, subject: change}}}\n\
    \x20\x20failing: {true_when: {evidence: {kind: test_result, result: fail}}}\n\
    actions:\n\
    \x20\x20merge: {precondition: {evidence: {kind: test_result, result: pass, subject: change}}}\n";

fn record(id: &str, subject: &str, revision: &str) -> EvidenceRecord {
    eval::read_evidence(&format!(
        "format: canon-evidence/1\nid: {id}\nkind: test_result\nresult: pass\n\
         subject: {subject}\nsubject_revision: {revision}\n"
    ))
    .expect("a canon-evidence/1 record")
}

fn explanation(evidence: &[EvidenceRecord]) -> Value {
    let ir =
        ir::compile(&model::parse(PROTOCOL).expect("protocol parses")).expect("protocol compiles");
    let case = eval::read_case(CASE).expect("a canon-case/1 snapshot");
    eval::evaluate(&ir, &case, evidence)
        .expect("decides")
        .explanation
        .expect("every decision carries an explanation")
}

/// Every record id listed anywhere in `value` with a `status`.
fn listed(value: &Value) -> Vec<String> {
    let mut ids = Vec::new();
    match value {
        Value::Object(map) => {
            if let (Some(id), Some(_)) = (
                map.get("evidence").and_then(Value::as_str),
                map.get("status"),
            ) {
                ids.push(id.to_owned());
            }
            for member in map.values() {
                ids.extend(listed(member));
            }
        }
        Value::Array(items) => items.iter().for_each(|item| ids.extend(listed(item))),
        _ => {}
    }
    ids
}

/// The record about `change` is excluded and the record about `docs` applies: the subject-bound
/// claim and action list the `change` record, excluded, and not the `docs` record.
#[test]
fn a_subject_bound_match_lists_only_records_about_its_subject_when_excluded() {
    let explained = explanation(&[
        record("change-1", "change", "c1"),
        record("docs-1", "docs", "d1"),
    ]);
    let excluded =
        json!({"evidence": "change-1", "reason": "revision_mismatch", "status": "excluded"});
    assert_eq!(
        explained["claims"]["reviewed"],
        json!({"because": [excluded], "value": "unknown"}),
        "the claim lists exactly the record its match reads"
    );
    assert_eq!(
        explained["actions"]["merge"],
        json!({
            "because": [
                {"evidence": "test_result", "present": false, "subject": "change"},
                excluded,
            ],
            "status": "blocked",
        }),
        "the action's evidence reason is followed by exactly the record its match reads"
    );
    for section in ["actions", "obligations", "outcomes"] {
        assert!(
            !listed(&explained[section]).contains(&"docs-1".to_owned()),
            "{section} lists the record about `docs`: {explained:#}"
        );
    }
    assert!(!listed(&explained["claims"]["reviewed"]).contains(&"docs-1".to_owned()));
}

/// No record about `change` at all, one about `docs`: the subject-bound match reads none, so its
/// claim gives the match as absent and its action's reason is followed by no record.
#[test]
fn a_subject_bound_match_with_no_record_about_its_subject_lists_none() {
    let explained = explanation(&[record("docs-1", "docs", "d1")]);
    assert_eq!(
        explained["claims"]["reviewed"],
        json!({
            "because": [{"kind": "test_result", "status": "absent", "subject": "change"}],
            "value": "unknown",
        }),
        "the claim's match reads no record, so it is absent"
    );
    assert_eq!(
        explained["actions"]["merge"],
        json!({
            "because": [{"evidence": "test_result", "present": false, "subject": "change"}],
            "status": "blocked",
        }),
        "a record about `docs` does not follow a reason bound to `change`"
    );
}

/// A match that names no subject reads records about any artifact, and lists them all.
#[test]
fn a_match_without_a_subject_lists_records_about_every_artifact() {
    let explained = explanation(&[
        record("change-1", "change", "c1"),
        record("docs-1", "docs", "d1"),
    ]);
    assert_eq!(
        explained["claims"]["failing"],
        json!({
            "because": [
                {"evidence": "change-1", "reason": "revision_mismatch", "status": "excluded"},
                {"evidence": "docs-1", "status": "applied"},
            ],
            "value": "false",
        })
    );
}
