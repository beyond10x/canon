//! Adversary cases for story:action-admissibility (wave 2026-10-04-w7, pass 1), against the
//! evaluator library: the reasons a `blocked` action gives for an unmet precondition.
//!
//! Driven from the unit's own contract: CANON-AUTHORITY-001's header ("`reasons` explain the status
//! (only the reasons that decide it)"), the `actions.rs` module docs ("the `reasons` that decide
//! that status"), the story's Outcome ("blocked with a reason naming the claim and its value") and
//! the evaluator module docs (`{claim: c, is: false}` is `not` of `c`).

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, CapabilityId, Json};
use serde_json::json;

const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {x: {revision: r1}}\n";

/// Three claims, each `true` with a `pass` record of its kind, `false` with a `fail` one and
/// `unknown` with none, and actions whose preconditions the validator accepts.
const PROTOCOL: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1}\n\
    artifacts: {x: {}}\n\
    evidence_kinds: {ka: {}, kb: {}, kc: {}}\n\
    claims:\n\
    \x20\x20a: {true_when: {evidence: {kind: ka, result: pass}}}\n\
    \x20\x20b: {true_when: {evidence: {kind: kb, result: pass}}}\n\
    \x20\x20c: {true_when: {evidence: {kind: kc, result: pass}}}\n\
    actions:\n\
    \x20\x20negated: {precondition: {not: {claim: a}}}\n\
    \x20\x20tested_false: {precondition: {claim: a, is: false}}\n\
    \x20\x20branch: {precondition: {all: [{claim: a}, {any: [{claim: b}, {claim: c}]}]}}\n\
    \x20\x20guarded: {precondition: {all: [{not: {claim: a}}, {claim: b}]}}\n\
    \x20\x20gated: {requires: [{capability: d}, {capability: z}]}\n";

fn compiled() -> ir::Ir {
    ir::compile(&model::parse(PROTOCOL).expect("protocol parses")).expect("protocol compiles")
}

/// The `actions` section for the given `(kind, result)` records, with no authority.
fn actions(ir: &ir::Ir, records: &[(&str, &str)]) -> Json {
    let case = eval::read_case(CASE).expect("case reads");
    let evidence: Vec<_> = records
        .iter()
        .map(|(kind, result)| {
            eval::read_evidence(&format!(
                "format: canon-evidence/1\nid: {kind}-1\nkind: {kind}\nresult: {result}\nsubject: x\nsubject_revision: r1\n"
            ))
            .expect("evidence reads")
        })
        .collect();
    eval::evaluate_with(ir, &case, &evidence, Supplied::default())
        .expect("evaluates")
        .actions
        .expect("an actions section")
}

/// `{not: {claim: a}}` and `{claim: a, is: false}` are the same predicate (eval module docs). With
/// `a` TRUE both are blocked because `a` is TRUE, and the story says the reason names the claim and
/// its value. `tested_false` does; `negated` gives `{"precondition": "false"}`, naming no claim.
#[test]
fn equivalent_negations_give_the_same_reason_naming_the_claim() {
    let found = actions(&compiled(), &[("ka", "pass")]);
    let expected = json!({"reasons": [{"claim": "a", "value": "true"}], "status": "blocked"});
    assert_eq!(found["tested_false"], expected, "tested_false: {found}");
    assert_eq!(found["negated"], expected, "negated: {found}");
}

/// `all: [a, any: [b, c]]` with `a` FALSE, `b` TRUE, `c` FALSE is FALSE because of `a` alone: the
/// `any` holds through `b`. Only the reasons that decide the status are listed, so `c` is not one.
#[test]
fn a_claim_in_a_branch_that_holds_is_not_a_reason() {
    let found = actions(
        &compiled(),
        &[("ka", "fail"), ("kb", "pass"), ("kc", "fail")],
    );
    assert_eq!(
        found["branch"],
        json!({"reasons": [{"claim": "a", "value": "false"}], "status": "blocked"}),
        "{found}"
    );
}

/// `all: [not a, b]` with `a` FALSE and `b` UNKNOWN is UNKNOWN because of `b` alone: `not a` holds
/// precisely because `a` is FALSE. Naming `a` tells the reader to change the one thing that is
/// already as the precondition wants it.
#[test]
fn a_claim_whose_negation_holds_is_not_a_reason() {
    let found = actions(&compiled(), &[("ka", "fail")]);
    assert_eq!(
        found["guarded"],
        json!({"reasons": [{"claim": "b", "value": "unknown"}], "status": "blocked"}),
        "{found}"
    );
}

/// `actions.rs` module docs: "Capability reasons are in capability order". `evaluate_with` takes
/// any `Ir` a caller builds (its fields are public; `claims.rs` guards a caller-built claim cycle),
/// so the order is the section's to keep, not the compiler's.
#[test]
fn capability_reasons_are_in_capability_order_for_a_caller_built_ir() {
    let mut ir = compiled();
    ir.actions
        .get_mut(&model::ActionId::new("gated"))
        .expect("gated is declared")
        .requires = vec![CapabilityId::new("z"), CapabilityId::new("d")];
    let found = actions(&ir, &[]);
    assert_eq!(
        found["gated"],
        json!({
            "reasons": [
                {"capability": "d", "decision": "none"},
                {"capability": "z", "decision": "none"}
            ],
            "status": "approval-required"
        }),
        "{found}"
    );
}
