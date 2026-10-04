//! Acceptance for story:evidence-freshness: conformance scenario `CANON-EVIDENCE-002` passes under
//! `canon conform run` over `fixtures/investigation/evidence-freshness.yaml`, with two expectations
//! and the evaluation instant the only input that changes: `explanation.supported` is `TRUE` at an
//! instant within its evidence's maximum age, and `UNKNOWN`, not `FALSE`, at an instant past it,
//! with the expired record listed as excluded for the reason `expired`. `canon evaluate --at`
//! prints the same `canon-decision/1` bytes the scenario expects.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use b10x_canon::conform::{self, EvaluateExpectation, EvaluateInputs, StepKind};

/// The scenario file this story adds to the registry.
const SCENARIO: &str = "conformance/scenarios/evidence-freshness.yaml";
/// The scenario's id, as `canon conform run` reports it.
const SCENARIO_ID: &str = "CANON-EVIDENCE-002";
/// The fixture: the base investigation protocol plus a maximum age on one evidence kind.
const FIXTURE: &str = "fixtures/investigation/evidence-freshness.yaml";
/// The base the fixture extends, unedited.
const BASE: &str = "fixtures/investigation/protocol.yaml";
/// The claim the two expectations are about.
const CLAIM: &str = "explanation.supported";

/// Read at run time, not compile time: a test binary reused from a shared build directory must
/// still read this tree's scenarios and fixtures.
fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn canon(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(repository_root())
        .args(args)
        .output()
        .expect("canon runs")
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("utf-8 output")
}

fn read(path: &str) -> String {
    std::fs::read_to_string(repository_root().join(path))
        .unwrap_or_else(|error| panic!("{path}: {error}"))
}

fn scenario() -> conform::Scenario {
    conform::parse(&read(SCENARIO))
        .unwrap_or_else(|error| panic!("{SCENARIO} does not parse: {error}"))
}

/// The two steps the acceptance names: id, and the claim's entry as canonical `canon-decision/1`
/// JSON writes it.
fn expected_steps() -> [(&'static str, String); 2] {
    [
        (
            "within-max-age",
            format!("\"{CLAIM}\": {{\n      \"value\": \"true\"\n    }}"),
        ),
        (
            "past-max-age",
            format!(
                "\"{CLAIM}\": {{\n      \"excluded_evidence\": [\n        {{\n          \
                 \"evidence\": \"observation-1\",\n          \"reason\": \"expired\"\n        \
                 }}\n      ],\n      \"value\": \"unknown\"\n    }}"
            ),
        ),
    ]
}

fn evaluate_step(step: &conform::Step) -> (&EvaluateInputs, &str) {
    let StepKind::Evaluate { inputs, expected } = &step.kind else {
        panic!("{SCENARIO}: step `{}` is not an evaluate step", step.id);
    };
    let EvaluateExpectation::Decision(decision) = expected else {
        panic!(
            "{SCENARIO}: step `{}` expects a refusal, not a decision",
            step.id
        );
    };
    (inputs, decision)
}

#[test]
fn the_scenario_holds_the_two_expectations_the_acceptance_names() {
    let scenario = scenario();
    assert_eq!(scenario.id, SCENARIO_ID);
    assert_eq!(scenario.covers, ["CANON-EVIDENCE-002"]);
    assert_eq!(scenario.fixture, FIXTURE);
    let expected = expected_steps();
    assert_eq!(
        scenario.steps.len(),
        expected.len(),
        "{SCENARIO}: one step per expectation"
    );
    for (step, (id, entry)) in scenario.steps.iter().zip(&expected) {
        assert_eq!(step.id, *id, "{SCENARIO}: step order");
        let (inputs, decision) = evaluate_step(step);
        assert!(
            inputs.at.is_some(),
            "step `{id}`: gives an evaluation instant"
        );
        assert!(
            decision.contains(entry.as_str()),
            "step `{id}`: expected {CLAIM} entry\n{entry}\nin\n{decision}"
        );
    }
    // The evaluation instant is the only input that changes.
    let (within, _) = evaluate_step(&scenario.steps[0]);
    let (past, _) = evaluate_step(&scenario.steps[1]);
    assert_eq!(within.case, past.case, "the case does not change");
    assert_eq!(
        within.evidence, past.evidence,
        "the evidence does not change"
    );
    assert_eq!(
        within.authority, past.authority,
        "the authority does not change"
    );
    assert_ne!(within.at, past.at, "the evaluation instant changes");
}

/// The fixture is the base with `max_age` added to evidence kinds the claim draws on, and nothing
/// else changed: removing every `max_age` line gives the base back, byte for byte.
#[test]
fn the_fixture_is_the_base_plus_a_maximum_age_on_evidence_the_claim_draws_on() {
    let fixture = read(FIXTURE);
    let base = read(BASE);
    let added: Vec<&str> = fixture
        .lines()
        .filter(|line| line.trim_start().starts_with("max_age:"))
        .collect();
    assert!(!added.is_empty(), "{FIXTURE} declares a maximum age");
    let without: String = fixture
        .lines()
        .filter(|line| !line.trim_start().starts_with("max_age:"))
        .map(|line| format!("{line}\n"))
        .collect();
    assert_eq!(without, base, "{FIXTURE} is {BASE} plus `max_age` only");
    // Each `max_age` sits under an evidence kind the claim's predicate names.
    let lines: Vec<&str> = fixture.lines().collect();
    for (at, line) in lines.iter().enumerate() {
        if !line.trim_start().starts_with("max_age:") {
            continue;
        }
        let kind = lines[..at]
            .iter()
            .rev()
            .find(|candidate| candidate.starts_with("  ") && !candidate.starts_with("    "))
            .expect("max_age sits under a declaration")
            .trim()
            .trim_end_matches(':');
        assert!(
            fixture.contains(&format!("kind: {kind}\n")),
            "{FIXTURE}: `max_age` on `{kind}`, which no evidence match names"
        );
    }
}

#[test]
fn canon_evidence_002_passes_under_conform_run() {
    let run = canon(&["conform", "run"]);
    let report = text(&run.stdout);
    let needle = format!("scenario `{SCENARIO_ID}`");
    let lines: Vec<&str> = report
        .lines()
        .filter(|line| line.contains(&needle))
        .collect();
    assert_eq!(
        lines,
        [format!("passed: scenario `{SCENARIO_ID}`")],
        "the registry's report on {SCENARIO_ID}:\n{report}{}",
        text(&run.stderr)
    );
}

/// The case snapshot both steps evaluate, as a `canon-case/1` file.
const CASE: &str = "format: canon-case/1\n\
                    id: INV-18\n\
                    protocol: investigation\n\
                    artifacts:\n  explanation: {revision: r1}\n";

/// The evidence both steps evaluate: the same records the scenario lists.
const EVIDENCE: [&str; 2] = [
    "format: canon-evidence/1\nid: observation-1\nkind: supporting_observation\n\
     subject: explanation\nsubject_revision: r1\nobserved_at: 2026-10-04T12:00:00Z\n",
    "format: canon-evidence/1\nid: falsification-1\nkind: falsification_attempt\n\
     result: survived\nsubject: explanation\nsubject_revision: r1\n\
     observed_at: 2026-10-04T12:00:00Z\n",
];

/// A fresh directory under this test target's scratch space.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("evidence-freshness-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("old scratch directory removed");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory created");
    dir
}

/// `canon evaluate --ir <file> --case <file> --evidence <dir> --at <instant>` over the compiled
/// fixture.
fn evaluate(step: &str, at: &str) -> Output {
    let dir = scratch(step);
    let compiled = canon(&["compile", "--path", FIXTURE]);
    assert_eq!(
        compiled.status.code(),
        Some(0),
        "the fixture compiles: {}",
        text(&compiled.stderr)
    );
    let ir = dir.join("protocol.ir.json");
    std::fs::write(&ir, &compiled.stdout).expect("IR written");
    let case = dir.join("case.yaml");
    std::fs::write(&case, CASE).expect("case written");
    let evidence = dir.join("evidence");
    std::fs::create_dir(&evidence).expect("evidence directory created");
    for (index, record) in EVIDENCE.iter().enumerate() {
        std::fs::write(evidence.join(format!("{index:02}.yaml")), record)
            .expect("evidence record written");
    }
    let path = |p: &Path| p.to_str().expect("utf-8 path").to_owned();
    canon(&[
        "evaluate",
        "--ir",
        &path(&ir),
        "--case",
        &path(&case),
        "--evidence",
        &path(&evidence),
        "--at",
        at,
    ])
}

#[test]
fn canon_evaluate_at_prints_the_decision_each_step_expects() {
    let scenario = scenario();
    assert_eq!(scenario.steps.len(), 2, "{SCENARIO}: two steps");
    for step in &scenario.steps {
        let (inputs, decision) = evaluate_step(step);
        let records: Vec<(&str, &str, Option<&str>, Option<&str>)> = inputs
            .evidence
            .iter()
            .map(|record| {
                (
                    record["id"].as_str().expect("a record names its id"),
                    record["kind"].as_str().expect("a record names its kind"),
                    record["result"].as_str(),
                    record["observed_at"].as_str(),
                )
            })
            .collect();
        let observed = Some("2026-10-04T12:00:00Z");
        assert_eq!(
            records,
            [
                ("observation-1", "supporting_observation", None, observed),
                (
                    "falsification-1",
                    "falsification_attempt",
                    Some("survived"),
                    observed
                ),
            ],
            "step `{}`: the scenario's evidence is the evidence written here",
            step.id
        );
        let at = inputs.at.as_deref().expect("the step gives an instant");
        let run = evaluate(&step.id, at);
        assert_eq!(
            text(&run.stderr),
            "",
            "step `{}`: canon evaluate stderr",
            step.id
        );
        assert_eq!(
            text(&run.stdout),
            decision,
            "step `{}`: canon evaluate stdout",
            step.id
        );
        assert_eq!(run.status.code(), Some(0), "step `{}`: exit", step.id);
    }
}
