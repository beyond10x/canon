//! The freshness exclusion stage and the evaluation instant it reads (`--at`): evidence older than
//! its kind allows at that instant no longer applies (design § 8, CANON-EVIDENCE-002).
//!
//! The instant is written as an [`crate::model::Instant`], `YYYY-MM-DDTHH:MM:SSZ` in UTC; any
//! other text is refused as `invalid-instant`. A record is excluded as `expired` when its kind
//! declares a `max_age`, the record gives its `observed_at`, an instant is given, and the instant
//! is later than `observed_at` by more than the maximum age. A record exactly as old as its
//! maximum age still applies; one observed after the instant is not expired. Without an instant,
//! or without `observed_at`, nothing expires: the evaluator reads no clock.

use std::collections::BTreeMap;

use super::Refusal;
use crate::ir::Ir;
use crate::model::{
    Case, EvidenceExclusion, EvidenceRecord, ExclusionReason, Instant as Written, one_line,
};

/// The evaluation instant, in seconds since `1970-01-01T00:00:00Z`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Instant(i64);

/// An `invalid-instant` refusal: `what` gives `text`, which is not an instant as Canon writes it.
pub(super) fn invalid_instant(what: &str, text: &str) -> Refusal {
    Refusal::new(
        "invalid-instant",
        format!(
            "{what} `{}` is not an instant in UTC written YYYY-MM-DDTHH:MM:SSZ",
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
            "the evaluation instant `2026-10-04 12:00` is not an instant in UTC written YYYY-MM-DDTHH:MM:SSZ"
        );
    }
}
