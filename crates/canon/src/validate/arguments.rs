//! Canon's argument-schema dialect (story:action-arguments): the JSON Schemas an action's
//! `arguments` may be.
//!
//! The dialect is a subset of JSON Schema 2020-12. Every schema in it is a valid 2020-12 schema
//! with the same meaning, so a runner can hand it to any 2020-12 validator; protocols load
//! offline, so it has no reference at all.
//!
//! - Every schema, the top level and each nested one, names exactly one `type`: `object`,
//!   `array`, `string`, `integer`, `number`, `boolean` or `null`. The top level is `type: object`.
//! - `title` and `description`, strings, anywhere.
//! - `enum`, on any type: a non-empty list of distinct JSON scalars, each of the schema's type.
//! - `object`: `properties` (a map of name to schema), `required` (a list of distinct names, each
//!   declared in `properties`), `additionalProperties` (a boolean only).
//! - `array`: `items` (one schema), `minItems`, `maxItems`.
//! - `string`: `minLength`, `maxLength`.
//! - `integer` and `number`: `minimum`, `maximum`, integers.
//! - `minItems`, `maxItems`, `minLength` and `maxLength` are non-negative integers, and each
//!   minimum is at most its maximum.
//! - Schemas nest at most [`MAX_SCHEMA_DEPTH`] deep: the top level is one
//!   level, and each schema in `properties` or `items` one more.
//!
//! Anything else is outside the dialect: `$ref`, `$defs`, `$schema`, `$id`, `allOf`, `anyOf`,
//! `oneOf`, `not`, `if`, `then`, `else`, `patternProperties`, `pattern`, `format`, `const`,
//! `exclusiveMinimum` and `exclusiveMaximum` by name, a list-valued `type`, a schema-valued
//! `additionalProperties`, a keyword on a type it does not apply to and any unknown keyword.
//!
//! [`check`] reports one problem per offending keyword, at the keyword's JSON Pointer inside the
//! schema (an element's, for an `enum` value or a `required` name): a schema that is not an object,
//! names no `type` or nests too deep is reported at its own pointer. A schema whose `type` is
//! missing or not one of the seven has its other keywords checked only for being in the dialect,
//! since which apply depends on the type. Problems come in document order, the keys of every
//! object in code-point order, and the walk keeps a stack of its own, never recursing.
//!
//! # Checking a call (story:argument-validator)
//!
//! [`check_arguments`] checks one call's arguments against an action's schema. It is a pure
//! function: it reads nothing but its two arguments, evaluates no call and takes no decision. A
//! runner that holds an `ir::Action` passes `action.arguments.as_ref()`; one that holds
//! `canon-ir/1` as JSON passes `ir["actions"][id]["arguments"]`, `None` when the key is absent.
//!
//! - An absent schema is `{"type": "object", "additionalProperties": false}`: the action takes
//!   only `{}`.
//! - A schema outside the dialect, which only a caller-built IR can hold, is returned as
//!   [`ArgumentsRefused::Schema`], the pairs of JSON Pointer inside the schema and reason that
//!   [`check`] gives `canon validate`, unchanged; no argument is checked.
//! - Otherwise every refusal is reported as [`ArgumentsRefused::Arguments`], each an
//!   [`ArgumentRefusal`]: the RFC 6901 pointer into the arguments (`""` for the whole document),
//!   the keyword that refused and one line of text. Meaning is that of JSON Schema 2020-12.
//! - `type`: an `integer` is any number with no fractional part, so `1.0` is an integer. When
//!   `type` refuses at a location, no other keyword of that schema is checked there.
//! - `enum`: values are equal when they are equal in JSON Schema's sense, numbers by exact value:
//!   `1` and `1.0` are one value, and two integers a double cannot tell apart are two.
//! - `required`: a missing name is refused at the object's pointer, the reason naming it, in the
//!   order the schema lists the names.
//! - `additionalProperties: false`: an undeclared member is refused at the member's pointer.
//! - `properties` and `items` descend into the members and items a schema describes, and only
//!   those, so the depth reached is at most [`MAX_SCHEMA_DEPTH`].
//! - `minItems`, `maxItems` count the array's items; `minLength`, `maxLength` count the string's
//!   Unicode code points.
//! - `minimum`, `maximum` compare exactly: an integer is never compared through a double, and a
//!   fractional number is compared by its exact value. A number the JSON text writes beyond the
//!   range of a 64-bit integer is the double `serde_json` reads it as.
//! - `title` and `description` never refuse.
//!
//! Refusals come in document order: a location's own refusals first, its keywords in code-point
//! order (`enum`, the maximums, the minimums, then `required`), then its members in code-point
//! order of their names, or its items in order. The same input gives the same refusals.

use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt;

use serde_json::{Map, Number, Value, json};

use crate::model::one_line;

/// How deep schemas nest: far below the bound on predicates, since every pass over a schema value
/// (parsing, writing, reading back, comparing and dropping it) recurses once per level.
pub(crate) const MAX_SCHEMA_DEPTH: usize = 32;

/// The seven types, each a keyword's applicability.
const TYPES: [&str; 7] = [
    "object", "array", "string", "integer", "number", "boolean", "null",
];

/// Keywords refused by name, whatever the type.
const REFUSED: [&str; 17] = [
    "$ref",
    "$defs",
    "$schema",
    "$id",
    "allOf",
    "anyOf",
    "oneOf",
    "not",
    "if",
    "then",
    "else",
    "patternProperties",
    "pattern",
    "format",
    "const",
    "exclusiveMinimum",
    "exclusiveMaximum",
];

/// The types a type-specific keyword applies to; `None` for a keyword not in the dialect.
fn applies_to(keyword: &str) -> Option<&'static [&'static str]> {
    Some(match keyword {
        "type" | "title" | "description" | "enum" => &TYPES,
        "properties" | "required" | "additionalProperties" => &["object"],
        "items" | "minItems" | "maxItems" => &["array"],
        "minLength" | "maxLength" => &["string"],
        "minimum" | "maximum" => &["integer", "number"],
        _ => return None,
    })
}

/// One offence: the JSON Pointer inside the schema and why it is refused.
pub(super) type Offence = (String, String);

/// Where a pointer token sits: its parent's index in the walk's arena, `None` for the root.
type At = Option<usize>;

/// What the walk does next, in document order.
enum Item<'a> {
    Schema {
        value: &'a Value,
        at: At,
        depth: usize,
    },
    Offence {
        at: At,
        reason: String,
    },
}

/// The tokens of every pointer the walk has made, each with its parent, so a deep schema costs one
/// token per level, not one pointer string.
#[derive(Default)]
struct Pointers {
    tokens: Vec<(At, String)>,
}

impl Pointers {
    fn child(&mut self, parent: At, token: impl Into<String>) -> At {
        self.tokens.push((parent, token.into()));
        Some(self.tokens.len() - 1)
    }

    /// The JSON Pointer (RFC 6901) of `at`: `""` for the root, each token escaped.
    fn render(&self, mut at: At) -> String {
        let mut tokens = Vec::new();
        while let Some(index) = at {
            let (parent, token) = &self.tokens[index];
            tokens.push(token.replace('~', "~0").replace('/', "~1"));
            at = *parent;
        }
        tokens
            .iter()
            .rev()
            .map(|token| format!("/{token}"))
            .collect()
    }
}

/// Every offence of `schema` against the dialect, in document order, as the module docs say.
pub(super) fn check(schema: &Value) -> Vec<Offence> {
    let mut pointers = Pointers::default();
    let mut offences = Vec::new();
    let mut stack = vec![Item::Schema {
        value: schema,
        at: None,
        depth: 1,
    }];
    while let Some(item) = stack.pop() {
        match item {
            Item::Offence { at, reason } => offences.push((pointers.render(at), reason)),
            Item::Schema { value, at, depth } => {
                let items = schema_items(value, at, depth, &mut pointers);
                stack.extend(items.into_iter().rev());
            }
        }
    }
    offences
}

/// What one schema contributes, in document order: its own offences, and its nested schemas at
/// the place of the keyword that holds them.
fn schema_items<'a>(
    value: &'a Value,
    at: At,
    depth: usize,
    pointers: &mut Pointers,
) -> Vec<Item<'a>> {
    let offence = |at: At, reason: String| Item::Offence { at, reason };
    let Value::Object(schema) = value else {
        return vec![offence(at, "the schema is not an object".to_owned())];
    };
    if depth > MAX_SCHEMA_DEPTH {
        return vec![offence(
            at,
            format!("the schema nests schemas deeper than {MAX_SCHEMA_DEPTH}"),
        )];
    }
    let mut items = Vec::new();
    let ty = match schema.get("type") {
        Some(Value::String(ty)) if TYPES.contains(&ty.as_str()) => Some(ty.as_str()),
        Some(_) => None,
        None => {
            items.push(offence(at, "the schema names no `type`".to_owned()));
            None
        }
    };
    let mut keywords: Vec<&String> = schema.keys().collect();
    keywords.sort();
    for keyword in keywords {
        let here = pointers.child(at, keyword.as_str());
        let value = &schema[keyword.as_str()];
        if REFUSED.contains(&keyword.as_str()) {
            items.push(offence(here, format!("`{keyword}` is outside the dialect")));
            continue;
        }
        let Some(types) = applies_to(keyword) else {
            items.push(offence(
                here,
                format!("`{keyword}` is not a keyword of the dialect"),
            ));
            continue;
        };
        if keyword == "type" {
            if let Some(reason) = type_offence(value, ty, at.is_none()) {
                items.push(offence(here, reason));
            }
            continue;
        }
        if keyword == "title" || keyword == "description" {
            if !value.is_string() {
                items.push(offence(here, format!("`{keyword}` is not a string")));
            }
            continue;
        }
        // Which keywords apply, and what their values may be, depends on the type.
        let Some(ty) = ty else { continue };
        if !types.contains(&ty) {
            items.push(offence(
                here,
                format!("`{keyword}` does not apply to `type: {ty}`"),
            ));
            continue;
        }
        match keyword.as_str() {
            "enum" => enum_items(value, ty, here, pointers, &mut items),
            "properties" => match value {
                Value::Object(properties) => {
                    let mut names: Vec<&String> = properties.keys().collect();
                    names.sort();
                    for name in names {
                        let at = pointers.child(here, name.as_str());
                        items.push(Item::Schema {
                            value: &properties[name.as_str()],
                            at,
                            depth: depth + 1,
                        });
                    }
                }
                _ => items.push(offence(
                    here,
                    "`properties` is not a map of names to schemas".to_owned(),
                )),
            },
            "required" => required_items(value, schema, here, pointers, &mut items),
            "additionalProperties" => match value {
                Value::Bool(_) => {}
                Value::Object(_) => items.push(offence(
                    here,
                    "a schema-valued `additionalProperties` is outside the dialect".to_owned(),
                )),
                _ => items.push(offence(
                    here,
                    "`additionalProperties` is not a boolean".to_owned(),
                )),
            },
            "items" => items.push(Item::Schema {
                value,
                at: here,
                depth: depth + 1,
            }),
            "minItems" | "maxItems" | "minLength" | "maxLength" => {
                if value.as_u64().is_none() {
                    items.push(offence(
                        here,
                        format!("`{keyword}` is not a non-negative integer"),
                    ));
                } else if let Some(reason) = above(schema, keyword) {
                    items.push(offence(here, reason));
                }
            }
            "minimum" | "maximum" => {
                if integer(value).is_none() {
                    items.push(offence(here, format!("`{keyword}` is not an integer")));
                } else if let Some(reason) = above(schema, keyword) {
                    items.push(offence(here, reason));
                }
            }
            other => unreachable!("`{other}` has an applicability but no check"),
        }
    }
    items
}

/// Why `type` is refused, if it is: a list, a name not of the seven, anything but a string, or a
/// top-level type other than `object`.
fn type_offence(value: &Value, ty: Option<&str>, top: bool) -> Option<String> {
    match (value, ty) {
        (Value::Array(_), _) => Some("a list-valued `type` is outside the dialect".to_owned()),
        (Value::String(name), None) => Some(format!("`{name}` is not a type of the dialect")),
        (_, None) => Some("`type` is not a string".to_owned()),
        (_, Some(ty)) if top && ty != "object" => Some(format!(
            "the top-level schema is `type: {ty}`; it must be `type: object`"
        )),
        _ => None,
    }
}

/// A JSON integer as a wide integer, for comparing bounds of either sign.
fn integer(value: &Value) -> Option<i128> {
    value
        .as_i64()
        .map(i128::from)
        .or_else(|| value.as_u64().map(i128::from))
}

/// For a minimum keyword whose value is well formed: why it is above its well-formed maximum.
fn above(schema: &Map<String, Value>, keyword: &str) -> Option<String> {
    let maximum = match keyword {
        "minItems" => "maxItems",
        "minLength" => "maxLength",
        "minimum" => "maximum",
        _ => return None,
    };
    let (low, high) = (integer(&schema[keyword])?, integer(schema.get(maximum)?)?);
    (low > high).then(|| format!("`{keyword}` {low} is above `{maximum}` {high}"))
}

/// The offences of an `enum` on a schema of type `ty`.
fn enum_items(value: &Value, ty: &str, here: At, pointers: &mut Pointers, items: &mut Vec<Item>) {
    let values = match value {
        Value::Array(values) if !values.is_empty() => values,
        _ => {
            items.push(Item::Offence {
                at: here,
                reason: "`enum` is not a non-empty list".to_owned(),
            });
            return;
        }
    };
    for (index, item) in values.iter().enumerate() {
        let reason = if item.is_array() || item.is_object() {
            Some(format!("`{item}` is not a scalar"))
        } else if !of_type(item, ty) {
            Some(format!("`{item}` is not of `type: {ty}`"))
        } else if values[..index].iter().any(|earlier| same(earlier, item)) {
            Some(format!("`{item}` is listed more than once"))
        } else {
            None
        };
        if let Some(reason) = reason {
            let at = pointers.child(here, index.to_string());
            items.push(Item::Offence { at, reason });
        }
    }
}

/// Whether a scalar is of a type: an integer is a number with no fraction, and a number any number.
fn of_type(value: &Value, ty: &str) -> bool {
    match ty {
        "string" => value.is_string(),
        "integer" => value.is_i64() || value.is_u64(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => false,
    }
}

/// JSON Schema's equality of two scalars: numbers by exact value, so `1` and `1.0` are one value
/// and two integers a double cannot tell apart are two.
pub(crate) fn same(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => compare(left, right) == Ordering::Equal,
        _ => left == right,
    }
}

fn integer_of(number: &Number) -> Option<i128> {
    number
        .as_i64()
        .map(i128::from)
        .or_else(|| number.as_u64().map(i128::from))
}

/// The exact order of two JSON numbers, each a 64-bit integer or a finite double.
fn compare(left: &Number, right: &Number) -> Ordering {
    match (integer_of(left), integer_of(right)) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(left), None) => against_double(left, double(right)),
        (None, Some(right)) => against_double(right, double(left)).reverse(),
        (None, None) => double(left)
            .partial_cmp(&double(right))
            .unwrap_or(Ordering::Equal),
    }
}

fn double(number: &Number) -> f64 {
    number.as_f64().unwrap_or(f64::NAN)
}

/// The exact order of an integer and a finite double.
fn against_double(integer: i128, double: f64) -> Ordering {
    // 2^127: every double at or beyond it is above every i128, and every double at or below its
    // negation is below.
    let limit = i128::MAX as f64;
    if double >= limit {
        return Ordering::Less;
    }
    if double <= -limit {
        return Ordering::Greater;
    }
    let floor = double.floor();
    match integer.cmp(&(floor as i128)) {
        Ordering::Equal if double > floor => Ordering::Less,
        order => order,
    }
}

/// The offences of `required` on an object schema.
fn required_items(
    value: &Value,
    schema: &Map<String, Value>,
    here: At,
    pointers: &mut Pointers,
    items: &mut Vec<Item>,
) {
    let Value::Array(names) = value else {
        items.push(Item::Offence {
            at: here,
            reason: "`required` is not a list of names".to_owned(),
        });
        return;
    };
    let declared = schema.get("properties").and_then(Value::as_object);
    let mut seen = BTreeSet::new();
    for (index, name) in names.iter().enumerate() {
        let reason = match name {
            Value::String(name) if !seen.insert(name.as_str()) => {
                Some(format!("`{name}` is listed more than once"))
            }
            Value::String(name) if !declared.is_some_and(|map| map.contains_key(name)) => {
                Some(format!("`{name}` is not declared in `properties`"))
            }
            Value::String(_) => None,
            other => Some(format!("`{other}` is not a name")),
        };
        if let Some(reason) = reason {
            let at = pointers.child(here, index.to_string());
            items.push(Item::Offence { at, reason });
        }
    }
}

/// One refusal of a call's arguments by the schema (story:argument-validator).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgumentRefusal {
    /// The RFC 6901 JSON Pointer into the arguments document; `""` is the whole document.
    pub pointer: String,
    /// The dialect keyword that refused: `type`, `enum`, `required`, `additionalProperties`,
    /// `minimum`, `maximum`, `minLength`, `maxLength`, `minItems` or `maxItems`.
    pub keyword: &'static str,
    /// Why, in one line of text.
    pub reason: String,
}

impl fmt::Display for ArgumentRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "arguments at `{}`: {}", self.pointer, self.reason)
    }
}

/// Why [`check_arguments`] refused a call (story:argument-validator).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgumentsRefused {
    /// The schema is outside the dialect: each pair is the JSON Pointer inside the schema and the
    /// reason, as `canon validate` reports it. No argument was checked.
    Schema(Vec<(String, String)>),
    /// The arguments do not satisfy the schema: every refusal, in document order. Never empty.
    Arguments(Vec<ArgumentRefusal>),
}

impl fmt::Display for ArgumentsRefused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgumentsRefused::Schema(offences) => {
                for (index, (pointer, reason)) in offences.iter().enumerate() {
                    let separator = if index == 0 { "" } else { "; " };
                    write!(f, "{separator}schema at `{pointer}`: {reason}")?;
                }
            }
            ArgumentsRefused::Arguments(refusals) => {
                for (index, refusal) in refusals.iter().enumerate() {
                    let separator = if index == 0 { "" } else { "; " };
                    write!(f, "{separator}{refusal}")?;
                }
            }
        }
        Ok(())
    }
}

impl std::error::Error for ArgumentsRefused {}

/// Checks one call's arguments against an action's argument schema: `Ok` when they satisfy it,
/// else every refusal. The schema is `Action::arguments`, `None` for an action that declares none,
/// which takes only `{}`. Pure; the module documentation states the meaning of each keyword and
/// the order of the refusals.
pub fn check_arguments(schema: Option<&Value>, arguments: &Value) -> Result<(), ArgumentsRefused> {
    let absent = json!({"type": "object", "additionalProperties": false});
    let schema = match schema {
        Some(schema) => {
            let offences = check(schema);
            if !offences.is_empty() {
                return Err(ArgumentsRefused::Schema(offences));
            }
            schema
        }
        None => &absent,
    };
    let mut walk = Walk::default();
    walk.value(schema, arguments);
    if walk.refusals.is_empty() {
        Ok(())
    } else {
        Err(ArgumentsRefused::Arguments(walk.refusals))
    }
}

/// The refusals so far, and the pointer of the value being checked.
#[derive(Default)]
struct Walk {
    pointer: String,
    refusals: Vec<ArgumentRefusal>,
}

impl Walk {
    fn refuse(&mut self, keyword: &'static str, reason: String) {
        self.refusals.push(ArgumentRefusal {
            pointer: self.pointer.clone(),
            keyword,
            reason,
        });
    }

    /// Checks `doc`, the member or item `token` of the current value, against `schema`.
    fn child(&mut self, token: &str, schema: &Value, doc: &Value) {
        let length = self.pointer.len();
        self.pointer.push('/');
        self.pointer
            .push_str(&token.replace('~', "~0").replace('/', "~1"));
        self.value(schema, doc);
        self.pointer.truncate(length);
    }

    /// Checks `doc` at the current pointer against `schema`, which `check` has found to be in the
    /// dialect: so every keyword read here is well formed, and the descent is at most
    /// [`MAX_SCHEMA_DEPTH`] deep.
    fn value(&mut self, schema: &Value, doc: &Value) {
        let Some(schema) = schema.as_object() else {
            return;
        };
        let Some(ty) = schema.get("type").and_then(Value::as_str) else {
            return;
        };
        if !is_type(doc, ty) {
            let reason = match doc {
                Value::Number(_) if ty == "integer" => {
                    "expected `integer`, found a number with a fractional part".to_owned()
                }
                _ => format!("expected `{ty}`, found {}", kind(doc)),
            };
            self.refuse("type", reason);
            return;
        }
        if let Some(Value::Array(values)) = schema.get("enum")
            && !values.iter().any(|value| same(value, doc))
        {
            self.refuse(
                "enum",
                "the value is not one of the `enum` values".to_owned(),
            );
        }
        match doc {
            Value::Number(number) => self.bounds(schema, number),
            Value::String(text) => {
                let length = text.chars().count() as u64;
                if let Some(maximum) = schema.get("maxLength").and_then(Value::as_u64)
                    && length > maximum
                {
                    self.refuse(
                        "maxLength",
                        format!("{length} code points is above `maxLength` {maximum}"),
                    );
                }
                if let Some(minimum) = schema.get("minLength").and_then(Value::as_u64)
                    && length < minimum
                {
                    self.refuse(
                        "minLength",
                        format!("{length} code points is below `minLength` {minimum}"),
                    );
                }
            }
            Value::Array(items) => self.array(schema, items),
            Value::Object(members) => self.object(schema, members),
            _ => {}
        }
    }

    fn bounds(&mut self, schema: &Map<String, Value>, number: &Number) {
        if let Some(Value::Number(maximum)) = schema.get("maximum")
            && compare(number, maximum) == Ordering::Greater
        {
            self.refuse("maximum", format!("{number} is above `maximum` {maximum}"));
        }
        if let Some(Value::Number(minimum)) = schema.get("minimum")
            && compare(number, minimum) == Ordering::Less
        {
            self.refuse("minimum", format!("{number} is below `minimum` {minimum}"));
        }
    }

    fn array(&mut self, schema: &Map<String, Value>, items: &[Value]) {
        let length = items.len() as u64;
        if let Some(maximum) = schema.get("maxItems").and_then(Value::as_u64)
            && length > maximum
        {
            self.refuse(
                "maxItems",
                format!("{length} items is above `maxItems` {maximum}"),
            );
        }
        if let Some(minimum) = schema.get("minItems").and_then(Value::as_u64)
            && length < minimum
        {
            self.refuse(
                "minItems",
                format!("{length} items is below `minItems` {minimum}"),
            );
        }
        if let Some(schema) = schema.get("items") {
            for (index, item) in items.iter().enumerate() {
                self.child(&index.to_string(), schema, item);
            }
        }
    }

    fn object(&mut self, schema: &Map<String, Value>, members: &Map<String, Value>) {
        if let Some(Value::Array(names)) = schema.get("required") {
            for name in names.iter().filter_map(Value::as_str) {
                if !members.contains_key(name) {
                    self.refuse(
                        "required",
                        format!("`{}` is required and missing", one_line(name)),
                    );
                }
            }
        }
        let declared = schema.get("properties").and_then(Value::as_object);
        let closed = schema.get("additionalProperties") == Some(&Value::Bool(false));
        let mut names: Vec<&String> = members.keys().collect();
        names.sort();
        for name in names {
            match declared.and_then(|properties| properties.get(name.as_str())) {
                Some(property) => self.child(name, property, &members[name.as_str()]),
                None if closed => {
                    let length = self.pointer.len();
                    self.pointer.push('/');
                    self.pointer
                        .push_str(&name.replace('~', "~0").replace('/', "~1"));
                    self.refuse(
                        "additionalProperties",
                        format!("`{}` is not declared in `properties`", one_line(name)),
                    );
                    self.pointer.truncate(length);
                }
                None => {}
            }
        }
    }
}

/// Whether a JSON value is of a dialect type: an integer is a number with no fractional part.
fn is_type(doc: &Value, ty: &str) -> bool {
    match (ty, doc) {
        ("object", Value::Object(_))
        | ("array", Value::Array(_))
        | ("string", Value::String(_))
        | ("number", Value::Number(_))
        | ("boolean", Value::Bool(_))
        | ("null", Value::Null) => true,
        ("integer", Value::Number(number)) => {
            integer_of(number).is_some() || double(number).fract() == 0.0
        }
        _ => false,
    }
}

/// What a JSON value is, for a reason.
fn kind(doc: &Value) -> &'static str {
    match doc {
        Value::Null => "`null`",
        Value::Bool(_) => "`boolean`",
        Value::Number(_) => "`number`",
        Value::String(_) => "`string`",
        Value::Array(_) => "`array`",
        Value::Object(_) => "`object`",
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{MAX_SCHEMA_DEPTH, check};

    /// A top-level object whose property `p` nests `levels` more object schemas.
    fn nested(levels: usize) -> Value {
        let mut schema = json!({"type": "object"});
        for _ in 0..levels {
            schema = json!({"type": "object", "properties": {"p": schema}});
        }
        schema
    }

    #[test]
    fn schemas_nest_within_the_bound_on_predicates() {
        assert_eq!(check(&nested(MAX_SCHEMA_DEPTH - 1)), Vec::new());
        let found = check(&nested(MAX_SCHEMA_DEPTH));
        assert_eq!(found.len(), 1, "{:?}", found.first());
        let (pointer, reason) = &found[0];
        assert_eq!(pointer, &"/properties/p".repeat(MAX_SCHEMA_DEPTH));
        assert_eq!(
            reason,
            &format!("the schema nests schemas deeper than {MAX_SCHEMA_DEPTH}")
        );
    }

    #[test]
    fn numbers_equal_in_value_are_one_enum_value() {
        let found = check(&json!({
            "type": "object",
            "properties": {"n": {"type": "number", "enum": [1, 1.0, 2.5]}}
        }));
        assert_eq!(
            found,
            vec![(
                "/properties/n/enum/1".to_owned(),
                "`1.0` is listed more than once".to_owned()
            )]
        );
    }
}
