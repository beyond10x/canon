//! Acceptance for story:invalidation-rules: conformance scenario `CANON-INVALIDATION-001` passes
//! under `canon conform run` over `fixtures/investigation/invalidation-rules.yaml`. The fixture
//! declares one invalidation rule, `dataset.revised`: a change of the dataset's revision
//! invalidates the support of `explanation.supported`. Its three expectations: the claim is `TRUE`
//! while the dataset is at the revision the evidence recorded; `UNKNOWN`, not `FALSE`, with the
//! evidence listed as excluded for `invalidated`, once only the dataset's revision moves; and a
//! rule naming an undeclared artifact is refused by `canon validate`, naming it.
//!
//! Exclusion is per claim (coordinator decision, wave 2026-10-04-w11): the invalidated record is
//! excluded from the claims the rule names and from every claim built on one of them, and no other
//! claim. So once the dataset moves, `explanation.plausible`, built on the named claim and reading
//! the same kind directly, is `UNKNOWN` and lists the record; `observation.recorded`, which reads
//! the same kind but is neither named nor built on a named claim, stays `TRUE` and lists nothing;
//! and `explanation.survived`, of another kind, stays `TRUE`. Beside them, `canon evaluate` prints
//! the bytes each step expects, and a rule naming an undeclared claim is refused too.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use b10x_canon::conform::{self, EvaluateExpectation, EvaluateInputs, StepKind};

/// The scenario file this story adds to the registry.
const SCENARIO: &str = "conformance/scenarios/invalidation-rules.yaml";
/// The scenario's id, as `canon conform run` reports it.
const SCENARIO_ID: &str = "CANON-INVALIDATION-001";
/// The fixture only this story's scenario names.
const FIXTURE: &str = "fixtures/investigation/invalidation-rules.yaml";
/// The claim the rule names.
const NAMED: &str = "explanation.supported";
/// A claim built on the named claim that also reads the named claim's kind directly.
const BUILT_ON: &str = "explanation.plausible";
/// A claim that reads the named claim's kind, neither named nor built on a named claim.
const SAME_KIND: &str = "observation.recorded";
/// A claim of another kind.
const OTHER_KIND: &str = "explanation.survived";
/// The rule's upstream artifact.
const UPSTREAM: &str = "dataset";
/// The record of the kind the named claim reads.
const INVALIDATED: &str = "observation-1";

/// One claim in one step: the claim, its value, and whether it lists the invalidated record.
type Expected = (&'static str, &'static str, bool);

/// The evaluate steps, in order after `compile`: id, and what each claim takes. Every claim the
/// fixture declares is here.
const STEPS: [(&str, [Expected; 4]); 2] = [
    (
        "upstream-at-recorded-revision",
        [
            (BUILT_ON, "true", false),
            (NAMED, "true", false),
            (OTHER_KIND, "true", false),
            (SAME_KIND, "true", false),
        ],
    ),
    (
        "upstream-revision-moved",
        [
            (BUILT_ON, "unknown", true),
            (NAMED, "unknown", true),
            (OTHER_KIND, "true", false),
            (SAME_KIND, "true", false),
        ],
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

fn evaluate_step<'a>(
    scenario: &'a conform::Scenario,
    id: &str,
) -> (&'a EvaluateInputs, &'a EvaluateExpectation) {
    let step = scenario
        .steps
        .iter()
        .find(|step| step.id == id)
        .unwrap_or_else(|| panic!("{SCENARIO}: no step `{id}`"));
    let StepKind::Evaluate { inputs, expected } = &step.kind else {
        panic!("{SCENARIO}: step `{id}` is not an evaluate step");
    };
    (inputs, expected)
}

/// One evidence record as a step gives it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Record {
    id: String,
    kind: String,
    result: Option<String>,
    subject: String,
    subject_revision: String,
    /// The dataset revision the record was observed against.
    upstream: String,
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
            let upstream = record["upstream_revisions"][UPSTREAM]
                .as_str()
                .unwrap_or_else(|| panic!("each record records the {UPSTREAM} revision"))
                .to_owned();
            Record {
                id: get("id"),
                kind: get("kind"),
                result: record["result"].as_str().map(str::to_owned),
                subject: get("subject"),
                subject_revision: get("subject_revision"),
                upstream,
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

/// A claim's entry as canonical `canon-decision/1` JSON writes it: with the invalidated record
/// listed, or with nothing excluded.
fn claim_entry(claim: &str, value: &str, lists_record: bool) -> String {
    if lists_record {
        format!(
            "\"{claim}\": {{\n      \"excluded_evidence\": [\n        {{\n          \"evidence\": \"{INVALIDATED}\",\n          \"reason\": \"invalidated\"\n        }}\n      ],\n      \"value\": \"{value}\"\n    }}"
        )
    } else {
        format!("\"{claim}\": {{\n      \"value\": \"{value}\"\n    }}")
    }
}

#[test]
fn the_scenario_holds_the_expectations_the_acceptance_names() {
    let scenario = scenario();
    assert_eq!(scenario.id, SCENARIO_ID);
    assert_eq!(scenario.covers, [SCENARIO_ID]);
    assert_eq!(scenario.fixture, FIXTURE);
    let ids: Vec<&str> = scenario.steps.iter().map(|step| step.id.as_str()).collect();
    let mut expected_ids = vec!["compile"];
    expected_ids.extend(STEPS.iter().map(|(id, _)| *id));
    assert_eq!(ids, expected_ids, "{SCENARIO}: step order");
    assert!(
        matches!(scenario.steps[0].kind, StepKind::Compile { .. }),
        "{SCENARIO}: the first step compiles the fixture"
    );

    for (id, claims) in STEPS {
        let (inputs, expected) = evaluate_step(&scenario, id);
        // Every record is about its subject's current revision, so the binding stage excludes
        // nothing, and records no instant, so freshness excludes nothing: only the rule can.
        for record in records(inputs) {
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
        for (claim, value, lists_record) in claims {
            assert!(
                decision.contains(&claim_entry(claim, value, lists_record)),
                "step `{id}`: expected {claim} = {value}, invalidated record listed: \
                 {lists_record}, in\n{decision}"
            );
        }
        assert_eq!(
            decision.matches("\"value\": ").count(),
            claims.len(),
            "step `{id}`: the decision gives exactly the claims listed here"
        );
    }

    // At the recorded revision every record was observed against the dataset's current revision.
    let (recorded, _) = evaluate_step(&scenario, STEPS[0].0);
    for record in records(recorded) {
        assert_eq!(record.upstream, current(recorded, UPSTREAM));
    }
    // Once it moves, only the dataset's revision differs: the same records, the same subject
    // revisions, and a case snapshot that differs in the dataset's revision alone.
    let (moved, _) = evaluate_step(&scenario, STEPS[1].0);
    assert_eq!(records(moved), records(recorded), "the same records");
    assert_ne!(current(moved, UPSTREAM), current(recorded, UPSTREAM));
    let mut case = moved.case.clone();
    case["artifacts"][UPSTREAM] = recorded.case["artifacts"][UPSTREAM].clone();
    assert_eq!(
        case, recorded.case,
        "the case differs in the dataset's revision only"
    );
    let kinds: Vec<(String, String)> = records(moved)
        .into_iter()
        .map(|record| (record.id, record.kind))
        .collect();
    assert_eq!(
        kinds,
        [
            (INVALIDATED.to_owned(), "supporting_observation".to_owned()),
            (
                "falsification-1".to_owned(),
                "falsification_attempt".to_owned()
            ),
        ]
    );
}

/// The declaration of `id` in a fixture section: from its key line to the next blank line.
fn declaration(fixture: &str, id: &str) -> String {
    let start = fixture
        .find(&format!("\n  {id}:\n"))
        .unwrap_or_else(|| panic!("{FIXTURE} declares {id}"));
    let body = &fixture[start + 1..];
    let end = body[1..].find("\n\n").map_or(body.len(), |at| at + 1);
    body[..end].to_owned()
}

/// The fixture's claims have the shapes the per-claim expectations rely on: the rule names one
/// claim; one unnamed claim reads its kind and tests no claim; one is built on the named claim and
/// reads its kind directly too.
#[test]
fn the_fixture_separates_named_built_on_and_unnamed_claims() {
    let fixture = read(FIXTURE);
    let kind = "      evidence:\n        kind: supporting_observation";
    let named = declaration(&fixture, NAMED);
    assert!(
        named.contains(kind) && !named.contains("claim:"),
        "{NAMED}:\n{named}"
    );
    let same_kind = declaration(&fixture, SAME_KIND);
    assert!(
        same_kind.contains(kind) && !same_kind.contains("claim:"),
        "{SAME_KIND} reads the kind and tests no claim:\n{same_kind}"
    );
    let built_on = declaration(&fixture, BUILT_ON);
    assert!(
        built_on.contains(&format!(
            "      any:\n        - claim: {NAMED}\n        - evidence:\n            kind: supporting_observation"
        )),
        "{BUILT_ON} is built on {NAMED} and reads the kind directly:\n{built_on}"
    );
    let rule = declaration(&fixture, "dataset.revised");
    assert!(
        rule.ends_with(&format!(
            "    upstream: {UPSTREAM}\n    invalidates:\n      - {NAMED}\n"
        )),
        "the rule names {NAMED} alone:\n{rule}"
    );
}

/// `invalidation:` is part of `protocol/1`: the fixture validates.
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
fn canon_invalidation_001_passes_under_conform_run() {
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
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("invalidation-rules-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("old scratch directory removed");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory created");
    dir
}

fn path(p: &Path) -> String {
    p.to_str().expect("utf-8 path").to_owned()
}

/// `canon validate` over the fixture with one edit, written to scratch.
fn validate_edited(name: &str, from: &str, to: &str) -> Output {
    let fixture = read(FIXTURE);
    assert_eq!(
        fixture.matches(from).count(),
        1,
        "{FIXTURE} holds `{from}` once"
    );
    let file = scratch(name).join("protocol.yaml");
    std::fs::write(&file, fixture.replace(from, to)).expect("edited fixture written");
    canon(&["validate", "--path", &path(&file)])
}

/// Acceptance, third expectation: a rule whose upstream artifact the protocol does not declare is
/// refused, naming the rule and the artifact. The unedited fixture validates
/// (`the_fixture_validates`), so the edit alone is refused.
#[test]
fn canon_validate_refuses_a_rule_naming_an_undeclared_artifact() {
    let run = validate_edited(
        "undeclared-artifact",
        "    upstream: dataset\n",
        "    upstream: hypothesis\n",
    );
    assert_eq!(
        (text(&run.stdout), text(&run.stderr), run.status.code()),
        (
            "",
            "error[undeclared-artifact]: invalidation rule `dataset.revised` references artifact `hypothesis`, which is not declared\n",
            Some(1)
        )
    );
}

/// A rule naming a claim the protocol does not declare is refused, naming the rule and the claim.
#[test]
fn canon_validate_refuses_a_rule_naming_an_undeclared_claim() {
    let run = validate_edited(
        "undeclared-claim",
        "      - explanation.supported\n",
        "      - explanation.refuted\n",
    );
    assert_eq!(
        (text(&run.stdout), text(&run.stderr), run.status.code()),
        (
            "",
            "error[undeclared-claim]: invalidation rule `dataset.revised` references claim `explanation.refuted`, which is not declared\n",
            Some(1)
        )
    );
}

/// A printed decision without its `explanation` section (story:explanation), which every decision
/// carries and the scenario's expectations, listing the sections they compare, leave out. The
/// section is a top-level member of the canonical JSON and is followed by `format`.
fn without_explanation(decision: &str) -> String {
    let start = decision
        .find("\n  \"explanation\": {\n")
        .unwrap_or_else(|| panic!("every decision carries an explanation: {decision}"));
    let end = start
        + decision[start..]
            .find("\n  },\n")
            .expect("the explanation section closes");
    format!(
        "{}{}",
        &decision[..start],
        &decision[end + "\n  },".len()..]
    )
}

/// `canon evaluate` over the compiled fixture, given each evaluate step's case and records as
/// files, prints exactly the `canon-decision/1` bytes the step expects.
#[test]
fn canon_evaluate_prints_the_decision_each_step_expects() {
    let scenario = scenario();
    for (id, _) in STEPS {
        let (inputs, expected) = evaluate_step(&scenario, id);
        let EvaluateExpectation::Decision(decision) = expected else {
            panic!("step `{id}` expects a decision");
        };
        let dir = scratch(id);
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
                 explanation: {{revision: {}}}\n  {UPSTREAM}: {{revision: {}}}\n",
                current(inputs, "explanation"),
                current(inputs, UPSTREAM)
            ),
        )
        .expect("case written");
        let evidence = dir.join("evidence");
        std::fs::create_dir(&evidence).expect("evidence directory created");
        for (index, record) in records(inputs).iter().enumerate() {
            let result = record
                .result
                .as_ref()
                .map_or(String::new(), |result| format!("result: {result}\n"));
            std::fs::write(
                evidence.join(format!("{index:02}.yaml")),
                format!(
                    "format: canon-evidence/1\nid: {}\nkind: {}\n{result}subject: {}\n\
                     subject_revision: {}\nupstream_revisions:\n  {UPSTREAM}: {}\n",
                    record.id,
                    record.kind,
                    record.subject,
                    record.subject_revision,
                    record.upstream
                ),
            )
            .expect("record written");
        }
        let run = canon(&[
            "evaluate",
            "--ir",
            &path(&ir),
            "--case",
            &path(&case),
            "--evidence",
            &path(&evidence),
        ]);
        assert_eq!(
            (
                without_explanation(text(&run.stdout)).as_str(),
                text(&run.stderr),
                run.status.code()
            ),
            (decision.as_str(), "", Some(0)),
            "step `{id}`: canon evaluate"
        );
    }
}
