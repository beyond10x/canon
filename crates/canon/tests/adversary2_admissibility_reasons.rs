//! Adversary cases for story:action-admissibility (wave 2026-10-04-w7, pass 2), against the
//! evaluator library: the reasons a `blocked` action gives, on the ground pass 1 did not cover —
//! deep nesting, a claim tested more than once in one precondition, an unmet precondition beside
//! capabilities that are denied or undecided, a precondition no test decides, and a protocol that
//! declares no action.
//!
//! Driven from the `actions.rs` module docs: "Each claim whose test decides that is a reason ...
//! once per claim in claim-id order", "A precondition decided by evidence matches alone gives the
//! one reason `{"precondition": <its value>}`. Authority is not consulted.", and "A protocol that
//! declares no action has no `actions` section."

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, ActionId, ClaimId, ClaimTest, Json, Predicate, Truth};
use serde_json::json;

const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {x: {revision: r1}}\n";

/// Claims `a`, `b`, `c`: each `true` with a `pass` record of its kind, `false` with a `fail` one,
/// `unknown` with none.
const HEAD: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1}\n\
    artifacts: {x: {}}\n\
    evidence_kinds: {ka: {}, kb: {}, kc: {}}\n\
    claims:\n\
    \x20\x20a: {true_when: {evidence: {kind: ka, result: pass}}}\n\
    \x20\x20b: {true_when: {evidence: {kind: kb, result: pass}}}\n\
    \x20\x20c: {true_when: {evidence: {kind: kc, result: pass}}}\n";

fn compiled(actions: &str) -> ir::Ir {
    let text = format!("{HEAD}actions:\n{actions}");
    ir::compile(&model::parse(&text).expect("protocol parses")).expect("protocol compiles")
}

/// Records giving each claim named in `values` that value; a claim not named is `unknown`.
fn records(values: &[(&str, Truth)]) -> Vec<model::EvidenceRecord> {
    values
        .iter()
        .filter_map(|(claim, value)| {
            let result = match value {
                Truth::True => "pass",
                Truth::False => "fail",
                Truth::Unknown => return None,
            };
            Some(
                eval::read_evidence(&format!(
                    "format: canon-evidence/1\nid: {claim}-1\nkind: k{claim}\nresult: {result}\nsubject: x\nsubject_revision: r1\n"
                ))
                .expect("evidence reads"),
            )
        })
        .collect()
}

fn actions_with(ir: &ir::Ir, values: &[(&str, Truth)], authority: Option<&str>) -> Json {
    let case = eval::read_case(CASE).expect("case reads");
    let supplied = Supplied {
        authority,
        ..Supplied::default()
    };
    eval::evaluate_with(ir, &case, &records(values), supplied)
        .expect("evaluates")
        .actions
        .expect("an actions section")
}

fn actions(ir: &ir::Ir, values: &[(&str, Truth)]) -> Json {
    actions_with(ir, values, None)
}

fn blocked_by(claims: &[(&str, &str)]) -> Json {
    let reasons: Vec<Json> = claims
        .iter()
        .map(|(claim, value)| json!({"claim": claim, "value": value}))
        .collect();
    json!({"reasons": reasons, "status": "blocked"})
}

/// A claim tested twice with opposite `is` in one `all` is FALSE or UNKNOWN whatever its value:
/// the claim is named once, with its own value, never twice and never the `precondition`
/// fallback.
#[test]
fn a_claim_tested_twice_in_one_precondition_is_named_once() {
    let ir = compiled("  twice: {precondition: {all: [{claim: a}, {claim: a, is: false}]}}\n");
    for (value, text) in [
        (Truth::True, "true"),
        (Truth::False, "false"),
        (Truth::Unknown, "unknown"),
    ] {
        let found = actions(&ir, &[("a", value)]);
        assert_eq!(
            found["twice"],
            blocked_by(&[("a", text)]),
            "a {text}: {found}"
        );
    }
}

/// `any` over all three `is` values of one claim is TRUE whatever the claim's value: admissible.
/// `all` over `is: unknown` and `is: true` of one claim is never TRUE: blocked, naming the claim.
#[test]
fn a_claim_tested_with_several_is_values() {
    let ir = compiled(
        "  always: {precondition: {any: [{claim: a}, {claim: a, is: false}, {claim: a, is: unknown}]}}\n\
         \x20\x20never: {precondition: {all: [{claim: a, is: unknown}, {claim: a}]}}\n\
         \x20\x20undecided: {precondition: {not: {claim: a, is: unknown}}}\n",
    );
    for (value, text) in [
        (Truth::True, "true"),
        (Truth::False, "false"),
        (Truth::Unknown, "unknown"),
    ] {
        let found = actions(&ir, &[("a", value)]);
        assert_eq!(
            found["always"],
            json!({"status": "admissible"}),
            "a {text}: {found}"
        );
        assert_eq!(
            found["never"],
            blocked_by(&[("a", text)]),
            "a {text}: {found}"
        );
    }
    let found = actions(&ir, &[]);
    assert_eq!(
        found["undecided"],
        blocked_by(&[("a", "unknown")]),
        "{found}"
    );
    let found = actions(&ir, &[("a", Truth::False)]);
    assert_eq!(
        found["undecided"],
        json!({"status": "admissible"}),
        "{found}"
    );
}

/// Polarity through three levels: `not any [not a, all [b, not c]]`. With `a` FALSE, `b` TRUE,
/// `c` FALSE every leaf is the opposite of what is wanted; with `a` UNKNOWN only the `all` branch
/// is opposed, so `a` does not decide and is not named.
#[test]
fn polarity_is_carried_through_nested_not_any_all() {
    let ir = compiled(
        "  nested: {precondition: {not: {any: [{not: {claim: a}}, {all: [{claim: b}, {not: {claim: c}}]}]}}}\n",
    );
    let found = actions(
        &ir,
        &[("a", Truth::False), ("b", Truth::True), ("c", Truth::False)],
    );
    assert_eq!(
        found["nested"],
        blocked_by(&[("a", "false"), ("b", "true"), ("c", "false")]),
        "{found}"
    );
    let found = actions(&ir, &[("b", Truth::True), ("c", Truth::False)]);
    assert_eq!(
        found["nested"],
        blocked_by(&[("b", "true"), ("c", "false")]),
        "{found}"
    );
}

/// Runs `work` on a thread with a large stack: a deep predicate is dropped recursively, which is
/// not the code under test.
fn on_big_stack<T: Send>(work: impl FnOnce() -> T + Send) -> T {
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .stack_size(256 << 20)
            .spawn_scoped(scope, work)
            .expect("spawns")
            .join()
            .expect("does not panic")
    })
}

/// A precondition nested as deep as `MAX_IR_DEPTH` lets an IR be read: 4095 `not`s (odd, so the
/// precondition is `not a`) and 2047 one-member `all`s around `{claim: a, is: false}`. Both are
/// blocked with `a` TRUE, naming `a`, without overflowing the evaluation stack.
#[test]
fn a_precondition_nested_to_the_ir_depth_bound_names_its_claim() {
    on_big_stack(|| {
        let mut ir = compiled("  deep_not: {}\n  deep_all: {}\n");
        let test = |is| {
            Predicate::Claim(ClaimTest {
                claim: ClaimId::new("a"),
                is,
            })
        };
        let mut deep_not = test(Truth::True);
        for _ in 0..4095 {
            deep_not = Predicate::Not(Box::new(deep_not));
        }
        let mut deep_all = test(Truth::False);
        for _ in 0..2047 {
            deep_all = Predicate::All(vec![deep_all]);
        }
        ir.actions
            .get_mut(&ActionId::new("deep_not"))
            .expect("declared")
            .precondition = deep_not;
        ir.actions
            .get_mut(&ActionId::new("deep_all"))
            .expect("declared")
            .precondition = deep_all;
        let started = std::time::Instant::now();
        let found = actions(&ir, &[("a", Truth::True)]);
        let took = started.elapsed();
        assert_eq!(found["deep_not"], blocked_by(&[("a", "true")]), "{found}");
        assert_eq!(found["deep_all"], blocked_by(&[("a", "true")]), "{found}");
        assert!(took.as_secs() < 20, "took {took:?}");
        drop(ir);
    });
}

/// An unmet precondition blocks, and its reasons are the precondition's only: a denied capability
/// is not added, and an undecided one does not make the action `approval-required`.
#[test]
fn an_unmet_precondition_blocks_with_only_its_own_reasons() {
    let ir = compiled(
        "  gated: {precondition: {claim: a}, requires: [{capability: d}, {capability: u}]}\n",
    );
    for authority in [
        None,
        Some("[{capability: d, decision: denied}]"),
        Some("[{capability: d, decision: granted}]"),
        Some("[{capability: d, decision: denied}, {capability: u, decision: denied}]"),
    ] {
        let found = actions_with(&ir, &[("a", Truth::False)], authority);
        assert_eq!(
            found["gated"],
            blocked_by(&[("a", "false")]),
            "{authority:?}: {found}"
        );
        let found = actions_with(&ir, &[], authority);
        assert_eq!(
            found["gated"],
            blocked_by(&[("a", "unknown")]),
            "{authority:?}: {found}"
        );
    }
}

/// `actions.rs` module docs: "A precondition decided by evidence matches alone gives the one
/// reason `{"precondition": <its value>}`", and the coordinator's pass-1 contract: that reason
/// "appears only for evidence-decided preconditions". `{any: []}` and `{not: {all: []}}` are FALSE
/// with no evidence match in them at all; the protocol compiles, and each is reported in the
/// evidence-decided form, so a reader is told evidence decided a precondition that reads none.
#[test]
fn a_precondition_with_no_evidence_match_is_not_reported_as_evidence_decided() {
    let ir = compiled(
        "  never: {precondition: {any: []}}\n  never_either: {precondition: {not: {all: []}}}\n",
    );
    let found = actions(&ir, &[]);
    for action in ["never", "never_either"] {
        assert_eq!(found[action]["status"], json!("blocked"), "{found}");
        let reasons = found[action]["reasons"].as_array().expect("reasons");
        assert!(
            !reasons.is_empty()
                && reasons
                    .iter()
                    .all(|reason| reason.get("precondition").is_none()),
            "{action}: no evidence match decides it, yet: {found}"
        );
    }
}

/// "A protocol that declares no action has no `actions` section": through `evaluate` and the
/// rendered `canon-decision/1`, not only the section function.
#[test]
fn a_protocol_without_actions_renders_no_actions_key() {
    let ir = ir::compile(&model::parse(HEAD).expect("parses")).expect("compiles");
    let case = eval::read_case(CASE).expect("case reads");
    let decision = eval::evaluate_with(
        &ir,
        &case,
        &[],
        Supplied {
            authority: Some("[{capability: d, decision: denied}]"),
            ..Supplied::default()
        },
    )
    .expect("evaluates");
    assert_eq!(decision.actions, None);
    let rendered = eval::render(&decision);
    assert!(!rendered.contains("\"actions\""), "{rendered}");
}
