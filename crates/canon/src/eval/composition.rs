//! Cases composed at run time (story:case-composition, Atlas ADR 0081): the `case_outcome` records
//! that carry each child case's outcome into its parent, and the `children` form over them.
//!
//! A record of the evidence kind `case_outcome` names one child case (`child_case`) and its outcome
//! (`result`): `open` while the child runs, otherwise its terminal outcome. The evidence check
//! (`evidence.rs`) refuses one without a child or an outcome, and a `child_case` on a record of
//! another kind; [`one_record_per_child`] then refuses two records for one child. The `children`
//! form reads every record the evidence check accepted, as the order forms read the history: a
//! child's outcome is a fact about the child case, which a later revision of the parent's
//! artifact, an expiry or an invalidation rule does not undo. A form with `revision: current`
//! (story:child-outcome-revision) reads only those whose `subject_revision` is the case snapshot's
//! current revision of their `subject`, and one with `subject` (story:child-outcome-subject) only
//! those whose `subject` is that artifact; [`read`] is that selection, the one the form's value
//! and its reason share.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Map;

use super::Refusal;
use crate::model::{
    ArtifactId, CASE_OUTCOME, CaseArtifact, CaseId, ChildOutcomes, Declarations, EvidenceRecord,
    Json, OPEN_OUTCOME, Truth, one_line,
};

/// The case snapshot's artifacts, each with its current revision: what a `children` form with
/// `revision: current` compares a record's subject revision with.
pub(crate) type Artifacts = Declarations<ArtifactId, CaseArtifact>;

/// Whether `record` carries a child case's outcome: one of the kind `case_outcome`.
pub(crate) fn is_case_outcome(record: &EvidenceRecord) -> bool {
    record.kind.as_str() == CASE_OUTCOME
}

/// The records of `history` that `form` reads, in the order given: every `case_outcome` record;
/// when the form says `revision: current`, only one whose `subject_revision` is the current
/// revision `artifacts` gives its `subject` (none when `artifacts` is `None`, or does not list the
/// subject); and when the form names a `subject`, only one about that artifact. Each key of the
/// form that selects records is one condition here.
pub(crate) fn read<'r>(
    form: &'r ChildOutcomes,
    history: &'r [EvidenceRecord],
    artifacts: Option<&'r Artifacts>,
) -> impl Iterator<Item = &'r EvidenceRecord> + 'r {
    history.iter().filter(move |record| {
        is_case_outcome(record)
            && (!form.current_only() || at_current_revision(record, artifacts))
            && form
                .subject
                .as_ref()
                .is_none_or(|subject| *subject == record.subject)
    })
}

/// Whether `record` is about the current revision of its subject, as `artifacts` gives it.
fn at_current_revision(record: &EvidenceRecord, artifacts: Option<&Artifacts>) -> bool {
    artifacts
        .and_then(|artifacts| artifacts.get(&record.subject))
        .is_some_and(|artifact| artifact.revision == record.subject_revision)
}

/// The `children` form's value over the records it reads ([`read`]) of `history`, every record
/// the evidence check accepted, with `artifacts` the case snapshot's current revisions: `unknown`
/// with no record read; `false` when a record's outcome is neither listed nor `open` (a child that
/// ended outside the set; a record without an outcome, which the evidence check refuses, counts as
/// one); otherwise `unknown` while a child is `open`, and `true` when every child ended with a
/// listed outcome.
pub(crate) fn value(
    form: &ChildOutcomes,
    history: &[EvidenceRecord],
    artifacts: Option<&Artifacts>,
) -> Truth {
    let mut any = false;
    let mut open = false;
    for record in read(form, history, artifacts) {
        any = true;
        match record.result.as_deref() {
            Some(outcome) if form.r#in.iter().any(|listed| listed == outcome) => {}
            Some(OPEN_OUTCOME) => open = true,
            _ => return Truth::False,
        }
    }
    if !any || open {
        Truth::Unknown
    } else {
        Truth::True
    }
}

/// The reason each `children` form among `forms` gives, named by its outcome list as the IR writes
/// it and by its `revision` and `subject` when the IR writes them, with its value over `history`
/// and `artifacts`: `{"children": [<outcome>, ...], "value": <value>}`, with `"revision":
/// "current"` added for a form that reads only the current revision and `"subject": <artifact>`
/// for a form that names one. Each form once, two that differ only in `revision` or only in
/// `subject` being two, in the order of their outcome lists, then no `revision` first, then no
/// `subject` first and then in subject order.
pub(crate) fn reasons<'a>(
    forms: impl IntoIterator<Item = &'a ChildOutcomes>,
    history: &[EvidenceRecord],
    artifacts: Option<&Artifacts>,
) -> Vec<Json> {
    // `any` and no key are one form, as the IR writes them.
    let forms: BTreeSet<(&[String], bool, Option<&ArtifactId>)> = forms
        .into_iter()
        .map(|form| {
            (
                form.r#in.as_slice(),
                form.current_only(),
                form.subject.as_ref(),
            )
        })
        .collect();
    forms
        .into_iter()
        .map(|(outcomes, current, subject)| {
            let form = ChildOutcomes {
                r#in: outcomes.to_vec(),
                revision: current.then(|| "current".parse().expect("a revision")),
                subject: subject.cloned(),
            };
            let listed = outcomes.iter().map(|outcome| Json::from(outcome.as_str()));
            let mut reason = Map::from_iter([
                ("children".to_owned(), listed.collect()),
                (
                    "value".to_owned(),
                    Json::from(value(&form, history, artifacts).to_string()),
                ),
            ]);
            if current {
                reason.insert("revision".to_owned(), Json::from("current"));
            }
            if let Some(subject) = subject {
                reason.insert("subject".to_owned(), Json::from(subject.as_str()));
            }
            Json::Object(reason)
        })
        .collect()
}

/// Refuses two `case_outcome` records for one child case, whatever their revisions, as
/// `duplicate-child-case`, naming the child and its records in evidence-id order and citing each.
/// Of several such children, the least child id is named, so the refusal does not depend on the
/// order the records are given in.
pub(super) fn one_record_per_child(evidence: &[EvidenceRecord]) -> Result<(), Refusal> {
    let mut by_child: BTreeMap<&CaseId, Vec<&EvidenceRecord>> = BTreeMap::new();
    for record in evidence.iter().filter(|record| is_case_outcome(record)) {
        if let Some(child) = &record.child_case {
            by_child.entry(child).or_default().push(record);
        }
    }
    let Some((child, mut records)) = by_child.into_iter().find(|(_, records)| records.len() > 1)
    else {
        return Ok(());
    };
    records.sort_by(|left, right| left.id.cmp(&right.id));
    let named: Vec<String> = records
        .iter()
        .map(|record| format!("`{}`", one_line(record.id.as_str())))
        .collect();
    let refusal = Refusal::new(
        "duplicate-child-case",
        format!(
            "child case `{}` has more than one case_outcome record: {}",
            one_line(child.as_str()),
            named.join(", ")
        ),
    );
    Err(records
        .iter()
        .fold(refusal, |refusal, record| refusal.citing(&record.id)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{EvidenceId, EvidenceKindId, Revision};

    /// The form's value over records about any revision: what every form without `revision`
    /// reads, whatever the case snapshot says.
    fn value(form: &ChildOutcomes, history: &[EvidenceRecord]) -> Truth {
        super::value(form, history, None)
    }

    /// A case snapshot's artifacts: `findings` at `revision`.
    fn findings_at(revision: &str) -> Artifacts {
        Declarations::new(vec![(
            ArtifactId::new("findings"),
            CaseArtifact {
                revision: Revision::new(revision),
                descends_from: None,
            },
        )])
    }

    fn about(revision: &str, record: EvidenceRecord) -> EvidenceRecord {
        EvidenceRecord {
            subject_revision: Revision::new(revision),
            ..record
        }
    }

    fn current(form: ChildOutcomes) -> ChildOutcomes {
        ChildOutcomes {
            revision: Some("current".parse().expect("a revision")),
            ..form
        }
    }

    /// story:child-outcome-revision: with `revision: current` the form reads only the records
    /// about the current revision of their subject; one about another revision neither
    /// establishes nor contradicts it, and with none read it is UNKNOWN. Without the key, or with
    /// `any`, it reads every record. Without the case's revisions, the bound form reads nothing.
    #[test]
    fn a_form_bound_to_the_current_revision_reads_only_records_about_it() {
        let bound = current(resolved());
        let any = ChildOutcomes {
            revision: Some("any".parse().expect("a revision")),
            ..resolved()
        };
        let at = findings_at("f2");
        let old_accepted = about("f1", outcome("o1", "c1", "accepted"));
        let old_abandoned = about("f1", outcome("o1", "c1", "abandoned"));
        let old_open = about("f1", outcome("o1", "c1", "open"));
        let new_declined = about("f2", outcome("o2", "c2", "declined"));
        let new_abandoned = about("f2", outcome("o2", "c2", "abandoned"));
        for (records, bound_value, any_value, why) in [
            (vec![], Truth::Unknown, Truth::Unknown, "no record"),
            (
                vec![old_accepted.clone()],
                Truth::Unknown,
                Truth::True,
                "only an earlier revision",
            ),
            (
                vec![old_accepted, new_declined.clone()],
                Truth::True,
                Truth::True,
                "the current revision decides",
            ),
            (
                vec![old_abandoned, new_declined.clone()],
                Truth::True,
                Truth::False,
                "an earlier abandoned child is not read",
            ),
            (
                vec![old_open, new_declined],
                Truth::True,
                Truth::Unknown,
                "an earlier open child is not read",
            ),
            (
                vec![new_abandoned],
                Truth::False,
                Truth::False,
                "a current child outside the set",
            ),
        ] {
            assert_eq!(
                super::value(&bound, &records, Some(&at)),
                bound_value,
                "bound: {why}"
            );
            assert_eq!(
                super::value(&any, &records, Some(&at)),
                any_value,
                "any: {why}"
            );
            assert_eq!(
                super::value(&resolved(), &records, Some(&at)),
                any_value,
                "no key: {why}"
            );
        }
        let at_f1 = [outcome("o1", "c1", "accepted")];
        assert_eq!(super::value(&bound, &at_f1, None), Truth::Unknown);
        assert_eq!(
            super::value(&bound, &at_f1, Some(&findings_at("f1"))),
            Truth::True
        );
    }

    /// Two forms that differ only in `revision` give two reasons, each named by its key, no key
    /// first; `any` and no key are one form, named without the key.
    #[test]
    fn reasons_name_the_form_with_its_revision() {
        let any = ChildOutcomes {
            revision: Some("any".parse().expect("a revision")),
            ..resolved()
        };
        let bound = current(resolved());
        let records = [about("f1", outcome("o1", "c1", "accepted"))];
        let reasons = super::reasons(
            [&bound, &any, &resolved()],
            &records,
            Some(&findings_at("f2")),
        );
        assert_eq!(
            Json::Array(reasons),
            serde_json::json!([
                {"children": ["accepted", "declined"], "value": "true"},
                {"children": ["accepted", "declined"], "revision": "current", "value": "unknown"},
            ])
        );
    }

    fn record(id: &str, kind: &str, child: Option<&str>, result: Option<&str>) -> EvidenceRecord {
        EvidenceRecord {
            format: "canon-evidence/1".to_owned(),
            id: EvidenceId::new(id),
            kind: EvidenceKindId::new(kind),
            result: result.map(str::to_owned),
            subject: ArtifactId::new("findings"),
            subject_revision: Revision::new("f1"),
            observed_at: None,
            upstream_revisions: Declarations::default(),
            producer: None,
            child_case: child.map(CaseId::new),
            item: None,
        }
    }

    fn outcome(id: &str, child: &str, result: &str) -> EvidenceRecord {
        record(id, CASE_OUTCOME, Some(child), Some(result))
    }

    fn resolved() -> ChildOutcomes {
        ChildOutcomes {
            r#in: vec!["accepted".to_owned(), "declined".to_owned()],
            revision: None,
            subject: None,
        }
    }

    fn about_artifact(subject: &str, revision: &str, record: EvidenceRecord) -> EvidenceRecord {
        EvidenceRecord {
            subject: ArtifactId::new(subject),
            subject_revision: Revision::new(revision),
            ..record
        }
    }

    fn naming(subject: &str, form: ChildOutcomes) -> ChildOutcomes {
        ChildOutcomes {
            subject: Some(ArtifactId::new(subject)),
            ..form
        }
    }

    /// story:child-outcome-subject: with `subject` the form reads only the records about that
    /// artifact, at whatever revision, or with `revision: current` too only those at its current
    /// revision; a record about another artifact neither establishes nor contradicts it. `findings`
    /// is at `f2` and `report` at `r1`.
    #[test]
    fn a_form_naming_a_subject_reads_only_records_about_it() {
        let at = Declarations::new(vec![
            (
                ArtifactId::new("findings"),
                CaseArtifact {
                    revision: Revision::new("f2"),
                    descends_from: None,
                },
            ),
            (
                ArtifactId::new("report"),
                CaseArtifact {
                    revision: Revision::new("r1"),
                    descends_from: None,
                },
            ),
        ]);
        let named = naming("findings", resolved());
        let named_now = current(naming("findings", resolved()));
        let old_abandoned = about_artifact("findings", "f1", outcome("o1", "c1", "abandoned"));
        let new_accepted = about_artifact("findings", "f2", outcome("o2", "c2", "accepted"));
        let report_abandoned = about_artifact("report", "r1", outcome("o3", "c3", "abandoned"));
        let report_accepted = about_artifact("report", "r1", outcome("o3", "c3", "accepted"));
        for (records, named_value, now_value, unnamed_value, why) in [
            (
                vec![report_accepted.clone()],
                Truth::Unknown,
                Truth::Unknown,
                Truth::True,
                "only a record about another artifact",
            ),
            (
                vec![new_accepted.clone(), report_abandoned.clone()],
                Truth::True,
                Truth::True,
                Truth::False,
                "another artifact's child ended outside the set",
            ),
            (
                vec![old_abandoned, new_accepted, report_accepted],
                Truth::False,
                Truth::True,
                Truth::True,
                "an earlier child of the subject ended outside the set",
            ),
        ] {
            assert_eq!(
                super::value(&named, &records, Some(&at)),
                named_value,
                "named: {why}"
            );
            assert_eq!(
                super::value(&named_now, &records, Some(&at)),
                now_value,
                "named and current: {why}"
            );
            assert_eq!(
                super::value(&current(resolved()), &records, Some(&at)),
                unnamed_value,
                "current, no subject: {why}"
            );
        }
    }

    /// Forms that differ only in `subject` give one reason each, named by it, no subject first and
    /// then in subject order, whatever the order given; one given twice is one reason.
    #[test]
    fn reasons_name_the_form_with_its_subject() {
        let records = [about_artifact(
            "report",
            "r1",
            outcome("o1", "c1", "abandoned"),
        )];
        let report = naming("report", resolved());
        let findings = naming("findings", resolved());
        let reasons = super::reasons(
            [&report, &resolved(), &findings, &report],
            &records,
            Some(&findings_at("f1")),
        );
        assert_eq!(
            Json::Array(reasons),
            serde_json::json!([
                {"children": ["accepted", "declined"], "value": "false"},
                {"children": ["accepted", "declined"], "subject": "findings", "value": "unknown"},
                {"children": ["accepted", "declined"], "subject": "report", "value": "false"},
            ])
        );
    }

    /// The story's values: UNKNOWN with no record and while a child is open, TRUE once every
    /// child ended with a listed outcome, FALSE once one ended outside the set, open or not.
    #[test]
    fn the_form_is_unknown_open_true_listed_and_false_outside() {
        let form = resolved();
        assert_eq!(value(&form, &[]), Truth::Unknown);
        assert_eq!(
            value(
                &form,
                &[outcome("o1", "c1", "accepted"), outcome("o2", "c2", "open")]
            ),
            Truth::Unknown
        );
        assert_eq!(
            value(
                &form,
                &[
                    outcome("o1", "c1", "accepted"),
                    outcome("o2", "c2", "declined")
                ]
            ),
            Truth::True
        );
        assert_eq!(
            value(
                &form,
                &[
                    outcome("o1", "c1", "accepted"),
                    outcome("o2", "c2", "abandoned")
                ]
            ),
            Truth::False
        );
        assert_eq!(
            value(
                &form,
                &[
                    outcome("o1", "c1", "open"),
                    outcome("o2", "c2", "abandoned")
                ]
            ),
            Truth::False
        );
    }

    /// Only `case_outcome` records are children: a record of another kind, whatever its result,
    /// takes no part, and alone leaves the form UNKNOWN. A record without an outcome, which the
    /// evidence check refuses, would count as a child that ended outside the set.
    #[test]
    fn only_case_outcome_records_are_children() {
        let form = resolved();
        let other = record("x1", "review", None, Some("abandoned"));
        assert_eq!(value(&form, std::slice::from_ref(&other)), Truth::Unknown);
        assert_eq!(
            value(&form, &[other, outcome("o1", "c1", "accepted")]),
            Truth::True
        );
        assert_eq!(
            value(&form, &[record("o1", CASE_OUTCOME, Some("c1"), None)]),
            Truth::False
        );
    }

    /// Two records for one child are refused naming the child and both records, in id order,
    /// citing each; of two such children, the least is named, whatever the order given.
    #[test]
    fn two_records_for_one_child_are_refused_naming_it() {
        assert_eq!(
            one_record_per_child(&[outcome("o1", "c1", "open"), outcome("o2", "c2", "open")]),
            Ok(())
        );
        let mut records = vec![
            outcome("o9", "c2", "open"),
            outcome("o4", "c2", "accepted"),
            outcome("o3", "c1", "accepted"),
            outcome("o1", "c1", "open"),
        ];
        for _ in 0..2 {
            let refusal = one_record_per_child(&records).expect_err("refused");
            assert_eq!(refusal.code(), "duplicate-child-case");
            assert_eq!(
                refusal.to_string(),
                "child case `c1` has more than one case_outcome record: `o1`, `o3`"
            );
            assert_eq!(
                refusal.evidence(),
                &[EvidenceId::new("o1"), EvidenceId::new("o3")]
            );
            records.reverse();
        }
    }
}
