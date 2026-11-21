//! The `missing_evidence` section of `canon-decision/1`: for each admissible action that may
//! produce evidence, the kinds it may produce that have no applicable record
//! (story:missing-evidence, CANON-FRONTIER-001).
//!
//! The section maps an action's id to the evidence kinds of its `may_produce` that no applicable
//! record ([`applicable`]) has, in kind-identifier order. An empty list means every kind the action
//! may produce has an applicable record. Not listed: an action that is `blocked` or
//! `approval-required`, and an action whose `may_produce` is empty. The section is written only
//! when at least one action is listed, so a decision without one is exactly what it was before the
//! section existed.
//!
//! It reads the action statuses the `actions` section holds, each action's `may_produce` and the
//! evidence records, and nothing else. It does not say that running the action succeeds, that it
//! produces the kind, which result a new record would carry, that a new record would pass an
//! independence demand, or that a claim would become `true`; it reads no run history; a kind that
//! declares `supplied_by` is listed like any other.

use std::collections::BTreeSet;

use crate::ir::Ir;
use crate::model::{
    ActionId, ClaimDecision, Declarations, EvidenceId, EvidenceKindId, EvidenceRecord, Json,
};

/// The records of `left` that are applicable: those no claim lists in its `excluded_evidence`.
///
/// **The rule.** A record is applicable when the revision-binding and freshness stages leave it
/// and no claim lists it in `excluded_evidence`, as `invalidated` or as `not_independent`. `left`
/// is the set those two stages leave (a record set aside by either is not in it, and is listed as
/// `revision_mismatch` or `expired`), and `claims` holds every claim's [`ClaimDecision`] of the
/// decision. The rule reads the record's place in the stages and nothing of its content: its
/// result does not matter, and neither does whether its kind declares `supplied_by`. A record one
/// claim lists, whichever claim and however many others read it without listing it, is not
/// applicable. The records come back in the order of `left`.
pub(crate) fn applicable<'a, 'c>(
    left: &'a [EvidenceRecord],
    claims: impl IntoIterator<Item = &'c ClaimDecision>,
) -> Vec<&'a EvidenceRecord> {
    let listed: BTreeSet<&EvidenceId> = claims
        .into_iter()
        .flat_map(|claim| &claim.excluded_evidence)
        .map(|exclusion| &exclusion.evidence)
        .collect();
    left.iter()
        .filter(|record| !listed.contains(&record.id))
        .collect()
}

/// The section, or `None` when no admissible action may produce a kind. `actions` is the decision's
/// `actions` section (an action is listed only when its `status` is `admissible`), `left` the
/// records the revision-binding and freshness stages left, and `claims` every claim's entry, as
/// [`applicable`] takes them.
pub(super) fn section<'c>(
    ir: &Ir,
    actions: Option<&Json>,
    left: &[EvidenceRecord],
    claims: impl IntoIterator<Item = &'c ClaimDecision>,
) -> Option<Declarations<ActionId, Vec<EvidenceKindId>>> {
    let actions = actions?;
    let present: BTreeSet<&EvidenceKindId> = applicable(left, claims)
        .into_iter()
        .map(|record| &record.kind)
        .collect();
    let listed: Vec<(ActionId, Vec<EvidenceKindId>)> = ir
        .actions
        .iter()
        .filter(|(_, action)| !action.may_produce.is_empty())
        .filter(|(id, _)| admissible(actions, id))
        .map(|(id, action)| {
            let kinds: BTreeSet<&EvidenceKindId> = action.may_produce.iter().collect();
            let lacking = kinds
                .into_iter()
                .filter(|kind| !present.contains(kind))
                .cloned()
                .collect();
            (id.clone(), lacking)
        })
        .collect();
    (!listed.is_empty()).then(|| Declarations::new(listed))
}

/// Whether the `actions` section gives `action` the status `admissible`.
fn admissible(actions: &Json, action: &ActionId) -> bool {
    actions
        .get(action.as_str())
        .and_then(|entry| entry.get("status"))
        .and_then(Json::as_str)
        == Some("admissible")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::{Supplied, evaluate_with, read_case, read_evidence, render};
    use crate::model::Decision;

    const PROTOCOL: &str = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        artifacts: {a: {}, up: {}}\n\
        evidence_kinds:\n\
        \x20\x20k: {max_age: 1h}\n\
        \x20\x20m: {}\n\
        \x20\x20s: {supplied_by: other}\n\
        claims:\n\
        \x20\x20c: {true_when: {evidence: {kind: k}}}\n\
        invalidation:\n\
        \x20\x20r: {upstream: up, invalidates: [c]}\n\
        actions:\n\
        \x20\x20both: {may_produce: [{evidence: m}, {evidence: k}]}\n\
        \x20\x20gated: {requires: [{capability: cap}], may_produce: [{evidence: m}]}\n\
        \x20\x20shut: {precondition: {any: []}, may_produce: [{evidence: m}]}\n\
        \x20\x20silent: {}\n\
        \x20\x20supplied: {may_produce: [{evidence: s}]}\n";

    const CASE: &str = "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r1}, up: {revision: u1}}\n";

    fn record(id: &str, kind: &str, extra: &str) -> String {
        format!(
            "format: canon-evidence/1\nid: {id}\nkind: {kind}\nresult: any\nsubject: a\nsubject_revision: r1\n{extra}"
        )
    }

    fn decide(case: &str, evidence: &[String], at: Option<&str>) -> Decision {
        let ir =
            crate::ir::compile(&crate::model::parse(PROTOCOL).expect("parses")).expect("compiles");
        let case = read_case(case).expect("case reads");
        let evidence: Vec<_> = evidence
            .iter()
            .map(|text| read_evidence(text).expect("evidence reads"))
            .collect();
        let supplied = Supplied {
            at,
            ..Supplied::default()
        };
        evaluate_with(&ir, &case, &evidence, supplied).expect("decides")
    }

    fn missing(decision: &Decision) -> Vec<(String, Vec<String>)> {
        decision
            .missing_evidence
            .as_ref()
            .expect("the section is written")
            .iter()
            .map(|(action, kinds)| {
                (
                    action.as_str().to_owned(),
                    kinds.iter().map(|kind| kind.as_str().to_owned()).collect(),
                )
            })
            .collect()
    }

    fn entry(action: &str, kinds: &[&str]) -> (String, Vec<String>) {
        (
            action.to_owned(),
            kinds.iter().map(|kind| (*kind).to_owned()).collect(),
        )
    }

    #[test]
    fn only_admissible_actions_that_produce_evidence_are_listed_with_their_kinds_in_order() {
        // `gated` is approval-required, `shut` blocked and `silent` produces nothing; `both` lists
        // m before k in the protocol and the section lists k first.
        let decision = decide(CASE, &[], None);
        assert_eq!(
            missing(&decision),
            [entry("both", &["k", "m"]), entry("supplied", &["s"])]
        );
    }

    #[test]
    fn a_record_of_a_kind_ends_its_listing_whatever_its_result_or_where_the_kind_comes_from() {
        let evidence = [
            record("e1", "k", ""),
            record("e2", "s", "upstream_revisions: {elsewhere: x1}\n"),
        ];
        let decision = decide(CASE, &evidence, None);
        assert_eq!(
            missing(&decision),
            [entry("both", &["m"]), entry("supplied", &[])]
        );
    }

    #[test]
    fn a_record_the_binding_freshness_or_invalidation_stage_keeps_from_the_claim_is_not_applicable()
    {
        let stale = record("e1", "k", "observed_at: 2026-01-01T00:00:00Z\n");
        let mismatch =
            "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r0\n";
        let invalidated = record("e1", "k", "upstream_revisions: {up: u0}\n");
        for (what, evidence, at) in [
            ("expired", stale, Some("2026-01-01T02:00:00Z")),
            ("revision_mismatch", mismatch.to_owned(), None),
            ("invalidated", invalidated, None),
        ] {
            let decision = decide(CASE, &[evidence], at);
            assert_eq!(missing(&decision)[0], entry("both", &["k", "m"]), "{what}");
        }
        let fresh = record("e1", "k", "observed_at: 2026-01-01T00:00:00Z\n");
        let decision = decide(CASE, &[fresh], Some("2026-01-01T00:30:00Z"));
        assert_eq!(missing(&decision)[0], entry("both", &["m"]));
    }

    #[test]
    fn a_decision_without_a_qualifying_action_has_no_section_and_is_written_as_before() {
        let protocol = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
            evidence_kinds: {k: {}}\nactions:\n  silent: {}\n  \
            shut: {precondition: {evidence: {kind: k}}, may_produce: [{evidence: k}]}\n";
        let ir =
            crate::ir::compile(&crate::model::parse(protocol).expect("parses")).expect("compiles");
        let case =
            read_case("format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r1}}\n")
                .expect("case reads");
        let decision = evaluate_with(&ir, &case, &[], Supplied::default()).expect("decides");
        assert_eq!(decision.missing_evidence, None);
        assert!(!render(&decision).contains("missing_evidence"));
    }

    #[test]
    fn applicable_drops_every_record_a_claim_lists_and_keeps_the_order() {
        use crate::model::{EvidenceExclusion, ExclusionReason, Truth};
        let left: Vec<EvidenceRecord> = ["e3", "e1", "e2"]
            .iter()
            .map(|id| read_evidence(&record(id, "k", "")).expect("evidence reads"))
            .collect();
        let claim = |ids: &[&str]| ClaimDecision {
            value: Truth::Unknown,
            excluded_evidence: ids
                .iter()
                .map(|id| EvidenceExclusion {
                    evidence: EvidenceId::new(*id),
                    reason: ExclusionReason::NotIndependent,
                })
                .collect(),
        };
        let claims = [claim(&[]), claim(&["e1"]), claim(&["e3"])];
        let ids: Vec<&str> = applicable(&left, &claims)
            .iter()
            .map(|record| record.id.as_str())
            .collect();
        assert_eq!(ids, ["e2"]);
        let none: Vec<&ClaimDecision> = Vec::new();
        assert_eq!(applicable(&left, none).len(), 3);
    }
}
