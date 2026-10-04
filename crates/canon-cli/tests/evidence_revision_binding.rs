//! Acceptance for story:evidence-revision-binding: conformance scenario `CANON-EVIDENCE-001`
//! passes under `canon conform run`, with three expectations: `explanation.supported` is `TRUE`
//! while its evidence is bound to the current revision of the explanation artifact; it is
//! `UNKNOWN`, not `FALSE`, with the same evidence when only the case snapshot advances that
//! artifact to a new revision; and an evidence record whose subject is not a declared artifact is
//! refused, the refusal naming that subject. A scenario compares only a refusal's code, so the
//! naming is checked here, through the library and through `canon evaluate`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use b10x_canon::conform::{self, EvaluateExpectation, EvaluateInputs, StepKind};
use b10x_canon::model::ArtifactId;

/// The scenario file this story adds to the registry.
const SCENARIO: &str = "conformance/scenarios/evidence-revision-binding.yaml";
/// The scenario's id, as `canon conform run` reports it.
const SCENARIO_ID: &str = "CANON-EVIDENCE-001";
/// The fixture only this story's scenario names.
const FIXTURE: &str = "fixtures/investigation/evidence-revision-binding.yaml";
/// The claim the expectations are about.
const CLAIM: &str = "explanation.supported";
/// The artifact the evidence is bound to.
const ARTIFACT: &str = "explanation";
/// The subject of the refused record: an artifact the fixture does not declare.
const UNDECLARED: &str = "hypothesis";
/// The refusal code the third step expects.
const REFUSAL: &str = "undeclared-artifact";

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

fn read(relative: &str) -> String {
    std::fs::read_to_string(repository_root().join(relative))
        .unwrap_or_else(|error| panic!("{relative}: {error}"))
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

fn scenario() -> conform::Scenario {
    conform::parse(&read(SCENARIO))
        .unwrap_or_else(|error| panic!("{SCENARIO} does not parse: {error}"))
}

fn inputs(step: &conform::Step) -> &EvaluateInputs {
    let StepKind::Evaluate { inputs, .. } = &step.kind else {
        panic!("{SCENARIO}: step `{}` is not an evaluate step", step.id);
    };
    inputs
}

fn expected(step: &conform::Step) -> &EvaluateExpectation {
    let StepKind::Evaluate { expected, .. } = &step.kind else {
        panic!("{SCENARIO}: step `{}` is not an evaluate step", step.id);
    };
    expected
}

/// The revision the case snapshot of a step gives the explanation artifact.
fn current_revision(inputs: &EvaluateInputs) -> &str {
    inputs.case["artifacts"][ARTIFACT]["revision"]
        .as_str()
        .expect("the case names the explanation's revision")
}

/// `(id, subject, subject revision)` of each evidence record of a step, in the order written.
fn bindings(inputs: &EvaluateInputs) -> Vec<(String, String, String)> {
    inputs
        .evidence
        .iter()
        .map(|record| {
            let get = |name: &str| {
                record[name]
                    .as_str()
                    .unwrap_or_else(|| panic!("an evidence record names its {name}"))
                    .to_owned()
            };
            (get("id"), get("subject"), get("subject_revision"))
        })
        .collect()
}

/// The claim's entry as canonical `canon-decision/1` JSON writes it, with no excluded evidence.
fn claim_entry(value: &str) -> String {
    format!("\"{CLAIM}\": {{\n      \"value\": \"{value}\"\n    }}")
}

/// The claim's entry with every record listed as excluded for a revision mismatch, in
/// evidence-id order.
fn excluded_entry(value: &str, mut excluded: Vec<&str>) -> String {
    excluded.sort_unstable();
    let items: Vec<String> = excluded
        .iter()
        .map(|id| {
            format!(
                "        {{\n          \"evidence\": \"{id}\",\n          \"reason\": \"revision_mismatch\"\n        }}"
            )
        })
        .collect();
    format!(
        "\"{CLAIM}\": {{\n      \"excluded_evidence\": [\n{}\n      ],\n      \"value\": \"{value}\"\n    }}",
        items.join(",\n")
    )
}

#[test]
fn the_scenario_holds_the_three_expectations_the_acceptance_names() {
    let scenario = scenario();
    assert_eq!(scenario.id, SCENARIO_ID);
    assert_eq!(scenario.covers, [SCENARIO_ID]);
    assert_eq!(scenario.fixture, FIXTURE);
    let ids: Vec<&str> = scenario.steps.iter().map(|step| step.id.as_str()).collect();
    assert_eq!(
        ids,
        ["bound-to-current", "subject-advanced", "undeclared-subject"],
        "{SCENARIO}: one step per expectation, in the acceptance's order"
    );
    let [bound, advanced, undeclared] = &scenario.steps[..] else {
        unreachable!("three steps");
    };

    // Step 1: every record bound to the explanation at the case's current revision; TRUE.
    let current = current_revision(inputs(bound));
    let bound_records = bindings(inputs(bound));
    assert!(!bound_records.is_empty(), "step 1 gives evidence");
    for (id, subject, revision) in &bound_records {
        assert_eq!(
            (subject.as_str(), revision.as_str()),
            (ARTIFACT, current),
            "step 1: `{id}` is bound to the current revision"
        );
    }
    let EvaluateExpectation::Decision(decision) = expected(bound) else {
        panic!("step 1 expects a decision");
    };
    assert!(
        decision.contains(&claim_entry("true")),
        "step 1: expected {CLAIM} = true in\n{decision}"
    );

    // Step 2: the same evidence; only the case snapshot advances the explanation. UNKNOWN, with
    // every record excluded for a revision mismatch.
    assert_eq!(
        inputs(advanced).evidence,
        inputs(bound).evidence,
        "step 2 gives the same evidence as step 1"
    );
    assert_ne!(
        current_revision(inputs(advanced)),
        current,
        "step 2 advances the explanation"
    );
    let mut advanced_case = inputs(advanced).case.clone();
    advanced_case["artifacts"][ARTIFACT]["revision"] =
        inputs(bound).case["artifacts"][ARTIFACT]["revision"].clone();
    assert_eq!(
        advanced_case,
        inputs(bound).case,
        "step 2's case differs from step 1's only in the explanation's revision"
    );
    let EvaluateExpectation::Decision(decision) = expected(advanced) else {
        panic!("step 2 expects a decision");
    };
    let excluded: Vec<&str> = bound_records.iter().map(|(id, ..)| id.as_str()).collect();
    assert!(
        decision.contains(&excluded_entry("unknown", excluded)),
        "step 2: expected {CLAIM} = unknown with every record excluded in\n{decision}"
    );
    assert!(!decision.contains("\"false\""), "step 2 is never FALSE");

    // Step 3: one record names a subject the fixture does not declare; refused.
    let protocol = b10x_canon::model::parse(&read(FIXTURE)).expect("the fixture parses");
    assert!(
        !protocol.artifacts.contains(&ArtifactId::new(UNDECLARED)),
        "the fixture does not declare `{UNDECLARED}`"
    );
    assert!(
        bindings(inputs(undeclared))
            .iter()
            .any(|(_, subject, _)| subject == UNDECLARED),
        "step 3 has a record about `{UNDECLARED}`"
    );
    assert_eq!(
        expected(undeclared),
        &EvaluateExpectation::Refusal(REFUSAL.to_owned()),
        "step 3 expects the refusal"
    );
}

#[test]
fn canon_evidence_001_passes_under_conform_run() {
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

/// The library refuses the scenario's third step with the code it expects, and the message names
/// the undeclared subject.
#[test]
fn the_refusal_names_the_undeclared_subject() {
    let scenario = scenario();
    let step = scenario
        .steps
        .iter()
        .find(|step| step.id == "undeclared-subject")
        .expect("the step exists");
    let inputs = inputs(step);
    let protocol = b10x_canon::model::parse(&read(FIXTURE)).expect("the fixture parses");
    let ir = b10x_canon::ir::compile(&protocol).expect("the fixture compiles");
    let case = b10x_canon::eval::case_from_value(&inputs.case).expect("the case reads");
    let evidence = inputs
        .evidence
        .iter()
        .map(b10x_canon::eval::evidence_from_value)
        .collect::<Result<Vec<_>, _>>()
        .expect("the evidence reads");
    let refusal = b10x_canon::eval::evaluate(&ir, &case, &evidence)
        .map(|decision| b10x_canon::eval::render(&decision))
        .expect_err("a record about an undeclared artifact is refused");
    assert_eq!(refusal.code(), REFUSAL, "{refusal}");
    assert!(
        refusal.to_string().contains(&format!("`{UNDECLARED}`")),
        "the refusal names the subject: {refusal}"
    );
}

/// A fresh directory under this test target's scratch space.
fn scratch(name: &str) -> PathBuf {
    let dir =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("evidence-revision-binding-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("old scratch directory removed");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory created");
    dir
}

/// `canon evaluate` refuses a record about an undeclared artifact with exit 1, naming the code
/// and the subject on standard error.
#[test]
fn canon_evaluate_refuses_an_undeclared_subject_naming_it() {
    let dir = scratch("undeclared-subject");
    let compiled = canon(&["compile", "--path", FIXTURE]);
    assert_eq!(compiled.status.code(), Some(0), "the fixture compiles");
    let ir = dir.join("protocol.ir.json");
    std::fs::write(&ir, &compiled.stdout).expect("IR written");
    let case = dir.join("case.yaml");
    std::fs::write(
        &case,
        "format: canon-case/1\nid: INV-18\nprotocol: investigation\nartifacts:\n  explanation: {revision: r1}\n",
    )
    .expect("case written");
    let evidence = dir.join("evidence");
    std::fs::create_dir(&evidence).expect("evidence directory created");
    std::fs::write(
        evidence.join("00.yaml"),
        "format: canon-evidence/1\nid: observation-1\nkind: supporting_observation\nsubject: explanation\nsubject_revision: r1\n",
    )
    .expect("written");
    std::fs::write(
        evidence.join("01.yaml"),
        format!(
            "format: canon-evidence/1\nid: falsification-1\nkind: falsification_attempt\nresult: survived\nsubject: {UNDECLARED}\nsubject_revision: r1\n"
        ),
    )
    .expect("written");
    let path = |p: &Path| p.to_str().expect("utf-8 path").to_owned();
    let run = canon(&[
        "evaluate",
        "--ir",
        &path(&ir),
        "--case",
        &path(&case),
        "--evidence",
        &path(&evidence),
    ]);
    let stderr = text(&run.stderr);
    assert_eq!(text(&run.stdout), "", "refusal: stdout");
    assert!(
        stderr.starts_with(&format!("error[{REFUSAL}]: ")),
        "refusal: stderr {stderr}"
    );
    assert!(
        stderr.contains(&format!("`{UNDECLARED}`")),
        "the refusal names the subject: {stderr}"
    );
    assert_eq!(stderr.lines().count(), 1, "one line: {stderr}");
    assert_eq!(run.status.code(), Some(1), "refusal: exit");
}
