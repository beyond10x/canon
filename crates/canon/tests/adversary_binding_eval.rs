//! Adversary pass 1 for story:evidence-revision-binding, against the evaluator library.
//!
//! `eval/binding.rs` module docs: a record bound to a revision that is not current "neither
//! establishes nor contradicts a claim about the current revision: a claim it alone decided is
//! `unknown`, never `false`". The story's outcome says the same: "Exclusion leaves a claim
//! `UNKNOWN`, never `FALSE`."

use b10x_canon::eval::{self, Refusal};
use b10x_canon::ir;
use b10x_canon::model::{self, Decision, Truth};

const PROTOCOL: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1}\n\
    artifacts: {a: {}, b: {}}\n\
    evidence_kinds: {k: {}, m: {}}\n\
    claims:\n\
      \x20\x20base: {true_when: {evidence: {kind: k, result: pass}}}\n\
      \x20\x20settled: {true_when: {not: {claim: base, is: unknown}}}\n\
      \x20\x20open: {true_when: {claim: base, is: unknown}}\n\
      \x20\x20other: {true_when: {evidence: {kind: m}}}\n";

fn case(a: &str, b: &str) -> String {
    format!(
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {{a: {{revision: {a}}}, b: {{revision: {b}}}}}\n"
    )
}

fn record(id: &str, kind: &str, subject: &str, revision: &str) -> String {
    format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\nresult: pass\nsubject: {subject}\nsubject_revision: {revision}\n"
    )
}

fn decide(case: &str, evidence: &[String]) -> Result<Decision, Refusal> {
    let ir = ir::compile(&model::parse(PROTOCOL).expect("parses")).expect("compiles");
    let case = eval::read_case(case)?;
    let evidence = evidence
        .iter()
        .map(|text| eval::read_evidence(text))
        .collect::<Result<Vec<_>, _>>()?;
    eval::evaluate(&ir, &case, &evidence)
}

fn value(decision: &Decision, claim: &str) -> Truth {
    decision
        .claims
        .iter()
        .find(|(id, _)| id.as_str() == claim)
        .map(|(_, entry)| entry.value)
        .unwrap_or_else(|| panic!("claim `{claim}` decided"))
}

/// `settled` is decided by `e1` alone: TRUE while `e1` is bound to the current revision of `a`.
/// Advance only the case snapshot and the binding docs promise `unknown`, never `false`.
#[test]
fn a_claim_the_stale_record_alone_decided_is_never_false() {
    let evidence = [record("e1", "k", "a", "r1")];
    let current = decide(&case("r1", "r1"), &evidence).expect("decides");
    assert_eq!(value(&current, "settled"), Truth::True, "precondition");
    assert_eq!(value(&current, "open"), Truth::False, "precondition");

    let advanced = decide(&case("r2", "r1"), &evidence).expect("decides");
    let exclusions: Vec<&str> = advanced
        .claims
        .iter()
        .find(|(id, _)| id.as_str() == "settled")
        .expect("settled")
        .1
        .excluded_evidence
        .iter()
        .map(|exclusion| exclusion.evidence.as_str())
        .collect();
    assert_eq!(exclusions, ["e1"], "e1 is excluded under `settled`");
    // Coordinator decision on adversary pass 1, finding F1: the behaviour is right and the old
    // doc was wrong. Excluding `e1` makes the evidence match on it `unknown`; `settled` and
    // `open` test `base` for `is: unknown`, so they are decided by that `unknown`: `settled`
    // goes TRUE -> FALSE and `open` FALSE -> TRUE.
    assert_eq!(
        value(&advanced, "base"),
        Truth::Unknown,
        "binding.rs docs: an evidence match only the excluded record decided is `unknown`"
    );
    assert_eq!(
        value(&advanced, "settled"),
        Truth::False,
        "binding.rs docs: a claim testing `is: unknown` is decided by that `unknown`"
    );
    assert_eq!(
        value(&advanced, "open"),
        Truth::True,
        "binding.rs docs: a claim testing `is: unknown` is decided by that `unknown`"
    );
}

/// Probe (expected green): two artifacts at different revisions, records in every order; the
/// rendered decision does not depend on order, and each record is checked against its own
/// subject's current revision.
#[test]
fn several_artifacts_at_different_revisions_bind_each_record_to_its_own_subject() {
    let records = [
        record("e1", "k", "a", "r2"),
        record("e2", "k", "b", "r2"),
        record("e3", "m", "b", "r1"),
        record("e4", "m", "a", "r1"),
    ];
    let mut renders = Vec::new();
    for rotation in 0..records.len() {
        let mut ordered = records.to_vec();
        ordered.rotate_left(rotation);
        ordered.swap(0, 1);
        let decision = decide(&case("r2", "r1"), &ordered).expect("decides");
        renders.push(eval::render(&decision));
    }
    assert!(renders.windows(2).all(|pair| pair[0] == pair[1]));
    let decision = decide(&case("r2", "r1"), &records).expect("decides");
    assert_eq!(value(&decision, "base"), Truth::True);
    assert_eq!(value(&decision, "other"), Truth::True);
    let under = |claim: &str| -> Vec<String> {
        decision
            .claims
            .iter()
            .find(|(id, _)| id.as_str() == claim)
            .expect("claim")
            .1
            .excluded_evidence
            .iter()
            .map(|exclusion| exclusion.evidence.as_str().to_owned())
            .collect()
    };
    assert_eq!(under("base"), ["e2"]);
    assert_eq!(under("settled"), ["e2"]);
    assert_eq!(under("other"), ["e4"]);
}

/// Probe (expected green): a record both undeclared and stale is refused, not excluded, wherever
/// it sits; a duplicate id is refused before binding runs; a record missing either binding field
/// is malformed.
#[test]
fn refusals_win_over_exclusion_and_missing_binding_fields_are_malformed() {
    let stale = record("s1", "k", "a", "r0");
    let undeclared = record("u1", "k", "nowhere", "r0");
    for order in [
        vec![stale.clone(), undeclared.clone()],
        vec![undeclared.clone(), stale.clone()],
    ] {
        let refusal = decide(&case("r1", "r1"), &order).expect_err("refused");
        assert_eq!(refusal.code(), "undeclared-artifact", "{refusal}");
        assert!(refusal.to_string().contains("`nowhere`"), "{refusal}");
    }
    let refusal = decide(
        &case("r1", "r1"),
        &[
            record("d", "k", "a", "r0"),
            record("d", "k", "nowhere", "r1"),
        ],
    )
    .expect_err("refused");
    assert_eq!(refusal.code(), "duplicate-identifier", "{refusal}");

    for text in [
        "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\n",
        "format: canon-evidence/1\nid: e1\nkind: k\nsubject_revision: r1\n",
    ] {
        let refusal = eval::read_evidence(text).expect_err("malformed");
        assert_eq!(refusal.code(), "malformed-input", "{refusal}");
    }
}
