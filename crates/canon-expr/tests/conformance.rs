use canon_expr::*;
use std::collections::BTreeMap;
fn catalog() -> Catalog {
    Catalog { format: "canon-catalog/1".into(), functions: BTreeMap::from([
      ("probe".into(), Function { parameters: vec![Parameter { name: "name".into(), value_type: Type::String, default: None }], returns: Type::Bool, implementation: Implementation::Observation(Provider { provider: "fixture".into(), operation: "probe".into(), digest: "sha256:fixture".into() }) }),
      ("size".into(), Function { parameters: vec![Parameter { name: "value".into(), value_type: Type::String, default: None }], returns: Type::Integer, implementation: Implementation::Pure(Builtin::Length) }),
    ]) }
}
fn bindings() -> Bindings { Bindings { values: BTreeMap::new(), source_identity: "source".into(), context_identity: "context".into() } }
fn context() -> EvalContext { EvalContext { now: 10, source_identity:"source".into(), context_identity:"context".into() } }
fn compile_one(s: &str) -> Plan { compile(&[s.into()], &catalog(), &bindings()).unwrap() }
fn truth(s: &str) -> Truth { evaluate(&compile_one(s), &[], &context()).unwrap().truth }
#[test] fn syntax_and_exact_numbers() {
 assert_eq!(truth("not 1 == 2 and 0.80 == 0.8"), Truth::True);
 assert_eq!(truth("-170141183460469231731687303715884105728 < 0"), Truth::True);
 assert_eq!(truth("170141183460469231731687303715884105727 > 17014118346046923173168730371588410572.7"), Truth::True);
 assert_eq!(truth("1e-255 > 0"), Truth::True);
 assert_eq!(truth("size(\"hé\") == 2"), Truth::True);
 for bad in ["true false", "1 < 2 < 3", "1e-256 == 0", "170141183460469231731687303715884105728 == 0", "unknown()", "1 and true", "\u{e9} == 1"] { assert!(compile(&[bad.into()], &catalog(), &bindings()).is_err(), "{bad}"); }
}
#[test] fn kleene_and_invalid_evidence() {
 assert_eq!(truth("false and probe(\"x\")"), Truth::False);
 assert_eq!(truth("true or probe(\"x\")"), Truth::True);
 assert_eq!(truth("true and probe(\"x\")"), Truth::Unknown);
 assert_eq!(truth("not probe(\"x\")"), Truth::Unknown);
 let p=compile_one("false and probe(\"x\")");
 assert_eq!(p.requests.len(),1);
 let obs=Observation { request_id:p.requests[0].id.clone(),plan_id:p.id.clone(),observed_at:9,valid_until:Some(10),outcome:Outcome::Known(Value::String("wrong".into())) };
 assert!(evaluate(&p,&[obs],&context()).is_err());
}
#[test] fn evidence_freshness_identity_dedup_and_replay() {
 let p=compile_one("probe(\"x\") and probe(name: \"x\")"); assert_eq!(p.requests.len(),1);
 let mut obs=Observation { request_id:p.requests[0].id.clone(),plan_id:p.id.clone(),observed_at:9,valid_until:Some(10),outcome:Outcome::Known(Value::Bool(true)) };
 assert_eq!(evaluate(&p,&[obs.clone()],&context()).unwrap().truth, Truth::True);
 obs.valid_until=Some(9); assert_eq!(evaluate(&p,&[obs.clone()],&context()).unwrap().truth,Truth::Unknown);
 obs.valid_until=None; obs.plan_id="other".into(); assert_eq!(evaluate(&p,&[obs.clone()],&context()).unwrap().truth,Truth::Unknown);
 let encoded=serde_json::to_vec(&p).unwrap(); let replay:Plan=serde_json::from_slice(&encoded).unwrap();
 assert_eq!(serde_json::to_vec(&evaluate(&p,&[],&context()).unwrap()).unwrap(),serde_json::to_vec(&evaluate(&replay,&[],&context()).unwrap()).unwrap());
 let mut tampered:serde_json::Value=serde_json::from_slice(&encoded).unwrap(); tampered["id"]="tampered".into(); assert!(serde_json::from_value::<Plan>(tampered).is_err());
}
#[test] fn recipes_are_typed_hygienic_and_bounded() {
 let mut cat=catalog(); cat.functions.insert("alias".into(), Function { parameters:vec![Parameter{name:"arg".into(),value_type:Type::String,default:None}],returns:Type::Bool,implementation:Implementation::Recipe("probe($arg)".into()) });
 let p=compile(&["alias(\"x\") and probe(\"x\")".into()],&cat,&bindings()).unwrap(); assert_eq!(p.requests.len(),1);
 cat.functions.get_mut("alias").unwrap().implementation=Implementation::Recipe("alias($arg)".into()); assert!(compile(&["alias(\"x\")".into()],&cat,&bindings()).is_err());
 cat.functions.get_mut("alias").unwrap().implementation=Implementation::Recipe("probe($ambient)".into()); assert!(compile(&["alias(\"x\")".into()],&cat,&bindings()).is_err());
 assert!(compile(&[],&catalog(),&bindings()).is_err());
 assert!(compile(&[format!("{}true{}", "(".repeat(10000), ")".repeat(10000))],&catalog(),&bindings()).is_err());
}
#[test] fn typed_collections_optional_and_acquisition_dependencies() {
 assert_eq!(truth("{answer: 42}.answer == [41,42][1]"),Truth::True);
 assert_eq!(truth("\"x\" in [\"x\",\"y\"]"),Truth::True);
 assert!(compile(&["probe(probe(\"x\"))".into()],&catalog(),&bindings()).is_err());
 let mut cat=catalog(); cat.functions.insert("present".into(),Function{parameters:vec![Parameter{name:"value".into(),value_type:Type::Optional(Box::new(Type::String)),default:None}],returns:Type::Bool,implementation:Implementation::Pure(Builtin::Present)});
 let mut b=bindings(); b.values.insert("none".into(),Value::Optional(None));
 let parsed=parse("present($none)").unwrap(); let checked=check(parsed,&cat,&BTreeMap::from([("none".into(),Type::Optional(Box::new(Type::String)))])).unwrap(); let p=plan(checked,&b).unwrap(); assert_eq!(evaluate(&p,&[],&context()).unwrap().truth,Truth::False);
}
