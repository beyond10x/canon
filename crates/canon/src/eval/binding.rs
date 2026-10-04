//! The revision-binding exclusion stage (design § 9): evidence applies only to the revision of its
//! subject it is bound to.
//!
//! Each record names one subject artifact and the revision of it the record concerns; the case
//! snapshot names the current revision of every declared artifact. The records are checked in the
//! order given: a record whose subject the protocol does not declare is refused as
//! `undeclared-artifact`, naming the record and its subject. A record whose subject revision is
//! not the subject's current revision is excluded with the reason `revision_mismatch`: claims are
//! evaluated as if it had not been given. An evidence match that only that record satisfied or
//! contradicted becomes `unknown`, never `false`. A claim that tests whether another claim
//! `is: unknown` is decided by that `unknown`, so excluding a record can turn such a claim `true`
//! or `false`.

use super::Refusal;
use crate::ir::Ir;
use crate::model::{Case, EvidenceExclusion, EvidenceRecord, ExclusionReason, one_line};

/// The records of `evidence` this stage keeps from claim evaluation, each with the reason
/// `revision_mismatch`. `evidence` is what earlier stages left.
pub(super) fn exclude(
    ir: &Ir,
    case: &Case,
    evidence: &[&EvidenceRecord],
) -> Result<Vec<EvidenceExclusion>, Refusal> {
    let mut excluded = Vec::new();
    for record in evidence {
        if !ir.artifacts.contains_key(&record.subject) {
            return Err(Refusal::new(
                "undeclared-artifact",
                format!(
                    "evidence `{}` is about artifact `{}`, which the protocol does not declare",
                    one_line(record.id.as_str()),
                    one_line(record.subject.as_str())
                ),
            ));
        }
        let current = case
            .artifacts
            .get(&record.subject)
            .map(|artifact| &artifact.revision);
        if current != Some(&record.subject_revision) {
            excluded.push(EvidenceExclusion {
                evidence: record.id.clone(),
                reason: ExclusionReason::RevisionMismatch,
            });
        }
    }
    Ok(excluded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ArtifactId, EVIDENCE_FORMAT, EvidenceId, EvidenceKindId, Revision};

    fn ir() -> Ir {
        crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
                 artifacts: {a: {}, b: {}}\nevidence_kinds: {k: {}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles")
    }

    fn case() -> Case {
        super::super::read_case(
            "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r2}, b: {revision: r1}}\n",
        )
        .expect("case reads")
    }

    fn record(id: &str, subject: &str, revision: &str) -> EvidenceRecord {
        EvidenceRecord {
            format: EVIDENCE_FORMAT.to_owned(),
            id: EvidenceId::new(id),
            kind: EvidenceKindId::new("k"),
            result: None,
            subject: ArtifactId::new(subject),
            subject_revision: Revision::new(revision),
            observed_at: None,
        }
    }

    fn excluded(records: &[EvidenceRecord]) -> Result<Vec<String>, Refusal> {
        let refs: Vec<&EvidenceRecord> = records.iter().collect();
        Ok(exclude(&ir(), &case(), &refs)?
            .into_iter()
            .map(|exclusion| {
                assert_eq!(exclusion.reason, ExclusionReason::RevisionMismatch);
                exclusion.evidence.as_str().to_owned()
            })
            .collect())
    }

    /// Only records bound to a revision that is not their subject's current one are excluded,
    /// each artifact checked against its own current revision.
    #[test]
    fn a_record_bound_to_a_revision_that_is_not_current_is_excluded() {
        let records = [
            record("current-a", "a", "r2"),
            record("stale-a", "a", "r1"),
            record("current-b", "b", "r1"),
            record("future-b", "b", "r2"),
        ];
        assert_eq!(
            excluded(&records).expect("no refusal"),
            ["stale-a", "future-b"]
        );
        assert_eq!(excluded(&[]).expect("no refusal"), Vec::<String>::new());
    }

    /// The first record, in the order given, whose subject is not declared is refused, naming
    /// the record and its subject.
    #[test]
    fn a_record_about_an_undeclared_artifact_is_refused_naming_it() {
        let records = [
            record("stale-a", "a", "r1"),
            record("e1", "nowhere", "r1"),
            record("e2", "elsewhere", "r1"),
        ];
        let refusal = excluded(&records).expect_err("refused");
        assert_eq!(refusal.code(), "undeclared-artifact");
        assert_eq!(
            refusal.to_string(),
            "evidence `e1` is about artifact `nowhere`, which the protocol does not declare"
        );
    }
}
