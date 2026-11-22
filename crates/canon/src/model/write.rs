//! Writing a `protocol/1` model out as text (story:protocol-writer). Pure: the caller keeps the
//! text.
//!
//! [`write`] gives canonical JSON, which [`parse`](super::parse) reads as YAML, so
//! `parse(&write(&p)) == Ok(p)` for every model `p` that `parse` gives in which every key, as
//! written with its escapes, is at most 1022 bytes (see "Keys"). The writer is hand-written
//! beside the model, as the `canon-ir/1` writer is: the model derives no `Serialize`, because a
//! derived one would write Rust variant tags and `null` for an absent option, which every optional
//! key refuses on read. Every model struct is destructured with no `..`, so a field added to the
//! model stops this file compiling until the writer writes it. `write` is infallible and returns
//! for every model whose nesting the calling thread's stack holds (see "Depth"). It writes a model
//! the validator refuses (a repeated identifier, a dangling reference) like any other, and
//! `canon validate` stays the only judge of validity.
//!
//! # Layout
//!
//! - Indent is two spaces, one member or item to a line, `"key": value` with one space after the
//!   colon. An empty object or list is `{}` or `[]` on its own line. The text ends with one
//!   newline.
//! - A declaration section keeps model order, repeated identifiers included.
//! - A key inside a body, and at the top level, follows the order the Rust struct declares its
//!   fields: `format`, `protocol` (`id`, `revision`, `description`), `artifacts`,
//!   `evidence_kinds`, `claims`, `obligations`, `actions`, `outcomes`, `invalidation`, `case`,
//!   `independence_dimensions`, `imports`. A predicate is an object of its form's key, then `is`
//!   beside `claim` and `in` beside `input`; inside an order form, `kind`, `result`, `first`,
//!   `then`, `last`, `in`, `revision` and `subject` follow the struct.
//! - Keys inside `arguments` follow the iteration order of `serde_json::Map` (sorted by key,
//!   unless the build keeps insertion order).
//!
//! # What is left out
//!
//! An absent `Option` and an empty defaulted section (`requires`, `may_produce`,
//! `records_together`, `revises`, `scope`, `refines`, `inputs` and each top-level declaration
//! section) are left out, because
//! `parse` reads a left-out key as the default and refuses an explicit `null`. `Some` of an empty
//! value is written (`case: {}`, `independent_on: []`), because the model tells it apart from
//! `None`. A required key is always written, empty or not (`values: []`, `invalidates: []`).
//!
//! # Spelling
//!
//! - A claim test is always written with `is`: `{"claim": "c", "is": "true"}`. A truth value is
//!   always the quoted word `"true"`, `"false"` or `"unknown"`.
//! - An outcome that requires an explicit decision is `"requires": {"decision": "<name>"}`.
//! - Every string and every key is quoted, so YAML's implicit typing (`true`, `null`, `~`, `1e3`,
//!   `<<`) cannot read an identifier as anything else. A case input value `parse` read from a
//!   boolean or a number (`values: [true, 2]`) is text in the model and is written as text.
//! - `"` is written `\"`, `\` is `\\`, and a line feed, carriage return, tab, backspace and form
//!   feed are `\n`, `\r`, `\t`, `\b` and `\f`. Every other character below U+0020 is `\uXXXX`
//!   with four uppercase hexadecimal digits, and so are U+007F to U+009F, U+2028, U+2029, U+FEFF,
//!   U+FFFE and U+FFFF: the characters YAML cannot hold as written, or reads as a line break. Every
//!   other character is written as itself. This is the escape set of `generate::quote`, copied and
//!   not shared because that function writes a YAML scalar and this one writes JSON; the short
//!   forms and the `\uXXXX` spelling are valid in both.
//! - A number in `arguments` is written as `serde_json` spells it. `parse` reads a YAML number as
//!   a 64-bit unsigned or signed integer, or else as a double, so it gives only those. See below
//!   for what a build keeps.
//!
//! # What the round trip keeps, and what it does not
//!
//! It keeps every model value `parse` gives: declaration order and repeated identifiers, `None`
//! against `Some` of an empty value, every string character, every key of `arguments`.
//!
//! It does not keep what the model does not hold: comments, YAML style, anchors and aliases
//! (aliases come out expanded), key order inside bodies and the order of sections, how a scalar
//! was spelled (`values: [true, 2]` comes out as `"true"` and `"2"`, a claim test written without
//! `is` comes out with it, a key written with its default such as `requires: []` is not written).
//! The second write of a parsed model is byte for byte the first: `write(&parse(&write(&p))?) ==
//! write(&p)`, and the same model gives the same bytes.
//!
//! # Numbers in `arguments`
//!
//! `serde_json::Number` holds a double or a 64-bit integer, or, with the crate feature
//! `arbitrary_precision`, the text of the number. The workspace build has the feature on, because
//! `b10x-canon-expr` turns it on and cargo unifies features; `cargo build -p b10x-canon` alone has
//! it off. `write` gives the spelling of the build it runs in, and `parse` always reads a YAML
//! number as `u64`, `i64` or `f64`, so:
//!
//! - In both builds, every number `parse` gives (an integer in `i64` or `u64`, or a finite double)
//!   is written as it is spelled and reads back equal: the integer's digits, a double as the
//!   shortest text that reads back the same double (`0.5`, `100.0`, `1e300`, `1.5e-7`, `5e-324`).
//! - Without the feature, a number is a double or a 64-bit integer and nothing else exists, so
//!   every model value that holds a number round-trips.
//! - With the feature, a number a program builds from other text is written as that text, except
//!   an integer beyond 64 bits, which is written as the double it names (`1e20`): `serde_yaml_ng`
//!   refuses an integer of 65 to 128 bits in `arguments`, and reads a longer one as a double. A
//!   number in the spelling `parse` gives reads back equal. One spelled otherwise (`1.50`, `1E5`,
//!   an integer beyond 64 bits) reads back as the number `parse` spells for the same double
//!   (`1.5`, `100000.0`, `1e20`), and writing that again gives the same bytes. One beyond the
//!   range of a double (`1e400`) is written as its text, and `parse` reads it as the string
//!   `"1e400"`, as it does in any document: no spelling reads back as that number.
//!
//! # Keys
//!
//! The YAML reader behind `parse` drops a simple key whose text runs more than 1024 bytes before
//! its `:`. A key is written quoted, so the quotes count and the key leaves 1022 bytes of written
//! text, escapes included: a `\`, a `"` and a line feed, carriage return, tab, backspace or form
//! feed take two bytes, every other escaped character six, and any other character its length in
//! UTF-8. The limit holds for an identifier and for a property
//! name inside `arguments`, and `parse` itself gives longer keys (a plain key of 1023 bytes, an
//! explicit `?` key of any length, a key whose escapes grow it when written). JSON has one spelling
//! for a key, so the round trip holds for a model in which every key is at most 1022 bytes as
//! written, and for a longer key `write` still returns JSON that `serde_json` reads with the key
//! intact, and `parse` refuses that text.
//!
//! # Depth
//!
//! `write` recurses per level of nesting of a predicate or of an `arguments` value, one call per
//! level and two per level of `all` or `any`, so it uses stack in proportion to the nesting depth
//! and returns for every model whose nesting the calling thread's stack holds. `parse` gives at
//! most 123 levels of predicate and 125 of `arguments` (`serde_yaml_ng`'s recursion limit), and
//! the written text nests exactly as deep as the model, so every model `parse` gives is written
//! and read back. A model built in Rust nested deeper than `parse` reads is written (a predicate
//! 400 deep, of any form, on a thread with a 2 MiB stack, in a debug build) as far as the stack
//! allows, and `parse` refuses its text.
//!
//! # Model values `parse` never gives
//!
//! - `arguments: Some(null)`: `parse` refuses an explicit `null` on `arguments`, so the model has
//!   no `protocol/1` spelling for it. A `null` inside the arguments reads back.
//! - A number beyond the range of a double, in the build that holds number text: it reads back as
//!   a string.
//!
//! Nothing here evaluates, validates or compiles the model, and no clock, file or network is read.

use std::fmt::Write as _;

use serde_json::{Number, Value};

use super::{
    Action, Artifact, Author, CapabilityRequirement, CaseInput, CaseOpening, CaseSection,
    ChildOutcomes, Claim, ClaimTest, Declarations, EvidenceAfter, EvidenceKind, EvidenceMatch,
    EvidenceOrder, EvidenceProduction, EvidenceSince, FirstResult, Import, IndependenceDimension,
    InputTest, InvalidationRule, LaterRecord, Obligation, Outcome, OutcomeRequirement, Predicate,
    Protocol, ProtocolHeader, RecordGroup,
};

/// The members of one object, each value already written one level deeper than its key.
type Members = Vec<(String, String)>;

/// Writes `protocol` as canonical JSON text that [`parse`](super::parse) reads back to an equal
/// model; see the module documentation for the layout, the spelling and the limits.
pub fn write(protocol: &Protocol) -> String {
    let mut text = protocol_object(protocol);
    text.push('\n');
    text
}

/// One key and its value, written one level deeper than `members`' object.
fn put(members: &mut Members, key: &str, value: String) {
    members.push((key.to_owned(), value));
}

/// `value` under `key`, when there is one.
fn put_some(members: &mut Members, key: &str, value: Option<String>) {
    if let Some(value) = value {
        put(members, key, value);
    }
}

/// `value` under `key`, unless it is `[]`: a defaulted list is left out when empty.
fn put_unless_empty(members: &mut Members, key: &str, value: String) {
    if value != "[]" && value != "{}" {
        put(members, key, value);
    }
}

fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}

/// An object at `depth`: `{}` when it has no member.
fn object(depth: usize, members: Members) -> String {
    if members.is_empty() {
        return "{}".to_owned();
    }
    let mut text = String::from("{\n");
    let last = members.len() - 1;
    for (index, (key, value)) in members.iter().enumerate() {
        let _ = write!(text, "{}{}: {value}", indent(depth + 1), quote(key));
        text.push_str(if index == last { "\n" } else { ",\n" });
    }
    text.push_str(&indent(depth));
    text.push('}');
    text
}

/// A list at `depth` of items already written one level deeper: `[]` when it has none.
fn list(depth: usize, items: Vec<String>) -> String {
    if items.is_empty() {
        return "[]".to_owned();
    }
    let mut text = String::from("[\n");
    let last = items.len() - 1;
    for (index, item) in items.iter().enumerate() {
        let _ = write!(text, "{}{item}", indent(depth + 1));
        text.push_str(if index == last { "\n" } else { ",\n" });
    }
    text.push_str(&indent(depth));
    text.push(']');
    text
}

/// A list of identifiers or other text.
fn names<T: AsRef<str>>(depth: usize, items: &[T]) -> String {
    list(
        depth,
        items.iter().map(|item| quote(item.as_ref())).collect(),
    )
}

/// A declaration section at `depth`, entries in model order, repeated keys included.
fn declarations<K: AsRef<str> + PartialEq, V>(
    depth: usize,
    entries: &Declarations<K, V>,
    body: impl Fn(usize, &V) -> String,
) -> String {
    object(
        depth,
        entries
            .iter()
            .map(|(key, value)| (key.as_ref().to_owned(), body(depth + 1, value)))
            .collect(),
    )
}

/// `text` as a JSON string that YAML reads back as `text`, as the module documentation says.
fn quote(text: &str) -> String {
    let mut quoted = String::with_capacity(text.len() + 2);
    quoted.push('"');
    for c in text.chars() {
        match c {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            '\u{8}' => quoted.push_str("\\b"),
            '\u{c}' => quoted.push_str("\\f"),
            '\u{7f}'..='\u{9f}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{feff}'
            | '\u{fffe}'
            | '\u{ffff}' => {
                let _ = write!(quoted, "\\u{:04X}", u32::from(c));
            }
            c if c < ' ' => {
                let _ = write!(quoted, "\\u{:04X}", u32::from(c));
            }
            c => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}

fn author(author: Author) -> String {
    quote(author.as_str())
}

fn protocol_object(protocol: &Protocol) -> String {
    let Protocol {
        format,
        protocol: header,
        artifacts,
        evidence_kinds,
        claims,
        obligations,
        actions,
        outcomes,
        invalidation,
        case,
        independence_dimensions,
        imports,
    } = protocol;
    let mut members = Members::new();
    put(&mut members, "format", quote(format));
    put(&mut members, "protocol", header_object(1, header));
    put_unless_empty(
        &mut members,
        "artifacts",
        declarations(1, artifacts, artifact),
    );
    put_unless_empty(
        &mut members,
        "evidence_kinds",
        declarations(1, evidence_kinds, evidence_kind),
    );
    put_unless_empty(&mut members, "claims", declarations(1, claims, claim));
    put_unless_empty(
        &mut members,
        "obligations",
        declarations(1, obligations, obligation),
    );
    put_unless_empty(&mut members, "actions", declarations(1, actions, action));
    put_unless_empty(&mut members, "outcomes", declarations(1, outcomes, outcome));
    put_unless_empty(
        &mut members,
        "invalidation",
        declarations(1, invalidation, invalidation_rule),
    );
    put_some(
        &mut members,
        "case",
        case.as_ref().map(|case| case_section(1, case)),
    );
    put_unless_empty(
        &mut members,
        "independence_dimensions",
        declarations(1, independence_dimensions, independence_dimension),
    );
    put_unless_empty(&mut members, "imports", declarations(1, imports, import));
    object(0, members)
}

fn header_object(depth: usize, header: &ProtocolHeader) -> String {
    let ProtocolHeader {
        id,
        revision,
        description,
    } = header;
    let mut members = Members::new();
    put(&mut members, "id", quote(id.as_str()));
    put(&mut members, "revision", revision.to_string());
    put_some(
        &mut members,
        "description",
        description.as_deref().map(quote),
    );
    object(depth, members)
}

fn artifact(depth: usize, artifact: &Artifact) -> String {
    let Artifact { description } = artifact;
    let mut members = Members::new();
    put_some(
        &mut members,
        "description",
        description.as_deref().map(quote),
    );
    object(depth, members)
}

fn evidence_kind(depth: usize, kind: &EvidenceKind) -> String {
    let EvidenceKind {
        description,
        max_age,
        supplied_by,
    } = kind;
    let mut members = Members::new();
    put_some(
        &mut members,
        "description",
        description.as_deref().map(quote),
    );
    put_some(
        &mut members,
        "max_age",
        max_age.as_ref().map(|age| quote(age.as_str())),
    );
    put_some(
        &mut members,
        "supplied_by",
        supplied_by
            .as_ref()
            .map(|protocol| quote(protocol.as_str())),
    );
    object(depth, members)
}

fn claim(depth: usize, claim: &Claim) -> String {
    let Claim {
        description,
        true_when,
    } = claim;
    let mut members = Members::new();
    put_some(
        &mut members,
        "description",
        description.as_deref().map(quote),
    );
    put(&mut members, "true_when", predicate(depth + 1, true_when));
    object(depth, members)
}

fn obligation(depth: usize, obligation: &Obligation) -> String {
    let Obligation {
        description,
        discharged_when,
        applicable_when,
    } = obligation;
    let mut members = Members::new();
    put_some(
        &mut members,
        "description",
        description.as_deref().map(quote),
    );
    put(
        &mut members,
        "discharged_when",
        predicate(depth + 1, discharged_when),
    );
    put_some(
        &mut members,
        "applicable_when",
        applicable_when
            .as_ref()
            .map(|when| predicate(depth + 1, when)),
    );
    object(depth, members)
}

fn action(depth: usize, action: &Action) -> String {
    let Action {
        description,
        precondition,
        requires,
        effect,
        may_produce,
        records_together,
        arguments,
        case_open,
        author: who,
        revises,
    } = action;
    let mut members = Members::new();
    put_some(
        &mut members,
        "description",
        description.as_deref().map(quote),
    );
    put_some(
        &mut members,
        "precondition",
        precondition.as_ref().map(|when| predicate(depth + 1, when)),
    );
    put_unless_empty(
        &mut members,
        "requires",
        list(
            depth + 1,
            requires
                .iter()
                .map(|requirement| capability_requirement(depth + 2, requirement))
                .collect(),
        ),
    );
    put_some(
        &mut members,
        "effect",
        effect.as_ref().map(|effect| quote(effect.as_str())),
    );
    put_unless_empty(
        &mut members,
        "may_produce",
        list(
            depth + 1,
            may_produce
                .iter()
                .map(|production| evidence_production(depth + 2, production))
                .collect(),
        ),
    );
    put_unless_empty(
        &mut members,
        "records_together",
        list(
            depth + 1,
            records_together
                .iter()
                .map(|group| record_group(depth + 2, group))
                .collect(),
        ),
    );
    put_some(
        &mut members,
        "arguments",
        arguments.as_ref().map(|json| json_value(depth + 1, json)),
    );
    put_some(
        &mut members,
        "case_open",
        case_open
            .as_ref()
            .map(|opening| case_opening(depth + 1, opening)),
    );
    put_some(&mut members, "author", who.map(author));
    put_unless_empty(&mut members, "revises", names(depth + 1, revises));
    object(depth, members)
}

fn case_opening(depth: usize, opening: &CaseOpening) -> String {
    let CaseOpening { protocol, subject } = opening;
    let mut members = Members::new();
    put(&mut members, "protocol", quote(protocol.as_str()));
    put(&mut members, "subject", quote(subject.as_str()));
    object(depth, members)
}

fn capability_requirement(depth: usize, requirement: &CapabilityRequirement) -> String {
    let CapabilityRequirement { capability, scope } = requirement;
    let mut members = Members::new();
    put(&mut members, "capability", quote(capability.as_str()));
    put_unless_empty(
        &mut members,
        "scope",
        declarations(depth + 1, scope, |_, value| quote(value.as_str())),
    );
    object(depth, members)
}

fn evidence_production(depth: usize, production: &EvidenceProduction) -> String {
    let EvidenceProduction {
        evidence,
        decided_by,
    } = production;
    let mut members = Members::new();
    put(&mut members, "evidence", quote(evidence.as_str()));
    put_some(&mut members, "decided_by", decided_by.map(author));
    object(depth, members)
}

fn record_group(depth: usize, group: &RecordGroup) -> String {
    let RecordGroup { kinds } = group;
    let mut members = Members::new();
    put(&mut members, "kinds", names(depth + 1, kinds));
    object(depth, members)
}

fn outcome(depth: usize, outcome: &Outcome) -> String {
    let Outcome {
        description,
        requires,
    } = outcome;
    let mut members = Members::new();
    put_some(
        &mut members,
        "description",
        description.as_deref().map(quote),
    );
    put(
        &mut members,
        "requires",
        match requires {
            OutcomeRequirement::Predicate(requirement) => predicate(depth + 1, requirement),
            OutcomeRequirement::Decision(requirement) => {
                let mut decision = Members::new();
                put(
                    &mut decision,
                    "decision",
                    quote(requirement.decision.as_str()),
                );
                put_some(
                    &mut decision,
                    "when",
                    requirement
                        .when
                        .as_ref()
                        .map(|when| predicate(depth + 2, when)),
                );
                object(depth + 1, decision)
            }
        },
    );
    object(depth, members)
}

fn invalidation_rule(depth: usize, rule: &InvalidationRule) -> String {
    let InvalidationRule {
        description,
        upstream,
        invalidates,
    } = rule;
    let mut members = Members::new();
    put_some(
        &mut members,
        "description",
        description.as_deref().map(quote),
    );
    put(&mut members, "upstream", quote(upstream.as_str()));
    put(&mut members, "invalidates", names(depth + 1, invalidates));
    object(depth, members)
}

fn case_section(depth: usize, case: &CaseSection) -> String {
    let CaseSection { inputs } = case;
    let mut members = Members::new();
    put_unless_empty(
        &mut members,
        "inputs",
        declarations(depth + 1, inputs, case_input),
    );
    object(depth, members)
}

fn case_input(depth: usize, input: &CaseInput) -> String {
    let CaseInput {
        description,
        values,
    } = input;
    let mut members = Members::new();
    put_some(
        &mut members,
        "description",
        description.as_deref().map(quote),
    );
    put(&mut members, "values", names(depth + 1, values));
    object(depth, members)
}

fn independence_dimension(depth: usize, dimension: &IndependenceDimension) -> String {
    let IndependenceDimension { description } = dimension;
    let mut members = Members::new();
    put_some(
        &mut members,
        "description",
        description.as_deref().map(quote),
    );
    object(depth, members)
}

fn import(depth: usize, import: &Import) -> String {
    let Import {
        revision,
        digest,
        refines,
    } = import;
    let mut members = Members::new();
    put(&mut members, "revision", revision.to_string());
    put(&mut members, "digest", quote(digest));
    put_unless_empty(&mut members, "refines", names(depth + 1, refines));
    object(depth, members)
}

fn predicate(depth: usize, predicate: &Predicate) -> String {
    let mut members = Members::new();
    match predicate {
        Predicate::All(inner) => put(&mut members, "all", predicates(depth + 1, inner)),
        Predicate::Any(inner) => put(&mut members, "any", predicates(depth + 1, inner)),
        Predicate::Not(inner) => put(&mut members, "not", self::predicate(depth + 1, inner)),
        Predicate::Evidence(matching) => {
            let EvidenceMatch {
                kind,
                result,
                subject,
                independent_on,
            } = matching;
            let mut body = Members::new();
            put(&mut body, "kind", quote(kind.as_str()));
            put_some(&mut body, "result", result.as_deref().map(quote));
            put_some(
                &mut body,
                "subject",
                subject.as_ref().map(|id| quote(id.as_str())),
            );
            put_some(
                &mut body,
                "independent_on",
                independent_on
                    .as_ref()
                    .map(|dimensions| names(depth + 2, dimensions)),
            );
            put(&mut members, "evidence", object(depth + 1, body));
        }
        Predicate::Input(test) => {
            let InputTest { input, r#in } = test;
            put(&mut members, "input", quote(input.as_str()));
            put(&mut members, "in", names(depth + 1, r#in));
        }
        Predicate::First(form) => {
            let FirstResult {
                kind,
                result,
                subject,
            } = form;
            let mut body = Members::new();
            put(&mut body, "kind", quote(kind.as_str()));
            put(&mut body, "result", quote(result));
            put_some(
                &mut body,
                "subject",
                subject.as_ref().map(|id| quote(id.as_str())),
            );
            put(&mut members, "first", object(depth + 1, body));
        }
        Predicate::Before(form) => {
            let EvidenceOrder {
                first,
                then,
                subject,
            } = form;
            let mut body = Members::new();
            put(&mut body, "first", quote(first.as_str()));
            put(&mut body, "then", quote(then.as_str()));
            put_some(
                &mut body,
                "subject",
                subject.as_ref().map(|id| quote(id.as_str())),
            );
            put(&mut members, "before", object(depth + 1, body));
        }
        Predicate::After(form) => {
            let EvidenceAfter {
                first,
                then,
                subject,
            } = form;
            let mut body = Members::new();
            put(&mut body, "first", quote(first.as_str()));
            put(&mut body, "then", later_record(depth + 2, then));
            put_some(
                &mut body,
                "subject",
                subject.as_ref().map(|id| quote(id.as_str())),
            );
            put(&mut members, "after", object(depth + 1, body));
        }
        Predicate::Since(form) => {
            let EvidenceSince {
                last,
                then,
                subject,
            } = form;
            let mut body = Members::new();
            put(&mut body, "last", quote(last.as_str()));
            put(&mut body, "then", later_record(depth + 2, then));
            put_some(
                &mut body,
                "subject",
                subject.as_ref().map(|id| quote(id.as_str())),
            );
            put(&mut members, "since", object(depth + 1, body));
        }
        Predicate::Children(form) => {
            let ChildOutcomes {
                r#in,
                revision,
                subject,
            } = form;
            let mut body = Members::new();
            put(&mut body, "in", names(depth + 2, r#in));
            put_some(
                &mut body,
                "revision",
                revision.map(|revision| quote(revision.as_str())),
            );
            put_some(
                &mut body,
                "subject",
                subject.as_ref().map(|id| quote(id.as_str())),
            );
            put(&mut members, "children", object(depth + 1, body));
        }
        Predicate::Claim(test) => {
            let ClaimTest { claim, is } = test;
            put(&mut members, "claim", quote(claim.as_str()));
            put(&mut members, "is", quote(&is.to_string()));
        }
    }
    object(depth, members)
}

fn predicates(depth: usize, members: &[Predicate]) -> String {
    list(
        depth,
        members
            .iter()
            .map(|member| predicate(depth + 1, member))
            .collect(),
    )
}

fn later_record(depth: usize, record: &LaterRecord) -> String {
    let LaterRecord { kind, result } = record;
    let mut members = Members::new();
    put(&mut members, "kind", quote(kind.as_str()));
    put(&mut members, "result", quote(result));
    object(depth, members)
}

/// A number as the build spells it, except an integer beyond 64 bits, which `parse` refuses as
/// digits (`serde_yaml_ng` offers it to the reader as 128 bits) and reads as a double as `1e20`.
/// Only a build that holds number text can hold one.
fn number_text(number: &Number) -> String {
    let text = number.to_string();
    if number.is_u64() || number.is_i64() {
        return text;
    }
    let digits = text.strip_prefix('-').unwrap_or(&text);
    if digits.bytes().all(|byte| byte.is_ascii_digit()) {
        let double = number.as_f64().and_then(Number::from_f64);
        if let Some(double) = double {
            return double.to_string();
        }
    }
    text
}

/// A JSON value, as `serde_json` holds it: a number as that build spells it.
fn json_value(depth: usize, value: &Value) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(value) => value.to_string(),
        Value::Number(number) => number_text(number),
        Value::String(text) => quote(text),
        Value::Array(items) => list(
            depth,
            items
                .iter()
                .map(|item| json_value(depth + 1, item))
                .collect(),
        ),
        Value::Object(map) => object(
            depth,
            map.iter()
                .map(|(key, value)| (key.clone(), json_value(depth + 1, value)))
                .collect(),
        ),
    }
}
