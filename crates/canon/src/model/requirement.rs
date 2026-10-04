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
//! A decision is not a predicate form: it can be written only as the whole of an outcome's
//! `requires`, never inside `all`, `any` or `not`, and never in a claim, obligation or action.

use serde::Deserialize;

use super::ids::{ClaimId, DecisionName};
use super::predicate::{EvidenceMatch, Predicate, Truth, WrittenPredicate};

/// What an outcome requires.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "WrittenRequirement")]
pub enum OutcomeRequirement {
    /// The outcome is legitimate when the predicate is `true`.
    Predicate(Predicate),
    /// The outcome is legitimate only with an explicit decision of this name, for the outcome,
    /// taken at the case snapshot's current revision.
    Decision(DecisionName),
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
            OutcomeRequirement::Decision(name) => Some(name),
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
        } = written;
        let predicate = WrittenPredicate {
            all,
            any,
            not,
            evidence,
            claim,
            is,
        };
        let Some(decision) = decision else {
            // With no key at all, name `decision` among what a requirement may be written as.
            if let WrittenPredicate {
                all: None,
                any: None,
                not: None,
                evidence: None,
                claim: None,
                is: None,
            } = predicate
            {
                return Err(
                    "a requirement has exactly one of `all`, `any`, `not`, `evidence`, \
                     `claim` or `decision`, found 0"
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
        } = predicate
        else {
            return Err(
                "a requirement is either `decision` alone or a predicate, not both".to_owned(),
            );
        };
        Ok(OutcomeRequirement::Decision(decision))
    }
}
