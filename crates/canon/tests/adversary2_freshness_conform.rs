//! Adversary pass 2 for story:evidence-freshness: a conformance scenario's `at` field.
//!
//! `conform/mod.rs`: an evaluate step passes `at` "through unread: the evaluator decides what
//! they mean". `eval/freshness.rs`: text that is not an instant is refused as `invalid-instant`;
//! without an instant nothing expires.

use std::path::PathBuf;

use b10x_canon::conform::{self, Verdict};

/// Read at run time: a test binary reused from a shared build directory must read this tree.
fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

const FIXTURE: &str = "fixtures/investigation/evidence-freshness.yaml";

fn fixture() -> String {
    std::fs::read_to_string(repository_root().join(FIXTURE)).expect("fixture reads")
}

/// One evaluate step over the freshness fixture's observation, observed at 12:00, with `at_line`
/// (a whole `      at: …` line, or empty) and `expect` (the lines under `expect:`).
fn scenario(at_line: &str, expect: &str) -> String {
    format!(
        "format: canon-conformance/1\nid: ADV2-AT\ncovers: [CANON-EVIDENCE-002]\nfixture: {FIXTURE}\n\
         steps:\n  - id: s\n    evaluate:\n      case:\n        format: canon-case/1\n        id: INV-18\n\
         \x20       protocol: investigation\n        artifacts:\n          explanation: {{revision: r1}}\n\
         \x20     evidence:\n        - format: canon-evidence/1\n          id: observation-1\n\
         \x20         kind: supporting_observation\n          subject: explanation\n\
         \x20         subject_revision: r1\n          observed_at: 2026-10-04T12:00:00Z\n\
         {at_line}    expect:\n{expect}"
    )
}

fn verdict(text: &str) -> Verdict {
    let scenario = conform::parse(text).unwrap_or_else(|error| panic!("parses: {error}\n{text}"));
    conform::run(&scenario, Ok(&fixture()))
}

/// The decision with the observation applying: the claim is unknown only because there is no
/// falsification attempt, and nothing is excluded.
const APPLIES: &str = "      decision: |\n        {\n          \"claims\": {\n            \
                       \"explanation.supported\": {\n              \"value\": \"unknown\"\n            \
                       }\n          }\n        }\n";

/// No `at`: nothing expires, even two years after the observation would be.
#[test]
fn a_step_without_at_expires_nothing() {
    assert_eq!(verdict(&scenario("", APPLIES)), Verdict::Passed);
}

/// A malformed `at` reaches the evaluator and is refused there as `invalid-instant`.
#[test]
fn a_malformed_at_is_an_invalid_instant_refusal() {
    for at in ["2026-10-04T12:00:00+00:00", "\"\"", "2026-02-30T00:00:00Z"] {
        assert_eq!(
            verdict(&scenario(
                &format!("      at: {at}\n"),
                "      refusal: invalid-instant\n"
            )),
            Verdict::Passed,
            "at: {at}"
        );
    }
}

/// `at:` with no value is refused when the scenario is read (reported by position only, as the
/// module docs say), not read as "no instant".
#[test]
fn an_explicit_null_at_does_not_parse() {
    for at in ["      at:\n", "      at: ~\n"] {
        let error = conform::parse(&scenario(at, APPLIES)).expect_err("refused");
        assert!(
            error
                .to_string()
                .starts_with("not a canon-conformance/1 scenario at line"),
            "{error}"
        );
    }
}

/// `at` written as a YAML number is passed through as text, so the evaluator refuses it as
/// `invalid-instant`: the docs say the evaluator decides what `at` means.
#[test]
fn an_at_written_as_a_number_reaches_the_evaluator() {
    let text = scenario("      at: 1791115200\n", "      refusal: invalid-instant\n");
    match conform::parse(&text) {
        Ok(scenario) => assert_eq!(conform::run(&scenario, Ok(&fixture())), Verdict::Passed),
        Err(error) => panic!("the scenario was refused before the evaluator saw `at`: {error}"),
    }
}
