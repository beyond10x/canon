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
//! An empty list (`[]`) decides nothing; an empty document is not a list. A capability no action requires may be decided; it changes no
//! action. Refused, in this order: text that is not such a list, or an entry with another key or
//! another decision, as `malformed-input` (`` `--authority` is not a canon-authority/1 document:
//! <why> ``); then, entry by entry in the order given, a capability that is not an identifier as
//! `invalid-identifier`, and a capability decided a second time as `duplicate-identifier`
//! (`` `--authority` decides capability `<c>` more than once ``).

use std::collections::BTreeMap;

use serde::Deserialize;

use super::read::{malformed, yaml};
use super::{Refusal, identifier};
use crate::model::{CapabilityId, one_line};

/// How the input is named in a refusal: the flag that gives it.
const INPUT: &str = "`--authority`";

/// One authority decision on one capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Grant {
    Granted,
    Denied,
}

impl Grant {
    /// The decision as `canon-authority/1` and `canon-decision/1` write it.
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Grant::Granted => "granted",
            Grant::Denied => "denied",
        }
    }
}

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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    capability: CapabilityId,
    decision: Grant,
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
    let entries: Vec<Entry> =
        serde_yaml_ng::from_value(value).map_err(|error| malformed(&what, error))?;
    let mut decisions = BTreeMap::new();
    for entry in entries {
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
