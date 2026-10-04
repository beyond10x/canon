//! Acceptance for story:subject-bound-evidence-match: conformance scenario `CANON-EVIDENCE-003`
//! passes under `canon conform run` over `fixtures/investigation/subject-bound-evidence-match.yaml`.
//! The fixture declares two artifacts, `explanation` and `dataset`; claim `explanation.survived`
//! is decided by an evidence match that names `subject: explanation`, and claim `attempt.survived`
//! by the same match without a subject. A record about the dataset, bound to the dataset's current
//! revision, does not satisfy the subject-bound match (`explanation.survived` is `UNKNOWN`, with
//! nothing excluded) while the unbound match keeps its behaviour (`attempt.survived` is `TRUE`);
//! and beside a refuted attempt about the explanation, it does not enter the subject-bound match's
//! count at all (`FALSE`, not the `UNKNOWN` of disagreeing records). `canon evaluate` prints the
//! bytes each step expects, and `canon validate` refuses a match whose subject is not a declared
//! artifact, naming it. A record the binding stage excludes is listed under a claim only when a
//! match the claim reaches would read it, so one about the dataset is not listed under
//! `explanation.survived` (coordinator decision, wave 2026-10-04-w10).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use b10x_canon::conform::{self, EvaluateExpectation, EvaluateInputs, StepKind};

/// The scenario file this story adds to the registry.
const SCENARIO: &str = "conformance/scenarios/subject-bound-evidence-match.yaml";
/// The scenario's id, as `canon conform run` reports it.
const SCENARIO_ID: &str = "CANON-EVIDENCE-003";
/// The fixture only this story's scenario names.
const FIXTURE: &str = "fixtures/investigation/subject-bound-evidence-match.yaml";
/// The claim whose evidence match names a subject.
const BOUND: &str = "explanation.survived";
/// The claim whose evidence match names none.
const UNBOUND: &str = "attempt.survived";
/// The artifact the bound match names.
const SUBJECT: &str = "explanation";
/// The other declared artifact.
const OTHER: &str = "dataset";

/// The steps, in order: id, and for an evaluate step the values the bound and the unbound claim
/// take.
const STEPS: [(&str, Option<(&str, &str)>); 4] = [
    ("compile", None),
    ("about-the-subject", Some(("true", "true"))),
    ("about-another-artifact", Some(("unknown", "true"))),
    (
        "another-artifact-does-not-count",
        Some(("false", "unknown")),
    ),
];

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

fn step<'a>(scenario: &'a conform::Scenario, id: &str) -> &'a conform::Step {
    scenario
        .steps
        .iter()
        .find(|step| step.id == id)
        .unwrap_or_else(|| panic!("{SCENARIO}: no step `{id}`"))
}

fn evaluate_step(step: &conform::Step) -> (&EvaluateInputs, &EvaluateExpectation) {
    let StepKind::Evaluate { inputs, expected } = &step.kind else {
        panic!("{SCENARIO}: step `{}` is not an evaluate step", step.id);
    };
    (inputs, expected)
}

/// One evidence record as a step gives it: id, kind, result, subject and subject revision.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Record {
    id: String,
    kind: String,
    result: Option<String>,
    subject: String,
    subject_revision: String,
}

fn records(inputs: &EvaluateInputs) -> Vec<Record> {
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
            Record {
                id: get("id"),
                kind: get("kind"),
                result: record["result"].as_str().map(str::to_owned),
                subject: get("subject"),
                subject_revision: get("subject_revision"),
            }
        })
        .collect()
}

/// The revision a step's case snapshot gives an artifact.
fn current(inputs: &EvaluateInputs, artifact: &str) -> String {
    inputs.case["artifacts"][artifact]["revision"]
        .as_str()
        .unwrap_or_else(|| panic!("the case names the current revision of `{artifact}`"))
        .to_owned()
}

/// A claim's entry as canonical `canon-decision/1` JSON writes it, with no excluded evidence.
fn claim_entry(claim: &str, value: &str) -> String {
    format!("\"{claim}\": {{\n      \"value\": \"{value}\"\n    }}")
}

#[test]
fn the_scenario_holds_the_expectations_the_acceptance_names() {
    let scenario = scenario();
    assert_eq!(scenario.id, SCENARIO_ID);
    assert_eq!(scenario.covers, [SCENARIO_ID]);
    assert_eq!(scenario.fixture, FIXTURE);
    let ids: Vec<&str> = scenario.steps.iter().map(|step| step.id.as_str()).collect();
    let expected_ids: Vec<&str> = STEPS.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, expected_ids, "{SCENARIO}: step order");
    assert!(
        matches!(scenario.steps[0].kind, StepKind::Compile { .. }),
        "{SCENARIO}: the first step compiles the fixture"
    );

    for (id, values) in STEPS {
        let Some((bound, unbound)) = values else {
            continue;
        };
        let (inputs, expected) = evaluate_step(step(&scenario, id));
        // Every record is bound to the current revision of its own subject, so the binding stage
        // excludes nothing: only the subject can decide what a match reads.
        let given = records(inputs);
        assert!(!given.is_empty(), "step `{id}` gives evidence");
        for record in &given {
            assert!(
                record.subject == SUBJECT || record.subject == OTHER,
                "step `{id}`: `{}` is about a declared artifact",
                record.id
            );
            assert_eq!(
                record.subject_revision,
                current(inputs, &record.subject),
                "step `{id}`: `{}` is bound to its subject's current revision",
                record.id
            );
        }
        let EvaluateExpectation::Decision(decision) = expected else {
            panic!("step `{id}` expects a decision");
        };
        for (claim, value) in [(BOUND, bound), (UNBOUND, unbound)] {
            assert!(
                decision.contains(&claim_entry(claim, value)),
                "step `{id}`: expected {claim} = {value} in\n{decision}"
            );
        }
        assert!(
            !decision.contains("excluded_evidence"),
            "step `{id}`: a record about another artifact is not of the match; nothing is excluded\n{decision}"
        );
    }

    // The record about another artifact is the record about the subject, moved to the dataset at
    // its current revision, and nothing else.
    let (about_subject, _) = evaluate_step(step(&scenario, "about-the-subject"));
    let (about_other, _) = evaluate_step(step(&scenario, "about-another-artifact"));
    assert_eq!(
        about_other.case, about_subject.case,
        "the two steps read one case snapshot"
    );
    let moved: Vec<Record> = records(about_subject)
        .into_iter()
        .map(|record| {
            assert_eq!(record.subject, SUBJECT, "step 2 is about the subject");
            Record {
                subject: OTHER.to_owned(),
                subject_revision: current(about_subject, OTHER),
                ..record
            }
        })
        .collect();
    assert_eq!(
        records(about_other),
        moved,
        "step 3 differs from step 2 only in the subject"
    );

    // Beside it, one refuted attempt about the subject: the bound match reads only that one.
    let (mixed, _) = evaluate_step(step(&scenario, "another-artifact-does-not-count"));
    let mixed = records(mixed);
    let about = |subject: &str| -> Vec<Option<String>> {
        mixed
            .iter()
            .filter(|record| record.subject == subject)
            .map(|record| record.result.clone())
            .collect()
    };
    assert_eq!(about(SUBJECT), [Some("refuted".to_owned())]);
    assert_eq!(about(OTHER), [Some("survived".to_owned())]);
}

/// The fixture's two claims are the same match, one naming the subject and one naming none.
#[test]
fn the_fixture_binds_one_claim_and_leaves_the_other_unbound() {
    let fixture = read(FIXTURE);
    let match_of = |claim: &str| -> String {
        let start = fixture
            .find(&format!("\n  {claim}:\n"))
            .unwrap_or_else(|| panic!("{FIXTURE} declares {claim}"));
        let body = &fixture[start + 1..];
        let end = body[1..].find("\n\n").map_or(body.len(), |at| at + 1);
        body[..end].to_owned()
    };
    let bound = match_of(BOUND);
    let unbound = match_of(UNBOUND);
    assert!(
        bound.contains(&format!(
            "      evidence:\n        kind: falsification_attempt\n        result: survived\n        subject: {SUBJECT}"
        )),
        "{BOUND} names the subject:\n{bound}"
    );
    assert!(
        unbound.contains(
            "      evidence:\n        kind: falsification_attempt\n        result: survived"
        ) && !unbound.contains("subject:"),
        "{UNBOUND} names no subject:\n{unbound}"
    );
}

/// `subject` on an evidence match is part of `protocol/1`: the fixture validates.
#[test]
fn the_fixture_validates() {
    let run = canon(&["validate", "--path", FIXTURE]);
    assert_eq!(
        (text(&run.stdout), text(&run.stderr), run.status.code()),
        ("valid: protocol `investigation` revision 1\n", "", Some(0)),
        "canon validate --path {FIXTURE}"
    );
}

#[test]
fn canon_evidence_003_passes_under_conform_run() {
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

/// A fresh directory under this test target's scratch space.
fn scratch(name: &str) -> PathBuf {
    let dir =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("subject-bound-evidence-match-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("old scratch directory removed");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory created");
    dir
}

fn path(p: &Path) -> String {
    p.to_str().expect("utf-8 path").to_owned()
}

/// `canon evaluate --ir <file> --case <file> --evidence <dir>` over the compiled fixture, for a
/// case snapshot at the given revisions of the subject and the other artifact, with these records.
fn evaluate(name: &str, subject_revision: &str, other_revision: &str, given: &[Record]) -> Output {
    let dir = scratch(name);
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
    std::fs::write(
        &case,
        format!(
            "format: canon-case/1\nid: INV-18\nprotocol: investigation\nartifacts:\n  \
             {SUBJECT}: {{revision: {subject_revision}}}\n  {OTHER}: {{revision: {other_revision}}}\n"
        ),
    )
    .expect("case written");
    let evidence = dir.join("evidence");
    std::fs::create_dir(&evidence).expect("evidence directory created");
    for (index, record) in given.iter().enumerate() {
        let result = record
            .result
            .as_ref()
            .map_or(String::new(), |result| format!("result: {result}\n"));
        std::fs::write(
            evidence.join(format!("{index:02}.yaml")),
            format!(
                "format: canon-evidence/1\nid: {}\nkind: {}\n{result}subject: {}\nsubject_revision: {}\n",
                record.id, record.kind, record.subject, record.subject_revision
            ),
        )
        .expect("record written");
    }
    canon(&[
        "evaluate",
        "--ir",
        &path(&ir),
        "--case",
        &path(&case),
        "--evidence",
        &path(&evidence),
    ])
}

/// `canon evaluate` over the compiled fixture, given each evaluate step's case and records as
/// files, prints exactly the `canon-decision/1` bytes the step expects.
#[test]
fn canon_evaluate_prints_the_decision_each_step_expects() {
    let scenario = scenario();
    for (id, values) in STEPS {
        if values.is_none() {
            continue;
        }
        let (inputs, expected) = evaluate_step(step(&scenario, id));
        let EvaluateExpectation::Decision(decision) = expected else {
            panic!("step `{id}` expects a decision");
        };
        let run = evaluate(
            id,
            &current(inputs, SUBJECT),
            &current(inputs, OTHER),
            &records(inputs),
        );
        assert_eq!(
            (text(&run.stdout), text(&run.stderr), run.status.code()),
            (decision.as_str(), "", Some(0)),
            "step `{id}`: canon evaluate"
        );
    }
}

/// A record the binding stage excludes is listed under a claim only when a match the claim reaches
/// would read it. One about the dataset, bound to a superseded revision of it, is listed under the
/// unbound claim and under the `any` with a dataset member, and not under the claim whose match is
/// bound to the explanation: it does not match, so it was never that claim's evidence. One about
/// the explanation, bound to a superseded revision of it, is listed under all three.
#[test]
fn an_excluded_record_about_another_artifact_is_not_listed_under_a_subject_bound_claim() {
    let attempt = |id: &str, subject: &str| Record {
        id: id.to_owned(),
        kind: "falsification_attempt".to_owned(),
        result: Some("survived".to_owned()),
        subject: subject.to_owned(),
        subject_revision: "old".to_owned(),
    };
    let excluded = |id: &str| {
        format!(
            "\"excluded_evidence\": [\n        {{\n          \"evidence\": \"{id}\",\n          \
             \"reason\": \"revision_mismatch\"\n        }}\n      ],\n      "
        )
    };
    let decision = |bound_listed: &str, unbound_listed: &str, either_listed: &str| {
        format!(
            "{{\n  \"case\": \"INV-18\",\n  \"claims\": {{\n    \"attempt.survived\": {{\n      \
             {unbound_listed}\"value\": \"unknown\"\n    }},\n    \"either.survived\": {{\n      \
             {either_listed}\"value\": \"unknown\"\n    }},\n    \"explanation.survived\": {{\n      \
             {bound_listed}\"value\": \"unknown\"\n    }}\n  }},\n  \"format\": \"canon-decision/1\",\n  \
             \"protocol\": \"investigation\",\n  \"protocol_revision\": 1\n}}\n"
        )
    };

    let about_other = evaluate(
        "excluded-about-another-artifact",
        "r1",
        "d2",
        &[attempt("falsification-1", OTHER)],
    );
    let listed = excluded("falsification-1");
    assert_eq!(
        (
            text(&about_other.stdout),
            text(&about_other.stderr),
            about_other.status.code()
        ),
        (decision("", &listed, &listed).as_str(), "", Some(0)),
        "a superseded record about the dataset"
    );

    let about_subject = evaluate(
        "excluded-about-the-subject",
        "r2",
        "d1",
        &[attempt("falsification-1", SUBJECT)],
    );
    assert_eq!(
        (
            text(&about_subject.stdout),
            text(&about_subject.stderr),
            about_subject.status.code()
        ),
        (decision(&listed, &listed, &listed).as_str(), "", Some(0)),
        "a superseded record about the explanation"
    );
}

/// A match may name only an artifact the protocol declares: `canon validate` refuses one that
/// names another, naming the claim and the artifact.
#[test]
fn canon_validate_refuses_a_match_about_an_undeclared_artifact() {
    let dir = scratch("undeclared-subject");
    let protocol = dir.join("protocol.yaml");
    std::fs::write(
        &protocol,
        "format: protocol/1\nprotocol: {id: investigation, revision: 1}\n\
         artifacts:\n  explanation: {}\n\
         evidence_kinds:\n  falsification_attempt: {}\n\
         claims:\n  explanation.survived:\n    true_when:\n      evidence:\n        \
         kind: falsification_attempt\n        result: survived\n        subject: hypothesis\n",
    )
    .expect("protocol written");
    let run = canon(&["validate", "--path", &path(&protocol)]);
    assert_eq!(
        (text(&run.stdout), text(&run.stderr), run.status.code()),
        (
            "",
            "error[undeclared-artifact]: claim `explanation.survived` references artifact `hypothesis`, which is not declared\n",
            Some(1)
        ),
        "canon validate --path {}",
        protocol.display()
    );
}
