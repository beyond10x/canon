//! story:decision-grounds against the evaluator library, over the fixture of CANON-GROUND-001
//! (`fixtures/investigation/decision-grounds.yaml`): what the conformance runner cannot hold,
//! because it compares a refusal by its code only and the explanation only as a whole. The grounds
//! of each entry are recorded in `computed_from` as given; empty grounds are no grounds; the
//! refusals name the entry and the ground; the reason of an evidence ground two claims list with
//! different reasons is the first claim's, in claim-id order; and the order of the explicit
//! decisions does not change the decision.

use std::path::PathBuf;

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, Decision};

/// Read at run time, so a test binary reused from another build directory reads this tree.
fn fixture() -> ir::Ir {
    let root = std::env::var_os("CARGO_MANIFEST_DIR").expect("run through cargo");
    let path = PathBuf::from(root).join("../../fixtures/investigation/decision-grounds.yaml");
    let text = std::fs::read_to_string(&path).expect("the fixture reads");
    ir::compile(&model::parse(&text).expect("the fixture parses")).expect("the fixture compiles")
}

const CASE: &str = "format: canon-case/1\nid: G-1\nprotocol: grounds\nrevision: c2\n\
                    artifacts: {spec: {revision: r2}, up: {revision: u2}}\n";

fn evaluate(supplied: Supplied<'_>) -> Result<Decision, eval::Refusal> {
    let case = eval::read_case(CASE).expect("case reads");
    eval::evaluate_with(&fixture(), &case, &[], supplied)
}

fn refusal(supplied: Supplied<'_>) -> (String, String) {
    let refused = evaluate(supplied).expect_err("refused");
    (refused.code().to_owned(), refused.to_string())
}

/// `computed_from` records each entry's grounds as given, whether or not they hold: the artifact
/// revisions by artifact, the evidence ids in id order, each once; an entry without grounds has
/// neither key.
#[test]
fn computed_from_records_the_grounds_of_each_entry_as_given() {
    let authority = "- {capability: x, decision: granted, principal: p, \
                     ground_revisions: {up: u1, spec: r2}, ground_evidence: [e9, e1, e9]}\n\
                     - {capability: x, decision: denied}\n";
    let decisions = "- {decision: d, outcome: o, principal: q, case_revision: c2, \
                     ground_evidence: [e1]}\n\
                     - {decision: d, outcome: o, principal: p, case_revision: c2, \
                     ground_revisions: {spec: r2}}\n";
    let decision = evaluate(Supplied {
        authority: Some(authority),
        decisions: Some(decisions),
        ..Supplied::default()
    })
    .expect("decides");
    let from = &decision.explanation.as_ref().expect("explained")["computed_from"];
    assert_eq!(
        from["authority"],
        serde_json::json!([
            {"capability": "x", "decision": "denied"},
            {"capability": "x", "decision": "granted", "principal": "p",
             "ground_revisions": {"spec": "r2", "up": "u1"}, "ground_evidence": ["e1", "e9"]},
        ])
    );
    assert_eq!(
        from["decisions"],
        serde_json::json!([
            {"case_revision": "c2", "decision": "d", "outcome": "o", "principal": "p",
             "ground_revisions": {"spec": "r2"}},
            {"case_revision": "c2", "decision": "d", "outcome": "o", "principal": "q",
             "ground_evidence": ["e1"]},
        ])
    );
}

/// Empty grounds are no grounds: an entry with `ground_revisions: {}` and `ground_evidence: []`
/// gives the bytes the same entry without them gives, the explanation included.
#[test]
fn empty_grounds_give_the_bytes_of_no_grounds() {
    let render = |authority: &str, decisions: &str| {
        eval::render(
            &evaluate(Supplied {
                authority: Some(authority),
                decisions: Some(decisions),
                ..Supplied::default()
            })
            .expect("decides"),
        )
    };
    assert_eq!(
        render(
            "- {capability: x, decision: granted, ground_revisions: {}, ground_evidence: []}\n",
            "- {decision: d, outcome: o, principal: p, case_revision: c2, \
             ground_revisions: {}, ground_evidence: []}\n",
        ),
        render(
            "- {capability: x, decision: granted}\n",
            "- {decision: d, outcome: o, principal: p, case_revision: c2}\n",
        )
    );
}

/// A denial set aside by its grounds, with nothing else covering the requirement, leaves it not
/// decided: approval is required, naming the failed ground, not blocked.
#[test]
fn a_set_aside_denial_alone_leaves_the_requirement_not_decided() {
    let decision = evaluate(Supplied {
        authority: Some("- {capability: x, decision: denied, ground_evidence: [e1]}\n"),
        ..Supplied::default()
    })
    .expect("decides");
    assert_eq!(
        decision.actions.expect("actions")["act"],
        serde_json::json!({"status": "approval-required", "reasons": [
            {"capability": "x", "decision": "none",
             "stale_grounds": [{"evidence": "e1", "present": false}]},
        ]})
    );
}

/// The refusals name the input, the entry and the ground: identifiers when the document is read,
/// an undeclared artifact after both documents are read, authority entries first.
#[test]
fn a_refused_ground_is_named_with_its_entry() {
    let message = |code: &str, text: &str| (code.to_owned(), text.to_owned());
    assert_eq!(
        refusal(Supplied {
            authority: Some("- {capability: x, decision: granted, ground_evidence: ['e 1']}\n"),
            ..Supplied::default()
        }),
        message(
            "invalid-identifier",
            "authority ground evidence `e 1` is empty or contains whitespace or a control character"
        )
    );
    assert_eq!(
        refusal(Supplied {
            decisions: Some(
                "- {decision: d, outcome: o, principal: p, case_revision: c2, \
                 ground_revisions: {spec: 'r 1'}}\n"
            ),
            ..Supplied::default()
        }),
        message(
            "invalid-identifier",
            "decision ground revision `r 1` is empty or contains whitespace or a control character"
        )
    );
    assert_eq!(
        refusal(Supplied {
            authority: Some(
                "- {capability: x, decision: granted, ground_revisions: {'s p': r1}}\n"
            ),
            ..Supplied::default()
        }),
        message(
            "invalid-identifier",
            "authority ground artifact `s p` is empty or contains whitespace or a control character"
        )
    );
    let undeclared_authority = "- {capability: x, decision: granted, \
                                ground_revisions: {spec: r2, nowhere: r1, elsewhere: r1}}\n";
    let undeclared_decision = "- {decision: d, outcome: o, principal: p, case_revision: c2, \
                               ground_revisions: {nowhere: r1}}\n";
    assert_eq!(
        refusal(Supplied {
            authority: Some(undeclared_authority),
            decisions: Some(undeclared_decision),
            ..Supplied::default()
        }),
        message(
            "undeclared-artifact",
            "`--authority` decision on capability `x` rests on artifact `elsewhere`, which the \
             protocol does not declare"
        )
    );
    assert_eq!(
        refusal(Supplied {
            decisions: Some(undeclared_decision),
            ..Supplied::default()
        }),
        message(
            "undeclared-artifact",
            "`--decisions` decision `d` for outcome `o` rests on artifact `nowhere`, which the \
             protocol does not declare"
        )
    );
    // Grounds are not part of the duplicate key, so the message is the one without grounds.
    assert_eq!(
        refusal(Supplied {
            decisions: Some(
                "- {decision: d, outcome: o, principal: p, case_revision: c2, ground_evidence: [e1]}\n\
                 - {decision: d, outcome: o, principal: p, case_revision: c2}\n"
            ),
            ..Supplied::default()
        }),
        message(
            "duplicate-identifier",
            "`--decisions` gives decision `d` for outcome `o` by `p` at case revision `c2` more \
             than once"
        )
    );
}

/// Every order of the explicit decisions gives the same decision, byte for byte.
#[test]
fn the_order_of_the_explicit_decisions_does_not_change_the_decision() {
    let entries = [
        "- {decision: d, outcome: o, principal: p, case_revision: c2, ground_revisions: {spec: r1}}\n",
        "- {decision: d, outcome: o, principal: q, case_revision: c2, ground_revisions: {up: u1}, \
         ground_evidence: [e1]}\n",
        "- {decision: d, outcome: o, principal: r, case_revision: c1, ground_revisions: {spec: r0}}\n",
    ];
    let render = |order: [usize; 3]| {
        let text: String = order.iter().map(|at| entries[*at]).collect();
        eval::render(
            &evaluate(Supplied {
                decisions: Some(&text),
                ..Supplied::default()
            })
            .expect("decides"),
        )
    };
    let first = render([0, 1, 2]);
    assert!(
        first.contains(r#""stale_grounds": ["#),
        "the decisions at c2 are set aside by their grounds: {first}"
    );
    for order in [[0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]] {
        assert_eq!(render(order), first, "{order:?}");
    }
}

/// An evidence ground whose record two claims list with different reasons fails with the reason of
/// the first of them in claim-id order: the claim reading under an invalidation rule lists it as
/// `invalidated`, the one demanding independence as `not_independent`.
#[test]
fn an_evidence_ground_two_claims_list_takes_the_first_claims_reason() {
    let protocol = |invalidated: &str, independent: &str| {
        format!(
            "format: protocol/1\nprotocol: {{id: p, revision: 1}}\n\
             artifacts: {{spec: {{}}, up: {{}}}}\n\
             independence_dimensions: {{other: {{}}}}\n\
             evidence_kinds: {{k: {{}}}}\n\
             claims:\n  \
             {invalidated}: {{true_when: {{evidence: {{kind: k}}}}}}\n  \
             {independent}: {{true_when: {{evidence: {{kind: k, independent_on: [other]}}}}}}\n\
             invalidation:\n  rule: {{upstream: up, invalidates: [{invalidated}]}}\n\
             actions:\n  act: {{requires: [{{capability: x}}]}}\n"
        )
    };
    let record = eval::read_evidence(
        "format: canon-evidence/1\nid: e1\nkind: k\nsubject: spec\nsubject_revision: r2\n\
         producer: someone\nupstream_revisions: {up: u1}\n",
    )
    .expect("evidence reads");
    let case = eval::read_case(
        "format: canon-case/1\nid: G-1\nprotocol: p\nartifacts: {spec: {revision: r2}, \
         up: {revision: u2}}\n",
    )
    .expect("case reads");
    let reason = |invalidated: &str, independent: &str| {
        let ir = ir::compile(&model::parse(&protocol(invalidated, independent)).expect("parses"))
            .expect("compiles");
        let decision = eval::evaluate_with(
            &ir,
            &case,
            std::slice::from_ref(&record),
            Supplied {
                authority: Some("- {capability: x, decision: granted, ground_evidence: [e1]}\n"),
                ..Supplied::default()
            },
        )
        .expect("decides");
        decision.actions.expect("actions")["act"]["reasons"][0]["stale_grounds"].clone()
    };
    assert_eq!(
        reason("a", "b"),
        serde_json::json!([{"evidence": "e1", "reason": "invalidated"}])
    );
    assert_eq!(
        reason("b", "a"),
        serde_json::json!([{"evidence": "e1", "reason": "not_independent"}])
    );
}
