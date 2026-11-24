//! The `canon-authority/1` authority decisions an evaluation may read (`--authority`): which
//! capabilities, at which scope, whoever runs the evaluation grants, grants only with approval or
//! denies (design § 33, Atlas ADR 0083 § Decision 1).
//!
//! The document is a YAML (or JSON) list of entries:
//!
//! ```yaml
//! - capability: metrics.read
//!   scope: {cluster: prod-eu}  # optional: absent, the decision covers every scope
//!   decision: granted          # or require_approval, or denied
//!   principal: ops-lead        # optional: who took it, recorded as given
//!   case_revision: c2          # optional: absent, the decision applies at every case revision
//!   ground_revisions: {spec: r2}  # optional: the artifact revisions it rested on
//!   ground_evidence: [e1]         # optional: the evidence records it rested on, each an id
//!                                 # or {id: e1, digest: sha256:<hex>}
//! ```

use std::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use super::Declarations;
use super::ids::{ArtifactId, CapabilityId, EvidenceDigest, EvidenceId, Principal, Revision};

/// The authority decisions format the evaluator reads.
pub const AUTHORITY_FORMAT: &str = "canon-authority/1";

identifier!(
    /// A key of a capability scope (`cluster`, `namespace`): what the scope narrows the capability
    /// to. An identifier.
    ScopeKey
);
identifier!(
    /// The value a capability scope gives one key (`prod-eu`). An identifier.
    ScopeValue
);

/// One entry of a `canon-authority/1` document: a capability, the scope the decision covers, the
/// decision and, optionally, who took it, the case revision it was taken at and the grounds it
/// rested on.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityDecision {
    /// The capability decided, an identifier an action's `requires` may name.
    #[serde(deserialize_with = "super::present::required")]
    pub capability: CapabilityId,
    /// The scope the decision covers: one value for each key it names, each key once. It covers a
    /// requirement of its capability when the requirement gives each of those keys the same value;
    /// a key it leaves out covers every value of it. Absent or empty, it covers every scope.
    #[serde(default, deserialize_with = "super::present::defaulted")]
    pub scope: Declarations<ScopeKey, ScopeValue>,
    /// Whether it is granted.
    pub decision: Grant,
    /// Who took it, an identifier, recorded as given: Canon resolves no identity, so it is matched
    /// against nothing and does not change admissibility.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub principal: Option<Principal>,
    /// The case revision it was taken at, an identifier: the decision applies only while the case
    /// snapshot's `revision` equals it, and to nothing when the snapshot has none. Absent, it
    /// applies at every case revision.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub case_revision: Option<Revision>,
    /// The artifact revisions the decision rested on, keyed by artifact, each key once
    /// (story:decision-grounds): a ground holds while its revision is the case snapshot's current
    /// revision of the artifact, compared for equality. The decision applies only while every
    /// ground holds, whatever it decided. Absent or empty, it rests on no artifact.
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

/// One evidence record a decision rested on, an entry of `ground_evidence`
/// (story:evidence-digest-binds-approvals). Written as the record's id alone (`e1`), or as a map
/// with the id and, optionally, the digest of the record's contents when the decision was taken
/// (`{id: e1, digest: 'sha256:…'}`, the digest `canon digest --evidence <file>` prints). A map
/// without a digest is the id alone. With a digest, the ground holds only while the record given
/// under the id has that digest; with other contents under the id it fails as `content_changed`,
/// whether or not the record is applicable. Refused as `malformed-input`, naming the entry: a map
/// with another key or without `id`, and a digest not written `sha256:` followed by 64 lowercase
/// hexadecimal digits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceGround {
    /// The record's id alone: the ground holds while a record with the id is applicable, whatever
    /// the rest of its contents.
    Id(EvidenceId),
    /// The record's id with, optionally, the digest of its contents.
    Cited(CitedEvidence),
}

impl EvidenceGround {
    /// The id of the record the ground names.
    pub fn id(&self) -> &EvidenceId {
        match self {
            EvidenceGround::Id(id) => id,
            EvidenceGround::Cited(cited) => &cited.id,
        }
    }

    /// The digest the ground gives, if any.
    pub fn digest(&self) -> Option<&EvidenceDigest> {
        match self {
            EvidenceGround::Id(_) => None,
            EvidenceGround::Cited(cited) => cited.digest.as_ref(),
        }
    }
}

/// Reads anything but a map exactly as an [`EvidenceId`] is read, so an entry written without the
/// map form reads, and is refused, as before the map form existed; a map as a [`CitedEvidence`],
/// whose digest must then be well-formed.
impl<'de> Deserialize<'de> for EvidenceGround {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_yaml_ng::Value::deserialize(deserializer)?;
        if !value.is_mapping() {
            return EvidenceId::deserialize(value)
                .map(EvidenceGround::Id)
                .map_err(D::Error::custom);
        }
        let cited = CitedEvidence::deserialize(value).map_err(D::Error::custom)?;
        if let Some(digest) = &cited.digest
            && !is_digest(digest.as_str())
        {
            return Err(D::Error::custom(format!(
                "ground evidence `{}` gives digest `{}`, which is not `{DIGEST_PREFIX}` followed by \
                 64 lowercase hexadecimal digits",
                super::one_line(cited.id.as_str()),
                super::one_line(digest.as_str())
            )));
        }
        Ok(EvidenceGround::Cited(cited))
    }
}

/// An evidence ground written as a map: the record's id and, optionally, the digest of its
/// contents.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CitedEvidence {
    /// The record's id, an identifier.
    #[serde(deserialize_with = "super::present::required")]
    pub id: EvidenceId,
    /// The digest of the record's contents when the decision was taken. Absent, the ground holds
    /// as the id alone does.
    #[serde(default, deserialize_with = "super::present::optional")]
    pub digest: Option<EvidenceDigest>,
}

/// What every evidence digest begins with.
const DIGEST_PREFIX: &str = "sha256:";

/// Whether `text` is [`DIGEST_PREFIX`] followed by 64 lowercase hexadecimal digits.
fn is_digest(text: &str) -> bool {
    text.strip_prefix(DIGEST_PREFIX).is_some_and(|hex| {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

/// An authority decision on one capability at one scope. Among the decisions covering a
/// requirement, `denied` wins over `require_approval`, which wins over `granted`; none covering it
/// is not granted. With its precondition `true`, an action is `admissible` when every requirement
/// is granted, `blocked` when one is denied, and `approval-required` when none is denied and one
/// requires approval or is not decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grant {
    /// The capability is granted.
    Granted,
    /// The capability is granted only with approval: asking for approval could grant it.
    RequireApproval,
    /// The capability is denied; asking again would not change it.
    Denied,
}

impl Grant {
    /// The decision as `canon-authority/1` and `canon-decision/1` write it.
    pub fn as_str(self) -> &'static str {
        match self {
            Grant::Granted => "granted",
            Grant::RequireApproval => "require_approval",
            Grant::Denied => "denied",
        }
    }

    /// How the decision ranks among those covering one requirement: the highest applies, so
    /// `denied` over `require_approval` over `granted`.
    pub fn precedence(self) -> u8 {
        match self {
            Grant::Granted => 0,
            Grant::RequireApproval => 1,
            Grant::Denied => 2,
        }
    }
}
