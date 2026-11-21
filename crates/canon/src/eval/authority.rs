//! The authority decisions an evaluation reads (`--authority`, `canon-authority/1`). Canon grants
//! nothing and resolves no identity: whoever runs the evaluation decides which capabilities are
//! granted, at which scope, and passes those decisions in (design § 33).
//!
//! # The `canon-authority/1` input
//!
//! A YAML (or JSON) list of decisions, each a capability, the scope it covers, the decision and,
//! optionally, who took it, the case revision it was taken at and the grounds it rested on:
//!
//! ```yaml
//! - capability: metrics.read
//!   scope: {cluster: prod-eu}  # optional: absent, the decision covers every scope
//!   decision: granted          # or require_approval, or denied
//!   principal: ops-lead        # optional: who took it, recorded as given
//!   case_revision: c2          # optional: absent, the decision applies at every case revision
//!   ground_revisions: {spec: r2}  # optional: the artifact revisions it rested on
//!   ground_evidence: [e1]         # optional: the evidence records it rested on
//! ```
//!
//! The entry type is [`crate::model::AuthorityDecision`]. An empty list (`[]`) decides nothing; an
//! empty document is not a list. A capability no action requires may be decided; it changes no
//! action. One capability may be decided at several scopes and at several case revisions; two
//! entries equal in capability, scope (the same keys with the same values, in any order), case
//! revision and principal are a duplicate, whatever their decisions and grounds. Refused, in this
//! order: text that is not such a list (a scope or `ground_revisions` that names a key twice
//! included), or an entry with another key, another decision or a missing key, as
//! `malformed-input` (`` `--authority` is not a canon-authority/1 document: entry <n>: <why> ``,
//! counting entries from 1, where a missing key is named: `` missing field `capability` ``); then,
//! entry by entry in the order given, a capability that is not an identifier as
//! `invalid-identifier`, then a scope key or value that is not one, in the order written, then a
//! principal, then a case revision, then a ground artifact or its revision, in the order written,
//! then a ground evidence id, each as `invalid-identifier`, then an entry equal to an earlier one
//! as `duplicate-identifier` (`` `--authority` decides capability `<c>` more than once ``, with ``
//! at scope {<key>: <value>, …} `` after the capability when the scope is not empty, its keys in
//! code-point order, then `` at case revision `<r>` `` and `` by `<p>` `` when those are given).
//! A ground artifact the protocol does not declare is refused later, as `undeclared-artifact`
//! (`grounds.rs`).
//!
//! An entry with a case revision is bound to it (story:authority-revision-binding): whatever its
//! decision, it applies only while the case snapshot's `revision` equals it, and to nothing when
//! the snapshot has no revision, the rule an explicit decision follows (`decisions.rs`). An entry
//! without one applies at every case revision. An entry with grounds applies only while every
//! ground holds (story:decision-grounds, `grounds.rs`), whatever its decision, so a denial whose
//! ground moved lapses too. The principal is recorded as given and matched against nothing: Canon
//! resolves no identity (design § 23), so it does not change admissibility.
//! [`Authority::applying_at`] keeps the entries that apply, once, and records the failed grounds
//! of each entry set aside by its grounds alone; [`Authority::decide`] then decides one requirement
//! from the entries that apply alone, so an entry that does not apply takes part in neither
//! coverage nor precedence, and [`Authority::stale_grounds`] names the failed grounds of the
//! set-aside entries that would have covered it.

use std::collections::{BTreeMap, BTreeSet};

use super::grounds::{Failed, Grounds, Standing};
use super::read::{entries, malformed, yaml};
use super::{Refusal, identifier};
use crate::ir::Scope;
use crate::model::{AuthorityDecision, CapabilityId, Grant, Principal, Revision, one_line};

/// How the input is named in a refusal: the flag that gives it.
const INPUT: &str = "`--authority`";

/// What tells two entries apart: capability, scope, case revision (none first) and principal
/// (none first), ordered in that order. Grounds are not part of it.
type Key = (CapabilityId, Scope, Option<Revision>, Option<Principal>);

/// What an entry decides, and what it rested on.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    grant: Grant,
    grounds: Grounds,
}

/// The authority decisions, read from the text given as `--authority`, keyed by capability, then
/// by scope, then by case revision and then by principal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Authority {
    decisions: BTreeMap<Key, Entry>,
    /// The entries [`Authority::applying_at`] set aside by their grounds alone, each with its
    /// capability, its scope and the grounds that failed. Empty in the decisions as read.
    lapsed: Vec<(CapabilityId, Scope, BTreeSet<Failed>)>,
}

/// One decision as given, what the explanation records.
pub(crate) struct Given<'a> {
    pub(crate) capability: &'a CapabilityId,
    pub(crate) scope: &'a Scope,
    pub(crate) case_revision: Option<&'a Revision>,
    pub(crate) principal: Option<&'a Principal>,
    /// `"granted"`, `"require_approval"` or `"denied"`.
    pub(crate) decision: &'static str,
    /// The grounds it rested on.
    pub(crate) grounds: &'a Grounds,
}

impl Authority {
    /// The decisions that apply at the case revision `current` (the case snapshot's `revision`)
    /// with the grounds as `standing` has them: each one whose grounds all hold that is unbound or
    /// bound to `current`. With no current revision, only unbound ones apply. An entry at its case
    /// revision (or with none) whose grounds do not all hold is kept aside with the grounds that
    /// failed, for [`Authority::stale_grounds`]; an entry bound to another case revision is
    /// dropped, its grounds unread.
    pub(super) fn applying_at(&self, current: Option<&Revision>, standing: &Standing) -> Authority {
        let mut decisions = BTreeMap::new();
        let mut lapsed = Vec::new();
        for (key, entry) in &self.decisions {
            let (capability, scope, bound, _) = key;
            if bound.is_some() && bound.as_ref() != current {
                continue;
            }
            let failed = standing.failed(&entry.grounds);
            if failed.is_empty() {
                decisions.insert(key.clone(), entry.clone());
            } else {
                lapsed.push((capability.clone(), scope.clone(), failed));
            }
        }
        Authority { decisions, lapsed }
    }

    /// The decision that applies to a requirement of `capability` at `scope` (empty: a requirement
    /// without a scope): of the decisions on `capability` that cover it ([`covers`]), the one
    /// [`Grant::precedence`] ranks highest, so `denied` over `require_approval` over `granted`.
    /// `None` when none covers it: the requirement is not granted (default deny). It reads every
    /// decision it holds, so the evaluator asks the decisions [`Authority::applying_at`] keeps.
    pub(super) fn decide(&self, capability: &CapabilityId, scope: &Scope) -> Option<Grant> {
        self.decisions
            .iter()
            .filter(|((decided, covered, _, _), _)| decided == capability && covers(covered, scope))
            .map(|(_, entry)| entry.grant)
            .max_by_key(|grant| grant.precedence())
    }

    /// The failed grounds of every entry [`Authority::applying_at`] set aside by its grounds that
    /// is on `capability` and covers `scope`, each once, in `stale_grounds` order. Empty for the
    /// decisions as read.
    pub(super) fn stale_grounds(
        &self,
        capability: &CapabilityId,
        scope: &Scope,
    ) -> BTreeSet<Failed> {
        self.lapsed
            .iter()
            .filter(|(decided, covered, _)| decided == capability && covers(covered, scope))
            .flat_map(|(_, _, failed)| failed.iter().cloned())
            .collect()
    }

    /// Every decision as given, in capability order, then scope order (none first), then case
    /// revision order (none first), then principal order (none first): what the explanation
    /// records (`crate::explain`), so it and the evaluator read one value.
    pub(crate) fn decisions(&self) -> impl Iterator<Item = Given<'_>> {
        self.decisions.iter().map(
            |((capability, scope, case_revision, principal), entry)| Given {
                capability,
                scope,
                case_revision: case_revision.as_ref(),
                principal: principal.as_ref(),
                decision: entry.grant.as_str(),
                grounds: &entry.grounds,
            },
        )
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
        if let Some(principal) = &entry.principal {
            identifier("authority principal", principal.as_str())?;
        }
        if let Some(case_revision) = &entry.case_revision {
            identifier("authority case revision", case_revision.as_str())?;
        }
        let grounds = Grounds::read("authority", &entry.ground_revisions, &entry.ground_evidence)?;
        // The YAML reader refuses a key written twice in one mapping, so each key is here once.
        let scope: Scope = entry
            .scope
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        let key = (
            entry.capability,
            scope,
            entry.case_revision,
            entry.principal,
        );
        if decisions.contains_key(&key) {
            let (capability, scope, case_revision, principal) = key;
            let mut at = String::new();
            if !scope.is_empty() {
                at.push_str(&format!(" at scope {}", shown(&scope)));
            }
            if let Some(case_revision) = case_revision {
                at.push_str(&format!(
                    " at case revision `{}`",
                    one_line(case_revision.as_str())
                ));
            }
            if let Some(principal) = principal {
                at.push_str(&format!(" by `{}`", one_line(principal.as_str())));
            }
            return Err(Refusal::new(
                "duplicate-identifier",
                format!(
                    "{INPUT} decides capability `{}`{at} more than once",
                    one_line(capability.as_str())
                ),
            ));
        }
        let entry = Entry {
            grant: entry.decision,
            grounds,
        };
        decisions.insert(key, entry);
    }
    Ok(Some(Authority {
        decisions,
        lapsed: Vec::new(),
    }))
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
                decisions: BTreeMap::new(),
                lapsed: Vec::new(),
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
            .map(|given| {
                (
                    given.capability.as_str(),
                    given.scope.clone(),
                    given.decision,
                )
            })
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

    /// A bound decision applies only at its case revision, whatever it decides, and none applies
    /// without a current revision; an unbound one applies at every revision. Only those that apply
    /// take part in precedence: a stale denial no longer outranks a grant.
    #[test]
    fn a_bound_decision_applies_only_at_its_case_revision() {
        let authority = read(Some(
            "- {capability: m, decision: granted}\n\
             - {capability: m, decision: denied, case_revision: c1}\n\
             - {capability: n, decision: granted, case_revision: c2, principal: p}\n",
        ))
        .expect("reads")
        .expect("given");
        let none = Scope::new();
        let standing = Standing::default();
        let at = |revision: Option<&str>, capability: &str| {
            let revision = revision.map(Revision::new);
            authority
                .applying_at(revision.as_ref(), &standing)
                .decide(&capability.into(), &none)
        };
        assert_eq!(at(Some("c1"), "m"), Some(Grant::Denied));
        assert_eq!(
            at(Some("c2"), "m"),
            Some(Grant::Granted),
            "the c1 denial lapsed"
        );
        assert_eq!(at(None, "m"), Some(Grant::Granted));
        assert_eq!(at(Some("c2"), "n"), Some(Grant::Granted));
        assert_eq!(at(Some("c1"), "n"), None);
        assert_eq!(
            at(None, "n"),
            None,
            "no current revision: no bound decision"
        );
        assert_eq!(
            authority.applying_at(None, &standing).decisions().count(),
            1,
            "only the unbound decision applies without a revision"
        );
        assert_eq!(authority.decisions().count(), 3, "every decision as given");
    }

    /// Entries are told apart by capability, scope, case revision and principal, and listed in
    /// that order, none first in each; the decision does not tell them apart.
    #[test]
    fn an_entry_is_decided_once_at_each_case_revision_and_principal() {
        let authority = read(Some(
            "- {capability: m, decision: granted, case_revision: c2, principal: q}\n\
             - {capability: m, decision: granted, case_revision: c2, principal: p}\n\
             - {capability: m, decision: granted, case_revision: c2}\n\
             - {capability: m, decision: denied, case_revision: c1}\n\
             - {capability: m, decision: granted, principal: p}\n\
             - {capability: m, decision: granted}\n",
        ))
        .expect("reads")
        .expect("given");
        let listed: Vec<(Option<&str>, Option<&str>)> = authority
            .decisions()
            .map(|given| {
                (
                    given.case_revision.map(Revision::as_str),
                    given.principal.map(Principal::as_str),
                )
            })
            .collect();
        assert_eq!(
            listed,
            [
                (None, None),
                (None, Some("p")),
                (Some("c1"), None),
                (Some("c2"), None),
                (Some("c2"), Some("p")),
                (Some("c2"), Some("q")),
            ]
        );
        let refused = refusal(
            "- {capability: m, scope: {cluster: a}, decision: granted, case_revision: c2, principal: p}\n\
             - {capability: m, scope: {cluster: a}, decision: denied, principal: p, case_revision: c2}\n",
        );
        assert_eq!(refused.code(), "duplicate-identifier", "{refused}");
        assert_eq!(
            refused.to_string(),
            "`--authority` decides capability `m` at scope {cluster: a} at case revision `c2` by \
             `p` more than once"
        );
        let refused = refusal(
            "- {capability: m, decision: granted, case_revision: c2}\n\
             - {capability: m, decision: denied, case_revision: c2}\n",
        );
        assert_eq!(
            refused.to_string(),
            "`--authority` decides capability `m` at case revision `c2` more than once"
        );
        let refused = refusal(
            "- {capability: m, decision: granted, principal: p}\n\
             - {capability: m, decision: granted, principal: p}\n",
        );
        assert_eq!(
            refused.to_string(),
            "`--authority` decides capability `m` by `p` more than once"
        );
    }

    /// A principal and a case revision are identifiers, checked after the scope, the principal
    /// first; written as null, each is malformed rather than absent.
    #[test]
    fn a_principal_and_a_case_revision_are_identifiers() {
        let refused = refusal("[{capability: m, decision: granted, principal: 'a b'}]");
        assert_eq!(refused.code(), "invalid-identifier", "{refused}");
        assert_eq!(
            refused.to_string(),
            "authority principal `a b` is empty or contains whitespace or a control character"
        );
        let refused = refusal("[{capability: m, decision: granted, case_revision: ''}]");
        assert_eq!(
            refused.to_string(),
            "authority case revision `` is empty or contains whitespace or a control character"
        );
        let refused = refusal(
            "[{capability: m, scope: {cluster: ''}, decision: granted, principal: '', case_revision: ''}]",
        );
        assert!(
            refused.to_string().starts_with("authority scope value"),
            "{refused}"
        );
        let refused =
            refusal("[{capability: m, decision: granted, principal: '', case_revision: ''}]");
        assert!(
            refused.to_string().starts_with("authority principal"),
            "{refused}"
        );
        for text in [
            "[{capability: m, decision: granted, principal: ~}]",
            "[{capability: m, decision: granted, case_revision: ~}]",
        ] {
            assert_eq!(refusal(text).code(), "malformed-input", "{text}");
        }
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
