//! The `canon-case/1` case snapshot: one concrete undertaking governed by one protocol, with the
//! current revision of each artifact the protocol declares (design § 4.2).

use serde::Deserialize;

use super::Declarations;
use super::ids::{ArtifactId, CaseId, ProtocolId, Revision};

/// The case snapshot format this model reads.
pub const CASE_FORMAT: &str = "canon-case/1";

/// A `canon-case/1` case snapshot as written.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    /// The document format; the evaluator accepts only [`CASE_FORMAT`].
    #[serde(deserialize_with = "super::present::required")]
    pub format: String,
    pub id: CaseId,
    /// The protocol that governs the case.
    pub protocol: ProtocolId,
    /// The current revision of each declared artifact, keyed by artifact id.
    #[serde(deserialize_with = "super::present::required")]
    pub artifacts: Declarations<ArtifactId, CaseArtifact>,
}

/// One declared artifact as the case snapshot records it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseArtifact {
    /// The artifact's current revision.
    pub revision: Revision,
}
