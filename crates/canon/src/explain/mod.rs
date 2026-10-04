//! Structured explanation of an evaluation (design § 13, § 37, § 41 item 13): why each claim that
//! is not `true`, each open obligation, each action that is not admissible and each blocked
//! outcome has the status it has, down to the evidence records that applied or were excluded, and
//! what the decision was computed from, so it can be reconstructed later (CANON-EXPLAIN-001).
//!
//! The explanation is the decision's `explanation` section, an object with these members:
//!
//! - `computed_from`: the protocol id and revision (`protocol`, `protocol_revision`), the Canon
//!   semantics that applied (`semantics`: `{"canon": <the Canon crate version>, "format":
//!   "canon-decision/1"}`), the case snapshot as given (`case`: its `format`, `id`, `protocol`,
//!   `artifacts` with each artifact's `revision`, and `revision` and `termination` when given),
//!   the ids of the evidence set (`evidence`), and, each only when supplied, the authority
//!   decisions (`authority`, `{"capability", "decision"}` entries), the explicit decisions
//!   (`decisions`, `{"case_revision", "decision", "outcome", "principal"}` entries) and the
//!   evaluation instant (`at`).
//! - `outcomes`: each blocked outcome, `{"because": [...], "status": "blocked"}`.
//! - `actions`: each action that is not admissible, `{"because": [...], "status": <status>}`.
//! - `obligations`: each open obligation, `{"because": [...], "status": "open"}`. Its reasons are
//!   those its `discharged_when` predicate is not `true` for, walked exactly as a blocked outcome's
//!   requirement is (`outcomes.rs`), so one predicate gives an obligation and an outcome the same
//!   reasons.
//! - `claims`: every claim whose value is not `true`, and every claim a reason above names,
//!   together with every claim one of them tests, through any number of claim references, `true`
//!   ones included, so every chain of reasons ends at evidence records. Each is `{"because": [...],
//!   "value": <value>}`: each claim its `true_when` predicate tests, `{"claim": <id>, "value":
//!   <value>}`, then each record an evidence match the claim reaches reads (as its
//!   `excluded_evidence` list counts them), written as below, then each evidence match it reaches
//!   that reads no record of the evidence set, `{"kind": <kind>, "status": "absent"}`, with
//!   `"subject": <artifact>` added when the match names one, in kind and then subject order (no
//!   subject first).
//!
//! A match reads a record of its kind and, when it names a subject, only one about that artifact:
//! the rule claim values are decided by (`crate::eval::reads`). A record about another artifact is
//! not listed under a match bound to a subject, neither as applied nor as excluded.
//!
//! The `because` of an outcome, action or obligation is the reasons the decision states for it,
//! in their order, each evidence-match reason `{"evidence": <kind>, "present": ...}` (with its
//! `"subject"` when the match names one) followed by every record of the evidence set that match
//! reads, in evidence-id order: `{"evidence": <id>,
//! "status": "applied"}`, or `{"evidence": <id>, "reason": <reason>, "status": "excluded"}` with
//! the reason its exclusion stage gave (`revision_mismatch`, `expired`). A reason `{"claim": <id>,
//! "value": <value>}` anywhere continues under `claims`. A member with nothing to explain is not
//! written.
//!
//! Every list is in a fixed order — claim reasons by claim id, evidence records and evidence ids by
//! evidence id, absent kinds by kind id, authority decisions by capability, explicit decisions by
//! decision, outcome, principal and case revision, each by Unicode code point; a `because` copied
//! from the decision keeps the decision's order — and objects are written with their keys in
//! code-point order, so the order the inputs are given in does not change the output
//! (CANON-DETERMINISM-001).

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

use crate::eval::{Authority, Decisions, reads, unmet_reasons};
use crate::ir::Ir;
use crate::model::{
    ArtifactId, Case, ClaimId, DECISION_FORMAT, Decision, EvidenceExclusion, EvidenceKindId,
    EvidenceRecord, Json, Predicate, Truth,
};

/// An explanation, the payload of the decision's `explanation` slot, as the module docs give it.
pub type Explanation = Json;

/// The version of the Canon crate whose semantics applied.
const CANON_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The supplied inputs the decision was evaluated from, as the evaluator read them: the
/// explanation records the values that applied and reads no input text a second time, so it
/// cannot record a decision other than the one the evaluator applied.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Inputs<'a> {
    /// The authority decisions `eval::authority` read from `--authority`.
    pub(crate) authority: Option<&'a Authority>,
    /// The explicit decisions `eval::decisions` read from `--decisions`.
    pub(crate) decisions: Option<&'a Decisions>,
    /// The evaluation instant, as given: `--at` has exactly one written form per instant, which
    /// `eval::freshness` has checked.
    pub(crate) at: Option<&'a str>,
}

/// The explanation of `decision`, computed from `ir`, `case`, `evidence`, the records the
/// exclusion stages `excluded` (each with its reason) and the supplied `inputs` it was evaluated
/// from. Always written: it records at least what the decision was computed from.
pub(crate) fn explain(
    ir: &Ir,
    case: &Case,
    evidence: &[EvidenceRecord],
    excluded: &[EvidenceExclusion],
    inputs: Inputs<'_>,
    decision: &Decision,
) -> Option<Explanation> {
    let records = Records::new(evidence, excluded);
    let values: BTreeMap<ClaimId, Truth> = decision
        .claims
        .iter()
        .map(|(id, entry)| (id.clone(), entry.value))
        .collect();
    let obligations = obligations(ir, decision, &values, &records);
    let actions = not_with_status(decision.actions.as_ref(), "admissible", &records);
    let outcomes = not_with_status(decision.outcomes.as_ref(), "legitimate", &records);
    // Every claim that is not `true`, and every claim a reason above names.
    let named = [&obligations, &actions, &outcomes]
        .into_iter()
        .flat_map(|section| section.values())
        .flat_map(|entry| entry["because"].as_array().into_iter().flatten())
        .filter_map(|reason| reason["claim"].as_str());
    let seeds: Vec<&ClaimId> = values
        .iter()
        .filter(|(_, value)| **value != Truth::True)
        .map(|(id, _)| id)
        .chain(named.filter_map(|name| values.keys().find(|id| id.as_str() == name)))
        .collect();
    let claims = claims(ir, seeds, &values, &records);

    let mut sections = Map::new();
    sections.insert(
        "computed_from".to_owned(),
        computed_from(ir, case, evidence, inputs),
    );
    for (name, entries) in [
        ("claims", claims),
        ("obligations", obligations),
        ("actions", actions),
        ("outcomes", outcomes),
    ] {
        if !entries.is_empty() {
            sections.insert(name.to_owned(), Value::Object(entries));
        }
    }
    Some(Value::Object(sections))
}

/// An evidence match as the explanation traces it: its kind, and the artifact it names, if any.
type Match<'a> = (&'a EvidenceKindId, Option<&'a ArtifactId>);

/// The evidence set as the explanation lists it: each record, and the reason it was excluded, if
/// it was.
struct Records<'a> {
    /// In evidence-id order.
    records: Vec<&'a EvidenceRecord>,
    excluded: BTreeMap<&'a str, &'a str>,
}

impl<'a> Records<'a> {
    fn new(evidence: &'a [EvidenceRecord], excluded: &'a [EvidenceExclusion]) -> Self {
        let mut records: Vec<&EvidenceRecord> = evidence.iter().collect();
        records.sort_by(|one, other| one.id.as_str().cmp(other.id.as_str()));
        Self {
            records,
            excluded: excluded
                .iter()
                .map(|exclusion| (exclusion.evidence.as_str(), exclusion.reason.as_str()))
                .collect(),
        }
    }

    /// Each record one of `matches` reads (`crate::eval::reads`), in evidence-id order:
    /// `{"evidence": <id>, "status": "applied"}` or `{"evidence": <id>, "reason": <reason>,
    /// "status": "excluded"}`.
    fn read_by<'m>(&self, matches: impl IntoIterator<Item = Match<'m>> + Clone) -> Vec<Value> {
        self.records
            .iter()
            .filter(|record| {
                matches
                    .clone()
                    .into_iter()
                    .any(|(kind, subject)| reads(kind, subject, record))
            })
            .map(|record| {
                let id = record.id.as_str();
                match self.excluded.get(id) {
                    Some(reason) => {
                        json!({"evidence": id, "reason": reason, "status": "excluded"})
                    }
                    None => json!({"evidence": id, "status": "applied"}),
                }
            })
            .collect()
    }

    /// Whether a record of the evidence set is one `matching` reads.
    fn any_read_by(&self, (kind, subject): Match<'_>) -> bool {
        self.records
            .iter()
            .any(|record| reads(kind, subject, record))
    }
}

/// `reasons` as `because`: each reason as given, and after an evidence match's reason
/// `{"evidence": <kind>, "present": ...}`, with its `"subject"` when it names one, each record that
/// match reads.
fn because(reasons: &[Value], records: &Records<'_>) -> Vec<Value> {
    let mut because = Vec::new();
    for reason in reasons {
        because.push(reason.clone());
        if let (Some(kind), Some(_)) = (reason["evidence"].as_str(), reason.get("present")) {
            let kind = EvidenceKindId::new(kind);
            let subject = reason["subject"].as_str().map(ArtifactId::new);
            because.extend(records.read_by([(&kind, subject.as_ref())]));
        }
    }
    because
}

/// Sorts by Unicode code point, which is byte order for UTF-8.
fn sorted<T: Ord>(mut items: Vec<T>) -> Vec<T> {
    items.sort();
    items
}

fn computed_from(ir: &Ir, case: &Case, evidence: &[EvidenceRecord], inputs: Inputs<'_>) -> Value {
    let mut from = Map::new();
    from.insert("protocol".to_owned(), json!(ir.protocol.id.as_str()));
    from.insert("protocol_revision".to_owned(), json!(ir.protocol.revision));
    from.insert(
        "semantics".to_owned(),
        json!({"canon": CANON_VERSION, "format": DECISION_FORMAT}),
    );
    from.insert("case".to_owned(), case_snapshot(case));
    let ids = sorted(evidence.iter().map(|record| record.id.as_str()).collect());
    from.insert("evidence".to_owned(), json!(ids));
    if let Some(read) = inputs.authority {
        from.insert("authority".to_owned(), authority(read));
    }
    if let Some(read) = inputs.decisions {
        from.insert("decisions".to_owned(), decisions(read));
    }
    if let Some(at) = inputs.at {
        from.insert("at".to_owned(), json!(at));
    }
    Value::Object(from)
}

fn case_snapshot(case: &Case) -> Value {
    let artifacts: Map<String, Value> = case
        .artifacts
        .iter()
        .map(|(id, artifact)| {
            (
                id.as_str().to_owned(),
                json!({"revision": artifact.revision.as_str()}),
            )
        })
        .collect();
    let mut snapshot = Map::new();
    snapshot.insert("format".to_owned(), json!(case.format));
    snapshot.insert("id".to_owned(), json!(case.id.as_str()));
    snapshot.insert("protocol".to_owned(), json!(case.protocol.as_str()));
    snapshot.insert("artifacts".to_owned(), Value::Object(artifacts));
    if let Some(revision) = &case.revision {
        snapshot.insert("revision".to_owned(), json!(revision.as_str()));
    }
    if let Some(termination) = &case.termination {
        snapshot.insert("termination".to_owned(), json!(termination.as_str()));
    }
    Value::Object(snapshot)
}

/// The authority decisions as the evaluator read and applied them, by capability.
fn authority(authority: &Authority) -> Value {
    sorted(
        authority
            .decisions()
            .map(|(capability, decision)| (capability.as_str(), decision))
            .collect(),
    )
    .into_iter()
    .map(|(capability, decision)| json!({"capability": capability, "decision": decision}))
    .collect()
}

/// The explicit decisions as the evaluator read them, by decision, outcome, principal and case
/// revision.
fn decisions(decisions: &Decisions) -> Value {
    sorted(
        decisions
            .entries()
            .iter()
            .map(|entry| {
                (
                    entry.decision.as_str(),
                    entry.outcome.as_str(),
                    entry.principal.as_str(),
                    entry.case_revision.as_str(),
                )
            })
            .collect(),
    )
    .into_iter()
    .map(|(decision, outcome, principal, case_revision)| {
        json!({
            "case_revision": case_revision,
            "decision": decision,
            "outcome": outcome,
            "principal": principal,
        })
    })
    .collect()
}

/// `{"claim": <id>, "value": <value>}` for each claim `predicate` tests, in claim-id order.
fn claim_reasons(predicate: &Predicate, values: &BTreeMap<ClaimId, Truth>) -> Vec<Value> {
    let tested: BTreeSet<&ClaimId> = predicate.claim_references().into_iter().collect();
    tested
        .into_iter()
        .map(|claim| {
            let value = values.get(claim).copied().unwrap_or(Truth::Unknown);
            json!({"claim": claim.as_str(), "value": value.to_string()})
        })
        .collect()
}

/// The evidence matches `claim` reaches, by kind and subject: those of its own predicate, and those
/// every claim it tests reaches, through any number of claim references. Each claim is visited
/// once.
fn reached_matches<'a>(ir: &'a Ir, claim: &'a ClaimId) -> BTreeSet<Match<'a>> {
    let mut matches = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut pending = vec![claim];
    while let Some(next) = pending.pop() {
        if !visited.insert(next) {
            continue;
        }
        let Some(declared) = ir.claims.get(next) else {
            continue;
        };
        declared.true_when.visit(&mut |node| match node {
            Predicate::Evidence(matching) => {
                matches.insert((&matching.kind, matching.subject.as_ref()));
            }
            Predicate::Claim(test) => pending.push(&test.claim),
            _ => {}
        });
    }
    matches
}

/// An entry for each of `seeds` and each claim one of them tests, through any number of claim
/// references, so every chain of claim reasons ends at evidence records.
fn claims(
    ir: &Ir,
    seeds: Vec<&ClaimId>,
    values: &BTreeMap<ClaimId, Truth>,
    records: &Records<'_>,
) -> Map<String, Value> {
    let mut entries = Map::new();
    let mut pending = seeds;
    while let Some(id) = pending.pop() {
        if entries.contains_key(id.as_str()) {
            continue;
        }
        let mut because = match ir.claims.get(id) {
            Some(declared) => {
                pending.extend(declared.true_when.claim_references());
                claim_reasons(&declared.true_when, values)
            }
            None => Vec::new(),
        };
        let matches = reached_matches(ir, id);
        because.extend(records.read_by(matches.iter().copied()));
        because.extend(
            matches
                .iter()
                .filter(|matching| !records.any_read_by(**matching))
                .map(|(kind, subject)| {
                    let mut absent = json!({"kind": kind.as_str(), "status": "absent"});
                    if let Some(subject) = subject {
                        absent["subject"] = json!(subject.as_str());
                    }
                    absent
                }),
        );
        let value = values.get(id).copied().unwrap_or(Truth::Unknown);
        entries.insert(
            id.as_str().to_owned(),
            json!({"because": because, "value": value.to_string()}),
        );
    }
    entries
}

/// Each open obligation, with the reasons its `discharged_when` predicate is not `true`, walked
/// as a blocked outcome's requirement is (`crate::eval::unmet_reasons`).
fn obligations(
    ir: &Ir,
    decision: &Decision,
    values: &BTreeMap<ClaimId, Truth>,
    records: &Records<'_>,
) -> Map<String, Value> {
    let Some(Value::Array(entries)) = &decision.obligations else {
        return Map::new();
    };
    entries
        .iter()
        .filter(|entry| entry["status"] == "open")
        .filter_map(|entry| {
            let id = entry["id"].as_str()?;
            let declared = ir.obligations.iter().find(|(key, _)| key.as_str() == id)?.1;
            // A discharge predicate tests claims only; the validator refuses evidence in one.
            let reasons = unmet_reasons(&declared.discharged_when, values, &[]);
            Some((
                id.to_owned(),
                json!({"because": because(&reasons, records), "status": "open"}),
            ))
        })
        .collect()
}

/// The entries of an `actions` or `outcomes` section whose status is not `settled`, each with its
/// status and its reasons as `because`.
fn not_with_status(
    section: Option<&Json>,
    settled: &str,
    records: &Records<'_>,
) -> Map<String, Value> {
    let Some(Value::Object(entries)) = section else {
        return Map::new();
    };
    entries
        .iter()
        .filter(|(_, entry)| entry["status"] != settled)
        .map(|(id, entry)| {
            let reasons = entry["reasons"].as_array().cloned().unwrap_or_default();
            (
                id.clone(),
                json!({"because": because(&reasons, records), "status": entry["status"].clone()}),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::{Supplied, evaluate_with, read_case, read_evidence};

    const PROTOCOL: &str = "format: protocol/1\n\
        protocol: {id: p, revision: 2}\n\
        artifacts: {a: {}}\n\
        evidence_kinds: {k: {}, m: {}}\n\
        claims:\n\
          \x20\x20c: {true_when: {all: [{evidence: {kind: k}}, {evidence: {kind: m}}]}}\n\
          \x20\x20d: {true_when: {claim: c}}\n\
        obligations:\n\
          \x20\x20o: {discharged_when: {claim: d}}\n\
        actions:\n\
          \x20\x20act: {precondition: {claim: c}, requires: [{capability: cap}]}\n\
          \x20\x20look: {}\n\
        outcomes:\n\
          \x20\x20done: {requires: {claim: d}}\n\
          \x20\x20closed: {requires: {decision: x}}\n";

    const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nrevision: c1\n\
                        artifacts: {a: {revision: r2}}\n";

    fn record(id: &str, kind: &str, revision: &str) -> EvidenceRecord {
        read_evidence(&format!(
            "format: canon-evidence/1\nid: {id}\nkind: {kind}\nsubject: a\n\
             subject_revision: {revision}\n"
        ))
        .expect("a canon-evidence/1 record")
    }

    fn explanation(evidence: &[EvidenceRecord], supplied: Supplied<'_>) -> Value {
        let ir =
            crate::ir::compile(&crate::model::parse(PROTOCOL).expect("parses")).expect("compiles");
        let case = read_case(CASE).expect("a canon-case/1 snapshot");
        evaluate_with(&ir, &case, evidence, supplied)
            .expect("decides")
            .explanation
            .expect("every decision carries an explanation")
    }

    /// An applied record, an excluded one and a kind no record has, each under the claim that
    /// reaches it; the claim built on it names it as a reason and lists the same records; the open
    /// obligation, the blocked action and the blocked outcome each give their reasons.
    #[test]
    fn each_status_is_traced_to_claims_and_records() {
        let explained = explanation(
            &[record("e2", "k", "r2"), record("e1", "k", "r1")],
            Supplied::default(),
        );
        let records = json!([
            {"evidence": "e1", "reason": "revision_mismatch", "status": "excluded"},
            {"evidence": "e2", "status": "applied"},
            {"kind": "m", "status": "absent"},
        ]);
        assert_eq!(
            explained["claims"]["c"],
            json!({"because": records, "value": "unknown"})
        );
        let mut built_on = vec![json!({"claim": "c", "value": "unknown"})];
        built_on.extend(records.as_array().expect("a list").iter().cloned());
        assert_eq!(
            explained["claims"]["d"],
            json!({"because": built_on, "value": "unknown"})
        );
        let on_d = json!([{"claim": "d", "value": "unknown"}]);
        assert_eq!(
            explained["obligations"],
            json!({"o": {"because": on_d, "status": "open"}})
        );
        assert_eq!(
            explained["outcomes"]["done"],
            json!({"because": on_d, "status": "blocked"})
        );
        assert_eq!(
            explained["actions"],
            json!({"act": {"because": [{"claim": "c", "value": "unknown"}], "status": "blocked"}}),
            "`look` is admissible and not explained"
        );
        assert_eq!(explained["computed_from"]["evidence"], json!(["e1", "e2"]));
    }

    /// Supplied inputs are recorded in a fixed order, whatever order they were given in, and an
    /// input not supplied is not written.
    #[test]
    fn supplied_inputs_are_recorded_in_a_fixed_order() {
        let given = |authority: &'static str, decisions: &'static str| Supplied {
            authority: Some(authority),
            at: Some("2026-10-04T00:00:00Z"),
            decisions: Some(decisions),
        };
        let one = explanation(
            &[],
            given(
                "- {capability: z, decision: denied}\n- {capability: cap, decision: granted}\n",
                "- {decision: x, outcome: closed, principal: q, case_revision: c1}\n\
                 - {decision: x, outcome: closed, principal: p, case_revision: c1}\n",
            ),
        );
        let two = explanation(
            &[],
            given(
                "- {capability: cap, decision: granted}\n- {capability: z, decision: denied}\n",
                "- {decision: x, outcome: closed, principal: p, case_revision: c1}\n\
                 - {decision: x, outcome: closed, principal: q, case_revision: c1}\n",
            ),
        );
        assert_eq!(one, two);
        let from = &one["computed_from"];
        assert_eq!(
            from["authority"],
            json!([
                {"capability": "cap", "decision": "granted"},
                {"capability": "z", "decision": "denied"},
            ])
        );
        assert_eq!(
            from["decisions"],
            json!([
                {"case_revision": "c1", "decision": "x", "outcome": "closed", "principal": "p"},
                {"case_revision": "c1", "decision": "x", "outcome": "closed", "principal": "q"},
            ])
        );
        assert_eq!(from["at"], json!("2026-10-04T00:00:00Z"));
        assert_eq!(from["case"]["revision"], json!("c1"));
        assert_eq!(
            from["semantics"],
            json!({"canon": CANON_VERSION, "format": "canon-decision/1"})
        );

        let bare = explanation(&[], Supplied::default());
        let mut keys: Vec<&String> = bare["computed_from"]
            .as_object()
            .expect("an object")
            .keys()
            .collect();
        keys.sort();
        assert_eq!(
            keys,
            [
                "case",
                "evidence",
                "protocol",
                "protocol_revision",
                "semantics"
            ]
        );
    }
}
