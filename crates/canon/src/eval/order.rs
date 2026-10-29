//! The order forms of the predicate (story:evidence-order-predicate, CANON-ORDER-001,
//! story:evidence-order-after, CANON-ORDER-002, and story:evidence-order-subsecond,
//! CANON-ORDER-003): whether the first record of a kind has a result (`first`), whether the first
//! record of one kind was observed strictly before the first record of another (`before`), and
//! whether a record of one kind with a result was observed strictly after the first record of
//! another (`after`).
//!
//! An order form reads the history: every record of its kind (about its `subject` when it names
//! one, by the rule evidence matches read by, [`super::reads`]) that the evidence check accepted,
//! whatever the exclusion stages decide. A record excluded as `revision_mismatch` or `expired`, or
//! invalidated for the claim being evaluated, is read: the exclusion stages say what holds of the
//! current revisions now, while order is a fact about what was observed and when. No record is
//! listed in a claim's `excluded_evidence` because of an order form.
//!
//! A record's place in the order is its `observed_at` read at the evaluation's resolution
//! ([`ticks`], the one place the order is read from): the coarsest precision written among the
//! records of the history that have an `observed_at`, each instant read as whole ticks of it with
//! the finer digits dropped. Ticks that differ are ordered and equal ticks are not, so the order is
//! a weak order, which `canon check`'s state space rests on. Evidence written all in milliseconds
//! orders two records 0.4 s apart within one second; one record written in whole seconds makes the
//! evaluation compare whole seconds, so `12:00:00.200Z` and `12:00:00.500Z` beside it are not
//! ordered. An order read so is always an order of the intervals the written instants name. With
//! whole seconds everywhere a tick is a second and every value is the 0.7.0 value. A record
//! without `observed_at` has no known place. Below, "instant" means the tick.
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
//! `{after: {first: K1, then: {kind: K2, result: R}}}` compares the first record of `K1` with the
//! records of `K2` that have result `R`; records of `K2` with another result, or none, take no part.
//! `unknown` with no record of `K1`; `false` with a record of `K1` and no record of `K2` with `R`;
//! otherwise `unknown` when a compared record has no `observed_at`, `true` when a record of `K2`
//! with `R` is later than the earliest instant of `K1`, `unknown` when none is later but one is at
//! that instant, and `false` when every one is earlier. With `K1` and `K2` the same kind, a record
//! is not later than itself: the one record at the earliest instant does not follow itself.
//!
//! A record bound to a revision of its subject that does not descend from the current one counts
//! like any other: Canon does not know how revisions descend (story:revision-ordering).

use serde_json::{Map, Value};

use super::claims::reads;
use crate::model::{
    ArtifactId, EvidenceAfter, EvidenceKindId, EvidenceOrder, EvidenceRecord, FirstResult,
    Predicate, Truth,
};

/// The value of an order form over the `history`; `None` for any other form.
pub(crate) fn value(predicate: &Predicate, history: &[EvidenceRecord]) -> Option<Truth> {
    let ticks = || ticks(history);
    match predicate {
        Predicate::First(form) => Some(first(form, history, &ticks())),
        Predicate::Before(form) => Some(before(form, history, &ticks())),
        Predicate::After(form) => Some(after(form, history, &ticks())),
        _ => None,
    }
}

/// Where each record of `history` stands in the order, by index: its `observed_at` read at the
/// evaluation's resolution, or `None` when it has none. This is the one place the order is read
/// from (story:evidence-order-subsecond).
///
/// The resolution is the coarsest precision written among the records of `history` whose
/// `observed_at` is an instant ([`crate::model::Instant::digits`]), and each instant is read as
/// whole ticks of it, the finer digits dropped ([`crate::model::Instant::ticks`]). Ticks that
/// differ are ordered and equal ticks are not, a weak order; two instants in different ticks name
/// intervals that do not overlap. With whole seconds everywhere a tick is a second. The evidence
/// check has refused an `observed_at` that is not an instant; one in a set a caller builds is
/// treated as missing and does not set the resolution.
pub(crate) fn ticks(history: &[EvidenceRecord]) -> Vec<Option<i128>> {
    let digits = history
        .iter()
        .filter_map(|record| record.observed_at.as_ref()?.digits())
        .min()
        .unwrap_or(0);
    history
        .iter()
        .map(|record| record.observed_at.as_ref()?.ticks(digits))
        .collect()
}

/// The indices of the records of `history` of `kind`, about `subject` when it names one.
fn of(
    kind: &EvidenceKindId,
    subject: Option<&ArtifactId>,
    history: &[EvidenceRecord],
) -> Vec<usize> {
    history
        .iter()
        .enumerate()
        .filter(|(_, record)| reads(kind, subject, record))
        .map(|(index, _)| index)
        .collect()
}

/// The earliest tick of the records at `indices`, or `None` when one of them has no known place.
/// `indices` is not empty.
fn earliest(indices: &[usize], ticks: &[Option<i128>]) -> Option<i128> {
    indices
        .iter()
        .map(|&index| ticks[index])
        .collect::<Option<Vec<i128>>>()?
        .into_iter()
        .min()
}

fn first(form: &FirstResult, history: &[EvidenceRecord], ticks: &[Option<i128>]) -> Truth {
    let records = of(&form.kind, form.subject.as_ref(), history);
    if records.is_empty() {
        return Truth::Unknown;
    }
    let Some(at) = earliest(&records, ticks) else {
        return Truth::Unknown;
    };
    let (with, without) = records
        .iter()
        .filter(|&&index| ticks[index] == Some(at))
        .fold((0usize, 0usize), |(with, without), &index| {
            if history[index].result.as_deref() == Some(form.result.as_str()) {
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

fn before(form: &EvidenceOrder, history: &[EvidenceRecord], ticks: &[Option<i128>]) -> Truth {
    let firsts = of(&form.first, form.subject.as_ref(), history);
    let thens = of(&form.then, form.subject.as_ref(), history);
    match (firsts.is_empty(), thens.is_empty()) {
        (true, true) | (false, true) => Truth::Unknown,
        (true, false) => Truth::False,
        (false, false) => match (earliest(&firsts, ticks), earliest(&thens, ticks)) {
            (Some(first), Some(then)) if first < then => Truth::True,
            (Some(first), Some(then)) if first > then => Truth::False,
            _ => Truth::Unknown,
        },
    }
}

fn after(form: &EvidenceAfter, history: &[EvidenceRecord], ticks: &[Option<i128>]) -> Truth {
    let firsts = of(&form.first, form.subject.as_ref(), history);
    if firsts.is_empty() {
        return Truth::Unknown;
    }
    let laters: Vec<usize> = of(&form.then.kind, form.subject.as_ref(), history)
        .into_iter()
        .filter(|&index| history[index].result.as_deref() == Some(form.then.result.as_str()))
        .collect();
    if laters.is_empty() {
        return Truth::False;
    }
    let Some(at) = earliest(&firsts, ticks) else {
        return Truth::Unknown;
    };
    let Some(later_ticks) = laters
        .iter()
        .map(|&index| ticks[index])
        .collect::<Option<Vec<i128>>>()
    else {
        return Truth::Unknown;
    };
    if later_ticks.iter().any(|tick| *tick > at) {
        return Truth::True;
    }
    // A record at the earliest tick is not ordered against the first record, unless it is the
    // first record itself, alone at that tick: a record is not later than itself.
    let at_first: Vec<usize> = firsts
        .into_iter()
        .filter(|&index| ticks[index] == Some(at))
        .collect();
    let itself = |index: usize| at_first == [index];
    if laters
        .iter()
        .zip(&later_ticks)
        .any(|(&index, tick)| *tick == at && !itself(index))
    {
        Truth::Unknown
    } else {
        Truth::False
    }
}

/// How the reasons order the order forms: every `after` first, then every `before`, then every
/// `first`, each by its fields as text (no subject before any subject).
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
        _ => None,
    }
}

/// The reason each of `forms` gives, with its value over the `history`, each form once, `after`
/// forms first, then `before` forms: `{"after": K1, "then": K2, "result": R, "value": <truth>}`,
/// `{"before": K1, "then": K2, "value": <truth>}` or `{"first": K, "result": R, "value": <truth>}`,
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

    /// Instants with a fraction order at the evaluation's resolution, the coarsest written
    /// (story:evidence-order-subsecond): records within one second are ordered when every record
    /// is written finely enough to tell them apart, and equal instants, one instant at two
    /// precisions, and anything in one second beside a whole-second record are not ordered.
    #[test]
    fn sub_second_instants_order_at_the_evaluation_resolution() {
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
                "after: the next tenth",
            ),
            (
                &after,
                vec![
                    k("k", at("2026-10-01T12:00:00Z")),
                    k("j", at("2026-10-01T12:00:00.100Z")),
                    red("r", at("2026-10-01T12:00:00.500Z")),
                ],
                U,
                "after: a whole-second record makes the evaluation compare seconds",
            ),
            (
                &after,
                vec![
                    k("k", at("2026-10-01T12:00:00.200Z")),
                    red("r", at("2026-10-01T12:00:00.500Z")),
                    record("x", "x", None, "b", at("2026-10-01T12:00:09Z")),
                ],
                U,
                "after: a whole-second record of a kind the form does not read coarsens it too",
            ),
            (
                &after,
                vec![
                    k("k", at("2026-10-01T12:00:00.200Z")),
                    red("r", at("2026-10-01T12:00:00.500Z")),
                    record("x", "x", None, "b", at("2026-10-01T12:00:09.000Z")),
                ],
                T,
                "after: every record in milliseconds",
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
                U,
                "before: a whole-second record makes the evaluation compare seconds",
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

    /// The order forms of Canon 0.7.0, over whole seconds, as the oracle the tick rules must agree
    /// with on every whole-second history.
    mod whole_seconds {
        use crate::model::{ArtifactId, EvidenceKindId, EvidenceRecord, Predicate, Truth};

        fn of<'r>(
            kind: &EvidenceKindId,
            subject: Option<&ArtifactId>,
            history: &'r [EvidenceRecord],
        ) -> Vec<&'r EvidenceRecord> {
            history
                .iter()
                .filter(|record| super::super::reads(kind, subject, record))
                .collect()
        }

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
