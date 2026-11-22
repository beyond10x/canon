//! What an outcome requires: a predicate over claim values and evidence, or an explicit decision
//! (design § 12, story:decision-outcomes).
//!
//! Written in source as a predicate, or as a map with the one key `decision` naming the decision:
//!
//! ```yaml
//! outcomes:
//!   inconclusive:
//!     requires:
//!       decision: explicitly_inconclusive
//! ```
//!
//! An explicit decision may carry a condition, `when`, a predicate: the decision counts toward the
//! outcome only while `when` is `true` (story:decision-condition).
//!
//! ```yaml
//! outcomes:
//!   withdrawn:
//!     requires:
//!       decision: withdraw
//!       when: {claim: publication.refused}
//! ```
//!
//! A decision is not a predicate form: it can be written only as the whole of an outcome's
//! `requires`, never inside `all`, `any` or `not`, and never in a claim, obligation or action.

use serde::Deserialize;

use super::ids::{ClaimId, DecisionName, InputId, InputValue};
use super::predicate::{
    ChildOutcomes, EvidenceAfter, EvidenceMatch, EvidenceOrder, EvidenceSince, FirstResult,
    Predicate, Truth, WrittenPredicate,
};

/// What an outcome requires.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "WrittenRequirement")]
pub enum OutcomeRequirement {
    /// The outcome is legitimate when the predicate is `true`.
    Predicate(Predicate),
    /// The outcome is legitimate only with an explicit decision of this name, for the outcome,
    /// taken at the case snapshot's current revision, and, when it has a condition, only while
    /// that is `true`.
    Decision(DecisionRequirement),
}

/// An explicit decision an outcome requires, with its optional condition
/// (story:decision-condition).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionRequirement {
    /// The decision's name.
    pub decision: DecisionName,
    /// The decision counts only while this is `true`; absent, it counts whenever it applies.
    pub when: Option<Predicate>,
}

impl OutcomeRequirement {
    /// The predicate, when the requirement is one.
    pub fn predicate(&self) -> Option<&Predicate> {
        match self {
            OutcomeRequirement::Predicate(predicate) => Some(predicate),
            OutcomeRequirement::Decision(_) => None,
        }
    }

    /// The decision's name, when the requirement is one.
    pub fn decision(&self) -> Option<&DecisionName> {
        match self {
            OutcomeRequirement::Predicate(_) => None,
            OutcomeRequirement::Decision(requirement) => Some(&requirement.decision),
        }
    }

    /// The decision's condition, when the requirement is a decision with one.
    pub fn when(&self) -> Option<&Predicate> {
        match self {
            OutcomeRequirement::Predicate(_) => None,
            OutcomeRequirement::Decision(requirement) => requirement.when.as_ref(),
        }
    }
}

impl From<Predicate> for OutcomeRequirement {
    fn from(predicate: Predicate) -> Self {
        OutcomeRequirement::Predicate(predicate)
    }
}

impl PartialEq<Predicate> for OutcomeRequirement {
    fn eq(&self, other: &Predicate) -> bool {
        self.predicate() == Some(other)
    }
}

/// An outcome's `requires` as written: the predicate forms, and `decision`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenRequirement {
    #[serde(default, deserialize_with = "super::present::form")]
    decision: Option<DecisionName>,
    #[serde(default, deserialize_with = "super::present::form")]
    all: Option<Vec<Predicate>>,
    #[serde(default, deserialize_with = "super::present::form")]
    any: Option<Vec<Predicate>>,
    #[serde(default, deserialize_with = "super::present::form")]
    not: Option<Box<Predicate>>,
    #[serde(default, deserialize_with = "super::present::form")]
    evidence: Option<EvidenceMatch>,
    #[serde(default, deserialize_with = "super::present::form")]
    claim: Option<ClaimId>,
    #[serde(default, deserialize_with = "super::present::optional")]
    is: Option<Truth>,
    #[serde(default, deserialize_with = "super::present::form")]
    input: Option<InputId>,
    // Not optional when its `input` is written: a null on it says it needs a value.
    #[serde(default, deserialize_with = "super::present::form")]
    r#in: Option<Vec<InputValue>>,
    #[serde(default, deserialize_with = "super::present::form")]
    first: Option<FirstResult>,
    #[serde(default, deserialize_with = "super::present::form")]
    before: Option<EvidenceOrder>,
    #[serde(default, deserialize_with = "super::present::form")]
    after: Option<EvidenceAfter>,
    #[serde(default, deserialize_with = "super::present::form")]
    since: Option<EvidenceSince>,
    #[serde(default, deserialize_with = "super::present::form")]
    children: Option<ChildOutcomes>,
    #[serde(default, deserialize_with = "super::present::form")]
    when: Option<Predicate>,
}

impl TryFrom<WrittenRequirement> for OutcomeRequirement {
    type Error = String;

    fn try_from(written: WrittenRequirement) -> Result<Self, Self::Error> {
        let WrittenRequirement {
            decision,
            all,
            any,
            not,
            evidence,
            claim,
            is,
            input,
            r#in: values,
            first,
            before,
            after,
            since,
            children,
            when,
        } = written;
        let predicate = WrittenPredicate {
            all,
            any,
            not,
            evidence,
            claim,
            is,
            input,
            r#in: values,
            first,
            before,
            after,
            since,
            children,
        };
        let Some(decision) = decision else {
            if when.is_some() {
                return Err("`when` is written only beside `decision`".to_owned());
            }
            // With no key at all, name `decision` among what a requirement may be written as.
            if let WrittenPredicate {
                all: None,
                any: None,
                not: None,
                evidence: None,
                claim: None,
                is: None,
                input: None,
                r#in: None,
                first: None,
                before: None,
                after: None,
                since: None,
                children: None,
            } = predicate
            {
                return Err(
                    "a requirement has exactly one of `all`, `any`, `not`, `evidence`, \
                     `claim`, `input`, `first`, `before`, `after`, `since`, `children` or `decision`, \
                     found 0"
                        .to_owned(),
                );
            }
            return Predicate::try_from(predicate).map(OutcomeRequirement::Predicate);
        };
        let WrittenPredicate {
            all: None,
            any: None,
            not: None,
            evidence: None,
            claim: None,
            is: None,
            input: None,
            r#in: None,
            first: None,
            before: None,
            after: None,
            since: None,
            children: None,
        } = predicate
        else {
            return Err(
                "a requirement is either `decision` alone or a predicate, not both".to_owned(),
            );
        };
        Ok(OutcomeRequirement::Decision(DecisionRequirement {
            decision,
            when,
        }))
    }
}
