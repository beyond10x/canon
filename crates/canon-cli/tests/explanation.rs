//! Acceptance for story:explanation: conformance scenario `CANON-EXPLAIN-001` (also covering
//! CANON-DETERMINISM-001) passes under `canon conform run` over
//! `fixtures/investigation/explanation.yaml`, with three expectations for the investigation
//! fixture with its only evidence bound to a superseded revision: the `explanation` section traces
//! the blocked `supported` outcome to `explanation.supported` being `UNKNOWN` and from there to
//! each excluded evidence record with reason `revision_mismatch`; a second run gives byte-identical
//! output; and the same evidence set in reverse order gives byte-identical output. `canon evaluate`
//! prints the same bytes the scenario expects, whatever order the evidence files are read in.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use b10x_canon::conform::{self, EvaluateExpectation, EvaluateInputs, StepKind};

/// The scenario file this story adds to the registry.
const SCENARIO: &str = "conformance/scenarios/explanation.yaml";
/// The scenario's id, as `canon conform run` reports it.
const SCENARIO_ID: &str = "CANON-EXPLAIN-001";
/// The fixture: a copy of the base investigation protocol, so no other story edits it.
const FIXTURE: &str = "fixtures/investigation/explanation.yaml";
/// The base the fixture copies.
const BASE: &str = "fixtures/investigation/protocol.yaml";
/// The steps, in order.
const STEPS: [&str; 3] = ["superseded-revision", "second-run", "reversed-evidence"];
/// Every section the decision carries for this fixture: each step expects the whole decision, so
/// equal expectations mean byte-identical output.
const SECTIONS: [&str; 8] = [
    "actions",
    "case",
    "claims",
    "explanation",
    "format",
    "outcomes",
    "protocol",
    "protocol_revision",
];

/// The blocked `supported` outcome as the explanation writes it, traced to the claim.
const OUTCOME_TRACE: &str = "    \"outcomes\": {\n      \"supported\": {\n        \"because\": [\n          \
                             {\n            \"claim\": \"explanation.supported\",\n            \
                             \"value\": \"unknown\"\n          }\n        ],\n        \
                             \"status\": \"blocked\"\n      }\n    }\n";
/// The claim as the explanation writes it, traced to each excluded record.
const CLAIM_TRACE: &str = "    \"claims\": {\n      \"explanation.supported\": {\n        \"because\": [\n          \
                           {\n            \"evidence\": \"falsification-1\",\n            \
                           \"reason\": \"revision_mismatch\",\n            \"status\": \"excluded\"\n          \
                           },\n          {\n            \"evidence\": \"observation-1\",\n            \
                           \"reason\": \"revision_mismatch\",\n            \"status\": \"excluded\"\n          \
                           }\n        ],\n        \"value\": \"unknown\"\n      }\n    },\n";

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

/// Each step's inputs and expected decision bytes.
fn steps() -> Vec<(String, EvaluateInputs, String)> {
    scenario()
        .steps
        .into_iter()
        .map(|step| {
            let StepKind::Evaluate { inputs, expected } = step.kind else {
                panic!("{SCENARIO}: step `{}` is not an evaluate step", step.id);
            };
            let EvaluateExpectation::Decision(decision) = expected else {
                panic!("{SCENARIO}: step `{}` does not expect a decision", step.id);
            };
            (step.id, inputs, decision)
        })
        .collect()
}

/// The top-level keys an expected decision writes, in the order written: the lines indented by
/// exactly two spaces that open a member.
fn top_level_keys(decision: &str) -> Vec<&str> {
    decision
        .lines()
        .filter_map(|line| line.strip_prefix("  \""))
        .filter(|rest| !rest.starts_with(' '))
        .filter_map(|rest| rest.split_once('"').map(|(key, _)| key))
        .collect()
}

/// The `explanation` member of an expected decision, from its opening line to its closing one.
fn explanation_section(decision: &str) -> &str {
    let start = decision
        .find("\n  \"explanation\": {\n")
        .expect("the expectation writes an `explanation` section")
        + 1;
    let end = decision[start..]
        .find("\n  },\n")
        .expect("the `explanation` section closes")
        + start
        + "\n  },\n".len();
    &decision[start..end]
}

#[test]
fn the_scenario_holds_the_expectations_the_acceptance_names() {
    let scenario = scenario();
    assert_eq!(scenario.id, SCENARIO_ID);
    assert_eq!(scenario.covers, [SCENARIO_ID, "CANON-DETERMINISM-001"]);
    assert_eq!(scenario.fixture, FIXTURE);

    let steps = steps();
    let ids: Vec<&str> = steps.iter().map(|(id, _, _)| id.as_str()).collect();
    assert_eq!(ids, STEPS, "{SCENARIO}: one step per expectation, in order");
    let (_, first, expected) = &steps[0];

    // The only evidence is bound to r1, which the case snapshot has superseded with r2.
    assert_eq!(
        first.case["artifacts"]["explanation"]["revision"].as_str(),
        Some("r2"),
        "the case snapshot's current revision of the explanation"
    );
    assert_eq!(first.evidence.len(), 2, "two evidence records");
    for record in &first.evidence {
        assert_eq!(record["subject"].as_str(), Some("explanation"));
        assert_eq!(
            record["subject_revision"].as_str(),
            Some("r1"),
            "every record is bound to the superseded revision"
        );
    }
    assert!(
        first.authority.is_none() && first.decisions.is_none() && first.at.is_none(),
        "nothing supplied beyond the case and the evidence"
    );

    // The trace: the blocked outcome to the UNKNOWN claim, the claim to each excluded record.
    assert_eq!(top_level_keys(expected), SECTIONS, "the whole decision");
    let explanation = explanation_section(expected);
    assert!(
        explanation.contains(OUTCOME_TRACE),
        "the outcome traced to the claim\n{OUTCOME_TRACE}\nin\n{explanation}"
    );
    assert!(
        explanation.contains(CLAIM_TRACE),
        "the claim traced to each excluded record\n{CLAIM_TRACE}\nin\n{explanation}"
    );

    // A second run: the same inputs, the same bytes.
    let (_, second, second_expected) = &steps[1];
    assert_eq!(
        second, first,
        "step `second-run` repeats the first step's inputs"
    );
    assert_eq!(
        second_expected, expected,
        "step `second-run`: byte-identical"
    );

    // The same evidence set in reverse order: the same bytes.
    let (_, reversed, reversed_expected) = &steps[2];
    let mut evidence = first.evidence.clone();
    evidence.reverse();
    assert_ne!(
        evidence, first.evidence,
        "reversing the evidence changes its order"
    );
    assert_eq!(
        reversed,
        &EvaluateInputs {
            evidence,
            ..first.clone()
        },
        "step `reversed-evidence` is the first step with the evidence reversed"
    );
    assert_eq!(
        reversed_expected, expected,
        "step `reversed-evidence`: byte-identical"
    );
}

/// The fixture is the base, unedited.
#[test]
fn the_fixture_is_a_copy_of_the_base() {
    assert_eq!(read(FIXTURE), read(BASE));
}

#[test]
fn canon_explain_001_passes_under_conform_run() {
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
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("explanation-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("old scratch directory removed");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory created");
    dir
}

/// The first step's case snapshot, as a `canon-case/1` file.
const CASE: &str = "format: canon-case/1\nid: INV-18\nprotocol: investigation\n\
                    artifacts:\n  explanation: {revision: r2}\n";
/// The first step's evidence records, in the order it gives them, as `canon-evidence/1` files.
const EVIDENCE: [&str; 2] = [
    "format: canon-evidence/1\nid: observation-1\nkind: supporting_observation\n\
     subject: explanation\nsubject_revision: r1\n",
    "format: canon-evidence/1\nid: falsification-1\nkind: falsification_attempt\n\
     result: survived\nsubject: explanation\nsubject_revision: r1\n",
];

/// `canon evaluate` over the compiled fixture for the first step's case, with the evidence files
/// named so that the CLI, which reads them in file-name order, reads them in `order`.
fn evaluate(name: &str, order: &[usize]) -> Output {
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
    std::fs::write(&case, CASE).expect("case written");
    let evidence = dir.join("evidence");
    std::fs::create_dir(&evidence).expect("evidence directory created");
    for (position, index) in order.iter().enumerate() {
        std::fs::write(evidence.join(format!("{position}.yaml")), EVIDENCE[*index])
            .expect("evidence written");
    }
    let path = |p: &Path| p.to_str().expect("utf-8 path").to_owned();
    let args = [
        "evaluate".to_owned(),
        "--ir".to_owned(),
        path(&ir),
        "--case".to_owned(),
        path(&case),
        "--evidence".to_owned(),
        path(&evidence),
    ];
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    canon(&args)
}

/// `canon evaluate` prints exactly the decision the scenario expects, on a second run and with the
/// evidence files read in reverse order.
#[test]
fn canon_evaluate_prints_the_expected_decision_in_any_evidence_order() {
    let steps = steps();
    let (_, inputs, expected) = &steps[0];
    let ids: Vec<Option<&str>> = inputs
        .evidence
        .iter()
        .map(|record| record["id"].as_str())
        .collect();
    assert_eq!(
        ids,
        [Some("observation-1"), Some("falsification-1")],
        "the evidence files written here are the first step's, in its order"
    );
    for (name, order) in [
        ("given-order", [0, 1]),
        ("given-order-again", [0, 1]),
        ("reversed-order", [1, 0]),
    ] {
        let run = evaluate(name, &order);
        assert_eq!(text(&run.stderr), "", "{name}: canon evaluate stderr");
        assert_eq!(text(&run.stdout), expected, "{name}: canon evaluate stdout");
        assert_eq!(run.status.code(), Some(0), "{name}: exit");
    }
}
