//! Adversary pass 1 for story:canon-check, run only in a scratch merge of
//! impl/subject-bound-evidence-match (working tree) with this unit's check module: the state space
//! puts every evidence record about the first declared artifact, so a match that names another
//! artifact never reads one.

use b10x_canon::check::check;
use b10x_canon::ir;
use b10x_canon::model;

fn report(source: &str) -> String {
    let compiled = ir::compile(&model::parse(source).expect("parses")).expect("compiles");
    match check(&compiled, None) {
        Ok(report) => report.to_string(),
        Err(refusal) => format!("refused {}: {refusal}", refusal.code()),
    }
}

/// `done` needs a record of `k` about `b`; `probe` may produce `k`. A case with such a record makes
/// `done` legitimate, so `done` is reachable and the check should report nothing.
#[test]
fn an_outcome_bound_to_the_second_artifact_is_reachable() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}, b: {}}\n\
        evidence_kinds: {k: {}}\n\
        claims: {b_seen: {true_when: {evidence: {kind: k, subject: b}}}}\n\
        actions: {probe: {may_produce: [{evidence: k}]}}\n\
        outcomes: {done: {requires: {claim: b_seen}}}\n";
    let text = report(source);
    assert!(
        !text.contains("unreachable-outcome"),
        "a reachable outcome was reported unreachable:\n{text}"
    );
}

/// `split` needs `k` about `a` to pass and `k` about `b` to fail: two records, one per artifact.
#[test]
fn two_matches_bound_to_different_artifacts_are_jointly_reachable() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}, b: {}}\n\
        evidence_kinds: {k: {}}\n\
        actions: {probe: {may_produce: [{evidence: k}]}}\n\
        outcomes:\n\
        \x20\x20split: {requires: {all: [{evidence: {kind: k, subject: a, result: pass}}, \
        {evidence: {kind: k, subject: b, result: fail}}]}}\n";
    let text = report(source);
    assert!(
        !text.contains("unreachable-outcome"),
        "a reachable outcome was reported unreachable:\n{text}"
    );
}
