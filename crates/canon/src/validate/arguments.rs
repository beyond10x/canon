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
//! - Schemas nest at most [`MAX_IR_DEPTH`] deep, the bound on predicates: the top level is one
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

use std::collections::BTreeSet;

use serde_json::{Map, Value};

use crate::eval::MAX_IR_DEPTH;

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
    if depth > MAX_IR_DEPTH {
        return vec![offence(
            at,
            format!("the schema nests schemas deeper than {MAX_IR_DEPTH}"),
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

/// JSON Schema's equality of two scalars: numbers by value, so `1` and `1.0` are one value.
fn same(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(left), Value::Number(right)) => {
            match (integer_of(left), integer_of(right)) {
                (Some(left), Some(right)) => left == right,
                _ => left.as_f64() == right.as_f64(),
            }
        }
        _ => left == right,
    }
}

fn integer_of(number: &serde_json::Number) -> Option<i128> {
    number
        .as_i64()
        .map(i128::from)
        .or_else(|| number.as_u64().map(i128::from))
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

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{MAX_IR_DEPTH, check};

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
        assert_eq!(check(&nested(MAX_IR_DEPTH - 1)), Vec::new());
        let found = check(&nested(MAX_IR_DEPTH));
        assert_eq!(found.len(), 1, "{:?}", found.first());
        let (pointer, reason) = &found[0];
        assert_eq!(pointer, &"/properties/p".repeat(MAX_IR_DEPTH));
        assert_eq!(
            reason,
            &format!("the schema nests schemas deeper than {MAX_IR_DEPTH}")
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
