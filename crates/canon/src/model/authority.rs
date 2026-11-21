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
//!   ground_evidence: [e1]         # optional: the evidence records it rested on
//! ```

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};

use super::Declarations;
use super::ids::{ArtifactId, CapabilityId, EvidenceId, Principal, Revision};

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
    /// The evidence records the decision rested on, by id: a ground holds while a record with
    /// that id is applicable (`missing_evidence`'s rule). An id no record has does not hold and is
    /// not refused. Absent or empty, it rests on no record.
    #[serde(default, deserialize_with = "super::present::defaulted")]
    pub ground_evidence: Vec<EvidenceId>,
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
