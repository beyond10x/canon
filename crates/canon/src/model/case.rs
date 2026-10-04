//! The `canon-case/1` case snapshot: one concrete undertaking governed by one protocol, with the
//! current revision of each artifact the protocol declares (design § 4.2), and optionally the
//! case's own revision, which explicit decisions are bound to.

use serde::Deserialize;

use super::Declarations;
use super::ids::{ArtifactId, CaseId, OutcomeId, ProtocolId, Revision};

/// The case snapshot format this model reads.
pub const CASE_FORMAT: &str = "canon-case/1";

/// A `canon-case/1` case snapshot as written.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    /// The document format; the evaluator accepts only [`CASE_FORMAT`].
    #[serde(deserialize_with = "super::present::required")]
    pub format: String,
    #[serde(deserialize_with = "super::present::required")]
    pub id: CaseId,
    /// The protocol that governs the case.
    #[serde(deserialize_with = "super::present::required")]
    pub protocol: ProtocolId,
    /// The current revision of each declared artifact, keyed by artifact id.
    #[serde(deserialize_with = "super::present::required")]
    pub artifacts: Declarations<ArtifactId, CaseArtifact>,
    /// The outcome the case terminated through; absent while the case is open. Whether the
    /// protocol declares it is checked by the outcomes section (story:outcomes).
    #[serde(default, deserialize_with = "super::present::optional")]
    pub termination: Option<OutcomeId>,
    /// The case's own current revision. An explicit decision applies only while its
    /// `case_revision` is this one; a snapshot without one has no explicit decision applying.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub revision: Option<Revision>,
}

/// One declared artifact as the case snapshot records it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseArtifact {
    /// The artifact's current revision.
    #[serde(deserialize_with = "super::present::required")]
    pub revision: Revision,
}
