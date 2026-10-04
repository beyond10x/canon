//! Adversary pass 2 for story:outcomes, against the evaluator library: the polarity walk that
//! names a blocked outcome's reasons, read against the module docs of `eval/outcomes.rs`.
//!
//! Those docs are the contract: walking from `requires` with the wanted value `true`, `not` flips
//! the wanted value, `all` and `any` descend only into the members whose value is not the wanted
//! one (and, since story:review-hardening-w7, the action rule: a connective that has members of
//! the opposite value is decided by those alone), and a claim test or evidence match whose own
//! value is not the wanted one is a reason; each
//! reason is named once; claims come first in claim-id order, then evidence kinds in kind order;
//! `{"requirement": "unsatisfiable"}` only when no test is named; a legitimate outcome carries no
//! reasons. Termination is refused last, after every other refusal (module docs of `eval`).
//!
//! `canon compile` sorts the members of every `all` and `any` (claims before evidence, each by
//! identifier) and drops repeated members, so a flat requirement cannot tell claim-id order from
//! the order the walk visits tests, nor deduplication from compilation. The cases below nest the
//! tests so that those orders differ.

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, ClaimId, ClaimTest, Json, OutcomeId, Predicate, Truth};

const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n";

/// `c` is `true` with a `k` record whose result is `pass`, `false` with one whose result is
/// `fail`, `unknown` without one; `d` is `true` with any `l` record and `unknown` without one.
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

fn blocked(reasons: Json) -> Json {
    serde_json::json!({"status": "blocked", "reasons": reasons})
}

/// `not` flips the wanted value each time it is passed, so an even number of `not`s over a test
/// wants what the bare test wants, and an odd number wants what `is: false` wants. A walk that
/// flips only once (wanted `false` stays `false` under a second `not`) names nothing here and
/// reports the requirement unsatisfiable.
#[test]
fn nested_negations_name_the_reason_the_equivalent_plain_test_names() {
    let ir = protocol(
        "  plain: {requires: {claim: c}}\n\
         \x20\x20twice: {requires: {not: {not: {claim: c}}}}\n\
         \x20\x20tested_false: {requires: {claim: c, is: false}}\n\
         \x20\x20thrice: {requires: {not: {not: {not: {claim: c}}}}}\n",
    );
    let refuted = outcomes(&ir, &[record("k", Some("fail"))]);
    assert_eq!(
        refuted["plain"],
        blocked(serde_json::json!([{"claim": "c", "value": "false"}])),
        "the control: {refuted:#}"
    );
    assert_eq!(refuted["twice"], refuted["plain"], "{refuted:#}");
    let passed = outcomes(&ir, &[record("k", Some("pass"))]);
    assert_eq!(
        passed["tested_false"],
        blocked(serde_json::json!([{"claim": "c", "value": "true"}])),
        "the control: {passed:#}"
    );
    assert_eq!(passed["thrice"], passed["tested_false"], "{passed:#}");
}

/// An `any` under a `not` must be `false`, so the walk descends into its members that are not
/// `false`: the `true` ones. `c` is `false` (what the negation wants) and is not named; the
/// matched `l` record is. A walk that descends into members that are not `true` regardless of
/// polarity names `c` instead.
#[test]
fn an_any_inside_a_not_inside_an_all_names_only_the_members_that_are_not_false() {
    let ir = protocol(
        "  deep: {requires: {all: [{not: {any: [{claim: c}, {evidence: {kind: l}}]}}, {claim: d}]}}\n",
    );
    let decided = outcomes(&ir, &[record("k", Some("fail")), record("l", None)]);
    assert_eq!(
        decided["deep"],
        // Coordinator decision, adversary pass 2 item 1: an evidence reason states `present`.
        blocked(serde_json::json!([{"evidence": "l", "present": true}])),
        "{decided:#}"
    );
}

/// `not: {all: [...]}` and `not: {any: [...]}` want their connective `false`: an `all` is blocked
/// by every member that is not `false`; an `any` by its `true` members when it has any, and
/// otherwise by every member that is not `false`; and a member that is already `false` is not
/// named.
#[test]
fn a_negated_connective_names_the_members_that_are_not_false() {
    let ir = protocol(
        "  not_all: {requires: {not: {all: [{claim: c}, {claim: d}]}}}\n\
         \x20\x20not_any: {requires: {not: {any: [{claim: c}, {evidence: {kind: l}}]}}}\n",
    );
    let both = outcomes(&ir, &[record("k", Some("pass")), record("l", None)]);
    assert_eq!(
        both["not_all"],
        blocked(serde_json::json!([
            {"claim": "c", "value": "true"},
            {"claim": "d", "value": "true"},
        ])),
        "{both:#}"
    );
    // Story review-hardening-w7: outcome reasons follow the action rule. Under `not`, an `any`
    // that is `true` is decided only by its `true` members; the `unknown` `l` match beside `c`
    // is not a reason.
    let only_c = outcomes(&ir, &[record("k", Some("pass"))]);
    assert_eq!(
        only_c["not_any"],
        blocked(serde_json::json!([{"claim": "c", "value": "true"}])),
        "{only_c:#}"
    );
}

/// A negated evidence match that is met (`l` was observed) keeps `not` from being `true`, so it is
/// named by its kind; a walk that names only matches which are not `true` names nothing and calls
/// the requirement unsatisfiable.
#[test]
fn a_negated_evidence_match_that_is_met_is_named_by_its_kind() {
    let ir = protocol("  unobserved: {requires: {not: {evidence: {kind: l}}}}\n");
    let decided = outcomes(&ir, &[record("l", None)]);
    assert_eq!(
        decided["unobserved"],
        // Coordinator decision, adversary pass 2 item 1: an evidence reason states `present`.
        blocked(serde_json::json!([{"evidence": "l", "present": true}])),
        "{decided:#}"
    );
}

/// One claim tested with different `is` values, in different branches the compiler cannot merge,
/// is named once, with its own value.
#[test]
fn a_claim_tested_with_different_is_values_is_named_once() {
    let ir = protocol(
        "  split: {requires: {all: [{claim: c}, {not: {claim: c, is: unknown}}]}}\n\
         \x20\x20either: {requires: {any: [{claim: c, is: true}, {claim: c, is: false}]}}\n\
         \x20\x20contradicts: {requires: {all: [{claim: c, is: true}, {claim: c, is: unknown}]}}\n",
    );
    let open = outcomes(&ir, &[]);
    let c_unknown = blocked(serde_json::json!([{"claim": "c", "value": "unknown"}]));
    assert_eq!(open["split"], c_unknown, "{open:#}");
    assert_eq!(open["either"], c_unknown, "{open:#}");
    assert_eq!(open["contradicts"], c_unknown, "{open:#}");
    let passed = outcomes(&ir, &[record("k", Some("pass"))]);
    assert_eq!(
        passed["contradicts"],
        blocked(serde_json::json!([{"claim": "c", "value": "true"}])),
        "{passed:#}"
    );
}

/// Claims first in claim-id order, then evidence kinds in kind order, whatever order the walk
/// reaches them in. Compiled, `all` lists its `any` member first, then `y`, then `a`; the `any`
/// lists `z`, then `m`. So the walk meets `z`, `m`, `y`, `a`; sorted by name alone the reasons are
/// `a`, `m`, `y`, `z`; the contract is `y`, `z`, `a`, `m`.
#[test]
fn reasons_are_claims_in_claim_id_order_then_evidence_kinds_in_kind_order() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {a: {}, m: {}}\n\
         claims:\n\
         \x20\x20y: {true_when: {evidence: {kind: a}}}\n\
         \x20\x20z: {true_when: {evidence: {kind: m}}}\n\
         outcomes:\n\
         \x20\x20o: {requires: {all: [{any: [{evidence: {kind: m}}, {claim: z}]}, {evidence: {kind: a}}, {claim: y}]}}\n";
    let ir = ir::compile(&model::parse(source).expect("parses")).expect("compiles");
    let decided = outcomes(&ir, &[]);
    assert_eq!(
        decided["o"],
        blocked(serde_json::json!([
            {"claim": "y", "value": "unknown"},
            {"claim": "z", "value": "unknown"},
            // Coordinator decision, adversary pass 2 item 1: an evidence reason states `present`.
            {"evidence": "a", "present": false},
            {"evidence": "m", "present": false},
        ])),
        "{decided:#}"
    );
}

/// A legitimate outcome is `{"status": "legitimate"}` and nothing else, even when a member of its
/// requirement is not `true`.
#[test]
fn a_legitimate_outcome_carries_no_reasons() {
    let ir = protocol(
        "  met: {requires: {any: [{claim: c}, {claim: d}, {not: {evidence: {kind: l}}}]}}\n",
    );
    let decided = outcomes(&ir, &[record("k", Some("pass"))]);
    assert_eq!(
        decided["met"],
        serde_json::json!({"status": "legitimate"}),
        "{decided:#}"
    );
}

/// The outcome refusals come last: a claim cycle and a malformed input are refused before a
/// termination is looked at, and a termination through a legitimate outcome is evaluated while
/// another declared outcome is blocked.
#[test]
fn termination_refusals_come_after_every_other_refusal() {
    let terminated = |outcome: &str| {
        eval::read_case(&format!("{CASE}termination: {outcome}\n")).expect("case reads")
    };
    let mut cyclic = protocol("  done: {requires: {claim: c}}\n");
    cyclic
        .claims
        .get_mut(&ClaimId::new("c"))
        .expect("declared")
        .true_when = Predicate::Claim(ClaimTest {
        claim: ClaimId::new("c"),
        is: Truth::True,
    });
    for outcome in ["abandoned", "done"] {
        let refusal = eval::evaluate(&cyclic, &terminated(outcome), &[]).expect_err(outcome);
        assert_eq!(refusal.code(), "claim-cycle", "{outcome}: {refusal}");
    }
    // Changed by story:decision-outcomes, which reads `canon-decisions/1`: text that is not a list
    // of decisions is refused as `malformed-input`, still before the termination is checked. The
    // skeleton refused any decisions as `unsupported-input`.
    let ir = protocol("  done: {requires: {claim: c}}\n  other: {requires: {claim: d}}\n");
    for outcome in ["abandoned", "done"] {
        let refusal = eval::evaluate_with(
            &ir,
            &terminated(outcome),
            &[],
            Supplied {
                decisions: Some("format: canon-decisions/1\n"),
                ..Supplied::default()
            },
        )
        .expect_err(outcome);
        assert_eq!(refusal.code(), "malformed-input", "{outcome}: {refusal}");
    }
    let decided = eval::evaluate(&ir, &terminated("done"), &[record("k", Some("pass"))])
        .expect("a termination through a legitimate outcome is evaluated")
        .outcomes
        .expect("outcomes");
    assert_eq!(
        decided["other"],
        blocked(serde_json::json!([{"claim": "d", "value": "unknown"}]))
    );
}

/// A requirement nested 4000 `not`s deep (an even number) over `c` is evaluated from the caller's
/// thread like the claims are (module docs of `eval`, "Depth"), and names `c` as the bare test
/// does.
#[test]
fn a_deeply_negated_requirement_is_walked_from_any_thread() {
    let mut ir = protocol("  deep: {requires: {claim: c}}\n");
    let mut requires = Predicate::Claim(ClaimTest {
        claim: ClaimId::new("c"),
        is: Truth::True,
    });
    for _ in 0..4000 {
        requires = Predicate::Not(Box::new(requires));
    }
    ir.outcomes
        .get_mut(&OutcomeId::new("deep"))
        .expect("declared")
        .requires = requires.into();
    let decided = outcomes(&ir, &[record("k", Some("fail"))]);
    assert_eq!(
        decided["deep"],
        blocked(serde_json::json!([{"claim": "c", "value": "false"}]))
    );
    // Dropping the chain recurses once per level; drop it where the evaluator ran its walk.
    std::thread::Builder::new()
        .stack_size(64 << 20)
        .spawn(move || drop(ir))
        .expect("thread")
        .join()
        .expect("drops");
}
