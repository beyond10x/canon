//! Checking a rule set before it is published (story:trigger-rule-publication,
//! `docs/design/trigger-rules.md` § 7): the digest of a rule ([`digest`]), the
//! `canon-trigger-index/1` index ([`read_index`], [`render_index`]) and [`check_set`]. What it
//! decides is written in the module docs of `crate::trigger`, which the reference page is built
//! from.

use std::collections::BTreeMap;
use std::fmt;

use super::evaluate::{Refusal, expect_format, protocols_by_id, read};
use super::{check, first_of_each};
use crate::ir::json::Value;
use crate::model::{
    self, Declarations, TRIGGER_INDEX_FORMAT, TRIGGER_PROTOCOLS_FORMAT, TriggerFieldType,
    TriggerIndex, TriggerInput, TriggerMode, TriggerProtocols, TriggerRule,
};

/// The canonical form of a rule, the bytes its [`digest`] is taken of: the JSON Canonicalization
/// Scheme (RFC 8785) of one object holding every field of the rule as read. `where` is left out
/// when the rule has none, and written as its text otherwise; `version` and `cap` are numbers;
/// `subject.fields` is a list of `{name, optional, type}` in declaration order, `optional` written
/// `false` when the document leaves it out, since the order of the declared fields decides which
/// one an undetermined row names; `start.input` is an object of `{field}` or `{const}`, `{}` when
/// the document leaves it out. Keys are sorted by their UTF-16 code units and every string is
/// written as RFC 8785 writes it. The rule is destructured without `..`, so a field added to it
/// does not compile until this reads it.
pub fn canonical_form(rule: &TriggerRule) -> String {
    let TriggerRule {
        format,
        id,
        version,
        owner,
        subject,
        r#where,
        start,
        mode,
        access,
        refire,
        cap,
    } = rule;
    let model::TriggerSubject {
        kind,
        fields,
        state,
    } = subject;
    let model::TriggerStart { protocol, input } = start;
    let fields = fields
        .iter()
        .map(|(name, field)| {
            object(vec![
                ("name", string(name.as_str())),
                ("optional", field.optional.to_string()),
                ("type", string(field.r#type.as_str())),
            ])
        })
        .collect::<Vec<_>>();
    let inputs = input
        .iter()
        .map(|(name, input)| {
            let value = match input {
                TriggerInput::Field(field) => object(vec![("field", string(field.as_str()))]),
                TriggerInput::Const(value) => object(vec![("const", string(value))]),
            };
            (name.as_str(), value)
        })
        .collect();
    let mut members = vec![
        ("access", string(access_text(*access))),
        ("cap", cap.to_string()),
        ("format", string(format)),
        ("id", string(id.as_str())),
        ("mode", string(mode_text(*mode))),
        ("owner", string(owner)),
        ("refire", string(refire_text(*refire))),
        (
            "start",
            object(vec![
                ("input", object(inputs)),
                ("protocol", string(protocol.as_str())),
            ]),
        ),
        (
            "subject",
            object(vec![
                ("fields", format!("[{}]", fields.join(","))),
                ("kind", string(kind.as_str())),
                (
                    "state",
                    object(vec![
                        ("field", string(state.field.as_str())),
                        ("is", string(&state.is)),
                    ]),
                ),
            ]),
        ),
        ("version", version.to_string()),
    ];
    if let Some(condition) = r#where {
        members.push(("where", string(condition)));
    }
    object(members)
}

fn access_text(access: model::TriggerAccess) -> &'static str {
    match access {
        model::TriggerAccess::ReadOnly => "read_only",
        model::TriggerAccess::Write => "write",
    }
}

fn mode_text(mode: TriggerMode) -> &'static str {
    match mode {
        TriggerMode::Offer => "offer",
        TriggerMode::Auto => "auto",
    }
}

fn refire_text(refire: model::TriggerRefire) -> &'static str {
    match refire {
        model::TriggerRefire::OncePerStateEntry => "once_per_state_entry",
        model::TriggerRefire::OncePerSubject => "once_per_subject",
    }
}

/// A string as RFC 8785 writes it, which is how `serde_json` writes one.
fn string(text: &str) -> String {
    serde_json::to_string(text).expect("a string is written as JSON")
}

/// An RFC 8785 object of members already written, keys sorted by their UTF-16 code units. A key
/// given twice is written once, as first given.
fn object(mut members: Vec<(&str, String)>) -> String {
    members.sort_by(|(left, _), (right, _)| left.encode_utf16().cmp(right.encode_utf16()));
    members.dedup_by(|later, earlier| later.0 == earlier.0);
    let members: Vec<String> = members
        .into_iter()
        .map(|(key, value)| format!("{}:{value}", string(key)))
        .collect();
    format!("{{{}}}", members.join(","))
}

/// The digest of a rule, the value a `canon-trigger-index/1` index holds for its key: `sha256:`
/// and the SHA-256, in lowercase hexadecimal, of its [`canonical_form`]. The layout, the key
/// order, the comments and a field written with its default value change nothing; every other
/// change of the rule changes it.
pub fn digest(rule: &TriggerRule) -> String {
    crate::ir::digest_of(&canonical_form(rule))
}

/// Reads a `canon-trigger-index/1` document.
pub fn read_index(source: &str) -> Result<TriggerIndex, Refusal> {
    read(source, TRIGGER_INDEX_FORMAT, |index: &TriggerIndex| {
        &index.format
    })
}

/// The canonical JSON of a `canon-trigger-index/1` document, as `canon-ir/1` is written: keys in
/// code-point order, two-space indent, one trailing newline. A key given twice is written once,
/// as first given.
pub fn render_index(index: &TriggerIndex) -> String {
    let rules = index
        .rules
        .iter()
        .map(|(key, digest)| (key.clone(), Value::string(digest.as_str())));
    Value::object([
        ("format", Value::string(index.format.as_str())),
        ("rules", first_of_each(rules)),
    ])
    .render()
}

/// One reason a rule set is not published: its code, the rule key it names (none for a refusal of
/// the whole input) and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    pub code: &'static str,
    pub rule: Option<String>,
    pub message: String,
}

impl Failure {
    fn of(code: &'static str, rule: &str, message: impl Into<String>) -> Self {
        Failure {
            code,
            rule: Some(rule.to_owned()),
            message: message.into(),
        }
    }

    fn whole(refusal: Refusal) -> Self {
        Failure {
            code: refusal.code,
            rule: None,
            message: refusal.message,
        }
    }
}

impl fmt::Display for Failure {
    /// `<rule key>: <why>`, or `<why>` alone for a refusal of the whole input.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.rule {
            Some(rule) => write!(f, "{}: {}", model::one_line(rule), self.message),
            None => f.write_str(&self.message),
        }
    }
}

/// Checks a rule set before it is published (see the module docs of `crate::trigger`): each rule
/// named by its source, the published index and the protocols. A set that passes gives its own
/// `canon-trigger-index/1` index, each rule key with its [`digest`], in key order. Otherwise every
/// failure, in rule-key order and, within a key, in the order of the checks; the order of `rules`
/// changes nothing but the sources a `duplicate-key` failure names, which are sorted.
///
/// Refused as a whole, one failure without a rule key, the first in this order deciding: the index
/// or the protocols of another format (`malformed-input`), a rule key listed twice in the index
/// (`malformed-input`), a protocol id listed twice in the protocols (`duplicate-protocol-id`), a
/// rule that is not a valid `canon-trigger/1` rule (`invalid-rule`), in rule-key order.
pub fn check_set(
    rules: &[(String, TriggerRule)],
    published: &TriggerIndex,
    protocols: &TriggerProtocols,
) -> Result<TriggerIndex, Vec<Failure>> {
    let whole = |refusal: Refusal| vec![Failure::whole(refusal)];
    expect_format("published", &published.format, TRIGGER_INDEX_FORMAT).map_err(whole)?;
    expect_format("protocols", &protocols.format, TRIGGER_PROTOCOLS_FORMAT).map_err(whole)?;
    let mut keys: Vec<&String> = published.rules.ids().collect();
    keys.sort();
    if let Some(pair) = keys.windows(2).find(|pair| pair[0] == pair[1]) {
        return Err(vec![Failure {
            code: "malformed-input",
            rule: None,
            message: format!(
                "published: rule `{}` is listed twice",
                model::one_line(pair[0])
            ),
        }]);
    }
    let by_id = protocols_by_id(protocols).map_err(whole)?;
    let mut by_key: BTreeMap<String, Vec<(&str, &TriggerRule)>> = BTreeMap::new();
    for (source, rule) in rules {
        by_key
            .entry(rule.key())
            .or_default()
            .push((source.as_str(), rule));
    }
    for (key, documents) in &by_key {
        for (_, rule) in documents {
            if let Some(problem) = check(rule).first() {
                return Err(vec![Failure {
                    code: "invalid-rule",
                    rule: None,
                    message: format!(
                        "rule `{}`: error[{}]: {problem}",
                        model::one_line(key),
                        problem.code
                    ),
                }]);
            }
        }
    }

    let mut failures = Vec::new();
    let mut index = Vec::with_capacity(by_key.len());
    for (key, documents) in &by_key {
        if let [(_, rule)] = documents.as_slice() {
            let before = failures.len();
            let digest = digest(rule);
            check_rule(key, rule, &digest, published, &by_id, &mut failures);
            if failures.len() == before {
                index.push((key.clone(), digest));
            }
        } else {
            let mut sources: Vec<String> = documents
                .iter()
                .map(|(source, _)| model::one_line(source))
                .collect();
            sources.sort();
            failures.push(Failure::of(
                "duplicate-key",
                key,
                format!(
                    "the rule key is given by {} documents: {}",
                    sources.len(),
                    sources.join(", ")
                ),
            ));
        }
    }
    if failures.is_empty() {
        Ok(TriggerIndex {
            format: TRIGGER_INDEX_FORMAT.to_owned(),
            rules: Declarations::new(index),
        })
    } else {
        Err(failures)
    }
}

/// The failures of one rule, the only document of its key, in this order: `key-republished`;
/// `unknown-protocol`, after which nothing else is checked; `guarded-auto`; `unknown-input` for
/// each input the protocol does not declare, then `missing-input` for each input of the protocol
/// the rule does not map, then `input-value-not-allowed` for each mapped input whose `{const}` is
/// not among its values or whose `{field}` is not a declared `string`, each in input-id order.
fn check_rule(
    key: &str,
    rule: &TriggerRule,
    digest: &str,
    published: &TriggerIndex,
    protocols: &BTreeMap<&model::ProtocolId, &model::TriggerProtocol>,
    failures: &mut Vec<Failure>,
) {
    if let Some(recorded) = published.rules.get(&key.to_owned())
        && recorded != digest
    {
        failures.push(Failure::of(
            "key-republished",
            key,
            format!(
                "the key is published with {}, this document has {digest}; a published rule is \
                 immutable, so a change is a new version",
                model::one_line(recorded)
            ),
        ));
    }
    let start = &rule.start;
    let Some(protocol) = protocols.get(&start.protocol) else {
        failures.push(Failure::of(
            "unknown-protocol",
            key,
            format!(
                "protocol `{}` is not in the protocols",
                model::one_line(start.protocol.as_str())
            ),
        ));
        return;
    };
    if protocol.guarded && rule.mode == TriggerMode::Auto {
        failures.push(Failure::of(
            "guarded-auto",
            key,
            format!(
                "protocol `{}` is guarded and the rule's mode is auto; a guarded protocol is not \
                 started unattended",
                model::one_line(start.protocol.as_str())
            ),
        ));
    }
    let mut mapped: Vec<_> = start.input.iter().collect();
    mapped.sort_by(|a, b| a.0.cmp(b.0));
    let mut declared: Vec<_> = protocol.inputs.ids().collect();
    declared.sort();
    for (name, _) in &mapped {
        if !protocol.inputs.contains(name) {
            failures.push(Failure::of(
                "unknown-input",
                key,
                format!(
                    "start.input.{}: protocol `{}` declares no such input",
                    model::one_line(name.as_str()),
                    model::one_line(start.protocol.as_str())
                ),
            ));
        }
    }
    for name in declared {
        if !start.input.contains(name) {
            failures.push(Failure::of(
                "missing-input",
                key,
                format!(
                    "protocol `{}` takes the case input `{}`, which start.input does not map",
                    model::one_line(start.protocol.as_str()),
                    model::one_line(name.as_str())
                ),
            ));
        }
    }
    for (name, input) in &mapped {
        let Some(values) = protocol.inputs.get(name) else {
            continue;
        };
        let why = match input {
            TriggerInput::Const(value)
                if !values.iter().any(|allowed| allowed.as_str() == value) =>
            {
                format!("`{}` is not among its values", model::one_line(value))
            }
            TriggerInput::Field(field) => match rule.subject.fields.get(field) {
                Some(declared) if declared.r#type != TriggerFieldType::String => format!(
                    "the field `{}` is declared {}, and the input's values are strings",
                    model::one_line(field.as_str()),
                    declared.r#type.as_str()
                ),
                _ => continue,
            },
            TriggerInput::Const(_) => continue,
        };
        let shown: Vec<&str> = values.iter().map(|value| value.as_str()).collect();
        failures.push(Failure::of(
            "input-value-not-allowed",
            key,
            format!(
                "start.input.{}: {why} ({})",
                model::one_line(name.as_str()),
                model::one_line(&shown.join(", "))
            ),
        ));
    }
}
