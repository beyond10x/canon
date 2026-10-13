//! The order forms of the predicate (story:evidence-order-predicate, CANON-ORDER-001): whether the
//! first record of a kind has a result (`first`), and whether the first record of one kind was
//! observed strictly before the first record of another (`before`).
//!
//! An order form reads the history: every record of its kind (about its `subject` when it names
//! one, by the rule evidence matches read by, [`super::reads`]) that the evidence check accepted,
//! whatever the exclusion stages decide. A record excluded as `revision_mismatch` or `expired`, or
//! invalidated for the claim being evaluated, is read: the exclusion stages say what holds of the
//! current revisions now, while order is a fact about what was observed and when. A record's place
//! in the order is its `observed_at`, compared in whole seconds; a record without one has no known
//! place. No record is listed in a claim's `excluded_evidence` because of an order form.
//!
//! `{first: {kind: K, result: R}}`: `unknown` with no record, and with a record that has no
//! `observed_at`; otherwise, of the records at the earliest instant, `true` when every one has
//! result `R`, `false` when none has it and `unknown` when some have it and some do not.
//!
//! `{before: {first: K1, then: K2}}`: `unknown` with no record of either kind; `false` with a
//! record of `K2` and none of `K1`, since a record of `K1` that comes later comes after it;
//! `unknown` with a record of `K1` and none of `K2`; with records of both, `unknown` when one of
//! them has no `observed_at`, and otherwise `true`, `false` or `unknown` as the earliest instant
//! of `K1` is earlier than, later than or equal to that of `K2`.
//!
//! A record bound to a revision of its subject that does not descend from the current one counts
//! like any other: Canon does not know how revisions descend (story:revision-ordering).

use serde_json::{Map, Value};

use super::claims::reads;
use crate::model::{
    ArtifactId, EvidenceKindId, EvidenceOrder, EvidenceRecord, FirstResult, Predicate, Truth,
};

/// The value of an order form over the `history`; `None` for any other form.
pub(crate) fn value(predicate: &Predicate, history: &[EvidenceRecord]) -> Option<Truth> {
    match predicate {
        Predicate::First(form) => Some(first(form, history)),
        Predicate::Before(form) => Some(before(form, history)),
        _ => None,
    }
}

/// The records of `history` of `kind`, about `subject` when it names one.
fn of<'r>(
    kind: &EvidenceKindId,
    subject: Option<&ArtifactId>,
    history: &'r [EvidenceRecord],
) -> Vec<&'r EvidenceRecord> {
    history
        .iter()
        .filter(|record| reads(kind, subject, record))
        .collect()
}

/// Where a record stands in the order: its `observed_at` in seconds, or `None` when it has none.
/// The evidence check has refused an `observed_at` that is not an instant; one in a set a caller
/// builds is treated as missing.
fn instant(record: &EvidenceRecord) -> Option<i64> {
    record.observed_at.as_ref()?.seconds()
}

/// The earliest instant of `records`, or `None` when one of them has no known place. `records` is
/// not empty.
fn earliest(records: &[&EvidenceRecord]) -> Option<i64> {
    records
        .iter()
        .map(|record| instant(record))
        .collect::<Option<Vec<i64>>>()?
        .into_iter()
        .min()
}

fn first(form: &FirstResult, history: &[EvidenceRecord]) -> Truth {
    let records = of(&form.kind, form.subject.as_ref(), history);
    if records.is_empty() {
        return Truth::Unknown;
    }
    let Some(at) = earliest(&records) else {
        return Truth::Unknown;
    };
    let (with, without) = records
        .iter()
        .filter(|record| instant(record) == Some(at))
        .fold((0usize, 0usize), |(with, without), record| {
            if record.result.as_deref() == Some(form.result.as_str()) {
                (with + 1, without)
            } else {
                (with, without + 1)
            }
        });
    match (with, without) {
        (_, 0) => Truth::True,
        (0, _) => Truth::False,
        _ => Truth::Unknown,
    }
}

fn before(form: &EvidenceOrder, history: &[EvidenceRecord]) -> Truth {
    let firsts = of(&form.first, form.subject.as_ref(), history);
    let thens = of(&form.then, form.subject.as_ref(), history);
    match (firsts.is_empty(), thens.is_empty()) {
        (true, true) | (false, true) => Truth::Unknown,
        (true, false) => Truth::False,
        (false, false) => match (earliest(&firsts), earliest(&thens)) {
            (Some(first), Some(then)) if first < then => Truth::True,
            (Some(first), Some(then)) if first > then => Truth::False,
            _ => Truth::Unknown,
        },
    }
}

/// How the reasons order the order forms: every `before` first, then every `first`, each by its
/// fields as text (no subject before any subject).
type Key<'a> = (u8, &'a str, &'a str, Option<&'a str>);

fn key(predicate: &Predicate) -> Option<Key<'_>> {
    match predicate {
        Predicate::Before(form) => Some((
            0,
            form.first.as_str(),
            form.then.as_str(),
            form.subject.as_ref().map(ArtifactId::as_str),
        )),
        Predicate::First(form) => Some((
            1,
            form.kind.as_str(),
            form.result.as_str(),
            form.subject.as_ref().map(ArtifactId::as_str),
        )),
        _ => None,
    }
}

/// The reason each of `forms` gives, with its value over the `history`, each form once, `before`
/// forms first: `{"before": K1, "then": K2, "value": <truth>}` or `{"first": K, "result": R,
/// "value": <truth>}`, with `"subject": A` added when the form names one. What is not an order form
/// is skipped.
pub(crate) fn reasons<'a>(
    forms: impl IntoIterator<Item = &'a Predicate>,
    history: &[EvidenceRecord],
) -> Vec<Value> {
    let mut keyed: Vec<(Key<'a>, &'a Predicate)> = forms
        .into_iter()
        .filter_map(|form| Some((key(form)?, form)))
        .collect();
    keyed.sort_by(|left, right| left.0.cmp(&right.0));
    keyed.dedup_by(|later, earlier| later.0 == earlier.0);
    keyed
        .into_iter()
        .filter_map(|(_, form)| {
            let truth = value(form, history)?.to_string();
            let (subject, mut entry) = match form {
                Predicate::Before(form) => (
                    form.subject.as_ref(),
                    Map::from_iter([
                        ("before".to_owned(), Value::from(form.first.as_str())),
                        ("then".to_owned(), Value::from(form.then.as_str())),
                    ]),
                ),
                Predicate::First(form) => (
                    form.subject.as_ref(),
                    Map::from_iter([
                        ("first".to_owned(), Value::from(form.kind.as_str())),
                        ("result".to_owned(), Value::from(form.result.as_str())),
                    ]),
                ),
                _ => return None,
            };
            if let Some(subject) = subject {
                entry.insert("subject".to_owned(), Value::from(subject.as_str()));
            }
            entry.insert("value".to_owned(), Value::from(truth));
            Some(Value::Object(entry))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Declarations, EvidenceId, Instant, Revision};

    fn record(
        id: &str,
        kind: &str,
        result: Option<&str>,
        subject: &str,
        at: Option<&str>,
    ) -> EvidenceRecord {
        EvidenceRecord {
            format: crate::model::EVIDENCE_FORMAT.to_owned(),
            id: EvidenceId::new(id),
            kind: EvidenceKindId::new(kind),
            result: result.map(str::to_owned),
            subject: ArtifactId::new(subject),
            subject_revision: Revision::new("r1"),
            observed_at: at.map(Instant::new),
            upstream_revisions: Declarations::default(),
        }
    }

    fn first_of(kind: &str, result: &str, subject: Option<&str>) -> Predicate {
        Predicate::First(FirstResult {
            kind: EvidenceKindId::new(kind),
            result: result.to_owned(),
            subject: subject.map(ArtifactId::new),
        })
    }

    fn before_of(first: &str, then: &str, subject: Option<&str>) -> Predicate {
        Predicate::Before(EvidenceOrder {
            first: EvidenceKindId::new(first),
            then: EvidenceKindId::new(then),
            subject: subject.map(ArtifactId::new),
        })
    }

    const T1: Option<&str> = Some("2026-10-01T09:00:00Z");
    const T2: Option<&str> = Some("2026-10-01T09:00:01Z");

    /// Every rule of `first`: no record, a record without an instant (even a later one), and the
    /// records at the earliest instant all, none or some with the result.
    #[test]
    fn first_reads_the_records_at_the_earliest_instant() {
        use Truth::{False as F, True as T, Unknown as U};
        let form = first_of("k", "failed", None);
        let red = |id, at| record(id, "k", Some("failed"), "a", at);
        let green = |id, at| record(id, "k", Some("passed"), "a", at);
        for (history, expected, why) in [
            (vec![], U, "no record"),
            (
                vec![record("x", "m", Some("failed"), "a", T1)],
                U,
                "no record of the kind",
            ),
            (vec![red("r", T1)], T, "red first"),
            (vec![green("g", T1), red("r", T2)], F, "green first"),
            (
                vec![red("r", T1), green("g", T2)],
                T,
                "red first, green later",
            ),
            (
                vec![red("r", T1), green("g", T1)],
                U,
                "red and green at one instant",
            ),
            (
                vec![red("r", T1), green("g", None)],
                U,
                "a record without an instant",
            ),
            (
                vec![record("n", "k", None, "a", T1)],
                F,
                "no result is not the result",
            ),
        ] {
            assert_eq!(value(&form, &history), Some(expected), "{why}");
        }
    }

    /// Every rule of `before`, including a `then` record without an instant and no `first` record,
    /// which is still `false`.
    #[test]
    fn before_compares_the_earliest_instants_of_two_kinds() {
        use Truth::{False as F, True as T, Unknown as U};
        let form = before_of("k", "m", None);
        let k = |id, at| record(id, "k", None, "a", at);
        let m = |id, at| record(id, "m", None, "a", at);
        for (history, expected, why) in [
            (vec![], U, "neither"),
            (vec![k("k1", T1)], U, "only the first kind"),
            (vec![m("m1", T1)], F, "only the second kind"),
            (
                vec![m("m1", None)],
                F,
                "only the second kind, without an instant",
            ),
            (vec![k("k1", T1), m("m1", T2)], T, "earlier"),
            (vec![k("k1", T2), m("m1", T1)], F, "later"),
            (vec![k("k1", T1), m("m1", T1)], U, "equal"),
            (
                vec![k("k1", T1), k("k2", None), m("m1", T2)],
                U,
                "a first record unplaced",
            ),
            (
                vec![k("k1", T1), m("m1", T2), m("m2", None)],
                U,
                "a then record unplaced",
            ),
            (
                vec![k("k1", T2), k("k0", T1), m("m1", T2)],
                T,
                "the earliest of each counts",
            ),
        ] {
            assert_eq!(value(&form, &history), Some(expected), "{why}");
        }
    }

    /// A form that names a subject reads only records about it, of both kinds for `before`.
    #[test]
    fn a_subject_restricts_what_an_order_form_reads() {
        let history = [
            record("g", "k", Some("passed"), "b", T1),
            record("r", "k", Some("failed"), "a", T2),
            record("m", "m", None, "b", T1),
        ];
        assert_eq!(
            value(&first_of("k", "failed", Some("a")), &history),
            Some(Truth::True)
        );
        assert_eq!(
            value(&first_of("k", "failed", None), &history),
            Some(Truth::False)
        );
        assert_eq!(
            value(&before_of("k", "m", Some("a")), &history),
            Some(Truth::Unknown)
        );
        assert_eq!(
            value(&before_of("k", "m", None), &history),
            Some(Truth::Unknown)
        );
    }

    /// Reasons come `before` first, each form once, with its subject when it names one.
    #[test]
    fn reasons_are_in_a_fixed_order_and_name_each_form_once() {
        let forms = [
            first_of("k", "failed", Some("a")),
            before_of("k", "m", None),
            first_of("k", "failed", Some("a")),
        ];
        let history = [record("r", "k", Some("failed"), "a", T1)];
        assert_eq!(
            reasons(&forms, &history),
            [
                serde_json::json!({"before": "k", "then": "m", "value": "unknown"}),
                serde_json::json!({"first": "k", "result": "failed", "subject": "a", "value": "true"}),
            ]
        );
    }
}
