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

/// The refusal type carries its three parts.
#[test]
fn a_refusal_has_a_pointer_a_keyword_and_a_reason() {
    let refusal = ArgumentRefusal {
        pointer: "/a".to_owned(),
        keyword: "type",
        reason: "r".to_owned(),
    };
    assert_eq!(refusal.clone(), refusal);
}
