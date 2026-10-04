//! Adversary pass 2 for story:explanation, after the merge that bound evidence matches to a
//! subject (story:subject-bound-evidence-match).
//!
//! Each case drives from a stated rule: the module docs of `crate::explain` (`computed_from`
//! records "the authority decisions (`authority`, `{"capability", "decision"}` entries)"; a
//! claim's `because` lists each record a match it reaches reads, then each match that reads none,
//! with its subject, in kind and then subject order; a reason `{"claim": <id>}` anywhere continues
//! under `claims`), design § 37 ("which authority decisions applied"), CANON-DETERMINISM-001
//! (equivalent inputs give byte-identical output) and the generated evaluation documents page.

use std::collections::BTreeSet;

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, Decision, EvidenceRecord};
use serde_json::{Value, json};

fn compiled(source: &str) -> ir::Ir {
    ir::compile(&model::parse(source).expect("protocol parses")).expect("protocol compiles")
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

const AUTHORITY_PROTOCOL: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {a: {}}\nevidence_kinds: {k: {}}\n\
    actions:\n  act: {requires: [{capability: cap}]}\n";

const AUTHORITY_CASE: &str =
    "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n";

/// The evaluator reads `decision: !granted` and `decision: !denied` (YAML's tagged form of an
/// enum variant) as `granted` and `denied`: the action's status shows it. `computed_from` must
/// record the decision that applied (design § 37), and the same decision written either way is an
/// equivalent input (CANON-DETERMINISM-001). `explain::authority` re-reads the text into a
/// `String` and records `""` for both, so the record no longer says which decision applied, and
/// a grant and a denial are recorded alike.
#[test]
fn a_tagged_authority_decision_is_recorded_as_the_decision_that_applied() {
    let ir = compiled(AUTHORITY_PROTOCOL);
    for (plain, tagged, status) in [
        (
            "- {capability: cap, decision: granted}\n",
            "- capability: cap\n  decision: !granted\n",
            "admissible",
        ),
        (
            "- {capability: cap, decision: denied}\n",
            "- capability: cap\n  decision: !denied\n",
            "blocked",
        ),
    ] {
        let with = |text: &str| {
            decide(
                &ir,
                AUTHORITY_CASE,
                &[],
                Supplied {
                    authority: Some(text),
                    ..Supplied::default()
                },
            )
        };
        let (plain, tagged) = (with(plain), with(tagged));
        assert_eq!(
            tagged.actions.as_ref().expect("actions")["act"]["status"],
            json!(status),
            "the evaluator reads the tagged form as the decision it names"
        );
        assert_eq!(plain.actions, tagged.actions);
        assert_eq!(
            explanation(&tagged)["computed_from"]["authority"],
            explanation(&plain)["computed_from"]["authority"],
            "the authority decision that applied, `{status}`, is recorded as it applied"
        );
        assert_eq!(eval::render(&plain), eval::render(&tagged));
    }
}

/// The generated evaluation documents page lists every key of `canon-decision/1`. Every decision
/// now carries an `explanation`; the page's row still says it is not written.
#[test]
fn the_documents_page_does_not_call_the_explanation_unwritten() {
    let page = std::path::Path::new(&std::env::var_os("CARGO_MANIFEST_DIR").expect("set by cargo"))
        .join("../../website/docs/reference/documents.md");
    let text = std::fs::read_to_string(&page).expect("the documents page reads");
    let row = text
        .lines()
        .find(|line| line.starts_with("| `explanation` |"))
        .expect("the page has an `explanation` row");
    assert!(
        !row.contains("not written yet"),
        "{}: the `explanation` row says it is not written: {row}",
        page.display()
    );
}

const SUBJECTS: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {a: {}, b: {}}\nevidence_kinds: {k: {}, l: {}}\n\
    claims:\n\
    \x20\x20on_a: {true_when: {evidence: {kind: k, subject: a}}}\n\
    \x20\x20on_b: {true_when: {all: [{claim: on_a}, {evidence: {kind: k, subject: b}}]}}\n\
    \x20\x20mixed: {true_when: {any: [{evidence: {kind: l, subject: b}}, {evidence: {kind: k}}, \
    {evidence: {kind: l, subject: a}}, {evidence: {kind: k, subject: a}}, {evidence: {kind: l}}]}}\n\
    obligations:\n\
    \x20\x20o: {discharged_when: {not: {claim: on_a}}}\n\
    actions:\n\
    \x20\x20act: {precondition: {not: {evidence: {kind: k, subject: a}}}}\n\
    outcomes:\n\
    \x20\x20done: {requires: {all: [{evidence: {kind: k, subject: a}}, {evidence: {kind: k, subject: b}}, {evidence: {kind: k}}]}}\n\
    \x20\x20none_mixed: {requires: {not: {claim: mixed}}}\n";

const SUBJECTS_CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {b: {revision: r2}, a: {revision: r2}}\n";

fn about(id: &str, kind: &str, subject: &str, revision: &str) -> EvidenceRecord {
    eval::read_evidence(&format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\nsubject: {subject}\n\
         subject_revision: {revision}\n"
    ))
    .expect("a canon-evidence/1 record")
}

/// `ea` (k about a) applies, `eb` (k about b) and `ec` (k about a) are bound to a superseded
/// revision. `on_a` reads only records about `a`, `on_b` is built on it and reads `b` as well,
/// and `mixed` reaches `k` through a bound and an unbound match and `l` through two bound matches
/// and an unbound one, none of which reads a record. Every order of the evidence set gives the
/// same bytes; each record is listed once under a claim, excluded ones as the claim's own
/// `excluded_evidence` lists them; absent matches come in kind and then subject order, no subject
/// first; and an outcome's evidence reason is followed only by the records its match reads.
#[test]
fn subject_reach_is_traced_once_in_a_fixed_order_whatever_the_evidence_order() {
    let ir = compiled(SUBJECTS);
    let records = [
        about("ec", "k", "a", "r1"),
        about("ea", "k", "a", "r2"),
        about("eb", "k", "b", "r1"),
    ];
    let orders: [[usize; 3]; 6] = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    let mut reference: Option<String> = None;
    for order in orders {
        let evidence: Vec<EvidenceRecord> = order.iter().map(|i| records[*i].clone()).collect();
        let rendered = eval::render(&decide(&ir, SUBJECTS_CASE, &evidence, Supplied::default()));
        match &reference {
            None => reference = Some(rendered),
            Some(first) => assert_eq!(&rendered, first, "order {order:?}"),
        }
    }
    let decision = decide(&ir, SUBJECTS_CASE, &records, Supplied::default());
    let explained = explanation(&decision);
    let applied = |id: &str| json!({"evidence": id, "status": "applied"});
    let stale =
        |id: &str| json!({"evidence": id, "reason": "revision_mismatch", "status": "excluded"});
    assert_eq!(
        explained["claims"],
        json!({
            "mixed": {"because": [
                applied("ea"), stale("eb"), stale("ec"),
                {"kind": "l", "status": "absent"},
                {"kind": "l", "status": "absent", "subject": "a"},
                {"kind": "l", "status": "absent", "subject": "b"},
            ], "value": "true"},
            "on_a": {"because": [applied("ea"), stale("ec")], "value": "true"},
            "on_b": {"because": [
                {"claim": "on_a", "value": "true"}, applied("ea"), stale("eb"), stale("ec"),
            ], "value": "unknown"},
        })
    );
    assert_eq!(
        explained["outcomes"],
        json!({
            "done": {"because": [
                {"evidence": "k", "present": false, "subject": "b"}, stale("eb"),
            ], "status": "blocked"},
            "none_mixed": {"because": [{"claim": "mixed", "value": "true"}], "status": "blocked"},
        })
    );
    assert_eq!(
        explained["actions"],
        json!({"act": {"because": [
            {"evidence": "k", "present": true, "subject": "a"}, applied("ea"), stale("ec"),
        ], "status": "blocked"}})
    );
    assert_eq!(
        explained["obligations"],
        json!({"o": {"because": [{"claim": "on_a", "value": "true"}], "status": "open"}})
    );
    // Each claim's excluded entries are exactly the decision's own `excluded_evidence`.
    for (id, entry) in decision.claims.iter() {
        let Some(listed) = explained["claims"].get(id.as_str()) else {
            continue;
        };
        let excluded: Vec<(String, String)> = listed["because"]
            .as_array()
            .expect("a list")
            .iter()
            .filter(|item| item["status"] == "excluded")
            .map(|item| {
                (
                    item["evidence"].as_str().expect("an id").to_owned(),
                    item["reason"].as_str().expect("a reason").to_owned(),
                )
            })
            .collect();
        let decided: Vec<(String, String)> = entry
            .excluded_evidence
            .iter()
            .map(|exclusion| {
                (
                    exclusion.evidence.as_str().to_owned(),
                    exclusion.reason.as_str().to_owned(),
                )
            })
            .collect();
        assert_eq!(excluded, decided, "claim {}", id.as_str());
    }
}

/// Every `{"claim": <id>}` reason anywhere in the explanation — under an outcome, an action, an
/// obligation or another claim, `true` claims included — continues under `claims`, with the
/// value the decision gives it.
#[test]
fn every_claim_a_reason_names_is_listed_with_its_decided_value() {
    let ir = compiled(SUBJECTS);
    for evidence in [
        vec![],
        vec![about("ea", "k", "a", "r2")],
        vec![about("ea", "k", "a", "r2"), about("eb", "k", "b", "r2")],
        vec![about("ec", "k", "a", "r1"), about("el", "l", "b", "r2")],
    ] {
        let decision = decide(&ir, SUBJECTS_CASE, &evidence, Supplied::default());
        let explained = explanation(&decision);
        let mut named = BTreeSet::new();
        collect_claims(explained, &mut named);
        for (claim, value) in named {
            let entry = &explained["claims"][&claim];
            assert!(
                entry.is_object(),
                "`{claim}` is named and not listed: {explained}"
            );
            assert_eq!(entry["value"], json!(value), "`{claim}`");
            let decided = decision
                .claims
                .iter()
                .find(|(id, _)| id.as_str() == claim)
                .expect("declared")
                .1
                .value
                .to_string();
            assert_eq!(value, decided, "`{claim}`");
        }
    }
}

/// A `true` claim that only a claim under `claims` tests — no outcome, action or obligation
/// names it — is still listed, so the chain from `top` reaches `e1`. Every other case seeds such a
/// claim through a section reason as well, so a `claims` that stops following claim references
/// (mutant M1: drop `pending.extend(declared.true_when.claim_references())`) passes all of them.
#[test]
fn a_true_claim_only_another_claim_tests_is_listed() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {}, m: {}}\n\
         claims:\n\
         \x20\x20base: {true_when: {evidence: {kind: k}}}\n\
         \x20\x20top: {true_when: {all: [{claim: base}, {evidence: {kind: m}}]}}\n",
    );
    let decision = decide(
        &ir,
        AUTHORITY_CASE,
        &[about("e1", "k", "a", "r1")],
        Supplied::default(),
    );
    assert_eq!(
        explanation(&decision)["claims"],
        json!({
            "base": {"because": [{"evidence": "e1", "status": "applied"}], "value": "true"},
            "top": {"because": [
                {"claim": "base", "value": "true"},
                {"evidence": "e1", "status": "applied"},
                {"kind": "m", "status": "absent"},
            ], "value": "unknown"},
        })
    );
}

fn collect_claims(value: &Value, named: &mut BTreeSet<(String, String)>) {
    match value {
        Value::Object(map) => {
            if let (Some(claim), Some(value)) = (
                map.get("claim").and_then(Value::as_str),
                map.get("value").and_then(Value::as_str),
            ) {
                named.insert((claim.to_owned(), value.to_owned()));
            }
            map.values()
                .for_each(|member| collect_claims(member, named));
        }
        Value::Array(items) => items.iter().for_each(|item| collect_claims(item, named)),
        _ => {}
    }
}
