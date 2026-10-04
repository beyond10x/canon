//! Acceptance for story:decision-outcomes: conformance scenario `CANON-OUTCOME-002` passes under
//! `canon conform run` over `fixtures/investigation/decision-outcomes.yaml`, with three
//! expectations: `inconclusive` is `blocked` naming `explicitly_inconclusive` with no decision
//! input; `legitimate` with a `canon-decisions/1` decision at the current case revision; `blocked`
//! again when that decision names a superseded case revision. A legitimate decided outcome records
//! who decided (`decided_by`, adversary pass 1 finding F4), and a fourth step gives the right
//! decision for another outcome, `supported`, which is refused as `undeclared-decision` (finding F5;
//! both coordinator decisions). `canon evaluate --decisions` prints the same `outcomes`, or refuses
//! with the same code, as the scenario expects.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use b10x_canon::conform::{self, EvaluateExpectation, EvaluateInputs, StepKind};

/// The scenario file this story adds to the registry.
const SCENARIO: &str = "conformance/scenarios/decision-outcomes.yaml";
/// The scenario's id, as `canon conform run` reports it.
const SCENARIO_ID: &str = "CANON-OUTCOME-002";
/// The fixture: the base investigation protocol plus one outcome that requires a decision.
const FIXTURE: &str = "fixtures/investigation/decision-outcomes.yaml";
/// The base the fixture extends, unedited.
const BASE: &str = "fixtures/investigation/protocol.yaml";
/// What the fixture adds to the base, and nothing else: the design § 12 `inconclusive` outcome.
const ADDED: &str = "\n  inconclusive:\n    description: The investigation ends without deciding the \
                     explanation, by an explicit decision.\n    requires:\n      decision: \
                     explicitly_inconclusive\n";

/// `inconclusive` blocked, naming the decision it requires, as canonical `canon-decision/1` JSON
/// writes the entry inside `outcomes`.
const BLOCKED: &str = "\"inconclusive\": {\n      \"reasons\": [\n        {\n          \
                       \"decision\": \"explicitly_inconclusive\",\n          \"present\": false\n        \
                       }\n      ],\n      \"status\": \"blocked\"\n    }";
/// `inconclusive` legitimate, with no reasons, recording who decided.
const LEGITIMATE: &str = "\"inconclusive\": {\n      \"decided_by\": {\n        \
                          \"decision\": \"explicitly_inconclusive\",\n        \
                          \"principals\": [\n          \"lead-investigator\"\n        ]\n      },\n      \
                          \"status\": \"legitimate\"\n    }";

/// What a step expects: the `inconclusive` entry of a decision, or a refusal's code.
#[derive(Clone, Copy)]
enum Expected {
    Entry(&'static str),
    Refused(&'static str),
}

/// The steps, in order: id, the case snapshot's revision, the outcome the one explicit decision
/// `explicitly_inconclusive` (taken at case revision `c2`) is given for, if any, and the
/// expectation.
const STEPS: [(&str, &str, Option<&str>, Expected); 4] = [
    ("no-decision", "c2", None, Expected::Entry(BLOCKED)),
    (
        "decided-at-current-revision",
        "c2",
        Some("inconclusive"),
        Expected::Entry(LEGITIMATE),
    ),
    (
        "decided-at-superseded-revision",
        "c3",
        Some("inconclusive"),
        Expected::Entry(BLOCKED),
    ),
    (
        "decided-for-another-outcome",
        "c2",
        Some("supported"),
        Expected::Refused("undeclared-decision"),
    ),
];

/// One explicit decision as a step gives it: its decision, outcome and case revision.
type Given<'a> = (Option<&'a str>, Option<&'a str>, Option<&'a str>);

/// The one explicit decision a step gives, taken for `outcome` at case revision `c2`, as a
/// `canon-decisions/1` document.
fn decisions(outcome: &str) -> String {
    format!(
        "- decision: explicitly_inconclusive\n  outcome: {outcome}\n  principal: lead-investigator\n  \
         case_revision: c2\n"
    )
}

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

fn evaluate_step(step: &conform::Step) -> (&EvaluateInputs, &EvaluateExpectation) {
    let StepKind::Evaluate { inputs, expected } = &step.kind else {
        panic!("{SCENARIO}: step `{}` is not an evaluate step", step.id);
    };
    (inputs, expected)
}

#[test]
fn the_scenario_holds_the_expectations_the_acceptance_names() {
    let scenario = scenario();
    assert_eq!(scenario.id, SCENARIO_ID);
    assert_eq!(scenario.covers, [SCENARIO_ID]);
    assert_eq!(scenario.fixture, FIXTURE);
    assert_eq!(
        scenario.steps.len(),
        STEPS.len(),
        "{SCENARIO}: one step per expectation"
    );
    for (step, (id, revision, decided_for, expected)) in scenario.steps.iter().zip(STEPS) {
        assert_eq!(step.id, id, "{SCENARIO}: step order");
        let (inputs, written) = evaluate_step(step);
        assert_eq!(
            inputs.case["revision"].as_str(),
            Some(revision),
            "step `{id}`: the case snapshot's revision"
        );
        let given: Option<Vec<Given>> = inputs.decisions.as_ref().map(|list| {
            list.iter()
                .map(|entry| {
                    (
                        entry["decision"].as_str(),
                        entry["outcome"].as_str(),
                        entry["case_revision"].as_str(),
                    )
                })
                .collect()
        });
        assert_eq!(
            given,
            decided_for.map(|outcome| vec![(
                Some("explicitly_inconclusive"),
                Some(outcome),
                Some("c2")
            )]),
            "step `{id}`: the explicit decisions given"
        );
        assert!(
            inputs.evidence.is_empty(),
            "step `{id}`: no evidence, so only the decision can make `inconclusive` legitimate"
        );
        match (expected, written) {
            (Expected::Entry(entry), EvaluateExpectation::Decision(decision)) => assert!(
                decision.contains(entry),
                "step `{id}`: expected the `inconclusive` entry\n{entry}\nin\n{decision}"
            ),
            (Expected::Refused(code), EvaluateExpectation::Refusal(written)) => {
                assert_eq!(written, code, "step `{id}`: the refusal expected");
            }
            (_, written) => panic!("step `{id}`: the scenario expects {written:?}"),
        }
    }
}

/// The fixture is the base with the design § 12 `inconclusive` outcome appended, and nothing else
/// changed.
#[test]
fn the_fixture_is_the_base_plus_an_outcome_that_requires_a_decision() {
    assert_eq!(read(FIXTURE), format!("{}{ADDED}", read(BASE)));
}

/// `requires: decision:` is part of `protocol/1`: the fixture validates.
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
fn canon_outcome_002_passes_under_conform_run() {
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
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("decision-outcomes-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("old scratch directory removed");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory created");
    dir
}

/// `canon evaluate --ir <file> --case <file> --evidence <empty dir> [--decisions <file>]` over the
/// compiled fixture, for a case snapshot at `revision`.
fn evaluate(step: &str, revision: &str, decisions: Option<&str>) -> Output {
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
    std::fs::write(
        &case,
        format!(
            "format: canon-case/1\nid: INV-18\nprotocol: investigation\nrevision: {revision}\n\
             artifacts:\n  explanation: {{revision: r1}}\n"
        ),
    )
    .expect("case written");
    let evidence = dir.join("evidence");
    std::fs::create_dir(&evidence).expect("evidence directory created");
    let path = |p: &Path| p.to_str().expect("utf-8 path").to_owned();
    let mut args = vec![
        "evaluate".to_owned(),
        "--ir".to_owned(),
        path(&ir),
        "--case".to_owned(),
        path(&case),
        "--evidence".to_owned(),
        path(&evidence),
    ];
    if let Some(decisions) = decisions {
        let file = dir.join("decisions.yaml");
        std::fs::write(&file, decisions).expect("decisions written");
        args.extend(["--decisions".to_owned(), path(&file)]);
    }
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    canon(&args)
}

/// `canon evaluate`, given the decisions each step gives, prints the `inconclusive` entry the step
/// expects, or refuses with the code it expects (exit 1, nothing on standard output).
#[test]
fn canon_evaluate_decisions_prints_the_outcome_each_step_expects() {
    for (id, revision, decided_for, expected) in STEPS {
        let run = evaluate(id, revision, decided_for.map(decisions).as_deref());
        match expected {
            Expected::Entry(entry) => {
                assert_eq!(text(&run.stderr), "", "step `{id}`: canon evaluate stderr");
                assert!(
                    text(&run.stdout).contains(entry),
                    "step `{id}`: expected the `inconclusive` entry\n{entry}\nin\n{}",
                    text(&run.stdout)
                );
                assert_eq!(run.status.code(), Some(0), "step `{id}`: exit");
            }
            Expected::Refused(code) => {
                assert!(
                    text(&run.stderr).starts_with(&format!("error[{code}]: ")),
                    "step `{id}`: canon evaluate stderr {}",
                    text(&run.stderr)
                );
                assert_eq!(text(&run.stdout), "", "step `{id}`: canon evaluate stdout");
                assert_eq!(run.status.code(), Some(1), "step `{id}`: exit");
            }
        }
    }
}
