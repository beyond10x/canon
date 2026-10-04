//! Acceptance for story:three-valued-claims: conformance scenario `CANON-CLAIM-001` passes under
//! `canon conform run`, with four expectations for `explanation.supported` (`UNKNOWN` with no
//! evidence, `FALSE` when the falsification attempt was refuted, `TRUE` when it survived, `UNKNOWN`
//! when two attempts disagree), and `canon evaluate --ir --case --evidence` prints the same
//! `canon-decision/1` bytes the scenario expects for each section it lists.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use b10x_canon::conform::{self, EvaluateExpectation, StepKind};

/// The scenario file this story adds to the registry.
const SCENARIO: &str = "conformance/scenarios/three-valued-claims.yaml";
/// The scenario's id, as `canon conform run` reports it.
const SCENARIO_ID: &str = "CANON-CLAIM-001";
/// The claim the four expectations are about.
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

fn scenario() -> conform::Scenario {
    let source = std::fs::read_to_string(repository_root().join(SCENARIO))
        .unwrap_or_else(|error| panic!("{SCENARIO}: {error}"));
    conform::parse(&source).unwrap_or_else(|error| panic!("{SCENARIO} does not parse: {error}"))
}

/// An evidence record by its kind and result.
type Record = (String, Option<String>);

/// The `(kind, result)` of each evidence record of an evaluate step, sorted.
fn evidence_of(inputs: &conform::EvaluateInputs) -> Vec<Record> {
    let mut evidence: Vec<Record> = inputs
        .evidence
        .iter()
        .map(|record| {
            (
                record["kind"]
                    .as_str()
                    .expect("an evidence record names its kind")
                    .to_owned(),
                record["result"].as_str().map(str::to_owned),
            )
        })
        .collect();
    evidence.sort();
    evidence
}

/// The four steps the acceptance names: id, evidence by `(kind, result)`, and the claim's value.
fn expected_steps() -> Vec<(&'static str, Vec<Record>, &'static str)> {
    let observation = || ("supporting_observation".to_owned(), None);
    let attempt = |result: &str| ("falsification_attempt".to_owned(), Some(result.to_owned()));
    let sorted = |mut evidence: Vec<Record>| {
        evidence.sort();
        evidence
    };
    vec![
        ("no-evidence", Vec::new(), "unknown"),
        (
            "refuted",
            sorted(vec![observation(), attempt("refuted")]),
            "false",
        ),
        (
            "survived",
            sorted(vec![observation(), attempt("survived")]),
            "true",
        ),
        (
            "conflicting",
            sorted(vec![observation(), attempt("survived"), attempt("refuted")]),
            "unknown",
        ),
    ]
}

/// The claim's entry as canonical `canon-decision/1` JSON writes it.
fn claim_entry(value: &str) -> String {
    format!("\"{CLAIM}\": {{\n      \"value\": \"{value}\"\n    }}")
}

#[test]
fn the_scenario_holds_the_four_expectations_the_acceptance_names() {
    let scenario = scenario();
    assert_eq!(scenario.id, SCENARIO_ID);
    assert_eq!(scenario.covers, ["CANON-CLAIM-001", "CANON-CLAIM-002"]);
    assert_eq!(
        scenario.fixture,
        "fixtures/investigation/three-valued-claims.yaml"
    );
    let expected = expected_steps();
    assert_eq!(
        scenario.steps.len(),
        expected.len(),
        "{SCENARIO}: one step per expectation"
    );
    for (step, (id, evidence, value)) in scenario.steps.iter().zip(expected) {
        assert_eq!(step.id, id, "{SCENARIO}: step order");
        let StepKind::Evaluate { inputs, expected } = &step.kind else {
            panic!("{SCENARIO}: step `{id}` is not an evaluate step");
        };
        assert_eq!(evidence_of(inputs), evidence, "step `{id}`: evidence");
        let EvaluateExpectation::Decision(decision) = expected else {
            panic!("{SCENARIO}: step `{id}` expects a refusal, not a decision");
        };
        assert!(
            decision.contains(&claim_entry(value)),
            "step `{id}`: expected {CLAIM} = {value} in\n{decision}"
        );
    }
}

#[test]
fn canon_claim_001_passes_under_conform_run() {
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

/// The case snapshot every step evaluates, as a `canon-case/1` file.
const CASE: &str = "format: canon-case/1\n\
                    id: INV-18\n\
                    protocol: investigation\n\
                    artifacts:\n  explanation: {revision: r1}\n";

/// One `canon-evidence/1` record about the explanation at its current revision.
fn record(id: &str, kind: &str, result: Option<&str>) -> String {
    let result = result.map_or(String::new(), |result| format!("result: {result}\n"));
    format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\n{result}subject: explanation\nsubject_revision: r1\n"
    )
}

/// The evidence files of each step, the same records the scenario lists.
fn evidence_files(step: &str) -> Vec<String> {
    let observation = record("observation-1", "supporting_observation", None);
    let attempt = |id: &str, result: &str| record(id, "falsification_attempt", Some(result));
    match step {
        "no-evidence" => Vec::new(),
        "refuted" => vec![observation, attempt("falsification-1", "refuted")],
        "survived" => vec![observation, attempt("falsification-1", "survived")],
        "conflicting" => vec![
            observation,
            attempt("falsification-1", "survived"),
            attempt("falsification-2", "refuted"),
        ],
        other => panic!("no evidence files for step `{other}`"),
    }
}

/// A fresh directory under this test target's scratch space.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("three-valued-claims-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("old scratch directory removed");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory created");
    dir
}

/// `canon evaluate --ir <file> --case <file> --evidence <dir>`: the IR is what `canon compile`
/// prints for the scenario's fixture, the case one `canon-case/1` document, and the evidence a
/// directory holding one `canon-evidence/1` document per `*.yaml` file.
fn evaluate(fixture: &str, step: &str) -> Output {
    let dir = scratch(step);
    let compiled = canon(&["compile", "--path", fixture]);
    assert_eq!(compiled.status.code(), Some(0), "the fixture compiles");
    let ir = dir.join("protocol.ir.json");
    std::fs::write(&ir, &compiled.stdout).expect("IR written");
    let case = dir.join("case.yaml");
    std::fs::write(&case, CASE).expect("case written");
    let evidence = dir.join("evidence");
    std::fs::create_dir(&evidence).expect("evidence directory created");
    for (index, record) in evidence_files(step).iter().enumerate() {
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
    ])
}

/// The top-level members of canonical JSON `document`, each its lines with the separating comma
/// removed, in order. A member opens on a line indented by exactly two spaces.
fn members(document: &str) -> Vec<(String, Vec<&str>)> {
    let lines: Vec<&str> = document.lines().collect();
    let body = &lines[1..lines.len().saturating_sub(1)];
    let mut members: Vec<(String, Vec<&str>)> = Vec::new();
    for line in body {
        if let Some(rest) = line.strip_prefix("  \"")
            && !rest.starts_with(' ')
        {
            let key = rest.split('"').next().unwrap_or_default().to_owned();
            members.push((key, Vec::new()));
        }
        let (_, member) = members.last_mut().expect("a member opens first");
        member.push(line);
    }
    for (_, member) in &mut members {
        let last = member.last_mut().expect("a member has a line");
        *last = last.strip_suffix(',').unwrap_or(last);
    }
    members
}

/// `document` holding only the top-level members `expected` lists, written as canonical JSON.
fn listed_sections(document: &str, expected: &str) -> String {
    let listed: Vec<String> = members(expected).into_iter().map(|(key, _)| key).collect();
    let kept: Vec<String> = members(document)
        .into_iter()
        .filter(|(key, _)| listed.contains(key))
        .map(|(_, member)| member.join("\n"))
        .collect();
    format!("{{\n{}\n}}\n", kept.join(",\n"))
}

#[test]
fn canon_evaluate_prints_the_decision_each_step_expects() {
    let scenario = scenario();
    assert_eq!(scenario.steps.len(), 4, "{SCENARIO}: four steps");
    for step in &scenario.steps {
        let StepKind::Evaluate {
            expected: EvaluateExpectation::Decision(decision),
            ..
        } = &step.kind
        else {
            panic!("step `{}` does not expect a decision", step.id);
        };
        let run = evaluate(&scenario.fixture, &step.id);
        assert_eq!(
            text(&run.stderr),
            "",
            "step `{}`: canon evaluate stderr",
            step.id
        );
        // Changed by story:action-admissibility: the decision of this fixture now carries an
        // `actions` section the expectation does not list. Like `canon conform run`, compare the
        // sections the expectation lists, each byte for byte as `canon evaluate` printed it.
        assert_eq!(
            listed_sections(text(&run.stdout), decision),
            *decision,
            "step `{}`: canon evaluate stdout",
            step.id
        );
        assert_eq!(run.status.code(), Some(0), "step `{}`: exit", step.id);
    }
}

/// A refusal exits 1 naming its code on standard error; an input that cannot be read exits 2.
#[test]
fn canon_evaluate_exits_1_on_a_refusal_and_2_on_an_unreadable_input() {
    let fixture = scenario().fixture;
    // The step's files, then the IR made non-canonical: it is refused, not evaluated.
    let dir = scratch("refusal");
    let compiled = canon(&["compile", "--path", &fixture]);
    let ir = dir.join("protocol.ir.json");
    let ir_text = String::from_utf8(compiled.stdout).expect("utf-8 IR");
    std::fs::write(&ir, ir_text.trim_end()).expect("IR written");
    let case = dir.join("case.yaml");
    std::fs::write(&case, CASE).expect("case written");
    let evidence = dir.join("evidence");
    std::fs::create_dir(&evidence).expect("evidence directory created");
    let path = |p: &Path| p.to_str().expect("utf-8 path").to_owned();
    let args = |ir: &Path, evidence: &Path| {
        vec![
            "evaluate".to_owned(),
            "--ir".to_owned(),
            path(ir),
            "--case".to_owned(),
            path(&case),
            "--evidence".to_owned(),
            path(evidence),
        ]
    };
    let run = |args: Vec<String>| {
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        canon(&args)
    };

    let refused = run(args(&ir, &evidence));
    assert_eq!(text(&refused.stdout), "", "refusal: stdout");
    assert!(
        text(&refused.stderr)
            .starts_with("error[malformed-input]: not canon-ir/1: not the canonical text"),
        "refusal: stderr {}",
        text(&refused.stderr)
    );
    assert_eq!(refused.status.code(), Some(1), "refusal: exit");

    let missing = run(args(&ir, &dir.join("no-such-directory")));
    assert_eq!(text(&missing.stdout), "", "unreadable: stdout");
    assert!(
        text(&missing.stderr).starts_with("error[unreadable]: "),
        "unreadable: stderr {}",
        text(&missing.stderr)
    );
    assert_eq!(missing.status.code(), Some(2), "unreadable: exit");
}

/// The evidence directory reads `*.yaml` and `*.json` records alike, and refuses any other entry
/// by name (exit 2) instead of dropping it.
#[test]
fn the_evidence_directory_reads_yaml_and_json_and_refuses_anything_else() {
    let fixture = scenario().fixture;
    let dir = scratch("evidence-directory");
    let compiled = canon(&["compile", "--path", &fixture]);
    let ir = dir.join("protocol.ir.json");
    std::fs::write(&ir, &compiled.stdout).expect("IR written");
    let case = dir.join("case.yaml");
    std::fs::write(&case, CASE).expect("case written");
    let evidence = dir.join("evidence");
    std::fs::create_dir(&evidence).expect("evidence directory created");
    let path = |p: &Path| p.to_str().expect("utf-8 path").to_owned();
    let run = || {
        canon(&[
            "evaluate",
            "--ir",
            &path(&ir),
            "--case",
            &path(&case),
            "--evidence",
            &path(&evidence),
        ])
    };
    // The `survived` step's records, one of them as JSON: TRUE, so the JSON record was read.
    std::fs::write(
        evidence.join("00.yaml"),
        record("observation-1", "supporting_observation", None),
    )
    .expect("written");
    std::fs::write(
        evidence.join("01.json"),
        "{\"format\": \"canon-evidence/1\", \"id\": \"falsification-1\", \"kind\": \"falsification_attempt\", \"result\": \"survived\", \"subject\": \"explanation\", \"subject_revision\": \"r1\"}\n",
    )
    .expect("written");
    let read = run();
    assert_eq!(read.status.code(), Some(0), "{}", text(&read.stderr));
    assert!(text(&read.stdout).contains("\"value\": \"true\""));

    for (name, make) in [("02.txt", true), ("02.yml", true), ("nested.yaml", false)] {
        let other = evidence.join(name);
        if make {
            std::fs::write(&other, "not read").expect("written");
        } else {
            std::fs::create_dir(&other).expect("created");
        }
        let refused = run();
        assert_eq!(refused.status.code(), Some(2), "{name}");
        assert_eq!(text(&refused.stdout), "", "{name}");
        assert!(
            text(&refused.stderr).starts_with(&format!(
                "error[unreadable]: {}: not an evidence record",
                path(&other)
            )),
            "{name}: {}",
            text(&refused.stderr)
        );
        if make {
            std::fs::remove_file(&other).expect("removed");
        } else {
            std::fs::remove_dir(&other).expect("removed");
        }
    }
}
