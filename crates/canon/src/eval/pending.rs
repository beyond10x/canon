//! The `pending_decisions` section of `canon-decision/1`: the decisions the evaluation waits on,
//! each a `canon-authority/1` or `canon-decisions/1` entry that would change a status in the same
//! decision (story:pending-decisions, CANON-PENDING-001).
//!
//! The section is an object with up to two members, each written only when it has an entry; the
//! section is written only when one of them is, so a decision with nothing pending is exactly what
//! it was before the section existed.
//!
//! - `authority`: one entry per reason `{"capability", "decision": "none" | "require_approval",
//!   "scope"?}` of an `approval-required` action of the `actions` section, `{"action",
//!   "capability", "scope"?, "current", "case_revision"?, "options": ["granted", "denied"]}`.
//!   `current` is the reason's decision, `scope` is written when the requirement has one, and
//!   `case_revision` is the case snapshot's `revision`, written when it has one. A `blocked`
//!   action has no entry, whether its precondition is not `true` or a requirement is denied:
//!   asking changes neither. Entries are ordered by action id, then capability, then scope: no
//!   scope first, then two scopes by their key-value pairs in key order, compared pair by pair,
//!   key before value, each by code point, a scope that is a prefix of the other first (`{cluster:
//!   a}` before `{cluster: a, namespace: b}` before `{cluster: b}` before `{namespace: a}`). That
//!   is the order the `actions` section gives a requirement's reasons in.
//! - `decisions`: one entry per outcome reason `{"decision": <name>, "present": false}` of the
//!   `outcomes` section, `{"outcome", "decision", "case_revision"?, "options"}`, in outcome-id
//!   order. `options` is `["taken"]` when the snapshot has a revision, and `[]`, with no
//!   `case_revision`, when it has none: a `canon-decisions/1` entry applies only at the
//!   snapshot's revision, so none can apply then.
//!
//! An option is the value of the entry a caller would add, with its own principal: for an
//! authority entry, with the `capability`, the `scope` and, when present, the `case_revision`; for
//! a decision entry, with the `decision`, the `outcome` and the `case_revision`. Precedence does
//! not change: a `granted` entry decides a requirement whose `current` is `require_approval` only
//! when no `require_approval` entry still covers it (`actions.rs`).
//!
//! The section reads only what the `actions` and `outcomes` stages computed, their reasons, and
//! the case revision: coverage, scope matching, lapse and precedence are decided there once, and
//! not again here. It computes no principal, holder, expiry, routing, recommendation or grounds.

use serde_json::{Map, Value};

use crate::model::{Json, Revision};

/// The options of an authority entry.
const AUTHORITY_OPTIONS: [&str; 2] = ["granted", "denied"];
/// The option of a decision entry when the snapshot has a revision.
const DECISION_OPTION: &str = "taken";

/// The section, or `None` when nothing is pending. `actions` and `outcomes` are the decision's
/// sections as `actions.rs` and `outcomes.rs` computed them, and `revision` the case snapshot's
/// `revision`.
pub(super) fn section(
    actions: Option<&Json>,
    outcomes: Option<&Json>,
    revision: Option<&Revision>,
) -> Option<Json> {
    let mut section = Map::new();
    let authority = authority(actions, revision);
    if !authority.is_empty() {
        section.insert("authority".to_owned(), Value::Array(authority));
    }
    let decisions = decisions(outcomes, revision);
    if !decisions.is_empty() {
        section.insert("decisions".to_owned(), Value::Array(decisions));
    }
    (!section.is_empty()).then_some(Value::Object(section))
}

/// A requirement as an authority entry is ordered: action id, capability, then the scope's
/// key-value pairs in key order (empty for no scope).
type RequirementKey<'a> = (&'a str, &'a str, Vec<(&'a str, &'a str)>);

/// The `authority` entries, in requirement order.
fn authority(actions: Option<&Json>, revision: Option<&Revision>) -> Vec<Json> {
    let mut entries: Vec<(RequirementKey<'_>, Json)> = members(actions)
        .filter(|(_, entry)| {
            entry.get("status").and_then(Value::as_str) == Some("approval-required")
        })
        .flat_map(|(action, entry)| reasons(entry).map(move |reason| (action, reason)))
        .filter_map(|(action, reason)| {
            let capability = reason.get("capability")?.as_str()?;
            let current = reason.get("decision")?.as_str()?;
            if current != "none" && current != "require_approval" {
                return None;
            }
            let scope = reason.get("scope").and_then(Value::as_object);
            let mut pairs: Vec<(&str, &str)> = scope
                .into_iter()
                .flatten()
                .filter_map(|(key, value)| Some((key.as_str(), value.as_str()?)))
                .collect();
            pairs.sort_unstable();
            let mut entry = Map::from_iter([
                ("action".to_owned(), Value::from(action)),
                ("capability".to_owned(), Value::from(capability)),
                ("current".to_owned(), Value::from(current)),
                (
                    "options".to_owned(),
                    Value::from(AUTHORITY_OPTIONS.to_vec()),
                ),
            ]);
            if let Some(scope) = scope {
                entry.insert("scope".to_owned(), Value::Object(scope.clone()));
            }
            if let Some(revision) = revision {
                entry.insert("case_revision".to_owned(), Value::from(revision.as_str()));
            }
            Some(((action, capability, pairs), Value::Object(entry)))
        })
        .collect();
    entries.sort_by(|(left, _), (right, _)| left.cmp(right));
    entries.into_iter().map(|(_, entry)| entry).collect()
}

/// The `decisions` entries, in outcome-id order: one per reason `{"decision", "present": false}`,
/// with or without the `stale_grounds` of a decision set aside by its grounds
/// (story:decision-grounds), which the entry does not copy; a reason with any other key is not
/// one.
fn decisions(outcomes: Option<&Json>, revision: Option<&Revision>) -> Vec<Json> {
    let mut entries: Vec<((&str, &str), Json)> = members(outcomes)
        .filter(|(_, entry)| entry.get("status").and_then(Value::as_str) == Some("blocked"))
        .flat_map(|(outcome, entry)| reasons(entry).map(move |reason| (outcome, reason)))
        .filter_map(|(outcome, reason)| {
            let reason = reason.as_object()?;
            let present = reason.get("present")?;
            let decision = reason.get("decision")?.as_str()?;
            let other_key = reason
                .keys()
                .any(|key| !matches!(key.as_str(), "decision" | "present" | "stale_grounds"));
            if other_key || present != &Value::Bool(false) {
                return None;
            }
            let options: Vec<&str> = revision.map(|_| DECISION_OPTION).into_iter().collect();
            let mut entry = Map::from_iter([
                ("outcome".to_owned(), Value::from(outcome)),
                ("decision".to_owned(), Value::from(decision)),
                ("options".to_owned(), Value::from(options)),
            ]);
            if let Some(revision) = revision {
                entry.insert("case_revision".to_owned(), Value::from(revision.as_str()));
            }
            Some(((outcome, decision), Value::Object(entry)))
        })
        .collect();
    entries.sort_by_key(|(key, _)| *key);
    entries.into_iter().map(|(_, entry)| entry).collect()
}

/// The members of a section that maps an id to an entry; none when the slot is empty.
fn members(section: Option<&Json>) -> impl Iterator<Item = (&str, &Json)> {
    section
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .map(|(id, entry)| (id.as_str(), entry))
}

/// An entry's `reasons`; none when it has none.
fn reasons(entry: &Json) -> impl Iterator<Item = &Json> {
    entry
        .get("reasons")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn at(revision: &str) -> Option<Revision> {
        Some(Revision::new(revision))
    }

    fn requirement(capability: &str, decision: &str, scope: Option<Json>) -> Json {
        let mut reason = json!({"capability": capability, "decision": decision});
        if let Some(scope) = scope {
            reason["scope"] = scope;
        }
        reason
    }

    /// Nothing pending, no section: no slot, an admissible action, a legitimate outcome, a blocked
    /// action whatever blocks it, and an outcome blocked by its predicate.
    #[test]
    fn nothing_pending_is_no_section() {
        assert_eq!(section(None, None, at("c2").as_ref()), None);
        let actions = json!({
            "free": {"status": "admissible"},
            "denied": {"status": "blocked", "reasons": [
                {"capability": "m", "decision": "denied"},
            ]},
            "unmet": {"status": "blocked", "reasons": [{"claim": "c", "value": "unknown"}]},
        });
        let outcomes = json!({
            "done": {"status": "legitimate"},
            "decided": {"status": "legitimate", "decided_by": {"decision": "d", "principals": []}},
            "unsupported": {"status": "blocked", "reasons": [
                {"evidence": "k", "present": false},
                {"requirement": "unsatisfiable"},
            ]},
        });
        assert_eq!(
            section(Some(&actions), Some(&outcomes), at("c2").as_ref()),
            None
        );
    }

    /// Entries are ordered by action, then capability, then scope: no scope first, then by the
    /// key-value pairs in key order, pair by pair, key before value, a prefix first; whatever
    /// order the section holds the reasons in. A comparison of values before keys would put
    /// `{namespace: a}` before `{cluster: b}`, and one by size `{cluster: b}` before `{cluster:
    /// a, namespace: b}`.
    #[test]
    fn authority_entries_follow_action_capability_and_scope_order() {
        let scope = |pairs: &[(&str, &str)]| {
            Some(Json::Object(
                pairs
                    .iter()
                    .map(|(key, value)| ((*key).to_owned(), Value::from(*value)))
                    .collect(),
            ))
        };
        let actions = json!({
            "b": {"status": "approval-required", "reasons": [requirement("m", "none", None)]},
            "a": {"status": "approval-required", "reasons": [
                requirement("z", "none", None),
                requirement("m", "none", scope(&[("namespace", "a")])),
                requirement("m", "require_approval", scope(&[("cluster", "b")])),
                requirement("m", "none", scope(&[("cluster", "a"), ("namespace", "b")])),
                requirement("m", "none", scope(&[("cluster", "a")])),
                requirement("m", "none", None),
            ]},
        });
        let found = section(Some(&actions), None, None).expect("a section");
        let order: Vec<(String, String, Json)> = found["authority"]
            .as_array()
            .expect("authority entries")
            .iter()
            .map(|entry| {
                (
                    entry["action"].as_str().unwrap_or_default().to_owned(),
                    entry["capability"].as_str().unwrap_or_default().to_owned(),
                    entry.get("scope").cloned().unwrap_or(Json::Null),
                )
            })
            .collect();
        assert_eq!(
            order,
            [
                ("a", "m", Json::Null),
                ("a", "m", json!({"cluster": "a"})),
                ("a", "m", json!({"cluster": "a", "namespace": "b"})),
                ("a", "m", json!({"cluster": "b"})),
                ("a", "m", json!({"namespace": "a"})),
                ("a", "z", Json::Null),
                ("b", "m", Json::Null),
            ]
            .map(|(action, capability, scope)| (
                action.to_owned(),
                capability.to_owned(),
                scope
            ))
        );
        assert!(found.get("decisions").is_none());
    }

    /// The entry carries the reason's decision as `current`, the scope when the requirement has
    /// one and the case revision when the snapshot has one; with none, a decision entry has no
    /// option.
    #[test]
    fn an_entry_carries_the_case_revision_only_when_the_snapshot_has_one() {
        let actions = json!({"a": {"status": "approval-required", "reasons": [
            requirement("m", "require_approval", Some(json!({"cluster": "x"}))),
        ]}});
        let outcomes = json!({
            "z": {"status": "blocked", "reasons": [{"decision": "dz", "present": false}]},
            "y": {"status": "blocked", "reasons": [{"decision": "dy", "present": false}]},
        });
        assert_eq!(
            section(Some(&actions), Some(&outcomes), at("c2").as_ref()),
            Some(json!({
                "authority": [{"action": "a", "capability": "m", "scope": {"cluster": "x"},
                    "current": "require_approval", "case_revision": "c2",
                    "options": ["granted", "denied"]}],
                "decisions": [
                    {"outcome": "y", "decision": "dy", "case_revision": "c2", "options": ["taken"]},
                    {"outcome": "z", "decision": "dz", "case_revision": "c2", "options": ["taken"]},
                ],
            }))
        );
        assert_eq!(
            section(None, Some(&outcomes), None),
            Some(json!({"decisions": [
                {"outcome": "y", "decision": "dy", "options": []},
                {"outcome": "z", "decision": "dz", "options": []},
            ]}))
        );
    }
}
