//! The `canon-authority/1` authority decisions an evaluation may read (`--authority`): which
//! capabilities whoever runs the evaluation grants or denies (design § 33).
//!
//! The document is a YAML (or JSON) list of entries:
//!
//! ```yaml
//! - capability: finding.publish
//!   decision: granted        # or denied
//! ```

use serde::Deserialize;

use super::ids::CapabilityId;

/// The authority decisions format the evaluator reads.
pub const AUTHORITY_FORMAT: &str = "canon-authority/1";

/// One entry of a `canon-authority/1` document: a capability and whether it is granted.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityDecision {
    /// The capability decided, an identifier an action's `requires` may name.
    #[serde(deserialize_with = "super::present::required")]
    pub capability: CapabilityId,
    /// Whether it is granted.
    pub decision: Grant,
}

/// An authority decision on one capability. With its precondition `true`, an action is
/// `admissible` when every capability it requires is granted, `blocked` when one is denied, and
/// `approval-required` when none is denied and one is not decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Grant {
    /// The capability is granted.
    Granted,
    /// The capability is denied; asking again would not change it.
    Denied,
}

impl Grant {
    /// The decision as `canon-authority/1` and `canon-decision/1` write it.
    pub fn as_str(self) -> &'static str {
        match self {
            Grant::Granted => "granted",
            Grant::Denied => "denied",
        }
    }
}
