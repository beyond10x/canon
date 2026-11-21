//! Fixed-seed generators of schemas in the argument-schema dialect and of JSON documents, and
//! `accepts`, the test oracle: JSON Schema 2020-12's meaning of a schema in the dialect, written
//! apart from the library (story:action-arguments, story:argument-validator).

#![allow(dead_code)]

use serde_json::{Value, json};

/// A small linear congruential generator, fixed seed.
pub struct Rng(pub u64);

impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }
    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    pub fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }
    pub fn subset(&mut self, pool: &[Value]) -> Vec<Value> {
        let picked: Vec<Value> = pool.iter().filter(|_| self.chance(50)).cloned().collect();
        if picked.is_empty() {
            vec![pool[self.below(pool.len() as u64) as usize].clone()]
        } else {
            picked
        }
    }
}

pub const TYPES: [&str; 7] = [
    "object", "array", "string", "integer", "number", "boolean", "null",
];

pub fn enum_pool(ty: &str) -> Vec<Value> {
    match ty {
        "integer" => vec![json!(0), json!(1), json!(2), json!(3)],
        "number" => vec![json!(0), json!(1), json!(1.5), json!(3)],
        "string" => vec![json!(""), json!("a"), json!("ab"), json!("abc")],
        "boolean" => vec![json!(true), json!(false)],
        "null" => vec![json!(null)],
        _ => Vec::new(),
    }
}

/// Re-rolls (adds, changes or removes) `keyword` of `schema`.
pub fn roll(
    rng: &mut Rng,
    schema: &mut serde_json::Map<String, Value>,
    keyword: &str,
    depth: usize,
) {
    let ty = schema["type"].as_str().unwrap().to_owned();
    if rng.chance(35) {
        schema.remove(keyword);
        return;
    }
    let value = match keyword {
        "enum" => Value::Array(rng.subset(&enum_pool(&ty))),
        "minimum" | "maximum" | "minLength" | "maxLength" | "minItems" | "maxItems" => {
            json!(rng.below(4))
        }
        "items" => generate(rng, depth + 1, false),
        "properties" => {
            let mut properties = serde_json::Map::new();
            for name in ["a", "b"] {
                if rng.chance(60) {
                    properties.insert(name.to_owned(), generate(rng, depth + 1, false));
                }
            }
            Value::Object(properties)
        }
        "required" => {
            let declared: Vec<Value> = schema
                .get("properties")
                .and_then(Value::as_object)
                .map(|p| p.keys().map(|k| json!(k)).collect())
                .unwrap_or_default();
            Value::Array(declared.into_iter().filter(|_| rng.chance(50)).collect())
        }
        "additionalProperties" => json!(rng.chance(50)),
        "title" => json!(format!("t{}", rng.below(3))),
        other => panic!("{other}"),
    };
    schema.insert(keyword.to_owned(), value);
}

pub fn keywords(ty: &str, depth: usize) -> Vec<&'static str> {
    let mut all = vec!["title"];
    match ty {
        "integer" | "number" => all.extend(["enum", "minimum", "maximum"]),
        "string" => all.extend(["enum", "minLength", "maxLength"]),
        "boolean" | "null" => all.push("enum"),
        "array" if depth < 3 => all.extend(["items", "minItems", "maxItems"]),
        "array" => all.extend(["minItems", "maxItems"]),
        "object" if depth < 3 => all.extend(["properties", "required", "additionalProperties"]),
        "object" => all.push("additionalProperties"),
        _ => {}
    }
    all
}

pub fn generate(rng: &mut Rng, depth: usize, top: bool) -> Value {
    let ty = if top {
        "object"
    } else {
        TYPES[rng.below(7) as usize]
    };
    let mut schema = serde_json::Map::new();
    schema.insert("type".to_owned(), json!(ty));
    for keyword in keywords(ty, depth) {
        if rng.chance(45) {
            roll(rng, &mut schema, keyword, depth);
        }
    }
    if schema.contains_key("required") {
        roll(rng, &mut schema, "required", depth);
    }
    Value::Object(schema)
}

/// `schema` with a few keywords re-rolled, recursively, and now and then `integer` and `number`
/// swapped.
pub fn mutate(rng: &mut Rng, schema: &Value, depth: usize, top: bool) -> Value {
    let mut schema = schema.as_object().unwrap().clone();
    let ty = schema["type"].as_str().unwrap().to_owned();
    if !top && rng.chance(8) {
        return generate(rng, depth, false);
    }
    if (ty == "integer" || ty == "number") && rng.chance(15) {
        schema.insert(
            "type".to_owned(),
            json!(if ty == "integer" { "number" } else { "integer" }),
        );
    }
    for keyword in keywords(&ty, depth) {
        if rng.chance(20) {
            roll(rng, &mut schema, keyword, depth);
        } else if keyword == "properties"
            && let Some(Value::Object(properties)) = schema.get("properties").cloned()
        {
            let mutated = properties
                .iter()
                .map(|(name, property)| (name.clone(), mutate(rng, property, depth + 1, false)))
                .collect();
            schema.insert("properties".to_owned(), Value::Object(mutated));
        } else if keyword == "items"
            && let Some(items) = schema.get("items").cloned()
        {
            schema.insert("items".to_owned(), mutate(rng, &items, depth + 1, false));
        }
    }
    if schema.contains_key("required") && rng.chance(30) {
        roll(rng, &mut schema, "required", depth);
    }
    Value::Object(schema)
}

/// A random JSON value over a small domain: `2.0` among the numbers, since JSON Schema counts it
/// an integer.
pub fn document(rng: &mut Rng, depth: usize) -> Value {
    match rng.below(if depth > 2 { 6 } else { 8 }) {
        0 => json!(null),
        1 => json!(rng.chance(50)),
        2 => json!(rng.below(5) as i64 - 1),
        3 => [json!(1.5), json!(2.0), json!(0.5)][rng.below(3) as usize].clone(),
        4 | 5 => json!(["", "a", "ab", "abc", "abcd"][rng.below(5) as usize]),
        6 => Value::Array(
            (0..rng.below(4))
                .map(|_| document(rng, depth + 1))
                .collect(),
        ),
        _ => object_document(rng, depth + 1),
    }
}

pub fn object_document(rng: &mut Rng, depth: usize) -> Value {
    let mut object = serde_json::Map::new();
    for name in ["a", "b", "c"] {
        if rng.chance(55) {
            object.insert(name.to_owned(), document(rng, depth));
        }
    }
    Value::Object(object)
}

/// JSON Schema 2020-12's meaning of a schema in the dialect.
pub fn accepts(schema: &Value, doc: &Value) -> bool {
    let s = schema.as_object().unwrap();
    let typed = match s["type"].as_str().unwrap() {
        "object" => doc.is_object(),
        "array" => doc.is_array(),
        "string" => doc.is_string(),
        "integer" => doc.as_f64().is_some_and(|n| n.fract() == 0.0),
        "number" => doc.is_number(),
        "boolean" => doc.is_boolean(),
        _ => doc.is_null(),
    };
    if !typed {
        return false;
    }
    if let Some(values) = s.get("enum")
        && !values
            .as_array()
            .unwrap()
            .iter()
            .any(|value| match (value.as_f64(), doc.as_f64()) {
                (Some(left), Some(right)) => left == right,
                _ => value == doc,
            })
    {
        return false;
    }
    let bound = |key: &str| s.get(key).map(|b| b.as_f64().unwrap());
    let outside = |n: f64, low: &str, high: &str| {
        bound(low).is_some_and(|m| n < m) || bound(high).is_some_and(|m| n > m)
    };
    if let Some(n) = doc.as_f64()
        && outside(n, "minimum", "maximum")
    {
        return false;
    }
    if let Some(text) = doc.as_str()
        && outside(text.chars().count() as f64, "minLength", "maxLength")
    {
        return false;
    }
    if let Some(items) = doc.as_array() {
        if outside(items.len() as f64, "minItems", "maxItems") {
            return false;
        }
        if let Some(item) = s.get("items")
            && !items.iter().all(|i| accepts(item, i))
        {
            return false;
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
        if let Some(required) = s.get("required")
            && !required
                .as_array()
                .unwrap()
                .iter()
                .all(|n| members.contains_key(n.as_str().unwrap()))
        {
            return false;
        }
        for (name, value) in members {
            let fine = properties
                .get(name)
                .map_or(additional, |p| accepts(p, value));
            if !fine {
                return false;
            }
        }
    }
    true
}
