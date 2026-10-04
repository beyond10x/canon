//! The `canon-evidence/1` evidence record: a typed observation about one subject artifact at one
//! revision (design § 4.5, § 9).

use serde::Deserialize;

use super::Declarations;
use super::ids::{ArtifactId, EvidenceId, EvidenceKindId, Instant, Revision};

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
    pub id: EvidenceId,
    /// The evidence kind, which the protocol declares.
    pub kind: EvidenceKindId,
    /// What the observation found; a record without one matches only an evidence match that names
    /// no result.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub result: Option<String>,
    /// The artifact the record is about.
    pub subject: ArtifactId,
    /// The revision of that artifact the record is about.
    pub subject_revision: Revision,
    /// When the observation was made; its age at the evaluation instant is measured from here. A
    /// record without it never expires.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub observed_at: Option<Instant>,
    /// The revision of each upstream artifact the observation was made against, keyed by
    /// artifact. A record without it is never invalidated.
    #[serde(default, deserialize_with = "super::present::required")]
    pub upstream_revisions: Declarations<ArtifactId, Revision>,
}
