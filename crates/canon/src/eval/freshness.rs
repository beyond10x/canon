//! The freshness exclusion stage and the evaluation instant it reads (`--at`): evidence older than
//! its kind allows at that instant no longer applies (design § 8, CANON-EVIDENCE-002).
//!
//! The instant is written as an [`crate::model::Instant`], `YYYY-MM-DDTHH:MM:SSZ` in UTC, or with
//! a fraction of one to nine digits, `YYYY-MM-DDTHH:MM:SS.fffZ`; any other text is refused as
//! `invalid-instant`. A record is excluded as `expired` when its kind declares a `max_age`, the
//! record gives its `observed_at`, an instant is given, and the instant is later than `observed_at`
//! by more than the maximum age. A record exactly as old as its maximum age still applies; one
//! observed after the instant is not expired. Without an instant, or without `observed_at`,
//! nothing expires: the evaluator reads no clock.
//!
//! Expiry counts whole seconds, as `max_age` is written in them: it reads the whole seconds of the
//! instant and of `observed_at` and ignores a fraction (story:evidence-order-subsecond), so
//! `12:00:00.9Z` and `12:00:00Z` are the same instant here. Only the order forms read a fraction.
//!
//! # Strict freshness
//!
//! The rules above are the default, and they trust the timestamps they are given: an undated
//! record and one observed after the instant apply, and without an instant nothing expires. A
//! caller that relies on freshness asks for the strict mode
//! (story:freshness-needs-trusted-timestamps): [`super::evaluate_with_freshness`] with the clock
//! skew it allows, `canon evaluate --strict [<SKEW>]` on the command line, where the skew is `0s`
//! when not given. The skew is written as a `max_age` is, a whole number and one unit, `s`, `m`,
//! `h` or `d` (`30s`, `5m`); any other text, or one too long to count in seconds, is refused as
//! `invalid-skew`, read right after the instant.
//!
//! The strict mode refuses, as this stage already does for a `max_age` it cannot read, rather than
//! excluding: an exclusion would give the decision an exclusion reason `canon-decision/1` does not
//! have, and a refusal changes no byte of any decision. After this stage's own `invalid-max-age`,
//! it refuses:
//!
//! - `missing-instant` when no instant is given and the protocol declares a kind with a `max_age`,
//!   naming the first such kind in identifier order (`` strict freshness needs the evaluation
//!   instant (`--at`): evidence kind `fresh` has max_age `1h` ``);
//! - then, for each record this stage sees (what revision binding left) of a kind with a
//!   `max_age`, in the order given: `undated-evidence` when it gives no `observed_at`, and
//!   `future-evidence` when its `observed_at` is later than the instant by more than the skew,
//!   counted in whole seconds as expiry is (`` evidence `e1` observed_at `2026-10-04T12:00:30Z`
//!   is later than the evaluation instant `2026-10-04T12:00:00Z` by 30 s, more than the allowed
//!   clock skew `0s` ``). Each names and cites the record.
//!
//! A protocol with no `max_age`, a record of a kind without one and a record revision binding
//! excluded are not read by the strict mode; where it refuses nothing, the decision is the
//! default's, byte for byte.

use std::collections::BTreeMap;

use super::Refusal;
use crate::ir::Ir;
use crate::model::{
    Age, Case, EvidenceExclusion, EvidenceRecord, ExclusionReason, Instant as Written, one_line,
};

/// The evaluation instant, in whole seconds since `1970-01-01T00:00:00Z`, its fraction ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Instant(i64);

/// An `invalid-instant` refusal: `what` gives `text`, which is not an instant as Canon writes it.
pub(super) fn invalid_instant(what: &str, text: &str) -> Refusal {
    Refusal::new(
        "invalid-instant",
        format!(
            "{what} `{}` is not an instant in UTC written YYYY-MM-DDTHH:MM:SSZ or YYYY-MM-DDTHH:MM:SS.fffZ (one to nine fraction digits)",
            one_line(text)
        ),
    )
}

/// Reads the evaluation instant given as `--at`, refusing text that is not an instant.
pub(super) fn instant(text: Option<&str>) -> Result<Option<Instant>, Refusal> {
    let Some(text) = text else {
        return Ok(None);
    };
    Written::new(text)
        .seconds()
        .map(|seconds| Some(Instant(seconds)))
        .ok_or_else(|| invalid_instant("the evaluation instant", text))
}

/// The records of `evidence` this stage keeps from claim evaluation, each with the reason
/// `expired`, in the order given. `evidence` is what earlier stages left; the evidence check has
/// already refused an `observed_at` that is not an instant.
///
/// Fails closed: an IR whose evidence kind has a `max_age` [`crate::model::Age::seconds`] cannot
/// read — which `canon compile` never produces, but a caller can build — is refused as
/// `invalid-max-age`, naming the first such kind in identifier order, whether or not an instant
/// is given. Such an age never silently disables expiry.
pub(super) fn exclude(
    ir: &Ir,
    _case: &Case,
    evidence: &[&EvidenceRecord],
    at: Option<&Instant>,
) -> Result<Vec<EvidenceExclusion>, Refusal> {
    let mut max_ages = BTreeMap::new();
    for (id, kind) in &ir.evidence_kinds {
        if let Some(max_age) = &kind.max_age {
            let seconds = max_age.seconds().ok_or_else(|| {
                Refusal::new(
                    "invalid-max-age",
                    format!(
                        "evidence kind `{}` has max_age `{}`, which the evaluator cannot read",
                        one_line(id.as_str()),
                        one_line(max_age.as_str())
                    ),
                )
            })?;
            max_ages.insert(id, seconds);
        }
    }
    let Some(&Instant(at)) = at else {
        return Ok(Vec::new());
    };
    let expired = evidence.iter().filter(|record| {
        let max_age = max_ages.get(&record.kind).copied();
        let observed = record.observed_at.as_ref().and_then(Written::seconds);
        match (max_age, observed) {
            (Some(max_age), Some(observed)) => at.saturating_sub(observed) > max_age,
            _ => false,
        }
    });
    Ok(expired
        .map(|record| EvidenceExclusion {
            evidence: record.id.clone(),
            reason: ExclusionReason::Expired,
        })
        .collect())
}

/// The strict freshness mode: the clock skew allowed, in whole seconds and as written, and the
/// evaluation instant as written, for its refusals to name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Strict<'a> {
    skew: i64,
    skew_text: &'a str,
    at_text: Option<&'a str>,
}

/// Reads the strict mode's allowed clock skew: `None` keeps the default, and text that is not an
/// age the evaluator can read is refused as `invalid-skew`. `at` is the evaluation instant as
/// given, which [`instant`] has read.
pub(super) fn strict<'a>(
    skew: Option<&'a str>,
    at: Option<&'a str>,
) -> Result<Option<Strict<'a>>, Refusal> {
    let Some(text) = skew else {
        return Ok(None);
    };
    let seconds = Age::new(text).seconds().ok_or_else(|| {
        Refusal::new(
            "invalid-skew",
            format!(
                "the allowed clock skew `{}` is not an age the evaluator can read: a whole number \
                 of seconds, minutes, hours or days, written with its unit s, m, h or d",
                one_line(text)
            ),
        )
    })?;
    Ok(Some(Strict {
        skew: seconds,
        skew_text: text,
        at_text: at,
    }))
}

/// The strict mode's refusals over `evidence`, the records this stage saw, once [`exclude`] has
/// read every `max_age` of `ir`, as the module docs give them: `missing-instant`, then each
/// record's `undated-evidence` or `future-evidence` in the order given. Without the strict mode it
/// refuses nothing.
pub(super) fn refuse_untrusted(
    ir: &Ir,
    evidence: &[&EvidenceRecord],
    at: Option<&Instant>,
    strict: Option<&Strict<'_>>,
) -> Result<(), Refusal> {
    let Some(strict) = strict else {
        return Ok(());
    };
    let Some(&Instant(at)) = at else {
        let first = ir
            .evidence_kinds
            .iter()
            .find_map(|(id, kind)| kind.max_age.as_ref().map(|age| (id, age)));
        return match first {
            Some((id, age)) => Err(Refusal::new(
                "missing-instant",
                format!(
                    "strict freshness needs the evaluation instant (`--at`): evidence kind `{}` \
                     has max_age `{}`",
                    one_line(id.as_str()),
                    one_line(age.as_str())
                ),
            )),
            None => Ok(()),
        };
    };
    for record in evidence {
        let Some(age) = ir
            .evidence_kinds
            .get(&record.kind)
            .and_then(|kind| kind.max_age.as_ref())
        else {
            continue;
        };
        let Some(observed_at) = &record.observed_at else {
            return Err(Refusal::new(
                "undated-evidence",
                format!(
                    "evidence `{}` gives no observed_at, and its kind `{}` has max_age `{}`: \
                     strict freshness needs the time it was observed",
                    one_line(record.id.as_str()),
                    one_line(record.kind.as_str()),
                    one_line(age.as_str())
                ),
            )
            .citing(&record.id));
        };
        // The evidence check has refused an `observed_at` that is not an instant.
        let later = observed_at
            .seconds()
            .map_or(0, |observed| observed.saturating_sub(at));
        if later > strict.skew {
            return Err(Refusal::new(
                "future-evidence",
                format!(
                    "evidence `{}` observed_at `{}` is later than the evaluation instant `{}` by \
                     {later} s, more than the allowed clock skew `{}`",
                    one_line(record.id.as_str()),
                    one_line(observed_at.as_str()),
                    one_line(strict.at_text.unwrap_or_default()),
                    one_line(strict.skew_text)
                ),
            )
            .citing(&record.id));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::{read_case, read_evidence};

    fn ir() -> Ir {
        crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
                 evidence_kinds: {fresh: {max_age: 1h}, lasting: {}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles")
    }

    fn record(id: &str, kind: &str, observed_at: Option<&str>) -> EvidenceRecord {
        let observed = observed_at.map_or(String::new(), |at| format!("observed_at: {at}\n"));
        read_evidence(&format!(
            "format: canon-evidence/1\nid: {id}\nkind: {kind}\nsubject: a\nsubject_revision: r1\n{observed}"
        ))
        .expect("record reads")
    }

    fn expired(at: Option<&str>, records: &[EvidenceRecord]) -> Vec<String> {
        let case =
            read_case("format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r1}}\n")
                .expect("case reads");
        let at = instant(at).expect("instant reads");
        let kept: Vec<&EvidenceRecord> = records.iter().collect();
        exclude(&ir(), &case, &kept, at.as_ref())
            .expect("no refusal")
            .into_iter()
            .map(|exclusion| {
                assert_eq!(exclusion.reason, ExclusionReason::Expired);
                exclusion.evidence.as_str().to_owned()
            })
            .collect()
    }

    /// Only a record of a kind with a maximum age, observed more than that age before the
    /// instant, expires; exactly that old, younger, observed later, without `observed_at`, of a
    /// kind without a maximum age, or with no instant given, it applies.
    #[test]
    fn a_record_expires_only_when_older_than_its_kind_allows_at_the_instant() {
        let records = [
            record("older", "fresh", Some("2026-10-04T10:59:59Z")),
            record("exactly", "fresh", Some("2026-10-04T11:00:00Z")),
            record("younger", "fresh", Some("2026-10-04T11:30:00Z")),
            record("later", "fresh", Some("2026-10-04T13:00:00Z")),
            record("unstamped", "fresh", None),
            record("lasting", "lasting", Some("2000-01-01T00:00:00Z")),
        ];
        assert_eq!(expired(Some("2026-10-04T12:00:00Z"), &records), ["older"]);
        assert_eq!(expired(None, &records), Vec::<String>::new());
        // A day later every stamped record of the kind is past its maximum age.
        assert_eq!(
            expired(Some("2026-10-05T12:00:00Z"), &records),
            ["older", "exactly", "younger", "later"]
        );
    }

    /// Expiry counts whole seconds and ignores a fraction on either side
    /// (story:evidence-order-subsecond): 0.9 s past the maximum age within the same second is not
    /// past it, and a fractional evaluation instant reads as its whole second.
    #[test]
    fn expiry_reads_whole_seconds_and_ignores_the_fraction() {
        let records = [
            record("exactly", "fresh", Some("2026-10-04T11:00:00.000Z")),
            record(
                "later_in_the_second",
                "fresh",
                Some("2026-10-04T11:00:00.999Z"),
            ),
            record(
                "a_second_older",
                "fresh",
                Some("2026-10-04T10:59:59.999999999Z"),
            ),
        ];
        assert_eq!(
            expired(Some("2026-10-04T12:00:00.900Z"), &records),
            ["a_second_older"]
        );
        assert_eq!(
            expired(Some("2026-10-04T12:00:00Z"), &records),
            ["a_second_older"]
        );
        assert_eq!(
            instant(Some("2026-10-04T12:00:00.999999999Z")),
            Ok(Some(Instant(1_791_115_200)))
        );
    }

    /// A caller-built IR with a `max_age` the evaluator cannot read is refused, with or without an
    /// instant, naming the kind; it never leaves the record applying.
    #[test]
    fn an_unreadable_maximum_age_in_a_caller_built_ir_is_refused() {
        let case =
            read_case("format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r1}}\n")
                .expect("case reads");
        let records = [record("old", "fresh", Some("2016-10-04T12:00:00Z"))];
        let kept: Vec<&EvidenceRecord> = records.iter().collect();
        for unreadable in ["1 hour", "99999999999999999999s"] {
            let mut built = ir();
            built
                .evidence_kinds
                .get_mut(&crate::model::EvidenceKindId::new("fresh"))
                .expect("fresh is compiled")
                .max_age = Some(crate::model::Age::new(unreadable));
            for at in [None, Some("2026-10-04T12:00:00Z")] {
                let at = instant(at).expect("instant reads");
                let refusal =
                    exclude(&built, &case, &kept, at.as_ref()).expect_err("refused, not ignored");
                assert_eq!(refusal.code(), "invalid-max-age");
                assert_eq!(
                    refusal.to_string(),
                    format!(
                        "evidence kind `fresh` has max_age `{unreadable}`, which the evaluator cannot read"
                    )
                );
            }
        }
    }

    #[test]
    fn an_evaluation_instant_that_is_not_an_instant_is_refused() {
        assert_eq!(instant(None), Ok(None));
        assert_eq!(
            instant(Some("2026-10-04T12:00:00Z")),
            Ok(Some(Instant(1_791_115_200)))
        );
        let refusal = instant(Some("2026-10-04 12:00")).expect_err("refused");
        assert_eq!(refusal.code(), "invalid-instant");
        assert_eq!(
            refusal.to_string(),
            "the evaluation instant `2026-10-04 12:00` is not an instant in UTC written YYYY-MM-DDTHH:MM:SSZ or YYYY-MM-DDTHH:MM:SS.fffZ (one to nine fraction digits)"
        );
    }
}
