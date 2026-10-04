//! The `actions` section of `canon-decision/1`: whether each declared action is admissible under
//! the claim values and the authority given (design § 10, CANON-AUTHORITY-001).
//!
//! Every action the protocol declares is listed, keyed by its id, with a `status` and, unless it is
//! admissible, the `reasons` that decide that status:
//!
//! - `blocked`, when the precondition is not `true`. Each claim whose test decides that is a
//!   reason, `{"claim": <id>, "value": <the claim's value>}`, once per claim in claim-id order. A
//!   test decides it when it is not what the precondition needs of it, with polarity carried
//!   through `not` (`{not: {claim: a}}` names `a` when `a` is `true`, as `{claim: a, is: false}`
//!   does); a test that is as needed is never named, nor is one in an `any` branch that holds. In
//!   an `all` that is `false`, only its `false` members decide it. Each evidence match that decides
//!   it, by the same rule, is a reason too, `{"evidence": <kind>, "present": <bool>}`, with
//!   `"subject": <artifact>` added when the match names one; once per kind and subject, in kind
//!   order and then subject order (no subject first), after the claim reasons, whether or not a
//!   claim is also named. `present` is whether a record the match reads applies (one of its kind
//!   and, when it names a subject, about that artifact): `false` when none does, `true` when one
//!   does (one under `not` that a record satisfies, or a match whose records have another
//!   result). A record about another artifact than the subject does not make it `true`. A
//!   precondition that no claim test and no evidence
//!   match decides, such as `{any: []}` or `{not: {all: []}}`, can never be `true`, and gives the
//!   one reason `{"requirement": "unsatisfiable"}`. Authority is not consulted.
//! - `blocked`, when the precondition is `true` and the authority denies a capability the action
//!   requires: each denied capability is a reason, `{"capability": <id>, "decision": "denied"}`.
//!   Asking again would not change a denial.
//! - `approval-required`, when the precondition is `true`, nothing is denied, and the authority
//!   decides some required capability not at all (no `--authority` decides nothing): each such
//!   capability is a reason, `{"capability": <id>, "decision": "none"}`.
//! - `admissible`, when the precondition is `true` and every required capability is granted. An
//!   action with no precondition and no requirement is always admissible.
//!
//! Capability reasons are in capability order, each capability once, whatever order the IR
//! lists them in. A protocol that declares no action has no `actions`
//! section.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};

use super::authority::Authority;
use super::claims::{predicate, reads};
use crate::ir::{Action, Ir};
use crate::model::{
    ArtifactId, CapabilityId, ClaimId, EvidenceKindId, EvidenceRecord, Grant, Json, Predicate,
    Truth,
};

/// The section, or `None` to leave the slot empty. `evidence` is what the claims were evaluated
/// over (after the exclusion stages); a precondition is evaluated over it and `claims` with
/// `super::claims::predicate`.
pub(super) fn section(
    ir: &Ir,
    claims: &BTreeMap<ClaimId, Truth>,
    evidence: &[EvidenceRecord],
    authority: Option<&Authority>,
) -> Option<Json> {
    if ir.actions.is_empty() {
        return None;
    }
    let actions = ir
        .actions
        .iter()
        .map(|(id, action)| {
            let (status, reasons) = admissibility(action, claims, evidence, authority);
            let mut entry = Map::new();
            entry.insert("status".to_owned(), Value::from(status));
            if !reasons.is_empty() {
                entry.insert("reasons".to_owned(), Value::Array(reasons));
            }
            (id.as_str().to_owned(), Value::Object(entry))
        })
        .collect();
    Some(Value::Object(actions))
}

/// One action's status and the reasons that decide it, as the module docs give them.
fn admissibility(
    action: &Action,
    claims: &BTreeMap<ClaimId, Truth>,
    evidence: &[EvidenceRecord],
    authority: Option<&Authority>,
) -> (&'static str, Vec<Value>) {
    let precondition = predicate(&action.precondition, claims, evidence);
    if precondition != Truth::True {
        return ("blocked", unmet(&action.precondition, claims, evidence));
    }
    let decision = |capability| authority.and_then(|authority| authority.decision(capability));
    // Each capability once, in capability order, whatever order the IR holds them in.
    let required: BTreeSet<&CapabilityId> = action.requires.iter().collect();
    let with = |wanted: Option<Grant>| -> Vec<Value> {
        required
            .iter()
            .filter(|capability| decision(capability) == wanted)
            .map(|capability| {
                reason([
                    ("capability", capability.as_str()),
                    ("decision", wanted.map_or("none", Grant::as_str)),
                ])
            })
            .collect()
    };
    let denied = with(Some(Grant::Denied));
    if !denied.is_empty() {
        return ("blocked", denied);
    }
    let undecided = with(None);
    if !undecided.is_empty() {
        return ("approval-required", undecided);
    }
    ("admissible", Vec::new())
}

/// The claim tests and evidence matches that decide a precondition, each named once.
#[derive(Default)]
struct Named<'a> {
    claims: BTreeSet<&'a ClaimId>,
    /// Each evidence match by kind and subject: two matches that differ only in their subject are
    /// two reasons.
    matches: BTreeSet<(&'a EvidenceKindId, Option<&'a ArtifactId>)>,
}

/// The reasons a precondition (not `true`) is unmet: each claim whose test decides it, with the
/// claim's value, then each evidence match (kind and subject) that decides it, with whether a
/// record it reads applies; or, when no test and no match decides it,
/// `{"requirement": "unsatisfiable"}`.
pub(super) fn unmet(
    precondition: &Predicate,
    claims: &BTreeMap<ClaimId, Truth>,
    evidence: &[EvidenceRecord],
) -> Vec<Value> {
    let mut named = Named::default();
    deciding(precondition, true, claims, evidence, &mut named);
    if named.claims.is_empty() && named.matches.is_empty() {
        return vec![reason([("requirement", "unsatisfiable")])];
    }
    let claim_reasons = named.claims.into_iter().map(|claim| {
        let value = claims.get(claim).copied().unwrap_or(Truth::Unknown);
        reason([("claim", claim.as_str()), ("value", &value.to_string())])
    });
    let evidence_reasons = named.matches.into_iter().map(|(kind, subject)| {
        let present = evidence.iter().any(|record| reads(kind, subject, record));
        let mut entry = Map::from_iter([
            ("evidence".to_owned(), Value::from(kind.as_str())),
            ("present".to_owned(), Value::Bool(present)),
        ]);
        if let Some(subject) = subject {
            entry.insert("subject".to_owned(), Value::from(subject.as_str()));
        }
        Value::Object(entry)
    });
    claim_reasons.chain(evidence_reasons).collect()
}

/// Adds to `named` each claim test and evidence match that decides why `node`, read with `wanted`
/// polarity (`true` outside a `not`, flipped by each `not`), is not what is wanted: under
/// `wanted`, `node` should be `true`; otherwise `false`. Called only on a node that is not what is
/// wanted.
///
/// - A claim test or an evidence match that is not what is wanted is named.
/// - A conjunction (`all` wanted true, `any` wanted false) is not what is wanted because of its
///   members that are the opposite of what is wanted, when there are any, and otherwise because
///   of its `unknown` members. A member that is as wanted never decides it.
/// - A disjunction (`any` wanted true, `all` wanted false) is not what is wanted because no member
///   is: every member decides it. One with no member names nothing: no test can satisfy it.
fn deciding<'a>(
    node: &'a Predicate,
    wanted: bool,
    claims: &BTreeMap<ClaimId, Truth>,
    evidence: &[EvidenceRecord],
    named: &mut Named<'a>,
) {
    // The value `node` would need, and the value that is the opposite.
    let (goal, opposite) = if wanted {
        (Truth::True, Truth::False)
    } else {
        (Truth::False, Truth::True)
    };
    let value = |member: &Predicate| predicate(member, claims, evidence);
    match node {
        Predicate::Claim(test) => {
            if value(node) != goal {
                named.claims.insert(&test.claim);
            }
        }
        Predicate::Evidence(matching) => {
            if value(node) != goal {
                named
                    .matches
                    .insert((&matching.kind, matching.subject.as_ref()));
            }
        }
        Predicate::Not(inner) => deciding(inner, !wanted, claims, evidence, named),
        Predicate::All(members) | Predicate::Any(members) => {
            let conjunction = matches!(node, Predicate::All(_)) == wanted;
            let unmet: Vec<&Predicate> = members
                .iter()
                .filter(|member| value(member) != goal)
                .collect();
            let opposed: Vec<&Predicate> = unmet
                .iter()
                .copied()
                .filter(|member| value(member) == opposite)
                .collect();
            let decide = if conjunction && !opposed.is_empty() {
                opposed
            } else {
                unmet
            };
            for member in decide {
                deciding(member, wanted, claims, evidence, named);
            }
        }
    }
}

fn reason<const N: usize>(fields: [(&str, &str); N]) -> Value {
    Value::Object(
        fields
            .into_iter()
            .map(|(key, value)| (key.to_owned(), Value::from(value)))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROTOCOL: &str = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        evidence_kinds: {k: {}}\n\
        claims:\n\
          \x20\x20a: {true_when: {evidence: {kind: k, result: pass}}}\n\
          \x20\x20b: {true_when: {evidence: {kind: k, result: fail}}}\n\
        actions:\n\
          \x20\x20both: {precondition: {all: [{claim: b}, {claim: a}, {claim: b}]}}\n\
          \x20\x20negated: {precondition: {claim: a, is: false}}\n\
          \x20\x20observed: {precondition: {evidence: {kind: k}}}\n\
          \x20\x20gated: {requires: [{capability: z}, {capability: m}, {capability: d}]}\n\
          \x20\x20free: {}\n";

    fn ir(text: &str) -> Ir {
        crate::ir::compile(&crate::model::parse(text).expect("parses")).expect("compiles")
    }

    fn authority(text: &str) -> Authority {
        super::super::authority::read(Some(text))
            .expect("reads")
            .expect("given")
    }

    /// The section as canonical-order JSON text, for `a` and `b` given and no evidence.
    fn section_of(a: Truth, b: Truth, authority: Option<&Authority>) -> String {
        let claims = BTreeMap::from([(ClaimId::new("a"), a), (ClaimId::new("b"), b)]);
        let section = section(&ir(PROTOCOL), &claims, &[], authority).expect("a section");
        serde_json::to_string(&section).expect("serializes")
    }

    /// An evidence reason is keyed by kind and subject and carries the subject when its match
    /// names one; `present` counts only the records the match reads. Two matches of one kind that
    /// differ only in their subject are two reasons, the one without a subject first.
    #[test]
    fn an_evidence_reason_names_its_subject_and_counts_only_records_it_reads() {
        let ir = ir("format: protocol/1\nprotocol: {id: p, revision: 1}\n\
             artifacts: {a: {}, b: {}}\nevidence_kinds: {k: {}}\n\
             actions:\n  \
             act: {precondition: {any: [{evidence: {kind: k, subject: b}}, \
             {evidence: {kind: k, result: pass, subject: a}}, {evidence: {kind: k, result: pass}}]}}\n");
        let record = EvidenceRecord {
            format: "canon-evidence/1".to_owned(),
            id: crate::model::EvidenceId::new("e"),
            kind: EvidenceKindId::new("k"),
            result: Some("fail".to_owned()),
            subject: ArtifactId::new("a"),
            subject_revision: crate::model::Revision::new("r1"),
            observed_at: None,
            upstream_revisions: crate::model::Declarations::default(),
        };
        let section = section(&ir, &BTreeMap::new(), &[record], None).expect("a section");
        assert_eq!(
            section,
            serde_json::json!({"act": {"status": "blocked", "reasons": [
                {"evidence": "k", "present": true},
                {"evidence": "k", "present": true, "subject": "a"},
                {"evidence": "k", "present": false, "subject": "b"},
            ]}})
        );
    }

    /// Each claim whose test decides the precondition is named once, in claim-id order; an
    /// evidence match that decides it names its kind. (Pass 1: red with the then `precondition`
    /// fallback removed, `observed` had no reason. Pass 2 replaced that fallback with the evidence
    /// reason, adversary pass 2 F2.)
    #[test]
    fn an_unmet_precondition_names_each_deciding_claim_test_and_evidence_match() {
        let found = section_of(Truth::True, Truth::Unknown, None);
        assert!(
            found.contains(
                r#""both":{"reasons":[{"claim":"b","value":"unknown"}],"status":"blocked"}"#
            ),
            "{found}"
        );
        assert!(
            found.contains(
                r#""negated":{"reasons":[{"claim":"a","value":"true"}],"status":"blocked"}"#
            ),
            "{found}"
        );
        assert!(
            found.contains(
                r#""observed":{"reasons":[{"evidence":"k","present":false}],"status":"blocked"}"#
            ),
            "{found}"
        );
        // An `all` that is false is decided by its false members only: `b` (unknown) is not one.
        let found = section_of(Truth::False, Truth::Unknown, None);
        assert!(
            found.contains(
                r#""both":{"reasons":[{"claim":"a","value":"false"}],"status":"blocked"}"#
            ),
            "{found}"
        );
    }

    /// Without `--authority` nothing is decided; a denial outranks an undecided capability; each
    /// list is in capability order. Red with the order of the two checks swapped: `gated` was
    /// `approval-required` while `m` was denied.
    #[test]
    fn capabilities_are_reported_in_order_and_a_denial_blocks() {
        let found = section_of(Truth::True, Truth::True, None);
        assert!(
            found.contains(r#""gated":{"reasons":[{"capability":"d","decision":"none"},{"capability":"m","decision":"none"},{"capability":"z","decision":"none"}],"status":"approval-required"}"#),
            "{found}"
        );
        assert!(
            found.contains(r#""free":{"status":"admissible"}"#),
            "{found}"
        );
        let given =
            authority("[{capability: m, decision: denied}, {capability: z, decision: granted}]");
        let found = section_of(Truth::True, Truth::True, Some(&given));
        assert!(
            found.contains(
                r#""gated":{"reasons":[{"capability":"m","decision":"denied"}],"status":"blocked"}"#
            ),
            "{found}"
        );
        let given = authority(
            "[{capability: d, decision: granted}, {capability: m, decision: granted}, {capability: z, decision: granted}]",
        );
        let found = section_of(Truth::True, Truth::True, Some(&given));
        assert!(
            found.contains(r#""gated":{"status":"admissible"}"#),
            "{found}"
        );
    }

    const EVIDENCE_PROTOCOL: &str = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        evidence_kinds: {k: {}, l: {}}\n\
        claims:\n\
          \x20\x20a: {true_when: {evidence: {kind: k, result: pass}}}\n\
        actions:\n\
          \x20\x20absent: {precondition: {not: {evidence: {kind: l}}}}\n\
          \x20\x20mixed: {precondition: {all: [{evidence: {kind: l}}, {claim: a}]}}\n\
          \x20\x20other_result: {precondition: {evidence: {kind: k, result: pass}}}\n\
          \x20\x20never: {precondition: {any: []}}\n\
          \x20\x20never_either: {precondition: {not: {all: []}}}\n";

    fn record(text: &str) -> EvidenceRecord {
        super::super::read_evidence(&format!(
            "format: canon-evidence/1\n{text}subject: x\nsubject_revision: r1\n"
        ))
        .expect("evidence reads")
    }

    /// An evidence match that decides the precondition is a reason after the claim reasons, kept
    /// beside them, with whether a record of its kind applies; a precondition no test or match
    /// decides is unsatisfiable. Red with evidence matches not named (`named.kinds` never filled):
    /// `mixed` named `a` alone and `absent` was unsatisfiable.
    #[test]
    fn evidence_reasons_follow_claim_reasons_and_no_test_is_unsatisfiable() {
        let ir = ir(EVIDENCE_PROTOCOL);
        let found = |a: Truth, evidence: &[EvidenceRecord]| {
            let claims = BTreeMap::from([(ClaimId::new("a"), a)]);
            let section = section(&ir, &claims, evidence, None).expect("a section");
            serde_json::to_string(&section).expect("serializes")
        };
        let none = found(Truth::Unknown, &[]);
        for expected in [
            r#""mixed":{"reasons":[{"claim":"a","value":"unknown"},{"evidence":"l","present":false}],"status":"blocked"}"#,
            r#""never":{"reasons":[{"requirement":"unsatisfiable"}],"status":"blocked"}"#,
            r#""never_either":{"reasons":[{"requirement":"unsatisfiable"}],"status":"blocked"}"#,
            // No record leaves a match `unknown`, and `not` keeps it `unknown`.
            r#""absent":{"reasons":[{"evidence":"l","present":false}],"status":"blocked"}"#,
        ] {
            assert!(none.contains(expected), "{expected}\n{none}");
        }
        let some = found(
            Truth::False,
            &[
                record("id: k-1\nkind: k\nresult: fail\n"),
                record("id: l-1\nkind: l\n"),
            ],
        );
        for expected in [
            r#""absent":{"reasons":[{"evidence":"l","present":true}],"status":"blocked"}"#,
            r#""other_result":{"reasons":[{"evidence":"k","present":true}],"status":"blocked"}"#,
            r#""mixed":{"reasons":[{"claim":"a","value":"false"}],"status":"blocked"}"#,
        ] {
            assert!(some.contains(expected), "{expected}\n{some}");
        }
    }

    #[test]
    fn a_protocol_without_actions_has_no_section() {
        let bare = ir("format: protocol/1\nprotocol: {id: p, revision: 1}\n");
        assert_eq!(section(&bare, &BTreeMap::new(), &[], None), None);
    }
}
