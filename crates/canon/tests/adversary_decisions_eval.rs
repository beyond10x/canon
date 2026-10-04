//! Adversary pass 1 for story:decision-outcomes (wave 2026-10-04-w8), against the evaluator
//! library: the `canon-decisions/1` input, the case snapshot's `revision`, and the `decision`
//! requirement through parse, validate, compile and `read_ir`.
//!
//! The contract read is the module docs of `eval` and `eval/decisions.rs`: refusals checked in the
//! order "case id, protocol id, the case revision when given, then each artifact id and revision";
//! a decision applies when its name, outcome and case revision all match; the same decision given
//! twice applies once; a snapshot without a revision has no decision applying; a `termination`
//! through a blocked outcome is refused as `illegitimate-termination` (decision-blocker
//! terminated-case-reevaluation, option A).

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, Json};

/// `done` requires the claim `c`; `inconclusive` and `declined` both require the decision `stop`.
const PROTOCOL: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
    evidence_kinds: {k: {}}\n\
    claims: {c: {true_when: {evidence: {kind: k}}}}\n\
    outcomes:\n\
    \x20\x20done: {requires: {claim: c}}\n\
    \x20\x20inconclusive: {requires: {decision: stop}}\n\
    \x20\x20declined: {requires: {decision: stop}}\n";

fn compiled() -> ir::Ir {
    ir::compile(&model::parse(PROTOCOL).expect("protocol parses")).expect("protocol compiles")
}

fn case(extra: &str) -> model::Case {
    eval::read_case(&format!(
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {{a: {{revision: r1}}}}\n{extra}"
    ))
    .expect("case reads")
}

fn outcomes(case: &model::Case, decisions: &str) -> Json {
    eval::evaluate_with(
        &compiled(),
        case,
        &[],
        Supplied {
            decisions: Some(decisions),
            ..Supplied::default()
        },
    )
    .expect("evaluates")
    .outcomes
    .expect("outcomes section")
}

fn status(outcomes: &Json, outcome: &str) -> String {
    outcomes[outcome]["status"]
        .as_str()
        .unwrap_or_else(|| panic!("{outcome}: no status in {outcomes}"))
        .to_owned()
}

/// The case snapshot's `revision` is checked as an identifier, after the protocol id and before
/// the artifacts (module docs of `eval`, "Refusals"). Nothing else in the suite gives a revision
/// that is not one: delete the check in `eval/case.rs` and every other test stays green, while a
/// snapshot whose revision no decision can ever name (`case_revision` is identifier-checked) is
/// accepted silently.
#[test]
fn a_case_revision_that_is_not_an_identifier_is_refused_in_its_place() {
    let ir = compiled();
    let refused = |text: &str| {
        let case = eval::read_case(text).expect("case reads");
        eval::evaluate(&ir, &case, &[]).expect_err(text)
    };
    let refusal = refused(
        "format: canon-case/1\nid: C-1\nprotocol: p\nrevision: 'c 2'\nartifacts: {a: {revision: r1}}\n",
    );
    assert_eq!(refusal.code(), "invalid-identifier", "{refusal}");
    assert!(
        refusal.to_string().starts_with("case revision "),
        "{refusal}"
    );
    // After the protocol id ...
    let refusal = refused(
        "format: canon-case/1\nid: C-1\nprotocol: 'p q'\nrevision: 'c 2'\nartifacts: {a: {revision: r1}}\n",
    );
    assert!(
        refusal.to_string().starts_with("case protocol identifier "),
        "{refusal}"
    );
    // ... and before the artifacts.
    let refusal = refused(
        "format: canon-case/1\nid: C-1\nprotocol: p\nrevision: 'c 2'\nartifacts: {'a b': {revision: r1}}\n",
    );
    assert!(
        refusal.to_string().starts_with("case revision "),
        "{refusal}"
    );
}

/// Several decisions for one outcome and one decision name taken for two outcomes that both
/// require it: each outcome is legitimate exactly when an entry names it at the current revision,
/// and nothing else changes. Changed by the coordinator's decision on finding F5: an exact
/// duplicate entry is refused as `duplicate-identifier`, and a decision for an outcome that does
/// not require it as `undeclared-decision`, where both were accepted silently before.
#[test]
fn several_duplicate_and_shared_decisions_each_apply_only_to_the_outcome_they_name() {
    let at_c2 = case("revision: c2\n");
    let decided = outcomes(
        &at_c2,
        "[{decision: stop, outcome: inconclusive, principal: p, case_revision: c1},\n \
          {decision: stop, outcome: inconclusive, principal: p, case_revision: c2},\n \
          {decision: stop, outcome: inconclusive, principal: q, case_revision: c2}]",
    );
    assert_eq!(status(&decided, "inconclusive"), "legitimate");
    assert_eq!(
        decided["declined"],
        serde_json::json!({"status": "blocked", "reasons": [{"decision": "stop", "present": false}]})
    );
    let both = outcomes(
        &at_c2,
        "[{decision: stop, outcome: declined, principal: p, case_revision: c2},\n \
          {decision: stop, outcome: inconclusive, principal: p, case_revision: c3}]",
    );
    assert_eq!(status(&both, "declined"), "legitimate");
    assert_eq!(status(&both, "inconclusive"), "blocked");
    // A decision for the claim-based outcome, and an exact duplicate entry, are refused.
    for (given, code) in [
        (
            "[{decision: stop, outcome: done, principal: p, case_revision: c2}]",
            "undeclared-decision",
        ),
        (
            "[{decision: stop, outcome: inconclusive, principal: p, case_revision: c2},\n \
              {decision: stop, outcome: inconclusive, principal: p, case_revision: c2}]",
            "duplicate-identifier",
        ),
    ] {
        let refusal = eval::evaluate_with(
            &compiled(),
            &at_c2,
            &[],
            Supplied {
                decisions: Some(given),
                ..Supplied::default()
            },
        )
        .expect_err(given);
        assert_eq!(refusal.code(), code, "{given}: {refusal}");
    }
}

/// A snapshot with no `revision` has no decision applying, whatever revision the decision names,
/// and a termination through a decided outcome is then refused; so is one at a superseded
/// revision (option A).
#[test]
fn without_a_current_revision_or_at_a_superseded_one_a_decided_termination_is_refused() {
    let given = "[{decision: stop, outcome: inconclusive, principal: p, case_revision: c2}]";
    assert_eq!(
        status(&outcomes(&case(""), given), "inconclusive"),
        "blocked"
    );
    for extra in [
        "termination: inconclusive\n",
        "revision: c3\ntermination: inconclusive\n",
    ] {
        let refusal = eval::evaluate_with(
            &compiled(),
            &case(extra),
            &[],
            Supplied {
                decisions: Some(given),
                ..Supplied::default()
            },
        )
        .expect_err(extra);
        assert_eq!(
            refusal.code(),
            "illegitimate-termination",
            "{extra}: {refusal}"
        );
    }
    let decided = eval::evaluate_with(
        &compiled(),
        &case("revision: c2\ntermination: inconclusive\n"),
        &[],
        Supplied {
            decisions: Some(given),
            ..Supplied::default()
        },
    )
    .expect("a termination through a decided outcome at its revision is evaluated");
    assert_eq!(
        status(&decided.outcomes.expect("outcomes"), "inconclusive"),
        "legitimate"
    );
}

/// The IR `canon compile` prints for a decision requirement reads back as the same IR, and an IR
/// that puts `decision` beside another key, or inside `all`, `any` or `not`, or in a claim, is
/// refused.
#[test]
fn a_decision_requirement_round_trips_through_read_ir_and_stands_alone() {
    let ir = compiled();
    let text = ir.canonical_json();
    assert!(
        text.contains("\"requires\": {\n        \"decision\": \"stop\"\n      }"),
        "{text}"
    );
    assert_eq!(eval::read_ir(&text), Ok(ir));
    let inconclusive = "\"requires\": {\n        \"decision\": \"stop\"\n      }";
    for replacement in [
        "\"requires\": {\n        \"decision\": \"stop\",\n        \"any\": []\n      }",
        "\"requires\": {\n        \"all\": [{\"decision\": \"stop\"}]\n      }",
        "\"requires\": {\n        \"not\": {\"decision\": \"stop\"}\n      }",
        "\"requires\": {\n        \"decision\": 7\n      }",
    ] {
        let edited = text.replacen(inconclusive, replacement, 1);
        assert_ne!(edited, text, "{replacement}");
        let refusal = eval::read_ir(&edited).expect_err(replacement);
        assert_eq!(
            refusal.code(),
            "malformed-input",
            "{replacement}: {refusal}"
        );
    }
    let claim = "\"true_when\": {\n        \"evidence\": {\n          \"kind\": \"k\",\n          \"result\": null\n        }\n      }";
    assert!(text.contains(claim), "{text}");
    let edited = text.replacen(claim, "\"true_when\": {\"decision\": \"stop\"}", 1);
    assert_eq!(
        eval::read_ir(&edited).expect_err("a claim").code(),
        "malformed-input"
    );
    for source in [
        "obligations: {o: {discharged_when: {decision: stop}}}\n",
        "actions: {x: {precondition: {decision: stop}}}\n",
        "outcomes: {o: {requires: {any: [{decision: stop}]}}}\n",
        "outcomes: {o: {requires: {decision: stop, is: true}}}\n",
        "outcomes: {o: {requires: {decision: [stop]}}}\n",
    ] {
        let text = format!("format: protocol/1\nprotocol: {{id: p, revision: 1}}\n{source}");
        assert!(model::parse(&text).is_err(), "{source} parses");
    }
}
