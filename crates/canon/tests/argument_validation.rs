//! story:argument-validator: `b10x_canon::validate::check_arguments` checks one call's arguments
//! against an action's argument schema, in Canon's argument-schema dialect and with JSON Schema
//! 2020-12 meaning, and reports every refusal in document order. Expectations 1 to 8 of the story,
//! over the actions of `fixtures/investigation/action-arguments.yaml`, schemas written here and the
//! fixed-seed generators of `tests/support/schemas.rs`, whose oracle `accepts` the function agrees
//! with.

use std::fs;
use std::path::PathBuf;

use b10x_canon::ir;
use b10x_canon::model;
use b10x_canon::validate::{self, ArgumentRefusal, ArgumentsRefused, check_arguments};
use serde_json::{Value, json};

#[path = "support/schemas.rs"]
mod schemas;

use schemas::{Rng, accepts, document, generate, mutate, object_document};

const FIXTURE: &str = "fixtures/investigation/action-arguments.yaml";

/// The repository root, read at run time.
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set"))
        .join("../..")
}

/// The schema `action` of the investigation fixture declares, as `canon-ir/1` carries it.
fn declared(action: &str) -> Value {
    let source = fs::read_to_string(root().join(FIXTURE)).expect("the fixture reads");
    let compiled = ir::compile(&model::parse(&source).expect("the fixture parses"))
        .expect("the fixture compiles")
        .canonical_json();
    let compiled: Value = serde_json::from_str(&compiled).expect("the IR is JSON");
    compiled["actions"][action]["arguments"].clone()
}

/// What `check_arguments` refuses, as `(pointer, keyword)`; panics if it refuses the schema.
fn refusals(schema: Option<&Value>, arguments: &Value) -> Vec<(String, String)> {
    match check_arguments(schema, arguments) {
        Ok(()) => Vec::new(),
        Err(ArgumentsRefused::Arguments(found)) => {
            assert!(!found.is_empty(), "an empty refusal list");
            found
                .into_iter()
                .map(|refusal| (refusal.pointer, refusal.keyword.to_owned()))
                .collect()
        }
        Err(ArgumentsRefused::Schema(found)) => panic!("the schema is refused: {found:?}"),
    }
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(pointer, keyword)| ((*pointer).to_owned(), (*keyword).to_owned()))
        .collect()
}

/// A property schema, an argument's JSON text and the `(pointer, keyword)` refusals it gives.
type Case<'a> = (&'a str, &'a str, &'a [(&'a str, &'a str)]);

/// Expectation 1: an absent schema is `{"type": "object", "additionalProperties": false}`.
#[test]
fn an_absent_schema_takes_only_the_empty_object() {
    assert_eq!(check_arguments(None, &json!({})), Ok(()));
    assert_eq!(
        refusals(None, &json!({"x": 1})),
        pairs(&[("/x", "additionalProperties")])
    );
    assert_eq!(refusals(None, &json!([])), pairs(&[("", "type")]));
}

/// Expectation 2: the fixture's `inspect`.
#[test]
fn the_fixture_inspect_is_checked_keyword_by_keyword() {
    let schema = declared("inspect");
    let schema = Some(&schema);
    assert_eq!(check_arguments(schema, &json!({"source": "logs"})), Ok(()));
    let cases = [
        (json!({}), ("", "required")),
        (json!({"source": ""}), ("/source", "minLength")),
        (json!({"source": "a", "limit": 0}), ("/limit", "minimum")),
        (json!({"source": "a", "limit": 51}), ("/limit", "maximum")),
        (
            json!({"source": "a", "filters": ["x"]}),
            ("/filters/0", "enum"),
        ),
        (
            json!({"source": "a", "y": 1}),
            ("/y", "additionalProperties"),
        ),
    ];
    for (arguments, (pointer, keyword)) in cases {
        assert_eq!(
            refusals(schema, &arguments),
            pairs(&[(pointer, keyword)]),
            "{arguments}"
        );
    }
}

/// The `required` reason names the missing member, and the `additionalProperties` one the
/// undeclared member.
#[test]
fn a_reason_names_the_member() {
    let schema = declared("inspect");
    let Err(ArgumentsRefused::Arguments(found)) = check_arguments(Some(&schema), &json!({"y": 1}))
    else {
        panic!("refused")
    };
    assert_eq!(found.len(), 2, "{found:?}");
    assert_eq!(
        (found[0].pointer.as_str(), found[0].keyword),
        ("", "required")
    );
    assert!(found[0].reason.contains("`source`"), "{}", found[0].reason);
    assert_eq!(
        (found[1].pointer.as_str(), found[1].keyword),
        ("/y", "additionalProperties")
    );
    assert!(found[1].reason.contains("`y`"), "{}", found[1].reason);
    for refusal in &found {
        assert!(!refusal.reason.contains('\n'), "{refusal:?}");
    }
}

/// Expectation 3: when `type` refuses at a location, no other keyword of the schema is checked
/// there.
#[test]
fn a_type_refusal_hides_the_other_keywords_of_that_schema() {
    let schema = declared("inspect");
    assert_eq!(
        refusals(Some(&schema), &json!({"source": 5})),
        pairs(&[("/source", "type")])
    );
    let object =
        json!({"type": "object", "required": ["a"], "properties": {"a": {"type": "null"}}});
    assert_eq!(refusals(Some(&object), &json!([1])), pairs(&[("", "type")]));
}

/// Expectation 4: an integer is a number with no fractional part; length counts code points.
#[test]
fn integers_and_lengths_have_their_2020_12_meaning() {
    let schema = declared("inspect");
    let schema = Some(&schema);
    let limit = |value: Value| json!({"source": "a", "limit": value});
    assert_eq!(check_arguments(schema, &limit(json!(3.0))), Ok(()));
    assert_eq!(
        refusals(schema, &limit(json!(3.5))),
        pairs(&[("/limit", "type")])
    );
    let wide = "\u{1F600}".repeat(200);
    assert_eq!(wide.len(), 800);
    assert_eq!(check_arguments(schema, &json!({"source": wide})), Ok(()));
    let wider = "\u{1F600}".repeat(201);
    assert_eq!(
        refusals(schema, &json!({"source": wider})),
        pairs(&[("/source", "maxLength")])
    );
}

/// Expectation 5: `minimum` and `maximum` are compared exactly, not through a double.
#[test]
fn bounds_are_compared_exactly_above_two_to_the_53() {
    let schema: Value = serde_json::from_str(
        r#"{"type": "object", "properties": {"n": {"type": "integer", "maximum": 9007199254740993}}}"#,
    )
    .expect("JSON");
    let n = |text: &str| -> Value {
        let arguments: Value = serde_json::from_str(&format!("{{\"n\": {text}}}")).expect("JSON");
        arguments
    };
    assert_eq!(
        check_arguments(Some(&schema), &n("9007199254740993")),
        Ok(())
    );
    assert_eq!(
        refusals(Some(&schema), &n("9007199254740994")),
        pairs(&[("/n", "maximum")])
    );
    assert_eq!(
        check_arguments(Some(&schema), &n("9007199254740992")),
        Ok(())
    );
    let low: Value = serde_json::from_str(
        r#"{"type": "object", "properties": {"n": {"type": "number", "minimum": -9007199254740993}}}"#,
    )
    .expect("JSON");
    assert_eq!(check_arguments(Some(&low), &n("-9007199254740993")), Ok(()));
    assert_eq!(
        refusals(Some(&low), &n("-9007199254740994")),
        pairs(&[("/n", "minimum")])
    );
    assert_eq!(
        refusals(Some(&low), &n("-9007199254740993.5")),
        pairs(&[("/n", "minimum")])
    );
    let number: Value = serde_json::from_str(
        r#"{"type": "object", "properties": {"n": {"type": "number", "minimum": 9007199254740993}}}"#,
    )
    .expect("JSON");
    assert_eq!(
        refusals(Some(&number), &n("9007199254740992.0")),
        pairs(&[("/n", "minimum")])
    );
    assert_eq!(
        check_arguments(Some(&number), &n("9007199254740993")),
        Ok(())
    );
    assert_eq!(
        check_arguments(Some(&number), &n("9007199254740994.0")),
        Ok(())
    );
}

/// `enum` equality is by mathematical value, exactly: `1` and `1.0` are one value, and two
/// integers a double cannot tell apart are two.
#[test]
fn enum_values_are_equal_by_exact_value() {
    let schema: Value = serde_json::from_str(
        r#"{"type": "object", "properties": {"n": {"type": "number", "enum": [1, 9007199254740993]}}}"#,
    )
    .expect("JSON");
    let n = |text: &str| -> Value {
        serde_json::from_str(&format!("{{\"n\": {text}}}")).expect("JSON")
    };
    assert_eq!(check_arguments(Some(&schema), &n("1.0")), Ok(()));
    assert_eq!(
        check_arguments(Some(&schema), &n("9007199254740993")),
        Ok(())
    );
    assert_eq!(
        refusals(Some(&schema), &n("9007199254740992.0")),
        pairs(&[("/n", "enum")])
    );
}

/// Whether serde_json keeps a number's text (its `arbitrary_precision` feature): on in `cargo test
/// --workspace`, where `b10x-canon-expr` turns it on, off in `cargo test -p b10x-canon`.
fn holds_text() -> bool {
    "0.10"
        .parse::<serde_json::Number>()
        .is_ok_and(|number| number.to_string() == "0.10")
}

/// `{"n": <text>}` as serde_json reads it; `None` when it refuses the text.
fn read_n(text: &str) -> Option<Value> {
    serde_json::from_str(&format!("{{\"n\": {text}}}")).ok()
}

/// A top-level schema whose property `n` is the JSON text `property`.
fn with_n(property: &str) -> Value {
    serde_json::from_str(&format!(
        r#"{{"type": "object", "properties": {{"n": {property}}}}}"#
    ))
    .expect("JSON")
}

/// What `check_arguments` refuses of `{"n": <text>}` under `with_n(property)`.
fn refusals_of(property: &str, text: &str) -> Vec<(String, String)> {
    let arguments = read_n(text).unwrap_or_else(|| panic!("serde_json reads {text}"));
    refusals(Some(&with_n(property)), &arguments)
}

/// Where serde_json keeps the text, a number is its exact decimal value; where it does not, it is
/// the double serde_json reads (`9007199254740992.5` is `9007199254740992`).
#[test]
fn a_written_fraction_is_its_exact_value_where_the_text_is_held() {
    let cases = [
        (
            r#"{"type": "number", "maximum": 9007199254740992}"#,
            "9007199254740992.5",
            "maximum",
        ),
        (r#"{"type": "integer"}"#, "9007199254740992.5", "type"),
    ];
    for (property, text, keyword) in cases {
        let expected = if holds_text() {
            pairs(&[("/n", keyword)])
        } else {
            Vec::new()
        };
        assert_eq!(
            refusals_of(property, text),
            expected,
            "{text} under {property}"
        );
    }
    let cases = [
        (
            r#"{"type": "number", "minimum": 9007199254740993}"#,
            "minimum",
        ),
        (r#"{"type": "number", "enum": [9007199254740993]}"#, "enum"),
    ];
    for (property, keyword) in cases {
        let expected = if holds_text() {
            Vec::new()
        } else {
            pairs(&[("/n", keyword)])
        };
        assert_eq!(
            refusals_of(property, "9007199254740993.0"),
            expected,
            "under {property}"
        );
    }
}

/// An integer literal beyond the 64-bit range is that integer where the text is held, and the
/// nearest double where it is not.
#[test]
fn an_integer_beyond_64_bits_is_exact_where_the_text_is_held() {
    let below = refusals_of(
        r#"{"type": "integer", "minimum": -9223372036854775808}"#,
        "-9223372036854775809",
    );
    let expected = if holds_text() {
        pairs(&[("/n", "minimum")])
    } else {
        Vec::new() // the double -2^63, equal to the bound
    };
    assert_eq!(below, expected);
    assert_eq!(
        refusals_of(
            r#"{"type": "integer", "maximum": 18446744073709551615}"#,
            "18446744073709551616"
        ),
        pairs(&[("/n", "maximum")])
    );
    let written = refusals_of(
        r#"{"type": "number", "enum": [18446744073709551615]}"#,
        "18446744073709551615.0",
    );
    let expected = if holds_text() {
        Vec::new()
    } else {
        pairs(&[("/n", "enum")]) // the double 2^64
    };
    assert_eq!(written, expected);
}

/// `1e400` has no finite double: where the text is held it is above every finite bound, an
/// integer, and equal only to a number of the same value; where it is not, serde_json refuses it.
/// `1e-400` is above 0 where the text is held, and the double 0 where it is not.
#[test]
fn a_number_beyond_the_double_range_is_its_exact_value_where_the_text_is_held() {
    if !holds_text() {
        assert_eq!(read_n("1e400"), None);
        assert_eq!(read_n("-1e400"), None);
        assert_eq!(
            refusals_of(r#"{"type": "integer", "maximum": 0}"#, "1e-400"),
            Vec::new()
        );
        return;
    }
    let cases: [Case; 10] = [
        (
            r#"{"type": "number", "maximum": 5}"#,
            "1e400",
            &[("/n", "maximum")],
        ),
        (r#"{"type": "number", "minimum": 0}"#, "1e400", &[]),
        (
            r#"{"type": "number", "minimum": 0}"#,
            "-1e400",
            &[("/n", "minimum")],
        ),
        (r#"{"type": "number", "maximum": 0}"#, "-1E+400", &[]),
        (
            r#"{"type": "number", "enum": [1.5]}"#,
            "1e400",
            &[("/n", "enum")],
        ),
        (r#"{"type": "integer"}"#, "1e400", &[]),
        (r#"{"type": "number", "enum": [1e400]}"#, "10e399", &[]),
        (
            r#"{"type": "number", "enum": [1e400]}"#,
            "1e401",
            &[("/n", "enum")],
        ),
        (
            r#"{"type": "number", "maximum": 0}"#,
            "1e-400",
            &[("/n", "maximum")],
        ),
        (r#"{"type": "integer"}"#, "1e-400", &[("/n", "type")]),
    ];
    for (property, text, expected) in cases {
        assert_eq!(
            refusals_of(property, text),
            pairs(expected),
            "{text} under {property}"
        );
    }
}

/// `canon validate`'s duplicate check uses the same comparison: `9007199254740993` and
/// `9007199254740993.0` are one value where the text is held, and two where it is not (the second
/// is the double 2^53); `9007199254740993` and `9007199254740992.0` are two in both.
#[test]
fn an_enum_lists_each_exact_value_once() {
    let schema = with_n(r#"{"type": "number", "enum": [9007199254740993, 9007199254740993.0]}"#);
    let found = check_arguments(Some(&schema), &json!({"n": 9_007_199_254_740_993_u64}));
    let expected = if holds_text() {
        Err(ArgumentsRefused::Schema(vec![(
            "/properties/n/enum/1".to_owned(),
            "`9007199254740993.0` is listed more than once".to_owned(),
        )]))
    } else {
        Ok(())
    };
    assert_eq!(found, expected);
    let distinct = with_n(r#"{"type": "number", "enum": [9007199254740993, 9007199254740992.0]}"#);
    assert_eq!(
        check_arguments(Some(&distinct), &json!({"n": 9_007_199_254_740_993_u64})),
        Ok(())
    );
    if holds_text() {
        // serde_json writes the exponent it holds with its sign: `10e399` is shown `10e+399`.
        let same = with_n(r#"{"type": "number", "enum": [1e400, 10e399]}"#);
        assert_eq!(
            check_arguments(Some(&same), &json!({})),
            Err(ArgumentsRefused::Schema(vec![(
                "/properties/n/enum/1".to_owned(),
                "`10e+399` is listed more than once".to_owned(),
            )]))
        );
    }
}

/// Values both builds hold alike, above 2^53 and beyond the 64-bit range, compared exactly.
#[test]
fn doubles_beyond_two_to_the_53_are_compared_exactly_in_both_builds() {
    let cases: [Case; 7] = [
        (
            r#"{"type": "integer", "maximum": 9007199254740993}"#,
            "9007199254740994.0",
            &[("/n", "maximum")],
        ),
        (
            r#"{"type": "integer", "maximum": 9007199254740994}"#,
            "9007199254740994.0",
            &[],
        ),
        (
            r#"{"type": "integer", "minimum": -9223372036854775808}"#,
            "-9223372036854777856.0",
            &[("/n", "minimum")],
        ),
        (
            r#"{"type": "number", "maximum": 18446744073709551615}"#,
            "1e300",
            &[("/n", "maximum")],
        ),
        (
            r#"{"type": "number", "minimum": -9223372036854775808}"#,
            "-1e300",
            &[("/n", "minimum")],
        ),
        (r#"{"type": "integer"}"#, "1e300", &[]),
        (
            r#"{"type": "number", "enum": [9007199254740994]}"#,
            "9007199254740994.0",
            &[],
        ),
    ];
    for (property, text, expected) in cases {
        assert_eq!(
            refusals_of(property, text),
            pairs(expected),
            "{text} under {property}"
        );
    }
}

/// The protocol `act` whose `arguments` is the YAML text `schema`, compiled: the schema as
/// `canon-ir/1` holds it, and the IR's canonical JSON text.
fn compiled(schema: &str) -> (Value, String) {
    let text = format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nactions:\n  act:\n    arguments: {schema}\n"
    );
    let ir = ir::compile(&model::parse(&text).expect("parses")).expect("compiles");
    let arguments = ir
        .actions
        .values()
        .next()
        .and_then(|action| action.arguments.clone());
    (arguments.expect("declared"), ir.canonical_json())
}

/// D4: a protocol number whose double is an integer in the 64-bit range, and whose shortest text
/// is another value, is held as that integer. `1152921504606846976.0` is 2^60 exactly, and its
/// shortest text `1.152921504606847e+18` is not: the schema holds 2^60, the IR writes it, and the
/// arguments 2^60 are its `enum` value in both builds.
#[test]
fn a_protocol_number_a_double_holds_exactly_is_that_integer() {
    let (schema, ir) =
        compiled("{type: object, properties: {n: {type: number, enum: [1152921504606846976.0]}}}");
    assert_eq!(
        schema["properties"]["n"]["enum"][0],
        json!(1_152_921_504_606_846_976_u64)
    );
    assert!(ir.contains("1152921504606846976"), "{ir}");
    assert_eq!(
        check_arguments(Some(&schema), &json!({"n": 1_152_921_504_606_846_976_u64})),
        Ok(())
    );
    let (schema, _) = compiled(
        "{type: object, properties: {n: {type: integer, minimum: -9223372036854775808.0, \
         enum: [1152921504606846976.0, 18446744073709549568.0]}}}",
    );
    assert_eq!(
        schema["properties"]["n"],
        json!({"type": "integer", "minimum": i64::MIN,
               "enum": [1_152_921_504_606_846_976_u64, 18_446_744_073_709_549_568_u64]})
    );
}

/// D4 changes no number whose shortest text is its value: integers up to 2^53, fractions, and the
/// doubles above 2^53 whose shortest text is exact keep the IR text they had.
#[test]
fn a_protocol_number_whose_shortest_text_is_exact_keeps_it() {
    let (_, ir) = compiled(
        "{type: object, properties: {n: {type: number, \
         enum: [3.0, -0.0, 1.5, 9007199254740992.0, 9007199254740994.0, 1e19]}}}",
    );
    let written = "[\n              3.0,\n              -0.0,\n              1.5,\n              \
                   9007199254740992.0,\n              9007199254740994.0,\n              1e+19\n";
    assert!(ir.contains(written), "{ir}");
}

/// A number a caller makes with `Number::from_f64` is not read from a protocol, so D4 does not
/// reach it: it is the double where the text is not held, and its shortest text where it is (2^60
/// is written `1.152921504606847e+18`, which is not 2^60). The module docs name this difference.
#[test]
fn a_number_a_caller_makes_from_a_double_is_the_double_or_its_shortest_text() {
    let double = serde_json::Number::from_f64(1_152_921_504_606_846_976.0).expect("finite");
    let schema =
        json!({"type": "object", "properties": {"n": {"type": "number", "enum": [double]}}});
    let found = refusals(Some(&schema), &json!({"n": 1_152_921_504_606_846_976_u64}));
    let expected = if holds_text() {
        pairs(&[("/n", "enum")])
    } else {
        Vec::new()
    };
    assert_eq!(found, expected);
}

/// A bound or an `integer` `enum` value written `-0` is the integer 0 where the text is held, and
/// the double -0.0, not an integer, where it is not: the third difference the module docs name.
#[test]
fn a_bound_written_minus_zero_is_an_integer_only_where_the_text_is_held() {
    for (property, pointer, keyword) in [
        (
            r#"{"type": "integer", "minimum": -0}"#,
            "/properties/n/minimum",
            "`minimum` is not an integer",
        ),
        (
            r#"{"type": "integer", "enum": [-0]}"#,
            "/properties/n/enum/0",
            "`-0.0` is not of `type: integer`",
        ),
    ] {
        let found = check_arguments(Some(&with_n(property)), &json!({"n": 0}));
        let expected = if holds_text() {
            Ok(())
        } else {
            Err(ArgumentsRefused::Schema(vec![(
                pointer.to_owned(),
                keyword.to_owned(),
            )]))
        };
        assert_eq!(found, expected, "{property}");
    }
}

/// Expectation 6: every refusal is reported, in document order with members in code-point order;
/// a location's own refusals come before its members', and the same input gives the same result.
#[test]
fn every_refusal_is_reported_in_document_order() {
    let schema = declared("inspect");
    let arguments = json!({"source": "", "limit": 0, "filters": ["x", "log", "y"], "z": 1});
    let first = check_arguments(Some(&schema), &arguments);
    assert_eq!(first, check_arguments(Some(&schema), &arguments));
    let Err(ArgumentsRefused::Arguments(found)) = first else {
        panic!("refused")
    };
    let seen: Vec<(&str, &str)> = found
        .iter()
        .map(|refusal| (refusal.pointer.as_str(), refusal.keyword))
        .collect();
    assert_eq!(
        seen,
        [
            ("/filters/0", "enum"),
            ("/filters/2", "enum"),
            ("/limit", "minimum"),
            ("/source", "minLength"),
            ("/z", "additionalProperties"),
        ]
    );
    let both = check_arguments(Some(&schema), &json!({"source": "", "limit": 0}));
    let Err(ArgumentsRefused::Arguments(both)) = both else {
        panic!("refused")
    };
    assert_eq!(
        both.iter()
            .map(|refusal| refusal.pointer.as_str())
            .collect::<Vec<_>>(),
        ["/limit", "/source"]
    );
    let own_first = check_arguments(Some(&schema), &json!({"y": 1}));
    let Err(ArgumentsRefused::Arguments(own_first)) = own_first else {
        panic!("refused")
    };
    assert_eq!(
        own_first
            .iter()
            .map(|refusal| (refusal.pointer.as_str(), refusal.keyword))
            .collect::<Vec<_>>(),
        [("", "required"), ("/y", "additionalProperties")]
    );
    let keywords = json!({"type": "object", "properties": {"s": {
        "type": "string", "enum": ["abc"], "minLength": 4, "maxLength": 9
    }}});
    let Err(ArgumentsRefused::Arguments(keywords)) =
        check_arguments(Some(&keywords), &json!({"s": "ab"}))
    else {
        panic!("refused")
    };
    assert_eq!(
        keywords
            .iter()
            .map(|refusal| refusal.keyword)
            .collect::<Vec<_>>(),
        ["enum", "minLength"]
    );
}

/// Expectation 7: a schema outside the dialect is returned as the pairs `canon validate` reports,
/// and no argument is checked.
#[test]
fn a_schema_outside_the_dialect_is_returned_as_validate_reports_it() {
    let schema = json!({"type": "object", "$ref": "#"});
    let expected = ArgumentsRefused::Schema(vec![(
        "/$ref".to_owned(),
        "`$ref` is outside the dialect".to_owned(),
    )]);
    assert_eq!(
        check_arguments(Some(&schema), &json!({"x": 1})),
        Err(expected)
    );
    assert_eq!(
        check_arguments(Some(&schema), &json!({})),
        Err(ArgumentsRefused::Schema(vec![(
            "/$ref".to_owned(),
            "`$ref` is outside the dialect".to_owned(),
        )]))
    );
    let text = format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nactions:\n  act:\n    arguments: {schema}\n"
    );
    let problems =
        validate::validate(&model::parse(&text).expect("parses")).expect_err("validate refuses it");
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert_eq!(
        problems[0].to_string(),
        "action `act` arguments at `/$ref`: `$ref` is outside the dialect"
    );
}

/// The dialect's own schema problems, as `canon validate` reports them for `schema`.
fn schema_problems(schema: &Value) -> Vec<(String, String)> {
    let text = format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nactions:\n  act:\n    arguments: {schema}\n"
    );
    let protocol = model::parse(&text).unwrap_or_else(|error| panic!("{schema} parses: {error}"));
    match validate::validate(&protocol) {
        Ok(()) => Vec::new(),
        Err(problems) => problems
            .iter()
            .map(|problem| {
                let line = problem.to_string();
                let rest = line
                    .strip_prefix("action `act` arguments at `")
                    .unwrap_or_else(|| panic!("{line}"));
                let (pointer, reason) = rest.split_once("`: ").unwrap_or_else(|| panic!("{line}"));
                (pointer.to_owned(), reason.to_owned())
            })
            .collect(),
    }
}

/// Expectation 8: over fixed-seed random schemas in the dialect and random documents,
/// `check_arguments` is `Ok` exactly when the oracle accepts, and a schema outside the dialect is
/// returned as `canon validate` reports it.
#[test]
fn check_arguments_agrees_with_the_oracle_on_random_schemas() {
    let mut rng = Rng(0x00c0_ffee_1234_5678);
    let (mut schemas_compared, mut outside, mut documents, mut accepted) = (0usize, 0usize, 0, 0);
    let mut disagreements = Vec::new();
    let mut check = |schema: &Value, rng: &mut Rng| {
        let problems = schema_problems(schema);
        if !problems.is_empty() {
            outside += 1;
            let refused = check_arguments(Some(schema), &json!({}));
            if refused != Err(ArgumentsRefused::Schema(problems.clone())) {
                disagreements.push(format!("{schema}: {refused:?} against {problems:?}"));
            }
            return;
        }
        schemas_compared += 1;
        for round in 0..300 {
            let doc = if round % 10 == 9 {
                document(rng, 0)
            } else {
                object_document(rng, 1)
            };
            let found = check_arguments(Some(schema), &doc);
            let expected = accepts(schema, &doc);
            documents += 1;
            accepted += usize::from(expected);
            let agrees = match &found {
                Ok(()) => expected,
                Err(ArgumentsRefused::Arguments(list)) => {
                    !expected && !list.is_empty() && found == check_arguments(Some(schema), &doc)
                }
                Err(ArgumentsRefused::Schema(_)) => false,
            };
            if !agrees {
                disagreements.push(format!(
                    "{schema} on {doc}: {found:?}, the oracle accepts {expected}"
                ));
                break;
            }
        }
    };
    for round in 0..4000 {
        let from = generate(&mut rng, 1, true);
        check(&from, &mut rng);
        if round % 4 != 0 {
            let to = mutate(&mut rng, &from, 1, true);
            check(&to, &mut rng);
        }
    }
    eprintln!(
        "compared {schemas_compared} schemas ({outside} outside the dialect) on {documents} \
         documents, {accepted} accepted"
    );
    assert!(
        schemas_compared > 1000,
        "only {schemas_compared} schemas compared"
    );
    assert!(accepted > 1000, "only {accepted} documents accepted");
    assert!(
        disagreements.is_empty(),
        "{} disagreements; first: {:#?}",
        disagreements.len(),
        &disagreements[..disagreements.len().min(3)]
    );
}

/// A refusal carries the escaped pointer, the keyword and the reason, and prints them on one line.
#[test]
fn a_refusal_has_a_pointer_a_keyword_and_a_reason() {
    let refused = check_arguments(None, &json!({"a/b": 1}));
    assert_eq!(
        refused,
        Err(ArgumentsRefused::Arguments(vec![ArgumentRefusal {
            pointer: "/a~1b".to_owned(),
            keyword: "additionalProperties",
            reason: "`a/b` is not declared in `properties`".to_owned(),
        }]))
    );
    assert_eq!(
        refused.expect_err("refused").to_string(),
        "arguments at `/a~1b`: `a/b` is not declared in `properties`"
    );
}
