//! Adversary pass 1 on story:decision-grounds: the rules of CANON-GROUND-001 that no conformance
//! step and no test of the unit pins, each found by a mutant of `eval/grounds.rs`,
//! `eval/authority.rs`, `eval/decisions.rs` or `eval/actions.rs` that left the unit's gate green.
//!
//! - `stale_grounds` names the failed grounds of the set-aside entries "on its capability that cover
//!   it" (`eval/mod.rs`, "Authority decisions"): an entry on another capability, or at a scope that
//!   does not cover the requirement, adds none.
//! - Only a `"decision": "none"` reason gains `stale_grounds`: a requirement an entry that applies
//!   decides as `require_approval` names no set-aside entry, whatever that entry decided.
//! - An outcome's reason names the set-aside decisions "of that name, for that outcome": a decision
//!   for another outcome requiring the same decision name adds nothing.
//! - A record of a kind no claim reads is applicable (story:missing-evidence, scope as built), so an
//!   evidence ground on it holds.

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, Decision};
use serde_json::{Value, json};

const PROTOCOL: &str = "format: protocol/1
protocol: {id: grounds-probe, revision: 1}
artifacts: {spec: {}, up: {}}
evidence_kinds: {k: {}, unread: {}}
claims:
  c: {true_when: {evidence: {kind: k}}}
actions:
  scoped: {requires: [{capability: x, scope: {ns: a}}]}
  other: {requires: [{capability: y}]}
outcomes:
  first: {requires: {decision: d}}
  second: {requires: {decision: d}}
";

const CASE: &str = "format: canon-case/1\nid: G-1\nprotocol: grounds-probe\nrevision: c2\n\
                    artifacts: {spec: {revision: r2}, up: {revision: u2}}\n";

fn evaluate(evidence: &[&str], supplied: Supplied<'_>) -> Decision {
    let ir = ir::compile(&model::parse(PROTOCOL).expect("parses")).expect("compiles");
    let case = eval::read_case(CASE).expect("case reads");
    let records: Vec<_> = evidence
        .iter()
        .map(|text| eval::read_evidence(text).expect("evidence reads"))
        .collect();
    eval::evaluate_with(&ir, &case, &records, supplied).expect("decides")
}

fn authority(text: &str) -> Decision {
    evaluate(
        &[],
        Supplied {
            authority: Some(text),
            ..Supplied::default()
        },
    )
}

fn action(decision: &Decision, id: &str) -> Value {
    decision.actions.as_ref().expect("actions")[id].clone()
}

fn outcome(decision: &Decision, id: &str) -> Value {
    decision.outcomes.as_ref().expect("outcomes")[id].clone()
}

/// Two entries set aside by their grounds, one on `x` and one on `y`: each requirement names the
/// failed ground of the entry on its own capability only.
#[test]
fn a_set_aside_entry_on_another_capability_adds_no_ground() {
    let decision = authority(
        "- {capability: y, decision: granted, ground_revisions: {up: u1}}\n\
         - {capability: x, scope: {ns: a}, decision: granted, ground_revisions: {spec: r1}}\n",
    );
    assert_eq!(
        action(&decision, "scoped"),
        json!({"status": "approval-required", "reasons": [
            {"capability": "x", "decision": "none", "scope": {"ns": "a"},
             "stale_grounds": [{"artifact": "spec", "reason": "revision_mismatch", "revision": "r1"}]},
        ]})
    );
    assert_eq!(
        action(&decision, "other"),
        json!({"status": "approval-required", "reasons": [
            {"capability": "y", "decision": "none",
             "stale_grounds": [{"artifact": "up", "reason": "revision_mismatch", "revision": "u1"}]},
        ]})
    );
}

/// An entry at `{ns: b}` set aside by its grounds would not have covered a requirement at
/// `{ns: a}`: the reason is the one no entry gives, without `stale_grounds`.
#[test]
fn a_set_aside_entry_at_a_scope_that_does_not_cover_adds_no_ground() {
    let decision = authority(
        "- {capability: x, scope: {ns: b}, decision: granted, ground_revisions: {spec: r1}}\n",
    );
    assert_eq!(
        action(&decision, "scoped"),
        json!({"status": "approval-required", "reasons": [
            {"capability": "x", "decision": "none", "scope": {"ns": "a"}},
        ]})
    );
}

/// A narrower denial set aside by its grounds takes no part in precedence beside an unscoped grant
/// that applies.
#[test]
fn a_set_aside_narrower_denial_beside_a_wider_grant_is_admissible() {
    let decision = authority(
        "- {capability: x, decision: granted}\n\
         - {capability: x, scope: {ns: a}, decision: denied, ground_revisions: {spec: r1}}\n",
    );
    assert_eq!(action(&decision, "scoped"), json!({"status": "admissible"}));
}

/// A requirement an entry that applies decides as `require_approval` is not undecided: its reason
/// gains no `stale_grounds`, whether the set-aside entry beside it granted or denied.
#[test]
fn a_require_approval_reason_names_no_stale_ground() {
    for set_aside in ["granted", "denied"] {
        let decision = authority(&format!(
            "- {{capability: x, decision: require_approval}}\n\
             - {{capability: x, decision: {set_aside}, principal: p, ground_revisions: {{spec: r1}}}}\n"
        ));
        assert_eq!(
            action(&decision, "scoped"),
            json!({"status": "approval-required", "reasons": [
                {"capability": "x", "decision": "require_approval", "scope": {"ns": "a"}},
            ]}),
            "set-aside entry {set_aside}"
        );
    }
}

/// Two outcomes require the decision `d`. A decision for `first` set aside by its grounds adds its
/// ground to `first`'s reason only; `second`, with no decision of its own, gets the plain reason.
#[test]
fn a_decision_for_another_outcome_adds_no_ground_to_this_one() {
    let decision = evaluate(
        &[],
        Supplied {
            decisions: Some(
                "- {decision: d, outcome: first, principal: p, case_revision: c2, \
                 ground_revisions: {spec: r1}}\n",
            ),
            ..Supplied::default()
        },
    );
    assert_eq!(
        outcome(&decision, "first"),
        json!({"status": "blocked", "reasons": [
            {"decision": "d", "present": false,
             "stale_grounds": [{"artifact": "spec", "reason": "revision_mismatch", "revision": "r1"}]},
        ]})
    );
    assert_eq!(
        outcome(&decision, "second"),
        json!({"status": "blocked", "reasons": [{"decision": "d", "present": false}]})
    );
}

/// A record of a kind no claim reads is applicable when the revision-binding stage leaves it, so
/// an evidence ground on it holds; bound to another revision of its subject it fails as
/// `revision_mismatch`.
#[test]
fn an_evidence_ground_on_a_record_no_claim_reads() {
    let record = |revision: &str| {
        format!(
            "format: canon-evidence/1\nid: e1\nkind: unread\nsubject: spec\n\
             subject_revision: {revision}\n"
        )
    };
    let grant = "- {capability: y, decision: granted, ground_evidence: [e1]}\n";
    let supplied = Supplied {
        authority: Some(grant),
        ..Supplied::default()
    };
    assert_eq!(
        action(&evaluate(&[&record("r2")], supplied), "other"),
        json!({"status": "admissible"})
    );
    assert_eq!(
        action(&evaluate(&[&record("r1")], supplied), "other"),
        json!({"status": "approval-required", "reasons": [
            {"capability": "y", "decision": "none",
             "stale_grounds": [{"evidence": "e1", "reason": "revision_mismatch"}]},
        ]})
    );
}
