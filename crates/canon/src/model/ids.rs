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

        identifier_impls!($name);
    };
}

/// An identifier read from any scalar: a string as written, and a scalar YAML reads as a boolean or
/// a number (`true`, `2`) as its text. One reader serves every document it appears in, so the
/// protocol, which `serde_yaml_ng` reads from text, and the case, which it reads from an already
/// parsed YAML value, read the same scalar to the same text.
macro_rules! scalar_identifier {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        /// Reads any scalar as its text, refusing an explicit null as every identifier does.
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                super::present::identifier::<D, Scalar>(deserializer).map(|scalar| Self(scalar.0))
            }
        }

        identifier_impls!($name);
    };
}

/// What every identifier type has besides its reader.
macro_rules! identifier_impls {
    ($name:ident) => {
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

/// A scalar read as its text: a string as written, a boolean as `true` or `false`, a number as
/// Rust writes it (`2`, `-1`, `0.5`). A sequence or a map is refused, naming what was expected.
struct Scalar(String);

impl<'de> Deserialize<'de> for Scalar {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Text;

        impl serde::de::Visitor<'_> for Text {
            type Value = Scalar;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a string, a boolean or a number")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Scalar, E> {
                Ok(Scalar(value.to_owned()))
            }

            fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Scalar, E> {
                Ok(Scalar(value))
            }

            fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Scalar, E> {
                Ok(Scalar(value.to_string()))
            }

            fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Scalar, E> {
                Ok(Scalar(value.to_string()))
            }

            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Scalar, E> {
                Ok(Scalar(value.to_string()))
            }

            fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Scalar, E> {
                Ok(Scalar(value.to_string()))
            }
        }

        deserializer.deserialize_any(Text)
    }
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
    /// An instant written `YYYY-MM-DDTHH:MM:SSZ` in UTC, or with a fraction of one to nine digits
    /// after the seconds, `YYYY-MM-DDTHH:MM:SS.fffZ`; uppercase `T` and `Z`, no leap second
    /// (`2026-10-04T12:00:00Z`, `2026-10-04T12:00:00.400Z`): when evidence was observed, and the
    /// evaluation instant. It names the interval its last written digit spans (`12:00:00Z` the
    /// whole second, `12:00:00.4Z` a tenth of it); order forms compare those intervals, and expiry
    /// reads the whole seconds and ignores the fraction.
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
    /// Who took an explicit decision, or who produced an evidence record, recorded as given: Canon
    /// resolves no identity and proves no production.
    Principal
);
identifier!(
    /// Identifies an independence dimension a protocol declares (`different_principal`). Canon
    /// gives no dimension a meaning; the trusted context that writes the case resolves it.
    IndependenceDimensionId
);
identifier!(
    /// Identifies an invalidation rule a protocol declares.
    InvalidationRuleId
);
identifier!(
    /// Identifies a case input a protocol declares.
    InputId
);
scalar_identifier!(
    /// One value a case input may take: an identifier its input declares. Read from any scalar,
    /// alike in the protocol and in the case: `values: [true, false]` declares the values `true`
    /// and `false`, and a case gives one as `flag: true`.
    InputValue
);
