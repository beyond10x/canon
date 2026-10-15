//! The `canon-requirements/1` catalogue `canon conform run --requirements` reads: the normative
//! requirements Canon implements or plans, each by its `CANON-*` id (story:conformance-suite).
//!
//! ```yaml
//! format: canon-requirements/1
//! requirements:
//!   CANON-CLAIM-001:
//!     status: implemented        # implemented | planned
//!     statement: A required positive predicate evaluating to UNKNOWN MUST NOT be treated as satisfied.
//!     source: design § 32        # where the requirement is stated
//! ```
//!
//! What the catalogue is checked against, the registry's scenarios, is `crate::conform`'s; this
//! file is the document's shape.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

use super::Declarations;

/// The catalogue format `canon conform run --requirements` reads.
pub const REQUIREMENTS_FORMAT: &str = "canon-requirements/1";

identifier!(
    /// Identifies one normative requirement: `CANON-` followed by an identifier.
    RequirementId
);

/// A `canon-requirements/1` document as written.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequirementCatalogue {
    /// The document format; only [`REQUIREMENTS_FORMAT`] is accepted.
    #[serde(deserialize_with = "super::present::required")]
    pub format: String,
    /// The requirements, keyed by id, in source order. A requirement listed twice is refused.
    #[serde(deserialize_with = "super::present::required")]
    pub requirements: Declarations<RequirementId, NormativeRequirement>,
}

/// One normative requirement.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NormativeRequirement {
    #[serde(deserialize_with = "super::present::required")]
    pub status: RequirementStatus,
    /// What the requirement demands, in the words of its source.
    #[serde(deserialize_with = "super::present::required")]
    pub statement: String,
    /// Where the requirement is stated: a section of the design, or the story that introduced it.
    #[serde(deserialize_with = "super::present::required")]
    pub source: String,
}

/// Whether a requirement is implemented, so a passing scenario must cover it, or only planned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RequirementStatus {
    /// Implemented: at least one passing scenario of the registry covers it.
    Implemented,
    /// Stated, not yet implemented: no scenario is needed, and a passing one covering it fails the
    /// run, so the catalogue is moved forward rather than drifting.
    Planned,
}
