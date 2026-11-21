//! Adversary cases for story:pending-decisions (wave 2026-11-20-w5, pass 1), against the evaluator
//! library: the `pending_decisions` section of `canon-decision/1`.
//!
//! Driven from CANON-PENDING-001 (`conformance/requirements.yaml`) and the story's § Outcome: one
//! `authority` entry per `none` or `require_approval` reason of an `approval-required` action, none
//! for a blocked action, ordered by action, capability and scope; one `decisions` entry per outcome
//! blocked with `{"decision": <name>, "present": false}`, in outcome order, `options` `["taken"]`
//! with a case revision and `[]` without; the section only when it has an entry. Each case builds
//! a ground the unit's scenario and tests do not: identifiers that spell the reason keys and
//! values, one decision name required by two outcomes, precedence and lapse at the edges, scopes
//! whose order differs between code point and a naive reading, and a terminated case.

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, Decision};
use serde_json::json;

fn compiled(protocol: &str) -> ir::Ir {
    ir::compile(&model::parse(protocol).expect("protocol parses")).expect("protocol compiles")
}

fn decide(ir: &ir::Ir, case: &str, authority: Option<&str>, decisions: Option<&str>) -> Decision {
    let case = eval::read_case(case).expect("case reads");
    eval::evaluate_with(
        ir,
        &case,
        &[],
        Supplied {
            authority,
            decisions,
            ..Supplied::default()
        },
    )
    .expect("decides")
}

const AT_C2: &str = "format: canon-case/1\nid: C\nprotocol: p\nrevision: c2\nartifacts: {}\n";
const NO_REVISION: &str = "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {}\n";

fn authority_entry(
    action: &str,
    capability: &str,
    current: &str,
    scope: Option<serde_json::Value>,
    revision: Option<&str>,
) -> serde_json::Value {
    let mut entry = json!({"action": action, "capability": capability, "current": current, "options": ["granted", "denied"]});
    if let Some(scope) = scope {
        entry["scope"] = scope;
    }
    if let Some(revision) = revision {
        entry["case_revision"] = json!(revision);
    }
    entry
}

/// Capabilities, outcomes and decisions named after the keys and values the section reads
/// (`none`, `require_approval`, `decision`, `present`, `denied`) are entries like any other: the
/// section reads a reason's fields, never an identifier's spelling.
#[test]
fn identifiers_that_spell_reason_keys_are_plain_entries() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
         actions:\n  \
         none:\n    requires: [{capability: none}, {capability: require_approval}, {capability: denied}]\n  \
         present:\n    requires: [{capability: decision, scope: {decision: none}}, {capability: present}]\n\
         outcomes:\n  \
         decision: {requires: {decision: present}}\n  \
         present: {requires: {decision: none}}\n",
    );
    let decision = decide(
        &ir,
        AT_C2,
        Some(
            "- {capability: present, decision: granted}\n- {capability: denied, decision: require_approval}\n",
        ),
        Some("- {decision: none, outcome: present, principal: q, case_revision: c1}\n"),
    );
    assert_eq!(
        decision.pending_decisions,
        Some(json!({
            "authority": [
                authority_entry("none", "denied", "require_approval", None, Some("c2")),
                authority_entry("none", "none", "none", None, Some("c2")),
                authority_entry("none", "require_approval", "none", None, Some("c2")),
                authority_entry("present", "decision", "none", Some(json!({"decision": "none"})), Some("c2")),
            ],
            "decisions": [
                {"outcome": "decision", "decision": "present", "case_revision": "c2", "options": ["taken"]},
                {"outcome": "present", "decision": "none", "case_revision": "c2", "options": ["taken"]},
            ],
        }))
    );
}

/// Two outcomes requiring one decision name are two entries; a decision taken at the snapshot's
/// revision for one removes only that one's entry, and one taken at another revision for the other
/// leaves its entry.
#[test]
fn one_decision_name_required_by_two_outcomes_is_decided_per_outcome() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
         outcomes:\n  y: {requires: {decision: d}}\n  x: {requires: {decision: d}}\n",
    );
    let decision = decide(&ir, AT_C2, None, None);
    assert_eq!(
        decision.pending_decisions,
        Some(json!({"decisions": [
            {"outcome": "x", "decision": "d", "case_revision": "c2", "options": ["taken"]},
            {"outcome": "y", "decision": "d", "case_revision": "c2", "options": ["taken"]},
        ]}))
    );
    let decision = decide(
        &ir,
        AT_C2,
        None,
        Some(
            "- {decision: d, outcome: x, principal: q, case_revision: c2}\n\
              - {decision: d, outcome: y, principal: q, case_revision: c1}\n",
        ),
    );
    assert_eq!(
        decision.pending_decisions,
        Some(json!({"decisions": [
            {"outcome": "y", "decision": "d", "case_revision": "c2", "options": ["taken"]},
        ]}))
    );
}

const SCOPED: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
     actions:\n  \
     read:\n    requires: [{capability: m, scope: {cluster: a}}, {capability: m, scope: {cluster: b}}]\n  \
     write:\n    requires: [{capability: w}, {capability: m, scope: {cluster: a}}]\n";

/// A `denied` and a `require_approval` entry covering one requirement: denial wins, so the action is
/// blocked and none of its requirements is listed, not even the one nothing decides. A `granted`
/// and a `require_approval` entry on one requirement: `require_approval` wins and the entry stays
/// with `current: require_approval`, even when the `granted` entry is bound to the snapshot's
/// revision and names the requirement's exact scope.
#[test]
fn precedence_at_the_edges_follows_the_actions_section() {
    let ir = compiled(SCOPED);
    let decision = decide(
        &ir,
        AT_C2,
        Some(
            "- {capability: m, scope: {cluster: a}, decision: require_approval}\n\
              - {capability: m, decision: denied}\n",
        ),
        None,
    );
    assert_eq!(decision.pending_decisions, None, "both actions are blocked");
    let decision = decide(
        &ir,
        AT_C2,
        Some(
            "- {capability: m, decision: require_approval}\n\
              - {capability: m, scope: {cluster: a}, decision: granted, case_revision: c2}\n\
              - {capability: m, scope: {cluster: b}, decision: granted}\n\
              - {capability: w, decision: granted}\n",
        ),
        None,
    );
    assert_eq!(
        decision.pending_decisions,
        Some(json!({"authority": [
            authority_entry("read", "m", "require_approval", Some(json!({"cluster": "a"})), Some("c2")),
            authority_entry("read", "m", "require_approval", Some(json!({"cluster": "b"})), Some("c2")),
            authority_entry("write", "m", "require_approval", Some(json!({"cluster": "a"})), Some("c2")),
        ]}))
    );
}

/// One capability required at two scopes, one granted: only the other is listed. One of two
/// requirements denied: the action is blocked and the other is not listed, while another action
/// needing the same undecided requirement still lists it.
#[test]
fn one_scope_granted_and_one_requirement_denied() {
    let ir = compiled(SCOPED);
    let decision = decide(
        &ir,
        AT_C2,
        Some(
            "- {capability: m, scope: {cluster: b}, decision: granted}\n- {capability: w, decision: denied}\n",
        ),
        None,
    );
    assert_eq!(
        decision.pending_decisions,
        Some(json!({"authority": [
            authority_entry("read", "m", "none", Some(json!({"cluster": "a"})), Some("c2")),
        ]}))
    );
}

/// A snapshot without a revision: an entry bound to any case revision applies to nothing, so a
/// requirement covered only by one is listed with `current: none`, no `case_revision`, and the
/// options still `granted` and `denied`; an unbound entry still decides.
#[test]
fn a_snapshot_without_revision_lapses_every_bound_entry() {
    let ir = compiled(SCOPED);
    let decision = decide(
        &ir,
        NO_REVISION,
        Some(
            "- {capability: m, decision: granted, case_revision: c2}\n\
              - {capability: m, scope: {cluster: b}, decision: require_approval, case_revision: c2}\n\
              - {capability: w, decision: granted}\n",
        ),
        None,
    );
    assert_eq!(
        decision.pending_decisions,
        Some(json!({"authority": [
            authority_entry("read", "m", "none", Some(json!({"cluster": "a"})), None),
            authority_entry("read", "m", "none", Some(json!({"cluster": "b"})), None),
            authority_entry("write", "m", "none", Some(json!({"cluster": "a"})), None),
        ]}))
    );
    let listed = ["pending_decisions".to_owned()].into_iter().collect();
    let bytes = eval::render_sections(&decision, &listed);
    assert!(!bytes.contains("case_revision"), "{bytes}");
}

/// Scopes ordered by code point, key before value: an upper-case key before a lower-case one, a
/// non-ASCII key last, `B` before `a`, and scalar values read as text (`10` before `9`).
#[test]
fn scope_order_is_by_code_point_key_before_value() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
         actions:\n  act:\n    requires:\n      \
         - {capability: m, scope: {\"\u{e9}\": a}}\n      \
         - {capability: m, scope: {zone: b}}\n      \
         - {capability: m, scope: {zone: 9}}\n      \
         - {capability: m, scope: {zone: a}}\n      \
         - {capability: m, scope: {Zone: z}}\n      \
         - {capability: m, scope: {zone: B}}\n      \
         - {capability: m, scope: {zone: 10}}\n      \
         - {capability: m}\n",
    );
    let decision = decide(&ir, NO_REVISION, None, None);
    let scopes: Vec<serde_json::Value> =
        decision.pending_decisions.expect("a section")["authority"]
            .as_array()
            .expect("authority entries")
            .iter()
            .map(|entry| {
                entry
                    .get("scope")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null)
            })
            .collect();
    assert_eq!(
        scopes,
        [
            serde_json::Value::Null,
            json!({"Zone": "z"}),
            json!({"zone": "10"}),
            json!({"zone": "9"}),
            json!({"zone": "B"}),
            json!({"zone": "a"}),
            json!({"zone": "b"}),
            json!({"\u{e9}": "a"}),
        ]
    );
}

/// The snapshot's `termination` is not read: a case terminated through an outcome whose decision
/// applies still lists the other outcome and the action waiting on approval.
#[test]
fn a_terminated_case_still_lists_what_is_pending() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
         actions:\n  act: {requires: [{capability: m}]}\n\
         outcomes:\n  done: {requires: {decision: d}}\n  other: {requires: {decision: e}}\n",
    );
    let decision = decide(
        &ir,
        "format: canon-case/1\nid: C\nprotocol: p\nrevision: c2\nartifacts: {}\ntermination: done\n",
        None,
        Some("- {decision: d, outcome: done, principal: q, case_revision: c2}\n"),
    );
    assert_eq!(
        decision.pending_decisions,
        Some(json!({
            "authority": [authority_entry("act", "m", "none", None, Some("c2"))],
            "decisions": [{"outcome": "other", "decision": "e", "case_revision": "c2", "options": ["taken"]}],
        }))
    );
}
