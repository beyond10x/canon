//! The `canon-properties/1` document `canon check --properties` reads: properties declared beside a
//! `protocol/1` document and checked in every state of its finite state space (story:canon-check).
//!
//! ```yaml
//! format: canon-properties/1
//! protocol: investigation
//! properties:
//!   inconclusive.rests.on.decision:
//!     description: Ending inconclusive rests on the explicit decision.
//!     subject:
//!       outcome: inconclusive      # or `action: <id>`
//!     independent_of:
//!       claim: explanation.supported
//! ```
//!
//! One property form, independence: the subject's status is the same in every two states that
//! agree on everything except the evidence the `independent_of` claim reads. What that means over
//! the state space is `crate::check`'s; this file is the document's shape.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

use super::Declarations;
use super::ids::{ActionId, ClaimId, OutcomeId, ProtocolId};

/// The properties format `canon check` reads.
pub const PROPERTIES_FORMAT: &str = "canon-properties/1";

identifier!(
    /// Identifies one property of a `canon-properties/1` document.
    PropertyId
);

/// A `canon-properties/1` document as written.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Properties {
    /// The document format; `canon check` accepts only [`PROPERTIES_FORMAT`].
    #[serde(deserialize_with = "super::present::required")]
    pub format: String,
    /// The id of the protocol the properties are about; `canon check` refuses another.
    pub protocol: ProtocolId,
    /// The properties, keyed by identifier. A property declared twice is refused.
    #[serde(default, deserialize_with = "super::present::required")]
    pub properties: Declarations<PropertyId, Property>,
}

/// One property: its subject's status is the same in every two states that agree on everything
/// except the evidence the `independent_of` claim reads.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Property {
    #[serde(default, deserialize_with = "super::present::optional")]
    pub description: Option<String>,
    /// The action or outcome whose status the property is about.
    pub subject: PropertySubject,
    /// What the subject's status must not depend on.
    pub independent_of: PropertyDependency,
}

/// What a property is about. Written as a map with exactly one of the keys `action` or `outcome`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "WrittenSubject")]
pub enum PropertySubject {
    /// The action's status: `admissible`, `approval-required` or `blocked`.
    Action(ActionId),
    /// The outcome's status: `legitimate` or `blocked`.
    Outcome(OutcomeId),
}

/// What a property's subject must not depend on. Written as a map with the one key `claim`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "WrittenDependency")]
pub enum PropertyDependency {
    /// The claim, standing for the evidence it reads, by kind and subject, directly or through the
    /// claims it tests.
    Claim(ClaimId),
}

/// A subject as written: each form a key, exactly one of them present.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenSubject {
    #[serde(default, deserialize_with = "super::present::optional")]
    action: Option<ActionId>,
    #[serde(default, deserialize_with = "super::present::optional")]
    outcome: Option<OutcomeId>,
}

impl TryFrom<WrittenSubject> for PropertySubject {
    type Error = String;

    fn try_from(written: WrittenSubject) -> Result<Self, Self::Error> {
        match (written.action, written.outcome) {
            (Some(action), None) => Ok(PropertySubject::Action(action)),
            (None, Some(outcome)) => Ok(PropertySubject::Outcome(outcome)),
            _ => Err("a subject has exactly one of `action` or `outcome`".to_owned()),
        }
    }
}

/// A dependency as written: the one key `claim`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WrittenDependency {
    claim: ClaimId,
}

impl TryFrom<WrittenDependency> for PropertyDependency {
    type Error = String;

    fn try_from(written: WrittenDependency) -> Result<Self, Self::Error> {
        Ok(PropertyDependency::Claim(written.claim))
    }
}
