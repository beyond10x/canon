//! Every row of the module's table and each keyword rule in both directions. Every narrowed or
//! widened case is spot-checked for soundness with a small checker of the dialect ([`accepts`]): a
//! witness document is accepted by the wider schema and refused by the narrower one, and no
//! document of a fixed corpus is accepted by the narrower schema and refused by the wider one.

use super::classify;
use crate::diff::Category;
use serde_json::{Value, json};

const NARROWED: (Category, &str) = (Category::Tightening, "arguments narrowed");
const WIDENED: (Category, &str) = (Category::Relaxation, "arguments widened");
const CHANGED: (Category, &str) = (Category::Breaking, "arguments changed");
const REWRITTEN: (Category, &str) = (Category::NoSemanticChange, "arguments rewritten");

/// Whether `schema`, in the dialect, accepts `doc`, by JSON Schema 2020-12's meaning.
fn accepts(schema: &Value, doc: &Value) -> bool {
    let s = schema.as_object().expect("a schema is an object");
    let typed = match s["type"].as_str().expect("a schema names one type") {
        "object" => doc.is_object(),
        "array" => doc.is_array(),
        "string" => doc.is_string(),
        "integer" => doc.as_f64().is_some_and(|n| n.fract() == 0.0),
        "number" => doc.is_number(),
        "boolean" => doc.is_boolean(),
        "null" => doc.is_null(),
        other => panic!("type {other} is outside the dialect"),
    };
    if !typed {
        return false;
    }
    if let Some(values) = s.get("enum") {
        let found = values.as_array().unwrap().iter().any(|v| json_eq(v, doc));
        if !found {
            return false;
        }
    }
    let bound = |key: &str| s.get(key).map(|b| b.as_f64().unwrap());
    if let Some(n) = doc.as_f64() {
        if bound("minimum").is_some_and(|m| n < m) || bound("maximum").is_some_and(|m| n > m) {
            return false;
        }
    }
    if let Some(text) = doc.as_str() {
        let len = text.chars().count() as f64;
        if bound("minLength").is_some_and(|m| len < m)
            || bound("maxLength").is_some_and(|m| len > m)
        {
            return false;
        }
    }
    if let Some(items) = doc.as_array() {
        let len = items.len() as f64;
        if bound("minItems").is_some_and(|m| len < m) || bound("maxItems").is_some_and(|m| len > m)
        {
            return false;
        }
        if let Some(item) = s.get("items") {
            if !items.iter().all(|i| accepts(item, i)) {
                return false;
            }
        }
    }
    if let Some(members) = doc.as_object() {
        let empty = serde_json::Map::new();
        let properties = s
            .get("properties")
            .map_or(&empty, |p| p.as_object().unwrap());
        let additional = s
            .get("additionalProperties")
            .is_none_or(|a| a.as_bool().unwrap());
        if let Some(required) = s.get("required") {
            let present = required.as_array().unwrap().iter();
            if !present
                .map(|n| n.as_str().unwrap())
                .all(|n| members.contains_key(n))
            {
                return false;
            }
        }
        for (name, value) in members {
            let fine = match properties.get(name) {
                Some(property) => accepts(property, value),
                None => additional,
            };
            if !fine {
                return false;
            }
        }
    }
    true
}

/// JSON equality as JSON Schema reads it: numbers by value.
fn json_eq(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(l), Value::Number(r)) => l.as_f64() == r.as_f64(),
        (Value::Array(l), Value::Array(r)) => {
            l.len() == r.len() && l.iter().zip(r).all(|(l, r)| json_eq(l, r))
        }
        (Value::Object(l), Value::Object(r)) => {
            l.len() == r.len()
                && l.iter()
                    .all(|(k, v)| r.get(k).is_some_and(|w| json_eq(v, w)))
        }
        _ => left == right,
    }
}

/// Every object over the names `a`, `b` and `c`, each absent or one of a pool of values.
fn corpus() -> Vec<Value> {
    let pool = [
        json!(null),
        json!(true),
        json!(-1),
        json!(0),
        json!(1),
        json!(2),
        json!(5),
        json!(10),
        json!(1.5),
        json!(""),
        json!("a"),
        json!("abcd"),
        json!([]),
        json!([1]),
        json!([1, 2, 3]),
        json!([5, 10]),
        json!(["a", "b"]),
        json!({}),
        json!({"x": 1}),
        json!({"x": 1.5}),
        json!({"x": "s"}),
        json!({"y": 1}),
        json!([{"x": 1}]),
    ];
    let choices: Vec<Option<&Value>> = std::iter::once(None).chain(pool.iter().map(Some)).collect();
    let mut docs = vec![json!(null), json!(1), json!("a"), json!([])];
    for a in &choices {
        for b in &choices {
            for c in &choices {
                let mut doc = serde_json::Map::new();
                for (name, value) in [("a", a), ("b", b), ("c", c)] {
                    if let Some(value) = value {
                        doc.insert(name.to_owned(), (*value).clone());
                    }
                }
                docs.push(Value::Object(doc));
            }
        }
    }
    docs
}

/// `narrow` narrows `wide` and `wide` widens `narrow`: classified so both ways, `witness` accepted
/// by `wide` and refused by `narrow`, and no corpus document accepted by `narrow` alone.
fn narrows(wide: Value, narrow: Value, witness: Value) {
    assert_eq!(
        classify(Some(&wide), Some(&narrow)),
        Some(NARROWED),
        "{wide} to {narrow}"
    );
    assert_eq!(
        classify(Some(&narrow), Some(&wide)),
        Some(WIDENED),
        "{narrow} to {wide}"
    );
    assert!(
        accepts(&wide, &witness),
        "{wide} refuses the witness {witness}"
    );
    assert!(
        !accepts(&narrow, &witness),
        "{narrow} accepts the witness {witness}"
    );
    for doc in corpus() {
        assert!(
            !accepts(&narrow, &doc) || accepts(&wide, &doc),
            "{narrow} accepts {doc}, which {wide} refuses"
        );
    }
}

/// `left` and `right` are changed, either way round.
fn changes(left: Value, right: Value) {
    assert_eq!(
        classify(Some(&left), Some(&right)),
        Some(CHANGED),
        "{left} to {right}"
    );
    assert_eq!(
        classify(Some(&right), Some(&left)),
        Some(CHANGED),
        "{right} to {left}"
    );
}

/// `left` and `right` are rewritten, either way round, and accept the same corpus documents.
fn rewrites(left: Value, right: Value) {
    assert_eq!(
        classify(Some(&left), Some(&right)),
        Some(REWRITTEN),
        "{left} to {right}"
    );
    assert_eq!(
        classify(Some(&right), Some(&left)),
        Some(REWRITTEN),
        "{right} to {left}"
    );
    for doc in corpus() {
        assert_eq!(
            accepts(&left, &doc),
            accepts(&right, &doc),
            "{left}, {right} on {doc}"
        );
    }
}

/// A top-level object schema whose property `a` has the schema `a`.
fn with_a(a: Value) -> Value {
    json!({"type": "object", "properties": {"a": a}})
}

// The table, row by row.

#[test]
fn absent_on_both_sides_is_no_change() {
    assert_eq!(classify(None, None), None);
}

#[test]
fn equal_schemas_are_no_change() {
    let schema = with_a(json!({"type": "integer", "minimum": 0, "description": "count"}));
    assert_eq!(classify(Some(&schema), Some(&schema.clone())), None);
}

#[test]
fn arguments_introduced_breaks() {
    let schema = json!({"type": "object"});
    assert_eq!(
        classify(None, Some(&schema)),
        Some((Category::Breaking, "arguments introduced"))
    );
}

#[test]
fn arguments_removed_breaks() {
    let schema = json!({"type": "object"});
    assert_eq!(
        classify(Some(&schema), None),
        Some((Category::Breaking, "arguments removed"))
    );
}

#[test]
fn title_or_description_anywhere_is_rewritten() {
    rewrites(
        json!({"type": "object", "title": "old"}),
        json!({"type": "object", "title": "new", "description": "added"}),
    );
    rewrites(
        with_a(json!({"type": "string", "description": "old"})),
        with_a(json!({"type": "string", "description": "new"})),
    );
    rewrites(
        with_a(json!({"type": "array", "items": {"type": "integer", "title": "old"}})),
        with_a(json!({"type": "array", "items": {"type": "integer"}})),
    );
    rewrites(
        with_a(json!({"type": "object", "properties": {"x": {"type": "null"}}})),
        with_a(json!({"type": "object", "properties": {"x": {"type": "null", "title": "x"}}})),
    );
}

#[test]
fn reordered_required_or_enum_is_rewritten() {
    rewrites(
        json!({"type": "object", "properties": {"a": {"type": "null"}, "b": {"type": "null"}},
               "required": ["a", "b"]}),
        json!({"type": "object", "properties": {"a": {"type": "null"}, "b": {"type": "null"}},
               "required": ["b", "a"]}),
    );
    rewrites(
        with_a(json!({"type": "integer", "enum": [1, 2]})),
        with_a(json!({"type": "integer", "enum": [2, 1]})),
    );
}

#[test]
fn additional_properties_true_is_absent() {
    rewrites(
        json!({"type": "object", "additionalProperties": true}),
        json!({"type": "object"}),
    );
}

// `type`.

#[test]
fn integer_to_number_widens_and_number_to_integer_narrows() {
    narrows(
        with_a(json!({"type": "number"})),
        with_a(json!({"type": "integer"})),
        json!({"a": 1.5}),
    );
}

#[test]
fn any_other_type_change_is_changed() {
    changes(
        with_a(json!({"type": "string"})),
        with_a(json!({"type": "integer"})),
    );
    changes(
        with_a(json!({"type": "object"})),
        with_a(json!({"type": "array"})),
    );
    changes(
        with_a(json!({"type": "null"})),
        with_a(json!({"type": "boolean"})),
    );
}

// `enum`.

#[test]
fn enum_introduced_narrows_and_removed_widens() {
    narrows(
        with_a(json!({"type": "integer"})),
        with_a(json!({"type": "integer", "enum": [1, 2]})),
        json!({"a": 5}),
    );
}

#[test]
fn enum_value_removed_narrows_and_added_widens() {
    narrows(
        with_a(json!({"type": "string", "enum": ["a", "abcd"]})),
        with_a(json!({"type": "string", "enum": ["a"]})),
        json!({"a": "abcd"}),
    );
}

#[test]
fn enum_value_added_and_another_removed_is_changed() {
    changes(
        with_a(json!({"type": "integer", "enum": [1, 2]})),
        with_a(json!({"type": "integer", "enum": [1, 5]})),
    );
}

// Bounds.

/// Each lower bound: introduced or raised narrows, removed or lowered widens.
#[test]
fn lower_bounds_introduced_or_raised_narrow() {
    for (ty, key, low, high, witness) in [
        ("integer", "minimum", 1, 2, json!(1)),
        ("number", "minimum", -1, 2, json!(1.5)),
        ("string", "minLength", 0, 2, json!("a")),
        ("array", "minItems", 1, 2, json!([1])),
    ] {
        let doc = json!({"a": witness});
        narrows(
            with_a(json!({"type": ty})),
            with_a(json!({"type": ty, key: high})),
            doc.clone(),
        );
        narrows(
            with_a(json!({"type": ty, key: low})),
            with_a(json!({"type": ty, key: high})),
            doc,
        );
    }
}

/// Each upper bound: introduced or lowered narrows, removed or raised widens.
#[test]
fn upper_bounds_introduced_or_lowered_narrow() {
    for (ty, key, high, low, witness) in [
        ("integer", "maximum", 10, 2, json!(5)),
        ("number", "maximum", 2, 1, json!(1.5)),
        ("string", "maxLength", 4, 1, json!("abcd")),
        ("array", "maxItems", 3, 1, json!([5, 10])),
    ] {
        let doc = json!({"a": witness});
        narrows(
            with_a(json!({"type": ty})),
            with_a(json!({"type": ty, key: low})),
            doc.clone(),
        );
        narrows(
            with_a(json!({"type": ty, key: high})),
            with_a(json!({"type": ty, key: low})),
            doc,
        );
    }
}

#[test]
fn one_bound_raised_and_the_other_lowered_or_raised() {
    narrows(
        with_a(json!({"type": "integer", "minimum": 0, "maximum": 10})),
        with_a(json!({"type": "integer", "minimum": 1, "maximum": 5})),
        json!({"a": 10}),
    );
    changes(
        with_a(json!({"type": "integer", "minimum": 0, "maximum": 5})),
        with_a(json!({"type": "integer", "minimum": 1, "maximum": 10})),
    );
    changes(
        with_a(json!({"type": "string", "minLength": 1})),
        with_a(json!({"type": "string", "maxLength": 3})),
    );
}

#[test]
fn integer_to_number_with_a_bound_raised_is_changed() {
    changes(
        with_a(json!({"type": "integer", "minimum": 0})),
        with_a(json!({"type": "number", "minimum": 1})),
    );
}

#[test]
fn a_bound_that_is_not_an_integer_is_changed() {
    changes(
        with_a(json!({"type": "number", "minimum": 0})),
        with_a(json!({"type": "number", "minimum": 0.5})),
    );
}

// `required`.

#[test]
fn required_name_added_narrows_and_removed_widens() {
    let properties = json!({"a": {"type": "integer"}, "b": {"type": "string"}});
    narrows(
        json!({"type": "object", "properties": properties, "required": ["a"]}),
        json!({"type": "object", "properties": properties, "required": ["a", "b"]}),
        json!({"a": 1}),
    );
    narrows(
        json!({"type": "object", "properties": properties}),
        json!({"type": "object", "properties": properties, "required": ["b"]}),
        json!({}),
    );
}

#[test]
fn required_name_added_and_another_removed_is_changed() {
    let properties = json!({"a": {"type": "integer"}, "b": {"type": "string"}});
    changes(
        json!({"type": "object", "properties": properties, "required": ["a"]}),
        json!({"type": "object", "properties": properties, "required": ["b"]}),
    );
}

// Effective schemas of an object's properties.

#[test]
fn property_declared_where_any_value_was_accepted_narrows() {
    narrows(
        json!({"type": "object"}),
        json!({"type": "object", "properties": {"a": {"type": "integer"}}}),
        json!({"a": "a"}),
    );
    narrows(
        json!({"type": "object", "additionalProperties": true}),
        json!({"type": "object", "properties": {"a": {"type": "integer"}},
               "additionalProperties": true}),
        json!({"a": "a"}),
    );
}

#[test]
fn property_declared_where_no_value_was_accepted_widens() {
    narrows(
        json!({"type": "object", "properties": {"a": {"type": "integer"}},
               "additionalProperties": false}),
        json!({"type": "object", "additionalProperties": false}),
        json!({"a": 1}),
    );
}

#[test]
fn additional_properties_false_narrows_and_true_widens() {
    let properties = json!({"a": {"type": "integer"}});
    narrows(
        json!({"type": "object", "properties": properties, "additionalProperties": true}),
        json!({"type": "object", "properties": properties, "additionalProperties": false}),
        json!({"a": 1, "b": 1}),
    );
    narrows(
        json!({"type": "object", "properties": properties}),
        json!({"type": "object", "properties": properties, "additionalProperties": false}),
        json!({"b": "a"}),
    );
}

/// The undeclared property becomes declared while `additionalProperties` becomes `false`: `a` is
/// "any value" to a schema, which narrows, and every other name is narrowed too.
#[test]
fn property_declared_as_additional_properties_closes() {
    narrows(
        json!({"type": "object"}),
        json!({"type": "object", "properties": {"a": {"type": "integer"}},
               "additionalProperties": false}),
        json!({"c": 1}),
    );
}

/// A property that had to be absent becomes required: neither schema accepts what the other does.
#[test]
fn required_property_added_where_no_value_was_accepted_is_changed() {
    changes(
        json!({"type": "object", "properties": {"b": {"type": "null"}},
               "additionalProperties": false}),
        json!({"type": "object", "properties": {"a": {"type": "integer"}, "b": {"type": "null"}},
               "required": ["a"], "additionalProperties": false}),
    );
}

/// One property narrowed and another widened is both.
#[test]
fn two_properties_in_opposite_directions_are_changed() {
    changes(
        json!({"type": "object", "properties": {"a": {"type": "integer"}, "b": {"type": "number"}}}),
        json!({"type": "object", "properties": {"a": {"type": "number"}, "b": {"type": "integer"}}}),
    );
}

// Recursion into `items` and properties.

#[test]
fn nested_change_inside_items() {
    narrows(
        with_a(json!({"type": "array", "items": {"type": "integer"}})),
        with_a(json!({"type": "array", "items": {"type": "integer", "minimum": 2}})),
        json!({"a": [1]}),
    );
    changes(
        with_a(json!({"type": "array", "items": {"type": "integer"}})),
        with_a(json!({"type": "array", "items": {"type": "string"}})),
    );
}

#[test]
fn items_declared_where_any_value_was_accepted_narrows() {
    narrows(
        with_a(json!({"type": "array"})),
        with_a(json!({"type": "array", "items": {"type": "string"}})),
        json!({"a": [1]}),
    );
}

#[test]
fn nested_change_inside_a_property() {
    narrows(
        with_a(json!({"type": "object", "properties": {"x": {"type": "number"}}})),
        with_a(json!({"type": "object", "properties": {"x": {"type": "integer"}}})),
        json!({"a": {"x": 1.5}}),
    );
    narrows(
        with_a(json!({"type": "array", "items": {"type": "object"}})),
        with_a(json!({"type": "array", "items": {"type": "object",
                      "properties": {"x": {"type": "integer"}}, "required": ["x"]}})),
        json!({"a": [{"y": 1}]}),
    );
}

#[test]
fn nested_rewrite_beside_a_narrowing_is_narrowed() {
    narrows(
        with_a(json!({"type": "object", "description": "old",
                      "properties": {"x": {"type": "integer"}}})),
        with_a(json!({"type": "object", "description": "new",
                      "properties": {"x": {"type": "integer", "maximum": 1}}})),
        json!({"a": {"x": 5}}),
    );
}

// The acceptance's cases, on one schema.

#[test]
fn acceptance_cases() {
    let base = json!({"type": "object", "properties": {"a": {"type": "integer"}},
                      "additionalProperties": false, "description": "base"});
    let required_added = json!({"type": "object",
        "properties": {"a": {"type": "integer"}, "b": {"type": "string"}},
        "required": ["b"], "additionalProperties": true, "description": "base"});
    let opened = json!({"type": "object", "properties": {"a": {"type": "integer"}},
                        "additionalProperties": true, "description": "base"});
    let open = json!({"type": "object", "properties": {"a": {"type": "integer"}},
                      "description": "base"});
    let open_required = json!({"type": "object",
        "properties": {"a": {"type": "integer"}, "b": {"type": "string"}},
        "required": ["b"], "description": "base"});
    let described = json!({"type": "object", "properties": {"a": {"type": "integer"}},
                           "additionalProperties": false, "description": "changed"});
    assert_eq!(classify(Some(&open), Some(&open_required)), Some(NARROWED));
    assert_eq!(classify(Some(&base), Some(&opened)), Some(WIDENED));
    assert_eq!(
        classify(Some(&opened), Some(&required_added)),
        Some(NARROWED)
    );
    assert_eq!(classify(Some(&base), Some(&required_added)), Some(CHANGED));
    assert_eq!(classify(Some(&base), Some(&described)), Some(REWRITTEN));
}

// What the rules cannot show.

#[test]
fn a_keyword_outside_the_dialect_in_a_changed_schema_is_changed() {
    changes(
        with_a(json!({"type": "string", "pattern": "^a"})),
        with_a(json!({"type": "string", "pattern": "^a", "maxLength": 3})),
    );
}

// The checker itself.

#[test]
fn checker_reads_the_dialect() {
    let schema = json!({"type": "object", "properties": {
        "a": {"type": "integer", "minimum": 0, "maximum": 5},
        "b": {"type": "array", "items": {"type": "string", "maxLength": 1}, "maxItems": 2}},
        "required": ["a"], "additionalProperties": false});
    assert!(accepts(&schema, &json!({"a": 5, "b": ["a"]})));
    assert!(accepts(&schema, &json!({"a": 1.0})));
    assert!(!accepts(&schema, &json!({"a": 6})));
    assert!(!accepts(&schema, &json!({"a": 1.5})));
    assert!(!accepts(&schema, &json!({"b": []})));
    assert!(!accepts(&schema, &json!({"a": 1, "b": ["ab"]})));
    assert!(!accepts(&schema, &json!({"a": 1, "c": null})));
    assert!(!accepts(&schema, &json!([])));
}
