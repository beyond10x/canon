//! Adversary pass 2 for story:evidence-freshness: the freshness stage over an IR the compiler did
//! not produce, and `--at` with no record carrying `observed_at`.
//!
//! `eval/mod.rs` treats an IR "which `canon compile` never produces, but a caller can build" as in
//! scope: a claim cycle built that way is refused, not evaluated. `ir::EvidenceKind::max_age` is
//! documented "Well-formed: the validator refuses a maximum age `Age::seconds` does not read", and
//! `freshness::exclude` relies on it. `Ir` and `ir::EvidenceKind` have public fields, so a library
//! caller can put any `Age` there.

use b10x_canon::eval::{self, Refusal, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, Age, Decision, EvidenceKindId};

const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n";

fn compiled() -> ir::Ir {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
                  evidence_kinds: {k: {max_age: 1h}, l: {}}\n\
                  claims: {c: {true_when: {evidence: {kind: k}}}, d: {true_when: {evidence: {kind: l}}}}\n";
    ir::compile(&model::parse(source).expect("protocol parses")).expect("protocol compiles")
}

fn record(id: &str, kind: &str, observed_at: Option<&str>) -> String {
    let observed = observed_at.map_or(String::new(), |at| format!("observed_at: {at}\n"));
    format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\nsubject: a\nsubject_revision: r1\n{observed}"
    )
}

fn decide(ir: &ir::Ir, evidence: &[String], at: Option<&str>) -> Result<Decision, Refusal> {
    let case = eval::read_case(CASE)?;
    let evidence = evidence
        .iter()
        .map(|text| eval::read_evidence(text))
        .collect::<Result<Vec<_>, _>>()?;
    eval::evaluate_with(
        ir,
        &case,
        &evidence,
        Supplied {
            at,
            ..Supplied::default()
        },
    )
}

/// A caller-built IR whose `max_age` is not an age (`1 hour`). A record observed ten years before
/// the instant is either refused with the IR, or expired; today it silently applies, so the claim
/// is `true` on evidence the protocol author meant to expire. Fail-open.
#[test]
fn a_caller_built_ir_with_a_malformed_max_age_does_not_silently_disable_expiry() {
    let mut ir = compiled();
    ir.evidence_kinds
        .get_mut(&EvidenceKindId::new("k"))
        .expect("kind k is compiled")
        .max_age = Some(Age::new("1 hour"));
    let result = decide(
        &ir,
        &[record("old", "k", Some("2016-10-04T12:00:00Z"))],
        Some("2026-10-04T12:00:00Z"),
    );
    match result {
        Err(_) => {}
        Ok(decision) => panic!(
            "a max_age the evaluator cannot read was ignored, so ten-year-old evidence applied:\n{}",
            eval::render(&decision)
        ),
    }
}

/// `--at` with no record carrying `observed_at`: nothing expires, and the decision is the one
/// given without an instant, byte for byte, although kind `k` declares a maximum age, but for the
/// explanation's record of the instant it was computed at (story:explanation, design § 37).
#[test]
fn an_instant_with_no_stamped_record_changes_nothing() {
    let ir = compiled();
    let evidence = [record("e1", "k", None), record("e2", "l", None)];
    let without = eval::render(&decide(&ir, &evidence, None).expect("decides"));
    let mut with =
        decide(&ir, &evidence, Some("2999-12-31T23:59:59Z")).expect("decides with an instant");
    let from = &mut with
        .explanation
        .as_mut()
        .expect("the decision carries an explanation")["computed_from"];
    assert_eq!(
        from["at"], "2999-12-31T23:59:59Z",
        "the instant is recorded"
    );
    from.as_object_mut()
        .expect("computed_from is an object")
        .remove("at");
    let with = eval::render(&with);
    assert_eq!(with, without);
    assert!(!with.contains("excluded_evidence"), "{with}");
}
