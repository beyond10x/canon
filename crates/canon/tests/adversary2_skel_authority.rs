//! Adversary cases for story:evaluator-skeleton (wave 2026-10-04-w6, pass 2): the conformance
//! runner passes a step's `authority` to the evaluator as YAML text (conform/mod.rs module docs),
//! re-serializing the YAML values the scenario holds with an `expect` that claims every YAML value
//! serializes back.
//!
//! Changed by story:action-admissibility: the evaluator now reads `canon-authority/1`, so an
//! authority list is evaluated or refused as `malformed-input` instead of refused as
//! `unsupported-input`. The intent is unchanged: whatever the runner hands over, it does not panic
//! and the step meets its stated expectation.

use b10x_canon::conform::{self, Verdict};

const PROTOCOL: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n";

/// A step expectation: a decision listing only the `case` section, or a refusal by its code.
const DECISION: &str = "decision: |\n        {\n          \"case\": \"C\"\n        }";
const MALFORMED: &str = "refusal: malformed-input";

fn verdict(authority: &str, expect: &str) -> Verdict {
    let source = format!(
        "format: canon-conformance/1\nid: S\ncovers: []\nfixture: p.yaml\nsteps:\n  - id: e\n    evaluate:\n      case: {{format: canon-case/1, id: C, protocol: p, artifacts: {{}}}}\n      evidence: []\n      authority: {authority}\n    expect:\n      {expect}\n"
    );
    let scenario = conform::parse(&source).expect("scenario parses");
    conform::run(&scenario, Ok(PROTOCOL))
}

/// Any authority list the scenario parser accepts reaches the evaluator and the runner does not
/// panic. A list of decisions is evaluated; anything else, including an entry whose key is a
/// sequence or a mapping, is refused as `malformed-input` (story:action-admissibility chose
/// refusal for the mapping-key entries: a key that is not a field name is not a decision).
#[test]
fn any_authority_the_scenario_accepts_reaches_the_evaluator() {
    for (authority, expect) in [
        ("[]", DECISION),
        ("[{capability: c, decision: granted}]", DECISION),
        ("[{[a, b]: c}]", MALFORMED),
        ("[{{x: 1}: c}]", MALFORMED),
        ("[!custom {capability: c}]", MALFORMED),
        ("[.nan, -.inf]", MALFORMED),
        ("[{? [a] : b}]", MALFORMED),
    ] {
        let found = std::panic::catch_unwind(|| verdict(authority, expect));
        assert!(
            matches!(found, Ok(Verdict::Passed)),
            "authority {authority}: {:?}",
            found.map_err(|_| "panicked")
        );
    }
}
