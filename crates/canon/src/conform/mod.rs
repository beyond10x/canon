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
//! imports: [fixtures/investigation/library.yaml] # optional: the documents the fixture imports
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
//!   - id: owner-missing
//!     trigger-validate:
//!       document: |
//!         { ... a canon-trigger/1 document ... }
//!     expect:
//!       refused: owner                      # the field every error names
//!       # or, instead of `refused`:
//!       # rule: stale-invoice-reminder@3    # the key of the valid rule
//!   - id: every-action-read-only
//!     trigger-protocols:
//!       documents:
//!         - |
//!           { ... a protocol/1 document ... }
//!     expect:
//!       protocols: |
//!         { ... the exact canon-trigger-protocols/1 bytes ... }
//!       # or, instead of `protocols`:
//!       # refusal: read-only-action-writes  # the code of every problem
//!   - id: auto-and-offer
//!     trigger-evaluate:
//!       rules:
//!         - state: active                   # draft | active | disabled
//!           document: |
//!             { ... a canon-trigger/1 document ... }
//!       snapshot: { ... a canon-trigger-snapshot/1 document ... }
//!       protocols: { ... a canon-trigger-protocols/1 document ... }
//!       history: { ... a canon-trigger-history/1 document ... }
//!       now: 2026-11-26T12:00:00Z
//!     expect:
//!       plan: |
//!         { ... the exact canon-trigger-plan/1 bytes ... }
//!       # or, instead of `plan`:
//!       # refusal: duplicate-row            # the code of the refusal
//! ```
//!
//! - `fixture` is the `protocol/1` document the scenario compiles, as a path relative to the
//!   working directory `canon conform run` runs in (the repository root, by convention). An
//!   absolute path (a leading `/` or `\`, or a drive prefix such as `C:`) or a path with a `..`
//!   component, split on `/` or `\`, is refused: the scenario does not parse. A path that passes
//!   that check but resolves outside the working directory through a symbolic link is refused
//!   too, unread: the scenario is reported `unreadable` with `fixture `<path>` resolves outside
//!   the working directory`.
//! - `imports` lists the `protocol/1` documents the fixture imports, directly or through another
//!   import (story:protocol-imports), each a path held to the same two checks as `fixture`: one
//!   that is empty, absolute or has a `..` component is refused and the scenario does not parse
//!   (`import `<path>` is not a relative path without `..` components`); one that resolves
//!   outside the working directory is refused unread, and the scenario is reported `unreadable`
//!   with `import `<path>` resolves outside the working directory`. Every step compiles the
//!   fixture composed with them (`ir::compose`), in any order; an import that cannot be read fails
//!   the step as `import `<path>` is unreadable: <why>`, one that does not parse as `import
//!   `<path>` does not parse: <kind and position>`, and a composition that is refused as
//!   `fixture does not compile: <code>: <why>; …`. Absent, the fixture imports nothing.
//! - `covers` entries are `CANON-` followed by an identifier, each listed once.
//! - `steps` is an ordered, non-empty list. Each step is exactly one of `compile`, `evaluate`,
//!   `trigger-validate`, `trigger-protocols` or `trigger-evaluate`, with exactly one expectation
//!   of the matching kind: a compile step expects `canon-ir`, an evaluate step expects `decision`
//!   or `refusal`, a trigger-validate step expects `rule` or `refused`, neither empty, a
//!   trigger-protocols step expects `protocols` or `refusal`, and a trigger-evaluate step expects
//!   `plan` or `refusal`.
//! - A trigger-evaluate step (story:trigger-evaluate) reads each of its `rules` entries'
//!   `document` as `canon trigger validate` reads a file, with the entry's `state`, and its
//!   `snapshot`, `protocols` and `history` as `canon trigger evaluate` reads its files, and
//!   evaluates them at `now` (`trigger::evaluate`); it does not read the fixture. It passes when
//!   the `canon-trigger-plan/1` text equals `plan` byte for byte, or when the evaluation, or a
//!   document it reads, is refused with the code `refusal`. Otherwise `<what differed>` is
//!   `canon-trigger-plan/1 differs from the expectation at line <n>`, `expected refusal `<code>`,
//!   the evaluation gave canon-trigger-plan/1`, `expected refusal `<code>`, the evaluation is
//!   refused: error[<code>]: <why>; …`, or `the evaluation is refused: error[<code>]: <why>; …`; a
//!   rule document that is not a valid rule is refused with each of its errors, named `rule <n>`,
//!   counted from 1.
//! - A trigger-protocols step (story:protocol-access-class) reads its non-empty list of
//!   `documents` as `canon trigger protocols` reads its files, in list order, each named
//!   `document <n>`, counted from 1; it does not read the fixture. Each document is parsed and
//!   compiled on its own, and the first one refused stops the step: one that does not parse is
//!   `parse`, one with imports `protocol-imports`, one that does not compile has each problem's
//!   code (`read-only-action-writes`, …); a set with two documents of one protocol id is
//!   `duplicate-protocol-id`. It passes when the `canon-trigger-protocols/1` text equals
//!   `protocols` byte for byte, or when the documents are refused and every problem has the code
//!   `refusal`. Otherwise `<what differed>` is `canon-trigger-protocols/1 differs from the
//!   expectation at line <n>`, `expected refusal `<code>`, the documents gave
//!   canon-trigger-protocols/1`, `expected refusal `<code>` alone, the documents are refused:
//!   error[<code>]: <why>; …`, or `the documents are refused: error[<code>]: <why>; …`.
//! - A trigger-validate step (story:trigger-rule-document) reads its `document` as
//!   `canon trigger validate` reads a file (`trigger::read`); it does not read the fixture. It
//!   passes when the document is a valid rule whose key, `<id>@<version>`, is `rule`, or when the
//!   document is refused and every error names the field `refused`. Otherwise `<what differed>` is
//!   `expected rule `<key>`, the document is rule `<key>``, `expected a refusal naming `<field>`,
//!   the document is rule `<key>``, or `expected … , the document is refused: error[<code>]:
//!   <field>: <why>; …`, listing every error.
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
//!
//! # The requirements catalogue
//!
//! The registry answers for a catalogue of the normative requirements Canon implements,
//! `conformance/requirements.yaml`, a `canon-requirements/1` document:
//!
//! ```yaml
//! format: canon-requirements/1
//! requirements:
//!   CANON-CLAIM-001:
//!     status: implemented        # implemented | planned
//!     statement: A required positive predicate evaluating to UNKNOWN MUST NOT be treated as satisfied.
//!     source: design § 32        # where the requirement is stated
//! ```
//!
//! Keys are `CANON-` followed by an identifier, each listed once; `status`, `statement` and
//! `source` are required, and the statement and source are not empty. Unknown keys are refused. A
//! `planned` requirement is stated but not yet implemented.
//!
//! `canon conform run --requirements <catalogue>` reads the catalogue before the registry; one
//! that cannot be read exits 2 with `error[unreadable]: <path>: <why>` on standard error, and one
//! that does not parse as a catalogue exits 1 with `error[catalogue]: <path>: <why>`, the
//! catalogue reported by kind and position only, as a registry file is. Either way nothing is
//! printed on standard output. Then the registry runs as above, and these checks follow:
//!
//! 1. **Coverage.** Every `implemented` requirement is covered by at least one scenario that
//!    passed; otherwise `uncovered: requirement `<id>``. A `planned` requirement needs no
//!    scenario; one a passing scenario covers is reported `planned requirement `<id>` is covered`,
//!    so the catalogue is moved forward rather than drifting. Both in catalogue order.
//! 2. **Unlisted.** A scenario that covers a requirement the catalogue does not list, passed or
//!    not, is reported `unlisted: requirement `<id>` covered by scenario `<scenario>``, in registry
//!    order.
//! 3. **Determinism.** Every evaluate step of every scenario whose fixture compiles runs three
//!    times: as written, again as written, and with its `evidence`, `authority` and `decisions`
//!    lists in reverse order, an equivalent input, since none of the three is ordered by position
//!    (CANON-DETERMINISM-001). The three outputs, each the canonical `canon-decision/1` document or
//!    the refusal's code, must be byte-identical; otherwise `nondeterministic: scenario `<id>` step
//!    `<step>`: <rerun|permuted> differs at line <n>`, `<n>` the first line, counted from 1, where
//!    that run's output differs from the first. In registry and step order. Every trigger-evaluate
//!    step runs three times the same way, the permuted run with its rules, the snapshot's kinds,
//!    each kind's rows, and the history's firings and activations in reverse order
//!    (CANON-TRIGGER-008); its output is the canonical `canon-trigger-plan/1` document or the
//!    refusal's code.
//!
//! The check lines come after the registry's lines and before its summary, and a second summary
//! line follows it:
//!
//! ```text
//! requirements: <n> implemented, <n> covered, <n> planned
//! ```
//!
//! `covered` counts the implemented requirements a passing scenario covers. The run exits 0 only
//! when the registry passed and no check failed, and 1 otherwise. Without `--requirements` the
//! report is the registry's alone, byte for byte.

mod requirements;

pub use requirements::{CatalogueError, parse_catalogue};

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer};

use crate::{eval, ir, model, trigger};

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
    /// The documents the fixture imports, as paths like `fixture`, in the order listed.
    pub imports: Vec<String>,
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
    /// Read one `canon-trigger/1` document as `canon trigger validate` does; the fixture is not
    /// read.
    TriggerValidate {
        document: String,
        expected: TriggerExpectation,
    },
    /// Read `protocol/1` documents as `canon trigger protocols` reads its files; the fixture is
    /// not read (story:protocol-access-class).
    TriggerProtocols {
        documents: Vec<String>,
        expected: ProtocolsExpectation,
    },
    /// Evaluate a rule set as `canon trigger evaluate` does; the fixture is not read
    /// (story:trigger-evaluate).
    TriggerEvaluate {
        inputs: TriggerEvaluateInputs,
        expected: PlanExpectation,
    },
}

/// The inputs of a trigger-evaluate step: each rule's state and `canon-trigger/1` text, and the
/// snapshot, protocols and history as YAML values, read as `canon trigger evaluate` reads its files.
#[derive(Debug, Clone, PartialEq)]
pub struct TriggerEvaluateInputs {
    pub rules: Vec<(model::TriggerRuleState, String)>,
    pub snapshot: serde_yaml_ng::Value,
    pub protocols: serde_yaml_ng::Value,
    pub history: serde_yaml_ng::Value,
    pub now: String,
}

/// What a trigger-evaluate step expects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanExpectation {
    /// The exact `canon-trigger-plan/1` bytes.
    Plan(String),
    /// The evaluation, or a document it reads, is refused with this code.
    Refusal(String),
}

/// What a trigger-protocols step expects of its documents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolsExpectation {
    /// The exact `canon-trigger-protocols/1` bytes.
    Protocols(String),
    /// The documents are refused, and every problem has this code.
    Refusal(String),
}

/// What a trigger-validate step expects of its document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerExpectation {
    /// The document is a valid rule with this key, `<id>@<version>`.
    Rule(String),
    /// The document is refused, and every error names this field.
    Refused(String),
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
    #[serde(default, deserialize_with = "optional")]
    imports: Option<Vec<String>>,
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
    #[serde(rename = "trigger-validate", default, deserialize_with = "optional")]
    trigger_validate: Option<RawTriggerValidate>,
    #[serde(rename = "trigger-protocols", default, deserialize_with = "optional")]
    trigger_protocols: Option<RawTriggerProtocols>,
    #[serde(rename = "trigger-evaluate", default, deserialize_with = "optional")]
    trigger_evaluate: Option<RawTriggerEvaluate>,
    #[serde(deserialize_with = "required")]
    expect: RawExpect,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTriggerEvaluate {
    #[serde(deserialize_with = "required")]
    rules: Vec<RawRuleEntry>,
    #[serde(deserialize_with = "required")]
    snapshot: serde_yaml_ng::Value,
    #[serde(deserialize_with = "required")]
    protocols: serde_yaml_ng::Value,
    #[serde(deserialize_with = "required")]
    history: serde_yaml_ng::Value,
    #[serde(deserialize_with = "required")]
    now: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRuleEntry {
    #[serde(deserialize_with = "required")]
    state: model::TriggerRuleState,
    #[serde(deserialize_with = "required")]
    document: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawCompile {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTriggerValidate {
    #[serde(deserialize_with = "required")]
    document: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTriggerProtocols {
    #[serde(deserialize_with = "required")]
    documents: Vec<String>,
}

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
    #[serde(default, deserialize_with = "optional")]
    rule: Option<String>,
    #[serde(default, deserialize_with = "optional")]
    refused: Option<String>,
    #[serde(default, deserialize_with = "optional")]
    protocols: Option<String>,
    #[serde(default, deserialize_with = "optional")]
    plan: Option<String>,
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
    let imports = raw.imports.unwrap_or_default();
    for import in &imports {
        if import.is_empty() || !is_confined(import) {
            return refuse(format!(
                "import `{}` is not a relative path without `..` components",
                model::one_line(import)
            ));
        }
    }
    if raw.steps.is_empty() {
        return refuse("steps is empty".to_owned());
    }
    let mut steps: Vec<Step> = Vec::with_capacity(raw.steps.len());
    for step in raw.steps {
        let id = step.id.clone();
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
        let kind = step_kind(&id, step)?;
        steps.push(Step { id, kind });
    }
    Ok(Scenario {
        id: raw.id,
        covers: raw.covers,
        fixture: raw.fixture,
        imports,
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

fn step_kind(id: &str, step: RawStep) -> Result<StepKind, ScenarioError> {
    let id = model::one_line(id);
    let RawStep {
        compile,
        evaluate,
        trigger_validate,
        trigger_protocols,
        trigger_evaluate,
        expect,
        id: _,
    } = step;
    if let Some(inputs) = trigger_evaluate {
        let alone = compile.is_none()
            && evaluate.is_none()
            && trigger_validate.is_none()
            && trigger_protocols.is_none();
        let expected = match expect {
            RawExpect {
                canon_ir: None,
                decision: None,
                refusal: None,
                rule: None,
                refused: None,
                protocols: None,
                plan: Some(plan),
            } => PlanExpectation::Plan(plan),
            RawExpect {
                canon_ir: None,
                decision: None,
                refusal: Some(refusal),
                rule: None,
                refused: None,
                protocols: None,
                plan: None,
            } if model::is_identifier(&refusal) => PlanExpectation::Refusal(refusal),
            _ if !alone => return refuse(exactly_one(&id)),
            _ => {
                return refuse(format!(
                    "trigger-evaluate step `{id}` must expect exactly one of `plan` or `refusal` (an identifier)"
                ));
            }
        };
        if !alone {
            return refuse(exactly_one(&id));
        }
        return Ok(StepKind::TriggerEvaluate {
            inputs: TriggerEvaluateInputs {
                rules: inputs
                    .rules
                    .into_iter()
                    .map(|entry| (entry.state, entry.document))
                    .collect(),
                snapshot: inputs.snapshot,
                protocols: inputs.protocols,
                history: inputs.history,
                now: inputs.now,
            },
            expected,
        });
    }
    match (
        compile,
        evaluate,
        trigger_validate,
        trigger_protocols,
        expect,
    ) {
        (
            Some(RawCompile {}),
            None,
            None,
            None,
            RawExpect {
                canon_ir: Some(expected_ir),
                decision: None,
                refusal: None,
                rule: None,
                refused: None,
                protocols: None,
                plan: None,
            },
        ) => Ok(StepKind::Compile { expected_ir }),
        (Some(_), None, None, None, _) => refuse(format!(
            "compile step `{id}` must expect `canon-ir` and nothing else"
        )),
        (None, Some(evaluate), None, None, expect) => {
            let expected = match expect {
                RawExpect {
                    canon_ir: None,
                    decision: Some(decision),
                    refusal: None,
                    rule: None,
                    refused: None,
                    protocols: None,
                    plan: None,
                } => EvaluateExpectation::Decision(decision),
                RawExpect {
                    canon_ir: None,
                    decision: None,
                    refusal: Some(refusal),
                    rule: None,
                    refused: None,
                    protocols: None,
                    plan: None,
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
        (None, None, Some(trigger), None, expect) => {
            let expected = match expect {
                RawExpect {
                    canon_ir: None,
                    decision: None,
                    refusal: None,
                    rule: Some(rule),
                    refused: None,
                    protocols: None,
                    plan: None,
                } if !rule.is_empty() => TriggerExpectation::Rule(rule),
                RawExpect {
                    canon_ir: None,
                    decision: None,
                    refusal: None,
                    rule: None,
                    refused: Some(field),
                    protocols: None,
                    plan: None,
                } if !field.is_empty() => TriggerExpectation::Refused(field),
                _ => {
                    return refuse(format!(
                        "trigger-validate step `{id}` must expect exactly one of `rule` or `refused`, not empty"
                    ));
                }
            };
            Ok(StepKind::TriggerValidate {
                document: trigger.document,
                expected,
            })
        }
        (None, None, None, Some(protocols), expect) => {
            let expected = match expect {
                RawExpect {
                    canon_ir: None,
                    decision: None,
                    refusal: None,
                    rule: None,
                    refused: None,
                    protocols: Some(expected),
                    plan: None,
                } => ProtocolsExpectation::Protocols(expected),
                RawExpect {
                    canon_ir: None,
                    decision: None,
                    refusal: Some(refusal),
                    rule: None,
                    refused: None,
                    protocols: None,
                    plan: None,
                } if model::is_identifier(&refusal) => ProtocolsExpectation::Refusal(refusal),
                _ => {
                    return refuse(format!(
                        "trigger-protocols step `{id}` must expect exactly one of `protocols` or `refusal` (an identifier)"
                    ));
                }
            };
            if protocols.documents.is_empty() {
                return refuse(format!("trigger-protocols step `{id}` lists no document"));
            }
            Ok(StepKind::TriggerProtocols {
                documents: protocols.documents,
                expected,
            })
        }
        _ => refuse(exactly_one(&id)),
    }
}

/// Why a step with none, or more than one, of the step kinds does not parse.
fn exactly_one(id: &str) -> String {
    format!(
        "step `{id}` must be exactly one of `compile`, `evaluate`, `trigger-validate`, `trigger-protocols` or `trigger-evaluate`"
    )
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
/// the scenario's fixture, or why it could not be read. The scenario's fixture is compiled with no
/// imported document; [`run_with_imports`] passes them.
pub fn run(scenario: &Scenario, fixture: Result<&str, &str>) -> Verdict {
    run_with_imports(scenario, fixture, &[])
}

/// Runs a scenario as [`run`] does, composing the fixture with `imports`: for each path the
/// scenario lists under `imports`, in order, its text or why it could not be read.
pub fn run_with_imports(
    scenario: &Scenario,
    fixture: Result<&str, &str>,
    imports: &[Result<String, String>],
) -> Verdict {
    run_compiled(scenario, &compiled_fixture(scenario, fixture, imports))
}

/// Runs a scenario's steps against its fixture compiled once, or against why it did not compile.
/// Every step of a scenario reads the same fixture and imports, so one compile serves them all and
/// each step sees what a compile of its own would give.
fn run_compiled(scenario: &Scenario, compiled: &Result<ir::Ir, String>) -> Verdict {
    for step in &scenario.steps {
        let failure = match &step.kind {
            StepKind::Compile { expected_ir } => compile_step(compiled, expected_ir),
            StepKind::Evaluate { inputs, expected } => evaluate_step(compiled, inputs, expected),
            StepKind::TriggerValidate { document, expected } => trigger_step(document, expected),
            StepKind::TriggerProtocols {
                documents,
                expected,
            } => protocols_step(documents, expected),
            StepKind::TriggerEvaluate { inputs, expected } => plan_step(inputs, expected),
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

#[cfg(test)]
thread_local! {
    /// How many fixtures this thread composed and compiled; the count tests read it.
    static COMPILES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// The fixture's compiled protocol, composed with its imports, or why the step that needs it fails.
fn compiled_fixture(
    scenario: &Scenario,
    fixture: Result<&str, &str>,
    imports: &[Result<String, String>],
) -> Result<ir::Ir, String> {
    let source = fixture.map_err(|why| {
        format!(
            "fixture `{}` is unreadable: {}",
            model::one_line(&scenario.fixture),
            model::one_line(why)
        )
    })?;
    let protocol = model::parse(source)
        .map_err(|_| format!("fixture does not parse: {}", fixture_parse_failure(source)))?;
    let mut imported = Vec::with_capacity(imports.len());
    for (path, text) in scenario.imports.iter().zip(imports) {
        let path = model::one_line(path);
        let text = text
            .as_deref()
            .map_err(|why| format!("import `{path}` is unreadable: {}", model::one_line(why)))?;
        let document = model::parse(text).map_err(|_| {
            format!(
                "import `{path}` does not parse: {}",
                fixture_parse_failure(text)
            )
        })?;
        imported.push(document);
    }
    #[cfg(test)]
    COMPILES.with(|count| count.set(count.get() + 1));
    ir::compose(&protocol, &imported)
        .and_then(|composed| ir::compile(&composed))
        .map_err(|problems| {
            let problems: Vec<String> = problems
                .iter()
                .map(|problem| format!("{}: {problem}", problem.code()))
                .collect();
            format!("fixture does not compile: {}", problems.join("; "))
        })
}

/// Reads a trigger-validate step's document with [`trigger::read`] and compares it with the
/// expectation (see the module docs).
fn trigger_step(document: &str, expected: &TriggerExpectation) -> Option<String> {
    let refused = |problems: &[trigger::Problem]| {
        problems
            .iter()
            .map(|problem| format!("error[{}]: {problem}", problem.code))
            .collect::<Vec<_>>()
            .join("; ")
    };
    match (trigger::read(document), expected) {
        (Ok(rule), TriggerExpectation::Rule(key)) if rule.key() == *key => None,
        (Ok(rule), TriggerExpectation::Rule(key)) => Some(format!(
            "expected rule `{}`, the document is rule `{}`",
            model::one_line(key),
            model::one_line(&rule.key())
        )),
        (Ok(rule), TriggerExpectation::Refused(field)) => Some(format!(
            "expected a refusal naming `{}`, the document is rule `{}`",
            model::one_line(field),
            model::one_line(&rule.key())
        )),
        (Err(problems), TriggerExpectation::Refused(field))
            if problems.iter().all(|problem| problem.field == *field) =>
        {
            None
        }
        (Err(problems), TriggerExpectation::Refused(field)) => Some(format!(
            "expected a refusal naming `{}` alone, the document is refused: {}",
            model::one_line(field),
            refused(&problems)
        )),
        (Err(problems), TriggerExpectation::Rule(key)) => Some(format!(
            "expected rule `{}`, the document is refused: {}",
            model::one_line(key),
            refused(&problems)
        )),
    }
}

/// Reads a trigger-protocols step's documents as `canon trigger protocols` reads its files, each
/// named `document <n>`, counted from 1, and compares the `canon-trigger-protocols/1` bytes or the
/// refusal with the expectation (see the module docs).
fn protocols_step(documents: &[String], expected: &ProtocolsExpectation) -> Option<String> {
    let outcome = trigger_protocols(documents);
    let refused = |problems: &[(&str, String)]| {
        problems
            .iter()
            .map(|(code, why)| format!("error[{code}]: {why}"))
            .collect::<Vec<_>>()
            .join("; ")
    };
    match (outcome, expected) {
        (Ok(printed), ProtocolsExpectation::Protocols(expected)) => {
            first_difference(&printed, expected).map(|line| {
                format!("canon-trigger-protocols/1 differs from the expectation at line {line}")
            })
        }
        (Ok(_), ProtocolsExpectation::Refusal(code)) => Some(format!(
            "expected refusal `{}`, the documents gave canon-trigger-protocols/1",
            model::one_line(code)
        )),
        (Err(problems), ProtocolsExpectation::Refusal(code))
            if problems.iter().all(|(found, _)| found == code) =>
        {
            None
        }
        (Err(problems), ProtocolsExpectation::Refusal(code)) => Some(format!(
            "expected refusal `{}` alone, the documents are refused: {}",
            model::one_line(code),
            refused(&problems)
        )),
        (Err(problems), ProtocolsExpectation::Protocols(_)) => {
            Some(format!("the documents are refused: {}", refused(&problems)))
        }
    }
}

/// The `canon-trigger-protocols/1` text of the documents, or each `(code, why)` of the first
/// document refused, in list order, or the duplicate protocol id of the set.
fn trigger_protocols(documents: &[String]) -> Result<String, Vec<(&'static str, String)>> {
    let mut set = Vec::with_capacity(documents.len());
    for (at, text) in documents.iter().enumerate() {
        let name = format!("document {}", at + 1);
        let Ok(protocol) = model::parse(text) else {
            return Err(vec![(
                "parse",
                format!("{name}: {}", fixture_parse_failure(text)),
            )]);
        };
        if let Some(first) = protocol.imports.ids().next() {
            return Err(vec![(
                "protocol-imports",
                format!(
                    "{name}: the protocol imports `{}`",
                    model::one_line(first.as_str())
                ),
            )]);
        }
        match ir::compile(&protocol) {
            Ok(compiled) => set.push((name, compiled)),
            Err(problems) => {
                return Err(problems
                    .iter()
                    .map(|problem| (problem.code(), format!("{name}: {problem}")))
                    .collect());
            }
        }
    }
    trigger::protocols(set.iter().map(|(name, compiled)| (name.as_str(), compiled)))
        .map(|document| trigger::render_protocols(&document))
        .map_err(|duplicate| vec![(trigger::DuplicateProtocolId::CODE, duplicate.to_string())])
}

/// Evaluates a trigger-evaluate step's inputs as `canon trigger evaluate` does and compares the
/// `canon-trigger-plan/1` bytes or the refusal with the expectation (see the module docs).
fn plan_step(inputs: &TriggerEvaluateInputs, expected: &PlanExpectation) -> Option<String> {
    let refused = |problems: &[(&str, String)]| {
        problems
            .iter()
            .map(|(code, why)| format!("error[{code}]: {why}"))
            .collect::<Vec<_>>()
            .join("; ")
    };
    match (trigger_plan(inputs, false), expected) {
        (Ok(printed), PlanExpectation::Plan(expected)) => {
            first_difference(&printed, expected).map(|line| {
                format!("canon-trigger-plan/1 differs from the expectation at line {line}")
            })
        }
        (Ok(_), PlanExpectation::Refusal(code)) => Some(format!(
            "expected refusal `{}`, the evaluation gave canon-trigger-plan/1",
            model::one_line(code)
        )),
        (Err(problems), PlanExpectation::Refusal(code))
            if problems.iter().all(|(found, _)| found == code) =>
        {
            None
        }
        (Err(problems), PlanExpectation::Refusal(code)) => Some(format!(
            "expected refusal `{}`, the evaluation is refused: {}",
            model::one_line(code),
            refused(&problems)
        )),
        (Err(problems), PlanExpectation::Plan(_)) => {
            Some(format!("the evaluation is refused: {}", refused(&problems)))
        }
    }
}

/// The `canon-trigger-plan/1` text of a trigger-evaluate step, or each `(code, why)` of its
/// refusal: every error of the first rule document that is not a valid rule, named `rule <n>`, or
/// the refusal of a document or of the evaluation. `permuted` reverses the rules, the snapshot's
/// kinds and each kind's rows, and the history's firings and activations before evaluating.
pub(crate) fn trigger_plan(
    inputs: &TriggerEvaluateInputs,
    permuted: bool,
) -> Result<String, Vec<(&'static str, String)>> {
    let one = |refusal: trigger::Refusal| vec![(refusal.code, refusal.message)];
    let mut rules = Vec::with_capacity(inputs.rules.len());
    for (at, (state, document)) in inputs.rules.iter().enumerate() {
        match trigger::read(document) {
            Ok(rule) => rules.push(trigger::RuleEntry {
                rule,
                state: *state,
            }),
            Err(problems) => {
                return Err(problems
                    .iter()
                    .map(|problem| (problem.code, format!("rule {}: {problem}", at + 1)))
                    .collect());
            }
        }
    }
    let text = |what: &str, value: &serde_yaml_ng::Value| {
        serde_yaml_ng::to_string(value).map_err(|error| {
            vec![(
                "malformed-input",
                format!(
                    "{what} cannot be serialized: {}",
                    model::one_line(&error.to_string())
                ),
            )]
        })
    };
    let mut snapshot = trigger::read_snapshot(&text("snapshot", &inputs.snapshot)?).map_err(one)?;
    let protocols = trigger::read_protocols(&text("protocols", &inputs.protocols)?).map_err(one)?;
    let mut history = trigger::read_history(&text("history", &inputs.history)?).map_err(one)?;
    if permuted {
        rules.reverse();
        let mut kinds: Vec<_> = snapshot
            .kinds
            .iter()
            .map(|(kind, rows)| (kind.clone(), rows.iter().rev().cloned().collect()))
            .collect();
        kinds.reverse();
        snapshot.kinds = model::Declarations::new(kinds);
        history.firings.reverse();
        history.activations.reverse();
    }
    trigger::evaluate(
        &rules,
        &snapshot,
        &protocols,
        &history,
        &model::Instant::new(inputs.now.as_str()),
    )
    .map(|plan| trigger::render_plan(&plan))
    .map_err(one)
}

fn compile_step(compiled: &Result<ir::Ir, String>, expected_ir: &str) -> Option<String> {
    let compiled = match compiled {
        Ok(compiled) => compiled.canonical_json(),
        Err(why) => return Some(why.clone()),
    };
    first_difference(&compiled, expected_ir)
        .map(|line| format!("canon-ir/1 differs from the expectation at line {line}"))
}

fn evaluate_step(
    compiled: &Result<ir::Ir, String>,
    inputs: &EvaluateInputs,
    expected: &EvaluateExpectation,
) -> Option<String> {
    let compiled = match compiled {
        Ok(compiled) => compiled,
        Err(why) => return Some(why.clone()),
    };
    let evaluation = match evaluation(compiled, inputs) {
        Ok(evaluation) => evaluation,
        Err(why) => return Some(why),
    };
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

/// Evaluates an evaluate step's inputs against the compiled fixture: the decision or the refusal,
/// or why the `authority` or `decisions` list cannot be written back as YAML (see the module
/// docs), in which case nothing is evaluated.
fn evaluation(
    compiled: &ir::Ir,
    inputs: &EvaluateInputs,
) -> Result<Result<model::Decision, eval::Refusal>, String> {
    let written = |list: &Option<Vec<serde_yaml_ng::Value>>, name: &str| {
        list.as_deref()
            .map(|list| list_text(name, list))
            .transpose()
    };
    let authority = written(&inputs.authority, "authority")?;
    let decisions = written(&inputs.decisions, "decisions")?;
    let supplied = eval::Supplied {
        authority: authority.as_deref(),
        at: inputs.at.as_deref(),
        decisions: decisions.as_deref(),
    };
    Ok(eval::case_from_value(&inputs.case).and_then(|case| {
        let evidence = inputs
            .evidence
            .iter()
            .map(eval::evidence_from_value)
            .collect::<Result<Vec<_>, _>>()?;
        eval::evaluate_with(compiled, &case, &evidence, supplied)
    }))
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
/// file-name order). `read_fixture` returns the text of a fixture or of an import a scenario lists,
/// or why it returned none; it is the caller's, so this function reads nothing itself. A file whose
/// scenario id an earlier file already used, or whose fixture or one of whose imports resolves
/// outside the working directory, is reported unreadable.
pub fn run_registry<F>(files: &[ScenarioFile], read_fixture: F) -> Report
where
    F: FnMut(&str) -> Result<String, FixtureError>,
{
    run_scenarios(files, read_fixture).0
}

/// Runs a registry as [`run_registry`] does, then checks it against a `canon-requirements/1`
/// catalogue (see the module docs): coverage, requirements the catalogue does not list, and the
/// determinism of every evaluate step.
pub fn run_registry_with_requirements<F>(
    files: &[ScenarioFile],
    read_fixture: F,
    catalogue: &model::RequirementCatalogue,
) -> CheckedReport
where
    F: FnMut(&str) -> Result<String, FixtureError>,
{
    let (report, ran) = run_scenarios(files, read_fixture);
    let checks = requirements::check(catalogue, &ran);
    CheckedReport { report, checks }
}

/// [`run_registry`]'s report, and every scenario it parsed with its verdict and compiled fixture.
fn run_scenarios<F>(files: &[ScenarioFile], mut read_fixture: F) -> (Report, Vec<requirements::Ran>)
where
    F: FnMut(&str) -> Result<String, FixtureError>,
{
    let mut ran = Vec::new();
    let mut report = Report::default();
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    'files: for file in files {
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
        let fixture = match read_fixture(&scenario.fixture) {
            Ok(text) => Ok(text),
            Err(FixtureError::Unreadable(why)) => Err(why),
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
        let mut imports = Vec::with_capacity(scenario.imports.len());
        for import in &scenario.imports {
            match read_fixture(import) {
                Ok(text) => imports.push(Ok(text)),
                Err(FixtureError::Unreadable(why)) => imports.push(Err(why)),
                Err(FixtureError::Outside) => {
                    report.unreadable(
                        &file.path,
                        &format!(
                            "import `{}` resolves outside the working directory",
                            model::one_line(import)
                        ),
                    );
                    continue 'files;
                }
            }
        }
        let fixture = fixture.as_deref().map_err(String::as_str);
        let compiled = compiled_fixture(&scenario, fixture, &imports);
        let verdict = run_compiled(&scenario, &compiled);
        report.scenario(&scenario.id, &verdict);
        ran.push(requirements::Ran {
            passed: verdict == Verdict::Passed,
            scenario,
            compiled: compiled.ok(),
        });
    }
    (report, ran)
}

/// The report `canon conform run --requirements` prints: the registry's lines, the line of each
/// failed check, the registry's summary and the `requirements:` summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedReport {
    report: Report,
    checks: requirements::Checks,
}

impl CheckedReport {
    /// Whether the registry passed ([`Report::all_passed`]) and every check passed.
    pub fn all_passed(&self) -> bool {
        self.report.all_passed() && self.checks.failures == 0
    }

    /// The report text, the two summary lines last.
    pub fn render(&self) -> String {
        format!(
            "{}{}",
            self.report.render_with(&self.checks.lines),
            self.checks.summary()
        )
    }
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
        self.render_with("")
    }

    /// The report text with `checks`, lines that each end in a newline, before the summary line.
    fn render_with(&self, checks: &str) -> String {
        let empty = if self.is_empty() {
            "no scenario ran: the registry holds no scenario file\n"
        } else {
            ""
        };
        format!(
            "{}{empty}{checks}conform: {} passed, {} failed, {} unreadable\n",
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

    /// A valid `canon-trigger/1` rule, `r@3`, indented as a step's `document` block.
    const RULE: &str = "        format: canon-trigger/1\n        id: r\n        version: 3\n        \
                        owner: o\n        subject:\n          kind: k\n          \
                        state: {field: s, is: x}\n          fields: {s: {type: string}}\n        \
                        start: {protocol: p}\n        mode: offer\n        access: write\n        \
                        refire: once_per_subject\n        cap: 1\n";

    fn trigger_verdict(document: &str, expect: &str) -> Verdict {
        let source = format!(
            "{HEAD}steps:\n  - id: t\n    trigger-validate:\n      document: |\n{document}    \
             expect: {{{expect}}}\n"
        );
        run(&parse(&source).expect("parses"), Ok(PROTOCOL))
    }

    #[test]
    fn a_trigger_validate_step_compares_the_rule_key_or_the_refused_field() {
        let failed = |reason: &str| Verdict::Failed {
            step: "t".to_owned(),
            reason: reason.to_owned(),
        };
        assert_eq!(trigger_verdict(RULE, "rule: r@3"), Verdict::Passed);
        assert_eq!(
            trigger_verdict(RULE, "rule: r@4"),
            failed("expected rule `r@4`, the document is rule `r@3`")
        );
        assert_eq!(
            trigger_verdict(RULE, "refused: owner"),
            failed("expected a refusal naming `owner`, the document is rule `r@3`")
        );
        let no_owner = RULE.replace("        owner: o\n", "");
        assert_eq!(
            trigger_verdict(&no_owner, "refused: owner"),
            Verdict::Passed
        );
        assert_eq!(
            trigger_verdict(&no_owner, "refused: mode"),
            failed(
                "expected a refusal naming `mode` alone, the document is refused: \
                 error[missing-field]: owner: this field is required"
            )
        );
        assert_eq!(
            trigger_verdict(&no_owner, "rule: r@3"),
            failed(
                "expected rule `r@3`, the document is refused: error[missing-field]: owner: this \
                 field is required"
            )
        );
        let two = RULE
            .replace("version: 3", "version: 0")
            .replace("cap: 1", "cap: 0");
        assert_eq!(
            trigger_verdict(&two, "refused: cap"),
            failed(
                "expected a refusal naming `cap` alone, the document is refused: \
                 error[invalid-value]: version: the version is a positive integer, not 0; \
                 error[invalid-value]: cap: the cap is a positive integer, not 0"
            )
        );
    }

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
                "step `c` must be exactly one of `compile`, `evaluate`, `trigger-validate`, `trigger-protocols` or `trigger-evaluate`",
            ),
            (
                format!(
                    "{HEAD}steps:\n  - id: c\n    compile: {{}}\n    evaluate: {{case: {{}}, evidence: []}}\n    expect: {{canon-ir: x}}\n"
                ),
                "step `c` must be exactly one of `compile`, `evaluate`, `trigger-validate`, `trigger-protocols` or `trigger-evaluate`",
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
            (
                format!(
                    "{HEAD}steps:\n  - id: e\n    evaluate: {{case: {{}}, evidence: []}}\n    expect: {{rule: r@1}}\n"
                ),
                "evaluate step `e` must expect exactly one of `decision` or `refusal`",
            ),
            (
                format!(
                    "{HEAD}steps:\n  - id: t\n    trigger-validate: {{document: x}}\n    expect: {{rule: r@1, refused: owner}}\n"
                ),
                "trigger-validate step `t` must expect exactly one of `rule` or `refused`",
            ),
            (
                format!(
                    "{HEAD}steps:\n  - id: t\n    trigger-validate: {{document: x}}\n    expect: {{refusal: r}}\n"
                ),
                "trigger-validate step `t` must expect exactly one of `rule` or `refused`",
            ),
            (
                format!(
                    "{HEAD}steps:\n  - id: t\n    trigger-validate: {{document: x}}\n    expect: {{refused: ''}}\n"
                ),
                "trigger-validate step `t` must expect exactly one of `rule` or `refused`",
            ),
            (
                format!(
                    "{HEAD}steps:\n  - id: t\n    compile: {{}}\n    trigger-validate: {{document: x}}\n    expect: {{canon-ir: x}}\n"
                ),
                "step `t` must be exactly one of `compile`, `evaluate`, `trigger-validate`, `trigger-protocols` or `trigger-evaluate`",
            ),
            (
                format!(
                    "{HEAD}steps:\n  - id: t\n    trigger-validate: {{}}\n    expect: {{rule: r@1}}\n"
                ),
                "not a canon-conformance/1 scenario at line",
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
                "evaluation refused: invalid-instant: the evaluation instant `yesterday` is not an instant in UTC written YYYY-MM-DDTHH:MM:SSZ or YYYY-MM-DDTHH:MM:SS.fffZ (one to nine fraction digits)"
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

    /// A scenario of one compile step and `evaluate_steps` evaluate steps, all of which pass on
    /// [`PROTOCOL`].
    fn many_step_scenario(id: &str, evaluate_steps: usize) -> String {
        let mut source = scenario_text(id, &compiled());
        for n in 0..evaluate_steps {
            source.push_str(&format!(
                "  - id: e{n}\n    evaluate:\n      case: {CASE}\n      evidence: []\n    expect:\n      decision: |\n{DECISION}"
            ));
        }
        source
    }

    fn compiles_during<T>(work: impl FnOnce() -> T) -> (T, usize) {
        let before = COMPILES.with(std::cell::Cell::get);
        let result = work();
        (result, COMPILES.with(std::cell::Cell::get) - before)
    }

    #[test]
    fn a_scenario_compiles_its_fixture_once_however_many_steps_it_has() {
        let scenario = parse(&many_step_scenario("S", 21)).expect("parses");
        assert_eq!(scenario.steps.len(), 22);
        let (verdict, compiles) = compiles_during(|| run(&scenario, Ok(PROTOCOL)));
        assert_eq!(verdict, Verdict::Passed);
        assert_eq!(compiles, 1, "a 22-step scenario run on its own");

        let files = [
            ScenarioFile {
                path: "a.yaml".to_owned(),
                text: Ok(many_step_scenario("A", 21)),
            },
            ScenarioFile {
                path: "b.yaml".to_owned(),
                text: Ok(many_step_scenario("B", 2)),
            },
        ];
        let (report, compiles) =
            compiles_during(|| run_registry(&files, |_| Ok(PROTOCOL.to_owned())));
        assert_eq!(
            report.render(),
            "passed: scenario `A`\npassed: scenario `B`\nconform: 2 passed, 0 failed, 0 unreadable\n"
        );
        assert_eq!(compiles, 2, "a registry of two scenarios, 22 and 3 steps");
    }

    /// The scenario of the review that found the repeated compile, `CANON-GROUND-001`: 22 steps.
    #[test]
    fn the_decision_grounds_scenario_compiles_its_fixture_once() {
        let files = [ScenarioFile {
            path: "conformance/scenarios/decision-grounds.yaml".to_owned(),
            text: Ok(
                include_str!("../../../../conformance/scenarios/decision-grounds.yaml").to_owned(),
            ),
        }];
        let fixture = include_str!("../../../../fixtures/investigation/decision-grounds.yaml");
        let (report, compiles) = compiles_during(|| {
            run_registry(&files, |path| {
                assert_eq!(path, "fixtures/investigation/decision-grounds.yaml");
                Ok(fixture.to_owned())
            })
        });
        assert_eq!(
            report.render(),
            "passed: scenario `CANON-GROUND-001`\nconform: 1 passed, 0 failed, 0 unreadable\n"
        );
        assert_eq!(compiles, 1, "CANON-GROUND-001 in a registry");
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

    /// A scenario's imports are held to both checks `fixture` is (story:protocol-imports): an
    /// empty, absolute or `..` path does not parse and is never read; one the reader finds outside
    /// the working directory reports the scenario unreadable; one that cannot be read or does not
    /// parse fails the step, naming it.
    #[test]
    fn imports_are_confined_like_the_fixture() {
        let with_imports = |imports: &str| {
            format!(
                "format: canon-conformance/1\nid: S\ncovers: []\nfixture: p.yaml\nimports: {imports}\nsteps:\n  - id: c\n    compile: {{}}\n    expect: {{canon-ir: x}}\n"
            )
        };
        for escaping in [
            "''",
            "../lib.yaml",
            "/lib.yaml",
            "'C:lib.yaml'",
            "'dir\\..\\lib.yaml'",
        ] {
            let report = run_registry(
                &[ScenarioFile {
                    path: "dir/a.yaml".to_owned(),
                    text: Ok(with_imports(&format!("[{escaping}]"))),
                }],
                |path| panic!("read {path}"),
            );
            assert!(
                report
                    .render()
                    .starts_with("unreadable: dir/a.yaml: import `"),
                "{escaping}: {}",
                report.render()
            );
            assert!(
                report
                    .render()
                    .contains("` is not a relative path without `..` components\n"),
                "{escaping}: {}",
                report.render()
            );
        }

        let outside = run_registry(
            &[ScenarioFile {
                path: "dir/a.yaml".to_owned(),
                text: Ok(with_imports("[lib.yaml]")),
            }],
            |path| match path {
                "p.yaml" => Ok(PROTOCOL.to_owned()),
                _ => Err(FixtureError::Outside),
            },
        );
        assert_eq!(
            outside.render(),
            "unreadable: dir/a.yaml: import `lib.yaml` resolves outside the working directory\n\
             conform: 0 passed, 0 failed, 1 unreadable\n"
        );

        let scenario = parse(&with_imports("[lib.yaml]")).expect("parses");
        let reason = |imports: &[Result<String, String>]| match run_with_imports(
            &scenario,
            Ok(PROTOCOL),
            imports,
        ) {
            Verdict::Failed { reason, .. } => reason,
            Verdict::Passed => panic!("passed"),
        };
        assert_eq!(
            reason(&[Err("gone".to_owned())]),
            "import `lib.yaml` is unreadable: gone"
        );
        assert_eq!(
            reason(&[Ok("format: [".to_owned())]),
            "import `lib.yaml` does not parse: not well-formed YAML at line 2 column 1"
        );
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

    fn catalogue(entries: &str) -> model::RequirementCatalogue {
        parse_catalogue(&format!(
            "format: canon-requirements/1\nrequirements:{entries}\n"
        ))
        .expect("catalogue parses")
    }

    fn checked(files: &[ScenarioFile], entries: &str) -> CheckedReport {
        run_registry_with_requirements(files, |_| Ok(PROTOCOL.to_owned()), &catalogue(entries))
    }

    fn one_file(text: String) -> [ScenarioFile; 1] {
        [ScenarioFile {
            path: "dir/a.yaml".to_owned(),
            text: Ok(text),
        }]
    }

    /// The scenario of [`scenario_text`] covers CANON-DETERMINISM-001 and passes.
    #[test]
    fn a_checked_registry_reports_coverage_between_the_scenarios_and_both_summaries() {
        let files = one_file(scenario_text("S", &compiled()));
        let entry = |id: &str, status: &str| {
            format!("\n  {id}: {{status: {status}, statement: s, source: d}}")
        };

        let covered = checked(&files, &entry("CANON-DETERMINISM-001", "implemented"));
        assert_eq!(
            covered.render(),
            format!(
                "{}requirements: 1 implemented, 1 covered, 0 planned\n",
                run_registry(&files, |_| Ok(PROTOCOL.to_owned())).render()
            )
        );
        assert!(covered.all_passed());

        let uncovered = checked(
            &files,
            &format!(
                "{}{}{}",
                entry("CANON-X-001", "implemented"),
                entry("CANON-DETERMINISM-001", "implemented"),
                entry("CANON-Y-001", "planned")
            ),
        );
        assert_eq!(
            uncovered.render(),
            "passed: scenario `S`\n\
             uncovered: requirement `CANON-X-001`\n\
             conform: 1 passed, 0 failed, 0 unreadable\n\
             requirements: 2 implemented, 1 covered, 1 planned\n"
        );
        assert!(!uncovered.all_passed());

        let planned = checked(&files, &entry("CANON-DETERMINISM-001", "planned"));
        assert_eq!(
            planned.render(),
            "passed: scenario `S`\n\
             planned requirement `CANON-DETERMINISM-001` is covered\n\
             conform: 1 passed, 0 failed, 0 unreadable\n\
             requirements: 0 implemented, 0 covered, 1 planned\n"
        );
        assert!(!planned.all_passed());

        let unlisted = checked(&files, " {}");
        assert_eq!(
            unlisted.render(),
            "passed: scenario `S`\n\
             unlisted: requirement `CANON-DETERMINISM-001` covered by scenario `S`\n\
             conform: 1 passed, 0 failed, 0 unreadable\n\
             requirements: 0 implemented, 0 covered, 0 planned\n"
        );
        assert!(!unlisted.all_passed());
    }

    /// A requirement only a failing scenario covers is not covered.
    #[test]
    fn a_failing_scenario_covers_nothing() {
        let files = one_file(scenario_text("S", "not the ir\n"));
        let report = checked(
            &files,
            "\n  CANON-DETERMINISM-001: {status: implemented, statement: s, source: d}",
        );
        assert!(
            report
                .render()
                .contains("uncovered: requirement `CANON-DETERMINISM-001`\n"),
            "{}",
            report.render()
        );
        assert!(!report.all_passed());
    }

    /// The evaluator checks evidence records in the order given and refuses at the first that
    /// fails, so two records refused for different reasons give a different refusal code when the
    /// list is reversed: the step is reported, although the scenario passes.
    #[test]
    fn an_evaluate_step_whose_reversed_evidence_differs_is_nondeterministic() {
        let evidence = "      evidence:\n\
             \x20       - {format: canon-evidence/1, id: e1, kind: k, subject: a, subject_revision: r1}\n\
             \x20       - {format: canon-evidence/1, id: e2, kind: 'k k', subject: a, subject_revision: r1}\n";
        let source = format!(
            "format: canon-conformance/1\nid: S\ncovers: [CANON-DETERMINISM-001]\nfixture: p.yaml\nsteps:\n  - id: e\n    evaluate:\n      case: {CASE}\n{evidence}    expect:\n      refusal: undeclared-evidence-kind\n"
        );
        let files = one_file(source);
        let report = checked(
            &files,
            "\n  CANON-DETERMINISM-001: {status: implemented, statement: s, source: d}",
        );
        assert_eq!(
            report.render(),
            "passed: scenario `S`\n\
             nondeterministic: scenario `S` step `e`: permuted differs at line 1\n\
             conform: 1 passed, 0 failed, 0 unreadable\n\
             requirements: 1 implemented, 1 covered, 0 planned\n"
        );
        assert!(!report.all_passed());

        // A step whose three runs give one decision is not reported.
        let deterministic = format!(
            "format: canon-conformance/1\nid: S\ncovers: [CANON-DETERMINISM-001]\nfixture: p.yaml\nsteps:\n  - id: e\n    evaluate:\n      case: {CASE}\n      evidence: []\n    expect:\n      decision: |\n{DECISION}"
        );
        assert!(
            checked(
                &one_file(deterministic),
                "\n  CANON-DETERMINISM-001: {status: implemented, statement: s, source: d}",
            )
            .all_passed()
        );
    }
}
