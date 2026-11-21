//! Adversary pass 1 on story:argument-validator: `validate::check_arguments` against its claims.
//!
//! - Claim 5: `canon validate`, `canon compile` and `canon diff` print what they printed on the
//!   base for every protocol. The base binary `canon-6f44325` (same argument-schema and diff code
//!   as the base 619fd03) prints the texts pinned here; the change of `same` to exact comparison
//!   changes them.
//! - Claim 3: numbers are compared exactly. With serde_json's `arbitrary_precision` feature a
//!   `Number` holds the text exactly, and `cargo test --workspace` turns that feature on, because
//!   `b10x-canon-expr` enables it. The function then compares through `as_f64`, which rounds, and
//!   is `None` for a number beyond the double range, which `double` turns into NaN. Each of those
//!   cases checks first that the `Number` holds the exact text; without the feature serde_json
//!   cannot hand the value over at all and the case asserts that instead.
//! - Mutants the unit's suite does not catch: pointer escaping, `type` hiding the other keywords,
//!   refusals at three depths.

use b10x_canon::diff;
use b10x_canon::ir;
use b10x_canon::model;
use b10x_canon::validate::{self, ArgumentsRefused, check_arguments};
use serde_json::{Value, json};

fn protocol(schema: &str, revision: u32) -> String {
    format!(
        "format: protocol/1\nprotocol: {{id: p, revision: {revision}}}\nactions:\n  act:\n    \
         arguments: {schema}\n"
    )
}

fn refusals(schema: Option<&Value>, arguments: &Value) -> Vec<(String, String)> {
    match check_arguments(schema, arguments) {
        Ok(()) => Vec::new(),
        Err(ArgumentsRefused::Arguments(found)) => found
            .into_iter()
            .map(|refusal| (refusal.pointer, refusal.keyword.to_owned()))
            .collect(),
        Err(ArgumentsRefused::Schema(found)) => panic!("the schema is refused: {found:?}"),
    }
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(pointer, keyword)| ((*pointer).to_owned(), (*keyword).to_owned()))
        .collect()
}

/// `{"n": <text>}` parsed by serde_json, and whether the number kept its text exactly (the
/// `arbitrary_precision` feature); `None` when serde_json refuses the text.
fn n(text: &str) -> Option<(Value, bool)> {
    let value: Value = serde_json::from_str(&format!("{{\"n\": {text}}}")).ok()?;
    let shown = value["n"].to_string();
    let exact = shown == text;
    Some((value, exact))
}

fn schema(text: &str) -> Value {
    serde_json::from_str(text).expect("JSON")
}

// ---- claim 5: no change to what `canon validate`, `compile` and `diff` print ----

/// Base: `error[action-arguments]: action `act` arguments at `/properties/n/enum/1`:
/// `9007199254740992.0` is listed more than once` (exit 1). HEAD: `valid` (exit 0), and `canon
/// compile` now compiles it.
#[test]
fn validate_still_refuses_the_enum_the_base_refused() {
    let text = protocol(
        "{type: object, properties: {n: {type: number, enum: [9007199254740993, 9007199254740992.0]}}}",
        1,
    );
    let problems = match validate::validate(&model::parse(&text).expect("parses")) {
        Ok(()) => Vec::new(),
        Err(problems) => problems.iter().map(ToString::to_string).collect(),
    };
    assert_eq!(
        problems,
        [
            "action `act` arguments at `/properties/n/enum/1`: `9007199254740992.0` is listed more than once"
        ],
        "claim 5: `canon validate` prints what it printed on the base"
    );
}

/// Base: `NO SEMANTIC CHANGE: action `act` arguments rewritten`. HEAD: `BREAKING: action `act`
/// arguments changed`.
#[test]
fn diff_still_prints_what_the_base_printed_for_an_enum_above_two_to_the_53() {
    let from = "{type: object, properties: {n: {type: number, enum: [9007199254740992.0]}}}";
    let to = "{type: object, properties: {n: {type: number, enum: [9007199254740993]}}}";
    let from = ir::compile(&model::parse(&protocol(from, 1)).expect("parses")).expect("compiles");
    let to = ir::compile(&model::parse(&protocol(to, 2)).expect("parses")).expect("compiles");
    let lines: Vec<String> = diff::diff(&from, &to)
        .expect("compares")
        .changes
        .into_iter()
        .filter(|change| change.what.starts_with("arguments"))
        .map(|change| format!("{}: {}", change.category, change.what))
        .collect();
    assert_eq!(
        lines,
        ["NO SEMANTIC CHANGE: arguments rewritten"],
        "claim 5: `canon diff` prints what it printed on the base"
    );
}

// ---- claim 3: exact comparison, with the number serde_json hands over ----

/// `1e400` under `maximum: 5`: 2020-12 refuses it.
#[test]
fn a_number_beyond_the_double_range_is_above_every_maximum() {
    let schema =
        schema(r#"{"type": "object", "properties": {"n": {"type": "number", "maximum": 5}}}"#);
    let Some((doc, _)) = n("1e400") else {
        return; // without arbitrary_precision serde_json refuses the text: no caller holds it
    };
    assert!(doc["n"].is_number() && doc["n"].as_f64().is_none(), "{doc}"); // held, no double
    assert_eq!(refusals(Some(&schema), &doc), pairs(&[("/n", "maximum")]));
}

/// `-1e400` under `minimum: 0`: 2020-12 refuses it.
#[test]
fn a_number_beyond_the_double_range_is_below_every_minimum() {
    let schema =
        schema(r#"{"type": "object", "properties": {"n": {"type": "number", "minimum": 0}}}"#);
    let Some((doc, _)) = n("-1e400") else {
        return;
    };
    assert!(doc["n"].is_number() && doc["n"].as_f64().is_none(), "{doc}"); // held, no double
    assert_eq!(refusals(Some(&schema), &doc), pairs(&[("/n", "minimum")]));
}

/// `1e400` against `enum: [1.5]`: not one of the values.
#[test]
fn a_number_beyond_the_double_range_is_not_an_enum_value() {
    let schema =
        schema(r#"{"type": "object", "properties": {"n": {"type": "number", "enum": [1.5]}}}"#);
    let Some((doc, _)) = n("1e400") else {
        return;
    };
    assert!(doc["n"].is_number() && doc["n"].as_f64().is_none(), "{doc}"); // held, no double
    assert_eq!(refusals(Some(&schema), &doc), pairs(&[("/n", "enum")]));
}

/// `9007199254740993.0` is the integer 9007199254740993 and meets `minimum: 9007199254740993`;
/// the unit's own test pins `9007199254740992.0` as refused and `9007199254740994.0` as accepted.
#[test]
fn a_fraction_written_number_equal_to_the_minimum_meets_it() {
    let schema = schema(
        r#"{"type": "object", "properties": {"n": {"type": "number", "minimum": 9007199254740993}}}"#,
    );
    let (doc, exact) = n("9007199254740993.0").expect("parses");
    if !exact {
        assert_eq!(doc["n"].as_f64(), Some(9_007_199_254_740_992.0));
        return; // the double serde_json reads, as the module docs say
    }
    assert_eq!(check_arguments(Some(&schema), &doc), Ok(()));
}

/// `9007199254740992.5` is above `maximum: 9007199254740992`.
#[test]
fn a_fraction_above_the_maximum_is_refused_by_its_exact_value() {
    let schema = schema(
        r#"{"type": "object", "properties": {"n": {"type": "number", "maximum": 9007199254740992}}}"#,
    );
    let (doc, exact) = n("9007199254740992.5").expect("parses");
    if !exact {
        return;
    }
    assert_eq!(refusals(Some(&schema), &doc), pairs(&[("/n", "maximum")]));
}

/// `-9223372036854775809` is below `minimum: -9223372036854775808`.
#[test]
fn an_integer_below_i64_is_below_the_i64_minimum() {
    let schema = schema(
        r#"{"type": "object", "properties": {"n": {"type": "integer", "minimum": -9223372036854775808}}}"#,
    );
    let (doc, exact) = n("-9223372036854775809").expect("parses");
    if !exact {
        return;
    }
    assert_eq!(refusals(Some(&schema), &doc), pairs(&[("/n", "minimum")]));
}

/// `9007199254740993.0` is the `enum` value `9007199254740993`.
#[test]
fn a_fraction_written_integer_is_its_enum_value() {
    let schema = schema(
        r#"{"type": "object", "properties": {"n": {"type": "number", "enum": [9007199254740993]}}}"#,
    );
    let (doc, exact) = n("9007199254740993.0").expect("parses");
    if !exact {
        return;
    }
    assert_eq!(check_arguments(Some(&schema), &doc), Ok(()));
}

// ---- mutants the unit's suite does not catch ----

/// RFC 6901: `~` is `~0` and `/` is `~1`, in a refused member and in a descended one.
#[test]
fn pointers_escape_tilde_and_slash() {
    assert_eq!(
        refusals(None, &json!({"a/b": 1, "m~n": 2, "~1": 3})),
        pairs(&[
            ("/a~1b", "additionalProperties"),
            ("/m~0n", "additionalProperties"),
            ("/~01", "additionalProperties"),
        ])
    );
    let schema = json!({"type": "object", "properties": {"x/y~": {"type": "string"}}});
    assert_eq!(
        refusals(Some(&schema), &json!({"x/y~": 5})),
        pairs(&[("/x~1y~0", "type")])
    );
}

/// When `type` refuses, `enum` of the same schema is not checked: the unit's expectation-3 test
/// uses schemas with no keyword that could refuse there, so dropping the early return passes it.
#[test]
fn a_type_refusal_hides_the_enum_of_that_schema() {
    let schema = json!({"type": "object", "properties": {"s": {"type": "string", "enum": ["a"]}}});
    assert_eq!(
        refusals(Some(&schema), &json!({"s": 5})),
        pairs(&[("/s", "type")])
    );
}

/// Refusals at three depths, a location's own first, members in code-point order, twice.
#[test]
fn refusals_at_three_depths_come_in_document_order() {
    let schema = json!({"type": "object", "required": ["r"], "properties": {
        "r": {"type": "string"},
        "p": {"type": "object", "required": ["w"], "properties": {
            "w": {"type": "null"},
            "q": {"type": "array", "maxItems": 1,
                  "items": {"type": "object", "additionalProperties": false}}
        }}
    }});
    let doc = json!({"p": {"q": [{"z": 1}, {"b": 2, "a": 3}]}});
    let first = refusals(Some(&schema), &doc);
    assert_eq!(first, refusals(Some(&schema), &doc));
    assert_eq!(
        first,
        pairs(&[
            ("", "required"),
            ("/p", "required"),
            ("/p/q", "maxItems"),
            ("/p/q/0/z", "additionalProperties"),
            ("/p/q/1/a", "additionalProperties"),
            ("/p/q/1/b", "additionalProperties"),
        ])
    );
}

// ---- purity and cost ----

/// A 10,000-deep document under a schema that describes one level: no stack overflow, and the
/// depth reached is the schema's.
#[test]
fn a_ten_thousand_deep_document_does_not_overflow() {
    let mut deep = json!([]);
    for _ in 0..10_000 {
        deep = Value::Array(vec![deep]); // `json!` would serialize it, recursing in the test
    }
    let mut doc = serde_json::Map::new();
    doc.insert("a".to_owned(), deep);
    doc.insert("b".to_owned(), json!(1));
    let doc = Value::Object(doc);
    let schema = json!({"type": "object", "additionalProperties": false, "properties": {
        "a": {"type": "array", "items": {"type": "array", "maxItems": 0}}
    }});
    assert_eq!(
        refusals(Some(&schema), &doc),
        pairs(&[("/a/0", "maxItems"), ("/b", "additionalProperties")])
    );
    std::mem::forget(doc); // dropping it would recurse 10,000 deep in the test, not the function
}

/// A small fixed-seed generator for the fuzz below.
struct Fuzz(u64);

impl Fuzz {
    fn below(&mut self, n: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) % n
    }

    fn number(&mut self) -> Value {
        let numbers = [
            json!(0),
            json!(-1),
            json!(i64::MIN),
            json!(i64::MAX),
            json!(u64::MAX),
            json!(9_007_199_254_740_993_u64),
            json!(1.5),
            json!(-0.0),
            json!(1e300),
            json!(-1e300),
            json!(f64::MAX),
            json!(9_007_199_254_740_992.0),
        ];
        numbers[self.below(numbers.len() as u64) as usize].clone()
    }

    fn value(&mut self, depth: u32) -> Value {
        match self.below(if depth > 3 { 5 } else { 7 }) {
            0 => Value::Null,
            1 => json!(self.below(2) == 0),
            2 | 3 => self.number(),
            4 => json!(["", "a", "a/b", "~", "\u{1F600}"][self.below(5) as usize]),
            5 => {
                let length = self.below(3);
                Value::Array((0..length).map(|_| self.value(depth + 1)).collect())
            }
            _ => {
                let mut map = serde_json::Map::new();
                for name in ["a", "b/c", "~d"] {
                    if self.below(2) == 0 {
                        let member = self.value(depth + 1);
                        map.insert(name.to_owned(), member);
                    }
                }
                Value::Object(map)
            }
        }
    }

    fn schema(&mut self, depth: u32, top: bool) -> Value {
        const TYPES: [&str; 7] = [
            "object", "array", "string", "integer", "number", "boolean", "null",
        ];
        const KEYWORDS: [&str; 14] = [
            "type",
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
            "title",
            "$ref",
        ];
        let mut map = serde_json::Map::new();
        let ty = if top {
            "object"
        } else {
            TYPES[self.below(7) as usize]
        };
        if self.below(20) != 0 {
            map.insert("type".to_owned(), json!(ty));
        }
        for keyword in KEYWORDS {
            if self.below(4) != 0 {
                continue;
            }
            let value = match keyword {
                "properties" if depth < 4 => {
                    let mut properties = serde_json::Map::new();
                    for name in ["a", "b/c", "~d"] {
                        if self.below(2) == 0 {
                            let schema = self.schema(depth + 1, false);
                            properties.insert(name.to_owned(), schema);
                        }
                    }
                    Value::Object(properties)
                }
                "items" if depth < 4 => self.schema(depth + 1, false),
                "required" => json!(["a", "b/c", "~d"][..self.below(4) as usize]),
                "additionalProperties" => json!(self.below(2) == 0),
                "type" => json!(TYPES[self.below(7) as usize]),
                "enum" => {
                    let length = 1 + self.below(3);
                    Value::Array((0..length).map(|_| self.value(4)).collect())
                }
                "minItems" | "maxItems" | "minLength" | "maxLength" => json!(self.below(4)),
                "minimum" | "maximum" if self.below(3) == 0 => self.number(),
                "minimum" | "maximum" => json!(self.below(5) as i64 - 2),
                _ => self.value(3),
            };
            map.insert(keyword.to_owned(), value);
        }
        Value::Object(map)
    }
}

/// Random schemas built from the dialect's keywords with values from a wide pool, many outside
/// the dialect, against random documents with extreme numbers: no panic, the same result twice,
/// and never an empty refusal list.
#[test]
fn random_schemas_and_documents_never_panic() {
    let mut fuzz = Fuzz(0x5eed_0001);
    let (mut inside, mut outside) = (0, 0);
    for _ in 0..4000 {
        let top = fuzz.below(10) != 0;
        let schema = fuzz.schema(1, top);
        match check_arguments(Some(&schema), &json!({})) {
            Err(ArgumentsRefused::Schema(_)) => outside += 1,
            _ => inside += 1,
        }
        for _ in 0..5 {
            let doc = fuzz.value(0);
            let first = check_arguments(Some(&schema), &doc);
            assert_eq!(
                first,
                check_arguments(Some(&schema), &doc),
                "{schema} on {doc}"
            );
            if let Err(ArgumentsRefused::Arguments(list)) = &first {
                assert!(!list.is_empty(), "{schema} on {doc}");
            }
        }
    }
    assert!(
        inside > 100 && outside > 100,
        "{inside} inside, {outside} outside"
    );
}
