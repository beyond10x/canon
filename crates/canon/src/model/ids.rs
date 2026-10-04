//! Typed identifiers.
//!
//! Every identifier Canon handles is a distinct type, so an action id cannot be passed where a
//! claim id is expected. Each wraps the identifier exactly as written in its document.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

macro_rules! identifier {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        /// Reads the identifier as text, refusing an explicit null rather than reading it as
        /// the text `~` or `null`.
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                super::present::identifier::<D, String>(deserializer).map(Self)
            }
        }

        impl $name {
            pub fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(id: &str) -> Self {
                Self::new(id)
            }
        }
    };
}

identifier!(
    /// Identifies a protocol, independently of its revision.
    ProtocolId
);
identifier!(
    /// Identifies one case governed by a protocol.
    CaseId
);
identifier!(
    /// Identifies an artifact a protocol declares.
    ArtifactId
);
identifier!(
    /// Identifies one revision of an artifact, or of the case itself: the case snapshot's own
    /// revision, and the case revision an explicit decision was taken at.
    Revision
);
identifier!(
    /// Identifies a claim a protocol declares.
    ClaimId
);
identifier!(
    /// Identifies one evidence record.
    EvidenceId
);
identifier!(
    /// Identifies a kind of evidence a protocol declares.
    EvidenceKindId
);
identifier!(
    /// Identifies an obligation a protocol declares.
    ObligationId
);
identifier!(
    /// Identifies an action a protocol declares.
    ActionId
);
identifier!(
    /// Identifies an outcome a protocol declares.
    OutcomeId
);
identifier!(
    /// Names a capability an action requires authority for.
    CapabilityId
);
identifier!(
    /// Names the class of effect an action has.
    EffectClass
);
identifier!(
    /// An instant written `YYYY-MM-DDTHH:MM:SSZ` in UTC, whole seconds, uppercase `T` and `Z`, no
    /// leap second (`2026-10-04T12:00:00Z`): when evidence was observed, and the evaluation
    /// instant.
    Instant
);
identifier!(
    /// A length of time: a whole number without leading zeros, followed by `s`, `m`, `h` or `d`
    /// (`5m`, `24h`).
    Age
);
identifier!(
    /// Names an explicit decision an outcome may require (`explicitly_inconclusive`).
    DecisionName
);
identifier!(
    /// Who took an explicit decision, recorded as given: Canon resolves no identity.
    Principal
);
identifier!(
    /// Identifies an invalidation rule a protocol declares.
    InvalidationRuleId
);
