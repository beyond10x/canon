//! Adversary pass 1 for story:subject-bound-evidence-match: the `reasons` an action and an outcome
//! state when an evidence match that names a `subject` is unmet.
//!
//! The protocol has the shape the ELS story needs: two artifacts, `change` and `docs`, and an
//! evidence kind `test_result`. A match bound to `change` must not be satisfied by a test result
//! about `docs`, and the eval module docs say such a record "is not of the match and neither
//! establishes nor contradicts it".
//!
//! The reasons for an unmet evidence match are `{"evidence": <kind>, "present": <bool>}`. The
//! `actions` docs (crates/canon/src/eval/actions.rs:14-16) say `present` "is whether a record of
//! that kind applies: `false` when none does, `true` when one does (one under `not` that a record
//! satisfies, or a match whose records have another result)". The `outcomes` docs
//! (crates/canon/src/eval/outcomes.rs:21-23) say "`false` for a required record that is missing,
//! `true` for a record present under `not`". A record about another artifact is neither of the
//! cases documented for `true`: the record the match requires is missing. So a record the match
//! does not read must not change the reason the match gives, and the reason must be the one the
//! match gives over no evidence at all.

use b10x_canon::eval;
use b10x_canon::ir;
use b10x_canon::model::{self, Json};

const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\n\
    artifacts: {change: {revision: c1}, docs: {revision: d1}}\n";

fn protocol(sections: &str) -> ir::Ir {
    let source = format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\n\
         artifacts: {{change: {{}}, docs: {{}}}}\n\
         evidence_kinds: {{test_result: {{}}}}\n{sections}"
    );
    ir::compile(&model::parse(&source).expect("protocol parses")).expect("protocol compiles")
}

/// A `test_result` record about `subject`, bound to that artifact's current revision in [`CASE`].
fn about(id: &str, subject: &str, result: &str) -> model::EvidenceRecord {
    let revision = match subject {
        "change" => "c1",
        "docs" => "d1",
        other => panic!("no artifact `{other}` in the case"),
    };
    eval::read_evidence(&format!(
        "format: canon-evidence/1\nid: {id}\nkind: test_result\nresult: {result}\n\
         subject: {subject}\nsubject_revision: {revision}\n"
    ))
    .expect("evidence reads")
}

fn decide(ir: &ir::Ir, evidence: &[model::EvidenceRecord]) -> model::Decision {
    let case = eval::read_case(CASE).expect("case reads");
    eval::evaluate(ir, &case, evidence).expect("decides")
}

fn actions(ir: &ir::Ir, evidence: &[model::EvidenceRecord]) -> Json {
    decide(ir, evidence)
        .actions
        .expect("the protocol declares actions")
}

fn outcomes(ir: &ir::Ir, evidence: &[model::EvidenceRecord]) -> Json {
    decide(ir, evidence)
        .outcomes
        .expect("the protocol declares outcomes")
}

const BOUND: &str = "{evidence: {kind: test_result, result: pass, subject: change}}";

/// `merge` needs a passing test result about the change. The only record is a passing test result
/// about the docs, at the docs' current revision: the match does not read it, so the precondition
/// is `unknown` exactly as with no record, and the record the precondition needs is missing.
#[test]
fn an_action_blocked_by_a_subject_bound_match_reports_the_missing_record_as_not_present() {
    let ir = protocol(&format!("actions:\n  merge: {{precondition: {BOUND}}}\n"));
    // The control is whatever the action gives over no record at all; whatever shape a reason
    // takes, a record the match does not read must not change it.
    let missing = actions(&ir, &[])["merge"].clone();
    assert_eq!(
        (&missing["status"], &missing["reasons"][0]["present"]),
        (&serde_json::json!("blocked"), &serde_json::json!(false)),
        "the control: no record, {missing:#}"
    );
    let decision = decide(&ir, &[about("t-docs", "docs", "pass")]);
    assert_eq!(
        decision.actions.as_ref().expect("actions")["merge"],
        missing,
        "a test result about the docs is not of a match bound to the change, yet the reason says a \
         record of the kind applies: {:#}",
        decision.actions.as_ref().expect("actions")
    );
}

/// The same requirement on an outcome: `released` requires a passing test result about the change,
/// and the only record is about the docs.
#[test]
fn an_outcome_blocked_by_a_subject_bound_match_reports_the_missing_record_as_not_present() {
    let ir = protocol(&format!("outcomes:\n  released: {{requires: {BOUND}}}\n"));
    let missing = outcomes(&ir, &[])["released"].clone();
    assert_eq!(
        (&missing["status"], &missing["reasons"][0]["present"]),
        (&serde_json::json!("blocked"), &serde_json::json!(false)),
        "the control: no record, {missing:#}"
    );
    let got = outcomes(&ir, &[about("t-docs", "docs", "pass")]);
    assert_eq!(
        got["released"], missing,
        "a test result about the docs is not of a match bound to the change, yet the reason says \
         a required record is present: {got:#}"
    );
}

/// Two matches of one kind that differ only in their subject: `released` needs a passing test
/// result about the change and one about the docs. The change's is there and passes, so only the
/// docs match is unmet, and the record it needs is missing. The reason must say so, as it does for
/// an outcome that requires the docs match alone over the same evidence.
#[test]
fn of_two_matches_differing_only_in_subject_the_unmet_one_reports_its_own_record_missing() {
    let ir = protocol(
        "outcomes:\n  \
         released: {requires: {all: [\
            {evidence: {kind: test_result, result: pass, subject: change}}, \
            {evidence: {kind: test_result, result: pass, subject: docs}}]}}\n  \
         documented: {requires: {evidence: {kind: test_result, result: pass, subject: docs}}}\n",
    );
    let got = outcomes(&ir, &[about("t-change", "change", "pass")]);
    // The docs match alone, over no record: the reason a missing docs record gives.
    let docs_missing = outcomes(&ir, &[])["documented"].clone();
    assert_eq!(
        docs_missing["reasons"][0]["present"],
        serde_json::json!(false),
        "the control: no record, {docs_missing:#}"
    );
    assert_eq!(
        (&got["released"], &got["documented"]),
        (&docs_missing, &docs_missing),
        "only the docs match is unmet and no record about the docs exists: {got:#}"
    );
}
