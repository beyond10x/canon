//! The `canon-decision/1` document: which protocol revision applied to which case, the
//! three-valued value of every claim the protocol declares (design § 8, § 14), and one optional
//! slot per further section: obligations, actions, outcomes and the explanation.

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
    /// Each declared obligation, `open` or `discharged`, in identifier order; written when the
    /// protocol declares an obligation. Its shape is fixed by CANON-OBLIGATION-001.
    pub obligations: Option<Json>,
    /// Each declared action, `admissible`, `approval-required` or `blocked`, with the reasons for
    /// a status other than `admissible`; written when the protocol declares an action. Its shape is
    /// fixed by CANON-AUTHORITY-001.
    pub actions: Option<Json>,
    /// Each declared outcome, `legitimate` or `blocked`, with the reasons it is blocked; written
    /// when the protocol declares an outcome. Its shape is fixed by CANON-OUTCOME-001 and, for an
    /// outcome that requires an explicit decision, by CANON-OUTCOME-002: a legitimate one records
    /// `decided_by` (`{"decision": <name>, "principals": [...]}`), and a blocked one gives the
    /// reason `{"decision": <name>, "present": false}`.
    pub outcomes: Option<Json>,
    /// Why each claim that is not `true`, each open obligation, each action that is not
    /// admissible and each blocked outcome has its status, traced to the evidence records that
    /// applied or were excluded, and what the decision was computed from (`computed_from`); the
    /// evaluator writes it on every decision. Its shape is fixed by CANON-EXPLAIN-001.
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
