//! Adversary pass 1 for story:outcomes, against the evaluator library: the `reasons` of a blocked
//! outcome when its requirement is a `not` over a claim or an evidence match.
//!
//! The module docs of `eval` state that `{claim: c, is: false}` "is `not` of" `{claim: c}`, and the
//! two give the same status for every value of `c`. A blocked outcome's reasons say why it is
//! blocked (design § 6 "not legitimate because ...", § 14 `blocked_outcomes[].because`), so two
//! requirements with the same meaning must give the same reasons, and a blocked outcome must give
//! at least one.

use b10x_canon::eval;
use b10x_canon::ir;
use b10x_canon::model::{self, Json};

const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n";

/// `c` is `true` with a `k` record whose result is `pass`, `false` with one whose result is
/// `fail`; `d` is `true` with any `l` record and `unknown` without one.
fn protocol(outcomes: &str) -> ir::Ir {
    let source = format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nartifacts: {{a: {{}}}}\n\
         evidence_kinds: {{k: {{}}, l: {{}}}}\n\
         claims:\n\
         \x20\x20c: {{true_when: {{evidence: {{kind: k, result: pass}}}}}}\n\
         \x20\x20d: {{true_when: {{evidence: {{kind: l}}}}}}\n\
         outcomes:\n{outcomes}"
    );
    ir::compile(&model::parse(&source).expect("protocol parses")).expect("protocol compiles")
}

fn record(kind: &str, result: Option<&str>) -> model::EvidenceRecord {
    let result = result.map(|r| format!("result: {r}\n")).unwrap_or_default();
    eval::read_evidence(&format!(
        "format: canon-evidence/1\nid: e-{kind}\nkind: {kind}\n{result}subject: a\nsubject_revision: r1\n"
    ))
    .expect("evidence reads")
}

fn outcomes(ir: &ir::Ir, evidence: &[model::EvidenceRecord]) -> Json {
    let case = eval::read_case(CASE).expect("case reads");
    eval::evaluate(ir, &case, evidence)
        .expect("decides")
        .outcomes
        .expect("the protocol declares outcomes")
}

/// `c` is `true`. `{not: {claim: c}}` and `{claim: c, is: false}` are both `false`, so both
/// outcomes are blocked by `c`; the `is: false` form names `c` with its value `true`, and the
/// `not` form must give the same reason.
#[test]
fn a_negation_over_a_true_claim_gives_the_reason_its_is_false_test_gives() {
    let ir = protocol(
        "  negated: {requires: {not: {claim: c}}}\n  tested_false: {requires: {claim: c, is: false}}\n",
    );
    let outcomes = outcomes(&ir, &[record("k", Some("pass"))]);
    assert_eq!(
        outcomes["tested_false"],
        serde_json::json!({"status": "blocked", "reasons": [{"claim": "c", "value": "true"}]}),
        "the control"
    );
    assert_eq!(
        outcomes["negated"], outcomes["tested_false"],
        "`not: {{claim: c}}` and `{{claim: c, is: false}}` mean the same and give different reasons: {outcomes:#}"
    );
}

/// `c` is `false`, `d` is `unknown`. Each outcome requires "not `c`" and `d`: the `not c` member is
/// met, so only `d` blocks. The `is: false` form names only `d`; the `not` form names `c` too, a
/// claim whose value is exactly what the requirement asks for.
#[test]
fn a_negated_claim_whose_negation_is_met_is_not_named_as_a_reason() {
    let ir = protocol(
        "  negated: {requires: {all: [{not: {claim: c}}, {claim: d}]}}\n  \
         tested_false: {requires: {all: [{claim: c, is: false}, {claim: d}]}}\n",
    );
    let outcomes = outcomes(&ir, &[record("k", Some("fail"))]);
    let only_d =
        serde_json::json!({"status": "blocked", "reasons": [{"claim": "d", "value": "unknown"}]});
    assert_eq!(outcomes["tested_false"], only_d, "the control");
    assert_eq!(
        outcomes["negated"], only_d,
        "`c` is `false`, which `not: {{claim: c}}` requires, and is still named as a reason: {outcomes:#}"
    );
}

/// An outcome whose requirement is an evidence match (which `canon compile` accepts in `requires`)
/// is blocked while no record of the kind exists. A blocked outcome names why; this one names
/// nothing.
#[test]
fn a_blocked_outcome_whose_requirement_is_an_evidence_match_states_a_reason() {
    let ir = protocol("  observed: {requires: {evidence: {kind: l}}}\n");
    let outcomes = outcomes(&ir, &[]);
    assert_eq!(outcomes["observed"]["status"], "blocked", "{outcomes:#}");
    let reasons = outcomes["observed"]["reasons"]
        .as_array()
        .expect("a blocked outcome carries a reasons list");
    assert!(
        !reasons.is_empty(),
        "a blocked outcome with no reason tells its reader nothing about why: {outcomes:#}"
    );
}
