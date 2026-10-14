//! Classification of a change to an action's argument schema (story:action-arguments).
//!
//! An action's `arguments` is a JSON Schema in Canon's argument-schema dialect, which the validator
//! has accepted. [`classify`] is a pure function of the two revisions' schemas, either absent, and
//! classifies a change by what it does to the argument documents a runner accepts:
//!
//! | Change | Category | Line |
//! |---|---|---|
//! | `arguments` introduced on an action | `BREAKING` | `arguments introduced` |
//! | `arguments` removed from an action | `BREAKING` | `arguments removed` |
//! | only `title` or `description` changed, anywhere in the schema | `NO SEMANTIC CHANGE` | `arguments rewritten` |
//! | the new schema accepts only argument documents the old one accepted | `TIGHTENING` | `arguments narrowed` |
//! | the old schema accepts only argument documents the new one accepts | `RELAXATION` | `arguments widened` |
//! | anything else, or both | `BREAKING` | `arguments changed` |
//!
//! An action without `arguments` takes only the empty object `{}`, and refuses any other
//! arguments. A schema introduced or removed is `BREAKING` whatever it accepts: the classifier
//! does not compare a schema with the empty object.
//!
//! Narrowing and widening are decided structurally and soundly, never guessed. Each keyword of a
//! schema constrains a document on its own, so the two schemas are compared keyword by keyword, and
//! each keyword that differs gives a direction:
//!
//! - `type`: `integer` to `number` widens and the reverse narrows; any other type change is
//!   changed, and the schemas' other keywords are not compared.
//! - `enum`: introduced, or a value removed, narrows; removed, or a value added, widens.
//! - A lower bound (`minimum`, `minLength`, `minItems`) introduced or raised narrows, removed or
//!   lowered widens; an upper bound (`maximum`, `maxLength`, `maxItems`) introduced or lowered
//!   narrows, removed or raised widens.
//! - `required`: a name added narrows, a name removed widens.
//! - An object's properties are compared name by name, over every name either revision declares,
//!   by their effective schemas: the declared schema, else "any value" when `additionalProperties`
//!   is absent or `true` and "no value" (the name must be absent) when it is `false`. Every other
//!   name is compared by `additionalProperties` alone, `true` or absent to `false` narrowing and
//!   the reverse widening. "Any value" to a schema narrows, "no value" to a schema widens, and
//!   "any value" to "no value" narrows; each the reverse way round, the reverse direction.
//! - An array's `items` is compared the same way, an absent `items` being "any value".
//! - Two declared schemas, a property's or the items', are compared by these rules, recursively.
//!
//! `title` and `description` give no direction. The directions of all keywords are combined: none
//! is `rewritten`, only narrowing is `narrowed`, only widening is `widened`, both is `changed`.
//! What the rules cannot show, such as a type changed to an unrelated one, a bound that is not an
//! integer or a keyword outside the dialect in a schema that differs, is `changed`.
//!
//! Two schemas are equal, and give no change, when they are equal as JSON values; a list whose
//! members are only reordered (`required`, `enum`) gives no direction, so it is `rewritten`.

#![allow(dead_code)]

use std::collections::BTreeSet;

use serde_json::{Map, Value};

use super::Category;

/// The category and line of a change to an action's `arguments`, or `None` when the two are equal.
pub(super) fn classify(
    from: Option<&Value>,
    to: Option<&Value>,
) -> Option<(Category, &'static str)> {
    match (from, to) {
        (None, None) => None,
        (None, Some(_)) => Some((Category::Breaking, "arguments introduced")),
        (Some(_), None) => Some((Category::Breaking, "arguments removed")),
        (Some(from), Some(to)) if from == to => None,
        (Some(from), Some(to)) => {
            let mut directions = Directions::default();
            directions.schema(from, to);
            Some(match directions {
                Directions {
                    undecided: true, ..
                }
                | Directions {
                    narrows: true,
                    widens: true,
                    ..
                } => (Category::Breaking, "arguments changed"),
                Directions { narrows: true, .. } => (Category::Tightening, "arguments narrowed"),
                Directions { widens: true, .. } => (Category::Relaxation, "arguments widened"),
                Directions { .. } => (Category::NoSemanticChange, "arguments rewritten"),
            })
        }
    }
}

/// Every keyword of the dialect; any other one in a schema that changed is undecided.
const DIALECT: [&str; 14] = [
    "type",
    "title",
    "description",
    "enum",
    "properties",
    "required",
    "additionalProperties",
    "items",
    "minItems",
    "maxItems",
    "minLength",
    "maxLength",
    "minimum",
    "maximum",
];

const LOWER_BOUNDS: [&str; 3] = ["minimum", "minLength", "minItems"];
const UPPER_BOUNDS: [&str; 3] = ["maximum", "maxLength", "maxItems"];

/// What a property name or an array's members may hold, by the effective schema of the module docs.
#[derive(Clone, Copy)]
enum Effective<'a> {
    Any,
    No,
    Schema(&'a Value),
}

/// The directions the keywords compared so far give, from the old schema to the new one.
#[derive(Debug, Default)]
struct Directions {
    /// The new schema accepts only what the old one did, for some keyword.
    narrows: bool,
    /// The old schema accepts only what the new one does, for some keyword.
    widens: bool,
    /// A keyword the rules cannot direct.
    undecided: bool,
}

impl Directions {
    /// Two schemas, keyword by keyword.
    fn schema(&mut self, from: &Value, to: &Value) {
        if from == to {
            return;
        }
        let (Some(from), Some(to)) = (from.as_object(), to.as_object()) else {
            self.undecided = true;
            return;
        };
        let outside =
            |schema: &Map<String, Value>| schema.keys().any(|key| !DIALECT.contains(&key.as_str()));
        if outside(from) || outside(to) {
            self.undecided = true;
            return;
        }
        match (types(from), types(to)) {
            (Some(old), Some(new)) if old == new => {}
            (Some("integer"), Some("number")) => self.widens = true,
            (Some("number"), Some("integer")) => self.narrows = true,
            _ => {
                self.undecided = true;
                return;
            }
        }
        self.enumeration(from.get("enum"), to.get("enum"));
        for key in LOWER_BOUNDS {
            self.bound(from.get(key), to.get(key), false);
        }
        for key in UPPER_BOUNDS {
            self.bound(from.get(key), to.get(key), true);
        }
        self.required(from.get("required"), to.get("required"));
        self.properties(from, to);
        self.effective(items(from), items(to));
    }

    /// `enum`: introduced or a value removed narrows, removed or a value added widens.
    fn enumeration(&mut self, from: Option<&Value>, to: Option<&Value>) {
        match (from, to) {
            (None, None) => {}
            (None, Some(_)) => self.narrows = true,
            (Some(_), None) => self.widens = true,
            (Some(from), Some(to)) => match (from.as_array(), to.as_array()) {
                (Some(from), Some(to)) => {
                    self.narrows |= from.iter().any(|value| !to.contains(value));
                    self.widens |= to.iter().any(|value| !from.contains(value));
                }
                _ => self.undecided = true,
            },
        }
    }

    /// A bound: introduced, or moved to admit less, narrows; removed, or moved to admit more,
    /// widens. An upper bound admits less when lowered, a lower bound when raised.
    fn bound(&mut self, from: Option<&Value>, to: Option<&Value>, upper: bool) {
        match (from, to) {
            (None, None) => {}
            (None, Some(_)) => self.narrows = true,
            (Some(_), None) => self.widens = true,
            (Some(from), Some(to)) => match (integer(from), integer(to)) {
                (Some(from), Some(to)) if from == to => {}
                (Some(from), Some(to)) => {
                    if (to > from) != upper {
                        self.narrows = true;
                    } else {
                        self.widens = true;
                    }
                }
                _ => self.undecided = true,
            },
        }
    }

    /// `required`: a name added narrows, a name removed widens.
    fn required(&mut self, from: Option<&Value>, to: Option<&Value>) {
        match (names(from), names(to)) {
            (Some(from), Some(to)) => {
                self.narrows |= !to.is_subset(&from);
                self.widens |= !from.is_subset(&to);
            }
            _ => self.undecided = true,
        }
    }

    /// Every property name either schema declares, by its effective schema, then every other name
    /// by `additionalProperties`.
    fn properties(&mut self, from: &Map<String, Value>, to: &Map<String, Value>) {
        let empty = Map::new();
        let (Some(from_declared), Some(to_declared)) =
            (declared(from, &empty), declared(to, &empty))
        else {
            self.undecided = true;
            return;
        };
        let (Some(from_rest), Some(to_rest)) = (additional(from), additional(to)) else {
            self.undecided = true;
            return;
        };
        let every: BTreeSet<&String> = from_declared.keys().chain(to_declared.keys()).collect();
        for name in every {
            let from = from_declared.get(name).map_or(from_rest, Effective::Schema);
            let to = to_declared.get(name).map_or(to_rest, Effective::Schema);
            self.effective(from, to);
        }
        self.effective(from_rest, to_rest);
    }

    /// Two effective schemas: "any value" admits most and "no value" least, a schema in between.
    fn effective(&mut self, from: Effective<'_>, to: Effective<'_>) {
        match (from, to) {
            (Effective::Any, Effective::Any) | (Effective::No, Effective::No) => {}
            (Effective::Any, _) | (_, Effective::No) => self.narrows = true,
            (_, Effective::Any) | (Effective::No, _) => self.widens = true,
            (Effective::Schema(from), Effective::Schema(to)) => self.schema(from, to),
        }
    }
}

/// The schema's one type, when it names one.
fn types(schema: &Map<String, Value>) -> Option<&str> {
    schema.get("type").and_then(Value::as_str)
}

/// A bound's value, when it is an integer.
fn integer(bound: &Value) -> Option<i128> {
    bound
        .as_i64()
        .map(i128::from)
        .or_else(|| bound.as_u64().map(i128::from))
}

/// The names of a `required`, none when absent; `None` when it is not a list of names.
fn names(required: Option<&Value>) -> Option<BTreeSet<&str>> {
    match required {
        None => Some(BTreeSet::new()),
        Some(required) => required.as_array()?.iter().map(Value::as_str).collect(),
    }
}

/// The schema's declared properties, none when absent; `None` when they are not a map.
fn declared<'a>(
    schema: &'a Map<String, Value>,
    empty: &'a Map<String, Value>,
) -> Option<&'a Map<String, Value>> {
    match schema.get("properties") {
        None => Some(empty),
        Some(properties) => properties.as_object(),
    }
}

/// What a name the schema does not declare may hold; `None` when `additionalProperties` is not
/// a boolean.
fn additional(schema: &Map<String, Value>) -> Option<Effective<'static>> {
    match schema.get("additionalProperties") {
        None | Some(Value::Bool(true)) => Some(Effective::Any),
        Some(Value::Bool(false)) => Some(Effective::No),
        Some(_) => None,
    }
}

/// What an array's members may hold: `items`, else any value.
fn items(schema: &Map<String, Value>) -> Effective<'_> {
    schema
        .get("items")
        .map_or(Effective::Any, Effective::Schema)
}

#[cfg(test)]
mod tests;
