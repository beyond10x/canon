//! Adversarial cases for story:case-inputs.
//!
//! The obligation rule the story chose (an `applicable_when` that is UNKNOWN keeps the obligation
//! `open` even when its `discharged_when` is TRUE) is checked here against what the same unit
//! states about it elsewhere: the fixture's outcome `finding.published` "requires every obligation
//! that applies discharged", and `canon diff` orders the introduction and removal of an
//! applicability predicate (RELAXATION and TIGHTENING). Each case evaluates real states and
//! compares the obligation statuses with what the unit's own documents say about them.

use std::path::PathBuf;

use b10x_canon::diff::{Category, diff};
use b10x_canon::eval::{evaluate, read_case, read_evidence};
use b10x_canon::ir::{Ir, compile};
use b10x_canon::model::{EvidenceRecord, Json, parse};

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn ir_of(source: &str) -> Ir {
    compile(&parse(source).expect("parses")).expect("compiles")
}

fn ir_at(path: &str) -> Ir {
    let text = std::fs::read_to_string(repository_root().join(path))
        .unwrap_or_else(|error| panic!("{path}: {error}"));
    ir_of(&text)
}

fn record(id: &str, kind: &str, result: Option<&str>) -> EvidenceRecord {
    let result = result.map_or(String::new(), |result| format!("result: {result}\n"));
    read_evidence(&format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\nsubject: explanation\n\
         subject_revision: r1\n{result}"
    ))
    .expect("an evidence record")
}

/// The decision's `obligations` and `outcomes` sections for a case of `investigation` (or `p`)
/// with the artifact `explanation` at `r1` and the given inputs.
fn decide(ir: &Ir, protocol: &str, inputs: &str, evidence: &[EvidenceRecord]) -> (Json, Json) {
    let case = read_case(&format!(
        "format: canon-case/1\nid: C\nprotocol: {protocol}\nartifacts:\n  explanation: {{revision: r1}}\n{inputs}"
    ))
    .expect("a case");
    let decision = evaluate(ir, &case, evidence).expect("evaluates");
    (
        decision.obligations.unwrap_or(Json::Null),
        decision.outcomes.unwrap_or(Json::Null),
    )
}

fn status_of(obligations: &Json, id: &str) -> String {
    obligations
        .as_array()
        .expect("the obligations section is an array")
        .iter()
        .find(|entry| entry["id"].as_str() == Some(id))
        .and_then(|entry| entry["status"].as_str())
        .unwrap_or_else(|| panic!("obligation `{id}` in {obligations}"))
        .to_owned()
}

fn categories_for(from: &Ir, to: &Ir, id: &str) -> Vec<(Category, String)> {
    diff(from, to)
        .expect("one protocol")
        .changes
        .into_iter()
        .filter(|change| change.id == id)
        .map(|change| (change.category, change.to_string()))
        .collect()
}

/// The fixture says `finding.published` is published "with every obligation that applies to the
/// case discharged", and the story's acceptance calls it the outcome "which requires every
/// applicable obligation discharged". With severity `low` and only a falsification attempt on
/// record, `explanation.supported` is UNKNOWN, so `falsification.recorded` is `open` (its
/// applicability is unknown), yet the outcome is `legitimate`. `canon generate` picks exactly this
/// state as the minimal witness of `finding.published`.
#[test]
fn an_open_obligation_does_not_leave_the_outcome_requiring_it_legitimate() {
    let ir = ir_at("fixtures/investigation/case-inputs.yaml");
    let evidence = [record("e1", "falsification_attempt", None)];
    let (obligations, outcomes) = decide(
        &ir,
        "investigation",
        "inputs:\n  severity: low\n",
        &evidence,
    );
    let obligation = status_of(&obligations, "falsification.recorded");
    let outcome = outcomes["finding.published"]["status"]
        .as_str()
        .expect("an outcome status")
        .to_owned();
    assert!(
        !(obligation == "open" && outcome == "legitimate"),
        "obligation `falsification.recorded` is {obligation} while `finding.published`, which \
         requires every applicable obligation discharged, is {outcome}\nobligations: \
         {obligations}\noutcomes: {outcomes}"
    );
}

/// The unit's own pair `lost-applicability` is classified TIGHTENING (the obligation now applies
/// in every case). In the state above the obligation goes from `open` (applicability unknown) to
/// `discharged`: less is required of the case, which is a relaxation.
#[test]
fn removing_an_applicability_predicate_never_discharges_an_open_obligation() {
    let pair = "fixtures/investigation/semantic-diff/lost-applicability";
    let (from, to) = (
        ir_at(&format!("{pair}/from.yaml")),
        ir_at(&format!("{pair}/to.yaml")),
    );
    let changes = categories_for(&from, &to, "falsification.recorded");
    assert!(
        changes
            .iter()
            .any(|(category, _)| *category == Category::Tightening),
        "the pair is classified as the unit says: {changes:?}"
    );
    let evidence = [record("e1", "falsification_attempt", None)];
    let inputs = "inputs:\n  severity: low\n";
    let before = status_of(
        &decide(&from, "investigation", inputs, &evidence).0,
        "falsification.recorded",
    );
    let after = status_of(
        &decide(&to, "investigation", inputs, &evidence).0,
        "falsification.recorded",
    );
    assert!(
        !(before == "open" && after == "discharged"),
        "classified {changes:?}, yet `falsification.recorded` goes from {before} to {after}"
    );
}

/// Introducing an applicability predicate is classified RELAXATION. When the predicate tests a
/// claim that is UNKNOWN, an obligation that was `discharged` becomes `open`: more is required of
/// the case, which is a tightening.
#[test]
fn introducing_an_applicability_predicate_never_opens_a_discharged_obligation() {
    let protocol = |guard: &str, revision: u32| {
        ir_of(&format!(
            "format: protocol/1\nprotocol: {{id: p, revision: {revision}}}\n\
             artifacts: {{explanation: {{}}}}\n\
             evidence_kinds: {{kg: {{}}, kd: {{}}}}\n\
             claims: {{g: {{true_when: {{evidence: {{kind: kg}}}}}}, d: {{true_when: {{evidence: {{kind: kd}}}}}}}}\n\
             obligations: {{o: {{discharged_when: {{claim: d}}{guard}}}}}\n"
        ))
    };
    let (from, to) = (
        protocol("", 1),
        protocol(", applicable_when: {claim: g}", 2),
    );
    let changes = categories_for(&from, &to, "o");
    assert_eq!(
        changes,
        [(
            Category::Relaxation,
            "RELAXATION: obligation `o` applicability predicate introduced".to_owned()
        )]
    );
    let evidence = [record("e1", "kd", None)];
    let before = status_of(&decide(&from, "p", "", &evidence).0, "o");
    let after = status_of(&decide(&to, "p", "", &evidence).0, "o");
    assert!(
        !(before == "discharged" && after == "open"),
        "classified RELAXATION, yet `o` goes from {before} to {after}"
    );
}

/// A claim strengthened is classified TIGHTENING on its own. A claim an applicability predicate
/// tests positively has the opposite effect there: strengthened, the obligation applies in fewer
/// cases. Here the claim only guards the obligation, and strengthening it takes the obligation
/// from `open` to `not_applicable`, which is a relaxation the report calls a tightening.
#[test]
fn strengthening_a_claim_an_applicability_predicate_tests_is_not_a_tightening() {
    let protocol = |guard: &str, revision: u32| {
        ir_of(&format!(
            "format: protocol/1\nprotocol: {{id: p, revision: {revision}}}\n\
             artifacts: {{explanation: {{}}}}\n\
             evidence_kinds: {{kg: {{}}, kh: {{}}, kd: {{}}}}\n\
             claims: {{g: {{true_when: {guard}}}, d: {{true_when: {{evidence: {{kind: kd}}}}}}}}\n\
             obligations: {{o: {{applicable_when: {{claim: g}}, discharged_when: {{claim: d}}}}}}\n"
        ))
    };
    let from = protocol("{evidence: {kind: kg}}", 1);
    let to = protocol(
        "{all: [{evidence: {kind: kg}}, {evidence: {kind: kh, result: pass}}]}",
        2,
    );
    let report = diff(&from, &to).expect("one protocol").render();
    let evidence = [record("e1", "kg", None), record("e2", "kh", Some("fail"))];
    let before = status_of(&decide(&from, "p", "", &evidence).0, "o");
    let after = status_of(&decide(&to, "p", "", &evidence).0, "o");
    assert_eq!(
        (before.as_str(), after.as_str()),
        ("open", "not_applicable"),
        "the state this case is about"
    );
    assert!(
        report.contains("BREAKING") || report.contains("RELAXATION"),
        "`o` goes from {before} to {after}, a relaxation, and the report is only:\n{report}"
    );
}

/// Two input tests on one input whose `in` lists share a prefix are different tests: an `any` of
/// them keeps both, and the members' written order does not change the IR. Kills the mutant that
/// drops the length tie-break from the canonical order of input tests, under which `sorted` merges
/// `[a]` and `[a, b]` as equal and keeps whichever was written first.
#[test]
fn input_tests_sharing_a_prefix_are_both_kept_in_canonical_order() {
    let protocol = |members: &str| {
        ir_of(&format!(
            "format: protocol/1\nprotocol: {{id: p, revision: 1}}\n\
             case: {{inputs: {{s: {{values: [a, b, c]}}, t: {{values: [a]}}}}}}\n\
             actions: {{x: {{precondition: {{any: [{members}]}}}}}}\n"
        ))
        .canonical_json()
    };
    let (short, long, other) = (
        "{input: s, in: [a]}",
        "{input: s, in: [b, a]}",
        "{input: t, in: [a]}",
    );
    let written = protocol(&format!("{short}, {long}, {other}"));
    for members in [
        format!("{long}, {short}, {other}"),
        format!("{other}, {long}, {short}"),
        format!("{other}, {short}, {long}"),
    ] {
        assert_eq!(protocol(&members), written, "{members}");
    }
    let compact: String = written.split_whitespace().collect();
    assert!(
        compact.contains(
            r#""any":[{"input":{"id":"s","in":["a"]}},{"input":{"id":"s","in":["a","b"]}},{"input":{"id":"t","in":["a"]}}]"#
        ),
        "{written}"
    );
}

/// A protocol may declare the input values `true` and `false` unquoted (YAML booleans, read as the
/// text `true` and `false`), and a case written the same way, `flag: true`, is refused as
/// malformed: the protocol is valid, yet no case written in its own spelling evaluates. Either the
/// validator refuses a value that is not written as a string, or the case reads it as the protocol
/// does.
#[test]
fn an_input_value_the_protocol_accepts_is_accepted_in_a_case_written_alike() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
                  artifacts: {explanation: {}}\n\
                  case: {inputs: {flag: {values: [true, false]}}}\n";
    let compiled = parse(source)
        .ok()
        .and_then(|protocol| compile(&protocol).ok());
    let case = read_case(
        "format: canon-case/1\nid: C\nprotocol: p\nartifacts:\n  explanation: {revision: r1}\n\
         inputs:\n  flag: true\n",
    );
    match (compiled, case) {
        (None, _) => {}
        (Some(ir), Ok(case)) => {
            evaluate(&ir, &case, &[]).expect("a case in the protocol's own spelling evaluates");
        }
        (Some(_), Err(refusal)) => panic!(
            "the protocol declaring `values: [true, false]` compiles, and the case giving \
             `flag: true` is refused: {refusal}"
        ),
    }
}
