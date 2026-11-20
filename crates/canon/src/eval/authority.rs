//! The authority decisions an evaluation reads (`--authority`, `canon-authority/1`). Canon grants
//! nothing and resolves no identity: whoever runs the evaluation decides which capabilities are
//! granted, at which scope, and passes those decisions in (design § 33).
//!
//! # The `canon-authority/1` input
//!
//! A YAML (or JSON) list of decisions, each a capability, the scope it covers and the decision:
//!
//! ```yaml
//! - capability: metrics.read
//!   scope: {cluster: prod-eu}  # optional: absent, the decision covers every scope
//!   decision: granted          # or require_approval, or denied
//! ```
//!
//! The entry type is [`crate::model::AuthorityDecision`]. An empty list (`[]`) decides nothing; an
//! empty document is not a list. A capability no action requires may be decided; it changes no
//! action. One capability may be decided at several scopes; two decisions on it with equal scopes
//! (the same keys with the same values, in any order) are a duplicate. Refused, in this order: text
//! that is not such a list (a scope that names a key twice included), or an entry with another
//! key, another decision or a missing key, as `malformed-input` (`` `--authority` is not a
//! canon-authority/1 document: entry <n>: <why> ``, counting entries from 1, where a missing key is
//! named: `` missing field `capability` ``); then, entry by entry in the order given, a capability
//! that is not an identifier as `invalid-identifier`, then a scope key or value that is not one,
//! in the order written, as `invalid-identifier`, then a capability decided a second time at an
//! equal scope as `duplicate-identifier` (`` `--authority` decides capability `<c>` more than once
//! ``, or `` … capability `<c>` at scope {<key>: <value>, …} more than once `` when the scope is
//! not empty, its keys in code-point order).
//!
//! [`Authority::decide`] decides one requirement from these decisions.

use std::collections::BTreeMap;

use super::read::{entries, malformed, yaml};
use super::{Refusal, identifier};
use crate::ir::Scope;
use crate::model::{AuthorityDecision, CapabilityId, Grant, one_line};

/// How the input is named in a refusal: the flag that gives it.
const INPUT: &str = "`--authority`";

/// The authority decisions, read from the text given as `--authority`, keyed by capability and
/// then by scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Authority {
    decisions: BTreeMap<(CapabilityId, Scope), Grant>,
}

impl Authority {
    /// The decision that applies to a requirement of `capability` at `scope` (empty: a requirement
    /// without a scope): of the decisions on `capability` that cover it ([`covers`]), the one
    /// [`Grant::precedence`] ranks highest, so `denied` over `require_approval` over `granted`.
    /// `None` when none covers it: the requirement is not granted (default deny).
    pub(super) fn decide(&self, capability: &CapabilityId, scope: &Scope) -> Option<Grant> {
        self.decisions
            .iter()
            .filter(|((decided, covered), _)| decided == capability && covers(covered, scope))
            .map(|(_, grant)| *grant)
            .max_by_key(|grant| grant.precedence())
    }

    /// Every decision as given, `(capability, scope, "granted" | "require_approval" | "denied")`,
    /// in capability order and then scope order (none first): what the explanation records
    /// (`crate::explain`), so it and the evaluator read one value.
    pub(crate) fn decisions(&self) -> impl Iterator<Item = (&CapabilityId, &Scope, &'static str)> {
        self.decisions
            .iter()
            .map(|((capability, scope), grant)| (capability, scope, grant.as_str()))
    }
}

/// Whether a decision at scope `decision` covers a requirement at scope `requirement`: the
/// requirement gives every key the decision gives a value for that same value. A key the decision
/// leaves out covers every value of it, so a decision without a scope covers every requirement;
/// coverage never widens inwards, so a decision naming a key the requirement leaves out covers
/// nothing of it.
pub(crate) fn covers(decision: &Scope, requirement: &Scope) -> bool {
    decision
        .iter()
        .all(|(key, value)| requirement.get(key) == Some(value))
}

/// A scope as a refusal names it, `{<key>: <value>, …}` in key order, every key and value on one
/// line.
pub(crate) fn shown(scope: &Scope) -> String {
    let members: Vec<String> = scope
        .iter()
        .map(|(key, value)| format!("{}: {}", one_line(key.as_str()), one_line(value.as_str())))
        .collect();
    format!("{{{}}}", members.join(", "))
}

/// Reads the authority decisions, or `None` when none were given.
pub(super) fn read(text: Option<&str>) -> Result<Option<Authority>, Refusal> {
    let Some(text) = text else {
        return Ok(None);
    };
    let what = format!("{INPUT} is not a canon-authority/1 document");
    let value = yaml(text, &what)?;
    // An empty document reads as null, which would otherwise read as an empty list.
    if !value.is_sequence() {
        return Err(malformed(&what, "expected a list of decisions"));
    }
    let mut decisions = BTreeMap::new();
    for entry in entries::<AuthorityDecision>(value, &what)? {
        identifier("authority capability", entry.capability.as_str())?;
        for (key, value) in entry.scope.iter() {
            identifier("authority scope key", key.as_str())?;
            identifier("authority scope value", value.as_str())?;
        }
        // The YAML reader refuses a key written twice in one mapping, so each key is here once.
        let scope: Scope = entry
            .scope
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        let key = (entry.capability, scope);
        if decisions.contains_key(&key) {
            let (capability, scope) = key;
            let at_scope = if scope.is_empty() {
                String::new()
            } else {
                format!(" at scope {}", shown(&scope))
            };
            return Err(Refusal::new(
                "duplicate-identifier",
                format!(
                    "{INPUT} decides capability `{}`{at_scope} more than once",
                    one_line(capability.as_str())
                ),
            ));
        }
        decisions.insert(key, entry.decision);
    }
    Ok(Some(Authority { decisions }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ScopeKey, ScopeValue};

    fn refusal(text: &str) -> Refusal {
        read(Some(text)).expect_err(text)
    }

    fn scope(members: &[(&str, &str)]) -> Scope {
        members
            .iter()
            .map(|(key, value)| (ScopeKey::new(*key), ScopeValue::new(*value)))
            .collect()
    }

    #[test]
    fn decisions_are_read_by_capability() {
        let authority = read(Some(
            "- {capability: a, decision: granted}\n- {capability: b, decision: denied}\n",
        ))
        .expect("reads")
        .expect("given");
        let none = Scope::new();
        assert_eq!(authority.decide(&"a".into(), &none), Some(Grant::Granted));
        assert_eq!(authority.decide(&"b".into(), &none), Some(Grant::Denied));
        assert_eq!(authority.decide(&"c".into(), &none), None);
        assert_eq!(read(None), Ok(None));
        assert_eq!(
            read(Some("[]")),
            Ok(Some(Authority {
                decisions: BTreeMap::new()
            }))
        );
    }

    /// A decision covers a requirement when the requirement gives each key the decision names the
    /// same value; a key the decision leaves out covers every value; never inwards.
    #[test]
    fn a_decision_covers_from_the_wildcard_outwards_never_inwards() {
        let exact = scope(&[("cluster", "prod-eu"), ("namespace", "foo")]);
        let cluster = scope(&[("cluster", "prod-eu")]);
        let other = scope(&[("cluster", "prod-us")]);
        let namespace = scope(&[("namespace", "foo")]);
        let none = Scope::new();
        for decision in [&none, &cluster, &namespace, &exact] {
            assert!(
                covers(decision, &exact),
                "{decision:?} covers the exact scope"
            );
        }
        assert!(!covers(&other, &exact));
        assert!(!covers(&exact, &cluster), "never inwards");
        assert!(!covers(&cluster, &none), "never inwards to no scope");
        assert!(covers(&none, &none));
        assert!(!covers(&scope(&[("region", "eu")]), &exact));
    }

    /// Of the decisions covering a requirement, denied wins over require_approval over granted;
    /// a decision that does not cover it takes no part.
    #[test]
    fn denied_wins_over_require_approval_over_granted() {
        let exact = scope(&[("cluster", "prod-eu"), ("namespace", "foo")]);
        let decide = |text: &str| {
            read(Some(text))
                .expect("reads")
                .expect("given")
                .decide(&"m".into(), &exact)
        };
        assert_eq!(
            decide(
                "- {capability: m, scope: {cluster: prod-eu}, decision: granted}\n\
                 - {capability: m, scope: {cluster: prod-eu, namespace: foo}, decision: require_approval}\n"
            ),
            Some(Grant::RequireApproval)
        );
        assert_eq!(
            decide(
                "- {capability: m, decision: denied}\n\
                 - {capability: m, scope: {cluster: prod-eu, namespace: foo}, decision: granted}\n\
                 - {capability: m, scope: {cluster: prod-eu}, decision: require_approval}\n"
            ),
            Some(Grant::Denied)
        );
        assert_eq!(
            decide(
                "- {capability: m, scope: {cluster: prod-us}, decision: denied}\n\
                 - {capability: m, scope: {namespace: foo}, decision: granted}\n"
            ),
            Some(Grant::Granted)
        );
        assert_eq!(
            decide("- {capability: m, scope: {namespace: bar}, decision: granted}\n"),
            None
        );
        assert_eq!(
            decide("- {capability: n, decision: granted}\n"),
            None,
            "another capability decides nothing of it"
        );
    }

    #[test]
    fn input_that_is_not_a_list_of_decisions_is_refused() {
        for text in [
            "any text: not a list\n",
            "[{capability: a}]",
            "[{capability: a, decision: maybe}]",
            "[{capability: a, decision: granted, by: someone}]",
            "[{capability: ~, decision: granted}]",
            "[{capability: a, decision: requireapproval}]",
            "[{capability: a, scope: ~, decision: granted}]",
            "[{capability: a, scope: [cluster], decision: granted}]",
            "[{capability: a, scope: {cluster: ~}, decision: granted}]",
            "[.nan]",
            // Red before the list check: both read as an empty list.
            "",
            "~",
        ] {
            let refused = refusal(text);
            assert_eq!(refused.code(), "malformed-input", "{text:?}: {refused}");
            assert!(
                refused
                    .to_string()
                    .starts_with("`--authority` is not a canon-authority/1 document: "),
                "{text:?}: {refused}"
            );
        }
    }

    /// A key left out of an entry is refused naming the entry, counted from 1, and the key; so is
    /// one left out of an explicit decision (`decisions.rs` reads its list the same way).
    #[test]
    fn a_left_out_key_is_refused_naming_the_entry_and_the_key() {
        let refused = refusal("- {capability: a, decision: granted}\n- {decision: denied}\n");
        assert_eq!(
            refused.to_string(),
            "`--authority` is not a canon-authority/1 document: entry 2: missing field `capability`"
        );
        let refused = refusal("[{capability: a}]");
        assert_eq!(
            refused.to_string(),
            "`--authority` is not a canon-authority/1 document: entry 1: missing field `decision`"
        );
        let refused =
            super::super::decisions::read(Some("- {decision: d, outcome: o, principal: p}\n"))
                .expect_err("refused");
        assert_eq!(
            refused.to_string(),
            "`--decisions` is not a canon-decisions/1 document: entry 1: missing field \
             `case_revision`"
        );
    }

    #[test]
    fn a_capability_is_an_identifier_decided_once() {
        let refused = refusal("[{capability: 'a b', decision: granted}]");
        assert_eq!(refused.code(), "invalid-identifier", "{refused}");
        let refused =
            refusal("[{capability: a, decision: granted}, {capability: a, decision: denied}]");
        assert_eq!(refused.code(), "duplicate-identifier", "{refused}");
        assert_eq!(
            refused.to_string(),
            "`--authority` decides capability `a` more than once"
        );
    }

    /// One capability at several scopes is the normal case; at an equal scope, whatever the order
    /// its keys are written in and whatever the decision, it is a duplicate. An empty scope is no
    /// scope.
    #[test]
    fn a_capability_is_decided_once_at_each_scope() {
        let authority = read(Some(
            "- {capability: m, decision: denied}\n\
             - {capability: m, scope: {cluster: prod-eu}, decision: granted}\n\
             - {capability: m, scope: {cluster: prod-us}, decision: require_approval}\n",
        ))
        .expect("reads")
        .expect("given");
        let listed: Vec<(&str, Scope, &str)> = authority
            .decisions()
            .map(|(capability, scope, decision)| (capability.as_str(), scope.clone(), decision))
            .collect();
        assert_eq!(
            listed,
            [
                ("m", Scope::new(), "denied"),
                ("m", scope(&[("cluster", "prod-eu")]), "granted"),
                ("m", scope(&[("cluster", "prod-us")]), "require_approval"),
            ]
        );
        let refused = refusal(
            "- {capability: m, scope: {namespace: foo, cluster: prod-eu}, decision: granted}\n\
             - {capability: m, scope: {cluster: prod-eu, namespace: foo}, decision: denied}\n",
        );
        assert_eq!(refused.code(), "duplicate-identifier", "{refused}");
        assert_eq!(
            refused.to_string(),
            "`--authority` decides capability `m` at scope {cluster: prod-eu, namespace: foo} \
             more than once"
        );
        let refused = refusal(
            "- {capability: m, decision: granted}\n- {capability: m, scope: {}, decision: denied}\n",
        );
        assert_eq!(
            refused.to_string(),
            "`--authority` decides capability `m` more than once"
        );
    }

    /// A scope's keys and values are identifiers, and a key is named once in an entry: the YAML
    /// reader refuses one named twice before any entry is read.
    #[test]
    fn a_scope_names_each_key_once_with_identifiers() {
        let refused = refusal("[{capability: m, scope: {'a b': x}, decision: granted}]");
        assert_eq!(refused.code(), "invalid-identifier", "{refused}");
        assert_eq!(
            refused.to_string(),
            "authority scope key `a b` is empty or contains whitespace or a control character"
        );
        let refused = refusal("[{capability: m, scope: {cluster: ''}, decision: granted}]");
        assert_eq!(
            refused.to_string(),
            "authority scope value `` is empty or contains whitespace or a control character"
        );
        let refused = refusal(
            "- {capability: m, decision: granted}\n\
             - {capability: m, scope: {cluster: a, cluster: b}, decision: granted}\n",
        );
        assert_eq!(refused.code(), "malformed-input", "{refused}");
        let refused = refused.to_string();
        assert!(
            refused.starts_with("`--authority` is not a canon-authority/1 document: ")
                && refused.contains("duplicate entry with key \"cluster\""),
            "{refused}"
        );
    }
}
