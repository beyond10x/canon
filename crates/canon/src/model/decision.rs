//! The `canon-decision/1` document: which protocol revision applied to which case, the
//! three-valued value of every claim the protocol declares (design § 8, § 14), and one optional
//! slot per section the later evaluator stories fill.

use super::Declarations;
use super::ids::{CaseId, ClaimId, EvidenceId, ProtocolId};
use super::predicate::Truth;

/// The decision format the evaluator writes.
pub const DECISION_FORMAT: &str = "canon-decision/1";

/// An untyped section payload: what each section slot of [`Decision`] holds. A section's shape is
/// fixed by its story's conformance scenarios; typing the slots is a later model story.
pub type Json = serde_json::Value;

/// A `canon-decision/1` document. A section slot that is `None` is not serialized.
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
    /// Each obligation's status; filled by story:obligations, its shape fixed by that story's
    /// conformance scenarios.
    pub obligations: Option<Json>,
    /// The admissible actions; filled by story:action-admissibility, its shape fixed by that
    /// story's conformance scenarios.
    pub actions: Option<Json>,
    /// Each outcome's status; filled by story:outcomes, its shape fixed by that story's
    /// conformance scenarios.
    pub outcomes: Option<Json>,
    /// The structured explanation; filled by story:explanation, its shape fixed by that story's
    /// conformance scenarios.
    pub explanation: Option<Json>,
}

/// One claim's entry in a decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimDecision {
    pub value: Truth,
    /// The evidence an exclusion stage kept from this claim, in evidence-id order. Empty unless a
    /// stage excluded something; an empty list is not serialized.
    pub excluded_evidence: Vec<EvidenceExclusion>,
}

/// One evidence record an exclusion stage kept from a claim, and why.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EvidenceExclusion {
    pub evidence: EvidenceId,
    pub reason: ExclusionReason,
}

/// Why evidence was excluded before claims were evaluated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExclusionReason {
    /// Bound to a revision of its subject that is not the current one.
    RevisionMismatch,
    /// Older than its kind allows at the evaluation instant.
    Expired,
    /// Invalidated by an invalidation rule.
    Invalidated,
}

impl ExclusionReason {
    /// The reason as `canon-decision/1` writes it.
    pub fn as_str(self) -> &'static str {
        match self {
            ExclusionReason::RevisionMismatch => "revision_mismatch",
            ExclusionReason::Expired => "expired",
            ExclusionReason::Invalidated => "invalidated",
        }
    }
}
