//! Adversary cases for story:evidence-freshness against `eval::evaluate_with`.
//!
//! Each case drives from a stated rule: design § 8 (expired evidence leaves a claim `UNKNOWN`,
//! never `FALSE`; `UNKNOWN` and `FALSE` must never be collapsed), the module docs of
//! `eval/freshness.rs` (exactly `max_age` old still applies; observed after the instant is not
//! expired; without an instant or `observed_at` nothing expires) and of `eval/mod.rs` (the check
//! order: evidence records before supplied inputs; each claim lists the excluded records of a kind
//! it reaches, in evidence-id order; the order of `evidence` does not matter).

use b10x_canon::eval::{self, Refusal, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, Decision, ExclusionReason, Truth};

const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n";

fn compiled(kinds: &str, claims: &str) -> ir::Ir {
    let source = format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nartifacts: {{a: {{}}}}\n\
         evidence_kinds: {kinds}\nclaims: {claims}\n"
    );
    ir::compile(&model::parse(&source).expect("protocol parses")).expect("protocol compiles")
}

fn record(id: &str, kind: &str, result: Option<&str>, observed_at: Option<&str>) -> String {
    let mut text = format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\nsubject: a\nsubject_revision: r1\n"
    );
    if let Some(result) = result {
        text.push_str(&format!("result: {result}\n"));
    }
    if let Some(at) = observed_at {
        text.push_str(&format!("observed_at: {at}\n"));
    }
    text
}

fn decide(ir: &ir::Ir, evidence: &[String], at: Option<&str>) -> Result<Decision, Refusal> {
    let case = eval::read_case(CASE)?;
    let evidence = evidence
        .iter()
        .map(|text| eval::read_evidence(text))
        .collect::<Result<Vec<_>, _>>()?;
    eval::evaluate_with(
        ir,
        &case,
        &evidence,
        Supplied {
            at,
            ..Supplied::default()
        },
    )
}

/// `(value, [(evidence, reason)])` of one claim.
fn entry(decision: &Decision, claim: &str) -> (Truth, Vec<(String, &'static str)>) {
    let (_, entry) = decision
        .claims
        .iter()
        .find(|(id, _)| id.as_str() == claim)
        .unwrap_or_else(|| panic!("claim {claim} is in the decision"));
    (
        entry.value,
        entry
            .excluded_evidence
            .iter()
            .map(|exclusion| {
                (
                    exclusion.evidence.as_str().to_owned(),
                    exclusion.reason.as_str(),
                )
            })
            .collect(),
    )
}

fn expired(ids: &[&str]) -> Vec<(String, &'static str)> {
    ids.iter()
        .map(|id| ((*id).to_owned(), ExclusionReason::Expired.as_str()))
        .collect()
}

/// `max_age: 0s`: a record observed at the instant itself applies, one observed a second before
/// it has expired, and one observed after it (a negative age) applies.
#[test]
fn a_zero_maximum_age_expires_everything_observed_before_the_instant_and_nothing_else() {
    let ir = compiled(
        "{k: {max_age: 0s}}",
        "{c: {true_when: {evidence: {kind: k}}}}",
    );
    let at = Some("2026-10-04T12:00:00Z");
    for (observed, value, excluded) in [
        ("2026-10-04T12:00:00Z", Truth::True, expired(&[])),
        ("2026-10-04T11:59:59Z", Truth::Unknown, expired(&["e"])),
        ("2026-10-04T12:00:01Z", Truth::True, expired(&[])),
    ] {
        let decision = decide(&ir, &[record("e", "k", None, Some(observed))], at).expect("decides");
        assert_eq!(
            entry(&decision, "c"),
            (value, excluded),
            "observed {observed}"
        );
    }
}

/// The boundary at a large maximum age: exactly `max_age` old applies, a second older expires.
#[test]
fn exactly_max_age_old_applies_and_one_second_older_expires() {
    let ir = compiled(
        "{k: {max_age: 30d}}",
        "{c: {true_when: {evidence: {kind: k}}}}",
    );
    let at = Some("2026-03-01T00:00:00Z");
    // 2026 is not a leap year: 30 days before 2026-03-01 is 2026-01-30.
    for (observed, value) in [
        ("2026-01-30T00:00:00Z", Truth::True),
        ("2026-01-29T23:59:59Z", Truth::Unknown),
    ] {
        let decision = decide(&ir, &[record("e", "k", None, Some(observed))], at).expect("decides");
        assert_eq!(entry(&decision, "c").0, value, "observed {observed}");
    }
}

/// The widest instants and the longest age: no overflow, no panic, and nothing expires when the
/// age covers the whole calendar; with `0s` the oldest record expires at the latest instant.
#[test]
fn the_extremes_of_the_calendar_and_of_an_age_neither_overflow_nor_expire_wrongly() {
    let claims = "{c: {true_when: {evidence: {kind: k}}}}";
    let long = compiled("{k: {max_age: 9223372036854775807s}}", claims);
    let zero = compiled("{k: {max_age: 0s}}", claims);
    let old = [record("e", "k", None, Some("0000-01-01T00:00:00Z"))];
    let late = [record("e", "k", None, Some("9999-12-31T23:59:59Z"))];
    let last = Some("9999-12-31T23:59:59Z");
    let first = Some("0000-01-01T00:00:00Z");
    assert_eq!(
        entry(&decide(&long, &old, last).unwrap(), "c").0,
        Truth::True
    );
    assert_eq!(
        entry(&decide(&long, &late, first).unwrap(), "c").0,
        Truth::True
    );
    assert_eq!(
        entry(&decide(&zero, &late, first).unwrap(), "c").0,
        Truth::True
    );
    assert_eq!(
        entry(&decide(&zero, &old, last).unwrap(), "c"),
        (Truth::Unknown, expired(&["e"]))
    );
}

/// Design § 8: a contradicting record that has expired no longer contradicts. The claim is FALSE
/// while the refuting record applies and UNKNOWN, not FALSE and not TRUE, once it has expired.
#[test]
fn an_expired_refutation_leaves_the_claim_unknown_not_false() {
    let ir = compiled(
        "{f: {max_age: 1h}}",
        "{c: {true_when: {evidence: {kind: f, result: survived}}}}",
    );
    let evidence = [record(
        "r",
        "f",
        Some("refuted"),
        Some("2026-10-04T12:00:00Z"),
    )];
    let within = decide(&ir, &evidence, Some("2026-10-04T12:30:00Z")).expect("decides");
    assert_eq!(entry(&within, "c"), (Truth::False, expired(&[])));
    let past = decide(&ir, &evidence, Some("2026-10-04T14:00:00Z")).expect("decides");
    assert_eq!(entry(&past, "c"), (Truth::Unknown, expired(&["r"])));
}

/// One expired record is listed once under each claim that reaches its kind, directly or through
/// a claim it tests, and under no claim that does not; the list is in evidence-id order whatever
/// order the records were given in.
#[test]
fn expired_records_are_listed_once_per_reaching_claim_in_evidence_id_order() {
    let ir = compiled(
        "{k: {max_age: 1m}, other: {}}",
        "{direct: {true_when: {all: [{evidence: {kind: k}}, {evidence: {kind: k}}]}}, \
         derived: {true_when: {claim: direct}}, \
         unrelated: {true_when: {evidence: {kind: other}}}}",
    );
    let at = Some("2026-10-04T12:00:00Z");
    let stale = Some("2026-10-04T11:00:00Z");
    let forward = [
        record("e-b", "k", None, stale),
        record("e-a", "k", None, stale),
        record("o", "other", None, stale),
    ];
    let mut backward = forward.clone();
    backward.reverse();
    let one = decide(&ir, &forward, at).expect("decides");
    let two = decide(&ir, &backward, at).expect("decides");
    assert_eq!(
        eval::render(&one),
        eval::render(&two),
        "evidence order matters"
    );
    assert_eq!(
        entry(&one, "direct"),
        (Truth::Unknown, expired(&["e-a", "e-b"]))
    );
    assert_eq!(
        entry(&one, "derived"),
        (Truth::Unknown, expired(&["e-a", "e-b"]))
    );
    assert_eq!(entry(&one, "unrelated"), (Truth::True, expired(&[])));
}

/// A `max_age` on a kind no record has changes nothing: the decision at any instant is the one
/// without an instant, byte for byte, but for the explanation's record of the instant it was
/// computed at (story:explanation, design § 37).
#[test]
fn a_maximum_age_on_a_kind_no_record_has_changes_nothing() {
    let ir = compiled(
        "{k: {}, unused: {max_age: 1s}}",
        "{c: {true_when: {evidence: {kind: k}}}}",
    );
    let evidence = [record("e", "k", None, Some("2000-01-01T00:00:00Z"))];
    let without = eval::render(&decide(&ir, &evidence, None).expect("decides"));
    let mut with = decide(&ir, &evidence, Some("2026-10-04T12:00:00Z")).expect("decides");
    let from = &mut with
        .explanation
        .as_mut()
        .expect("the decision carries an explanation")["computed_from"];
    assert_eq!(
        from["at"], "2026-10-04T12:00:00Z",
        "the instant is recorded"
    );
    from.as_object_mut()
        .expect("computed_from is an object")
        .remove("at");
    let with = eval::render(&with);
    assert_eq!(with, without);
}

/// The evidence set is checked before the supplied inputs are read: with both a malformed
/// `observed_at` and a malformed `--at`, the record is what is refused, naming it; a malformed
/// `observed_at` is refused with no instant given at all.
#[test]
fn a_malformed_observed_at_is_refused_before_the_instant_is_read() {
    let ir = compiled("{k: {}}", "{c: {true_when: {evidence: {kind: k}}}}");
    let evidence = [record("e", "k", None, Some("2026-10-04T12:00:00z"))];
    for at in [None, Some("yesterday"), Some("2026-10-04T12:00:00Z")] {
        let refusal = decide(&ir, &evidence, at).expect_err("refused");
        assert_eq!(refusal.code(), "invalid-instant", "at {at:?}");
        assert!(
            refusal.to_string().starts_with("evidence `e` observed_at "),
            "at {at:?}: {refusal}"
        );
    }
}
