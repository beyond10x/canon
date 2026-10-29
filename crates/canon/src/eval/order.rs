//! The order forms of the predicate (story:evidence-order-predicate, CANON-ORDER-001,
//! story:evidence-order-after, CANON-ORDER-002, and story:evidence-order-since, CANON-ORDER-004):
//! whether the first record of a kind has a result (`first`), whether the first record of one kind
//! was observed strictly before the first record of another (`before`), whether a record of one
//! kind with a result was observed strictly after the first record of another (`after`), and
//! whether a record of one kind with a result, at the subject revision of the latest record of
//! another, was observed after that record (`since`).
//!
//! An order form reads the history: every record of its kind (about its `subject` when it names
//! one, by the rule evidence matches read by, [`super::reads`]) that the evidence check accepted,
//! whatever the exclusion stages decide. A record excluded as `revision_mismatch` or `expired`, or
//! invalidated for the claim being evaluated, is read: the exclusion stages say what holds of the
//! current revisions now, while order is a fact about what was observed and when. A record's place
//! in the order is the interval its `observed_at` names, at the resolution it is written in
//! (story:evidence-order-subsecond, CANON-ORDER-003): `12:00:00Z` is the second
//! `[12:00:00, 12:00:01)` and `12:00:00.4Z` is `[12:00:00.4, 12:00:00.5)`. A record is observed
//! before another when its interval ends at or before the other's begins; two records whose
//! intervals overlap are not ordered: equal instants, `12:00:00.4Z` beside `12:00:00.40Z`, and
//! `12:00:00Z` beside `12:00:00.4Z`. Over whole seconds this is the order of the seconds, with
//! records in one second unordered. A record without `observed_at` has no known place. No record
//! is listed in a claim's `excluded_evidence` because of an order form.
//!
//! `{first: {kind: K, result: R}}`: `unknown` with no record, and with a record that has no
//! `observed_at`; otherwise, of the records that may be first (those no record of the kind is
//! observed before), `true` when every one has result `R`, `false` when none has it and `unknown`
//! when some have it and some do not.
//!
//! `{before: {first: K1, then: K2}}`: `unknown` with no record of either kind; `false` with a
//! record of `K2` and none of `K1`, since a record of `K1` that comes later comes after it;
//! `unknown` with a record of `K1` and none of `K2`; with records of both, `unknown` when one of
//! them has no `observed_at`, and otherwise `true` when some record of `K1` is observed before
//! every record of `K2`, `false` when some record of `K2` is observed before every record of `K1`,
//! and `unknown` when neither is.
//!
//! `{after: {first: K1, then: {kind: K2, result: R}}}` compares the records of `K1` with the
//! candidates, the records of `K2` that have result `R`; records of `K2` with another result, or
//! none, take no part. `unknown` with no record of `K1`; `false` with a record of `K1` and no
//! candidate; otherwise `unknown` when a compared record has no `observed_at`, `true` when some
//! candidate is observed after some record of `K1` (the first record of `K1` is no later than that
//! one), `false` when every candidate is observed before every record of `K1` other than itself,
//! and `unknown` otherwise. With `K1` and `K2` the same kind, a record is not later than itself:
//! one record of the kind with `R` and no other is `false`, not `unknown`.
//!
//! `{since: {last: K1, then: {kind: K2, result: R}}}` compares the anchors, the records of `K1`,
//! with the candidates, the records of `K2` that have result `R`. The latest anchors are those no
//! anchor is observed after. `unknown` with no anchor; `false` with an anchor and no candidate;
//! otherwise `unknown` when an anchor or a candidate has no `observed_at`. Otherwise each latest
//! anchor reads the candidates about its subject at its subject revision, other than itself:
//! `true` when one is observed after it, `false` when every one is observed before it or there is
//! none, and `unknown` otherwise. The form is `true` when every latest anchor gives `true`, `false`
//! when every one gives `false`, and `unknown` otherwise. With `K1` and `K2` the same kind, a
//! record is not later than itself, and no candidate is observed after the latest anchor, so the
//! form is never `true`.
//!
//! A record bound to a revision of its subject that does not descend from the current one counts
//! like any other: Canon does not know how revisions descend (story:revision-ordering). For the
//! same reason, a `since` candidate at a revision descended from its anchor's does not count.

use serde_json::{Map, Value};

use super::claims::reads;
use crate::model::{
    ArtifactId, EvidenceAfter, EvidenceKindId, EvidenceOrder, EvidenceRecord, EvidenceSince,
    FirstResult, Predicate, Truth,
};

/// The value of an order form over the `history`; `None` for any other form.
pub(crate) fn value(predicate: &Predicate, history: &[EvidenceRecord]) -> Option<Truth> {
    match predicate {
        Predicate::First(form) => Some(first(form, history)),
        Predicate::Before(form) => Some(before(form, history)),
        Predicate::After(form) => Some(after(form, history)),
        Predicate::Since(form) => Some(since(form, history)),
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

/// The interval a record's `observed_at` names, in nanoseconds, from its start (inclusive) to its
/// end (exclusive) ([`crate::model::Instant::interval`]).
type Interval = (i128, i128);

/// Where a record stands in the order: the interval its `observed_at` names, or `None` when it has
/// none. The evidence check has refused an `observed_at` that is not an instant; one in a set a
/// caller builds is treated as missing.
fn interval(record: &EvidenceRecord) -> Option<Interval> {
    record.observed_at.as_ref()?.interval()
}

/// The intervals of `records`, in their order, or `None` when one of them has no known place.
fn intervals(records: &[&EvidenceRecord]) -> Option<Vec<Interval>> {
    records.iter().map(|record| interval(record)).collect()
}

/// Whether a record at `earlier` is observed before one at `later`: its interval ends at or
/// before the other's begins. Overlapping intervals, equal ones included, are not ordered, and an
/// interval is never before itself.
fn precedes(earlier: Interval, later: Interval) -> bool {
    earlier.1 <= later.0
}

/// The earliest end of `intervals`, which is not empty: a record ending there is observed before
/// every record beginning at or after it.
fn earliest_end(intervals: &[Interval]) -> i128 {
    intervals
        .iter()
        .map(|interval| interval.1)
        .min()
        .unwrap_or(i128::MAX)
}

/// The earliest start of `intervals`, which is not empty.
fn earliest_start(intervals: &[Interval]) -> i128 {
    intervals
        .iter()
        .map(|interval| interval.0)
        .min()
        .unwrap_or(i128::MAX)
}

fn first(form: &FirstResult, history: &[EvidenceRecord]) -> Truth {
    let records = of(&form.kind, form.subject.as_ref(), history);
    if records.is_empty() {
        return Truth::Unknown;
    }
    let Some(placed) = intervals(&records) else {
        return Truth::Unknown;
    };
    // A record may be first when no record of the kind is observed before it, that is when it
    // begins before every record of the kind ends.
    let end = earliest_end(&placed);
    let (with, without) = records
        .iter()
        .zip(&placed)
        .filter(|(_, interval)| interval.0 < end)
        .fold((0usize, 0usize), |(with, without), (record, _)| {
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
        (false, false) => match (intervals(&firsts), intervals(&thens)) {
            // Some record of `first` is observed before every record of `then`.
            (Some(first), Some(then)) if earliest_end(&first) <= earliest_start(&then) => {
                Truth::True
            }
            // Some record of `then` is observed before every record of `first`.
            (Some(first), Some(then)) if earliest_end(&then) <= earliest_start(&first) => {
                Truth::False
            }
            _ => Truth::Unknown,
        },
    }
}

fn after(form: &EvidenceAfter, history: &[EvidenceRecord]) -> Truth {
    let firsts = of(&form.first, form.subject.as_ref(), history);
    if firsts.is_empty() {
        return Truth::Unknown;
    }
    let laters: Vec<&EvidenceRecord> = of(&form.then.kind, form.subject.as_ref(), history)
        .into_iter()
        .filter(|record| record.result.as_deref() == Some(form.then.result.as_str()))
        .collect();
    if laters.is_empty() {
        return Truth::False;
    }
    let Some(first_intervals) = intervals(&firsts) else {
        return Truth::Unknown;
    };
    let Some(later_intervals) = intervals(&laters) else {
        return Truth::Unknown;
    };
    // Some candidate is observed after some record of the first kind, and so after the first one.
    let end = earliest_end(&first_intervals);
    if later_intervals.iter().any(|later| end <= later.0) {
        return Truth::True;
    }
    // Every candidate is observed before every record of the first kind other than itself: a
    // record is not later than itself.
    let before_every_other = |candidate: &EvidenceRecord, at: Interval| {
        firsts
            .iter()
            .zip(&first_intervals)
            .all(|(first, first_at)| std::ptr::eq(*first, candidate) || precedes(at, *first_at))
    };
    if laters
        .iter()
        .zip(&later_intervals)
        .all(|(candidate, at)| before_every_other(candidate, *at))
    {
        Truth::False
    } else {
        Truth::Unknown
    }
}

fn since(form: &EvidenceSince, history: &[EvidenceRecord]) -> Truth {
    let anchors = of(&form.last, form.subject.as_ref(), history);
    if anchors.is_empty() {
        return Truth::Unknown;
    }
    let candidates: Vec<&EvidenceRecord> = of(&form.then.kind, form.subject.as_ref(), history)
        .into_iter()
        .filter(|record| record.result.as_deref() == Some(form.then.result.as_str()))
        .collect();
    if candidates.is_empty() {
        return Truth::False;
    }
    let Some(anchor_intervals) = intervals(&anchors) else {
        return Truth::Unknown;
    };
    let Some(candidate_intervals) = intervals(&candidates) else {
        return Truth::Unknown;
    };
    // What one anchor gives: over the candidates at its subject and subject revision, other than
    // itself (a record is not later than itself).
    let from = |anchor: &EvidenceRecord, at: Interval| {
        let mut at_its_revision = candidates
            .iter()
            .zip(&candidate_intervals)
            .filter(|(candidate, _)| {
                !std::ptr::eq(**candidate, anchor)
                    && candidate.subject == anchor.subject
                    && candidate.subject_revision == anchor.subject_revision
            })
            .map(|(_, candidate_at)| *candidate_at);
        if at_its_revision
            .clone()
            .any(|candidate_at| precedes(at, candidate_at))
        {
            Truth::True
        } else if at_its_revision.all(|candidate_at| precedes(candidate_at, at)) {
            Truth::False
        } else {
            Truth::Unknown
        }
    };
    // The latest anchors: those no anchor is observed after. Intervals are a strict order, so
    // there is at least one.
    let mut given = anchors
        .iter()
        .zip(&anchor_intervals)
        .filter(|(_, at)| !anchor_intervals.iter().any(|other| precedes(**at, *other)))
        .map(|(anchor, at)| from(anchor, *at));
    let Some(one) = given.next() else {
        return Truth::Unknown;
    };
    if one != Truth::Unknown && given.all(|other| other == one) {
        one
    } else {
        Truth::Unknown
    }
}

/// How the reasons order the order forms: every `after` first, then every `before`, then every
/// `first`, then every `since`, each by its fields as text (no subject before any subject).
type Key<'a> = (u8, &'a str, &'a str, &'a str, Option<&'a str>);

fn key(predicate: &Predicate) -> Option<Key<'_>> {
    match predicate {
        Predicate::After(form) => Some((
            0,
            form.first.as_str(),
            form.then.kind.as_str(),
            form.then.result.as_str(),
            form.subject.as_ref().map(ArtifactId::as_str),
        )),
        Predicate::Before(form) => Some((
            1,
            form.first.as_str(),
            form.then.as_str(),
            "",
            form.subject.as_ref().map(ArtifactId::as_str),
        )),
        Predicate::First(form) => Some((
            2,
            form.kind.as_str(),
            form.result.as_str(),
            "",
            form.subject.as_ref().map(ArtifactId::as_str),
        )),
        Predicate::Since(form) => Some((
            3,
            form.last.as_str(),
            form.then.kind.as_str(),
            form.then.result.as_str(),
            form.subject.as_ref().map(ArtifactId::as_str),
        )),
        _ => None,
    }
}

/// The reason each of `forms` gives, with its value over the `history`, each form once, `after`
/// forms first, then `before`, then `first`, then `since` forms: `{"after": K1, "then": K2,
/// "result": R, "value": <truth>}`, `{"before": K1, "then": K2, "value": <truth>}`, `{"first": K,
/// "result": R, "value": <truth>}` or `{"since": K1, "then": K2, "result": R, "value": <truth>}`,
/// with `"subject": A` added when the form names one. What is not an order form is skipped.
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
                Predicate::After(form) => (
                    form.subject.as_ref(),
                    Map::from_iter([
                        ("after".to_owned(), Value::from(form.first.as_str())),
                        ("then".to_owned(), Value::from(form.then.kind.as_str())),
                        ("result".to_owned(), Value::from(form.then.result.as_str())),
                    ]),
                ),
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
                Predicate::Since(form) => (
                    form.subject.as_ref(),
                    Map::from_iter([
                        ("since".to_owned(), Value::from(form.last.as_str())),
                        ("then".to_owned(), Value::from(form.then.kind.as_str())),
                        ("result".to_owned(), Value::from(form.then.result.as_str())),
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
            producer: None,
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

    fn after_of(first: &str, then: &str, result: &str, subject: Option<&str>) -> Predicate {
        Predicate::After(EvidenceAfter {
            first: EvidenceKindId::new(first),
            then: crate::model::LaterRecord {
                kind: EvidenceKindId::new(then),
                result: result.to_owned(),
            },
            subject: subject.map(ArtifactId::new),
        })
    }

    const T0: Option<&str> = Some("2026-10-01T08:59:59Z");

    /// Every rule of `after` over two kinds: no first record, no later record with the result,
    /// records without an instant (and one of another result, which takes no part), earlier,
    /// equal and later instants, and the earliest record of the first kind counting.
    #[test]
    fn after_compares_later_records_with_the_first_record_of_a_kind() {
        use Truth::{False as F, True as T, Unknown as U};
        let form = after_of("k", "m", "failed", None);
        let k = |id, at| record(id, "k", None, "a", at);
        let red = |id, at| record(id, "m", Some("failed"), "a", at);
        let green = |id, at| record(id, "m", Some("passed"), "a", at);
        for (history, expected, why) in [
            (vec![], U, "nothing"),
            (vec![red("r", T2)], U, "no first record"),
            (vec![k("k", T1)], F, "no later record"),
            (vec![k("k", None)], F, "no later record, the first unplaced"),
            (vec![k("k", T1), green("g", T2)], F, "only another result"),
            (vec![k("k", T1), red("r", T2)], T, "later"),
            (vec![k("k", T1), red("r", T0)], F, "earlier"),
            (vec![k("k", T1), red("r", T1)], U, "equal"),
            (vec![k("k", T1), red("r", T0), red("s", T2)], T, "one later"),
            (
                vec![k("k", T1), red("r", T1), red("s", T2)],
                T,
                "equal and later",
            ),
            (
                vec![k("k", T1), red("r", T0), red("s", T1)],
                U,
                "earlier and equal",
            ),
            (
                vec![k("k", None), red("r", T2)],
                U,
                "a first record unplaced",
            ),
            (
                vec![k("k", T1), k("j", None), red("r", T2)],
                U,
                "one first record unplaced",
            ),
            (
                vec![k("k", T1), red("r", None)],
                U,
                "a later record unplaced",
            ),
            (
                vec![k("k", T1), red("r", T2), green("g", None)],
                T,
                "another result unplaced takes no part",
            ),
            (
                vec![k("k", T2), k("j", T0), red("r", T1)],
                T,
                "the earliest first record counts",
            ),
        ] {
            assert_eq!(value(&form, &history), Some(expected), "{why}");
        }
    }

    /// `after` over one kind: a record is not later than itself, while another record at the same
    /// instant is not ordered against it.
    #[test]
    fn after_over_one_kind_does_not_order_a_record_after_itself() {
        use Truth::{False as F, True as T, Unknown as U};
        let form = after_of("m", "m", "failed", None);
        let red = |id, at| record(id, "m", Some("failed"), "a", at);
        let green = |id, at| record(id, "m", Some("passed"), "a", at);
        for (history, expected, why) in [
            (vec![red("r", T1)], F, "one record"),
            (vec![red("r", T1), red("s", T1)], U, "two at one instant"),
            (
                vec![green("g", T1), red("r", T1)],
                U,
                "another result at the instant",
            ),
            (vec![red("r", T1), red("s", T2)], T, "a second later"),
            (vec![green("g", T1), red("r", T2)], T, "after a pass"),
            (vec![red("r", T1), green("g", T2)], F, "only a pass later"),
            (vec![green("g", T1)], F, "no record with the result"),
            (vec![red("r", None)], U, "unplaced"),
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

    /// `after` reasons come before every other, ordered by first kind, later kind, result and
    /// subject, each once; the ones of `before` and `first` keep their order after them.
    #[test]
    fn after_reasons_come_first_and_name_each_form_once() {
        let forms = [
            first_of("k", "failed", None),
            after_of("k", "m", "passed", None),
            before_of("k", "m", None),
            after_of("k", "m", "failed", Some("a")),
            after_of("k", "m", "failed", None),
            after_of("k", "m", "passed", None),
        ];
        let history = [
            record("k", "k", Some("failed"), "a", T1),
            record("r", "m", Some("failed"), "a", T2),
        ];
        assert_eq!(
            reasons(&forms, &history),
            [
                serde_json::json!({"after": "k", "then": "m", "result": "failed", "value": "true"}),
                serde_json::json!({"after": "k", "then": "m", "result": "failed", "subject": "a", "value": "true"}),
                serde_json::json!({"after": "k", "then": "m", "result": "passed", "value": "false"}),
                serde_json::json!({"before": "k", "then": "m", "value": "true"}),
                serde_json::json!({"first": "k", "result": "failed", "value": "true"}),
            ]
        );
    }

    /// Instants with a fraction order by the intervals they name (story:evidence-order-subsecond):
    /// records within one second are ordered when their intervals do not overlap, and equal
    /// instants, one instant at two precisions and a whole second beside a fraction within it are
    /// not ordered.
    #[test]
    fn sub_second_instants_order_by_the_intervals_they_name() {
        use Truth::{False as F, True as T, Unknown as U};
        let at = |time: &'static str| Some(time);
        let k = |id, time| record(id, "k", None, "a", time);
        let red = |id, time| record(id, "m", Some("failed"), "a", time);
        let green = |id, time| record(id, "m", Some("passed"), "a", time);
        let after = after_of("k", "m", "failed", None);
        let again = after_of("m", "m", "failed", None);
        let before = before_of("k", "m", None);
        let first = first_of("m", "failed", None);
        let cases = [
            (
                &after,
                vec![
                    k("k", at("2026-10-01T12:00:00.100Z")),
                    red("r", at("2026-10-01T12:00:00.500Z")),
                ],
                T,
                "after: 0.4 s later",
            ),
            (
                &after,
                vec![
                    k("k", at("2026-10-01T12:00:00.500Z")),
                    red("r", at("2026-10-01T12:00:00.100Z")),
                ],
                F,
                "after: 0.4 s earlier",
            ),
            (
                &after,
                vec![
                    k("k", at("2026-10-01T12:00:00.500Z")),
                    red("r", at("2026-10-01T12:00:00.500Z")),
                ],
                U,
                "after: equal",
            ),
            (
                &after,
                vec![
                    k("k", at("2026-10-01T12:00:00.4Z")),
                    red("r", at("2026-10-01T12:00:00.40Z")),
                ],
                U,
                "after: one instant at two precisions",
            ),
            (
                &after,
                vec![
                    k("k", at("2026-10-01T12:00:00Z")),
                    red("r", at("2026-10-01T12:00:00.500Z")),
                ],
                U,
                "after: a fraction within the whole second",
            ),
            (
                &after,
                vec![
                    k("k", at("2026-10-01T12:00:00.500Z")),
                    red("r", at("2026-10-01T12:00:00Z")),
                ],
                U,
                "after: the whole second around the fraction",
            ),
            (
                &after,
                vec![
                    k("k", at("2026-10-01T12:00:00Z")),
                    red("r", at("2026-10-01T12:00:01.200Z")),
                ],
                T,
                "after: the next second",
            ),
            (
                &after,
                vec![
                    k("k", at("2026-10-01T12:00:00.4Z")),
                    red("r", at("2026-10-01T12:00:00.5Z")),
                ],
                T,
                "after: the interval ends where the other begins",
            ),
            (
                &after,
                vec![
                    k("k", at("2026-10-01T12:00:00Z")),
                    k("j", at("2026-10-01T12:00:00.100Z")),
                    red("r", at("2026-10-01T12:00:00.500Z")),
                ],
                T,
                "after: later than one record of the first kind, though not the whole second",
            ),
            (
                &again,
                vec![red("r", at("2026-10-01T12:00:00.5Z"))],
                F,
                "one kind: one record",
            ),
            (
                &again,
                vec![
                    red("r", at("2026-10-01T12:00:00.5Z")),
                    red("s", at("2026-10-01T12:00:00.6Z")),
                ],
                T,
                "one kind: a second record later in the second",
            ),
            (
                &again,
                vec![
                    red("r", at("2026-10-01T12:00:00Z")),
                    red("s", at("2026-10-01T12:00:00.6Z")),
                ],
                U,
                "one kind: a record within the whole second of another",
            ),
            (
                &again,
                vec![
                    red("r", at("2026-10-01T12:00:00.1Z")),
                    green("g", at("2026-10-01T12:00:00.2Z")),
                ],
                F,
                "one kind: only a pass later",
            ),
            (
                &before,
                vec![
                    k("k", at("2026-10-01T12:00:00.100Z")),
                    red("r", at("2026-10-01T12:00:00.500Z")),
                ],
                T,
                "before: earlier in the second",
            ),
            (
                &before,
                vec![
                    k("k", at("2026-10-01T12:00:00.500Z")),
                    red("r", at("2026-10-01T12:00:00.100Z")),
                ],
                F,
                "before: later in the second",
            ),
            (
                &before,
                vec![
                    k("k", at("2026-10-01T12:00:00Z")),
                    red("r", at("2026-10-01T12:00:00.100Z")),
                ],
                U,
                "before: mixed precision",
            ),
            (
                &before,
                vec![
                    k("k", at("2026-10-01T12:00:00Z")),
                    k("j", at("2026-10-01T12:00:00.050Z")),
                    red("r", at("2026-10-01T12:00:00.100Z")),
                ],
                T,
                "before: one record of the first kind before every one of the second",
            ),
            (
                &first,
                vec![
                    red("r", at("2026-10-01T12:00:00.2Z")),
                    green("g", at("2026-10-01T12:00:00.3Z")),
                ],
                T,
                "first: the failure earlier in the second",
            ),
            (
                &first,
                vec![
                    green("g", at("2026-10-01T12:00:00.2Z")),
                    red("r", at("2026-10-01T12:00:00.3Z")),
                ],
                F,
                "first: the pass earlier in the second",
            ),
            (
                &first,
                vec![
                    red("r", at("2026-10-01T12:00:00Z")),
                    green("g", at("2026-10-01T12:00:00.3Z")),
                ],
                U,
                "first: either may be first",
            ),
            (
                &first,
                vec![
                    red("r", at("2026-10-01T12:00:00Z")),
                    red("s", at("2026-10-01T12:00:00.3Z")),
                    green("g", at("2026-10-01T12:00:01.5Z")),
                ],
                T,
                "first: both that may be first fail",
            ),
        ];
        for (form, history, expected, why) in cases {
            assert_eq!(value(form, &history), Some(expected), "{why}");
        }
    }

    /// xorshift64*, a fixed seed: the same histories on every run.
    struct Rng(u64);

    impl Rng {
        fn below(&mut self, n: usize) -> usize {
            self.0 ^= self.0 >> 12;
            self.0 ^= self.0 << 25;
            self.0 ^= self.0 >> 27;
            (self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) % n as u64) as usize
        }
    }

    /// The order forms of Canon 0.7.0, over whole seconds, as the oracle the interval rules must
    /// agree with on every whole-second history.
    mod whole_seconds {
        use super::super::of;
        use crate::model::{EvidenceRecord, Predicate, Truth};

        fn at(record: &EvidenceRecord) -> Option<i64> {
            record.observed_at.as_ref()?.seconds()
        }

        fn earliest(records: &[&EvidenceRecord]) -> Option<i64> {
            records
                .iter()
                .map(|record| at(record))
                .collect::<Option<Vec<i64>>>()?
                .into_iter()
                .min()
        }

        pub(super) fn value(predicate: &Predicate, history: &[EvidenceRecord]) -> Truth {
            match predicate {
                Predicate::First(form) => {
                    let records = of(&form.kind, form.subject.as_ref(), history);
                    let Some(first) = (!records.is_empty()).then(|| earliest(&records)).flatten()
                    else {
                        return Truth::Unknown;
                    };
                    let results: Vec<bool> = records
                        .iter()
                        .filter(|record| at(record) == Some(first))
                        .map(|record| record.result.as_deref() == Some(form.result.as_str()))
                        .collect();
                    if results.iter().all(|with| *with) {
                        Truth::True
                    } else if results.iter().all(|with| !with) {
                        Truth::False
                    } else {
                        Truth::Unknown
                    }
                }
                Predicate::Before(form) => {
                    let firsts = of(&form.first, form.subject.as_ref(), history);
                    let thens = of(&form.then, form.subject.as_ref(), history);
                    match (firsts.is_empty(), thens.is_empty()) {
                        (_, true) => Truth::Unknown,
                        (true, false) => Truth::False,
                        (false, false) => match (earliest(&firsts), earliest(&thens)) {
                            (Some(first), Some(then)) if first < then => Truth::True,
                            (Some(first), Some(then)) if first > then => Truth::False,
                            _ => Truth::Unknown,
                        },
                    }
                }
                Predicate::After(form) => {
                    let firsts = of(&form.first, form.subject.as_ref(), history);
                    if firsts.is_empty() {
                        return Truth::Unknown;
                    }
                    let laters: Vec<&EvidenceRecord> =
                        of(&form.then.kind, form.subject.as_ref(), history)
                            .into_iter()
                            .filter(|record| {
                                record.result.as_deref() == Some(form.then.result.as_str())
                            })
                            .collect();
                    if laters.is_empty() {
                        return Truth::False;
                    }
                    let Some(first) = earliest(&firsts) else {
                        return Truth::Unknown;
                    };
                    let Some(instants) = laters
                        .iter()
                        .map(|record| at(record))
                        .collect::<Option<Vec<i64>>>()
                    else {
                        return Truth::Unknown;
                    };
                    if instants.iter().any(|instant| *instant > first) {
                        return Truth::True;
                    }
                    let at_first: Vec<&EvidenceRecord> = firsts
                        .into_iter()
                        .filter(|record| at(record) == Some(first))
                        .collect();
                    let itself = |record: &EvidenceRecord| matches!(at_first.as_slice(), [only] if std::ptr::eq(*only, record));
                    if laters
                        .iter()
                        .zip(&instants)
                        .any(|(record, instant)| *instant == first && !itself(record))
                    {
                        Truth::Unknown
                    } else {
                        Truth::False
                    }
                }
                _ => unreachable!("only order forms are compared"),
            }
        }
    }

    /// Every order form a history of two kinds can be asked about, over two subjects.
    fn every_form() -> Vec<Predicate> {
        let mut forms = Vec::new();
        for subject in [None, Some("a")] {
            for (one, other) in [("k", "m"), ("m", "k"), ("m", "m"), ("k", "k")] {
                forms.push(before_of(one, other, subject));
                for result in ["failed", "passed"] {
                    forms.push(after_of(one, other, result, subject));
                }
            }
            for kind in ["k", "m"] {
                for result in ["failed", "passed"] {
                    forms.push(first_of(kind, result, subject));
                }
            }
        }
        forms
    }

    /// A random history of up to five records of two kinds, about two subjects, with any result
    /// or none, at one of three whole seconds or without `observed_at`.
    fn random_history(rng: &mut Rng) -> Vec<EvidenceRecord> {
        const SECONDS: [Option<&str>; 4] = [
            Some("2026-10-01T09:00:00Z"),
            Some("2026-10-01T09:00:01Z"),
            Some("2026-10-01T09:00:02Z"),
            None,
        ];
        (0..rng.below(6))
            .map(|index| {
                let at = SECONDS[if rng.below(8) == 0 { 3 } else { rng.below(3) }];
                record(
                    &format!("e{index}"),
                    ["k", "m"][rng.below(2)],
                    [Some("failed"), Some("passed"), None][rng.below(3)],
                    ["a", "b"][rng.below(2)],
                    at,
                )
            })
            .collect()
    }

    /// Over whole seconds, every order form has the value Canon 0.7.0 gave it.
    #[test]
    fn over_whole_seconds_every_value_is_the_one_before_fractions() {
        let forms = every_form();
        let mut rng = Rng(0x5EC0_4D5A_F7E2_0001);
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..20_000 {
            let history = random_history(&mut rng);
            for form in &forms {
                let expected = whole_seconds::value(form, &history);
                assert_eq!(
                    value(form, &history),
                    Some(expected),
                    "{form:?} over {history:?}"
                );
                seen.insert(expected.to_string());
            }
        }
        assert_eq!(seen.len(), 3, "every value is reached: {seen:?}");
    }

    /// Writing more digits only narrows an instant to a part of its interval, so it can settle an
    /// order that was `unknown` and never turns `true` into `false` or back.
    #[test]
    fn a_finer_instant_never_reverses_a_known_order() {
        let forms = every_form();
        let mut rng = Rng(0x5EC0_4D5A_F7E2_0002);
        let (mut settled, mut compared) = (0, 0);
        for _ in 0..20_000 {
            let coarse = random_history(&mut rng);
            let fine: Vec<EvidenceRecord> = coarse
                .iter()
                .map(|record| {
                    let mut fine = record.clone();
                    if let Some(at) = &record.observed_at
                        && rng.below(2) == 0
                    {
                        let digits: String = (0..1 + rng.below(9))
                            .map(|_| char::from(b'0' + rng.below(10) as u8))
                            .collect();
                        let text = at.as_str().strip_suffix('Z').expect("an instant");
                        fine.observed_at = Some(Instant::new(format!("{text}.{digits}Z")));
                    }
                    fine
                })
                .collect();
            for form in &forms {
                let before = value(form, &coarse).expect("an order form");
                let after = value(form, &fine).expect("an order form");
                if before != Truth::Unknown {
                    compared += 1;
                    assert_eq!(after, before, "{form:?}: {coarse:?} refined to {fine:?}");
                } else if after != Truth::Unknown {
                    settled += 1;
                }
            }
        }
        assert!(
            compared > 0 && settled > 0,
            "{compared} compared, {settled} settled"
        );
    }
}
