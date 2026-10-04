//! Adversary cases for story:three-valued-claims, against the evaluator library.
//!
//! Each case states the rule it drives from: design § 8 (three-valued claims), the module docs of
//! `b10x_canon::eval` and `eval::read`, or the story's refusal list.

use b10x_canon::eval::{self, Refusal};
use b10x_canon::model::{self, Decision, Truth};
use b10x_canon::{ir, model::Protocol};

const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n";

fn compiled(source: &str) -> ir::Ir {
    ir::compile(&model::parse(source).expect("protocol parses")).expect("protocol compiles")
}

fn record(id: &str, kind: &str, result: &str) -> String {
    format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\nresult: {result}\nsubject: a\nsubject_revision: r1\n"
    )
}

fn decide(ir: &ir::Ir, case: &str, evidence: &[String]) -> Result<Decision, Refusal> {
    let case = eval::read_case(case)?;
    let evidence = evidence
        .iter()
        .map(|text| eval::read_evidence(text))
        .collect::<Result<Vec<_>, _>>()?;
    eval::evaluate(ir, &case, &evidence)
}

fn value(decision: &Decision, claim: &str) -> Truth {
    decision
        .claims
        .iter()
        .find(|(id, _)| id.as_str() == claim)
        .map(|(_, entry)| entry.value)
        .unwrap_or_else(|| panic!("claim {claim} is in the decision"))
}

/// A claim built on another claim, written the way design § 12 writes an outcome's requirement
/// (`claim: explanation.supported`, `is` left at its default).
const LAYERED: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1}\n\
    artifacts: {a: {}}\n\
    evidence_kinds: {tests: {}}\n\
    claims:\n\
    \x20\x20tests.pass: {true_when: {evidence: {kind: tests, result: pass}}}\n\
    \x20\x20release.ready: {true_when: {claim: tests.pass}}\n\
    \x20\x20release.blocked: {true_when: {not: {claim: tests.pass}}}\n";

/// Design § 8: "tests never run -> UNKNOWN", and UNKNOWN and FALSE "must never be collapsed". A
/// claim that is `tests.pass` by reference must be UNKNOWN exactly when `tests.pass` is: with no
/// evidence at all, nothing contradicts it.
#[test]
fn a_claim_defined_as_another_claim_is_unknown_when_that_claim_is_unknown() {
    let ir = compiled(LAYERED);
    let decision = decide(&ir, CASE, &[]).expect("decides");
    assert_eq!(value(&decision, "tests.pass"), Truth::Unknown);
    assert_eq!(
        value(&decision, "release.ready"),
        Truth::Unknown,
        "`release.ready: {{claim: tests.pass}}` with no evidence: FALSE would claim the evidence \
         contradicts it, and there is no evidence"
    );
}

/// Design § 8: "TRUE: applicable evidence establishes the predicate." With an empty evidence set
/// nothing is established, so no claim built from evidence matches and default claim tests can be
/// TRUE.
#[test]
fn with_no_evidence_a_negated_claim_reference_is_not_true() {
    let ir = compiled(LAYERED);
    let decision = decide(&ir, CASE, &[]).expect("decides");
    assert_ne!(
        value(&decision, "release.blocked"),
        Truth::True,
        "`release.blocked: {{not: {{claim: tests.pass}}}}` is TRUE with no evidence at all"
    );
}

/// Inlining a referenced claim's predicate does not change a value: the reference and the
/// predicate it names agree on every evidence set.
#[test]
fn a_claim_reference_agrees_with_the_predicate_it_names_on_every_evidence_set() {
    let ir = compiled(LAYERED);
    for evidence in [
        vec![],
        vec![record("e1", "tests", "pass")],
        vec![record("e1", "tests", "fail")],
        vec![record("e1", "tests", "pass"), record("e2", "tests", "fail")],
    ] {
        let decision = decide(&ir, CASE, &evidence).expect("decides");
        assert_eq!(
            value(&decision, "release.ready"),
            value(&decision, "tests.pass"),
            "evidence {evidence:?}"
        );
    }
}

/// `eval::read` docs: `read_ir` "accepts exactly the bytes `canon compile` prints". A protocol
/// description is free text (no rule restricts it), and `canon-ir/1` writes every character at or
/// above U+0020 as itself; each of these is valid compiler output and must read back.
#[test]
fn every_ir_canon_compile_prints_reads_back() {
    let mut refused = Vec::new();
    for (name, text) in [
        ("line separator U+2028", "first\u{2028}second"),
        ("paragraph separator U+2029", "first\u{2029}second"),
        ("next line U+0085", "first\u{85}second"),
        ("delete U+007F", "first\u{7f}second"),
        ("C1 control U+0080", "first\u{80}second"),
        ("noncharacter U+FFFE", "first\u{fffe}second"),
    ] {
        let mut protocol: Protocol =
            model::parse("format: protocol/1\nprotocol: {id: p, revision: 1}\n").expect("parses");
        protocol.protocol.description = Some(text.to_owned());
        let compiled = ir::compile(&protocol).expect("compiles");
        match eval::read_ir(&compiled.canonical_json()) {
            Ok(read) if read == compiled => {}
            Ok(_) => refused.push(format!("{name}: read back as a different IR")),
            Err(refusal) => refused.push(format!("{name}: {}: {refusal}", refusal.code())),
        }
    }
    assert!(
        refused.is_empty(),
        "IR printed by canon compile is refused by read_ir:\n  {}",
        refused.join("\n  ")
    );
}

/// `eval::read` docs: "`case_from_value` and `evidence_from_value` read one already parsed as
/// YAML, as a conformance scenario holds it". `canon evaluate` reads text, `canon conform run`
/// reads values; one document must get one answer on both paths.
#[test]
fn a_document_reads_the_same_from_text_and_from_a_value() {
    for case in [
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: 1}}\n",
        "format: canon-case/1\nid: 18\nprotocol: p\nartifacts: {a: {revision: r1}}\n",
    ] {
        let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(case).expect("yaml");
        assert_eq!(
            eval::read_case(case).is_ok(),
            eval::case_from_value(&value).is_ok(),
            "case {case:?}: text gives {:?}, value gives {:?}",
            eval::read_case(case),
            eval::case_from_value(&value)
        );
    }
    let evidence = "format: canon-evidence/1\nid: e1\nkind: tests\nresult: 1\nsubject: a\nsubject_revision: 2\n";
    let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(evidence).expect("yaml");
    assert_eq!(
        eval::read_evidence(evidence).is_ok(),
        eval::evidence_from_value(&value).is_ok(),
        "evidence: text gives {:?}, value gives {:?}",
        eval::read_evidence(evidence),
        eval::evidence_from_value(&value)
    );
}

/// The refusal list in the `eval` module docs checks every identifier of a case and of an
/// evidence record. The unit's own refusal test covers the case id, an artifact revision and the
/// subject revision only; these are the others, each of which would otherwise be evaluated.
#[test]
fn every_identifier_the_refusal_list_names_is_checked() {
    let ir = compiled(LAYERED);
    let good = record("e1", "tests", "pass");
    for (case, evidence, message) in [
        (
            CASE.replace("protocol: p", "protocol: 'p q'"),
            vec![],
            "case protocol identifier `p q`",
        ),
        (
            CASE.replace("a: {revision: r1}", "'a b': {revision: r1}"),
            vec![],
            "case artifact identifier `a b`",
        ),
        (
            CASE.to_owned(),
            vec![good.replace("id: e1", "id: 'e 1'")],
            "evidence identifier `e 1`",
        ),
        (
            CASE.to_owned(),
            vec![good.replace("kind: tests", "kind: 'te sts'")],
            "evidence `e1` kind identifier `te sts`",
        ),
        (
            CASE.to_owned(),
            vec![good.replace("subject: a", "subject: 'a b'")],
            "evidence `e1` subject identifier `a b`",
        ),
    ] {
        let refusal = decide(&ir, &case, &evidence).expect_err(message);
        assert_eq!(refusal.code(), "invalid-identifier", "{message}: {refusal}");
        assert!(
            refusal.to_string().starts_with(message),
            "{message}: {refusal}"
        );
    }
}

/// A misspelt `result` must not turn a refutation into a record without a result: the evidence
/// record's fields are closed, as the case's are.
#[test]
fn an_evidence_record_with_an_unknown_field_is_refused() {
    let typo = record("e1", "tests", "refuted").replace("result:", "reslut:");
    let refusal = eval::read_evidence(&typo).expect_err("an unknown field is refused");
    assert_eq!(refusal.code(), "malformed-input", "{refusal}");
    let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(&typo).expect("yaml");
    assert_eq!(
        eval::evidence_from_value(&value).map_err(|r| r.code()),
        Err("malformed-input")
    );
}
