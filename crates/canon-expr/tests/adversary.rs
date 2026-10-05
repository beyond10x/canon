use canon_expr::*;
use std::collections::BTreeMap;

fn catalog() -> Catalog {
    Catalog {
        format: "canon-catalog/1".into(),
        functions: BTreeMap::new(),
    }
}

fn bindings() -> Bindings {
    Bindings {
        values: BTreeMap::new(),
        source_identity: "source".into(),
        context_identity: "context".into(),
    }
}

#[test]
fn checked_optional_absence_plan_roundtrips() {
    let mut values = bindings();
    values.values.insert("absent".into(), Value::Optional(None));
    let checked = check(
        parse("$absent == $absent").unwrap(),
        &catalog(),
        &BTreeMap::from([("absent".into(), Type::Optional(Box::new(Type::String)))]),
    )
    .unwrap();
    let original = plan(checked, &values).unwrap();
    let context = EvalContext {
        now: 0,
        source_identity: "source".into(),
        context_identity: "context".into(),
    };
    assert_eq!(
        evaluate(&original, &[], &context).unwrap().truth,
        Truth::True
    );
    let serialized = serde_json::to_vec(&original).unwrap();
    let replay: Plan = serde_json::from_slice(&serialized)
        .expect("a checked valid plan must retain enough type context for replay");
    assert_eq!(evaluate(&replay, &[], &context).unwrap().truth, Truth::True);
}

#[test]
fn canonical_format_preserves_valid_nesting_budget() {
    let source = format!("{}true", "not ".repeat(20));
    let parsed = parse(&source).unwrap();
    let formatted = parsed.canonical_source();
    let reparsed = parse(&formatted)
        .expect("canonical formatter must not make an admitted expression exceed nesting limits");
    assert_eq!(formatted, reparsed.canonical_source());
}

#[test]
fn acquired_argument_cannot_exceed_value_payload_budget() {
    let mut cat = catalog();
    cat.functions.insert(
        "probe".into(),
        Function {
            parameters: vec![Parameter {
                name: "parts".into(),
                value_type: Type::List(Box::new(Type::String)),
                default: None,
            }],
            returns: Type::Bool,
            implementation: Implementation::Observation(Provider {
                provider: "fixture".into(),
                operation: "probe".into(),
                digest: "fixture-v1".into(),
            }),
        },
    );
    let mut b = bindings();
    b.values
        .insert("part".into(), Value::String("x".repeat(40_000)));
    let result = compile(&["probe([$part, $part])".into()], &cat, &b);
    assert!(
        result.is_err(),
        "compiled request contains an argument that Value::validate refuses: {:?}",
        result
            .as_ref()
            .map(|p| p.requests[0].arguments["parts"].validate())
    );
}

#[test]
fn canonical_field_access_does_not_turn_into_a_catalog_name() {
    let mut cat = catalog();
    cat.functions.insert(
        "root".into(),
        Function {
            parameters: vec![],
            returns: Type::Record(BTreeMap::from([("field".into(), Type::Bool)])),
            implementation: Implementation::Recipe("{field: false}".into()),
        },
    );
    cat.functions.insert(
        "root.field".into(),
        Function {
            parameters: vec![],
            returns: Type::Bool,
            implementation: Implementation::Recipe("true".into()),
        },
    );
    let context = EvalContext {
        now: 0,
        source_identity: "source".into(),
        context_identity: "context".into(),
    };
    let original = "(root).field";
    let formatted = parse(original).unwrap().canonical_source();
    let original_plan = compile(&[original.into()], &cat, &bindings()).unwrap();
    assert_eq!(
        evaluate(&original_plan, &[], &context).unwrap().truth,
        Truth::False
    );
    let formatted_plan = compile(&[formatted], &cat, &bindings()).unwrap();
    assert_eq!(
        evaluate(&formatted_plan, &[], &context).unwrap().truth,
        Truth::False,
        "explicit field access must not resolve a different dotted catalog name"
    );
}
