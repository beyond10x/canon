//! Adversary cases for story:evaluator-skeleton (wave 2026-10-04-w6, pass 2): the conformance
//! runner passes a step's `authority` to the evaluator as YAML text (conform/mod.rs module docs),
//! re-serializing the YAML values the scenario holds with an `expect` that claims every YAML value
//! serializes back.

use b10x_canon::conform::{self, Verdict};

const PROTOCOL: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n";

fn verdict(authority: &str) -> Verdict {
    let source = format!(
        "format: canon-conformance/1\nid: S\ncovers: []\nfixture: p.yaml\nsteps:\n  - id: e\n    evaluate:\n      case: {{format: canon-case/1, id: C, protocol: p, artifacts: {{}}}}\n      evidence: []\n      authority: {authority}\n    expect: {{refusal: unsupported-input}}\n"
    );
    let scenario = conform::parse(&source).expect("scenario parses");
    conform::run(&scenario, Ok(PROTOCOL))
}

/// Any authority list the scenario parser accepts reaches the evaluator, which refuses it as
/// `unsupported-input` until story:action-admissibility reads it: the runner does not panic.
#[test]
fn any_authority_the_scenario_accepts_reaches_the_evaluator() {
    for authority in [
        "[]",
        "[{capability: c, decision: granted}]",
        "[{[a, b]: c}]",
        "[{{x: 1}: c}]",
        "[!custom {capability: c}]",
        "[.nan, -.inf]",
        "[{? [a] : b}]",
    ] {
        let found = std::panic::catch_unwind(|| verdict(authority));
        assert!(
            matches!(found, Ok(Verdict::Passed)),
            "authority {authority}: {:?}",
            found.map_err(|_| "panicked")
        );
    }
}
