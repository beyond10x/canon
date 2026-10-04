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
//! Evaluate steps call the library entry point story:three-valued-claims provides. Until it lands,
//! an evaluate step fails as `failed: scenario `<id>` step `<step>`: evaluate steps are not
//! supported yet`.

use std::collections::BTreeMap;
use std::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

use crate::{ir, model};

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
    /// Evaluate a case against the compiled fixture. Kept as written until the evaluator lands.
    Evaluate {
        inputs: EvaluateInputs,
        expected: EvaluateExpectation,
    },
}

/// The inputs of an evaluate step. Their shapes are defined by story:three-valued-claims; until
/// then they are read as YAML values and not interpreted.
#[derive(Debug, Clone, PartialEq)]
pub struct EvaluateInputs {
    pub case: serde_yaml_ng::Value,
    pub evidence: Vec<serde_yaml_ng::Value>,
    pub authority: Option<Vec<serde_yaml_ng::Value>>,
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
            StepKind::Evaluate { .. } => Some("evaluate steps are not supported yet".to_owned()),
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

fn compile_step(
    scenario: &Scenario,
    fixture: Result<&str, &str>,
    expected_ir: &str,
) -> Option<String> {
    let source = match fixture {
        Ok(source) => source,
        Err(why) => {
            return Some(format!(
                "fixture `{}` is unreadable: {}",
                model::one_line(&scenario.fixture),
                model::one_line(why)
            ));
        }
    };
    let protocol = match model::parse(source) {
        Ok(protocol) => protocol,
        Err(_) => {
            return Some(format!(
                "fixture does not parse: {}",
                fixture_parse_failure(source)
            ));
        }
    };
    let compiled = match ir::compile(&protocol) {
        Ok(compiled) => compiled.canonical_json(),
        Err(problems) => {
            let problems: Vec<String> = problems
                .iter()
                .map(|problem| format!("{}: {problem}", problem.code()))
                .collect();
            return Some(format!("fixture does not compile: {}", problems.join("; ")));
        }
    };
    first_difference(&compiled, expected_ir)
        .map(|line| format!("canon-ir/1 differs from the expectation at line {line}"))
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

    #[test]
    fn an_evaluate_step_fails_as_unsupported_after_the_steps_before_it() {
        let source = format!(
            "{HEAD}steps:\n  - id: e\n    evaluate:\n      case: {{id: c}}\n      evidence: []\n      authority: []\n      at: 2026-10-04T00:00:00Z\n    expect: {{refusal: undeclared-outcome}}\n"
        );
        let scenario = parse(&source).expect("parses");
        assert_eq!(
            run(&scenario, Ok(PROTOCOL)),
            Verdict::Failed {
                step: "e".to_owned(),
                reason: "evaluate steps are not supported yet".to_owned()
            }
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
