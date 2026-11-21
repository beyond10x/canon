//! The grounds of an authority or explicit decision (story:decision-grounds): what an entry of
//! `canon-authority/1` or `canon-decisions/1` rested on, and whether each still holds.
//!
//! An entry may name `ground_revisions`, artifact revisions keyed by artifact, and
//! `ground_evidence`, evidence record ids. Both are optional; absent, each is empty, and an entry
//! with neither evaluates as it did before grounds existed. Grounds are a set: an evidence id named
//! twice is one ground.
//!
//! - An artifact ground holds while its revision equals the case snapshot's current revision of
//!   the artifact, compared for equality, the rule revision binding (`binding.rs`) applies to a
//!   record's `subject_revision`.
//! - An evidence ground holds while a record with its id is applicable: given, left by the
//!   revision-binding and freshness stages, and listed in no claim's `excluded_evidence`
//!   ([`super::missing::applicable`], the one rule `missing_evidence` reads). An id no given
//!   record has does not hold, and is not refused.
//!
//! An entry applies only when every ground holds and, when it has a case revision, that revision
//! is the case snapshot's `revision`, whatever it decided. `authority.rs` and `decisions.rs` decide
//! that once, before coverage, precedence and `decided_by` read the entries ([`Standing`] says
//! which grounds hold), so the `actions` and `outcomes` sections see only the entries that apply.
//! An entry set aside by its case revision adds none of its grounds to a reason: it would not apply
//! whatever its grounds. An entry at its case revision (or with none) whose grounds fail is set
//! aside by its grounds, and each ground that failed is a [`Failed`].
//!
//! A reason that a set-aside entry leaves undecided lists the failed grounds of every such entry
//! under `stale_grounds` ([`stale_grounds`]), each once, artifact grounds first, by artifact and
//! then revision, then evidence grounds by id:
//!
//! - `{"artifact": <id>, "reason": "revision_mismatch", "revision": <the ground's revision>}`;
//! - `{"evidence": <id>, "reason": <reason>}`, with the reason the record was excluded with: the
//!   one the revision-binding or freshness stage gave it, else the one the first claim listing it,
//!   in claim-id order, gives (`invalidated` or `not_independent`);
//! - `{"evidence": <id>, "present": false}` when no given record has the id.
//!
//! So the order of the entries in a document does not change the result. Grounds do not enter
//! either document's duplicate key.
//!
//! Refused: a ground artifact, revision or evidence id that is not an identifier, as
//! `invalid-identifier`, when the document is read ([`Grounds::read`]), and, after all three
//! supplied inputs are read, a ground artifact the protocol does not declare, as
//! `undeclared-artifact` ([`declared`]).

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};

use super::authority::Authority;
use super::decisions::Decisions;
use super::{Refusal, identifier};
use crate::ir::Ir;
use crate::model::{
    ArtifactId, Case, ClaimDecision, Declarations, EvidenceExclusion, EvidenceId, EvidenceRecord,
    ExclusionReason, Revision, one_line,
};

/// What an entry rested on: artifact revisions by artifact, and evidence record ids.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Grounds {
    pub(crate) revisions: BTreeMap<ArtifactId, Revision>,
    pub(crate) evidence: BTreeSet<EvidenceId>,
}

impl Grounds {
    /// The grounds an entry gives, each checked as an identifier, in the order written: each
    /// artifact and then its revision, then each evidence id; `what` names the document's entries
    /// in a refusal (`authority`, `decision`).
    pub(super) fn read(
        what: &str,
        revisions: &Declarations<ArtifactId, Revision>,
        evidence: &[EvidenceId],
    ) -> Result<Grounds, Refusal> {
        for (artifact, revision) in revisions.iter() {
            identifier(&format!("{what} ground artifact"), artifact.as_str())?;
            identifier(&format!("{what} ground revision"), revision.as_str())?;
        }
        for id in evidence {
            identifier(&format!("{what} ground evidence"), id.as_str())?;
        }
        Ok(Grounds {
            revisions: revisions
                .iter()
                .map(|(artifact, revision)| (artifact.clone(), revision.clone()))
                .collect(),
            evidence: evidence.iter().cloned().collect(),
        })
    }

    /// Adds the grounds as given to an entry of `computed_from.authority` or
    /// `computed_from.decisions` (`crate::explain`): `"ground_revisions": {<artifact>: <revision>,
    /// …}` and `"ground_evidence": [<id>, …]` in id order, each id once, each only when not
    /// empty, so an entry without grounds is written as before.
    pub(crate) fn explain(&self, entry: &mut Map<String, Value>) {
        if !self.revisions.is_empty() {
            let revisions: Map<String, Value> = self
                .revisions
                .iter()
                .map(|(artifact, revision)| {
                    (artifact.as_str().to_owned(), Value::from(revision.as_str()))
                })
                .collect();
            entry.insert("ground_revisions".to_owned(), Value::Object(revisions));
        }
        if !self.evidence.is_empty() {
            let ids: Vec<Value> = self
                .evidence
                .iter()
                .map(|id| Value::from(id.as_str()))
                .collect();
            entry.insert("ground_evidence".to_owned(), Value::Array(ids));
        }
    }
}

/// A ground that does not hold, as `stale_grounds` lists it. Ordered as listed: artifact grounds
/// first, by artifact and then revision, then evidence grounds by id.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Failed {
    /// An artifact ground whose revision is not the current one.
    Artifact(ArtifactId, Revision),
    /// An evidence ground whose record is not applicable: excluded with the reason, or not given
    /// (`None`).
    Evidence(EvidenceId, Option<ExclusionReason>),
}

impl Failed {
    fn json(&self) -> Value {
        let fields: Vec<(&str, Value)> = match self {
            Failed::Artifact(artifact, revision) => vec![
                ("artifact", Value::from(artifact.as_str())),
                (
                    "reason",
                    Value::from(ExclusionReason::RevisionMismatch.as_str()),
                ),
                ("revision", Value::from(revision.as_str())),
            ],
            Failed::Evidence(id, Some(reason)) => vec![
                ("evidence", Value::from(id.as_str())),
                ("reason", Value::from(reason.as_str())),
            ],
            Failed::Evidence(id, None) => vec![
                ("evidence", Value::from(id.as_str())),
                ("present", Value::Bool(false)),
            ],
        };
        Value::Object(
            fields
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value))
                .collect::<Map<_, _>>(),
        )
    }
}

/// The `stale_grounds` value of a reason: every failed ground, each once, in [`Failed`] order; or
/// `None` when there is none, and the reason is written without the key.
pub(super) fn stale_grounds(failed: &BTreeSet<Failed>) -> Option<Value> {
    (!failed.is_empty()).then(|| Value::Array(failed.iter().map(Failed::json).collect()))
}

/// Which grounds hold in one evaluation: the case snapshot's current revisions, the records that
/// are applicable and the reason each other given record was excluded with.
#[derive(Debug, Default)]
pub(crate) struct Standing<'a> {
    current: BTreeMap<&'a ArtifactId, &'a Revision>,
    applicable: BTreeSet<&'a EvidenceId>,
    excluded: BTreeMap<&'a EvidenceId, ExclusionReason>,
}

impl<'a> Standing<'a> {
    /// The standing of every ground in an evaluation of `case`: `left` is what the revision-binding
    /// and freshness stages left, `excluded` the records they set aside with their reasons, and
    /// `claims` every claim's entry, in claim-id order, as [`super::missing::applicable`] takes
    /// them. The records given are `left` and `excluded` together.
    pub(super) fn new(
        case: &'a Case,
        left: &'a [EvidenceRecord],
        excluded: &'a [EvidenceExclusion],
        claims: impl IntoIterator<Item = &'a ClaimDecision>,
    ) -> Standing<'a> {
        let claims: Vec<&ClaimDecision> = claims.into_iter().collect();
        let applicable = super::missing::applicable(left, claims.iter().copied())
            .into_iter()
            .map(|record| &record.id)
            .collect();
        let mut reasons: BTreeMap<&EvidenceId, ExclusionReason> = BTreeMap::new();
        let listed = excluded
            .iter()
            .chain(claims.iter().flat_map(|claim| &claim.excluded_evidence));
        for exclusion in listed {
            reasons
                .entry(&exclusion.evidence)
                .or_insert(exclusion.reason);
        }
        Standing {
            current: case
                .artifacts
                .iter()
                .map(|(artifact, at)| (artifact, &at.revision))
                .collect(),
            applicable,
            excluded: reasons,
        }
    }

    /// The grounds of `grounds` that do not hold. Empty when every one holds.
    pub(super) fn failed(&self, grounds: &Grounds) -> BTreeSet<Failed> {
        let artifacts = grounds
            .revisions
            .iter()
            .filter(|(artifact, revision)| self.current.get(artifact) != Some(revision))
            .map(|(artifact, revision)| Failed::Artifact(artifact.clone(), revision.clone()));
        let evidence = grounds
            .evidence
            .iter()
            .filter(|id| !self.applicable.contains(id))
            .map(|id| Failed::Evidence(id.clone(), self.excluded.get(id).copied()));
        artifacts.chain(evidence).collect()
    }
}

/// Refuses a ground artifact the protocol does not declare, as `undeclared-artifact`: each
/// authority entry in the order the explanation lists them (capability, scope, case revision,
/// principal), then each explicit decision in the order given, each entry's artifacts in id order.
pub(super) fn declared(
    ir: &Ir,
    authority: Option<&Authority>,
    decisions: Option<&Decisions>,
) -> Result<(), Refusal> {
    let undeclared = |grounds: &Grounds| -> Option<ArtifactId> {
        grounds
            .revisions
            .keys()
            .find(|artifact| !ir.artifacts.contains_key(*artifact))
            .cloned()
    };
    for given in authority.into_iter().flat_map(Authority::decisions) {
        if let Some(artifact) = undeclared(given.grounds) {
            return Err(Refusal::new(
                "undeclared-artifact",
                format!(
                    "`--authority` decision on capability `{}` rests on artifact `{}`, which the \
                     protocol does not declare",
                    one_line(given.capability.as_str()),
                    one_line(artifact.as_str())
                ),
            ));
        }
    }
    for (entry, grounds) in decisions.into_iter().flat_map(Decisions::grounded) {
        if let Some(artifact) = undeclared(grounds) {
            return Err(Refusal::new(
                "undeclared-artifact",
                format!(
                    "`--decisions` decision `{}` for outcome `{}` rests on artifact `{}`, which \
                     the protocol does not declare",
                    one_line(entry.decision.as_str()),
                    one_line(entry.outcome.as_str()),
                    one_line(artifact.as_str())
                ),
            ));
        }
    }
    Ok(())
}
