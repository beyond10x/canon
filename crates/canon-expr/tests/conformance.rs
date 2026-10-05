use canon_expr::*;
use std::collections::BTreeMap;

#[test]
fn complete_three_valued_tables() {
    let values = [Truth::True, Truth::False, Truth::Unknown];
    let and = [
        [Truth::True, Truth::False, Truth::Unknown],
        [Truth::False, Truth::False, Truth::False],
        [Truth::Unknown, Truth::False, Truth::Unknown],
    ];
    let or = [
        [Truth::True, Truth::True, Truth::True],
        [Truth::True, Truth::False, Truth::Unknown],
        [Truth::True, Truth::Unknown, Truth::Unknown],
    ];
    for (operator, expected) in [("and", and), ("or", or)] {
        let p = compile_one(&format!("probe(\"a\") {operator} probe(\"b\")"));
        for (a, av) in values.iter().enumerate() {
            for (b, bv) in values.iter().enumerate() {
                let evidence = p
                    .requests
                    .iter()
                    .map(|r| {
                        let truth = if r.arguments["name"] == Value::String("a".into()) {
                            av
                        } else {
                            bv
                        };
                        Observation {
                            request_id: r.id.clone(),
                            plan_id: p.id.clone(),
                            observed_at: 10,
                            valid_until: None,
                            outcome: match truth {
                                Truth::True => Outcome::Known(Value::Bool(true)),
                                Truth::False => Outcome::Known(Value::Bool(false)),
                                Truth::Unknown => Outcome::Unavailable("fixture".into()),
                            },
                        }
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    evaluate(&p, &evidence, &context()).unwrap().truth,
                    expected[a][b]
                );
            }
        }
    }
}

#[test]
fn generated_parse_format_parse_and_malformed_corpus() {
    let atoms = [
        "true",
        "false",
        "1 == 1.0",
        "-2 < 1",
        "\"é\\u0061\" == \"éa\"",
        "[1,2][0] == 1",
        "{a: true}.a",
    ];
    for a in atoms {
        for b in atoms {
            for op in ["and", "or", "==", "!="] {
                let source = format!("({a}) {op} ({b})");
                let first = parse(&source).unwrap();
                let canonical = first.canonical_source();
                let second = parse(&canonical).unwrap();
                assert_eq!(canonical, second.canonical_source());
                assert_eq!(truth(&source), truth(&canonical));
            }
        }
    }
    let alphabet = [
        "(", ")", "[", "]", "{", "}", "$", "a", "1", "\"", "\\", "\0", "é", "==", ":",
    ];
    let mut state = 1u64;
    for _ in 0..2000 {
        let mut text = String::new();
        for _ in 0..32 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            text.push_str(alphabet[(state as usize) % alphabet.len()]);
        }
        let _ = parse(&text);
    }
    assert!(parse(&vec!["true"; 2000].join(" and ")).is_err());
    assert!(parse(&format!("{}true{}", "(".repeat(33), ")".repeat(33))).is_err());
}

#[test]
fn canonical_spacing_preserves_source_byte_budget() {
    let literal = format!("\"{}\"", "x".repeat(32765));
    let source = format!("{literal}=={literal}");
    assert_eq!(source.len(), MAX_SOURCE);
    let formatted = parse(&source).unwrap().canonical_source();
    assert!(parse(&formatted).is_ok());
    assert_eq!(truth(&formatted), Truth::True);
}

#[test]
fn numeric_json_is_exact_and_canonical() {
    let text = "170141183460469231731687303715884105727";
    let json = serde_json::from_str(text).unwrap();
    let v = Value::from_json(&json, &Type::Integer).unwrap();
    assert_eq!(v.to_json().unwrap().to_string(), text);
    assert_eq!(
        truth("1e-255 < 2e-255 and -2e-255 < -1e-255 and 0 == -0.000"),
        Truth::True
    );
    assert!(Value::Integer("01".into()).validate().is_err());
    assert!(
        Value::Decimal(Exact {
            coefficient: "10".into(),
            scale: 1
        })
        .validate()
        .is_err()
    );
    let p = compile_one("-(-170141183460469231731687303715884105728) == 1");
    assert!(evaluate(&p, &[], &context()).is_err());
    assert!(compile(&["1e39 == 0".into()], &catalog(), &bindings()).is_err());
}

#[test]
fn signatures_defaults_and_collection_contexts() {
    let mut cat = catalog();
    cat.functions.insert(
        "count".into(),
        Function {
            parameters: vec![Parameter {
                name: "values".into(),
                value_type: Type::List(Box::new(Type::Integer)),
                default: Some(Value::List(vec![])),
            }],
            returns: Type::Integer,
            implementation: Implementation::Pure(Builtin::Length),
        },
    );
    for expr in [
        "count() == 0",
        "count([]) == 0",
        "count(values: [1,2]) == 2",
    ] {
        assert_eq!(
            evaluate(
                &compile(&[expr.into()], &cat, &bindings()).unwrap(),
                &[],
                &context()
            )
            .unwrap()
            .truth,
            Truth::True
        );
    }
    cat.functions.get_mut("count").unwrap().returns = Type::Bool;
    assert!(validate_catalog(&cat).is_err());
    for expr in [
        "probe(name: \"x\", name: \"y\")",
        "probe(name: \"x\", \"y\")",
        "probe(extra: \"x\")",
        "{x:1}.missing == 1",
    ] {
        assert!(compile(&[expr.into()], &catalog(), &bindings()).is_err());
    }
}

#[test]
fn recipe_substitution_and_type_budgets_are_enforced() {
    let mut cat = catalog();
    cat.functions.insert(
        "double".into(),
        Function {
            parameters: vec![Parameter {
                name: "x".into(),
                value_type: Type::Bool,
                default: None,
            }],
            returns: Type::Bool,
            implementation: Implementation::Recipe("$x and $x".into()),
        },
    );
    let mut source = "true".to_owned();
    for _ in 0..20 {
        source = format!("double({source})");
    }
    assert!(compile(&[source], &cat, &bindings()).is_err());
    let mut t = Type::String;
    for _ in 0..100 {
        t = Type::Optional(Box::new(t));
    }
    assert!(t.validate().is_err());
    let mut v = Value::Bool(true);
    for _ in 0..100 {
        v = Value::Optional(Some(Box::new(v)));
    }
    assert!(v.validate().is_err());
}

#[test]
fn malformed_and_stale_evidence_cannot_certify() {
    let p = compile_one("true or probe(\"x\")");
    let r = &p.requests[0];
    let observation = Observation {
        request_id: r.id.clone(),
        plan_id: p.id.clone(),
        observed_at: 10,
        valid_until: None,
        outcome: Outcome::Known(Value::Bool(false)),
    };
    assert!(evaluate(&p, &[observation.clone(), observation.clone()], &context()).is_err());
    let mut bad = observation.clone();
    bad.request_id = "unrequested".into();
    assert!(evaluate(&p, &[bad], &context()).is_err());
    let mut context = context();
    context.source_identity = "dirty-source".into();
    assert_eq!(
        evaluate(&p, &[observation], &context).unwrap().truth,
        Truth::Unknown
    );
    let mut changed = p.clone();
    changed.id = "tampered".into();
    assert!(evaluate(&changed, &[], &context).is_err());
}

#[test]
fn replay_retains_empty_collection_and_optional_contexts() {
    for (ty, value) in [
        (
            Type::Optional(Box::new(Type::Integer)),
            Value::Optional(None),
        ),
        (Type::List(Box::new(Type::String)), Value::List(vec![])),
        (Type::Map(Box::new(Type::Bool)), Value::Map(BTreeMap::new())),
    ] {
        let mut b = bindings();
        b.values.insert("value".into(), value);
        let checked = check(
            parse("$value == $value").unwrap(),
            &catalog(),
            &BTreeMap::from([("value".into(), ty)]),
        )
        .unwrap();
        let plan = plan(checked, &b).unwrap();
        let roundtrip: Plan = serde_json::from_slice(&serde_json::to_vec(&plan).unwrap()).unwrap();
        assert_eq!(
            evaluate(&roundtrip, &[], &context()).unwrap().truth,
            Truth::True
        );
    }
}

#[test]
fn evaluated_and_bound_collection_payloads_obey_limits() {
    let mut b = bindings();
    b.values
        .insert("part".into(), Value::String("x".repeat(40000)));
    let plan = compile(&["[$part,$part] == [$part,$part]".into()], &catalog(), &b).unwrap();
    assert!(evaluate(&plan, &[], &context()).is_err());
    let repeated = vec!["$part"; 100].join(",");
    assert!(compile(&[format!("[{repeated}] == [{repeated}]")], &catalog(), &b).is_err());
}
fn catalog() -> Catalog {
    Catalog {
        format: "canon-catalog/1".into(),
        functions: BTreeMap::from([
            (
                "probe".into(),
                Function {
                    parameters: vec![Parameter {
                        name: "name".into(),
                        value_type: Type::String,
                        default: None,
                    }],
                    returns: Type::Bool,
                    implementation: Implementation::Observation(Provider {
                        provider: "fixture".into(),
                        operation: "probe".into(),
                        digest: "sha256:fixture".into(),
                    }),
                },
            ),
            (
                "size".into(),
                Function {
                    parameters: vec![Parameter {
                        name: "value".into(),
                        value_type: Type::String,
                        default: None,
                    }],
                    returns: Type::Integer,
                    implementation: Implementation::Pure(Builtin::Length),
                },
            ),
        ]),
    }
}
fn bindings() -> Bindings {
    Bindings {
        values: BTreeMap::new(),
        source_identity: "source".into(),
        context_identity: "context".into(),
    }
}
fn context() -> EvalContext {
    EvalContext {
        now: 10,
        source_identity: "source".into(),
        context_identity: "context".into(),
    }
}
fn compile_one(s: &str) -> Plan {
    compile(&[s.into()], &catalog(), &bindings()).unwrap()
}
fn truth(s: &str) -> Truth {
    evaluate(&compile_one(s), &[], &context()).unwrap().truth
}
#[test]
fn syntax_and_exact_numbers() {
    assert_eq!(truth("not 1 == 2 and 0.80 == 0.8"), Truth::True);
    assert_eq!(
        truth("-170141183460469231731687303715884105728 < 0"),
        Truth::True
    );
    assert_eq!(
        truth("170141183460469231731687303715884105727 > 17014118346046923173168730371588410572.7"),
        Truth::True
    );
    assert_eq!(truth("1e-255 > 0"), Truth::True);
    assert_eq!(truth("size(\"hé\") == 2"), Truth::True);
    for bad in [
        "true false",
        "1 < 2 < 3",
        "1e-256 == 0",
        "170141183460469231731687303715884105728 == 0",
        "unknown()",
        "1 and true",
        "\u{e9} == 1",
    ] {
        assert!(
            compile(&[bad.into()], &catalog(), &bindings()).is_err(),
            "{bad}"
        );
    }
}
#[test]
fn kleene_and_invalid_evidence() {
    assert_eq!(truth("false and probe(\"x\")"), Truth::False);
    assert_eq!(truth("true or probe(\"x\")"), Truth::True);
    assert_eq!(truth("true and probe(\"x\")"), Truth::Unknown);
    assert_eq!(truth("not probe(\"x\")"), Truth::Unknown);
    let p = compile_one("false and probe(\"x\")");
    assert_eq!(p.requests.len(), 1);
    let obs = Observation {
        request_id: p.requests[0].id.clone(),
        plan_id: p.id.clone(),
        observed_at: 9,
        valid_until: Some(10),
        outcome: Outcome::Known(Value::String("wrong".into())),
    };
    assert!(evaluate(&p, &[obs], &context()).is_err());
}
#[test]
fn evidence_freshness_identity_dedup_and_replay() {
    let p = compile_one("probe(\"x\") and probe(name: \"x\")");
    assert_eq!(p.requests.len(), 1);
    let mut obs = Observation {
        request_id: p.requests[0].id.clone(),
        plan_id: p.id.clone(),
        observed_at: 9,
        valid_until: Some(10),
        outcome: Outcome::Known(Value::Bool(true)),
    };
    assert_eq!(
        evaluate(&p, &[obs.clone()], &context()).unwrap().truth,
        Truth::True
    );
    obs.valid_until = Some(9);
    assert_eq!(
        evaluate(&p, &[obs.clone()], &context()).unwrap().truth,
        Truth::Unknown
    );
    obs.valid_until = None;
    obs.plan_id = "other".into();
    assert_eq!(
        evaluate(&p, &[obs.clone()], &context()).unwrap().truth,
        Truth::Unknown
    );
    let encoded = serde_json::to_vec(&p).unwrap();
    let replay: Plan = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(
        serde_json::to_vec(&evaluate(&p, &[], &context()).unwrap()).unwrap(),
        serde_json::to_vec(&evaluate(&replay, &[], &context()).unwrap()).unwrap()
    );
    let mut tampered: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
    tampered["id"] = "tampered".into();
    assert!(serde_json::from_value::<Plan>(tampered).is_err());
}
#[test]
fn recipes_are_typed_hygienic_and_bounded() {
    let mut cat = catalog();
    cat.functions.insert(
        "alias".into(),
        Function {
            parameters: vec![Parameter {
                name: "arg".into(),
                value_type: Type::String,
                default: None,
            }],
            returns: Type::Bool,
            implementation: Implementation::Recipe("probe($arg)".into()),
        },
    );
    let p = compile(&["alias(\"x\") and probe(\"x\")".into()], &cat, &bindings()).unwrap();
    assert_eq!(p.requests.len(), 1);
    cat.functions.get_mut("alias").unwrap().implementation =
        Implementation::Recipe("alias($arg)".into());
    assert!(compile(&["alias(\"x\")".into()], &cat, &bindings()).is_err());
    cat.functions.get_mut("alias").unwrap().implementation =
        Implementation::Recipe("probe($ambient)".into());
    assert!(compile(&["alias(\"x\")".into()], &cat, &bindings()).is_err());
    assert!(compile(&[], &catalog(), &bindings()).is_err());
    assert!(
        compile(
            &[format!("{}true{}", "(".repeat(10000), ")".repeat(10000))],
            &catalog(),
            &bindings()
        )
        .is_err()
    );
}
#[test]
fn typed_collections_optional_and_acquisition_dependencies() {
    assert_eq!(truth("{answer: 42}.answer == [41,42][1]"), Truth::True);
    assert_eq!(truth("\"x\" in [\"x\",\"y\"]"), Truth::True);
    assert!(compile(&["probe(probe(\"x\"))".into()], &catalog(), &bindings()).is_err());
    let mut cat = catalog();
    cat.functions.insert(
        "present".into(),
        Function {
            parameters: vec![Parameter {
                name: "value".into(),
                value_type: Type::Optional(Box::new(Type::String)),
                default: None,
            }],
            returns: Type::Bool,
            implementation: Implementation::Pure(Builtin::Present),
        },
    );
    let mut b = bindings();
    b.values.insert("none".into(), Value::Optional(None));
    let parsed = parse("present($none)").unwrap();
    let checked = check(
        parsed,
        &cat,
        &BTreeMap::from([("none".into(), Type::Optional(Box::new(Type::String)))]),
    )
    .unwrap();
    let p = plan(checked, &b).unwrap();
    assert_eq!(evaluate(&p, &[], &context()).unwrap().truth, Truth::False);
}
