//! Conformance scenarios and their runner. Pure: the caller reads the scenario files and fixtures
//! and passes their text in; `canon conform run` is that caller.
//!
//! # The `canon-conformance/1` scenario format
//!
//! A scenario file is a YAML document holding exactly one named scenario:
//!
//! ```yaml
//! format: canon-conformance/1
//! id: CANON-CLAIM-001                       # the scenario id, unique within the registry
//! covers: [CANON-CLAIM-001, CANON-CLAIM-002] # the CANON-* requirements it covers; may be empty
//! fixture: fixtures/investigation/protocol.yaml
//! steps:
//!   - id: compile                           # the step id, unique within the scenario
//!     compile: {}
//!     expect:
//!       canon-ir: |
//!         { ... the exact canon-ir/1 bytes `canon compile` prints ... }
//!   - id: no-evidence
//!     evaluate:
//!       case: { ... a canon-case/1 document ... }
//!       evidence: [ ... canon-evidence/1 records ... ]
//!       authority: [ ... canon-authority/1 decisions ... ]  # optional
//!       decisions: [ ... canon-decisions/1 decisions ... ]  # optional
//!       at: 2026-10-04T00:00:00Z                           # optional evaluation instant
//!     expect:
//!       decision: |
//!         { ... the exact canon-decision/1 bytes ... }
//!       # or, instead of `decision`:
//!       # refusal: <the identifier the refusal names>
//! ```
//!
//! - `fixture` is the `protocol/1` document the scenario compiles, as a path relative to the
//!   working directory `canon conform run` runs in (the repository root, by convention). An
//!   absolute path (a leading `/` or `\`, or a drive prefix such as `C:`) or a path with a `..`
//!   component, split on `/` or `\`, is refused: the scenario does not parse. A path that passes
//!   that check but resolves outside the working directory through a symbolic link is refused
//!   too, unread: the scenario is reported `unreadable` with `fixture `<path>` resolves outside
//!   the working directory`.
//! - `covers` entries are `CANON-` followed by an identifier, each listed once.
//! - `steps` is an ordered, non-empty list. Each step is exactly one of `compile` or `evaluate`,
//!   with exactly one expectation of the matching kind: a compile step expects `canon-ir`, an
//!   evaluate step expects `decision` or `refusal`.
//! - An expectation is compared byte for byte with the step's output. A YAML literal block (`|`)
//!   keeps one trailing newline, which is the one `canon compile` prints.
//! - Unknown keys are refused: a file with one does not parse as a scenario.
//!
//! # The registry
//!
//! The registry is the directory `conformance/scenarios/`: one file per story, named
//! `<story-name>.yaml`. There is no index file. `canon conform run` reads every `*.yaml` file of
//! the directory (`--scenarios <dir>`, default `conformance/scenarios`) in sorted file-name order
//! and runs each scenario's steps in order, stopping a scenario at its first failing step.
//!
//! # The report
//!
//! `canon conform run` prints one line per file, in the order read, then one summary line, all on
//! standard output:
//!
//! ```text
//! passed: scenario `<id>`
//! failed: scenario `<id>` step `<step>`: <what differed>
//! unreadable: <path>: <why>
//! conform: <n> passed, <n> failed, <n> unreadable
//! ```
//!
//! For a compile step whose output differs, `<what differed>` is `canon-ir/1 differs from the
//! expectation at line <n>`, `<n>` being the first differing line, counted from 1. A fixture that
//! does not parse is reported by kind and position only (`fixture does not parse: not well-formed
//! YAML at line <l> column <c>`, or `not a protocol/1 document at …`), never by its content. So is
//! a registry file that does not deserialize as a scenario (`unreadable: <path>: not well-formed
//! YAML at …`, or `not a canon-conformance/1 scenario at …`). `<path>` is the scenario directory
//! as given, joined with the file name.
//!
//! A registry with no `*.yaml` file runs no scenario and is refused: the line `no scenario ran:
//! the registry holds no scenario file` comes before the summary.
//!
//! Exit status: 0 when at least one scenario ran and every scenario passed; 1 when any scenario
//! failed, any file was unreadable, or no scenario ran; 2 when the registry directory itself
//! cannot be read, with `error[unreadable]: <dir>: <why>` on standard error and nothing on
//! standard output. The same registry produces byte-identical output on every run.
//!
//! An evaluate step compiles the fixture, reads `case` as a `canon-case/1` document and each
//! `evidence` entry as a `canon-evidence/1` document, and calls [`eval::evaluate_with`], passing
//! `authority` and `decisions` (each as YAML text) and `at` through unread: the evaluator reads
//! them as `canon-authority/1` decisions, `canon-decisions/1` explicit decisions and the evaluation
//! instant, and refuses each when it is not one (`malformed-input`, `invalid-instant`, …). The
//! `authority` and `decisions` lists are each written back as YAML that reads as the same list (in
//! flow style where block style cannot write an entry); a list that cannot be fails the step as
//! `<list> entry <n> cannot be serialized: <why>` (`authority entry 2 …`, `decisions entry 1 …`),
//! and the registry run goes on.
//!
//! A step expecting `decision` compares section by section. The expectation's top-level keys are
//! the sections it lists; only those are compared, each on its own and byte for byte: written one
//! section per top-level member, as canonical JSON writes them, in any order, the step passes when
//! each listed section's lines equal the decision's for that section (the comma that separates
//! members aside). An expectation not laid out that way passes only when it equals, byte for byte,
//! the decision's canonical JSON holding only the listed sections ([`eval::render_sections`]). A
//! section the decision carries and the expectation does not list is not compared, so a later
//! story that adds a section leaves earlier expectations passing; a listed section the decision
//! does not carry is a difference. An expectation that is not a JSON
//! object is compared with the whole decision ([`eval::render`]); one that lists no section fails
//! as `the expected canon-decision/1 lists no section`. Otherwise `<what differed>` is
//! `canon-decision/1 section `<section>` differs from the expectation at line <n>`: `<section>` is
//! the first section the expectation writes whose lines differ from the decision's, and `<n>` the
//! expectation's line, counted from 1, where it first differs (the line that opens it, when the
//! decision does not carry it). When no single section differs, as when the expectation is not
//! laid out as canonical JSON, it is `canon-decision/1 differs from the expectation at line <n>`,
//! `<n>` the first differing line. A refused evaluation is `evaluation refused: <code>: <why>`.
//!
//! A step expecting `refusal` passes when the evaluation is refused with exactly that code (a case
//! or evidence entry that is not a document of its format is refused as `malformed-input`);
//! otherwise `<what differed>` is `expected refusal `<code>`, the evaluation gave a decision` or
//! `expected refusal `<code>`, the evaluation refused with `<code>`: <why>`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

use crate::{eval, ir, model};

/// The scenario format this module reads.
pub const FORMAT: &str = "canon-conformance/1";

/// The prefix every requirement a scenario covers carries.
pub const REQUIREMENT_PREFIX: &str = "CANON-";

/// One named scenario, as read from one scenario file.
#[derive(Debug, Clone, PartialEq)]
pub struct Scenario {
    pub id: String,
    pub covers: Vec<String>,
    pub fixture: String,
    pub steps: Vec<Step>,
}

/// One step of a scenario, with its inputs and its expectation.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub id: String,
    pub kind: StepKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StepKind {
    /// Compile the fixture and expect exactly these `canon-ir/1` bytes.
    Compile { expected_ir: String },
    /// Evaluate a case against the compiled fixture.
    Evaluate {
        inputs: EvaluateInputs,
        expected: EvaluateExpectation,
    },
}

/// The inputs of an evaluate step, as YAML values; [`run`] reads `case` and `evidence` as
/// `canon-case/1` and `canon-evidence/1` documents.
#[derive(Debug, Clone, PartialEq)]
pub struct EvaluateInputs {
    pub case: serde_yaml_ng::Value,
    pub evidence: Vec<serde_yaml_ng::Value>,
    pub authority: Option<Vec<serde_yaml_ng::Value>>,
    pub decisions: Option<Vec<serde_yaml_ng::Value>>,
    pub at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluateExpectation {
    /// The exact `canon-decision/1` bytes.
    Decision(String),
    /// The identifier the refusal names.
    Refusal(String),
}

/// The text does not parse as a `canon-conformance/1` scenario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioError {
    message: String,
}

impl fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ScenarioError {}

fn refuse<T>(message: String) -> Result<T, ScenarioError> {
    Err(ScenarioError { message })
}

/// A key written with no value (`key:`, `key: ~`) is refused rather than read as absent.
fn required<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)?.ok_or_else(|| {
        D::Error::custom("an explicit null is not allowed here; leave the key out instead")
    })
}

fn optional<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    required(deserializer).map(Some)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawScenario {
    #[serde(deserialize_with = "required")]
    format: String,
    #[serde(deserialize_with = "required")]
    id: String,
    #[serde(deserialize_with = "required")]
    covers: Vec<String>,
    #[serde(deserialize_with = "required")]
    fixture: String,
    #[serde(deserialize_with = "required")]
    steps: Vec<RawStep>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawStep {
    #[serde(deserialize_with = "required")]
    id: String,
    #[serde(default, deserialize_with = "optional")]
    compile: Option<RawCompile>,
    #[serde(default, deserialize_with = "optional")]
    evaluate: Option<RawEvaluate>,
    #[serde(deserialize_with = "required")]
    expect: RawExpect,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCompile {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEvaluate {
    #[serde(deserialize_with = "required")]
    case: serde_yaml_ng::Value,
    #[serde(deserialize_with = "required")]
    evidence: Vec<serde_yaml_ng::Value>,
    #[serde(default, deserialize_with = "optional")]
    authority: Option<Vec<serde_yaml_ng::Value>>,
    #[serde(default, deserialize_with = "optional")]
    decisions: Option<Vec<serde_yaml_ng::Value>>,
    #[serde(default, deserialize_with = "optional")]
    at: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawExpect {
    #[serde(rename = "canon-ir", default, deserialize_with = "optional")]
    canon_ir: Option<String>,
    #[serde(default, deserialize_with = "optional")]
    decision: Option<String>,
    #[serde(default, deserialize_with = "optional")]
    refusal: Option<String>,
}

/// Parses `canon-conformance/1` scenario text.
pub fn parse(source: &str) -> Result<Scenario, ScenarioError> {
    let raw: RawScenario = match serde_yaml_ng::from_str(source) {
        Ok(raw) => raw,
        Err(error) => {
            return refuse(parse_failure(
                source,
                "not a canon-conformance/1 scenario",
                error.location(),
            ));
        }
    };
    if raw.format != FORMAT {
        return refuse(format!(
            "format is `{}`, expected `{FORMAT}`",
            model::one_line(&raw.format)
        ));
    }
    if !model::is_identifier(&raw.id) {
        return refuse(format!(
            "scenario id `{}` is not an identifier",
            model::one_line(&raw.id)
        ));
    }
    for (index, requirement) in raw.covers.iter().enumerate() {
        let named = requirement
            .strip_prefix(REQUIREMENT_PREFIX)
            .is_some_and(model::is_identifier);
        if !named {
            return refuse(format!(
                "covers `{}`, which is not a {REQUIREMENT_PREFIX}* requirement",
                model::one_line(requirement)
            ));
        }
        if raw.covers[..index].contains(requirement) {
            return refuse(format!(
                "covers `{}` more than once",
                model::one_line(requirement)
            ));
        }
    }
    if raw.fixture.is_empty() {
        return refuse("fixture is empty".to_owned());
    }
    if !is_confined(&raw.fixture) {
        return refuse(format!(
            "fixture `{}` is not a relative path without `..` components",
            model::one_line(&raw.fixture)
        ));
    }
    if raw.steps.is_empty() {
        return refuse("steps is empty".to_owned());
    }
    let mut steps: Vec<Step> = Vec::with_capacity(raw.steps.len());
    for step in raw.steps {
        let id = step.id;
        if !model::is_identifier(&id) {
            return refuse(format!(
                "step id `{}` is not an identifier",
                model::one_line(&id)
            ));
        }
        if steps.iter().any(|earlier| earlier.id == id) {
            return refuse(format!(
                "step `{}` is declared more than once",
                model::one_line(&id)
            ));
        }
        let kind = step_kind(&id, step.compile, step.evaluate, step.expect)?;
        steps.push(Step { id, kind });
    }
    Ok(Scenario {
        id: raw.id,
        covers: raw.covers,
        fixture: raw.fixture,
        steps,
    })
}

/// Whether a fixture path stays below the working directory as written: it is not absolute on any
/// platform (no leading `/` or `\`, no drive prefix such as `C:`) and no component, split on `/`
/// or `\`, is `..`. Decided on the text alone, the same way on every platform, without touching
/// the filesystem.
///
/// This is the first of two checks. The text cannot see a symbolic link, so the caller's fixture
/// reader makes the second: it resolves the path against the filesystem and returns
/// [`FixtureError::Outside`], unread, when the result is not below the resolved working directory
/// (`canon conform run` does).
pub fn is_confined(path: &str) -> bool {
    let rooted = path.starts_with(['/', '\\']);
    let mut chars = path.chars();
    let drive = matches!(
        (chars.next(), chars.next()),
        (Some(letter), Some(':')) if letter.is_ascii_alphabetic()
    );
    !rooted && !drive && !path.split(['/', '\\']).any(|component| component == "..")
}

fn step_kind(
    id: &str,
    compile: Option<RawCompile>,
    evaluate: Option<RawEvaluate>,
    expect: RawExpect,
) -> Result<StepKind, ScenarioError> {
    let id = model::one_line(id);
    match (compile, evaluate, expect) {
        (
            Some(RawCompile {}),
            None,
            RawExpect {
                canon_ir: Some(expected_ir),
                decision: None,
                refusal: None,
            },
        ) => Ok(StepKind::Compile { expected_ir }),
        (Some(_), None, _) => refuse(format!(
            "compile step `{id}` must expect `canon-ir` and nothing else"
        )),
        (None, Some(evaluate), expect) => {
            let expected = match expect {
                RawExpect {
                    canon_ir: None,
                    decision: Some(decision),
                    refusal: None,
                } => EvaluateExpectation::Decision(decision),
                RawExpect {
                    canon_ir: None,
                    decision: None,
                    refusal: Some(refusal),
                } if model::is_identifier(&refusal) => EvaluateExpectation::Refusal(refusal),
                _ => {
                    return refuse(format!(
                        "evaluate step `{id}` must expect exactly one of `decision` or `refusal` (an identifier)"
                    ));
                }
            };
            Ok(StepKind::Evaluate {
                inputs: EvaluateInputs {
                    case: evaluate.case,
                    evidence: evaluate.evidence,
                    authority: evaluate.authority,
                    decisions: evaluate.decisions,
                    at: evaluate.at,
                },
                expected,
            })
        }
        (Some(_), Some(_), _) | (None, None, _) => refuse(format!(
            "step `{id}` must be exactly one of `compile` or `evaluate`"
        )),
    }
}

/// The result of running one scenario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Passed,
    /// The first step that did not meet its expectation, and why.
    Failed {
        step: String,
        reason: String,
    },
}

/// Runs a scenario's steps in order, stopping at the first that fails. `fixture` is the text of
/// the scenario's fixture, or why it could not be read.
pub fn run(scenario: &Scenario, fixture: Result<&str, &str>) -> Verdict {
    for step in &scenario.steps {
        let failure = match &step.kind {
            StepKind::Compile { expected_ir } => compile_step(scenario, fixture, expected_ir),
            StepKind::Evaluate { inputs, expected } => {
                evaluate_step(scenario, fixture, inputs, expected)
            }
        };
        if let Some(reason) = failure {
            return Verdict::Failed {
                step: step.id.clone(),
                reason,
            };
        }
    }
    Verdict::Passed
}

/// The fixture's compiled protocol, or why the step that needs it fails.
fn compiled_fixture(scenario: &Scenario, fixture: Result<&str, &str>) -> Result<ir::Ir, String> {
    let source = fixture.map_err(|why| {
        format!(
            "fixture `{}` is unreadable: {}",
            model::one_line(&scenario.fixture),
            model::one_line(why)
        )
    })?;
    let protocol = model::parse(source)
        .map_err(|_| format!("fixture does not parse: {}", fixture_parse_failure(source)))?;
    ir::compile(&protocol).map_err(|problems| {
        let problems: Vec<String> = problems
            .iter()
            .map(|problem| format!("{}: {problem}", problem.code()))
            .collect();
        format!("fixture does not compile: {}", problems.join("; "))
    })
}

fn compile_step(
    scenario: &Scenario,
    fixture: Result<&str, &str>,
    expected_ir: &str,
) -> Option<String> {
    let compiled = match compiled_fixture(scenario, fixture) {
        Ok(compiled) => compiled.canonical_json(),
        Err(why) => return Some(why),
    };
    first_difference(&compiled, expected_ir)
        .map(|line| format!("canon-ir/1 differs from the expectation at line {line}"))
}

fn evaluate_step(
    scenario: &Scenario,
    fixture: Result<&str, &str>,
    inputs: &EvaluateInputs,
    expected: &EvaluateExpectation,
) -> Option<String> {
    let compiled = match compiled_fixture(scenario, fixture) {
        Ok(compiled) => compiled,
        Err(why) => return Some(why),
    };
    let written = |list: &Option<Vec<serde_yaml_ng::Value>>, name: &str| {
        list.as_deref()
            .map(|list| list_text(name, list))
            .transpose()
    };
    let authority = match written(&inputs.authority, "authority") {
        Ok(authority) => authority,
        Err(why) => return Some(why),
    };
    let decisions = match written(&inputs.decisions, "decisions") {
        Ok(decisions) => decisions,
        Err(why) => return Some(why),
    };
    let supplied = eval::Supplied {
        authority: authority.as_deref(),
        at: inputs.at.as_deref(),
        decisions: decisions.as_deref(),
    };
    let evaluation = eval::case_from_value(&inputs.case).and_then(|case| {
        let evidence = inputs
            .evidence
            .iter()
            .map(eval::evidence_from_value)
            .collect::<Result<Vec<_>, _>>()?;
        eval::evaluate_with(&compiled, &case, &evidence, supplied)
    });
    match (expected, evaluation) {
        (EvaluateExpectation::Decision(expected), Ok(decision)) => {
            decision_difference(&decision, expected)
        }
        (EvaluateExpectation::Decision(_), Err(refusal)) => {
            Some(format!("evaluation refused: {}: {refusal}", refusal.code()))
        }
        (EvaluateExpectation::Refusal(code), Ok(_)) => Some(format!(
            "expected refusal `{}`, the evaluation gave a decision",
            model::one_line(code)
        )),
        (EvaluateExpectation::Refusal(code), Err(refusal)) if refusal.code() == code => None,
        (EvaluateExpectation::Refusal(code), Err(refusal)) => Some(format!(
            "expected refusal `{}`, the evaluation refused with `{}`: {refusal}",
            model::one_line(code),
            refusal.code()
        )),
    }
}

/// Why a fixture that [`model::parse`] refused does not parse, by kind and position only. A
/// scenario names its fixture by path, so the file may be anything; the deserializer's own message
/// quotes the offending value, and that value must not reach the report.
fn fixture_parse_failure(source: &str) -> String {
    parse_failure(
        source,
        "not a protocol/1 document",
        serde_yaml_ng::from_str::<model::Protocol>(source)
            .err()
            .and_then(|error| error.location()),
    )
}

/// Why a document did not deserialize, by kind and position only, for every file the runner reads
/// (a registry file and a fixture alike): `not well-formed YAML` when the text is not YAML at
/// all, otherwise `shape`, each at the position the deserializer that refused it gives. (Both
/// readers place a duplicate key at line 1 column 1.) The deserializer's own message is never
/// used: it quotes the offending value or key, and names internal types.
fn parse_failure(
    source: &str,
    shape: &'static str,
    location: Option<serde_yaml_ng::Location>,
) -> String {
    let (kind, location) = match serde_yaml_ng::from_str::<serde_yaml_ng::Value>(source) {
        Err(error) => ("not well-formed YAML", error.location()),
        Ok(_) => (shape, location),
    };
    match location {
        Some(at) => format!("{kind} at line {} column {}", at.line(), at.column()),
        None => kind.to_owned(),
    }
}

/// A step's `authority` or `decisions` list (`name`) as the YAML text the evaluator reads: what
/// `serde_yaml_ng` writes, or, where it cannot write an entry (a mapping key that is itself a
/// mapping), the list in flow style. Either is used only when it reads back as the list the
/// scenario holds; otherwise the step fails as `<name> entry <n> cannot be serialized: <why>`, `<n>`
/// counted from 1.
fn list_text(name: &str, list: &[serde_yaml_ng::Value]) -> Result<String, String> {
    let reads_back = |text: &str| {
        serde_yaml_ng::from_str::<Vec<serde_yaml_ng::Value>>(text).is_ok_and(|back| back == list)
    };
    if let Some(text) = serde_yaml_ng::to_string(list)
        .ok()
        .filter(|text| reads_back(text))
    {
        return Ok(text);
    }
    let flow = format!(
        "[{}]\n",
        list.iter().map(flow_yaml).collect::<Vec<_>>().join(", ")
    );
    if reads_back(&flow) {
        return Ok(flow);
    }
    let (entry, why) = list
        .iter()
        .enumerate()
        .find_map(|(index, value)| {
            let why = match serde_yaml_ng::to_string(value) {
                Err(error) => error.to_string(),
                Ok(_)
                    if serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&flow_yaml(value))
                        .is_ok_and(|back| back == *value) =>
                {
                    return None;
                }
                Ok(_) => "it does not read back as written".to_owned(),
            };
            Some((index + 1, why))
        })
        .unwrap_or((list.len(), "it does not read back as written".to_owned()));
    Err(format!(
        "{name} entry {entry} cannot be serialized: {}",
        model::one_line(&why)
    ))
}

/// One YAML value in flow style, on one line: strings double-quoted (JSON string syntax is a YAML
/// double-quoted scalar), collections in `[…]` and `{…}`, a mapping key written as any value is.
fn flow_yaml(value: &serde_yaml_ng::Value) -> String {
    use serde_yaml_ng::Value;
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => serde_yaml_ng::to_string(number)
            .map(|text| text.trim_end().to_owned())
            .unwrap_or_else(|_| number.to_string()),
        Value::String(text) => serde_json::to_string(text).expect("a string serializes as JSON"),
        Value::Sequence(items) => format!(
            "[{}]",
            items.iter().map(flow_yaml).collect::<Vec<_>>().join(", ")
        ),
        Value::Mapping(entries) => format!(
            "{{{}}}",
            entries
                .iter()
                .map(|(key, item)| format!("{}: {}", flow_yaml(key), flow_yaml(item)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Value::Tagged(tagged) => format!("{} {}", tagged.tag, flow_yaml(&tagged.value)),
    }
}

/// How `decision` differs from an expected `canon-decision/1`, comparing only the sections the
/// expectation lists (see the module docs), or `None` when it does not.
fn decision_difference(decision: &model::Decision, expected: &str) -> Option<String> {
    let listed: Option<BTreeSet<String>> =
        serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(expected)
            .ok()
            .map(|sections| sections.keys().cloned().collect());
    let Some(listed) = listed else {
        return first_difference(&eval::render(decision), expected)
            .map(|line| format!("canon-decision/1 differs from the expectation at line {line}"));
    };
    if listed.is_empty() {
        return Some("the expected canon-decision/1 lists no section".to_owned());
    }
    let actual = eval::render_sections(decision, &listed);
    if laid_out_by_section(expected, &listed) {
        // Each listed section on its own, byte for byte, in whatever order the expectation
        // writes them.
        return differing_section(&actual, expected).map(|(section, line)| {
            format!(
                "canon-decision/1 section `{}` differs from the expectation at line {line}",
                model::one_line(&section)
            )
        });
    }
    let line = first_difference(&actual, expected)?;
    Some(match differing_section(&actual, expected) {
        Some((section, line)) => format!(
            "canon-decision/1 section `{}` differs from the expectation at line {line}",
            model::one_line(&section)
        ),
        None => format!("canon-decision/1 differs from the expectation at line {line}"),
    })
}

/// Whether the expectation is laid out one listed section per top-level member, as canonical JSON
/// writes them: `{` on the first line, `}` and a newline at the end, and every line in between in
/// exactly one member, each listed key opening exactly one member. The members may come in any
/// order.
fn laid_out_by_section(expected: &str, listed: &BTreeSet<String>) -> bool {
    let Some(body) = expected
        .strip_prefix("{\n")
        .and_then(|rest| rest.strip_suffix("}\n"))
    else {
        return false;
    };
    let members = members(expected);
    let keys: BTreeSet<&String> = members.iter().map(|(key, _, _)| key).collect();
    let lines: usize = members.iter().map(|(_, _, lines)| lines.len()).sum();
    keys.len() == members.len()
        && keys.len() == listed.len()
        && keys.iter().all(|key| listed.contains(*key))
        && lines == body.split_inclusive('\n').count()
}

/// The first section the expectation writes, in its order, whose lines differ from the same
/// section of `actual`, with the expectation's line, counted from 1, where it first differs (the
/// section's first line when `actual` does not carry it). `None` when every section the
/// expectation writes as a top-level member matches, as when the expectation is not laid out as
/// canonical JSON.
fn differing_section(actual: &str, expected: &str) -> Option<(String, usize)> {
    let actual: BTreeMap<String, Vec<&str>> = members(actual)
        .into_iter()
        .map(|(key, _, lines)| (key, lines))
        .collect();
    for (key, start, lines) in members(expected) {
        let Some(found) = actual.get(&key) else {
            return Some((key, start));
        };
        if *found != lines {
            let offset = found.iter().zip(&lines).take_while(|(a, e)| a == e).count();
            return Some((key, start + offset));
        }
    }
    None
}

/// The top-level members of JSON laid out canonically: each key, the number of the line that opens
/// it (counted from 1), and its lines, the separating comma after the last one removed. A line ends
/// at `\n` only: every other byte, a `\r` before the newline included, is part of the line, so a
/// line that differs in any byte differs.
fn members(text: &str) -> Vec<(String, usize, Vec<&str>)> {
    let mut members: Vec<(String, usize, Vec<&str>)> = Vec::new();
    let mut open = false;
    let written = text
        .split_inclusive('\n')
        .map(|line| line.strip_suffix('\n').unwrap_or(line));
    for (index, line) in written.enumerate() {
        if let Some(key) = top_level_key(line) {
            members.push((key, index + 1, vec![line]));
            open = true;
        } else if line == "}" {
            open = false;
        } else if let Some((_, _, lines)) = members.last_mut().filter(|_| open) {
            lines.push(line);
        }
    }
    for (_, _, lines) in &mut members {
        if let Some(last) = lines.last_mut() {
            *last = last.strip_suffix(',').unwrap_or(last);
        }
    }
    members
}

/// The key a line opens when it is a top-level member of canonical JSON: indented by exactly two
/// spaces, then a JSON string, then `:`.
fn top_level_key(line: &str) -> Option<String> {
    let rest = line.strip_prefix("  \"")?;
    let mut escaped = false;
    let end = rest.char_indices().find_map(|(at, c)| {
        if escaped {
            escaped = false;
            None
        } else if c == '\\' {
            escaped = true;
            None
        } else {
            (c == '"').then_some(at)
        }
    })?;
    if !rest[end + 1..].starts_with(':') {
        return None;
    }
    serde_json::from_str(&format!("\"{}\"", &rest[..end])).ok()
}

/// The first line, counted from 1, at which two texts differ, or `None` when they are equal. A
/// line includes its newline, so a missing or extra trailing newline is a difference.
pub fn first_difference(actual: &str, expected: &str) -> Option<usize> {
    if actual == expected {
        return None;
    }
    let mut actual = actual.split_inclusive('\n');
    let mut expected = expected.split_inclusive('\n');
    let mut line = 1;
    loop {
        match (actual.next(), expected.next()) {
            (Some(a), Some(e)) if a == e => line += 1,
            _ => return Some(line),
        }
    }
}

/// One file of a registry: its path as it is reported, and its text or why it could not be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioFile {
    pub path: String,
    pub text: Result<String, String>,
}

/// Why the caller's fixture reader returned no text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureError {
    /// The fixture could not be read; the step that needs it fails, saying why.
    Unreadable(String),
    /// The fixture path passed [`is_confined`] but resolves outside the working directory (through
    /// a symbolic link); it was not read, and the scenario is reported unreadable.
    Outside,
}

/// Runs every scenario of a registry, in the order given (the caller passes files in sorted
/// file-name order). `read_fixture` returns a fixture's text, or why it returned none; it is the
/// caller's, so this function reads nothing itself. A file whose scenario id an earlier file
/// already used, or whose fixture resolves outside the working directory, is reported unreadable.
pub fn run_registry<F>(files: &[ScenarioFile], mut read_fixture: F) -> Report
where
    F: FnMut(&str) -> Result<String, FixtureError>,
{
    let mut report = Report::default();
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    for file in files {
        let scenario = match &file.text {
            Err(why) => {
                report.unreadable(&file.path, &model::one_line(why));
                continue;
            }
            Ok(text) => match parse(text) {
                Ok(scenario) => scenario,
                Err(error) => {
                    report.unreadable(&file.path, &error.to_string());
                    continue;
                }
            },
        };
        if let Some(earlier) = seen.get(&scenario.id) {
            report.unreadable(
                &file.path,
                &format!(
                    "scenario id `{}` is already used by {}",
                    model::one_line(&scenario.id),
                    model::one_line(earlier)
                ),
            );
            continue;
        }
        seen.insert(scenario.id.clone(), &file.path);
        let verdict = match read_fixture(&scenario.fixture) {
            Ok(text) => run(&scenario, Ok(&text)),
            Err(FixtureError::Unreadable(why)) => run(&scenario, Err(&why)),
            Err(FixtureError::Outside) => {
                report.unreadable(
                    &file.path,
                    &format!(
                        "fixture `{}` resolves outside the working directory",
                        model::one_line(&scenario.fixture)
                    ),
                );
                continue;
            }
        };
        report.scenario(&scenario.id, &verdict);
    }
    report
}

/// The report `canon conform run` prints: one line per file, then a summary line.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    lines: String,
    passed: usize,
    failed: usize,
    unreadable: usize,
}

impl Report {
    /// Records a scenario's verdict. A failure's reason is already one line, as [`run`] builds it.
    pub fn scenario(&mut self, id: &str, verdict: &Verdict) {
        let id = model::one_line(id);
        match verdict {
            Verdict::Passed => {
                self.passed += 1;
                self.lines.push_str(&format!("passed: scenario `{id}`\n"));
            }
            Verdict::Failed { step, reason } => {
                self.failed += 1;
                self.lines.push_str(&format!(
                    "failed: scenario `{id}` step `{}`: {reason}\n",
                    model::one_line(step)
                ));
            }
        }
    }

    /// Records a file that could not be read, or does not parse, as a scenario. `why` is already
    /// one line: every reason this module builds quotes document text through [`model::one_line`].
    pub fn unreadable(&mut self, path: &str, why: &str) {
        self.unreadable += 1;
        self.lines
            .push_str(&format!("unreadable: {}: {why}\n", model::one_line(path)));
    }

    /// Whether at least one scenario ran, every file held a scenario and every scenario passed. An
    /// empty registry has not passed: nothing was checked.
    pub fn all_passed(&self) -> bool {
        self.passed > 0 && self.failed == 0 && self.unreadable == 0
    }

    /// Whether the registry held no file at all.
    fn is_empty(&self) -> bool {
        self.passed == 0 && self.failed == 0 && self.unreadable == 0
    }

    /// The report text, summary line last.
    pub fn render(&self) -> String {
        let empty = if self.is_empty() {
            "no scenario ran: the registry holds no scenario file\n"
        } else {
            ""
        };
        format!(
            "{}{empty}conform: {} passed, {} failed, {} unreadable\n",
            self.lines, self.passed, self.failed, self.unreadable
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROTOCOL: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n";

    fn compiled() -> String {
        ir::compile(&model::parse(PROTOCOL).expect("parses"))
            .expect("compiles")
            .canonical_json()
    }

    fn scenario_text(id: &str, expected_ir: &str) -> String {
        let indented: String = expected_ir
            .lines()
            .map(|line| format!("        {line}\n"))
            .collect();
        format!(
            "format: canon-conformance/1\nid: {id}\ncovers: [CANON-DETERMINISM-001]\nfixture: p.yaml\nsteps:\n  - id: compile\n    compile: {{}}\n    expect:\n      canon-ir: |\n{indented}"
        )
    }

    fn refusal(source: &str) -> String {
        parse(source).expect_err("refused").to_string()
    }

    const HEAD: &str = "format: canon-conformance/1\nid: S\ncovers: []\nfixture: p.yaml\n";

    #[test]
    fn a_compile_scenario_parses_and_passes_on_its_own_ir() {
        let scenario = parse(&scenario_text("S", &compiled())).expect("parses");
        assert_eq!(scenario.id, "S");
        assert_eq!(scenario.covers, ["CANON-DETERMINISM-001"]);
        assert_eq!(scenario.steps.len(), 1);
        assert_eq!(run(&scenario, Ok(PROTOCOL)), Verdict::Passed);
    }

    #[test]
    fn the_scenario_shape_is_enforced() {
        let step = "steps:\n  - id: c\n    compile: {}\n    expect: {canon-ir: x}\n";
        let cases = [
            (
                format!("{HEAD}{step}extra: 1\n"),
                "not a canon-conformance/1 scenario at line 9 column 1",
            ),
            (
                format!("format: canon-conformance/2\nid: S\ncovers: []\nfixture: p.yaml\n{step}"),
                "format is `canon-conformance/2`, expected `canon-conformance/1`",
            ),
            (
                format!(
                    "format: canon-conformance/1\nid: S\ncovers: [CLAIM-001]\nfixture: p.yaml\n{step}"
                ),
                "covers `CLAIM-001`, which is not a CANON-* requirement",
            ),
            (
                format!(
                    "format: canon-conformance/1\nid: S\ncovers: [CANON-A, CANON-A]\nfixture: p.yaml\n{step}"
                ),
                "covers `CANON-A` more than once",
            ),
            (
                format!(
                    "format: canon-conformance/1\nid: 'a b'\ncovers: []\nfixture: p.yaml\n{step}"
                ),
                "scenario id `a b` is not an identifier",
            ),
            (
                "format: canon-conformance/1\nid: S\ncovers: []\nfixture:\nsteps: []\n".to_owned(),
                "not a canon-conformance/1 scenario at line 1 column 1",
            ),
            (
                "format: canon-conformance/1\nid: S\ncovers: []\nfixture: ''\nsteps: []\n"
                    .to_owned(),
                "fixture is empty",
            ),
            (format!("{HEAD}steps: []\n"), "steps is empty"),
            (
                format!("{HEAD}covers: []\n{step}"),
                "not well-formed YAML at line 1 column 1",
            ),
            (
                format!(
                    "{HEAD}steps:\n  - id: c\n    compile: {{}}\n    expect: {{canon-ir: x}}\n  - id: c\n    compile: {{}}\n    expect: {{canon-ir: x}}\n"
                ),
                "step `c` is declared more than once",
            ),
            (
                format!("{HEAD}steps:\n  - id: c\n    expect: {{canon-ir: x}}\n"),
                "step `c` must be exactly one of `compile` or `evaluate`",
            ),
            (
                format!(
                    "{HEAD}steps:\n  - id: c\n    compile: {{}}\n    evaluate: {{case: {{}}, evidence: []}}\n    expect: {{canon-ir: x}}\n"
                ),
                "step `c` must be exactly one of `compile` or `evaluate`",
            ),
            (
                format!(
                    "{HEAD}steps:\n  - id: c\n    compile: {{}}\n    expect: {{decision: x}}\n"
                ),
                "compile step `c` must expect `canon-ir` and nothing else",
            ),
            (
                format!(
                    "{HEAD}steps:\n  - id: c\n    compile: {{}}\n    expect: {{canon-ir: x, refusal: r}}\n"
                ),
                "compile step `c` must expect `canon-ir` and nothing else",
            ),
            (
                format!(
                    "{HEAD}steps:\n  - id: e\n    evaluate: {{case: {{}}, evidence: []}}\n    expect: {{decision: x, refusal: r}}\n"
                ),
                "evaluate step `e` must expect exactly one of `decision` or `refusal`",
            ),
            (
                format!(
                    "{HEAD}steps:\n  - id: e\n    evaluate: {{case: {{}}, evidence: []}}\n    expect: {{canon-ir: x}}\n"
                ),
                "evaluate step `e` must expect exactly one of `decision` or `refusal`",
            ),
            (
                format!(
                    "{HEAD}steps:\n  - id: e\n    evaluate: {{case: {{}}, evidence: []}}\n    expect: {{refusal: 'not one'}}\n"
                ),
                "evaluate step `e` must expect exactly one of `decision` or `refusal`",
            ),
            (
                format!(
                    "{HEAD}steps:\n  - id: e\n    evaluate: {{case: {{}}}}\n    expect: {{refusal: r}}\n"
                ),
                "not a canon-conformance/1 scenario at line 7 column 15",
            ),
        ];
        for (source, expected) in cases {
            let message = refusal(&source);
            assert!(message.contains(expected), "{source}\n=> {message}");
        }
    }

    const CASE: &str = "{format: canon-case/1, id: c, protocol: p, artifacts: {}}";

    const DECISION: &str = "        {\n          \"case\": \"c\",\n          \"claims\": {},\n          \"format\": \"canon-decision/1\",\n          \"protocol\": \"p\",\n          \"protocol_revision\": 1\n        }\n";

    fn evaluate_verdict(inputs: &str, expect: &str) -> Verdict {
        let source =
            format!("{HEAD}steps:\n  - id: e\n    evaluate:\n{inputs}    expect:\n{expect}");
        run(&parse(&source).expect("parses"), Ok(PROTOCOL))
    }

    fn failed(reason: &str) -> Verdict {
        Verdict::Failed {
            step: "e".to_owned(),
            reason: reason.to_owned(),
        }
    }

    #[test]
    fn an_evaluate_step_compares_the_decision_byte_for_byte() {
        let inputs = format!("      case: {CASE}\n      evidence: []\n");
        assert_eq!(
            evaluate_verdict(&inputs, &format!("      decision: |\n{DECISION}")),
            Verdict::Passed
        );
        assert_eq!(
            evaluate_verdict(
                &inputs,
                &format!(
                    "      decision: |\n{}",
                    DECISION.replace("\"c\"", "\"other\"")
                )
            ),
            failed("canon-decision/1 section `case` differs from the expectation at line 2")
        );
        assert_eq!(
            evaluate_verdict(
                &inputs.replace("protocol: p", "protocol: q"),
                &format!("      decision: |\n{DECISION}")
            ),
            failed(
                "evaluation refused: protocol-mismatch: case `c` is governed by protocol `q`, not by `p`"
            )
        );
    }

    #[test]
    fn an_evaluate_step_expecting_a_refusal_compares_its_code() {
        let mismatched = format!(
            "      case: {}\n      evidence: []\n",
            CASE.replace("p,", "q,")
        );
        assert_eq!(
            evaluate_verdict(&mismatched, "      refusal: protocol-mismatch\n"),
            Verdict::Passed
        );
        assert_eq!(
            evaluate_verdict(&mismatched, "      refusal: undeclared-artifact\n"),
            failed(
                "expected refusal `undeclared-artifact`, the evaluation refused with `protocol-mismatch`: case `c` is governed by protocol `q`, not by `p`"
            )
        );
        assert_eq!(
            evaluate_verdict(
                &format!("      case: {CASE}\n      evidence: []\n"),
                "      refusal: protocol-mismatch\n"
            ),
            failed("expected refusal `protocol-mismatch`, the evaluation gave a decision")
        );
        assert_eq!(
            evaluate_verdict(
                "      case: {id: c}\n      evidence: []\n",
                "      refusal: malformed-input\n"
            ),
            Verdict::Passed,
            "a case that is not a canon-case/1 document is refused as malformed-input"
        );
    }

    #[test]
    fn an_evaluate_step_compares_only_the_sections_its_expectation_lists() {
        let inputs = format!("      case: {CASE}\n      evidence: []\n");
        let decision = |text: &str| {
            let indented: String = text.lines().map(|l| format!("        {l}\n")).collect();
            format!("      decision: |\n{indented}")
        };
        // Sections the decision carries and the expectation leaves out are not compared.
        assert_eq!(
            evaluate_verdict(&inputs, &decision("{\n  \"claims\": {}\n}")),
            Verdict::Passed
        );
        // A listed section that differs is named, at the expectation's line.
        assert_eq!(
            evaluate_verdict(
                &inputs,
                &decision("{\n  \"claims\": {},\n  \"protocol\": \"q\"\n}")
            ),
            failed("canon-decision/1 section `protocol` differs from the expectation at line 3")
        );
        // A listed section the decision does not carry is a difference, named.
        assert_eq!(
            evaluate_verdict(
                &inputs,
                &decision("{\n  \"claims\": {},\n  \"obligations\": {}\n}")
            ),
            failed("canon-decision/1 section `obligations` differs from the expectation at line 3")
        );
        // An expectation listing no section passes nothing.
        assert_eq!(
            evaluate_verdict(&inputs, &decision("{}")),
            failed("the expected canon-decision/1 lists no section")
        );
        // An expectation that is not a JSON object is compared with the whole decision.
        assert_eq!(
            evaluate_verdict(&inputs, &decision("not json")),
            failed("canon-decision/1 differs from the expectation at line 1")
        );
    }

    /// An authority list is written back as YAML that reads as the same list: block style where
    /// it can be, flow style where block style cannot write an entry (a mapping key that is a
    /// mapping). The refusal, for a list neither style writes back, guards against a value no
    /// known YAML input produces.
    #[test]
    fn an_authority_list_is_written_back_as_yaml_that_reads_the_same() {
        use serde_yaml_ng::Value;
        let list = |text: &str| -> Vec<Value> { serde_yaml_ng::from_str(text).expect("yaml") };
        for written in [
            "[]",
            "[{capability: c, decision: granted}]",
            "[{{x: 1}: c}, \"quoted \\\" and \\n\", !custom {a: [1, .nan]}]",
        ] {
            let authority = list(written);
            let text = list_text("authority", &authority).expect(written);
            assert_eq!(list(&text), authority, "{written} as {text}");
        }
    }

    /// Each listed section is compared on its own, so the order the expectation writes them in
    /// does not matter; a section that differs is still named, at its own line.
    #[test]
    fn listed_sections_are_compared_each_on_its_own_in_any_order() {
        let inputs = format!("      case: {CASE}\n      evidence: []\n");
        let decision = |text: &str| {
            let indented: String = text.lines().map(|l| format!("        {l}\n")).collect();
            format!("      decision: |\n{indented}")
        };
        assert_eq!(
            evaluate_verdict(
                &inputs,
                &decision(
                    "{\n  \"protocol_revision\": 1,\n  \"claims\": {},\n  \"case\": \"c\"\n}"
                )
            ),
            Verdict::Passed
        );
        assert_eq!(
            evaluate_verdict(
                &inputs,
                &decision(
                    "{\n  \"protocol_revision\": 1,\n  \"case\": \"d\",\n  \"claims\": {}\n}"
                )
            ),
            failed("canon-decision/1 section `case` differs from the expectation at line 3")
        );
        // A section written twice is not one section per member: compared as a whole, it fails.
        assert_eq!(
            evaluate_verdict(
                &inputs,
                &decision("{\n  \"case\": \"c\",\n  \"case\": \"c\"\n}")
            ),
            failed("canon-decision/1 differs from the expectation at line 2")
        );
    }

    #[test]
    fn a_top_level_key_is_read_from_a_two_space_member_line_only() {
        assert_eq!(top_level_key("  \"claims\": {"), Some("claims".to_owned()));
        assert_eq!(top_level_key("  \"a\\\"b\": 1,"), Some("a\"b".to_owned()));
        assert_eq!(top_level_key("    \"value\": \"true\""), None);
        assert_eq!(top_level_key("  \"unterminated"), None);
        assert_eq!(top_level_key("{"), None);
    }

    #[test]
    fn an_evaluate_step_passes_authority_and_an_instant_to_the_evaluator() {
        let base = format!("      case: {CASE}\n      evidence: []\n");
        let expect = format!("      decision: |\n{DECISION}");
        // Changed by story:action-admissibility, which reads `canon-authority/1`: an empty list is
        // evaluated, and an entry that is not a decision is refused as `malformed-input`, which
        // only the evaluator can do. The skeleton refused any authority as `unsupported-input`.
        assert_eq!(
            evaluate_verdict(&format!("{base}      authority: []\n"), &expect),
            Verdict::Passed
        );
        assert_eq!(
            evaluate_verdict(
                &format!("{base}      authority: [1]\n"),
                "      refusal: malformed-input\n"
            ),
            Verdict::Passed
        );
        assert_eq!(
            evaluate_verdict(&format!("{base}      at: 2026-10-04T00:00:00Z\n"), &expect),
            Verdict::Passed
        );
        assert_eq!(
            evaluate_verdict(&format!("{base}      at: yesterday\n"), &expect),
            failed(
                "evaluation refused: invalid-instant: the evaluation instant `yesterday` is not an instant in UTC written YYYY-MM-DDTHH:MM:SSZ"
            )
        );
        assert_eq!(
            evaluate_verdict(
                &format!("{base}      at: yesterday\n"),
                "      refusal: invalid-instant\n"
            ),
            Verdict::Passed
        );
    }

    /// A `decisions` list reaches the evaluator as YAML text, as `authority` does: an entry that is
    /// not a `canon-decisions/1` decision is refused by the evaluator, not dropped by the runner, so
    /// the step that expects a decision fails as a refused evaluation.
    #[test]
    fn an_evaluate_step_passes_decisions_to_the_evaluator() {
        let base = format!("      case: {CASE}\n      evidence: []\n");
        let expect = format!("      decision: |\n{DECISION}");
        assert_eq!(evaluate_verdict(&base, &expect), Verdict::Passed);
        let Verdict::Failed { reason, .. } =
            evaluate_verdict(&format!("{base}      decisions: [1]\n"), &expect)
        else {
            panic!("a decisions entry that is not a decision reaches the evaluator and is refused");
        };
        assert!(reason.starts_with("evaluation refused: "), "{reason}");
    }

    #[test]
    fn an_evaluate_step_reports_a_fixture_that_does_not_compile_first() {
        let source = format!(
            "{HEAD}steps:\n  - id: e\n    evaluate:\n      case: {CASE}\n      evidence: []\n      authority: []\n    expect: {{refusal: r}}\n"
        );
        let scenario = parse(&source).expect("parses");
        assert_eq!(
            run(&scenario, Err("gone")),
            failed("fixture `p.yaml` is unreadable: gone")
        );
    }

    #[test]
    fn a_compile_step_reports_why_the_fixture_did_not_compile_to_its_expectation() {
        let scenario = parse(&scenario_text("S", &compiled())).expect("parses");
        let failed = |fixture: Result<&str, &str>| match run(&scenario, fixture) {
            Verdict::Failed { step, reason } => {
                assert_eq!(step, "compile");
                reason
            }
            Verdict::Passed => panic!("passed"),
        };
        assert_eq!(
            failed(Err("No such file\n(os error 2)")),
            "fixture `p.yaml` is unreadable: No such file\\n(os error 2)"
        );
        assert!(failed(Ok("format: [")).starts_with("fixture does not parse: "));
        assert_eq!(
            failed(Ok(
                "format: protocol/1\nprotocol: {id: 'a b', revision: 1}\n"
            )),
            "fixture does not compile: invalid-identifier: protocol identifier `a b` is empty or contains whitespace or a control character"
        );
        assert_eq!(
            failed(Ok("format: protocol/1\nprotocol: {id: p, revision: 2}\n")),
            format!(
                "canon-ir/1 differs from the expectation at line {}",
                compiled()
                    .lines()
                    .position(|line| line.contains("\"revision\""))
                    .expect("revision line")
                    + 1
            )
        );
    }

    #[test]
    fn first_difference_counts_lines_from_one_and_sees_a_trailing_newline() {
        assert_eq!(first_difference("a\nb\n", "a\nb\n"), None);
        assert_eq!(first_difference("a\nb\n", "a\nc\n"), Some(2));
        assert_eq!(first_difference("a\nb\n", "x\nb\n"), Some(1));
        assert_eq!(first_difference("a\nb\n", "a\nb"), Some(2));
        assert_eq!(first_difference("a\nb\n", "a\nb\nc\n"), Some(3));
        assert_eq!(first_difference("", "a"), Some(1));
    }

    #[test]
    fn a_registry_reports_each_file_and_refuses_a_reused_scenario_id() {
        let ok = scenario_text("S", &compiled());
        let files = [
            ScenarioFile {
                path: "dir/a.yaml".to_owned(),
                text: Ok(ok.clone()),
            },
            ScenarioFile {
                path: "dir/b.yaml".to_owned(),
                text: Ok(ok),
            },
            ScenarioFile {
                path: "dir/c\n.yaml".to_owned(),
                text: Err("Permission denied".to_owned()),
            },
        ];
        let mut asked = Vec::new();
        let report = run_registry(&files, |fixture| {
            asked.push(fixture.to_owned());
            Ok(PROTOCOL.to_owned())
        });
        assert_eq!(asked, ["p.yaml"]);
        assert_eq!(
            report.render(),
            "passed: scenario `S`\n\
             unreadable: dir/b.yaml: scenario id `S` is already used by dir/a.yaml\n\
             unreadable: dir/c\\n.yaml: Permission denied\n\
             conform: 1 passed, 0 failed, 2 unreadable\n"
        );
        assert!(!report.all_passed());
        assert!(run_registry(&files[..1], |_| Ok(PROTOCOL.to_owned())).all_passed());
    }

    #[test]
    fn an_empty_registry_is_refused_not_passed() {
        let report = run_registry(&[], |_| panic!("no fixture is read"));
        assert!(!report.all_passed());
        assert_eq!(
            report.render(),
            "no scenario ran: the registry holds no scenario file\n\
             conform: 0 passed, 0 failed, 0 unreadable\n"
        );
        let only_unreadable = run_registry(
            &[ScenarioFile {
                path: "dir/a.yaml".to_owned(),
                text: Err("Permission denied".to_owned()),
            }],
            |_| panic!("no fixture is read"),
        );
        assert!(
            !only_unreadable.render().contains("no scenario ran"),
            "a registry with a file is not empty"
        );
    }

    #[test]
    fn fixture_paths_are_confined_to_the_working_directory() {
        for confined in [
            "p.yaml",
            "fixtures/investigation/protocol.yaml",
            "./p.yaml",
            "a/..b/p.yaml",
            "a/b../p.yaml",
            "...",
            "dir\\p.yaml",
        ] {
            assert!(is_confined(confined), "{confined}");
        }
        for escaping in [
            "/etc/hosts",
            "\\server\\share\\p.yaml",
            "C:\\p.yaml",
            "c:p.yaml",
            "..",
            "../p.yaml",
            "a/../../p.yaml",
            "a/..",
            "a\\..\\..\\p.yaml",
            "a/b/..\\p.yaml",
        ] {
            assert!(!is_confined(escaping), "{escaping}");
            let source = format!(
                "format: canon-conformance/1\nid: S\ncovers: []\nfixture: '{escaping}'\nsteps:\n  - id: c\n    compile: {{}}\n    expect: {{canon-ir: x}}\n"
            );
            assert_eq!(
                refusal(&source),
                format!(
                    "fixture `{}` is not a relative path without `..` components",
                    model::one_line(escaping)
                ),
                "{escaping}"
            );
        }
    }

    #[test]
    fn an_escaping_fixture_reports_the_scenario_unreadable_and_is_never_read() {
        let source = "format: canon-conformance/1\nid: S\ncovers: []\nfixture: ../outside.yaml\nsteps:\n  - id: c\n    compile: {}\n    expect: {canon-ir: x}\n";
        let report = run_registry(
            &[ScenarioFile {
                path: "dir/a.yaml".to_owned(),
                text: Ok(source.to_owned()),
            }],
            |fixture| panic!("read {fixture}"),
        );
        assert_eq!(
            report.render(),
            "unreadable: dir/a.yaml: fixture `../outside.yaml` is not a relative path without `..` components\n\
             conform: 0 passed, 0 failed, 1 unreadable\n"
        );
        assert!(!report.all_passed());
    }

    #[test]
    fn a_fixture_that_does_not_parse_is_reported_by_kind_and_position_not_content() {
        let scenario = parse(&scenario_text("S", &compiled())).expect("parses");
        let reason = |fixture: &str| match run(&scenario, Ok(fixture)) {
            Verdict::Failed { reason, .. } => reason,
            Verdict::Passed => panic!("passed"),
        };
        let cases = [
            (
                "token: SECRET-VALUE-1\npassword: [SECRET-VALUE-2\n",
                "fixture does not parse: not well-formed YAML at line 3 column 1",
            ),
            (
                "format: protocol/1\nprotocol: SECRET-VALUE-3\n",
                "fixture does not parse: not a protocol/1 document at line 2 column 11",
            ),
            (
                "SECRET-VALUE-4: 1\n",
                "fixture does not parse: not a protocol/1 document at line 1 column 1",
            ),
            (
                "format: protocol/1\nprotocol: {id: p, revision: 1}\nSECRET-VALUE-5: x\n",
                "fixture does not parse: not a protocol/1 document at line 3 column 1",
            ),
        ];
        for (fixture, expected) in cases {
            assert!(model::parse(fixture).is_err(), "{fixture}");
            let reason = reason(fixture);
            assert!(!reason.contains("SECRET"), "{reason}");
            assert_eq!(reason, expected, "{fixture}");
        }
    }

    #[test]
    fn a_registry_file_that_does_not_parse_is_reported_by_kind_and_position_not_content() {
        let cases = [
            (
                "SECRET-SCALAR-VALUE\n",
                "not a canon-conformance/1 scenario at line 1 column 1",
            ),
            (
                "key: [SECRET-VALUE\n",
                "not well-formed YAML at line 2 column 1",
            ),
            (
                "format: canon-conformance/1\nid: S\ncovers: []\nfixture: p.yaml\nsteps: SECRET-VALUE\n",
                "not a canon-conformance/1 scenario at line 5 column 8",
            ),
            (
                "format: canon-conformance/1\nid: S\ncovers: []\nfixture: p.yaml\nsteps: []\nSECRET-KEY: 1\n",
                "not a canon-conformance/1 scenario at line 6 column 1",
            ),
        ];
        for (source, expected) in cases {
            let message = refusal(source);
            for leaked in ["SECRET", "Raw", "struct", "invalid type", "unknown field"] {
                assert!(!message.contains(leaked), "{leaked}: {message}");
            }
            assert_eq!(message, expected, "{source}");
            let report = run_registry(
                &[ScenarioFile {
                    path: "dir/a.yaml".to_owned(),
                    text: Ok(source.to_owned()),
                }],
                |_| panic!("no fixture is read"),
            );
            assert_eq!(
                report.render(),
                format!(
                    "unreadable: dir/a.yaml: {expected}\nconform: 0 passed, 0 failed, 1 unreadable\n"
                )
            );
        }
    }

    #[test]
    fn a_fixture_the_reader_refuses_as_outside_reports_the_scenario_unreadable() {
        let report = run_registry(
            &[ScenarioFile {
                path: "dir/a.yaml".to_owned(),
                text: Ok(scenario_text("S", &compiled())),
            }],
            |_| Err(FixtureError::Outside),
        );
        assert_eq!(
            report.render(),
            "unreadable: dir/a.yaml: fixture `p.yaml` resolves outside the working directory\n\
             conform: 0 passed, 0 failed, 1 unreadable\n"
        );
        assert!(!report.all_passed());
    }
}
