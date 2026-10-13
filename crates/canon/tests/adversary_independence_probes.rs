//! Adversarial probes of CANON-INDEPENDENCE-001 (story:evidence-independence): a record that fails
//! a demand never decides a claim, action or outcome; the order of the case's map, its dimension
//! lists and the evidence set does not change one byte of the decision; and the decision and its
//! explanation list the same `not_independent` records.

use std::collections::BTreeSet;

use b10x_canon::eval;
use b10x_canon::ir::{self, Ir};
use b10x_canon::model::{self, EvidenceRecord};
use serde_json::Value;

/// Every evidence match demands independence; claims reach them through `not`, `any`, `all`,
/// claim tests (`is: false` too), an invalidation rule and a max age; actions and outcomes read
/// them directly and through claims.
const PROTOCOL: &str = r#"format: protocol/1
protocol: {id: p, revision: 1}
artifacts: {change: {}, base: {}}
independence_dimensions: {d: {}, e: {}, f: {}}
evidence_kinds:
  review: {}
  scan: {max_age: 1h}
claims:
  approved: {true_when: {evidence: {kind: review, result: approved, independent_on: [d]}}}
  strongly: {true_when: {evidence: {kind: review, result: approved, independent_on: [e, d]}}}
  not_rejected: {true_when: {not: {evidence: {kind: review, result: rejected, independent_on: [d]}}}}
  either: {true_when: {any: [{claim: approved, is: true}, {evidence: {kind: scan, result: pass, independent_on: [f]}}]}}
  both: {true_when: {all: [{claim: approved, is: false}, {not: {evidence: {kind: scan, independent_on: [f]}}}]}}
  bound: {true_when: {evidence: {kind: review, subject: change, result: approved, independent_on: [d]}}}
actions:
  merge:
    precondition:
      all:
        - {claim: not_rejected, is: true}
        - {evidence: {kind: review, result: approved, independent_on: [d]}}
  revert:
    precondition: {any: [{claim: approved, is: false}, {not: {evidence: {kind: scan, result: pass, independent_on: [f]}}}]}
outcomes:
  done: {requires: {claim: strongly, is: true}}
  dropped: {requires: {claim: approved, is: false}}
invalidation:
  base_moved: {upstream: base, invalidates: [approved]}
"#;

fn compiled() -> Ir {
    ir::compile(&model::parse(PROTOCOL).expect("parses")).expect("compiles")
}

fn case(independence: &str) -> model::Case {
    eval::read_case(&format!(
        "format: canon-case/1\nid: C-1\nprotocol: p\n\
         artifacts: {{change: {{revision: r1}}, base: {{revision: b1}}}}\n{independence}"
    ))
    .expect("a case")
}

fn record(id: &str, kind: &str, result: Option<&str>, producer: Option<&str>) -> EvidenceRecord {
    let result = result.map_or(String::new(), |r| format!("result: {r}\n"));
    let producer = producer.map_or(String::new(), |p| format!("producer: {p}\n"));
    eval::read_evidence(&format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\n{result}subject: change\n\
         subject_revision: r1\nupstream_revisions: {{base: b1}}\n{producer}"
    ))
    .expect("a record")
}

fn decide(ir: &Ir, case: &model::Case, evidence: &[EvidenceRecord]) -> Value {
    let decision = eval::evaluate(ir, case, evidence).expect("decides");
    serde_json::from_str(&eval::render(&decision)).expect("JSON")
}

/// Claim values and action and outcome statuses: what a record may never decide alone.
fn verdicts(decision: &Value) -> Vec<(String, String)> {
    let mut found = Vec::new();
    for (id, claim) in decision["claims"].as_object().expect("claims") {
        found.push((format!("claim {id}"), claim["value"].to_string()));
    }
    for section in ["actions", "outcomes"] {
        for (id, entry) in decision[section].as_object().into_iter().flatten() {
            found.push((format!("{section} {id}"), entry["status"].to_string()));
        }
    }
    found
}

/// The case's map: `none` lists no dimension, `sub` only `d` of `[d, e]`, `other` only `e` and
/// `f` (no `d`); `ghost` is not listed at all, and a record may name no producer.
const MAP: &str = "independence: {none: [], sub: [e], other: [e, f]}\n";

/// Any set of records each of which fails every demand that would read it decides exactly what
/// no evidence decides: every claim, action and outcome is as with an empty evidence set.
#[test]
fn records_that_fail_every_demand_decide_nothing() {
    let ir = compiled();
    // `sub` is on `e` only: it fails `[d]` and `[d, e]`. `other` is on `e` and `f`: it fails every
    // review demand but meets `scan`'s `[f]`, so it is used only for review records.
    let failing_producers = [
        None,
        Some("none"),
        Some("sub"),
        Some("ghost"),
        Some("other"),
    ];
    let empty = verdicts(&decide(&ir, &case(MAP), &[]));
    for producer in failing_producers {
        for results in [
            vec![Some("approved")],
            vec![Some("rejected")],
            vec![None],
            vec![Some("approved"), Some("rejected")],
        ] {
            let mut evidence: Vec<EvidenceRecord> = results
                .iter()
                .enumerate()
                .map(|(at, result)| record(&format!("r{at}"), "review", *result, producer))
                .collect();
            if producer != Some("other") {
                evidence.push(record("s0", "scan", Some("fail"), producer));
                evidence.push(record("s1", "scan", Some("pass"), producer));
            }
            let found = verdicts(&decide(&ir, &case(MAP), &evidence));
            assert_eq!(found, empty, "producer {producer:?}, results {results:?}");
        }
    }
}

/// A producer listed on a superset of the demand meets it; one listed on a subset does not.
#[test]
fn a_superset_meets_a_demand_and_a_subset_does_not() {
    let ir = compiled();
    let map = "independence: {wide: [f, e, d], narrow: [d]}\n";
    let wide = decide(
        &ir,
        &case(map),
        &[record("r0", "review", Some("approved"), Some("wide"))],
    );
    assert_eq!(wide["claims"]["strongly"]["value"], "true");
    assert_eq!(wide["claims"]["approved"]["value"], "true");
    let narrow = decide(
        &ir,
        &case(map),
        &[record("r0", "review", Some("approved"), Some("narrow"))],
    );
    assert_eq!(narrow["claims"]["strongly"]["value"], "unknown");
    assert_eq!(narrow["claims"]["approved"]["value"], "true");
}

/// The decision, explanation included, is byte-identical whatever the order of the case's map,
/// of each producer's dimensions and of the evidence set.
#[test]
fn order_of_the_map_the_dimensions_and_the_evidence_does_not_matter() {
    let ir = compiled();
    let maps = [
        "independence: {a: [d, e], b: [], c: [f], n: [e]}\n",
        "independence: {n: [e], c: [f], b: [], a: [e, d]}\n",
        "independence: {b: [], a: [e, d], n: [e], c: [f]}\n",
    ];
    let evidence = vec![
        record("r0", "review", Some("approved"), Some("a")),
        record("r1", "review", Some("rejected"), Some("b")),
        record("r2", "review", Some("rejected"), None),
        record("r3", "review", Some("approved"), Some("n")),
        record("s0", "scan", Some("pass"), Some("c")),
        record("s1", "scan", Some("fail"), Some("b")),
    ];
    let mut orders = vec![evidence.clone()];
    let mut reversed = evidence.clone();
    reversed.reverse();
    orders.push(reversed);
    for shift in 1..evidence.len() {
        let mut rotated = evidence.clone();
        rotated.rotate_left(shift);
        orders.push(rotated);
    }
    let first = eval::render(&eval::evaluate(&ir, &case(maps[0]), &evidence).expect("decides"));
    for map in maps {
        for order in &orders {
            let found = eval::render(&eval::evaluate(&ir, &case(map), order).expect("decides"));
            assert_eq!(found, first, "{map}");
        }
    }
}

/// Under each claim, the records the decision excludes as `not_independent` are those the
/// explanation lists as excluded with that reason; and each blocked action's or outcome's evidence
/// reason that names `independent_on` is followed by a `not_independent` entry exactly for the
/// records of its kind that fail that demand.
#[test]
fn decision_and_explanation_agree_on_not_independent() {
    let ir = compiled();
    let map = "independence: {a: [d, e], b: [], c: [f], n: [e]}\n";
    let evidence = vec![
        record("r0", "review", Some("approved"), Some("n")),
        record("r1", "review", Some("rejected"), Some("b")),
        record("r2", "review", Some("approved"), None),
        record("s0", "scan", Some("pass"), Some("b")),
        record("s1", "scan", Some("fail"), Some("c")),
    ];
    let decision = decide(&ir, &case(map), &evidence);
    let listed = |entries: &Value| -> BTreeSet<String> {
        entries
            .as_array()
            .into_iter()
            .flatten()
            .filter(|entry| entry["reason"] == "not_independent")
            .map(|entry| entry["evidence"].as_str().expect("an id").to_owned())
            .collect()
    };
    for (id, claim) in decision["claims"].as_object().expect("claims") {
        assert_eq!(
            listed(&claim["excluded_evidence"]),
            listed(&decision["explanation"]["claims"][id]["because"]),
            "claim {id}: {decision:#}"
        );
    }
    let independence = [
        ("a", vec!["d", "e"]),
        ("b", vec![]),
        ("c", vec!["f"]),
        ("n", vec!["e"]),
    ];
    let mut demands_seen = 0;
    for section in ["actions", "outcomes"] {
        for (id, entry) in decision["explanation"][section]
            .as_object()
            .into_iter()
            .flatten()
        {
            let because = entry["because"].as_array().expect("because");
            for (at, reason) in because.iter().enumerate() {
                let Some(demand) = reason["independent_on"].as_array() else {
                    continue;
                };
                let kind = reason["evidence"].as_str().expect("a kind");
                demands_seen += 1;
                let expected: BTreeSet<String> = evidence
                    .iter()
                    .filter(|record| record.kind.as_str() == kind)
                    .filter(|record| {
                        let listed = record.producer.as_ref().and_then(|p| {
                            independence.iter().find(|(name, _)| *name == p.as_str())
                        });
                        !demand.iter().all(|d| {
                            listed.is_some_and(|(_, dims)| dims.contains(&d.as_str().unwrap()))
                        })
                    })
                    .map(|record| record.id.as_str().to_owned())
                    .collect();
                let following: Vec<Value> = because[at + 1..]
                    .iter()
                    .take_while(|next| next.get("status").is_some())
                    .cloned()
                    .collect();
                assert_eq!(
                    listed(&Value::Array(following)),
                    expected,
                    "{section} {id}: {entry:#}"
                );
            }
        }
    }
    assert!(
        demands_seen >= 1,
        "no demanding reason was checked: {decision:#}"
    );
}
