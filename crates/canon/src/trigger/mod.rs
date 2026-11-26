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

use std::collections::BTreeMap;
use std::fmt;

use canon_expr::{Builtin, Catalog, CheckedExpr, Diagnostic, Function, Implementation, Parameter};

use crate::model::{self, TRIGGER_FORMAT, TriggerFieldType, TriggerInput, TriggerRule};

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

#[cfg(test)]
mod tests;
