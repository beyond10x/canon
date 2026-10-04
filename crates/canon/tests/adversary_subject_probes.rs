//! Adversary pass 1 for story:subject-bound-evidence-match: subject-bound matches in the places
//! the scenario does not reach. Each case drives from the eval module docs ("a record about
//! another artifact, even at that artifact's current revision, is not of the match and neither
//! establishes nor contradicts it"; "One that names no subject reads records about any
//! artifact"), the `canon-ir/1` docs (`subject` written only when declared) and the ELS need the
//! story names: a test result about another artifact must not satisfy `tests.pass`.

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, Decision, Truth};

const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\n\
    artifacts: {change: {revision: c1}, docs: {revision: d1}}\n";

fn compile(kinds: &str, sections: &str) -> ir::Ir {
    let source = format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\n\
         artifacts: {{change: {{}}, docs: {{}}}}\nevidence_kinds: {kinds}\n{sections}"
    );
    ir::compile(&model::parse(&source).expect("protocol parses")).expect("protocol compiles")
}

/// A `test_result` record: id, subject, the revision it is bound to, result, observed_at.
fn record(
    id: &str,
    subject: &str,
    revision: &str,
    result: &str,
    observed_at: Option<&str>,
) -> model::EvidenceRecord {
    let observed = observed_at
        .map(|at| format!("observed_at: {at}\n"))
        .unwrap_or_default();
    eval::read_evidence(&format!(
        "format: canon-evidence/1\nid: {id}\nkind: test_result\nresult: {result}\n\
         subject: {subject}\nsubject_revision: {revision}\n{observed}"
    ))
    .expect("evidence reads")
}

fn decide(ir: &ir::Ir, evidence: &[model::EvidenceRecord], at: Option<&str>) -> Decision {
    let case = eval::read_case(CASE).expect("case reads");
    eval::evaluate_with(
        ir,
        &case,
        evidence,
        Supplied {
            at,
            ..Supplied::default()
        },
    )
    .expect("decides")
}

fn claim(decision: &Decision, id: &str) -> (Truth, Vec<(String, &'static str)>) {
    let (_, entry) = decision
        .claims
        .iter()
        .find(|(claim, _)| claim.as_str() == id)
        .unwrap_or_else(|| panic!("claim {id}"));
    (
        entry.value,
        entry
            .excluded_evidence
            .iter()
            .map(|e| (e.evidence.as_str().to_owned(), e.reason.as_str()))
            .collect(),
    )
}

/// The ELS shape: `tests.pass` bound to the change, an obligation discharged by it, an action and
/// an outcome that require it. A passing test result about the docs, at the docs' current
/// revision, discharges nothing, admits nothing and legitimates nothing.
#[test]
fn a_test_result_about_another_artifact_does_not_satisfy_tests_pass_anywhere() {
    let ir = compile(
        "{test_result: {}}",
        "claims:\n  tests.pass: {true_when: {evidence: {kind: test_result, result: pass, subject: change}}}\n\
         obligations:\n  tested: {discharged_when: {claim: tests.pass}}\n\
         actions:\n  merge: {precondition: {claim: tests.pass}}\n\
         outcomes:\n  released: {requires: {claim: tests.pass}}\n",
    );
    let decision = decide(&ir, &[record("t-docs", "docs", "d1", "pass", None)], None);
    assert_eq!(claim(&decision, "tests.pass"), (Truth::Unknown, vec![]));
    assert_eq!(
        decision.obligations,
        Some(serde_json::json!([{"id": "tested", "status": "open"}]))
    );
    assert_eq!(
        decision.actions,
        Some(serde_json::json!({"merge": {"status": "blocked",
            "reasons": [{"claim": "tests.pass", "value": "unknown"}]}}))
    );
    assert_eq!(
        decision.outcomes,
        Some(serde_json::json!({"released": {"status": "blocked",
            "reasons": [{"claim": "tests.pass", "value": "unknown"}]}}))
    );
}

/// Under `not` and `any`, a subject-bound match is decided by its own subject's records only.
#[test]
fn subject_bound_matches_under_not_and_any_read_only_their_subject() {
    let ir = compile(
        "{test_result: {}}",
        "claims:\n  \
         change.not_passing: {true_when: {not: {evidence: {kind: test_result, result: pass, subject: change}}}}\n  \
         any.not_passing: {true_when: {not: {evidence: {kind: test_result, result: pass}}}}\n  \
         either.passes: {true_when: {any: [\
            {evidence: {kind: test_result, result: pass, subject: change}}, \
            {evidence: {kind: test_result, result: pass, subject: docs}}]}}\n  \
         both.pass: {true_when: {all: [\
            {evidence: {kind: test_result, result: pass, subject: change}}, \
            {evidence: {kind: test_result, result: pass, subject: docs}}]}}\n",
    );
    let decision = decide(
        &ir,
        &[
            record("t-change", "change", "c1", "fail", None),
            record("t-docs", "docs", "d1", "pass", None),
        ],
        None,
    );
    let values: Vec<(&str, Truth)> = [
        "change.not_passing",
        "any.not_passing",
        "either.passes",
        "both.pass",
    ]
    .into_iter()
    .map(|id| (id, claim(&decision, id).0))
    .collect();
    assert_eq!(
        values,
        [
            ("change.not_passing", Truth::True),
            ("any.not_passing", Truth::Unknown),
            ("either.passes", Truth::True),
            ("both.pass", Truth::False),
        ]
    );
}

/// Freshness and binding exclusions combined with subject binding: the change's record is expired
/// and the docs' record is bound to a superseded docs revision. The bound claim lists only the
/// change's record; the unbound claim lists both; a claim built on the bound one inherits only its
/// list.
#[test]
fn exclusions_are_listed_only_under_the_matches_that_would_read_them() {
    let ir = compile(
        "{test_result: {max_age: 1h}}",
        "claims:\n  \
         tests.pass: {true_when: {evidence: {kind: test_result, result: pass, subject: change}}}\n  \
         any.pass: {true_when: {evidence: {kind: test_result, result: pass}}}\n  \
         ready: {true_when: {claim: tests.pass}}\n",
    );
    let decision = decide(
        &ir,
        &[
            record(
                "t-change",
                "change",
                "c1",
                "pass",
                Some("2026-10-04T08:00:00Z"),
            ),
            record("t-docs", "docs", "d0", "pass", Some("2026-10-04T11:59:00Z")),
        ],
        Some("2026-10-04T12:00:00Z"),
    );
    let change = ("t-change".to_owned(), "expired");
    let docs = ("t-docs".to_owned(), "revision_mismatch");
    assert_eq!(
        claim(&decision, "tests.pass"),
        (Truth::Unknown, vec![change.clone()])
    );
    assert_eq!(
        claim(&decision, "any.pass"),
        (Truth::Unknown, vec![change.clone(), docs])
    );
    assert_eq!(claim(&decision, "ready"), (Truth::Unknown, vec![change]));
}

/// Two matches differing only in subject stay two members, the IR writes `subject` after
/// `result`, and the text reads back to the same IR. A protocol without a subject writes no
/// `subject` key at all.
#[test]
fn subject_round_trips_through_the_ir_and_is_absent_when_not_declared() {
    let bound = compile(
        "{test_result: {}}",
        "claims:\n  c: {true_when: {any: [\
            {evidence: {kind: test_result, subject: docs}}, \
            {evidence: {kind: test_result, subject: change}}, \
            {evidence: {kind: test_result}}, \
            {evidence: {kind: test_result, subject: change}}]}}\n",
    );
    let text = bound.canonical_json();
    let members: Vec<&str> = text
        .match_indices("\"evidence\": {")
        .map(|(at, _)| &text[at..at + text[at..].find('}').expect("closed") + 1])
        .collect();
    assert_eq!(
        members.len(),
        3,
        "one duplicate dropped, two subjects kept:\n{text}"
    );
    assert!(
        members[0].contains("\"result\": null") && !members[0].contains("subject"),
        "{text}"
    );
    assert!(
        members[1].contains("\"result\": null,") && members[1].contains("\"subject\": \"change\""),
        "{text}"
    );
    assert!(members[2].contains("\"subject\": \"docs\""), "{text}");
    assert_eq!(eval::read_ir(&text), Ok(bound));

    let unbound = compile(
        "{test_result: {}}",
        "claims:\n  c: {true_when: {evidence: {kind: test_result, result: pass}}}\n",
    );
    let text = unbound.canonical_json();
    assert!(!text.contains("subject"), "{text}");
    assert!(
        text.contains("\"evidence\": {\n          \"kind\": \"test_result\",\n          \"result\": \"pass\"\n        }"),
        "{text}"
    );

    let edited = text.replace(
        "\"result\": \"pass\"\n",
        "\"result\": \"pass\",\n          \"subject\": \"nowhere\"\n",
    );
    let refusal = eval::read_ir(&edited).expect_err("an undeclared subject in the IR");
    assert_eq!(refusal.code(), "malformed-input");
    assert!(
        refusal.to_string().contains("undeclared-artifact"),
        "{refusal}"
    );
}
