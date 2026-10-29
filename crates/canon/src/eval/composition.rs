//! Cases composed at run time (story:case-composition, Atlas ADR 0081): the `case_outcome` records
//! that carry each child case's outcome into its parent, and the `children` form over them.
//!
//! A record of the evidence kind `case_outcome` names one child case (`child_case`) and its outcome
//! (`result`): `open` while the child runs, otherwise its terminal outcome. The evidence check
//! (`evidence.rs`) refuses one without a child or an outcome, and a `child_case` on a record of
//! another kind; [`one_record_per_child`] then refuses two records for one child. The `children`
//! form reads every record the evidence check accepted, as the order forms read the history: a
//! child's outcome is a fact about the child case, which a later revision of the parent's
//! artifact, an expiry or an invalidation rule does not undo.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Map;

use super::Refusal;
use crate::model::{
    CASE_OUTCOME, CaseId, ChildOutcomes, EvidenceRecord, Json, OPEN_OUTCOME, Truth, one_line,
};

/// Whether `record` carries a child case's outcome: one of the kind `case_outcome`.
pub(crate) fn is_case_outcome(record: &EvidenceRecord) -> bool {
    record.kind.as_str() == CASE_OUTCOME
}

/// The `children` form's value over `history`, every record the evidence check accepted: `unknown`
/// with no `case_outcome` record; `false` when a record's outcome is neither listed nor `open` (a
/// child that ended outside the set; a record without an outcome, which the evidence check
/// refuses, counts as one); otherwise `unknown` while a child is `open`, and `true` when every
/// child ended with a listed outcome.
pub(crate) fn value(form: &ChildOutcomes, history: &[EvidenceRecord]) -> Truth {
    let mut any = false;
    let mut open = false;
    for record in history.iter().filter(|record| is_case_outcome(record)) {
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
/// it, with its value over `history`: `{"children": [<outcome>, ...], "value": <value>}`, each
/// form once, in the order of those lists.
pub(crate) fn reasons<'a>(
    forms: impl IntoIterator<Item = &'a [String]>,
    history: &[EvidenceRecord],
) -> Vec<Json> {
    let forms: BTreeSet<&[String]> = forms.into_iter().collect();
    forms
        .into_iter()
        .map(|outcomes| {
            let form = ChildOutcomes {
                r#in: outcomes.to_vec(),
            };
            let listed = outcomes.iter().map(|outcome| Json::from(outcome.as_str()));
            Json::Object(Map::from_iter([
                ("children".to_owned(), listed.collect()),
                (
                    "value".to_owned(),
                    Json::from(value(&form, history).to_string()),
                ),
            ]))
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
    use crate::model::{ArtifactId, Declarations, EvidenceId, EvidenceKindId, Revision};

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
        }
    }

    fn outcome(id: &str, child: &str, result: &str) -> EvidenceRecord {
        record(id, CASE_OUTCOME, Some(child), Some(result))
    }

    fn resolved() -> ChildOutcomes {
        ChildOutcomes {
            r#in: vec!["accepted".to_owned(), "declined".to_owned()],
        }
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
