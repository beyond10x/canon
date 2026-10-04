//! Adversary pass 2 for story:conformance-runner (wave 2026-10-04-w3): line endings and trailing
//! newlines between a scenario's expected `canon-ir/1` and the compiled IR, through the YAML
//! layer rather than `first_difference` alone; and an evaluate-only scenario of the shape
//! story:three-valued-claims writes.

use std::path::PathBuf;

use b10x_canon::conform::{self, StepKind, Verdict};
use b10x_canon::{ir, model};

fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn read(path: &str) -> String {
    std::fs::read_to_string(repository_root().join(path)).expect(path)
}

fn fixture() -> String {
    read("fixtures/investigation/protocol.yaml")
}

fn passing_scenario() -> String {
    read("crates/canon-cli/tests/conform/passing/compile-passes.yaml")
}

fn compiled_lines() -> usize {
    ir::compile(&model::parse(&fixture()).expect("parses"))
        .expect("compiles")
        .canonical_json()
        .lines()
        .count()
}

fn reason(scenario_text: &str) -> Option<String> {
    let scenario = conform::parse(scenario_text).expect("scenario parses");
    match conform::run(&scenario, Ok(&fixture())) {
        Verdict::Passed => None,
        Verdict::Failed { reason, .. } => Some(reason),
    }
}

/// A scenario file checked out with CRLF line endings (YAML 1.2 § 5.4: line breaks in scalar
/// content are normalized) is the same scenario and still passes.
#[test]
fn a_crlf_scenario_file_passes_like_its_lf_original() {
    let crlf = passing_scenario().replace('\n', "\r\n");
    assert_eq!(reason(&passing_scenario()), None, "LF original");
    assert_eq!(reason(&crlf), None, "CRLF copy");
}

/// A fixture with CRLF line endings compiles to the same IR.
#[test]
fn a_crlf_fixture_compiles_to_the_same_ir() {
    let scenario = conform::parse(&passing_scenario()).expect("parses");
    let crlf = fixture().replace('\n', "\r\n");
    assert_eq!(conform::run(&scenario, Ok(&crlf)), Verdict::Passed);
}

/// `|-` strips the final newline the compiled IR ends with: the difference is on the last line.
#[test]
fn a_stripped_final_newline_differs_at_the_last_line() {
    let stripped = passing_scenario().replace("canon-ir: |\n", "canon-ir: |-\n");
    assert_eq!(
        reason(&stripped),
        Some(format!(
            "canon-ir/1 differs from the expectation at line {}",
            compiled_lines()
        ))
    );
}

/// `|+` keeps an extra trailing blank line: the difference is on the line after the last.
#[test]
fn an_extra_trailing_blank_line_differs_after_the_last_line() {
    let kept = passing_scenario().replace("canon-ir: |\n", "canon-ir: |+\n") + "\n";
    assert_eq!(
        reason(&kept),
        Some(format!(
            "canon-ir/1 differs from the expectation at line {}",
            compiled_lines() + 1
        ))
    );
}

/// An expectation whose second line ends in CRLF differs at line 2, not line 1 and not the end.
#[test]
fn a_crlf_inside_the_expectation_differs_at_its_line() {
    let scenario = conform::parse(&passing_scenario()).expect("parses");
    let expected = match &scenario.steps[0].kind {
        StepKind::Compile { expected_ir } => expected_ir.clone(),
        other => panic!("not a compile step: {other:?}"),
    };
    let mut lines: Vec<String> = expected.split_inclusive('\n').map(str::to_owned).collect();
    lines[1] = lines[1].replace('\n', "\r\n");
    let mut changed = scenario.clone();
    changed.steps[0].kind = StepKind::Compile {
        expected_ir: lines.concat(),
    };
    assert_eq!(
        conform::run(&changed, Ok(&fixture())),
        Verdict::Failed {
            step: "compile".to_owned(),
            reason: "canon-ir/1 differs from the expectation at line 2".to_owned()
        }
    );
}

/// The scenario story:three-valued-claims's acceptance describes — four evaluate steps, no
/// compile step, evidence records as mappings, no authority, no instant — parses into four
/// evaluate steps, and until that story wires the evaluator it fails at the first.
#[test]
fn a_three_valued_claims_shaped_scenario_parses_and_fails_unsupported_at_its_first_step() {
    let evaluate = |id: &str, evidence: &str| {
        format!(
            "  - id: {id}\n    evaluate:\n      case: {{format: canon-case/1, id: c1, protocol: investigation, artifacts: {{explanation: 1}}}}\n      evidence: {evidence}\n    expect:\n      decision: |\n        {{\"claims\": {{}}}}\n"
        )
    };
    let observation = "{id: e1, kind: supporting_observation, result: observed, subject: explanation, subject_revision: 1}";
    let survived = "{id: e2, kind: falsification_attempt, result: survived, subject: explanation, subject_revision: 1}";
    let refuted = "{id: e3, kind: falsification_attempt, result: refuted, subject: explanation, subject_revision: 1}";
    let text = format!(
        "format: canon-conformance/1\nid: CANON-CLAIM-001\ncovers: [CANON-CLAIM-001, CANON-CLAIM-002]\nfixture: fixtures/investigation/three-valued-claims.yaml\nsteps:\n{}{}{}{}",
        evaluate("no-evidence", "[]"),
        evaluate("refuted", &format!("[{observation}, {refuted}]")),
        evaluate("survived", &format!("[{observation}, {survived}]")),
        evaluate(
            "disagreeing",
            &format!("[{observation}, {survived}, {refuted}]")
        ),
    );
    let scenario = conform::parse(&text).expect("parses");
    let kinds: Vec<(&str, usize)> = scenario
        .steps
        .iter()
        .map(|step| match &step.kind {
            StepKind::Evaluate { inputs, .. } => (step.id.as_str(), inputs.evidence.len()),
            other => panic!("not an evaluate step: {other:?}"),
        })
        .collect();
    assert_eq!(
        kinds,
        [
            ("no-evidence", 0),
            ("refuted", 2),
            ("survived", 2),
            ("disagreeing", 3)
        ]
    );
    assert_eq!(
        conform::run(&scenario, Ok(&fixture())),
        Verdict::Failed {
            step: "no-evidence".to_owned(),
            reason: "evaluate steps are not supported yet".to_owned()
        }
    );
}
