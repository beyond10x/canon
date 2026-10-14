//! story:action-arguments: an action's optional `arguments`, a JSON Schema in Canon's
//! argument-schema dialect. `canon validate` accepts the investigation fixture whose actions carry
//! schemas and refuses every schema outside the dialect as `action-arguments`, one problem per
//! offending keyword, naming the action and the keyword's JSON Pointer; `canon compile` carries
//! each schema unchanged into `canon-ir/1`, and writes no `arguments` for an action without one.

use std::fs;
use std::path::PathBuf;

use b10x_canon::diff;
use b10x_canon::eval;
use b10x_canon::ir;
use b10x_canon::model;
use serde_json::{Value, json};

const FIXTURE: &str = "fixtures/investigation/action-arguments.yaml";

/// The repository root, read at run time: a test binary reused from a shared build directory
/// would otherwise read the tree it was built from.
fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set"))
        .join("../..")
}

fn fixture() -> String {
    fs::read_to_string(root().join(FIXTURE)).expect("the fixture reads")
}

/// A protocol with one action `act` whose `arguments` is `schema`, written as flow YAML.
fn with_schema(schema: &str) -> String {
    format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\n\
         actions:\n  act:\n    arguments: {schema}\n"
    )
}

/// Every problem of `text` as `(code, message)`; empty when it compiles.
fn problems(text: &str) -> Vec<(String, String)> {
    let parsed = model::parse(text).unwrap_or_else(|error| panic!("parses: {error}\n{text}"));
    match ir::compile(&parsed) {
        Ok(_) => Vec::new(),
        Err(problems) => problems
            .iter()
            .map(|problem| (problem.code().to_owned(), problem.to_string()))
            .collect(),
    }
}

fn refused(schema: &str) -> Vec<(String, String)> {
    problems(&with_schema(schema))
}

fn one(message: &str) -> Vec<(String, String)> {
    vec![("action-arguments".to_owned(), message.to_owned())]
}

fn compiled_json(text: &str) -> String {
    ir::compile(&model::parse(text).expect("parses"))
        .expect("compiles")
        .canonical_json()
}

#[test]
fn the_fixture_validates() {
    assert_eq!(problems(&fixture()), Vec::new());
}

/// The acceptance's refusals, each one problem naming the action, the pointer and the reason.
#[test]
fn the_acceptance_refusals_name_the_action_and_the_pointer() {
    let cases: &[(&str, &str)] = &[
        (
            "{type: string}",
            "action `act` arguments at `/type`: the top-level schema is `type: string`; it must be `type: object`",
        ),
        (
            "{type: object, properties: {a: {type: string, $ref: '#/$defs/a'}}}",
            "action `act` arguments at `/properties/a/$ref`: `$ref` is outside the dialect",
        ),
        (
            "{type: object, $ref: 'https://example.com/arguments.json'}",
            "action `act` arguments at `/$ref`: `$ref` is outside the dialect",
        ),
        (
            "{type: object, properties: {a: {type: string, anyOf: [{type: string}]}}}",
            "action `act` arguments at `/properties/a/anyOf`: `anyOf` is outside the dialect",
        ),
        (
            "{type: object, properties: {a: {description: no type}}}",
            "action `act` arguments at `/properties/a`: the schema names no `type`",
        ),
        (
            "{type: object, properties: {a: {type: string}}, required: [a, b]}",
            "action `act` arguments at `/required/1`: `b` is not declared in `properties`",
        ),
        (
            "{type: object, properties: {n: {type: integer, minimum: 5, maximum: 2}}}",
            "action `act` arguments at `/properties/n/minimum`: `minimum` 5 is above `maximum` 2",
        ),
        (
            "{type: object, properties: {s: {type: string, minimum: 1}}}",
            "action `act` arguments at `/properties/s/minimum`: `minimum` does not apply to `type: string`",
        ),
        (
            "{type: object, frobnicate: true}",
            "action `act` arguments at `/frobnicate`: `frobnicate` is not a keyword of the dialect",
        ),
    ];
    for (schema, message) in cases {
        assert_eq!(refused(schema), one(message), "{schema}");
    }
}

/// Every keyword the story lists as refused is refused where it is written, naming its pointer.
#[test]
fn every_keyword_outside_the_dialect_is_refused_at_its_pointer() {
    for keyword in [
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
    ] {
        let schema =
            format!("{{type: object, properties: {{a: {{type: string, '{keyword}': x}}}}}}");
        let found = refused(&schema);
        assert_eq!(found.len(), 1, "{keyword}: {found:?}");
        let (code, message) = &found[0];
        assert_eq!(code, "action-arguments", "{keyword}");
        assert!(
            message.starts_with(&format!(
                "action `act` arguments at `/properties/a/{keyword}`: "
            )),
            "{keyword}: {message}"
        );
    }
}

/// A list-valued `type` and a schema-valued `additionalProperties` are outside the dialect.
#[test]
fn a_list_valued_type_and_a_schema_valued_additional_properties_are_refused() {
    let found = refused("{type: object, properties: {a: {type: [string, 'null']}}}");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0]
            .1
            .starts_with("action `act` arguments at `/properties/a/type`: ")
    );
    let found = refused("{type: object, additionalProperties: {type: string}}");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0]
            .1
            .starts_with("action `act` arguments at `/additionalProperties`: ")
    );
}

/// Bounds, enums, names and annotations outside the dialect, each refused at its own pointer.
#[test]
fn malformed_values_of_dialect_keywords_are_refused_at_their_pointers() {
    for (schema, pointer) in [
        (
            "{type: object, properties: {s: {type: string, minLength: -1}}}",
            "/properties/s/minLength",
        ),
        (
            "{type: object, properties: {s: {type: string, maxLength: 1.5}}}",
            "/properties/s/maxLength",
        ),
        (
            "{type: object, properties: {s: {type: string, minLength: 3, maxLength: 2}}}",
            "/properties/s/minLength",
        ),
        (
            "{type: object, properties: {l: {type: array, items: {type: string}, minItems: 3, maxItems: 1}}}",
            "/properties/l/minItems",
        ),
        (
            "{type: object, properties: {l: {type: array, items: 5}}}",
            "/properties/l/items",
        ),
        (
            "{type: object, properties: {n: {type: number, minimum: 0.5}}}",
            "/properties/n/minimum",
        ),
        (
            "{type: object, properties: {e: {type: string, enum: []}}}",
            "/properties/e/enum",
        ),
        (
            "{type: object, properties: {e: {type: string, enum: [a, a]}}}",
            "/properties/e/enum/1",
        ),
        (
            "{type: object, properties: {e: {type: string, enum: [a, 1]}}}",
            "/properties/e/enum/1",
        ),
        (
            "{type: object, properties: {e: {type: integer, enum: [1, [2]]}}}",
            "/properties/e/enum/1",
        ),
        ("{type: object, title: 5}", "/title"),
        ("{type: object, properties: [a]}", "/properties"),
        ("{type: object, properties: {a: 5}}", "/properties/a"),
        (
            "{type: object, properties: {a: {type: string}}, required: [a, a]}",
            "/required/1",
        ),
        ("{type: object, required: a}", "/required"),
        (
            "{type: object, additionalProperties: 'no'}",
            "/additionalProperties",
        ),
        (
            "{type: object, properties: {a: {type: text}}}",
            "/properties/a/type",
        ),
        (
            "{type: object, properties: {'a/b~c': {type: string, oneOf: []}}}",
            "/properties/a~1b~0c/oneOf",
        ),
        (
            "{type: object, properties: {l: {type: array, items: {type: object, properties: {x: {type: string, maxItems: 1}}}}}}",
            "/properties/l/items/properties/x/maxItems",
        ),
    ] {
        let found = refused(schema);
        assert_eq!(found.len(), 1, "{schema}: {found:?}");
        assert_eq!(found[0].0, "action-arguments", "{schema}");
        assert!(
            found[0]
                .1
                .starts_with(&format!("action `act` arguments at `{pointer}`: ")),
            "{schema}: {}",
            found[0].1
        );
    }
}

/// A schema that is not a mapping at all is refused at the root, and several offending keywords
/// give one problem each, in document order.
#[test]
fn every_offending_keyword_is_its_own_problem_in_document_order() {
    assert_eq!(
        refused("5"),
        one("action `act` arguments: the schema is not an object")
    );
    let found: Vec<String> = refused(
        "{type: object, properties: {b: {type: string, pattern: x}, a: {type: integer, format: y}}, allOf: []}",
    )
    .into_iter()
    .map(|(code, message)| {
        assert_eq!(code, "action-arguments");
        message
    })
    .collect();
    let pointers: Vec<&str> = found
        .iter()
        .map(|message| message.split('`').nth(3).expect("a pointer"))
        .collect();
    assert_eq!(
        pointers,
        ["/allOf", "/properties/a/format", "/properties/b/pattern"],
        "{found:?}"
    );
}

/// A null written on `arguments` is refused like every other optional key's.
#[test]
fn an_explicit_null_is_not_an_absent_schema() {
    let error = model::parse(&with_schema("~")).expect_err("a null schema is refused");
    assert!(error.to_string().contains("explicit null"), "{error}");
}

/// `canon compile` writes each action's schema as declared, as JSON, and no `arguments` key for an
/// action without one; the bytes are canonical, so `read_ir` reads them back.
#[test]
fn the_ir_carries_each_schema_as_declared() {
    let source = fixture();
    let text = compiled_json(&source);
    assert_eq!(text, compiled_json(&source), "the IR is deterministic");
    let declared: Value = serde_yaml_ng::from_str(&source).expect("the fixture is YAML");
    let compiled: Value = serde_json::from_str(&text).expect("the IR is JSON");
    for action in ["inspect", "attempt_falsification"] {
        assert_eq!(
            compiled["actions"][action]["arguments"], declared["actions"][action]["arguments"],
            "{action}"
        );
    }
    let publish = compiled["actions"]["publish_finding"]
        .as_object()
        .expect("an action is an object");
    assert!(!publish.contains_key("arguments"), "{publish:?}");
    assert_eq!(
        compiled["actions"]["attempt_falsification"]["arguments"]["properties"]["method"]["properties"]
            ["confidence"],
        json!({"type": "number", "minimum": 0, "maximum": 1})
    );
    let read = eval::read_ir(&text).expect("read_ir reads what compile prints");
    assert_eq!(read.canonical_json(), text);
}

/// A protocol without any `arguments` compiles to the IR it compiled to before the field existed:
/// no action has the key.
#[test]
fn an_action_without_arguments_has_no_key() {
    let text = compiled_json(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nactions: {act: {description: d}}\n",
    );
    assert!(!text.contains("\"arguments\""), "{text}");
}

/// Whatever the classifier says about a changed schema, `canon diff` never reports it as no
/// change, and reports nothing for an unchanged one.
#[test]
fn a_changed_schema_is_never_silent() {
    let compile = |schema: &str, revision: u32| {
        let text = with_schema(schema).replace("revision: 1", &format!("revision: {revision}"));
        ir::compile(&model::parse(&text).expect("parses")).expect("compiles")
    };
    let from = compile("{type: object, properties: {a: {type: string}}}", 1);
    let same = compile("{type: object, properties: {a: {type: string}}}", 2);
    let to = compile(
        "{type: object, properties: {a: {type: string}}, required: [a]}",
        2,
    );
    assert!(
        diff::diff(&from, &same)
            .expect("compares")
            .changes
            .is_empty()
    );
    let changes = diff::diff(&from, &to).expect("compares").changes;
    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(changes[0].section, "action");
    assert_eq!(changes[0].id, "act");
    assert!(changes[0].what.starts_with("arguments "), "{changes:?}");
}
