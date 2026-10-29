//! The `canon-evidence/1` evidence record: a typed observation about one subject artifact at one
//! revision (design § 4.5, § 9).

use serde::Deserialize;

use super::Declarations;
use super::ids::{ArtifactId, CaseId, EvidenceId, EvidenceKindId, Instant, Principal, Revision};

/// The evidence record format this model reads.
pub const EVIDENCE_FORMAT: &str = "canon-evidence/1";

/// One `canon-evidence/1` record as written. `subject` and `subject_revision` bind it: the
/// evaluator refuses a record whose subject the protocol does not declare, and applies a record
/// only while its subject revision is the case's current revision of that subject.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRecord {
    /// The document format; the evaluator accepts only [`EVIDENCE_FORMAT`].
    #[serde(deserialize_with = "super::present::required")]
    pub format: String,
    #[serde(deserialize_with = "super::present::required")]
    pub id: EvidenceId,
    /// The evidence kind, which the protocol declares.
    #[serde(deserialize_with = "super::present::required")]
    pub kind: EvidenceKindId,
    /// What the observation found; a record without one matches only an evidence match that names
    /// no result.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub result: Option<String>,
    /// The artifact the record is about.
    #[serde(deserialize_with = "super::present::required")]
    pub subject: ArtifactId,
    /// The revision of that artifact the record is about.
    #[serde(deserialize_with = "super::present::required")]
    pub subject_revision: Revision,
    /// When the observation was made; its age at the evaluation instant is measured from here. A
    /// record without it never expires.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub observed_at: Option<Instant>,
    /// The revision of each upstream artifact the observation was made against, keyed by
    /// artifact. A record without it is never invalidated.
    #[serde(default, deserialize_with = "super::present::defaulted")]
    pub upstream_revisions: Declarations<ArtifactId, Revision>,
    /// Who produced the record, recorded as given: Canon does not prove it. A record without one
    /// is of no evidence match that demands independence.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub producer: Option<Principal>,
    /// The child case a `case_outcome` record is about (story:case-composition): one record per
    /// child case, its `result` the child's outcome, `open` while the child runs. Written only on a
    /// `case_outcome` record, which must name its child and give an outcome.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub child_case: Option<CaseId>,
}
