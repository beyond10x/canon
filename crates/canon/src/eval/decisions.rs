//! The explicit decisions an evaluation reads (`--decisions`, `canon-decisions/1`), which outcome
//! requirements may name (story:decision-outcomes). Canon decides nothing and resolves no
//! identity: whoever runs the evaluation passes in the decisions that were taken.
//!
//! # The `canon-decisions/1` input
//!
//! A YAML (or JSON) list of decisions, each naming the decision, the outcome it was taken for, who
//! took it and the case revision it was taken at:
//!
//! ```yaml
//! - decision: explicitly_inconclusive
//!   outcome: inconclusive
//!   principal: lead-investigator
//!   case_revision: c2
//! ```
//!
//! An empty list (`[]`) decides nothing; an empty document is not a list. Refused, in this order:
//! text that is not such a list, or an entry with another key or a missing one, as
//! `malformed-input` (`` `--decisions` is not a canon-decisions/1 document: <why> ``); then, entry
//! by entry in the order given, a decision, outcome, principal or case revision that is not an
//! identifier, in that order, as `invalid-identifier`, and an entry that repeats an earlier one
//! exactly (all four keys equal) as `duplicate-identifier`, naming it. The `outcomes` section
//! (`outcomes.rs`), which has the protocol, then refuses a decision for an outcome the protocol
//! does not declare as `undeclared-outcome`, and one whose outcome does not require a decision of
//! its name as `undeclared-decision`.
//!
//! A decision applies to an outcome when the outcome requires a decision of its name, the decision
//! names that outcome, and its case revision is the case snapshot's `revision`. A decision taken
//! at another case revision has been superseded and applies to nothing; so does every decision
//! when the snapshot has no revision. The outcome records every principal whose decision applied,
//! sorted by Unicode code point, so the order the decisions are given in does not change the
//! result and no principal who decided is dropped (design § 37).

use super::read::{malformed, yaml};
use super::{Refusal, identifier};
use crate::model::{DecisionName, ExplicitDecision, OutcomeId, Principal, Revision, one_line};

/// How the input is named in a refusal: the flag that gives it.
const INPUT: &str = "`--decisions`";

/// The explicit decisions, read from the text given as `--decisions`, in the order given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Decisions {
    entries: Vec<ExplicitDecision>,
}

impl Decisions {
    /// Every decision, in the order given.
    pub(super) fn entries(&self) -> &[ExplicitDecision] {
        &self.entries
    }

    /// Every principal who took the decision `name` for `outcome` at the case revision `current`,
    /// one per entry naming all three, sorted by Unicode code point. Empty when none did, and
    /// always with no current revision.
    pub(super) fn decided_by(
        &self,
        name: &DecisionName,
        outcome: &OutcomeId,
        current: Option<&Revision>,
    ) -> Vec<&Principal> {
        let Some(current) = current else {
            return Vec::new();
        };
        let mut principals: Vec<&Principal> = self
            .entries
            .iter()
            .filter(|entry| {
                entry.decision == *name
                    && entry.outcome == *outcome
                    && entry.case_revision == *current
            })
            .map(|entry| &entry.principal)
            .collect();
        principals.sort_by(|left, right| left.as_str().chars().cmp(right.as_str().chars()));
        principals.dedup();
        principals
    }
}

/// Reads the explicit decisions, or `None` when none were given.
pub(super) fn read(text: Option<&str>) -> Result<Option<Decisions>, Refusal> {
    let Some(text) = text else {
        return Ok(None);
    };
    let what = format!("{INPUT} is not a canon-decisions/1 document");
    let value = yaml(text, &what)?;
    // An empty document reads as null, which would otherwise read as an empty list.
    if !value.is_sequence() {
        return Err(malformed(&what, "expected a list of decisions"));
    }
    let entries: Vec<ExplicitDecision> =
        serde_yaml_ng::from_value(value).map_err(|error| malformed(&what, error))?;
    for (at, entry) in entries.iter().enumerate() {
        identifier("decision name", entry.decision.as_str())?;
        identifier("decision outcome", entry.outcome.as_str())?;
        identifier("decision principal", entry.principal.as_str())?;
        identifier("decision case revision", entry.case_revision.as_str())?;
        if entries[..at].contains(entry) {
            return Err(Refusal::new(
                "duplicate-identifier",
                format!(
                    "{INPUT} gives decision `{}` for outcome `{}` by `{}` at case revision `{}` \
                     more than once",
                    one_line(entry.decision.as_str()),
                    one_line(entry.outcome.as_str()),
                    one_line(entry.principal.as_str()),
                    one_line(entry.case_revision.as_str())
                ),
            ));
        }
    }
    Ok(Some(Decisions { entries }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENTRY: &str = "- {decision: explicitly_inconclusive, outcome: inconclusive, principal: lead, case_revision: c2}\n";

    fn refusal(text: &str) -> Refusal {
        read(Some(text)).expect_err(text)
    }

    #[test]
    fn a_decision_applies_only_with_its_name_outcome_and_case_revision() {
        let decisions = read(Some(ENTRY)).expect("reads").expect("given");
        let name = DecisionName::new("explicitly_inconclusive");
        let outcome = OutcomeId::new("inconclusive");
        let c2 = Revision::new("c2");
        let lead = Principal::new("lead");
        assert_eq!(decisions.decided_by(&name, &outcome, Some(&c2)), [&lead]);
        assert!(
            decisions
                .decided_by(&name, &outcome, Some(&Revision::new("c3")))
                .is_empty()
        );
        assert!(decisions.decided_by(&name, &outcome, None).is_empty());
        assert!(
            decisions
                .decided_by(&name, &OutcomeId::new("declined"), Some(&c2))
                .is_empty()
        );
        assert!(
            decisions
                .decided_by(&DecisionName::new("other"), &outcome, Some(&c2))
                .is_empty()
        );
        assert_eq!(read(None), Ok(None));
        assert_eq!(
            read(Some("[]")).expect("reads").expect("given").entries(),
            []
        );
    }

    /// Several principals at the current revision: every one is recorded, sorted by code point,
    /// whatever order they are given in; one at a superseded revision is not counted.
    #[test]
    fn every_principal_whose_decision_applies_is_recorded_in_code_point_order() {
        let name = DecisionName::new("d");
        let outcome = OutcomeId::new("o");
        let c2 = Revision::new("c2");
        for text in [
            "[{decision: d, outcome: o, principal: zed, case_revision: c2},\n \
              {decision: d, outcome: o, principal: amy, case_revision: c2},\n \
              {decision: d, outcome: o, principal: abe, case_revision: c1}]",
            "[{decision: d, outcome: o, principal: abe, case_revision: c1},\n \
              {decision: d, outcome: o, principal: amy, case_revision: c2},\n \
              {decision: d, outcome: o, principal: zed, case_revision: c2}]",
        ] {
            let decisions = read(Some(text)).expect("reads").expect("given");
            assert_eq!(
                decisions.decided_by(&name, &outcome, Some(&c2)),
                [&Principal::new("amy"), &Principal::new("zed")],
                "{text}"
            );
        }
    }

    #[test]
    fn an_entry_given_twice_is_refused_naming_it() {
        let refused = refusal(
            "[{decision: d, outcome: o, principal: p, case_revision: c2},\n \
              {decision: d, outcome: o, principal: q, case_revision: c2},\n \
              {decision: d, outcome: o, principal: p, case_revision: c2}]",
        );
        assert_eq!(refused.code(), "duplicate-identifier", "{refused}");
        assert_eq!(
            refused.to_string(),
            "`--decisions` gives decision `d` for outcome `o` by `p` at case revision `c2` more than once"
        );
    }

    #[test]
    fn input_that_is_not_a_list_of_decisions_is_refused() {
        for text in [
            "format: canon-decisions/1\n",
            "[{decision: d, outcome: o, principal: p}]",
            "[{decision: d, outcome: o, principal: p, case_revision: c, by: someone}]",
            "[{decision: ~, outcome: o, principal: p, case_revision: c}]",
            "[{decision: d, outcome: o, principal: p, case_revision: 2}]",
            "[1]",
            "",
            "~",
        ] {
            let refused = refusal(text);
            assert_eq!(refused.code(), "malformed-input", "{text:?}: {refused}");
            assert!(
                refused
                    .to_string()
                    .starts_with("`--decisions` is not a canon-decisions/1 document: "),
                "{text:?}: {refused}"
            );
        }
    }

    #[test]
    fn every_name_in_a_decision_is_an_identifier() {
        for (text, named) in [
            (
                "[{decision: 'a b', outcome: o, principal: p, case_revision: c}]",
                "decision name",
            ),
            (
                "[{decision: d, outcome: 'a b', principal: p, case_revision: c}]",
                "decision outcome",
            ),
            (
                "[{decision: d, outcome: o, principal: 'a b', case_revision: c}]",
                "decision principal",
            ),
            (
                "[{decision: d, outcome: o, principal: p, case_revision: 'a b'}]",
                "decision case revision",
            ),
        ] {
            let refused = refusal(text);
            assert_eq!(refused.code(), "invalid-identifier", "{text}: {refused}");
            assert!(refused.to_string().starts_with(named), "{text}: {refused}");
        }
    }
}
