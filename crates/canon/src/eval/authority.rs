//! The authority decisions an evaluation reads (`--authority`, `canon-authority/1`). Canon grants
//! nothing and resolves no identity: whoever runs the evaluation decides which capabilities are
//! granted, and passes those decisions in (design § 33).
//!
//! # The `canon-authority/1` input
//!
//! A YAML (or JSON) list of decisions, each a capability and whether it is granted:
//!
//! ```yaml
//! - capability: finding.publish
//!   decision: granted        # or denied
//! ```
//!
//! The entry type is [`crate::model::AuthorityDecision`]. An empty list (`[]`) decides nothing; an
//! empty document is not a list. A capability no action requires may be decided; it changes no
//! action. Refused, in this order: text that is not such a list, or an entry with another key,
//! another decision or a missing key, as `malformed-input` (`` `--authority` is not a
//! canon-authority/1 document: entry <n>: <why> ``, counting entries from 1, where a missing key is
//! named: `` missing field `capability` ``); then, entry by entry in the order given, a capability
//! that is not an identifier as `invalid-identifier`, and a capability decided a second time as
//! `duplicate-identifier` (`` `--authority` decides capability `<c>` more than once ``).

use std::collections::BTreeMap;

use super::read::{entries, malformed, yaml};
use super::{Refusal, identifier};
use crate::model::{AuthorityDecision, CapabilityId, Grant, one_line};

/// How the input is named in a refusal: the flag that gives it.
const INPUT: &str = "`--authority`";

/// The authority decisions, read from the text given as `--authority`, keyed by capability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Authority {
    decisions: BTreeMap<CapabilityId, Grant>,
}

impl Authority {
    /// The decision on `capability`, or `None` when the authority does not decide it.
    pub(super) fn decision(&self, capability: &CapabilityId) -> Option<Grant> {
        self.decisions.get(capability).copied()
    }

    /// Every decision as it applies, `(capability, "granted" | "denied")`, in capability order:
    /// what the explanation records (`crate::explain`), so it and the evaluator read one value.
    pub(crate) fn decisions(&self) -> impl Iterator<Item = (&CapabilityId, &'static str)> {
        self.decisions
            .iter()
            .map(|(capability, grant)| (capability, grant.as_str()))
    }
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
        if decisions.contains_key(&entry.capability) {
            return Err(Refusal::new(
                "duplicate-identifier",
                format!(
                    "{INPUT} decides capability `{}` more than once",
                    one_line(entry.capability.as_str())
                ),
            ));
        }
        decisions.insert(entry.capability, entry.decision);
    }
    Ok(Some(Authority { decisions }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refusal(text: &str) -> Refusal {
        read(Some(text)).expect_err(text)
    }

    #[test]
    fn decisions_are_read_by_capability() {
        let authority = read(Some(
            "- {capability: a, decision: granted}\n- {capability: b, decision: denied}\n",
        ))
        .expect("reads")
        .expect("given");
        assert_eq!(authority.decision(&"a".into()), Some(Grant::Granted));
        assert_eq!(authority.decision(&"b".into()), Some(Grant::Denied));
        assert_eq!(authority.decision(&"c".into()), None);
        assert_eq!(read(None), Ok(None));
        assert_eq!(
            read(Some("[]")),
            Ok(Some(Authority {
                decisions: BTreeMap::new()
            }))
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
}
