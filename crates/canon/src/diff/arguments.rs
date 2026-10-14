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

use super::Category;
use serde_json::Value;

/// The category and line of a change to an action's `arguments`, or `None` when the two are equal.
pub(super) fn classify(
    from: Option<&Value>,
    to: Option<&Value>,
) -> Option<(Category, &'static str)> {
    let _ = (from, to);
    todo!("story:action-arguments")
}

#[cfg(test)]
mod tests;
