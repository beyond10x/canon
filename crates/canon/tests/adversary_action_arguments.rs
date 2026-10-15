//! Adversarial probes of story:action-arguments: a schema `canon compile` writes is read back by
//! `read_ir` whatever finite number it holds, and the published `protocol/1` JSON Schema carries
//! the field the validator accepts.

use std::fs;
use std::path::PathBuf;

use b10x_canon::diff;
use b10x_canon::eval;
use b10x_canon::ir;
use b10x_canon::model;
use serde_json::{Value, json};

/// The repository root, read at run time.
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set"))
        .join("../..")
}

/// A protocol with one action whose argument `n` is a number with the one-value enum `value`.
fn with_number(value: &str) -> String {
    format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nactions:\n  act:\n    arguments:\n      \
         type: object\n      properties:\n        n:\n          type: number\n          enum: [{value}]\n"
    )
}

/// A fixed-seed stream of finite doubles: every bit pattern an LCG gives that is finite, beside
/// decimals a schema author writes.
fn doubles() -> Vec<f64> {
    let mut values = vec![
        0.1,
        0.3,
        1.5,
        2.5e-3,
        0.30000000000000004,
        1.2345678901234567,
    ];
    let mut state: u64 = 0x2545_f491_4f6c_dd1d;
    while values.len() < 4000 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let value = f64::from_bits(state);
        if value.is_finite() && value != 0.0 {
            values.push(value);
        }
    }
    for k in 1..2000u32 {
        values.push(f64::from(k) / 7.0 * 10f64.powi((k % 40) as i32 - 20));
    }
    values
}

/// `read_ir` accepts exactly the bytes `canon compile` prints (eval/read.rs:4-7). A number in an
/// argument schema is written as `serde_json` writes the double YAML read, and `read_ir` parses it
/// back with `serde_json`; every compiled IR must read back.
#[test]
fn every_finite_number_in_a_schema_reads_back() {
    let mut failures = Vec::new();
    for value in doubles() {
        let written = format!("{value:e}");
        let read: f64 = serde_yaml_ng::from_str(&written).expect("YAML reads the number");
        assert_eq!(
            read.to_bits(),
            value.to_bits(),
            "YAML reads {written} exactly"
        );
        let protocol = model::parse(&with_number(&written)).expect("parses");
        let text = ir::compile(&protocol)
            .unwrap_or_else(|problems| panic!("{written} compiles: {problems:?}"))
            .canonical_json();
        if let Err(refusal) = eval::read_ir(&text) {
            let line = text
                .lines()
                .find(|line| {
                    line.trim_start()
                        .starts_with(|c: char| c == '-' || c.is_ascii_digit())
                })
                .unwrap_or("")
                .trim()
                .to_owned();
            failures.push(format!("{written} (written {line}): {refusal:?}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} compiled IRs read_ir refuses; first: {:#?}",
        failures.len(),
        &failures[..failures.len().min(3)]
    );
}

/// The same, for numbers a schema author writes as they are: physical constants and plain decimals
/// in an `enum`. A protocol whose IR `read_ir` refuses cannot be evaluated at all.
#[test]
fn a_plainly_written_number_in_a_schema_reads_back() {
    let refused: Vec<&str> = [
        "0.1",
        "0.7",
        "3.14159",
        "2.718281828459045",
        "9.81",
        "99.99",
        "1e-10",
        "6.02214076e23",
        "1.602176634e-19",
        "6.62607015e-34",
        "1.380649e-23",
        "299792458.5",
    ]
    .into_iter()
    .filter(|value| {
        let protocol = model::parse(&with_number(value)).expect("parses");
        let text = ir::compile(&protocol).expect("compiles").canonical_json();
        eval::read_ir(&text).is_err()
    })
    .collect();
    assert_eq!(
        refused,
        Vec::<&str>::new(),
        "read_ir refuses the compiled IR"
    );
}

/// A top-level object whose property `l` is `levels - 1` arrays deep, flow YAML.
fn items_deep(levels: usize) -> String {
    let mut schema = "{type: string}".to_owned();
    for _ in 1..levels.saturating_sub(1) {
        schema = format!("{{type: array, items: {schema}}}");
    }
    format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nactions:\n  act:\n    \
         arguments: {{type: object, properties: {{l: {schema}}}}}\n"
    )
}

/// The bound is reachable from YAML through `items` (one YAML level per schema level): 32 levels
/// compile and read back, 33 are refused as `action-arguments`, not by the YAML parser.
#[test]
fn the_schema_depth_bound_holds_at_32_and_33_through_items() {
    let protocol = model::parse(&items_deep(32)).expect("32 levels parse");
    let text = ir::compile(&protocol)
        .unwrap_or_else(|problems| panic!("32 levels compile: {problems:?}"))
        .canonical_json();
    eval::read_ir(&text).expect("32 levels read back");
    let protocol = model::parse(&items_deep(33)).expect("33 levels parse");
    let problems = ir::compile(&protocol).expect_err("33 levels are refused");
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert_eq!(problems[0].code(), "action-arguments", "{}", problems[0]);
}

/// `read_ir` from JSON does not pass through the YAML parser's limit: an IR whose schema nests
/// 2000 levels (within `MAX_IR_DEPTH` of JSON nesting) is refused as malformed input, without a
/// crash.
#[test]
fn a_deep_schema_in_canon_ir_is_refused_not_crashed() {
    let shallow = ir::compile(&model::parse(&items_deep(3)).expect("parses"))
        .expect("compiles")
        .canonical_json();
    for levels in [65usize, 2000] {
        let mut deep = String::from("{\"type\": \"string\"}");
        for _ in 0..levels {
            deep = format!("{{\"items\": {deep}, \"type\": \"array\"}}");
        }
        let refusal = eval::read_ir(&deep_ir(&shallow, &deep)).expect_err("a deep schema");
        assert_eq!(refusal.code(), "malformed-input", "{levels}: {refusal:?}");
    }
}

/// `shallow` with the schema of property `l` replaced by `deep`, as text.
fn deep_ir(shallow: &str, deep: &str) -> String {
    let marker = "\"l\": ";
    let start = shallow.find(marker).expect("property l") + marker.len();
    let mut depth = 0usize;
    let mut end = start;
    for (offset, byte) in shallow[start..].bytes().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    end = start + offset + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    format!("{}{deep}{}", &shallow[..start], &shallow[end..])
}

/// The published JSON Schema of `protocol/1` describes the field `canon validate` accepts: the
/// fixture's actions carry `arguments`, and the schema's action is `additionalProperties: false`.
#[test]
fn the_published_protocol_schema_declares_arguments() {
    let path = root().join("website/static/schemas/protocol-1.schema.json");
    let schema: Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("the schema reads")).expect("JSON");
    let action = &schema["$defs"]["Action"];
    assert_eq!(
        action["additionalProperties"],
        Value::Bool(false),
        "{action}"
    );
    assert!(
        action["properties"].get("arguments").is_some(),
        "website/static/schemas/protocol-1.schema.json: Action has no `arguments`, so it refuses \
         fixtures/investigation/action-arguments.yaml, which canon validate accepts: {}",
        action["properties"]
    );
    let reference = fs::read_to_string(root().join("website/docs/reference/protocol.md"))
        .expect("the reference reads");
    assert!(
        reference.contains("| `arguments` |"),
        "website/docs/reference/protocol.md has no `arguments` row"
    );
}

// ---- Second pass, against b6657c5 ----

/// A protocol with one action `act` whose `arguments` is `schema`, flow YAML, at `revision`.
fn protocol(schema: &str, revision: u32) -> String {
    format!(
        "format: protocol/1\nprotocol: {{id: p, revision: {revision}}}\nactions:\n  act:\n    \
         arguments: {schema}\n"
    )
}

/// Every problem `canon validate` reports for `schema`, as `(code, message)`.
fn problems_of(schema: &str) -> Vec<(String, String)> {
    let text = protocol(schema, 1);
    let parsed = model::parse(&text).unwrap_or_else(|error| panic!("{schema} parses: {error}"));
    match ir::compile(&parsed) {
        Ok(_) => Vec::new(),
        Err(problems) => problems
            .iter()
            .map(|problem| (problem.code().to_owned(), problem.to_string()))
            .collect(),
    }
}

/// A top-level object whose property `p` nests object schemas `levels` deep in all, flow YAML.
fn properties_deep(levels: usize) -> String {
    let mut schema = "{type: string}".to_owned();
    for _ in 1..levels {
        schema = format!("{{type: object, properties: {{p: {schema}}}}}");
    }
    schema
}

/// The dialect promises 32 levels, "each schema in `properties` or `items` one more". Through
/// `properties` a schema level costs two YAML levels, so 32 levels must still parse, compile and
/// read back, and 33 must be refused as `action-arguments`, not by the YAML parser (whose
/// recursion limit refused 64 levels this way).
#[test]
fn the_schema_depth_bound_holds_at_32_and_33_through_properties() {
    let deepest_parsing = (1..=70)
        .take_while(|&levels| model::parse(&protocol(&properties_deep(levels), 1)).is_ok())
        .last()
        .unwrap_or(0);
    let at_32 = model::parse(&protocol(&properties_deep(32), 1));
    assert!(
        at_32.is_ok(),
        "a 32-level schema through `properties` does not parse (deepest that parses: {deepest_parsing}): {}",
        at_32
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default()
    );
    let text = ir::compile(&at_32.unwrap())
        .expect("32 levels compile")
        .canonical_json();
    eval::read_ir(&text).expect("32 levels read back");
    let at_33 = model::parse(&protocol(&properties_deep(33), 1))
        .unwrap_or_else(|error| panic!("33 levels are refused by YAML, not the dialect: {error}"));
    let problems = ir::compile(&at_33).expect_err("33 levels are refused");
    assert_eq!(problems[0].code(), "action-arguments", "{}", problems[0]);
}

/// Every way out of the document a 2020-12 validator would follow is refused at its pointer:
/// `$ref` to http, to a relative file, to `file:`; `$dynamicRef`, `$dynamicAnchor`, `$anchor`,
/// `$id` rebasing (nested too), `$vocabulary`; and one hidden in a place the walk does not descend
/// (`$defs`, a schema-valued `additionalProperties`, an `enum` value) is refused once, at the place.
/// A property *named* `$ref` or `$id` is a name, not a keyword, and is accepted.
#[test]
fn every_reference_form_is_refused_at_its_pointer() {
    let refused: &[(&str, &str)] = &[
        ("{type: object, $ref: 'http://example.com/a.json'}", "/$ref"),
        (
            "{type: object, properties: {a: {type: string, $ref: 'other.yaml#/a'}}}",
            "/properties/a/$ref",
        ),
        (
            "{type: object, properties: {a: {type: string, $ref: 'file:///etc/passwd'}}}",
            "/properties/a/$ref",
        ),
        ("{type: object, $dynamicRef: '#meta'}", "/$dynamicRef"),
        ("{type: object, $dynamicAnchor: meta}", "/$dynamicAnchor"),
        ("{type: object, $anchor: a}", "/$anchor"),
        ("{type: object, $vocabulary: {}}", "/$vocabulary"),
        ("{type: object, $id: 'https://evil.example/'}", "/$id"),
        (
            "{type: object, properties: {l: {type: array, items: {type: string, $id: 'https://evil.example/s'}}}}",
            "/properties/l/items/$id",
        ),
        (
            "{type: object, $defs: {a: {type: string, $ref: 'http://x'}}}",
            "/$defs",
        ),
        (
            "{type: object, additionalProperties: {$ref: 'http://x'}}",
            "/additionalProperties",
        ),
        (
            "{type: object, properties: {a: {type: string, enum: [{$ref: 'http://x'}]}}}",
            "/properties/a/enum/0",
        ),
    ];
    let mut wrong = Vec::new();
    for (schema, pointer) in refused {
        let found = problems_of(schema);
        let expected = format!("action `act` arguments at `{pointer}`: ");
        if found.len() != 1
            || found[0].0 != "action-arguments"
            || !found[0].1.starts_with(&expected)
        {
            wrong.push(format!("{schema}: {found:?}"));
        }
    }
    for schema in [
        "{type: object, properties: {$ref: {type: string}}, required: [$ref]}",
        "{type: object, properties: {$id: {type: object, properties: {$dynamicRef: {type: string}}}}}",
    ] {
        let found = problems_of(schema);
        if !found.is_empty() {
            wrong.push(format!("{schema} (a name, not a keyword): {found:?}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// A top level that is not `type: object` is one `action-arguments` problem, whatever it is: a
/// boolean schema (valid 2020-12, `true` accepts anything), a list, a string, a mapping without
/// `type`, a list-valued or null `type`, another type.
#[test]
fn every_non_object_top_level_is_one_problem() {
    let mut wrong = Vec::new();
    for schema in [
        "true",
        "false",
        "[]",
        "'object'",
        "5",
        "{}",
        "{type: [object]}",
        "{type: ~}",
        "{type: 'null'}",
        "{type: array, items: {type: string}}",
        "{title: t}",
    ] {
        let found = problems_of(schema);
        if found.len() != 1 || found[0].0 != "action-arguments" {
            wrong.push(format!("{schema}: {found:?}"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// `read_ir` holds an IR's schema to the dialect, as it holds the rest of the protocol to the
/// validator: a `$ref` or a non-object top level edited into a canonical IR is refused.
#[test]
fn read_ir_refuses_a_schema_outside_the_dialect() {
    let text =
        ir::compile(&model::parse(&protocol("{type: object, title: X}", 1)).expect("parses"))
            .expect("compiles")
            .canonical_json();
    assert!(text.contains("\"title\": \"X\""), "{text}");
    eval::read_ir(&text).expect("the original reads");
    let with_ref = text.replace("\"title\": \"X\"", "\"$ref\": \"http://x\"");
    assert!(
        eval::read_ir(&with_ref).is_err(),
        "read_ir accepts {with_ref}"
    );
    let with_string = text.replace("\"type\": \"object\"", "\"type\": \"string\"");
    assert!(
        eval::read_ir(&with_string).is_err(),
        "read_ir accepts {with_string}"
    );
}

/// The `arguments` lines `canon diff` reports from `from` to `to`.
fn argument_lines(from: &str, to: &str) -> Vec<String> {
    let from = ir::compile(&model::parse(&protocol(from, 1)).expect("parses")).expect("compiles");
    let to = ir::compile(&model::parse(&protocol(to, 2)).expect("parses")).expect("compiles");
    diff::diff(&from, &to)
        .expect("compares")
        .changes
        .into_iter()
        .filter(|change| change.what.starts_with("arguments"))
        .map(|change| format!("{}: {}", change.category, change.what))
        .collect()
}

/// The validator compares enum values by JSON Schema's equality, `1` and `1.0` being one value
/// (validate/arguments.rs:348-359), and the classifier's rule is "a value removed narrows, a value
/// added widens". Writing `1.0` for `1` removes and adds no value, so it is no change, or at most
/// `arguments rewritten`; never `BREAKING`.
#[test]
fn an_enum_value_rewritten_as_an_equal_number_is_not_breaking() {
    let lines = argument_lines(
        "{type: object, properties: {n: {type: number, enum: [1, 2]}}}",
        "{type: object, properties: {n: {type: number, enum: [1.0, 2]}}}",
    );
    assert!(
        lines.is_empty() || lines == ["NO SEMANTIC CHANGE: arguments rewritten"],
        "{lines:?}"
    );
}

/// The published JSON Schema of `protocol/1` leaves `arguments` untyped (the ESS field is `Json`),
/// so it names `canon validate` as what holds the schema to the dialect: an editor validating
/// against it accepts `arguments: 5`, which `canon validate` refuses as `action-arguments`.
#[test]
fn the_published_schema_names_the_validator_for_arguments() {
    let path = root().join("website/static/schemas/protocol-1.schema.json");
    let schema: Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("reads")).expect("JSON");
    let arguments = &schema["$defs"]["Action"]["properties"]["arguments"];
    assert!(
        arguments["description"]
            .as_str()
            .is_some_and(|text| text.contains("action-arguments")),
        "{arguments}"
    );
}

// A fixed-seed property test of the classifier's soundness, through `canon diff`.

/// A small linear congruential generator, fixed seed.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }
    fn subset(&mut self, pool: &[Value]) -> Vec<Value> {
        let picked: Vec<Value> = pool.iter().filter(|_| self.chance(50)).cloned().collect();
        if picked.is_empty() {
            vec![pool[self.below(pool.len() as u64) as usize].clone()]
        } else {
            picked
        }
    }
}

const TYPES: [&str; 7] = [
    "object", "array", "string", "integer", "number", "boolean", "null",
];

fn enum_pool(ty: &str) -> Vec<Value> {
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
fn roll(rng: &mut Rng, schema: &mut serde_json::Map<String, Value>, keyword: &str, depth: usize) {
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

fn keywords(ty: &str, depth: usize) -> Vec<&'static str> {
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

fn generate(rng: &mut Rng, depth: usize, top: bool) -> Value {
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
fn mutate(rng: &mut Rng, schema: &Value, depth: usize, top: bool) -> Value {
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
fn document(rng: &mut Rng, depth: usize) -> Value {
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

fn object_document(rng: &mut Rng, depth: usize) -> Value {
    let mut object = serde_json::Map::new();
    for name in ["a", "b", "c"] {
        if rng.chance(55) {
            object.insert(name.to_owned(), document(rng, depth));
        }
    }
    Value::Object(object)
}

/// JSON Schema 2020-12's meaning of a schema in the dialect.
fn accepts(schema: &Value, doc: &Value) -> bool {
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

/// Over random pairs of schemas in the dialect (fixed seed), whatever `canon diff` claims holds on
/// random documents: `narrowed` means every document the new schema accepts the old one did,
/// `widened` the reverse, `rewritten` and no line that both accept the same documents.
#[test]
fn every_classification_canon_diff_makes_is_sound_on_random_schemas() {
    let mut rng = Rng(0x00c0_ffee_1234_5678);
    let (mut compared, mut skipped) = (0usize, 0usize);
    let mut unsound = Vec::new();
    let mut seen = std::collections::BTreeMap::<String, usize>::new();
    for round in 0..4000 {
        let from = generate(&mut rng, 1, true);
        let to = if round % 4 == 0 {
            generate(&mut rng, 1, true)
        } else {
            mutate(&mut rng, &from, 1, true)
        };
        let (from_text, to_text) = (from.to_string(), to.to_string());
        if !problems_of(&from_text).is_empty() || !problems_of(&to_text).is_empty() {
            skipped += 1;
            continue;
        }
        compared += 1;
        let lines = argument_lines(&from_text, &to_text);
        let line = lines.first().cloned().unwrap_or_else(|| "none".to_owned());
        *seen.entry(line.clone()).or_default() += 1;
        for _ in 0..300 {
            let doc = object_document(&mut rng, 1);
            let (old, new) = (accepts(&from, &doc), accepts(&to, &doc));
            let wrong = match line.as_str() {
                "TIGHTENING: arguments narrowed" => new && !old,
                "RELAXATION: arguments widened" => old && !new,
                "NO SEMANTIC CHANGE: arguments rewritten" | "none" => old != new,
                _ => false,
            };
            if wrong {
                unsound.push(format!(
                    "{line}: {from_text} -> {to_text} on {doc} (old {old}, new {new})"
                ));
                break;
            }
        }
    }
    eprintln!("compared {compared}, skipped {skipped}: {seen:?}");
    assert!(
        compared > 1000,
        "only {compared} pairs compared ({skipped} skipped)"
    );
    for line in [
        "TIGHTENING: arguments narrowed",
        "RELAXATION: arguments widened",
        "BREAKING: arguments changed",
        "NO SEMANTIC CHANGE: arguments rewritten",
    ] {
        assert!(
            seen.get(line).is_some_and(|&n| n >= 20),
            "{line} seldom drawn: {seen:?}"
        );
    }
    assert!(
        unsound.is_empty(),
        "{} unsound of {compared} ({seen:?}); first: {:#?}",
        unsound.len(),
        &unsound[..unsound.len().min(3)]
    );
}
