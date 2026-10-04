//! Adversary cases for story:evaluator-skeleton (wave 2026-10-04-w6, pass 2), driven through the
//! library from the per-section comparison the unit documents in `conform/mod.rs`: "only those are
//! compared, each on its own and byte for byte".

use b10x_canon::conform::{self, Verdict};

const PROTOCOL: &str =
    "format: protocol/1\nprotocol: {id: p, revision: 1}\nclaims:\n  c: {true_when: {all: []}}\n";

/// One evaluate step over `PROTOCOL` with no evidence. `expect` is the YAML of the step's
/// expectation, written as it stands in the scenario file.
fn verdict(inputs: &str, expect: &str) -> Verdict {
    let source = format!(
        "format: canon-conformance/1\nid: S\ncovers: []\nfixture: p.yaml\nsteps:\n  - id: e\n    evaluate:\n      case: {{format: canon-case/1, id: C, protocol: p, artifacts: {{}}}}\n      evidence: []\n{inputs}    expect:\n      {expect}\n"
    );
    let scenario = conform::parse(&source).expect("scenario parses");
    conform::run(&scenario, Ok(PROTOCOL))
}

/// conform/mod.rs module docs: a listed section is compared "byte for byte"; `first_difference`
/// (the whole-text comparison the step used before this unit) counts a line with its newline, so
/// any byte added to a line is a difference. An expectation whose `case` line carries a carriage
/// return before its newline is not the decision's bytes, and still passes: the member parser
/// reads lines with `str::lines`, which drops a `\r` before `\n`, and the JSON parser that decides
/// the listed sections reads `\r` as whitespace.
#[test]
fn a_carriage_return_in_a_listed_section_is_a_difference() {
    // Control: the same expectation without the carriage return passes.
    assert_eq!(
        verdict("", r#"decision: "{\n  \"case\": \"C\"\n}\n""#),
        Verdict::Passed,
        "control"
    );
    let with_cr = verdict("", r#"decision: "{\n  \"case\": \"C\"\r\n}\n""#);
    assert!(
        matches!(&with_cr, Verdict::Failed { step, .. } if step == "e"),
        "an expectation that is not the decision's bytes passed: {with_cr:?}"
    );
    let two_sections = verdict(
        "",
        r#"decision: "{\n  \"case\": \"C\",\r\n  \"protocol\": \"p\"\r\n}\n""#,
    );
    assert!(
        matches!(&two_sections, Verdict::Failed { .. }),
        "an expectation that is not the decision's bytes passed: {two_sections:?}"
    );
}
