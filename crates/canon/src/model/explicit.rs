//! The `canon-decisions/1` explicit decisions an evaluation may read (`--decisions`): who decided
//! what, for which outcome, at which case revision (story:decision-outcomes).
//!
//! The document is a YAML (or JSON) list of entries:
//!
//! ```yaml
//! - decision: explicitly_inconclusive
//!   outcome: inconclusive
//!   principal: lead-investigator
//!   case_revision: c2
//! ```

use serde::Deserialize;

use super::ids::{DecisionName, OutcomeId, Principal, Revision};

/// The explicit decisions format the evaluator reads.
pub const DECISIONS_FORMAT: &str = "canon-decisions/1";

/// One entry of a `canon-decisions/1` document. It applies to `outcome` when the outcome requires
/// a decision named `decision` and `case_revision` is the case snapshot's revision.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExplicitDecision {
    /// Which decision was taken.
    #[serde(deserialize_with = "super::present::required")]
    pub decision: DecisionName,
    /// The outcome it was taken for.
    #[serde(deserialize_with = "super::present::required")]
    pub outcome: OutcomeId,
    /// Who took it.
    #[serde(deserialize_with = "super::present::required")]
    pub principal: Principal,
    /// The case revision it was taken at.
    #[serde(deserialize_with = "super::present::required")]
    pub case_revision: Revision,
}
