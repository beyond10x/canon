//! Adversary passes 1 and 2 on story:argument-validator: `validate::check_arguments` against its
//! claims.
//!
//! - D1 (decided in review): one exact comparison of numbers. `canon validate` accepts an `enum`
//!   holding `9007199254740993` and `9007199254740992.0`, and `canon diff` prints `BREAKING` for
//!   `[9007199254740992.0]` to `[9007199254740993]`. Pass 1 pinned the base's outputs; pass 2
//!   re-pins them to these.
//! - D2: the comparison is exact however `serde_json` is built. With `arbitrary_precision` a
//!   `Number` read from JSON holds its text, and `cargo test --workspace` turns that feature on,
//!   because `b10x-canon-expr` enables it; `cargo install canon-cli` and `cargo test -p b10x-canon`
//!   do not. The pass 1 cases of claim 3 check the text path and return early without the feature.
//! - Pass 2: a number a protocol's YAML writes with a fraction or an exponent is a double that
//!   holds the literal exactly, yet with `arbitrary_precision` it is compared by the shortest text
//!   of that double, which is another decimal above 2^53. Those cases are red in
//!   `cargo test --workspace` and green in `cargo test -p b10x-canon`. A fuzz compares the exact
//!   decimal comparison with an oracle written here, digit strings aligned at the point.
//! - Mutants the unit's suite did not catch in pass 1: pointer escaping, `type` hiding the other
//!   keywords, refusals at three depths.

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

/// What `canon validate` prints for a protocol whose action `act` has `schema` as its arguments.
fn validate_lines(schema: &str) -> Vec<String> {
    match validate::validate(&model::parse(&protocol(schema, 1)).expect("parses")) {
        Ok(()) => Vec::new(),
        Err(problems) => problems.iter().map(ToString::to_string).collect(),
    }
}

/// The `arguments` lines `canon diff` prints from revision `from` to `to` of the schema.
fn diff_lines(from: &str, to: &str) -> Vec<String> {
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

// ---- D1: the exact comparison's outputs of `canon validate` and `canon diff` ----

/// Base: refused as `9007199254740992.0` listed more than once. D1: the two are distinct values,
/// so `canon validate` prints `valid` and `canon compile` compiles it.
#[test]
fn validate_accepts_the_enum_d1_keeps_distinct() {
    assert_eq!(
        validate_lines(
            "{type: object, properties: {n: {type: number, enum: [9007199254740993, 9007199254740992.0]}}}"
        ),
        Vec::<String>::new(),
        "D1: `canon validate` accepts two values a double cannot tell apart"
    );
}

/// Base: `NO SEMANTIC CHANGE: action `act` arguments rewritten`. D1: `BREAKING: action `act`
/// arguments changed`, since the old and the new value are different numbers.
#[test]
fn diff_prints_breaking_for_an_enum_value_changed_above_two_to_the_53() {
    assert_eq!(
        diff_lines(
            "{type: object, properties: {n: {type: number, enum: [9007199254740992.0]}}}",
            "{type: object, properties: {n: {type: number, enum: [9007199254740993]}}}"
        ),
        ["BREAKING: arguments changed"],
        "D1: `canon diff` reports the changed value"
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

// ---- pass 2: a protocol's YAML number against the arguments, with the text held ----

/// Whether serde_json holds a number's text (`arbitrary_precision`).
fn holds_text() -> bool {
    "0.10"
        .parse::<serde_json::Number>()
        .is_ok_and(|number| number.to_string() == "0.10")
}

/// The arguments schema of action `act` as `ir::compile` holds it, numbers as the YAML reader
/// made them.
fn compiled_arguments(schema: &str) -> Value {
    let compiled =
        ir::compile(&model::parse(&protocol(schema, 1)).expect("parses")).expect("compiles");
    compiled
        .actions
        .values()
        .next()
        .and_then(|action| action.arguments.clone())
        .expect("the action declares arguments")
}

/// `1152921504606846976.0` in the protocol is 2^60, a double holds it exactly, and the arguments
/// `1152921504606846976` are 2^60: one value by JSON Schema 2020-12. With the text held the
/// schema's number is compared as `1152921504606847000`, the shortest text of that double.
#[test]
fn a_yaml_enum_value_a_double_holds_exactly_is_that_integer() {
    let schema = compiled_arguments(
        "{type: object, properties: {n: {type: number, enum: [1152921504606846976.0]}}}",
    );
    let arguments: Value = serde_json::from_str(r#"{"n": 1152921504606846976}"#).expect("JSON");
    assert_eq!(check_arguments(Some(&schema), &arguments), Ok(()));
}

/// `1152921504606846976` and `1152921504606846976.0` are one value, so `canon validate` refuses
/// the second as listed twice, as the shipped binary (no `arbitrary_precision`) does.
#[test]
fn validate_refuses_an_integer_and_its_exact_double_as_one_value() {
    let lines = validate_lines(
        "{type: object, properties: {n: {type: number, enum: [1152921504606846976, 1152921504606846976.0]}}}",
    );
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(
        lines[0].starts_with("action `act` arguments at `/properties/n/enum/1`: ")
            && lines[0].ends_with("is listed more than once"),
        "{lines:?}"
    );
}

/// `[1152921504606846976.0]` to `[1152921504606846976]` rewrites one value: no semantic change.
#[test]
fn diff_finds_no_change_when_an_enum_value_is_rewritten_as_its_exact_double() {
    assert_eq!(
        diff_lines(
            "{type: object, properties: {n: {type: number, enum: [1152921504606846976.0]}}}",
            "{type: object, properties: {n: {type: number, enum: [1152921504606846976]}}}"
        ),
        ["NO SEMANTIC CHANGE: arguments rewritten"]
    );
}

/// The same literal, `9007199254740993.0`, in the protocol's `enum` and in the arguments. The
/// YAML reader rounds the schema's to 2^53; with the text held the arguments keep 2^53 + 1, so the
/// call is refused although both files write one value.
#[test]
fn the_same_literal_in_the_protocol_and_the_arguments_is_one_enum_value() {
    let schema = compiled_arguments(
        "{type: object, properties: {n: {type: number, enum: [9007199254740993.0]}}}",
    );
    let arguments: Value = serde_json::from_str(r#"{"n": 9007199254740993.0}"#).expect("JSON");
    assert_eq!(check_arguments(Some(&schema), &arguments), Ok(()));
}

// ---- pass 2: the exact decimal comparison against an oracle written here ----

/// A number `±digits × 10^exponent`, with the text it is written as.
struct Written {
    negative: bool,
    digits: String,
    exponent: i64,
}

/// The oracle: the value as its integer and fraction digit strings, aligned at the point, with no
/// leading zero in the integer part and no trailing zero in the fraction; `None` for zero.
fn fixed_point(written: &Written) -> Option<(bool, String, String)> {
    let digits = written.digits.trim_start_matches('0');
    let (whole, fraction) = if written.exponent >= 0 {
        (
            format!("{digits}{}", "0".repeat(written.exponent as usize)),
            String::new(),
        )
    } else {
        let places = (-written.exponent) as usize;
        let padded = format!(
            "{}{digits}",
            "0".repeat(places.saturating_sub(digits.len()))
        );
        let (whole, fraction) = padded.split_at(padded.len() - places);
        (whole.to_owned(), fraction.to_owned())
    };
    let whole = whole.trim_start_matches('0').to_owned();
    let fraction = fraction.trim_end_matches('0').to_owned();
    if whole.is_empty() && fraction.is_empty() {
        None
    } else {
        Some((written.negative, whole, fraction))
    }
}

/// The oracle's order of two values.
fn oracle(left: &Written, right: &Written) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let sign = |value: &Option<(bool, String, String)>| match value {
        None => 0,
        Some((true, ..)) => -1,
        Some((false, ..)) => 1,
    };
    let (left, right) = (fixed_point(left), fixed_point(right));
    let order = sign(&left).cmp(&sign(&right));
    if order != Ordering::Equal {
        return order;
    }
    let (Some((negative, lw, lf)), Some((_, rw, rf))) = (left, right) else {
        return Ordering::Equal;
    };
    let width = lf.len().max(rf.len());
    let magnitude = lw
        .len()
        .cmp(&rw.len())
        .then_with(|| lw.cmp(&rw))
        .then_with(|| format!("{lf:0<width$}").cmp(&format!("{rf:0<width$}")));
    if negative {
        magnitude.reverse()
    } else {
        magnitude
    }
}

/// A fixed-seed generator.
struct Rng(u64);

impl Rng {
    fn below(&mut self, n: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) % n
    }

    fn digits(&mut self, count: u64) -> String {
        (0..count)
            .map(|_| char::from(b'0' + self.below(10) as u8))
            .collect()
    }

    /// A value over a small domain, so that equal values and near misses are frequent.
    fn written(&mut self) -> Written {
        let wide = self.below(10) == 0;
        let length = 1 + self.below(if wide { 40 } else { 4 });
        let mut digits = self.digits(length);
        if self.below(3) == 0 {
            digits = digits.trim_end_matches('0').to_owned() + &"0".repeat(self.below(4) as usize);
        }
        Written {
            negative: self.below(2) == 0,
            digits,
            exponent: self.below(13) as i64 - 6,
        }
    }

    /// The same value written as JSON in one of several forms, `exponent` written with a sign or
    /// leading zeros now and then.
    fn text(&mut self, value: &Written) -> String {
        let sign = if value.negative { "-" } else { "" };
        let mut digits = value.digits.trim_start_matches('0').to_owned();
        let mut exponent = value.exponent;
        // Trailing zeros moved into the exponent and back.
        let moved = if digits.is_empty() { 0 } else { self.below(3) };
        for _ in 0..moved {
            digits.push('0');
            exponent -= 1;
        }
        if digits.is_empty() {
            digits.push('0');
        }
        let written_exponent = |exponent: i64, rng: &mut Rng| -> String {
            if exponent == 0 && rng.below(2) == 0 {
                return String::new();
            }
            let e = if rng.below(2) == 0 { "e" } else { "E" };
            let sign = match (exponent < 0, rng.below(3)) {
                (true, _) => "-",
                (false, 0) => "+",
                _ => "",
            };
            let zeros = "0".repeat(rng.below(3) as usize);
            format!("{e}{sign}{zeros}{}", exponent.unsigned_abs())
        };
        let fraction_zeros = "0".repeat(self.below(3) as usize);
        match self.below(3) {
            // `digits e exponent`
            0 => {
                let exponent = written_exponent(exponent, self);
                format!("{sign}{digits}{exponent}")
            }
            // `0.digits e (exponent + length)`
            1 => {
                let shifted = exponent + digits.len() as i64;
                let exponent = written_exponent(shifted, self);
                format!("{sign}0.{digits}{fraction_zeros}{exponent}")
            }
            // `d.igits e (exponent + length - 1)`, the point inside the digits
            _ => {
                let (first, rest) = digits.split_at(1);
                let shifted = exponent + rest.len() as i64;
                let exponent = written_exponent(shifted, self);
                if rest.is_empty() && fraction_zeros.is_empty() {
                    format!("{sign}{first}{exponent}")
                } else {
                    format!("{sign}{first}.{rest}{fraction_zeros}{exponent}")
                }
            }
        }
    }
}

/// With the text held: over 300,000 random pairs, two numbers are one `enum` value exactly when
/// the oracle finds them equal, and `type: integer` holds exactly when the oracle finds no
/// fraction; over 100,000 more, `minimum` and `maximum` at a 64-bit integer agree with the oracle.
/// Without `arbitrary_precision` the text is not held and the case asserts only that.
#[test]
fn the_decimal_comparison_agrees_with_an_aligned_digit_oracle() {
    if !holds_text() {
        return;
    }
    let mut rng = Rng(0x00ad_0002);
    let (mut equal, mut integers, mut disagreements) = (0usize, 0usize, Vec::new());
    for round in 0..300_000 {
        let left = rng.written();
        let right = if round % 3 == 0 {
            Written {
                negative: left.negative,
                digits: left.digits.clone(),
                exponent: left.exponent,
            }
        } else {
            rng.written()
        };
        let (left_text, right_text) = (rng.text(&left), rng.text(&right));
        let schema = schema(&format!(
            r#"{{"type": "object", "properties": {{"e": {{"type": "number", "enum": [{left_text}]}}, "i": {{"type": "integer"}}}}}}"#
        ));
        let doc: Value =
            serde_json::from_str(&format!(r#"{{"e": {right_text}, "i": {right_text}}}"#))
                .expect("JSON");
        let found = refusals(Some(&schema), &doc);
        let same = oracle(&left, &right) == std::cmp::Ordering::Equal;
        let integer = fixed_point(&right).is_none_or(|(_, _, fraction)| fraction.is_empty());
        let mut expected = Vec::new();
        if !same {
            expected.push(("/e", "enum"));
        }
        if !integer {
            expected.push(("/i", "type"));
        }
        equal += usize::from(same);
        integers += usize::from(integer);
        if found != pairs(&expected) && disagreements.len() < 5 {
            disagreements.push(format!("{left_text} against {right_text}: {found:?}"));
        }
    }
    let bounds = [
        "0",
        "1",
        "-1",
        "10",
        "-100",
        "123",
        "9007199254740993",
        "-9223372036854775808",
        "18446744073709551615",
    ];
    for _ in 0..100_000 {
        let bound = bounds[rng.below(bounds.len() as u64) as usize];
        let value = if rng.below(4) == 0 {
            let negative = bound.starts_with('-');
            Written {
                negative,
                digits: bound.trim_start_matches('-').to_owned(),
                exponent: 0,
            }
        } else {
            rng.written()
        };
        let text = rng.text(&value);
        let bound_value = Written {
            negative: bound.starts_with('-'),
            digits: bound.trim_start_matches('-').to_owned(),
            exponent: 0,
        };
        let schema = schema(&format!(
            r#"{{"type": "object", "properties": {{"n": {{"type": "number", "minimum": {bound}, "maximum": {bound}}}}}}}"#
        ));
        let doc: Value = serde_json::from_str(&format!(r#"{{"n": {text}}}"#)).expect("JSON");
        let expected = match oracle(&value, &bound_value) {
            std::cmp::Ordering::Less => pairs(&[("/n", "minimum")]),
            std::cmp::Ordering::Equal => Vec::new(),
            std::cmp::Ordering::Greater => pairs(&[("/n", "maximum")]),
        };
        let found = refusals(Some(&schema), &doc);
        if found != expected && disagreements.len() < 10 {
            disagreements.push(format!("{text} against {bound}: {found:?}"));
        }
    }
    assert!(
        equal > 50_000 && integers > 50_000,
        "{equal} equal, {integers} integers"
    );
    assert!(disagreements.is_empty(), "{disagreements:#?}");
}

/// With the text held: exponents beyond 64 and beyond 128 bits on both operands, and digits that
/// differ only at the 400th place.
#[test]
fn huge_exponents_and_long_digits_compare_exactly() {
    if !holds_text() {
        return;
    }
    let huge = "9".repeat(45);
    let one = |text: &str, other: &str| -> bool {
        let schema = schema(&format!(
            r#"{{"type": "object", "properties": {{"n": {{"type": "number", "enum": [{text}]}}}}}}"#
        ));
        let doc: Value = serde_json::from_str(&format!(r#"{{"n": {other}}}"#)).expect("JSON");
        check_arguments(Some(&schema), &doc).is_ok()
    };
    let same = [
        (format!("1e{huge}"), format!("10e{}8", "9".repeat(44))),
        (format!("1e{huge}"), format!("0.1e1{}", "0".repeat(45))),
        (format!("1e-{huge}"), format!("10e-1{}", "0".repeat(45))),
        (
            format!("-25e-{huge}"),
            format!("-2.50e-{}8", "9".repeat(44)),
        ),
        ("1e+0005".to_owned(), "100000".to_owned()),
        ("100e-2".to_owned(), "1".to_owned()),
        ("1.0e1".to_owned(), "10".to_owned()),
        ("0e5".to_owned(), "-0".to_owned()),
        ("0.000".to_owned(), "0e-99999999999999999999".to_owned()),
    ];
    for (left, right) in &same {
        assert!(one(left, right), "{left} is {right}");
    }
    let tail = "0".repeat(398);
    let different = [
        (format!("1.{tail}1"), format!("1.{tail}2")),
        (format!("-1.{tail}1"), format!("-1.{tail}2")),
        (format!("1e{huge}"), format!("1e{}", "9".repeat(44))),
        (format!("1e-{huge}"), "0".to_owned()),
        (format!("-1e-{huge}"), "0".to_owned()),
        (format!("1e{huge}"), format!("-1e{huge}")),
        ("1e-99999999999999999999".to_owned(), "0".to_owned()),
    ];
    for (left, right) in &different {
        assert!(!one(left, right), "{left} is not {right}");
        assert!(!one(right, left), "{right} is not {left}");
    }
    let bound = |bound: &str, text: &str| -> Vec<(String, String)> {
        let schema = schema(&format!(
            r#"{{"type": "object", "properties": {{"n": {{"type": "number", "minimum": {bound}, "maximum": {bound}}}}}}}"#
        ));
        let doc: Value = serde_json::from_str(&format!(r#"{{"n": {text}}}"#)).expect("JSON");
        refusals(Some(&schema), &doc)
    };
    assert_eq!(
        bound("0", &format!("1e-{huge}")),
        pairs(&[("/n", "maximum")])
    );
    assert_eq!(
        bound("0", &format!("-1e-{huge}")),
        pairs(&[("/n", "minimum")])
    );
    assert_eq!(
        bound("-9223372036854775808", &format!("-1e{huge}")),
        pairs(&[("/n", "minimum")])
    );
    assert_eq!(
        bound("18446744073709551615", &format!("1e{huge}")),
        pairs(&[("/n", "maximum")])
    );
    assert_eq!(bound("1", "1e-0"), Vec::new());
    assert_eq!(
        bound("1", &format!("1.{tail}1")),
        pairs(&[("/n", "maximum")])
    );
}
