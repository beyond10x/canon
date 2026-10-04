//! Adversary cases for story:evaluator-skeleton (wave 2026-10-04-w6, pass 2): the order the
//! validator documents for discharge predicates, validate/mod.rs module docs item 4: "those in
//! obligations (their discharge predicates, each followed by every evidence match it holds ...)".

use b10x_canon::{model, validate};

/// Two obligations, each with an unresolved reference and an evidence match: each obligation's
/// evidence matches follow its own references, before the next obligation's. The unit's own test
/// has one obligation, so reporting every obligation's references first and every evidence match
/// after (a second loop over the obligations) stays green without this case.
#[test]
fn each_discharge_predicate_is_followed_by_its_own_evidence_matches() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        evidence_kinds: {k: {}}\n\
        obligations:\n\
        \x20 o1: {discharged_when: {all: [{claim: x1}, {evidence: {kind: k}}]}}\n\
        \x20 o2: {discharged_when: {any: [{not: {evidence: {kind: k}}}, {claim: x2}]}}\n";
    let protocol = model::parse(source).expect("parses");
    let problems: Vec<String> = validate::validate(&protocol)
        .expect_err("refused")
        .iter()
        .map(|problem| format!("{}: {problem}", problem.code()))
        .collect();
    assert_eq!(
        problems,
        [
            "undeclared-claim: obligation `o1` references claim `x1`, which is not declared",
            "evidence-in-discharge: obligation `o1` tests evidence kind `k` at `discharged_when.all[1].evidence`; a discharge predicate tests only claim values",
            "undeclared-claim: obligation `o2` references claim `x2`, which is not declared",
            "evidence-in-discharge: obligation `o2` tests evidence kind `k` at `discharged_when.any[0].not.evidence`; a discharge predicate tests only claim values",
        ]
    );
}
