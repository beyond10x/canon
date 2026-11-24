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
//!   ground_revisions: {spec: r2}  # optional: the artifact revisions it rested on
//!   ground_evidence: [e1]         # optional: the evidence records it rested on, each an id
//!                                 # or {id: e1, digest: sha256:<hex>}
//! ```

use serde::Deserialize;

use super::Declarations;
use super::authority::EvidenceGround;
use super::ids::{ArtifactId, DecisionName, OutcomeId, Principal, Revision};

/// The explicit decisions format the evaluator reads.
pub const DECISIONS_FORMAT: &str = "canon-decisions/1";

/// One entry of a `canon-decisions/1` document. It applies to `outcome` when the outcome requires
/// a decision named `decision`, `case_revision` is the case snapshot's revision and every ground
/// holds.
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
    /// The artifact revisions the decision rested on, keyed by artifact, each key once
    /// (story:decision-grounds): a ground holds while its revision is the case snapshot's current
    /// revision of the artifact, compared for equality. Absent or empty, it rests on no artifact.
    #[serde(default, deserialize_with = "super::present::defaulted")]
    pub ground_revisions: Declarations<ArtifactId, Revision>,
    /// The evidence records the decision rested on, each by id or by id and digest
    /// ([`EvidenceGround`]): a ground holds while a record with that id is applicable, the rule of
    /// `missing_evidence` (given, left by the revision-binding and freshness stages, and listed in
    /// no claim's `excluded_evidence`), so expiry and invalidation reach the decision through it,
    /// and, when it gives a digest, while that record has the digest. An id no record has does not
    /// hold and is not refused. Absent or empty, it rests on no record.
    #[serde(default, deserialize_with = "super::present::defaulted")]
    pub ground_evidence: Vec<EvidenceGround>,
}
