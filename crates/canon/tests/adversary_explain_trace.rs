//! Adversary cases for story:explanation against `eval::evaluate_with` and its `explanation`
//! section.
//!
//! Each case drives from a stated rule: the story's outcome ("every non-TRUE claim, open
//! obligation, non-admissible action and blocked outcome ... carries a structured explanation: a
//! chain of reasons down to the evidence records that applied or were excluded, each exclusion
//! with the reason"), the website status row for the structured explanation (the same words), the
//! comment of `conformance/scenarios/explanation.yaml` ("each blocked outcome, non-admissible
//! action and open obligation with its status and `because`, the reasons that decide it") and the
//! module docs of `crate::explain` (a member with nothing to explain is not written; the case
//! snapshot as given; every list in a fixed order, so input order does not change the output).

use std::collections::BTreeSet;

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, Decision, EvidenceRecord};
use serde_json::{Value, json};

fn compiled(source: &str) -> ir::Ir {
    ir::compile(&model::parse(source).expect("protocol parses")).expect("protocol compiles")
}

fn record(id: &str, kind: &str, revision: &str, extra: &str) -> EvidenceRecord {
    eval::read_evidence(&format!(
        "format: canon-evidence/1\nid: '{id}'\nkind: {kind}\nsubject: a\n\
         subject_revision: {revision}\n{extra}"
    ))
    .expect("a canon-evidence/1 record")
}

fn decide(
    ir: &ir::Ir,
    case: &str,
    evidence: &[EvidenceRecord],
    supplied: Supplied<'_>,
) -> Decision {
    let case = eval::read_case(case).expect("a canon-case/1 snapshot");
    eval::evaluate_with(ir, &case, evidence, supplied).expect("decides")
}

fn explanation(decision: &Decision) -> &Value {
    decision
        .explanation
        .as_ref()
        .expect("every decision carries an explanation")
}

/// Every evidence record the chain from `because` reaches, following each claim reason into
/// `claims`: `(id, status, reason)`. A reason `{"evidence": <kind>, "present": ...}` names a
/// kind, not a record, and is not collected.
fn traced(explanation: &Value, because: &Value) -> BTreeSet<(String, String, String)> {
    let mut found = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut pending: Vec<Value> = because.as_array().cloned().unwrap_or_default();
    while let Some(reason) = pending.pop() {
        if let Some(claim) = reason["claim"].as_str() {
            if visited.insert(claim.to_owned()) {
                pending.extend(
                    explanation["claims"][claim]["because"]
                        .as_array()
                        .cloned()
                        .unwrap_or_default(),
                );
            }
        } else if let (Some(id), Some(status)) =
            (reason["evidence"].as_str(), reason["status"].as_str())
        {
            found.insert((
                id.to_owned(),
                status.to_owned(),
                reason["reason"].as_str().unwrap_or("").to_owned(),
            ));
        }
    }
    found
}

fn excluded(id: &str, reason: &str) -> (String, String, String) {
    (id.to_owned(), "excluded".to_owned(), reason.to_owned())
}

fn applied(id: &str) -> (String, String, String) {
    (id.to_owned(), "applied".to_owned(), String::new())
}

/// An outcome whose requirement is an evidence match, blocked because its only record is bound
/// to a superseded revision. The story's outcome and the website status row promise the blocked
/// outcome's explanation reaches "each evidence record that applied or was excluded and why"; the
/// explanation copies the outcome's reason `{"evidence": "k", "present": false}` and names `e1`
/// only in `computed_from`, so the excluded record and its reason are nowhere in the chain.
#[test]
fn a_blocked_outcome_on_an_excluded_record_traces_to_that_record() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {}}\n\
         outcomes:\n  done: {requires: {evidence: {kind: k}}}\n",
    );
    let case = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r2}}\n";
    let decision = decide(
        &ir,
        case,
        &[record("e1", "k", "r1", "")],
        Supplied::default(),
    );
    let explained = explanation(&decision);
    let outcome = &explained["outcomes"]["done"];
    assert_eq!(outcome["status"], "blocked", "{explained:#}");
    assert!(
        traced(explained, &outcome["because"]).contains(&excluded("e1", "revision_mismatch")),
        "the blocked outcome's chain names e1 excluded as revision_mismatch:\n{explained:#}"
    );
}

/// The same for an action whose precondition is an evidence match and whose only record has
/// expired: the non-admissible action's explanation must reach `e1` excluded as `expired`.
#[test]
fn a_blocked_action_on_an_expired_record_traces_to_that_record() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {max_age: 1d}}\n\
         actions:\n  act: {precondition: {evidence: {kind: k}}}\n",
    );
    let case = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n";
    let decision = decide(
        &ir,
        case,
        &[record(
            "e1",
            "k",
            "r1",
            "observed_at: 2000-01-01T00:00:00Z\n",
        )],
        Supplied {
            at: Some("2026-10-04T00:00:00Z"),
            ..Supplied::default()
        },
    );
    let explained = explanation(&decision);
    let action = &explained["actions"]["act"];
    assert_eq!(action["status"], "blocked", "{explained:#}");
    assert!(
        traced(explained, &action["because"]).contains(&excluded("e1", "expired")),
        "the blocked action's chain names e1 excluded as expired:\n{explained:#}"
    );
}

/// An obligation to refute a claim (`discharged_when: {claim: c, is: false}`, the shape
/// `eval/obligations.rs`'s own tests declare as `m.refuted`) is open while `c` is `true`. Its
/// explanation's only reason is `{"claim": "c", "value": "true"}`, and `claims` lists no `c`
/// because `c` is `true`, so the open obligation's chain stops before `e1`, the record that made
/// `c` true.
#[test]
fn an_open_obligation_on_a_true_claim_traces_to_its_evidence() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {}}\n\
         claims:\n  c: {true_when: {evidence: {kind: k}}}\n\
         obligations:\n  refuted: {discharged_when: {claim: c, is: false}}\n",
    );
    let case = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n";
    let decision = decide(
        &ir,
        case,
        &[record("e1", "k", "r1", "")],
        Supplied::default(),
    );
    let explained = explanation(&decision);
    let obligation = &explained["obligations"]["refuted"];
    assert_eq!(obligation["status"], "open", "{explained:#}");
    assert!(
        traced(explained, &obligation["because"]).contains(&applied("e1")),
        "the open obligation's chain names e1, which applied:\n{explained:#}"
    );
}

/// The scenario's own comment gives `outcomes` and `obligations` the same `because`: "the reasons
/// that decide it". For one predicate, `all: [a, b]` with `a` true and `b` unknown, the outcome's
/// reasons name only `b`; the obligation's name `a` as well, which does not decide it.
#[test]
fn an_obligation_and_an_outcome_on_one_predicate_give_the_same_reasons() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {}, l: {}}\n\
         claims:\n  a: {true_when: {evidence: {kind: k}}}\n  b: {true_when: {evidence: {kind: l}}}\n\
         obligations:\n  both: {discharged_when: {all: [{claim: a}, {claim: b}]}}\n\
         outcomes:\n  both: {requires: {all: [{claim: a}, {claim: b}]}}\n",
    );
    let case = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n";
    let decision = decide(
        &ir,
        case,
        &[record("e1", "k", "r1", "")],
        Supplied::default(),
    );
    let explained = explanation(&decision);
    assert_eq!(
        explained["obligations"]["both"]["because"], explained["outcomes"]["both"]["because"],
        "{explained:#}"
    );
}

/// Every claim reason anywhere in the explanation carries the value the decision gives that
/// claim. With `c` false, a claim built on it and an obligation on it both say `false`; a mutant
/// that wrote every claim reason as `unknown` passes every other case in the suite, whose claims
/// are all `unknown`.
#[test]
fn every_claim_reason_carries_the_decisions_own_value() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {}}\n\
         claims:\n  c: {true_when: {evidence: {kind: k, result: pass}}}\n  d: {true_when: {claim: c}}\n\
         obligations:\n  o: {discharged_when: {claim: d}}\n",
    );
    let case = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n";
    let decision = decide(
        &ir,
        case,
        &[record("e1", "k", "r1", "result: fail\n")],
        Supplied::default(),
    );
    let explained = explanation(&decision);
    assert_eq!(
        explained["claims"]["d"]["because"][0],
        json!({"claim": "c", "value": "false"}),
        "{explained:#}"
    );
    assert_eq!(
        explained["obligations"]["o"]["because"],
        json!([{"claim": "d", "value": "false"}]),
        "{explained:#}"
    );
    let mut pending = vec![explained.clone()];
    let mut checked = 0;
    while let Some(node) = pending.pop() {
        match node {
            Value::Object(map) => {
                if let (Some(Value::String(claim)), Some(Value::String(value))) =
                    (map.get("claim"), map.get("value"))
                {
                    let (_, entry) = decision
                        .claims
                        .iter()
                        .find(|(id, _)| id.as_str() == claim)
                        .expect("a claim the decision gives");
                    assert_eq!(*value, entry.value.to_string(), "claim `{claim}`");
                    checked += 1;
                }
                pending.extend(map.into_iter().map(|(_, value)| value));
            }
            Value::Array(items) => pending.extend(items),
            _ => {}
        }
    }
    assert_eq!(checked, 2, "two claim reasons: d on c, o on d");
}

/// Only open obligations and blocked outcomes are explained: a discharged obligation and a
/// legitimate outcome are not written. A mutant that dropped the `open` filter passes every other
/// case in the suite, none of which explains a decision with a discharged obligation.
#[test]
fn a_discharged_obligation_and_a_legitimate_outcome_are_not_explained() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {}, l: {}}\n\
         claims:\n  c: {true_when: {evidence: {kind: k}}}\n  u: {true_when: {evidence: {kind: l}}}\n\
         obligations:\n  done: {discharged_when: {claim: c}}\n  open: {discharged_when: {claim: u}}\n\
         outcomes:\n  ok: {requires: {claim: c}}\n  no: {requires: {claim: u}}\n",
    );
    let case = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n";
    let decision = decide(
        &ir,
        case,
        &[record("e1", "k", "r1", "")],
        Supplied::default(),
    );
    let explained = explanation(&decision);
    let keys = |section: &str| -> Vec<String> {
        explained[section]
            .as_object()
            .map(|map| map.keys().cloned().collect())
            .unwrap_or_default()
    };
    assert_eq!(keys("obligations"), ["open"], "{explained:#}");
    assert_eq!(keys("outcomes"), ["no"], "{explained:#}");
    assert_eq!(keys("claims"), ["u"], "{explained:#}");
}

/// The case snapshot is recorded as given, `termination` included.
#[test]
fn the_case_snapshot_records_its_termination() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {}}\n\
         claims:\n  c: {true_when: {evidence: {kind: k}}}\n\
         outcomes:\n  ok: {requires: {claim: c}}\n",
    );
    let case = "format: canon-case/1\nid: C-1\nprotocol: p\nrevision: c7\ntermination: ok\n\
                artifacts: {a: {revision: r1}}\n";
    let decision = decide(
        &ir,
        case,
        &[record("e1", "k", "r1", "")],
        Supplied::default(),
    );
    assert_eq!(
        explanation(&decision)["computed_from"]["case"],
        json!({
            "artifacts": {"a": {"revision": "r1"}},
            "format": "canon-case/1",
            "id": "C-1",
            "protocol": "p",
            "revision": "c7",
            "termination": "ok",
        })
    );
}

/// Every permutation of a four-record evidence set (one applied, one expired, one bound to a
/// superseded revision, one of another kind) with identifiers outside ASCII, and the authority and
/// explicit decisions in every order given, renders the same bytes.
#[test]
fn every_input_order_renders_the_same_bytes() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {max_age: 1d}, l: {}}\n\
         claims:\n  'ζ': {true_when: {all: [{evidence: {kind: k}}, {evidence: {kind: l, result: pass}}]}}\n\
         \x20\x20'Z': {true_when: {claim: 'ζ'}}\n\
         actions:\n  act: {precondition: {claim: 'Z'}, requires: [{capability: 'ｃap'}, {capability: cap}]}\n\
         outcomes:\n  closed: {requires: {decision: x}}\n  done: {requires: {claim: 'Z'}}\n",
    );
    let case = "format: canon-case/1\nid: C-1\nprotocol: p\nrevision: c1\n\
                artifacts: {a: {revision: r2}}\n";
    let records = [
        record("é", "k", "r2", "observed_at: 2026-10-03T12:00:00Z\n"),
        record("𝔸", "k", "r2", "observed_at: 2000-01-01T00:00:00Z\n"),
        record("Ω", "k", "r1", ""),
        record("z", "l", "r2", "result: fail\n"),
    ];
    let authority = [
        "- {capability: 'ｃap', decision: granted}\n",
        "- {capability: cap, decision: denied}\n",
        "- {capability: '𝔠', decision: granted}\n",
    ];
    let decisions = [
        "- {decision: x, outcome: closed, principal: 'ü', case_revision: c1}\n",
        "- {decision: x, outcome: closed, principal: '𝔭', case_revision: c0}\n",
        "- {decision: x, outcome: closed, principal: u, case_revision: c1}\n",
    ];
    let orders = permutations();
    assert_eq!(orders.len(), 24);
    let mut reference: Option<String> = None;
    for (n, order) in orders.iter().enumerate() {
        let evidence: Vec<EvidenceRecord> = order.iter().map(|i| records[*i].clone()).collect();
        let pick = |texts: &[&str; 3]| -> String {
            let rotate = n % 3;
            let mut picked: Vec<&str> = texts.to_vec();
            picked.rotate_left(rotate);
            if n % 2 == 1 {
                picked.reverse();
            }
            picked.concat()
        };
        let authority = pick(&authority);
        let decisions = pick(&decisions);
        let rendered = eval::render(&decide(
            &ir,
            case,
            &evidence,
            Supplied {
                authority: Some(&authority),
                at: Some("2026-10-04T00:00:00Z"),
                decisions: Some(&decisions),
            },
        ));
        match &reference {
            None => reference = Some(rendered),
            Some(first) => assert_eq!(&rendered, first, "order {order:?}"),
        }
    }
    let reference = reference.expect("rendered");
    let explained: Value = serde_json::from_str(&reference).expect("canonical JSON");
    let explained = &explained["explanation"];
    assert_eq!(
        explained["computed_from"]["evidence"],
        json!(["z", "é", "Ω", "𝔸"])
    );
    assert_eq!(
        explained["claims"]["ζ"]["because"],
        json!([
            {"evidence": "z", "status": "applied"},
            {"evidence": "é", "status": "applied"},
            {"evidence": "Ω", "reason": "revision_mismatch", "status": "excluded"},
            {"evidence": "𝔸", "reason": "expired", "status": "excluded"},
        ])
    );
}

/// Every ordering of `0..4`.
fn permutations() -> Vec<Vec<usize>> {
    (0..256usize)
        .map(|n| {
            (0..4)
                .map(|digit| (n >> (2 * digit)) & 3)
                .collect::<Vec<_>>()
        })
        .filter(|order| order.iter().collect::<BTreeSet<_>>().len() == 4)
        .collect()
}
