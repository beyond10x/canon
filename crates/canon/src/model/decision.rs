//! The `canon-decision/1` document: which protocol revision applied to which case, and the
//! three-valued value of every claim the protocol declares (design § 8, § 14).

use super::Declarations;
use super::ids::{CaseId, ClaimId, ProtocolId};
use super::predicate::Truth;

/// The decision format the evaluator writes.
pub const DECISION_FORMAT: &str = "canon-decision/1";

/// A `canon-decision/1` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    /// Always [`DECISION_FORMAT`].
    pub format: String,
    pub case: CaseId,
    pub protocol: ProtocolId,
    /// The revision of the compiled protocol that applied.
    pub protocol_revision: u64,
    /// Every declared claim, in identifier order.
    pub claims: Declarations<ClaimId, ClaimDecision>,
}

/// One claim's entry in a decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimDecision {
    pub value: Truth,
}
