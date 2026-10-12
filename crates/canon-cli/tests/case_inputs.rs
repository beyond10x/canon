//! Acceptance for story:case-inputs: a protocol declares case inputs, a case gives their values,
//! and obligations and preconditions depend on them. The fixture
//! `fixtures/investigation/case-inputs.yaml` declares the input `severity` (`low`, `high`), the
//! obligation `peer.review`, which applies when severity is `high`, the obligation
//! `falsification.recorded`, which applies when the claim `explanation.supported` holds, the
//! outcome `finding.published`, which requires every obligation that applies discharged, and the
//! action `publish_finding`, admissible when severity is `low` or `explanation.supported` is TRUE.
//!
//! One test, `case_inputs_derive_obligations`, holds expectations 1 to 12 of the story's
//! acceptance: conformance scenario `CANON-INPUT-001` passes under `canon conform run`, with a step
//! for each of expectations 2 to 6; `canon evaluate` refuses a case that leaves out an input, gives
//! a value the input does not declare or names an input the protocol does not declare, naming
//! them; and `canon validate` refuses each of the four invalid input fixtures, naming the input or
//! the value. Expectation 13 (`canon diff` over revision pairs) is held by
//! `diff_classifies_investigation_revisions` in `semantic_diff.rs`, over the pairs
//! `gained-applicability`, `lost-applicability`, `added-input-value` and `removed-input-value`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use b10x_canon::conform::{self, EvaluateExpectation, EvaluateInputs, StepKind};

/// The scenario file this story adds to the registry.
const SCENARIO: &str = "conformance/scenarios/case-inputs.yaml";
/// The scenario's id, as `canon conform run` reports it.
const SCENARIO_ID: &str = "CANON-INPUT-001";
/// The fixture the scenario compiles.
const FIXTURE: &str = "fixtures/investigation/case-inputs.yaml";

/// The scenario's steps, in order: id, the severity its case gives (`None`: none) and what it
/// expects, a decision or a refusal's code.
const STEPS: [(&str, Option<&str>, Expected); 7] = [
    ("low-no-evidence", Some("low"), Expected::Decision),
    (
        "low-supported-and-attempted",
        Some("low"),
        Expected::Decision,
    ),
    ("high-no-evidence", Some("high"), Expected::Decision),
    ("high-reviewed", Some("high"), Expected::Decision),
    ("severity-missing", None, Expected::Refused("missing-input")),
    (
        "severity-undeclared-value",
        Some("medium"),
        Expected::Refused("undeclared-input-value"),
    ),
    (
        "input-undeclared",
        Some("high"),
        Expected::Refused("undeclared-input"),
    ),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Expected {
    Decision,
    Refused(&'static str),
}

/// `peer.review` with `status`, as canonical `canon-decision/1` JSON writes the entry inside
/// `obligations`.
fn peer_review(status: &str) -> String {
    format!("\"id\": \"peer.review\",\n      \"status\": \"{status}\"\n    }}")
}

/// `falsification.recorded` open, its one reason naming the UNKNOWN claim it applies by.
const FALSIFICATION_OPEN: &str = "\"id\": \"falsification.recorded\",\n      \"reasons\": [\n        \
                                  {\n          \"claim\": \"explanation.supported\",\n          \
                                  \"value\": \"unknown\"\n        }\n      ],\n      \
                                  \"status\": \"open\"\n    }";
/// `publish_finding` admissible, inside `actions`.
const PUBLISH_ADMISSIBLE: &str = "\"publish_finding\": {\n      \"status\": \"admissible\"\n    }";
/// `publish_finding` blocked, its first reason naming `explanation.supported`, inside `actions`.
const PUBLISH_BLOCKED: &str = "\"publish_finding\": {\n      \"reasons\": [\n        {\n          \
                               \"claim\": \"explanation.supported\",\n          \
                               \"value\": \"unknown\"\n        },";
/// `finding.published` legitimate, inside `outcomes`.
const PUBLISHED: &str = "\"finding.published\": {\n      \"status\": \"legitimate\"\n    }";

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

fn evaluate_step(step: &conform::Step) -> (&EvaluateInputs, &EvaluateExpectation) {
    let StepKind::Evaluate { inputs, expected } = &step.kind else {
        panic!("{SCENARIO}: step `{}` is not an evaluate step", step.id);
    };
    (inputs, expected)
}

/// The `outcomes` section of a decision expectation, from its key to the end.
fn outcomes_of(decision: &str) -> &str {
    let at = decision
        .find("\"outcomes\": {")
        .unwrap_or_else(|| panic!("the expectation lists `outcomes`:\n{decision}"));
    &decision[at..]
}

/// Expectation 1, and 2 to 6 as the scenario's steps hold them: `canon conform run` reports the
/// scenario passed, so each step's expectation is what the evaluator decides.
fn scenario_holds_expectations_2_to_6() {
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

    let scenario = conform::parse(&read(SCENARIO))
        .unwrap_or_else(|error| panic!("{SCENARIO} does not parse: {error}"));
    assert_eq!(scenario.id, SCENARIO_ID);
    assert_eq!(scenario.covers, [SCENARIO_ID]);
    assert_eq!(scenario.fixture, FIXTURE);
    let ids: Vec<&str> = scenario.steps.iter().map(|step| step.id.as_str()).collect();
    assert_eq!(
        ids,
        STEPS.map(|(id, _, _)| id),
        "{SCENARIO}: the steps, in order"
    );

    let mut decisions = Vec::new();
    for (step, (id, severity, expected)) in scenario.steps.iter().zip(STEPS) {
        let (inputs, written) = evaluate_step(step);
        assert_eq!(
            inputs.case["inputs"]["severity"].as_str(),
            severity,
            "step `{id}`: the severity the case gives"
        );
        match (expected, written) {
            (Expected::Decision, EvaluateExpectation::Decision(decision)) => {
                decisions.push((id, inputs, decision.as_str()));
            }
            (Expected::Refused(code), EvaluateExpectation::Refusal(written)) => {
                assert_eq!(written, code, "step `{id}`: the refusal expected");
            }
            (_, written) => panic!("step `{id}`: the scenario expects {written:?}"),
        }
    }
    let &[low, low_settled, high, high_reviewed] = decisions.as_slice() else {
        panic!("four decision steps");
    };

    // 2. Severity low: `peer.review` is not_applicable, and `finding.published` is not held back
    // by it: no reason names its claim or the severity, and with the other obligation discharged
    // the outcome is legitimate without a peer review.
    let (_, inputs, decision) = low;
    assert!(
        inputs.evidence.is_empty(),
        "`low-no-evidence` gives no evidence"
    );
    assert!(
        decision.contains(&peer_review("not_applicable")),
        "{decision}"
    );
    let outcomes = outcomes_of(decision);
    assert!(outcomes.contains("\"status\": \"blocked\""), "{outcomes}");
    for held_back_by in ["peer.approved", "severity"] {
        assert!(
            !outcomes.contains(held_back_by),
            "{held_back_by} in\n{outcomes}"
        );
    }
    let (_, inputs, decision) = low_settled;
    assert_eq!(inputs.evidence.len(), 2, "an observation and an attempt");
    assert!(
        decision.contains(&peer_review("not_applicable")),
        "{decision}"
    );
    assert!(decision.contains(PUBLISHED), "{decision}");

    // 3. Severity high, no evidence: `peer.review` is open.
    let (_, inputs, decision) = high;
    assert!(
        inputs.evidence.is_empty(),
        "`high-no-evidence` gives no evidence"
    );
    assert!(decision.contains(&peer_review("open")), "{decision}");

    // 4. Severity high, with the discharging evidence: `peer.review` is discharged.
    let (_, inputs, decision) = high_reviewed;
    let kinds: Vec<Option<&str>> = inputs
        .evidence
        .iter()
        .map(|record| record["kind"].as_str())
        .collect();
    assert_eq!(kinds, [Some("peer_review")]);
    assert_eq!(inputs.evidence[0]["result"].as_str(), Some("approved"));
    assert!(decision.contains(&peer_review("discharged")), "{decision}");

    // 5. `publish_finding` admissible at severity low, blocked naming `explanation.supported` at
    // severity high, both without evidence.
    assert!(low.2.contains(PUBLISH_ADMISSIBLE), "{}", low.2);
    assert!(high.2.contains(PUBLISH_BLOCKED), "{}", high.2);

    // 6. No evidence: `falsification.recorded` is open, its reason naming `explanation.supported`.
    for (id, _, decision) in [low, high, high_reviewed] {
        assert!(
            decision.contains(FALSIFICATION_OPEN),
            "step `{id}`:\n{decision}"
        );
    }
}

/// A fresh directory under this test target's scratch space.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("case-inputs-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("old scratch directory removed");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory created");
    dir
}

/// `canon evaluate --ir <file> --case <file> --evidence <empty dir>` over the compiled fixture,
/// for a case snapshot that gives `inputs` (YAML lines under `inputs:`, or none).
fn evaluate(name: &str, inputs: &[&str]) -> Output {
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
    let mut case = "format: canon-case/1\nid: INV-18\nprotocol: investigation\n\
                    artifacts:\n  explanation: {revision: r1}\n"
        .to_owned();
    if !inputs.is_empty() {
        case.push_str("inputs:\n");
        for input in inputs {
            case.push_str(&format!("  {input}\n"));
        }
    }
    let case_path = dir.join("case.yaml");
    std::fs::write(&case_path, case).expect("case written");
    let evidence = dir.join("evidence");
    std::fs::create_dir(&evidence).expect("evidence directory created");
    let path = |p: &Path| p.to_str().expect("utf-8 path").to_owned();
    let args = [
        "evaluate".to_owned(),
        "--ir".to_owned(),
        path(&ir),
        "--case".to_owned(),
        path(&case_path),
        "--evidence".to_owned(),
        path(&evidence),
    ];
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    canon(&args)
}

/// Expectations 7 to 9: `canon evaluate` refuses a case whose inputs do not fit the protocol,
/// exiting 1 and naming what does not fit; a case that fits decides.
fn evaluate_refuses_inputs_that_do_not_fit() {
    let decided = evaluate("severity-low", &["severity: low"]);
    assert_eq!(
        text(&decided.stderr),
        "",
        "a case with severity low decides"
    );
    assert!(
        text(&decided.stdout).contains(&peer_review("not_applicable")),
        "{}",
        text(&decided.stdout)
    );
    assert_eq!(decided.status.code(), Some(0));

    for (name, inputs, expected) in [
        (
            "severity-missing",
            &[][..],
            "error[missing-input]: case `INV-18` does not give a value for input `severity`\n",
        ),
        (
            "severity-medium",
            &["severity: medium"][..],
            "error[undeclared-input-value]: case `INV-18` gives input `severity` the value \
             `medium`, which the input does not declare\n",
        ),
        (
            "urgency",
            &["severity: high", "urgency: soon"][..],
            "error[undeclared-input]: case `INV-18` gives input `urgency`, which the protocol \
             does not declare\n",
        ),
    ] {
        let run = evaluate(name, inputs);
        assert_eq!(text(&run.stderr), expected, "{name}: canon evaluate stderr");
        assert_eq!(text(&run.stdout), "", "{name}: canon evaluate stdout");
        assert_eq!(run.status.code(), Some(1), "{name}: exit");
    }
}

/// Expectations 10 to 12: `canon validate` refuses each invalid input fixture, exiting 1 and
/// naming the input or the value; the fixture itself validates.
fn validate_refuses_each_invalid_input_declaration() {
    let run = canon(&["validate", "--path", FIXTURE]);
    assert_eq!(
        (text(&run.stdout), text(&run.stderr), run.status.code()),
        ("valid: protocol `investigation` revision 1\n", "", Some(0)),
        "canon validate --path {FIXTURE}"
    );
    for (file, expected) in [
        (
            "input-empty-values.yaml",
            "error[empty-input-values]: input `urgency` declares no value\n",
        ),
        (
            "input-duplicate-value.yaml",
            "error[duplicate-input-value]: input `severity` lists value `low` more than once\n",
        ),
        (
            "input-undeclared.yaml",
            "error[undeclared-input]: action `publish_finding` references input `urgency`, which \
             is not declared\n",
        ),
        (
            "input-undeclared-value.yaml",
            "error[undeclared-input-value]: obligation `peer.review` tests input `severity` for \
             value `critical`, which the input does not declare\n",
        ),
    ] {
        let path = format!("fixtures/investigation/invalid/{file}");
        let run = canon(&["validate", "--path", &path]);
        assert_eq!(text(&run.stderr), expected, "{file}");
        assert_eq!(text(&run.stdout), "", "{file}");
        assert_eq!(run.status.code(), Some(1), "{file}");
    }
}

#[test]
fn case_inputs_derive_obligations() {
    scenario_holds_expectations_2_to_6();
    evaluate_refuses_inputs_that_do_not_fit();
    validate_refuses_each_invalid_input_declaration();
}
