//! Trigger rules: the `canon-trigger/1` document, one rule saying when rows of a kind in a state
//! start a protocol (story:trigger-rule-document, `docs/design/trigger-rules.md` §§ 3-4). Pure:
//! the caller reads the document and passes its text in; `canon trigger validate` is that caller.
//!
//! # Reading a rule
//!
//! [`read`] reads one document into [`model::TriggerRule`] and checks it field by field. Every
//! error names the field it is about, as a dotted path from the document's root (`owner`,
//! `subject.state.field`, `start.input.invoice`), and carries one of these codes:
//!
//! | Code | Field named |
//! |---|---|
//! | `parse` | none: the text is not well-formed YAML, or not a mapping. |
//! | `missing-field` | a required field left out (`owner`, `subject`, `start.protocol`, `mode`). |
//! | `unknown-field` | a field `canon-trigger/1` does not have (`subject.colour`). |
//! | `invalid-value` | a field whose value the format does not allow: a `mode` other than `offer` or `auto`, an `access` other than `read_only` or `write`, a `refire` other than `once_per_state_entry` or `once_per_subject`, a field `type` other than `string`, `integer`, `decimal` or `bool`, a `version` or `cap` that is not a positive integer, a `format` other than `canon-trigger/1`, an `id`, `subject.kind`, `start.protocol` or field name that is not an identifier, an empty `owner`, a field declared twice. |
//! | `undeclared-field` | `subject.state.field` naming a field `subject.fields` does not declare, or one that is optional or not a `string`; `start.input.<name>` whose `field` is not declared. |
//! | `invalid-input` | `start.input.<name>` written as anything but `{field: <name>}` or `{const: <string>}`, such as an expression; an input name that is not an identifier or is given twice. |
//! | `invalid-where` | `where`, when it does not compile with `canon-expr/1` against the rule's catalog; the message holds `canon-expr/1`'s diagnostic code, message and byte span. |
//!
//! A document without the shape of the format (a field missing or unknown, a value of the wrong
//! kind or outside its enumeration, an input that is not `{field}` or `{const}`) is reported with
//! its first error alone. One with the shape is reported with every error, in the order `format`,
//! `id`, `version`, `owner`, `subject.kind`, `subject.fields`, `subject.state.field`, `where`,
//! `start.protocol`, `start.input` (in input order), `cap`. A valid rule is known by its key,
//! `<id>@<version>` ([`model::TriggerRule::key`]).
//!
//! # The `where` language
//!
//! `where` is one `canon-expr/1` expression of type Bool, compiled unchanged with
//! `canon_expr::check`. Its one context variable is `$row` ([`row_type`]): a `Record` with one entry
//! per declared field, `String`, `Integer`, `Decimal` or `Bool` for the field types `string`,
//! `integer`, `decimal` and `bool`, and `Optional` of that type for an optional field. Its catalog
//! ([`catalog`], format `canon-catalog/1`) holds only `Pure` builtins and no `Observation` provider
//! or recipe, so a rule can call nothing that reaches outside the row:
//!
//! | Name | Signature | Present when |
//! |---|---|---|
//! | `contains` | `contains(list: List<String>, item: String) -> Bool`, `Pure(Contains)` | always |
//! | `present_string` | `present_string(value: Optional<String>) -> Bool`, `Pure(Present)` | an optional field has type `string` |
//! | `present_integer` | `present_integer(value: Optional<Integer>) -> Bool`, `Pure(Present)` | an optional field has type `integer` |
//! | `present_decimal` | `present_decimal(value: Optional<Decimal>) -> Bool`, `Pure(Present)` | an optional field has type `decimal` |
//! | `present_bool` | `present_bool(value: Optional<Bool>) -> Bool`, `Pure(Present)` | an optional field has type `bool` |
//!
//! A `where` that calls another name, reads a field the rule does not declare or another
//! variable, passes an argument of another type, or is not Bool, is refused as `invalid-where`. An
//! optional field is tested with its `present_<type>` function; `canon-expr/1` compares it with
//! nothing else. `$row.<field>` reaches a field whose name is letters, digits and `_`, not starting
//! with a digit. Absent, `where` is true for every row in the state.
//!
//! # The protocols a rule is checked against
//!
//! A rule's `access` is checked against the protocol it starts, never its source
//! (story:protocol-access-class, `docs/design/trigger-rules.md` § 5). Each action of `protocol/1`
//! may declare `access: read_only` or `access: write`, absent meaning `write`; `canon validate`
//! refuses `read_only` on an action that `revises` an artifact or has a `case_open`
//! (`read-only-action-writes`). A protocol's class is `read_only` when every action is, a
//! protocol without actions included, else `write`; it is guarded when one of its actions
//! `requires` a capability, since it cannot run that action until an authority decision grants
//! the capability. `canon-ir/1` carries both at its top level, as `access_class` and `guarded`.
//!
//! `canon trigger protocols --path <file> ...` compiles each `protocol/1` file on its own and
//! prints [`protocols`] of the set as a `canon-trigger-protocols/1` document
//! ([`render_protocols`]), the `protocols` argument a trigger evaluation reads:
//!
//! ```json
//! {
//!   "format": "canon-trigger-protocols/1",
//!   "protocols": {
//!     "send-reminder": {
//!       "class": "write",
//!       "guarded": false,
//!       "inputs": {
//!         "channel": [
//!           "email",
//!           "letter"
//!         ]
//!       }
//!     }
//!   }
//! }
//! ```
//!
//! One entry per protocol, keyed by protocol id; `inputs` holds the case inputs the protocol
//! declares, each with its values, sorted, each once. The files are read in sorted order of their
//! paths, a path that resolves to a file already read is that file once, and the output is the
//! same whatever order the paths are given in. Refused, exit 1, naming the file: a document that
//! does not parse (`parse`), that has imports (`protocol-imports`: a protocol with imports
//! compiles only with the documents it imports) or that does not compile (each problem with its
//! code); and two files that declare one protocol id (`duplicate-protocol-id`, naming the id and
//! every file that declares it), since a rule names a protocol by id alone. The first file refused
//! in sorted order decides; a file that cannot be read exits 2.
//!
//! # Evaluating a rule set
//!
//! [`evaluate`] lists every match of every rule of a set against a snapshot of rows, with the
//! firing id an evaluator would submit it under and a verdict (story:trigger-evaluate,
//! `docs/design/trigger-rules.md` § 6). It is a pure function of the rules with their states, the
//! snapshot, the protocols, the history and the instant `now`: it reads no clock, no network and
//! no file, and the same arguments give the same plan, byte for byte, whatever the order of the
//! rules, kinds, rows, protocols, firings, activations and approvals. It starts nothing; the
//! evaluator that links it
//! does. `canon trigger evaluate --rules <set> --snapshot <file> --protocols <file> --history
//! <file> --now <instant>` reads the documents and prints the plan ([`render_plan`]).
//!
//! The rule set, `canon-trigger-set/1`, lists each `canon-trigger/1` file, relative to the set
//! file's directory or absolute, with its state, `draft`, `active` or `disabled`: a rule's state
//! is deployment state, not part of its document.
//!
//! ```yaml
//! format: canon-trigger-set/1
//! rules:
//!   - {path: overdue-reminder.yaml, state: active}
//!   - {path: overdue-offer.yaml, state: draft}
//! ```
//!
//! The snapshot, `canon-trigger-snapshot/1`, lists the rows of each kind, each with its `id`
//! (unique within its kind), optionally the instant `entered_at` it entered its current state, and
//! its field values; the row id is not a field, so a rule that reads it declares a field the row
//! carries. The protocols are a `canon-trigger-protocols/1` document, as `canon trigger protocols`
//! prints it. The history, `canon-trigger-history/1`, lists the evaluator's `firings` (`firing_id`,
//! `rule`, `at`), `activations` (`rule`, `at`, and optionally `baseline`, rows written
//! `<kind>:<row id>`) and `approvals` (`rule`, `by`, `at`), each list empty when left out. An
//! approval counts for the exact rule key it names, never for another version of the rule; Canon
//! reads `by` and never decides who may approve. Every instant (`now`, `entered_at`, `at`) is a
//! Canon instant, `YYYY-MM-DDTHH:MM:SSZ` in UTC with an optional fraction.
//!
//! ```yaml
//! format: canon-trigger-history/1
//! firings:
//!   - {firing_id: overdue-reminder@1/invoice:inv-9/2026-11-20T08:00:00Z, rule: overdue-reminder@1, at: 2026-11-26T08:00:00Z}
//! activations:
//!   - {rule: overdue-reminder@1, at: 2026-11-19T00:00:00Z}
//! approvals:
//!   - {rule: overdue-reminder@1, by: billing-lead, at: 2026-11-19T00:00:00Z}
//! ```
//!
//! ```yaml
//! format: canon-trigger-snapshot/1
//! kinds:
//!   invoice:
//!     - id: inv-1
//!       entered_at: 2026-11-20T09:00:00Z
//!       fields: {id: inv-1, status: overdue, amount: 250, segment: retail}
//! ```
//!
//! A row of the rule's `subject.kind` is decided in this order:
//!
//! 1. Its state field absent or null is `undetermined` with the reason `field-missing`, a value
//!    that is not a string `field-type`; a string other than `is` is no match, and the row is not
//!    listed.
//! 2. In the state, each declared field in declaration order: a required one absent or null is
//!    `undetermined`, `field-missing`; a value of another type than declared is `undetermined`,
//!    `field-type`. A value has its type when it is a string for `string`, a number with no
//!    fraction or exponent for `integer`, any number for `decimal`, and `true` or `false` for
//!    `bool`; an optional field may be absent or null. The entry names the field. `where` is not
//!    evaluated.
//! 3. `where`, with `$row` bound to the declared fields: true is a match, false no match, and
//!    anything else (`UNKNOWN`, or a value over `canon-expr/1`'s bounds) `undetermined` with the
//!    reason `where-unknown`. Absent, `where` is true.
//!
//! Each match, and each undetermined row, is one entry of the plan, `canon-trigger-plan/1`:
//!
//! ```json
//! {
//!   "entries": [
//!     {
//!       "firing_id": "overdue-reminder@1/invoice:inv-1/2026-11-20T09:00:00Z",
//!       "input": {
//!         "channel": "email",
//!         "invoice": "inv-1"
//!       },
//!       "kind": "invoice",
//!       "protocol": "send-reminder",
//!       "row": "inv-1",
//!       "rule": "overdue-reminder@1",
//!       "rule_state": "active",
//!       "verdict": "would-start"
//!     }
//!   ],
//!   "format": "canon-trigger-plan/1"
//! }
//! ```
//!
//! - `rule` is the rule key and `rule_state` the rule's state; `kind`, `row` and `protocol` are
//!   the subject kind, the row id and `start.protocol`.
//! - `input` holds each `start.input`: a `{field}` input is the row's value as read (null when an
//!   optional field is absent), a `{const}` input its string. The value of an input the protocol
//!   declares is checked against the input's values (`input-value-not-allowed` below); an input
//!   the protocol does not declare is not checked here, but when the set is published.
//! - `firing_id` is `<rule key>/<kind>:<row id>/<entered_at>` under `once_per_state_entry`, the
//!   instant as the snapshot writes it, so a row that enters its state again gets a new one, and
//!   `<rule key>/<kind>:<row id>` under `once_per_subject`. A row without `entered_at` under
//!   `once_per_state_entry` has none.
//! - `verdict` is the first that applies, and `reason` says why for `refused`, `undetermined` and
//!   `held`:
//!   1. `refused`, the first reason that applies: the rule's state is `disabled` (`disabled`); its
//!      protocol is not in the protocols (`unknown-protocol`); the protocol is guarded and the
//!      rule's mode `auto` (`guarded-auto`); the rule's `access` is not the protocol's class
//!      (`access-mismatch`); the `access` is `write`, the mode `auto` and the history holds no
//!      approval of the rule key (`write-unapproved`); a row without `entered_at` under
//!      `once_per_state_entry` (`entered-at-missing`); the value of an input the protocol
//!      declares, the row's value of a `{field}` input or a `{const}` input's string, not among
//!      the input's values (`input-value-not-allowed`): a value that is absent, null or not a
//!      string is not among them.
//!   2. `undetermined` (with its `reason`, and the `field` it names).
//!   3. `baseline` when the latest activation of the rule key in the history is at or after the
//!      row's `entered_at` (each instant read as the start of the interval it names) or an
//!      activation of it lists the row, listed and never started.
//!   4. `held` when a firing with the entry's firing id is in the history (`already-fired`), or
//!      when the rule key's firings on the UTC day of `now` have reached the rule's `cap`
//!      (`cap-reached`): the distinct firing ids of the rule key's firings whose `at` falls on that
//!      day, whether before or after `now`, then each earlier `would-start` entry of the rule key
//!      in plan order. An entry held at the cap or offered does not count; a `would-start` entry
//!      does.
//!   5. `offer` for a rule in mode `offer`; `would-start` for one in mode `auto`.
//!
//! Every state is evaluated by the same steps, so a `draft` rule is simulated against real state;
//! an evaluator submits only `would-start` entries of an `active` rule, with the firing id as the
//! idempotency key, and records the firing: evaluating again with it in the history lists the
//! match `held`, `already-fired`. The entries are ordered by rule key, kind and row id, each by
//! code point.
//!
//! The evaluation is refused as a whole, exit 1 with one line `error[<code>]: <why>`, the first in
//! this order deciding: `now` that is not an instant (`invalid-instant`); a document of another
//! format (`malformed-input`); a rule that is not valid (`invalid-rule`); two rules of one rule
//! key (`duplicate-rule`); a kind listed twice or a row id listed twice within its kind
//! (`duplicate-row`), a kind or row id with whitespace (`malformed-input`), or an `entered_at`
//! that is not an instant (`invalid-instant`); a protocol id listed twice in the protocols
//! (`duplicate-protocol-id`); a firing's, an activation's or an approval's `at` that is not an
//! instant (`invalid-instant`). `canon trigger evaluate` reads the set, then each rule file in sorted order of its path, then the snapshot, the
//! protocols and the history: a file that cannot be read exits 2, a document that does not read as
//! its format is `malformed-input` naming the file, and a rule file that is not a valid rule
//! prints its errors as `canon trigger validate` does, each line naming the file.
//!
//! # Publishing a rule set
//!
//! A published rule is immutable: a change is a new version (story:trigger-rule-publication,
//! `docs/design/trigger-rules.md` § 7). [`check_set`] checks a set of rules before it is
//! published, against the index of the rule keys already published and the protocols; `canon
//! trigger check-set --path <rule file> ... --published <index> --protocols <file>` reads the
//! files and prints its result. Canon stores nothing: the index is the caller's, who adds the
//! set's own index to it once the set is published.
//!
//! The digest of a rule ([`digest`], `canon trigger digest --path <rule file>`) is `sha256:` and
//! the SHA-256, in lowercase hexadecimal, of the rule's canonical form ([`canonical_form`]): the
//! JSON Canonicalization Scheme (RFC 8785) of one object holding every field of the rule as read,
//! `version` and `cap` as decimal integers, `where` left out when the rule has none,
//! `subject.fields` as a list of `{name, optional, type}` in declaration order (the order decides
//! which field an undetermined row names), `optional` written `false` and `start.input` written
//! `{}` when the document leaves them out. So the layout, the key order, the comments and a field
//! written with its default value change nothing, and every other change of the rule changes the
//! digest. Only a valid rule has one.
//!
//! The index, `canon-trigger-index/1`, maps each published rule key to the digest of its
//! document:
//!
//! ```yaml
//! format: canon-trigger-index/1
//! rules:
//!   stale@1: sha256:2785572303ce15cfe7a3c5f58c846746fc1466b4c78b3980edde1944335c62af
//! ```
//!
//! The rules are grouped by rule key. A key that two or more documents of the set share fails
//! `duplicate-key`, naming each document, and is not checked further. Each other rule is checked
//! in this order, every failure reported:
//!
//! 1. `key-republished`: the index holds its key with another digest. The same document, or the
//!    same rule written another way, against the index that holds it passes; two versions of one
//!    rule id are two keys.
//! 2. `unknown-protocol`: its protocol is not in the protocols; nothing below is checked.
//! 3. `guarded-auto`: the protocol is guarded and the rule's mode is `auto`, which
//!    [`evaluate`] would refuse at every match.
//! 4. `unknown-input`: a `start.input` name the protocol does not declare, each in input-id order.
//! 5. `missing-input`: a case input of the protocol that `start.input` does not map, each in
//!    input-id order; every case input is required when a case opens.
//! 6. `input-value-not-allowed`: a mapped input whose `{const}` is not among the input's values,
//!    or whose `{field}` is not a declared `string` field, each in input-id order. A `{field}`
//!    input of a `string` field passes here; its row value is checked by [`evaluate`].
//!
//! A set without a failure passes, and the output is its own `canon-trigger-index/1` index
//! ([`render_index`]), each rule key with its digest, in key order:
//!
//! ```json
//! {
//!   "format": "canon-trigger-index/1",
//!   "rules": {
//!     "stale@1": "sha256:2785572303ce15cfe7a3c5f58c846746fc1466b4c78b3980edde1944335c62af"
//!   }
//! }
//! ```
//!
//! Otherwise each failure is one line `error[<code>]: <rule key>: <why>`, exit 1, in rule-key
//! order and within a key in the order above. The order of the rules changes no byte of either.
//! Refused as a whole, one line `error[<code>]: <why>`, the first in this order deciding: an index
//! or protocols document of another format (`malformed-input`), a rule key listed twice in the
//! index (`malformed-input`), a protocol id listed twice in the protocols
//! (`duplicate-protocol-id`). `canon trigger check-set` reads each rule file in sorted order of its
//! path, a path that resolves to a file already read being that file once, then the index and the
//! protocols: a file that cannot be read exits 2, a rule file that is not a valid rule prints its
//! errors as `canon trigger validate` does, each line naming the file, and an index or protocols
//! document that does not read as its format is `malformed-input` naming the file.

mod evaluate;
mod publication;

pub use evaluate::{
    Refusal, RuleEntry, evaluate, read_history, read_protocols, read_set, read_snapshot,
    render_plan,
};
pub use publication::{Failure, canonical_form, check_set, digest, read_index, render_index};

use std::collections::BTreeMap;
use std::fmt;

use canon_expr::{Builtin, Catalog, CheckedExpr, Diagnostic, Function, Implementation, Parameter};

use crate::ir::{self, json::Value};
use crate::model::{
    self, Declarations, ProtocolId, TRIGGER_FORMAT, TRIGGER_PROTOCOLS_FORMAT, TriggerFieldType,
    TriggerInput, TriggerProtocol, TriggerProtocols, TriggerRule,
};

/// The name `$row` has in a rule's `where`.
pub const ROW: &str = "row";

/// One error of a `canon-trigger/1` document: its code, the field it names (empty for `parse`),
/// and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    pub code: &'static str,
    pub field: String,
    pub message: String,
}

impl Problem {
    fn new(code: &'static str, field: impl Into<String>, message: impl Into<String>) -> Self {
        Problem {
            code,
            field: field.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for Problem {
    /// `<field>: <why>`, or `<why>` alone when the error names no field.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.field.is_empty() {
            f.write_str(&self.message)
        } else {
            write!(f, "{}: {}", self.field, self.message)
        }
    }
}

/// Reads one `canon-trigger/1` document and checks it field by field (see the module docs): the
/// rule, or every error found, each naming its field.
pub fn read(source: &str) -> Result<TriggerRule, Vec<Problem>> {
    let deserializer = serde_yaml_ng::Deserializer::from_str(source);
    let rule: TriggerRule = match serde_path_to_error::deserialize(deserializer) {
        Ok(rule) => rule,
        Err(error) => return Err(vec![shape_problem(source, &error)]),
    };
    let problems = check(&rule);
    if problems.is_empty() {
        Ok(rule)
    } else {
        Err(problems)
    }
}

/// The one error of a document serde could not read as a `canon-trigger/1` rule.
fn shape_problem(
    source: &str,
    error: &serde_path_to_error::Error<serde_yaml_ng::Error>,
) -> Problem {
    let inner = error.inner();
    let at = |what: &str| match inner.location() {
        Some(at) => format!("{what} at line {} column {}", at.line(), at.column()),
        None => what.to_owned(),
    };
    if serde_yaml_ng::from_str::<serde_yaml_ng::Value>(source).is_err() {
        return Problem::new("parse", "", at("not well-formed YAML"));
    }
    let mut path = segments(error.path());
    let mut message = bare_message(inner, &path.join("."));
    // A key of `subject.fields` or `start.input` is an identifier, which serde reads through an
    // option, and the path tracker does not see a key read so: it writes `?`. The entry is found
    // again by reading each entry of that map on its own.
    if let Some(at) = path.iter().position(|segment| segment == UNKNOWN)
        && let Some((found, why)) = failing_entry(source, &path[..at])
    {
        path = found;
        message = why;
    }
    if path.len() >= 3 && path[0] == "start" && path[1] == "input" {
        return Problem::new("invalid-input", path[..3].join("."), message);
    }
    if let Some(missing) = quoted_after(&message, "missing field `") {
        let mut field = path;
        field.push(missing);
        return Problem::new("missing-field", field.join("."), "this field is required");
    }
    if message.contains("unknown field `") && !path.is_empty() {
        return Problem::new(
            "unknown-field",
            path.join("."),
            "canon-trigger/1 has no such field",
        );
    }
    if path.is_empty() {
        return Problem::new(
            "parse",
            "",
            format!("not a canon-trigger/1 document: {message}"),
        );
    }
    Problem::new("invalid-value", path.join("."), message)
}

/// How a path segment the tracker could not name is written.
const UNKNOWN: &str = "?";

/// A tracked path as field names, each shown on one line.
fn segments(path: &serde_path_to_error::Path) -> Vec<String> {
    path.iter()
        .map(|segment| match segment {
            serde_path_to_error::Segment::Map { key } => model::one_line(key),
            serde_path_to_error::Segment::Seq { index } => format!("[{index}]"),
            serde_path_to_error::Segment::Enum { variant } => model::one_line(variant),
            serde_path_to_error::Segment::Unknown => UNKNOWN.to_owned(),
        })
        .collect()
}

/// The first entry of the map at `map` (`subject.fields` or `start.input`) that does not read as
/// its value type, with the path within it and why; `None` for another map, or when every entry
/// reads.
fn failing_entry(source: &str, map: &[String]) -> Option<(Vec<String>, String)> {
    let document: serde_yaml_ng::Value = serde_yaml_ng::from_str(source).ok()?;
    let mut at = &document;
    for key in map {
        at = at.get(key.as_str())?;
    }
    let serde_yaml_ng::Value::Mapping(entries) = at else {
        return None;
    };
    let field = [String::from("subject"), String::from("fields")];
    let input = [String::from("start"), String::from("input")];
    for (key, value) in entries {
        let mut path = map.to_vec();
        let Some(key) = key.as_str() else {
            path.push(UNKNOWN.to_owned());
            return Some((path, "a key is not a string".to_owned()));
        };
        let read = if map == field {
            entry::<model::TriggerField>(value)
        } else if map == input {
            entry::<TriggerInput>(value)
        } else {
            return None;
        };
        if let Err((within, why)) = read {
            path.push(model::one_line(key));
            path.extend(within);
            return Some((path, why));
        }
    }
    None
}

/// Reads one map entry's value as `T`: the path within it and why, when it does not read.
fn entry<T: serde::de::DeserializeOwned>(
    value: &serde_yaml_ng::Value,
) -> Result<(), (Vec<String>, String)> {
    // Read on its own, a null reads as an empty map; in the document it is a value left out.
    if value.is_null() {
        return Err((
            Vec::new(),
            "this key is required and needs a value; an explicit null is not one".to_owned(),
        ));
    }
    serde_path_to_error::deserialize::<_, T>(value.clone())
        .map(|_| ())
        .map_err(|error| {
            let within = segments(error.path());
            let why = bare_message(error.inner(), &within.join("."));
            (within, why)
        })
}

/// The deserializer's message without the path it writes first and the position it writes last,
/// both of which the problem carries otherwise.
fn bare_message(error: &serde_yaml_ng::Error, path: &str) -> String {
    let mut text = error.to_string();
    if let Some(at) = error.location() {
        let suffix = format!(" at line {} column {}", at.line(), at.column());
        if let Some(stripped) = text.strip_suffix(&suffix) {
            text = stripped.to_owned();
        }
    }
    if !path.is_empty()
        && let Some(stripped) = text.strip_prefix(&format!("{path}: "))
    {
        text = stripped.to_owned();
    }
    model::one_line(&text)
}

/// The text between `opening` and the next backtick, as serde writes a field name.
fn quoted_after(message: &str, opening: &str) -> Option<String> {
    let start = message.find(opening)? + opening.len();
    let end = message[start..].find('`')?;
    Some(message[start..start + end].to_owned())
}

/// The checks of a document that has the format's shape, in the order the module docs give.
fn check(rule: &TriggerRule) -> Vec<Problem> {
    let mut problems = Vec::new();
    if rule.format != TRIGGER_FORMAT {
        problems.push(Problem::new(
            "invalid-value",
            "format",
            format!(
                "`{}` is not `{TRIGGER_FORMAT}`",
                model::one_line(&rule.format)
            ),
        ));
    }
    if !model::is_identifier(rule.id.as_str()) {
        problems.push(not_identifier("id", rule.id.as_str()));
    }
    if rule.version == 0 {
        problems.push(Problem::new(
            "invalid-value",
            "version",
            "the version is a positive integer, not 0",
        ));
    }
    if rule.owner.trim().is_empty() {
        problems.push(Problem::new("invalid-value", "owner", "the owner is empty"));
    }
    let subject = &rule.subject;
    if !model::is_identifier(subject.kind.as_str()) {
        problems.push(not_identifier("subject.kind", subject.kind.as_str()));
    }
    for (index, (name, _)) in subject.fields.iter().enumerate() {
        let field = format!("subject.fields.{}", model::one_line(name.as_str()));
        if !model::is_identifier(name.as_str()) {
            problems.push(not_identifier(&field, name.as_str()));
        } else if subject
            .fields
            .ids()
            .take(index)
            .any(|earlier| earlier == name)
        {
            problems.push(Problem::new(
                "invalid-value",
                field,
                "the field is declared more than once",
            ));
        }
    }
    let state = &subject.state;
    match subject.fields.get(&state.field) {
        None => problems.push(Problem::new(
            "undeclared-field",
            "subject.state.field",
            format!(
                "`{}` is not a field of subject.fields",
                model::one_line(state.field.as_str())
            ),
        )),
        Some(declared) if declared.optional || declared.r#type != TriggerFieldType::String => {
            problems.push(Problem::new(
                "undeclared-field",
                "subject.state.field",
                format!(
                    "`{}` is declared {}{}; the state field is a required string",
                    model::one_line(state.field.as_str()),
                    if declared.optional { "optional " } else { "" },
                    declared.r#type.as_str()
                ),
            ));
        }
        Some(_) => {}
    }
    if let Err(diagnostic) = compile_where(rule) {
        problems.push(Problem::new(
            "invalid-where",
            "where",
            format!(
                "{}: {} (bytes {}..{})",
                diagnostic.code,
                model::one_line(&diagnostic.message),
                diagnostic.start,
                diagnostic.end
            ),
        ));
    }
    let start = &rule.start;
    if !model::is_identifier(start.protocol.as_str()) {
        problems.push(not_identifier("start.protocol", start.protocol.as_str()));
    }
    for (index, (name, input)) in start.input.iter().enumerate() {
        let field = format!("start.input.{}", model::one_line(name.as_str()));
        if !model::is_identifier(name.as_str()) {
            problems.push(Problem::new(
                "invalid-input",
                field,
                "the input name is not an identifier",
            ));
            continue;
        }
        if start.input.ids().take(index).any(|earlier| earlier == name) {
            problems.push(Problem::new(
                "invalid-input",
                field,
                "the input is given more than once",
            ));
            continue;
        }
        if let TriggerInput::Field(row_field) = input
            && !subject.fields.contains(row_field)
        {
            problems.push(Problem::new(
                "undeclared-field",
                field,
                format!(
                    "`{}` is not a field of subject.fields",
                    model::one_line(row_field.as_str())
                ),
            ));
        }
    }
    if rule.cap == 0 {
        problems.push(Problem::new(
            "invalid-value",
            "cap",
            "the cap is a positive integer, not 0",
        ));
    }
    problems
}

fn not_identifier(field: &str, value: &str) -> Problem {
    Problem::new(
        "invalid-value",
        field,
        format!("`{}` is not an identifier", model::one_line(value)),
    )
}

/// The `canon-expr/1` type of a declared field's value in `$row`, before `Optional`.
fn scalar(field_type: TriggerFieldType) -> canon_expr::Type {
    match field_type {
        TriggerFieldType::String => canon_expr::Type::String,
        TriggerFieldType::Integer => canon_expr::Type::Integer,
        TriggerFieldType::Decimal => canon_expr::Type::Decimal,
        TriggerFieldType::Bool => canon_expr::Type::Bool,
    }
}

/// The type of `$row` in a rule's `where`: a `Record` of the declared fields, an optional one
/// `Optional` of its type. A field declared twice counts once, as first declared.
pub fn row_type(rule: &TriggerRule) -> canon_expr::Type {
    let mut fields = BTreeMap::new();
    for (name, field) in rule.subject.fields.iter() {
        let ty = scalar(field.r#type);
        let ty = if field.optional {
            canon_expr::Type::Optional(Box::new(ty))
        } else {
            ty
        };
        fields.entry(name.as_str().to_owned()).or_insert(ty);
    }
    canon_expr::Type::Record(fields)
}

/// The catalog a rule's `where` is compiled against: `contains` over strings, and
/// `present_<type>` for the type of each optional declared field (see the module docs).
pub fn catalog(rule: &TriggerRule) -> Catalog {
    let parameter = |name: &str, value_type: canon_expr::Type| Parameter {
        name: name.to_owned(),
        value_type,
        default: None,
    };
    let mut functions = BTreeMap::new();
    functions.insert(
        "contains".to_owned(),
        Function {
            parameters: vec![
                parameter(
                    "list",
                    canon_expr::Type::List(Box::new(canon_expr::Type::String)),
                ),
                parameter("item", canon_expr::Type::String),
            ],
            returns: canon_expr::Type::Bool,
            implementation: Implementation::Pure(Builtin::Contains),
        },
    );
    for (_, field) in rule
        .subject
        .fields
        .iter()
        .filter(|(_, field)| field.optional)
    {
        functions.insert(
            format!("present_{}", field.r#type.as_str()),
            Function {
                parameters: vec![parameter(
                    "value",
                    canon_expr::Type::Optional(Box::new(scalar(field.r#type))),
                )],
                returns: canon_expr::Type::Bool,
                implementation: Implementation::Pure(Builtin::Present),
            },
        );
    }
    Catalog {
        format: "canon-catalog/1".to_owned(),
        functions,
    }
}

/// Compiles a rule's `where` with `canon-expr/1` against its [`catalog`], `$row` typed by
/// [`row_type`]: the checked expression, `None` when the rule has no `where`, or
/// `canon-expr/1`'s diagnostic.
pub fn compile_where(rule: &TriggerRule) -> Result<Option<CheckedExpr>, Diagnostic> {
    let Some(source) = &rule.r#where else {
        return Ok(None);
    };
    let parsed = canon_expr::parse(source)?;
    let context = BTreeMap::from([(ROW.to_owned(), row_type(rule))]);
    canon_expr::check(parsed, &catalog(rule), &context).map(Some)
}

/// What a trigger rule is checked against in one compiled protocol: its access class, whether it
/// is guarded, and its case inputs with their values, sorted, each once
/// (story:protocol-access-class).
pub fn protocol(compiled: &ir::Ir) -> TriggerProtocol {
    TriggerProtocol {
        class: compiled.access_class(),
        guarded: compiled.guarded(),
        inputs: Declarations::new(
            compiled
                .inputs
                .iter()
                .map(|(id, input)| (id.clone(), input.values.clone()))
                .collect(),
        ),
    }
}

/// A set of protocols that holds two of one protocol id: `canon-trigger-protocols/1` is keyed by
/// protocol id, so the set is refused (`duplicate-protocol-id`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateProtocolId {
    /// The protocol id.
    pub id: ProtocolId,
    /// The names of every member that declares it, sorted.
    pub sources: Vec<String>,
}

impl DuplicateProtocolId {
    /// The stable code of the refusal.
    pub const CODE: &'static str = "duplicate-protocol-id";
}

impl fmt::Display for DuplicateProtocolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sources: Vec<String> = self
            .sources
            .iter()
            .map(|source| model::one_line(source))
            .collect();
        write!(
            f,
            "protocol `{}` is declared by {}",
            model::one_line(self.id.as_str()),
            sources.join(" and ")
        )
    }
}

/// The `canon-trigger-protocols/1` document of a set of compiled protocols, each named by its
/// source: one entry per protocol, keyed by protocol id, in id order whatever order the set is
/// given in. A set with two members of one id is refused, the least such id named.
pub fn protocols<'a>(
    set: impl IntoIterator<Item = (&'a str, &'a ir::Ir)>,
) -> Result<TriggerProtocols, DuplicateProtocolId> {
    let mut by_id: BTreeMap<&ProtocolId, Vec<(&str, &ir::Ir)>> = BTreeMap::new();
    for (source, compiled) in set {
        by_id
            .entry(&compiled.protocol.id)
            .or_default()
            .push((source, compiled));
    }
    let mut entries = Vec::with_capacity(by_id.len());
    for (id, members) in by_id {
        if let [(_, compiled)] = members.as_slice() {
            entries.push((id.clone(), protocol(compiled)));
        } else {
            let mut sources: Vec<String> = members
                .iter()
                .map(|(source, _)| (*source).to_owned())
                .collect();
            sources.sort();
            return Err(DuplicateProtocolId {
                id: id.clone(),
                sources,
            });
        }
    }
    Ok(TriggerProtocols {
        format: TRIGGER_PROTOCOLS_FORMAT.to_owned(),
        protocols: Declarations::new(entries),
    })
}

/// The canonical JSON of a `canon-trigger-protocols/1` document, as `canon-ir/1` is written: keys
/// in code-point order, two-space indent, one trailing newline. An entry repeated in
/// `protocols` or `inputs` is written once, as first given.
pub fn render_protocols(document: &TriggerProtocols) -> String {
    let entries = document.protocols.iter().map(|(id, entry)| {
        let inputs = entry.inputs.iter().map(|(input, values)| {
            let values = values
                .iter()
                .map(|value| Value::string(value.as_str()))
                .collect();
            (input.as_str().to_owned(), Value::Array(values))
        });
        let entry = Value::object([
            ("class", Value::string(entry.class.as_str())),
            ("guarded", Value::Bool(entry.guarded)),
            ("inputs", first_of_each(inputs)),
        ]);
        (id.as_str().to_owned(), entry)
    });
    Value::object([
        ("format", Value::string(document.format.as_str())),
        ("protocols", first_of_each(entries)),
    ])
    .render()
}

/// An object of the entries, a key given twice keeping its first value.
fn first_of_each(entries: impl Iterator<Item = (String, Value)>) -> Value {
    let mut object = BTreeMap::new();
    for (key, value) in entries {
        object.entry(key).or_insert(value);
    }
    Value::Object(object)
}

#[cfg(test)]
mod tests;
