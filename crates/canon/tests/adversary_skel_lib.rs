//! Adversary cases for story:evaluator-skeleton (wave 2026-10-04-w6, pass 1), driven through the
//! library from statements the unit wrote: the story's per-section comparison and the validator's
//! documented order of unresolved references.

use b10x_canon::conform::{self, Verdict};
use b10x_canon::{model, validate};

const PROTOCOL: &str =
    "format: protocol/1\nprotocol: {id: p, revision: 1}\nclaims:\n  c: {true_when: {all: []}}\n";

/// One evaluate step over `PROTOCOL` with no evidence, expecting `decision` (a literal block).
fn scenario(decision: &str) -> String {
    let indented: String = decision
        .lines()
        .map(|line| format!("        {line}\n"))
        .collect();
    format!(
        "format: canon-conformance/1\nid: S\ncovers: []\nfixture: p.yaml\nsteps:\n  - id: e\n    evaluate:\n      case: {{format: canon-case/1, id: C, protocol: p, artifacts: {{}}}}\n      evidence: []\n    expect:\n      decision: |\n{indented}"
    )
}

fn verdict(decision: &str) -> Verdict {
    let scenario = conform::parse(&scenario(decision)).expect("scenario parses");
    conform::run(&scenario, Ok(PROTOCOL))
}

/// story:evaluator-skeleton § What it lands 4: "only the sections of `canon-decision/1` the
/// expectation lists are compared, each byte for byte." Two listed sections, each written exactly
/// as the decision writes it, pass in canonical order; listed in the other order, each section is
/// still byte for byte the decision's, and the step fails with no section named.
#[test]
fn listed_sections_each_byte_identical_pass_in_any_order() {
    assert_eq!(
        verdict("{\n  \"case\": \"C\",\n  \"protocol\": \"p\"\n}\n"),
        Verdict::Passed,
        "control: canonical order"
    );
    assert_eq!(
        verdict("{\n  \"protocol\": \"p\",\n  \"case\": \"C\"\n}\n"),
        Verdict::Passed,
        "each listed section equals the decision's byte for byte"
    );
}

/// validate/mod.rs module docs, item 4: "unresolved references: those in claims first, then those
/// in obligations (their discharge predicates), then those in actions, then those in outcomes".
/// No test of the unit has more than one section's unresolved reference, so moving the
/// obligations loop after the actions loop stays green without this.
#[test]
fn unresolved_discharge_references_come_after_claims_and_before_actions() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        claims:\n  c: {true_when: {claim: x}}\n\
        outcomes:\n  w: {requires: {claim: z}}\n\
        actions:\n  a: {precondition: {claim: y}}\n\
        obligations:\n  o: {discharged_when: {claim: v}}\n";
    let protocol = model::parse(source).expect("parses");
    let problems: Vec<String> = validate::validate(&protocol)
        .expect_err("refused")
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        problems,
        vec![
            "claim `c` references claim `x`, which is not declared".to_owned(),
            "obligation `o` references claim `v`, which is not declared".to_owned(),
            "action `a` references claim `y`, which is not declared".to_owned(),
            "outcome `w` references claim `z`, which is not declared".to_owned(),
        ]
    );
}
