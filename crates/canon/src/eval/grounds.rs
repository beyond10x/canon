//! The grounds of an authority or explicit decision (story:decision-grounds): what an entry of
//! `canon-authority/1` or `canon-decisions/1` rested on, and whether each still holds.
//!
//! An entry may name `ground_revisions`, artifact revisions keyed by artifact, and
//! `ground_evidence`, evidence records, each its id alone or a map of its id and, optionally, the
//! digest of its contents (`{id: e1, digest: 'sha256:…'}`, story:evidence-digest-binds-approvals,
//! [`crate::model::EvidenceGround`]). Both are optional; absent, each is empty, and an entry with
//! neither evaluates as it did before grounds existed. Grounds are a set: an evidence id named
//! twice is one ground, and so are an id alone and a map of the id without a digest; the same id
//! with a digest is a ground of its own.
//!
//! - An artifact ground holds while its revision equals the case snapshot's current revision of
//!   the artifact, compared for equality, the rule revision binding (`binding.rs`) applies to a
//!   record's `subject_revision`.
//! - An evidence ground holds while a record with its id is applicable: given, left by the
//!   revision-binding and freshness stages, and listed in no claim's `excluded_evidence`
//!   ([`super::missing::applicable`], the one rule `missing_evidence` reads). An id no given
//!   record has does not hold, and is not refused.
//! - An evidence ground with a digest holds only while, besides, the record given under its id
//!   has that digest ([`super::evidence_digest`]: the SHA-256 of the record's canonical form, so
//!   the key order and whitespace of its file do not change it). A record with another digest
//!   fails the ground as `content_changed`, whether or not it is applicable: it is not the record
//!   the entry rested on, so why it would not apply says nothing about that one.
//!
//! An evidence ground without a digest is checked by its id and by the fields that decide whether
//! the record given under it is applicable (story:approvals-bound-to-evidence-content): its
//! `subject_revision`, `observed_at` and `upstream_revisions`, read by the revision-binding,
//! freshness and invalidation rules, and its `producer` where a claim demands independence of it;
//! not by the rest of its contents. Canon cannot tell then that the record given under an id now
//! is not the one an entry rested on: with other contents under the same id, still applicable, the
//! entry still applies, and the decision changes only where a claim reads the replaced contents
//! (a claim over its `result`, say). Evidence IDs must be immutable once an approval cites them by
//! id alone: whoever stores the records and passes them as `--evidence` keeps each id bound to one
//! content, and gives a changed observation a new id; an approval that gives the digest needs no
//! such promise.
//!
//! An entry applies only when every ground holds and, when it has a case revision, that revision
//! is the case snapshot's `revision`, whatever it decided. `authority.rs` and `decisions.rs` decide
//! that once, before coverage, precedence and `decided_by` read the entries ([`Standing`] says
//! which grounds hold), so the `actions` and `outcomes` sections see only the entries that apply.
//! An entry set aside by its case revision adds none of its grounds to a reason: it would not apply
//! whatever its grounds do, so naming them would ask for a ground to hold again that restores
//! nothing. `stale_grounds` lists the failed grounds of the entries every other rule would let
//! apply. An entry at its case revision (or with none) whose grounds fail is set aside by its
//! grounds, and each ground that failed is a [`Failed`].
//!
//! A reason that a set-aside entry leaves undecided lists the failed grounds of every such entry
//! under `stale_grounds` ([`stale_grounds`]), each once, artifact grounds first, by artifact and
//! then revision, then evidence grounds by id:
//!
//! - `{"artifact": <id>, "reason": "revision_mismatch", "revision": <the ground's revision>}`;
//! - `{"evidence": <id>, "present": false}` when no given record has the id;
//! - `{"evidence": <id>, "reason": <reason>}`, with the reason the record was excluded with: the
//!   one the revision-binding or freshness stage gave it, else the one the first claim listing it,
//!   in claim-id order, gives (`invalidated` or `not_independent`);
//! - `{"evidence": <id>, "reason": "content_changed"}` when a ground with a digest finds another
//!   digest on the record under its id.
//!
//! One id is listed in that order when its grounds fail in more than one way. So the order of the
//! entries in a document does not change the result. Grounds do not enter either document's
//! duplicate key.
//!
//! Refused: a ground artifact, revision or evidence id that is not an identifier, as
//! `invalid-identifier`, when the document is read ([`Grounds::read`]), and, after all three
//! supplied inputs are read, a ground artifact the protocol does not declare, as
//! `undeclared-artifact` ([`declared`]). A malformed digest, and a map with another key or without
//! `id`, are refused before, as `malformed-input`, when the entry is read.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};

use super::authority::Authority;
use super::decisions::Decisions;
use super::{Refusal, identifier};
use crate::ir::Ir;
use crate::model::{
    ArtifactId, Case, ClaimDecision, Declarations, EvidenceDigest, EvidenceExclusion,
    EvidenceGround, EvidenceId, EvidenceRecord, ExclusionReason, Revision, one_line,
};

/// What an entry rested on: artifact revisions by artifact, and evidence records by id, each with
/// the digest it gives, if any.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Grounds {
    pub(crate) revisions: BTreeMap<ArtifactId, Revision>,
    pub(crate) evidence: BTreeSet<(EvidenceId, Option<EvidenceDigest>)>,
}

impl Grounds {
    /// The grounds an entry gives, each checked as an identifier, in the order written: each
    /// artifact and then its revision, then each evidence id; `what` names the document's entries
    /// in a refusal (`authority`, `decision`).
    pub(super) fn read(
        what: &str,
        revisions: &Declarations<ArtifactId, Revision>,
        evidence: &[EvidenceGround],
    ) -> Result<Grounds, Refusal> {
        for (artifact, revision) in revisions.iter() {
            identifier(&format!("{what} ground artifact"), artifact.as_str())?;
            identifier(&format!("{what} ground revision"), revision.as_str())?;
        }
        for ground in evidence {
            identifier(&format!("{what} ground evidence"), ground.id().as_str())?;
        }
        Ok(Grounds {
            revisions: revisions
                .iter()
                .map(|(artifact, revision)| (artifact.clone(), revision.clone()))
                .collect(),
            evidence: evidence
                .iter()
                .map(|ground| (ground.id().clone(), ground.digest().cloned()))
                .collect(),
        })
    }

    /// Adds the grounds as given to an entry of `computed_from.authority` or
    /// `computed_from.decisions` (`crate::explain`): `"ground_revisions": {<artifact>: <revision>,
    /// …}` and `"ground_evidence": [<ground>, …]`, each only when not empty, so an entry without
    /// grounds is written as before. An evidence ground without a digest is written as its id, one
    /// with a digest as `{"digest": <digest>, "id": <id>}`; in id order, the id alone first, each
    /// once.
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
            let grounds: Vec<Value> = self
                .evidence
                .iter()
                .map(|(id, digest)| match digest {
                    None => Value::from(id.as_str()),
                    Some(digest) => Value::Object(
                        [
                            ("digest".to_owned(), Value::from(digest.as_str())),
                            ("id".to_owned(), Value::from(id.as_str())),
                        ]
                        .into_iter()
                        .collect(),
                    ),
                })
                .collect();
            entry.insert("ground_evidence".to_owned(), Value::Array(grounds));
        }
    }
}

/// A ground that does not hold, as `stale_grounds` lists it. Ordered as listed: artifact grounds
/// first, by artifact and then revision, then evidence grounds by id and then by [`Lapse`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Failed {
    /// An artifact ground whose revision is not the current one.
    Artifact(ArtifactId, Revision),
    /// An evidence ground, and why it does not hold.
    Evidence(EvidenceId, Lapse),
}

/// Why an evidence ground does not hold, in the order `stale_grounds` lists one id's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Lapse {
    /// No given record has the id.
    Absent,
    /// The record under the id is not applicable: excluded with the reason.
    Excluded(ExclusionReason),
    /// The record under the id has another digest than the ground gives.
    ContentChanged,
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
            Failed::Evidence(id, Lapse::Excluded(reason)) => vec![
                ("evidence", Value::from(id.as_str())),
                ("reason", Value::from(reason.as_str())),
            ],
            Failed::Evidence(id, Lapse::ContentChanged) => vec![
                ("evidence", Value::from(id.as_str())),
                ("reason", Value::from("content_changed")),
            ],
            Failed::Evidence(id, Lapse::Absent) => vec![
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
/// are applicable, the reason each other given record was excluded with, and every record given,
/// for the grounds that give a digest.
#[derive(Debug, Default)]
pub(crate) struct Standing<'a> {
    current: BTreeMap<&'a ArtifactId, &'a Revision>,
    applicable: BTreeSet<&'a EvidenceId>,
    excluded: BTreeMap<&'a EvidenceId, ExclusionReason>,
    given: &'a [EvidenceRecord],
}

impl<'a> Standing<'a> {
    /// The standing of every ground in an evaluation of `case`: `given` is every record the
    /// evidence check accepted, each id once, `left` what the revision-binding and freshness
    /// stages left, `excluded` the records they set aside with their reasons, and `claims` every
    /// claim's entry, in claim-id order, as [`super::missing::applicable`] takes them.
    pub(super) fn new(
        case: &'a Case,
        given: &'a [EvidenceRecord],
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
            given,
        }
    }

    /// The grounds of `grounds` that do not hold. Empty when every one holds.
    pub(super) fn failed(&self, grounds: &Grounds) -> BTreeSet<Failed> {
        let artifacts = grounds
            .revisions
            .iter()
            .filter(|(artifact, revision)| self.current.get(artifact) != Some(revision))
            .map(|(artifact, revision)| Failed::Artifact(artifact.clone(), revision.clone()));
        let evidence = grounds.evidence.iter().filter_map(|(id, digest)| {
            self.lapse(id, digest.as_ref())
                .map(|lapse| Failed::Evidence(id.clone(), lapse))
        });
        artifacts.chain(evidence).collect()
    }

    /// Why the evidence ground `id`, with `digest` when it gives one, does not hold; `None` when it
    /// holds. The digest is computed only for a ground that gives one, so an evaluation without
    /// digests computes none.
    fn lapse(&self, id: &EvidenceId, digest: Option<&EvidenceDigest>) -> Option<Lapse> {
        if let Some(digest) = digest {
            match self.given.iter().find(|record| {
                #[cfg(test)]
                counted(&READ);
                record.id == *id
            }) {
                None => return Some(Lapse::Absent),
                Some(record) if hashed(record) != *digest => {
                    return Some(Lapse::ContentChanged);
                }
                Some(_) => {}
            }
        }
        if self.applicable.contains(id) {
            return None;
        }
        Some(match self.excluded.get(id) {
            Some(reason) => Lapse::Excluded(*reason),
            None => Lapse::Absent,
        })
    }
}

/// The digest of `record` ([`super::evidence_digest`]), counted under `HASHED` in tests.
fn hashed(record: &EvidenceRecord) -> EvidenceDigest {
    #[cfg(test)]
    counted(&HASHED);
    super::evidence_digest(record)
}

#[cfg(test)]
thread_local! {
    /// The given records [`Standing::lapse`] read to find the record under a ground's id on this
    /// thread, for the operation-count test.
    static READ: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// The record digests [`Standing::lapse`] computed on this thread.
    static HASHED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Counts one operation under `counter`.
#[cfg(test)]
fn counted(counter: &'static std::thread::LocalKey<std::cell::Cell<usize>>) {
    counter.with(|count| count.set(count.get() + 1));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn record(id: &str) -> EvidenceRecord {
        super::super::read_evidence(&format!(
            r#"{{"format":"canon-evidence/1","id":"{id}","kind":"k","subject":"spec","subject_revision":"r1"}}"#
        ))
        .expect("reads")
    }

    fn ground(id: &str, digest: &EvidenceDigest) -> Grounds {
        Grounds {
            revisions: BTreeMap::new(),
            evidence: BTreeSet::from([(EvidenceId::from(id), Some(digest.clone()))]),
        }
    }

    /// The given records read and the digests computed while `run` runs.
    fn counted<T>(run: impl FnOnce() -> T) -> (T, usize, usize) {
        READ.with(|count| count.set(0));
        HASHED.with(|count| count.set(0));
        let value = run();
        (
            value,
            READ.with(std::cell::Cell::get),
            HASHED.with(std::cell::Cell::get),
        )
    }

    /// Wave 15 review F1: grounds with a digest stay linear in the records given. 256 records and
    /// 1,024 grounds with a digest: each record cited once with its own digest (holds), the first
    /// record cited 256 more times (holds), each cited once with another digest
    /// (`content_changed`), and 256 ids no record has (absent). Finding a ground's record reads at
    /// most one record, and each record is hashed at most once, however often it is cited: at most
    /// 1,024 records read and 256 digests. A scan of the records for each ground reads about
    /// 2 x 256 x 256 and hashes 768.
    #[test]
    fn digest_grounds_read_one_record_each_and_hash_each_record_once() {
        const N: usize = 256;
        let case = super::super::read_case(
            r#"{"format":"canon-case/1","id":"C-1","protocol":"p","artifacts":{"spec":{"revision":"r1"}}}"#,
        )
        .expect("reads");
        let records: Vec<EvidenceRecord> = (0..N).map(|i| record(&format!("e{i:03}"))).collect();
        let digests: Vec<EvidenceDigest> =
            records.iter().map(super::super::evidence_digest).collect();
        let other = EvidenceDigest::new(format!("sha256:{}", "0".repeat(64)));
        let standing = Standing::new(
            &case,
            &records,
            &records,
            &[],
            std::iter::empty::<&ClaimDecision>(),
        );
        let mut cases: Vec<(Grounds, BTreeSet<Failed>)> = Vec::new();
        for (i, digest) in digests.iter().enumerate() {
            let id = format!("e{i:03}");
            cases.push((ground(&id, digest), BTreeSet::new()));
            cases.push((ground("e000", &digests[0]), BTreeSet::new()));
            cases.push((
                ground(&id, &other),
                BTreeSet::from([Failed::Evidence(id.as_str().into(), Lapse::ContentChanged)]),
            ));
            let absent = format!("x{i:03}");
            cases.push((
                ground(&absent, digest),
                BTreeSet::from([Failed::Evidence(absent.as_str().into(), Lapse::Absent)]),
            ));
        }
        let (failed, read, hashed) = counted(|| {
            cases
                .iter()
                .map(|(grounds, _)| standing.failed(grounds))
                .collect::<Vec<_>>()
        });
        for ((grounds, expected), failed) in cases.iter().zip(&failed) {
            assert_eq!(failed, expected, "{grounds:?}");
        }
        assert!(
            read <= cases.len(),
            "{} grounds with a digest over {N} records read {read} records, more than one each",
            cases.len()
        );
        assert!(
            hashed <= N,
            "{} grounds with a digest over {N} records computed {hashed} digests, more than {N}",
            cases.len()
        );
    }
}
