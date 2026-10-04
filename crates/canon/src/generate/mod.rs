//! Minimal witness scenarios for a compiled protocol (story:scenario-generation).
//!
//! [`generate`] walks the state space [`crate::check`] enumerates, evaluates every state with
//! Canon's own evaluator, and writes one `canon-conformance/1` scenario per declared outcome and per
//! action blocked in some state. Each scenario evaluates the witness state of its subject and
//! expects the evaluation's section for that subject. Pure and deterministic: no clock, network or
//! filesystem, and the same inputs give the same scenarios, byte for byte; `canon generate` writes
//! them to a directory.
//!
//! # Witnesses
//!
//! An outcome's witness is a state in which it is `legitimate`; an action's is a state in which it
//! is `blocked`. Of those, the witness is the state `canon check` would name: the one of least
//! weight (present evidence classes, decided capabilities and taken decisions), ties broken by its
//! rendering. Each present class is one evidence record, so the witness is minimal: removing any one
//! record leads to a state of less weight, in which the subject's status differs, and so does the
//! section the scenario expects. An action never blocked gets no scenario. An outcome legitimate in
//! no state has no witness, and generation is refused; so is a protocol that yields no scenario at
//! all.
//!
//! # The scenarios
//!
//! One scenario per subject, actions first, each group in identifier order. The file is
//! `action.<action>.blocked.yaml` or `outcome.<outcome>.legitimate.yaml`; the scenario id is the
//! protocol id, a dot and the file name without `.yaml`. A scenario covers no requirement, names
//! the protocol document as its fixture, exactly as given, and has one evaluate step, `witness`:
//!
//! - the case `witness` of the protocol at revision `r`, listing every declared artifact at `r`;
//! - one `canon-evidence/1` record per present class, ids `e0`, `e1`, … in dimension order, about
//!   its dimension's artifact at `r`, with its class's result when it has one and no observation
//!   instant;
//! - `authority`, each decided capability, when the witness decides one;
//! - `decisions`, each taken decision for every outcome that requires it, by the principal
//!   `canon-generate` at case revision `r`, when the witness takes one;
//! - the expectation `decision`: the evaluation's `actions` section for an action, its `outcomes`
//!   section for an outcome, as canonical JSON in a YAML literal block.
//!
//! Every string value, and every key taken from the protocol, is written as a YAML double-quoted
//! scalar: `"` and `\` are escaped, and every character YAML cannot hold as written (a control
//! character, U+0085, U+2028, U+2029, U+FEFF, U+FFFE or U+FFFF) as `\uXXXX`. Each
//! scenario is parsed back and run against the protocol text before it is returned; one that does
//! not pass is refused as `unrepresentable`.
//!
//! # Refusals
//!
//! - `fixture-not-confined`: the protocol path is empty, absolute or has a `..` component, so no
//!   scenario naming it as its fixture would parse. `canon generate` refuses with the same code a path that
//!   resolves, through a symbolic link, outside the working directory.
//! - `illegal-file-name-character`: an outcome or action that gets a scenario holds, in its
//!   identifier, a character a file name on a common filesystem cannot hold: `/`, `\`, `:`, `*`,
//!   `?`, `"`, `<`, `>`, `|` or a control character. The refusal names the identifier and the
//!   character.
//! - `file-name-collision`: two scenario file names are equal once each is put in Unicode NFC and
//!   then lowercased, uppercased and lowercased again, so a filesystem that ignores letter case or
//!   normalisation (the default on macOS and Windows) may hold one file for both. The refusal
//!   names both identifiers. This treats as equal: composed and decomposed forms (`é` and `e`
//!   with U+0301); letters that differ only in case, in either direction, including those whose
//!   lower case depends on position or that have more than one lower case (`Σ`, `σ` and `ς`;
//!   `ſ` and `s`; the micro sign and `μ`; the Kelvin sign and `k`; dotless `ı` and `i`); and a
//!   letter whose capital is two letters with those letters (`ß`, `ẞ` and `ss`). It may refuse a
//!   pair one filesystem keeps apart. It does not treat as equal a pair only a language-specific
//!   rule joins, such as the Turkish `İ` and `i`.
//! - `unreachable-outcome`: an outcome is legitimate in no state.
//! - `nothing-to-witness`: the protocol declares no outcome and no action is blocked in any state,
//!   so there is no scenario to write.
//! - `unrepresentable`: a scenario does not read back as one that passes.
//! - Every refusal [`crate::check::check`] makes, with its code: a space above the bound
//!   (`state-space-bound`) and an evaluation the evaluator refuses.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;
use std::fmt;
use std::fmt::Write as _;

use serde_json::Value;
use unicode_normalization::UnicodeNormalization;

use crate::check::{self, REVISION, Space, State};
use crate::conform::{self, Verdict};
use crate::eval::{self, Supplied};
use crate::ir::Ir;
use crate::model::{
    CASE_FORMAT, Case, CaseArtifact, CaseId, Declarations, EVIDENCE_FORMAT, Revision, one_line,
};

/// The case every scenario evaluates, and the id of its one step.
pub const WITNESS: &str = "witness";

/// Who takes an explicit decision in a generated scenario.
pub const PRINCIPAL: &str = "canon-generate";

/// Why generation was refused: a stable machine-readable code and a one-line message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    code: &'static str,
    message: String,
}

impl Refusal {
    fn new(code: &'static str, message: String) -> Self {
        Self { code, message }
    }

    /// A stable machine-readable code for the kind of refusal.
    pub fn code(&self) -> &'static str {
        self.code
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Refusal {}

impl From<check::Refusal> for Refusal {
    fn from(refusal: check::Refusal) -> Self {
        Refusal::new(refusal.code(), refusal.to_string())
    }
}

/// One generated scenario: its file name, its id and its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scenario {
    pub file: String,
    pub id: String,
    pub text: String,
}

/// What a scenario witnesses: a declared action blocked, or a declared outcome legitimate.
struct Subject<'a> {
    /// `action` or `outcome`.
    kind: &'static str,
    id: &'a str,
    /// `blocked` or `legitimate`.
    status: &'static str,
    /// The decision section that carries the subject's status.
    section: &'static str,
}

impl Subject<'_> {
    /// The scenario's file name without `.yaml`.
    fn stem(&self) -> String {
        format!("{}.{}.{}", self.kind, self.id, self.status)
    }

    /// The scenario's file name.
    fn file(&self) -> String {
        format!("{}.yaml", self.stem())
    }
}

/// Whether a file name on a common filesystem cannot hold `c`: a path separator (`/`, `\`), a
/// character Windows refuses in a name (`:`, `*`, `?`, `"`, `<`, `>`, `|`) or a control character.
fn is_illegal_in_file_name(c: char) -> bool {
    matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_control()
}

/// A file name as a filesystem that ignores letter case and Unicode normalisation sees it: in NFC,
/// then lowercased, uppercased and lowercased again, as the module docs say.
fn fold(file: &str) -> String {
    file.nfc()
        .collect::<String>()
        .to_lowercase()
        .to_uppercase()
        .to_lowercase()
}

/// The scenarios witnessing every outcome of `ir` and every action blocked in some state, as the
/// module docs say. `fixture` is the path of the protocol document as the scenarios name it, and
/// `source` the text `ir` was compiled from, against which each scenario is run before it is
/// returned. Pure and deterministic.
pub fn generate(ir: &Ir, fixture: &str, source: &str) -> Result<Vec<Scenario>, Refusal> {
    if fixture.is_empty() || !conform::is_confined(fixture) {
        return Err(Refusal::new(
            "fixture-not-confined",
            format!(
                "the protocol path `{}` is not a relative path without `..` components, so no \
                 scenario can name it as its fixture",
                one_line(fixture)
            ),
        ));
    }
    check::check(ir, None)?;
    let space = Space::new(ir);
    let size = space
        .size()
        .expect("canon check accepted the space, so its size fits");
    let states: Vec<State> = (0..size).map(|index| space.state(index)).collect();
    let evaluated = states
        .iter()
        .map(|state| check::evaluate(ir, &space, state))
        .collect::<Result<Vec<_>, _>>()?;

    let mut witnessed = Vec::new();
    for (at, id) in ir.actions.keys().enumerate() {
        let blocked = (0..states.len()).filter(|index| evaluated[*index].actions[at] == "blocked");
        if let Some(index) = check::witness(&space, &states, blocked) {
            let subject = Subject {
                kind: "action",
                id: id.as_str(),
                status: "blocked",
                section: "actions",
            };
            witnessed.push((subject, index));
        }
    }
    for (at, id) in ir.outcomes.keys().enumerate() {
        let legitimate =
            (0..states.len()).filter(|index| evaluated[*index].outcomes[at] == "legitimate");
        let Some(index) = check::witness(&space, &states, legitimate) else {
            return Err(Refusal::new(
                "unreachable-outcome",
                format!(
                    "outcome `{}` is legitimate in no state, so no scenario witnesses it",
                    one_line(id.as_str())
                ),
            ));
        };
        let subject = Subject {
            kind: "outcome",
            id: id.as_str(),
            status: "legitimate",
            section: "outcomes",
        };
        witnessed.push((subject, index));
    }

    if witnessed.is_empty() {
        return Err(Refusal::new(
            "nothing-to-witness",
            format!(
                "protocol `{}` revision {} declares no outcome and blocks no action in any state, \
                 so there is no scenario to write",
                one_line(ir.protocol.id.as_str()),
                ir.protocol.revision
            ),
        ));
    }
    for (subject, _) in &witnessed {
        if let Some(c) = subject.id.chars().find(|c| is_illegal_in_file_name(*c)) {
            return Err(Refusal::new(
                "illegal-file-name-character",
                format!(
                    "{} `{}` holds `{}`, which its scenario's file name cannot hold",
                    subject.kind,
                    one_line(subject.id),
                    c.escape_default()
                ),
            ));
        }
    }
    let folded: Vec<String> = witnessed
        .iter()
        .map(|(subject, _)| fold(&subject.file()))
        .collect();
    for (later, name) in folded.iter().enumerate() {
        if let Some(earlier) = folded[..later].iter().position(|other| other == name) {
            let (first, second) = (&witnessed[earlier].0, &witnessed[later].0);
            return Err(Refusal::new(
                "file-name-collision",
                format!(
                    "{} `{}` and {} `{}` have scenario file names that name one file where letter \
                     case or Unicode normalisation is ignored",
                    first.kind,
                    one_line(first.id),
                    second.kind,
                    one_line(second.id)
                ),
            ));
        }
    }
    witnessed
        .iter()
        .map(|(subject, index)| scenario(ir, &space, &states[*index], subject, fixture, source))
        .collect()
}

/// The scenario witnessing `subject` in `state`, checked to pass against `source`.
fn scenario(
    ir: &Ir,
    space: &Space<'_>,
    state: &State,
    subject: &Subject<'_>,
    fixture: &str,
    source: &str,
) -> Result<Scenario, Refusal> {
    let stem = subject.stem();
    let id = format!("{}.{stem}", ir.protocol.id.as_str());

    let evidence = space.evidence(state);
    let authority: Vec<Value> =
        serde_json::from_str(&space.authority(state)).expect("the space writes a JSON array");
    let mut decisions: Vec<Value> =
        serde_json::from_str(&space.decisions(state)).expect("the space writes a JSON array");
    for decision in &mut decisions {
        decision["principal"] = Value::from(PRINCIPAL);
    }
    let supplied = |list: &[Value]| (!list.is_empty()).then(|| Value::from(list).to_string());
    let (authority_text, decisions_text) = (supplied(&authority), supplied(&decisions));

    let revision = Revision::new(REVISION);
    let case = Case {
        format: CASE_FORMAT.to_owned(),
        id: CaseId::new(WITNESS),
        protocol: ir.protocol.id.clone(),
        artifacts: Declarations::new(
            ir.artifacts
                .keys()
                .map(|artifact| {
                    let current = CaseArtifact {
                        revision: revision.clone(),
                    };
                    (artifact.clone(), current)
                })
                .collect(),
        ),
        termination: None,
        revision: Some(revision),
    };
    let decision = eval::evaluate_with(
        ir,
        &case,
        &evidence,
        Supplied {
            authority: authority_text.as_deref(),
            at: None,
            decisions: decisions_text.as_deref(),
        },
    )
    .map_err(|refusal| Refusal::new(refusal.code(), refusal.to_string()))?;
    let expected = eval::render_sections(&decision, &BTreeSet::from([subject.section.to_owned()]));

    let mut text = String::new();
    let mut line = |indent: usize, content: &str| {
        let _ = writeln!(text, "{:indent$}{content}", "");
    };
    line(0, &format!("format: {}", quote(conform::FORMAT)));
    line(0, &format!("id: {}", quote(&id)));
    line(0, "covers: []");
    line(0, &format!("fixture: {}", quote(fixture)));
    line(0, "steps:");
    line(2, &format!("- id: {}", quote(WITNESS)));
    line(4, "evaluate:");
    line(6, "case:");
    line(8, &format!("format: {}", quote(CASE_FORMAT)));
    line(8, &format!("id: {}", quote(WITNESS)));
    line(8, &format!("protocol: {}", quote(ir.protocol.id.as_str())));
    line(8, &format!("revision: {}", quote(REVISION)));
    if ir.artifacts.is_empty() {
        line(8, "artifacts: {}");
    } else {
        line(8, "artifacts:");
        for artifact in ir.artifacts.keys() {
            line(10, &format!("{}:", quote(artifact.as_str())));
            line(12, &format!("revision: {}", quote(REVISION)));
        }
    }
    if evidence.is_empty() {
        line(6, "evidence: []");
    } else {
        line(6, "evidence:");
        for record in &evidence {
            line(8, &format!("- format: {}", quote(EVIDENCE_FORMAT)));
            line(10, &format!("id: {}", quote(record.id.as_str())));
            line(10, &format!("kind: {}", quote(record.kind.as_str())));
            if let Some(result) = &record.result {
                line(10, &format!("result: {}", quote(result)));
            }
            line(10, &format!("subject: {}", quote(record.subject.as_str())));
            line(
                10,
                &format!(
                    "subject_revision: {}",
                    quote(record.subject_revision.as_str())
                ),
            );
        }
    }
    let field = |entry: &Value, key: &str| {
        quote(
            entry[key]
                .as_str()
                .expect("the space writes every field as a string"),
        )
    };
    if !authority.is_empty() {
        line(6, "authority:");
        for entry in &authority {
            line(8, &format!("- capability: {}", field(entry, "capability")));
            line(10, &format!("decision: {}", field(entry, "decision")));
        }
    }
    if !decisions.is_empty() {
        line(6, "decisions:");
        for entry in &decisions {
            line(8, &format!("- decision: {}", field(entry, "decision")));
            line(10, &format!("outcome: {}", field(entry, "outcome")));
            line(10, &format!("principal: {}", field(entry, "principal")));
            line(
                10,
                &format!("case_revision: {}", field(entry, "case_revision")),
            );
        }
    }
    line(4, "expect:");
    line(6, "decision: |");
    for expected_line in expected.lines() {
        line(8, expected_line);
    }

    let unrepresentable = |why: String| {
        Refusal::new(
            "unrepresentable",
            format!(
                "the scenario for {} `{}` does not read back as one that passes: {}",
                subject.kind,
                one_line(subject.id),
                one_line(&why)
            ),
        )
    };
    let parsed = conform::parse(&text).map_err(|error| unrepresentable(error.to_string()))?;
    if let Verdict::Failed { step, reason } = conform::run(&parsed, Ok(source)) {
        return Err(unrepresentable(format!("step `{step}`: {reason}")));
    }
    Ok(Scenario {
        file: subject.file(),
        id,
        text,
    })
}

/// `text` as a YAML double-quoted scalar that reads back as `text`, as the module docs say.
fn quote(text: &str) -> String {
    let mut quoted = String::with_capacity(text.len() + 2);
    quoted.push('"');
    for c in text.chars() {
        match c {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            ' '..='~' => quoted.push(c),
            '\u{85}' | '\u{2028}' | '\u{2029}' | '\u{feff}' | '\u{fffe}' | '\u{ffff}' => {
                let _ = write!(quoted, "\\u{:04X}", u32::from(c));
            }
            c if u32::from(c) >= 0xa0 => quoted.push(c),
            c => {
                let _ = write!(quoted, "\\u{:04X}", u32::from(c));
            }
        }
    }
    quoted.push('"');
    quoted
}
