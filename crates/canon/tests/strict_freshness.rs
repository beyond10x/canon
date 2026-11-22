//! Acceptance for story:freshness-needs-trusted-timestamps, against the library.
//!
//! By default the freshness stage keeps an undated record of a kind with `max_age`, keeps a record
//! observed after the evaluation instant, and expires nothing without an instant; the tests named
//! `by_default_*` pin that, byte for byte. In the strict mode (`eval::evaluate_with_freshness`
//! with an allowed clock skew, `--strict` on the command line) each of the three is
//! refused, naming the record or the kind: `undated-evidence`, `future-evidence` and
//! `missing-instant`; a skew that is not an age is refused as `invalid-skew`.

use b10x_canon::eval::{self, Refusal, Supplied};
use b10x_canon::ir;
use b10x_canon::model;

const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n";
const AT: &str = "2026-10-04T12:00:00Z";

/// A protocol with one kind that has a maximum age (`fresh`) and one that has none (`lasting`),
/// and a claim over each.
fn compiled() -> ir::Ir {
    compiled_with("{fresh: {max_age: 1h}, lasting: {}}")
}

fn compiled_with(kinds: &str) -> ir::Ir {
    let source = format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nartifacts: {{a: {{}}}}\n\
         evidence_kinds: {kinds}\nclaims:\n  \
         f: {{true_when: {{evidence: {{kind: fresh, result: pass}}}}}}\n  \
         l: {{true_when: {{evidence: {{kind: lasting, result: pass}}}}}}\n"
    );
    ir::compile(&model::parse(&source).expect("protocol parses")).expect("protocol compiles")
}

fn record(id: &str, kind: &str, revision: &str, observed_at: Option<&str>) -> String {
    let observed = observed_at.map_or(String::new(), |at| format!("observed_at: {at}\n"));
    format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\nresult: pass\nsubject: a\n\
         subject_revision: {revision}\n{observed}"
    )
}

/// The decision as `canon evaluate` prints it, or the refusal, with the strict mode off
/// (`strict` `None`) or on with the allowed skew `strict`.
fn decide(
    ir: &ir::Ir,
    evidence: &[String],
    at: Option<&str>,
    strict: Option<&str>,
) -> Result<String, Refusal> {
    let case = eval::read_case(CASE)?;
    let evidence = evidence
        .iter()
        .map(|text| eval::read_evidence(text))
        .collect::<Result<Vec<_>, _>>()?;
    let supplied = Supplied {
        at,
        ..Supplied::default()
    };
    eval::evaluate_with_freshness(ir, &case, &evidence, supplied, strict)
        .map(|decision| eval::render(&decision))
}

/// The same inputs through `evaluate_with`, the path every existing caller takes.
fn decide_by_default(ir: &ir::Ir, evidence: &[String], at: Option<&str>) -> String {
    let case = eval::read_case(CASE).expect("case reads");
    let evidence = evidence
        .iter()
        .map(|text| eval::read_evidence(text).expect("record reads"))
        .collect::<Vec<_>>();
    let supplied = Supplied {
        at,
        ..Supplied::default()
    };
    eval::render(&eval::evaluate_with(ir, &case, &evidence, supplied).expect("decides"))
}

/// `(code, message, cited records)` of a refusal.
fn refused(result: Result<String, Refusal>) -> (&'static str, String, Vec<String>) {
    let refusal = result.expect_err("refused");
    let cited = refusal
        .evidence()
        .iter()
        .map(|id| id.as_str().to_owned())
        .collect();
    (refusal.code(), refusal.to_string(), cited)
}

/// The claim `f` is true from the record: the record applied.
fn applied(decision: &str) -> bool {
    decision.contains("\"f\": {\n      \"value\": \"true\"\n    }")
}

#[test]
fn by_default_an_undated_record_of_a_kind_with_max_age_applies() {
    let ir = compiled();
    let evidence = [record("e1", "fresh", "r1", None)];
    let decision = decide_by_default(&ir, &evidence, Some(AT));
    assert!(applied(&decision), "{decision}");
    assert_eq!(
        decide(&ir, &evidence, Some(AT), None).expect("decides"),
        decision
    );
}

#[test]
fn by_default_a_record_observed_after_the_instant_applies() {
    let ir = compiled();
    let evidence = [record("e1", "fresh", "r1", Some("2026-10-04T13:00:00Z"))];
    let decision = decide_by_default(&ir, &evidence, Some(AT));
    assert!(applied(&decision), "{decision}");
    assert_eq!(
        decide(&ir, &evidence, Some(AT), None).expect("decides"),
        decision
    );
}

#[test]
fn by_default_no_instant_expires_nothing() {
    let ir = compiled();
    let evidence = [record("e1", "fresh", "r1", Some("2000-01-01T00:00:00Z"))];
    let decision = decide_by_default(&ir, &evidence, None);
    assert!(applied(&decision), "{decision}");
    assert_eq!(
        decide(&ir, &evidence, None, None).expect("decides"),
        decision
    );
}

#[test]
fn strict_refuses_an_undated_record_of_a_kind_with_max_age() {
    let ir = compiled();
    let evidence = [
        record("e0", "fresh", "r1", Some("2026-10-04T11:30:00Z")),
        record("e1", "fresh", "r1", None),
    ];
    assert_eq!(
        refused(decide(&ir, &evidence, Some(AT), Some("0s"))),
        (
            "undated-evidence",
            "evidence `e1` gives no observed_at, and its kind `fresh` has max_age `1h`: strict \
             freshness needs the time it was observed"
                .to_owned(),
            vec!["e1".to_owned()]
        )
    );
}

#[test]
fn strict_refuses_a_record_observed_after_the_instant_with_no_skew_allowed() {
    let ir = compiled();
    let evidence = [record("e1", "fresh", "r1", Some("2026-10-04T12:00:01Z"))];
    assert_eq!(
        refused(decide(&ir, &evidence, Some(AT), Some("0s"))),
        (
            "future-evidence",
            "evidence `e1` observed_at `2026-10-04T12:00:01Z` is later than the evaluation \
             instant `2026-10-04T12:00:00Z` by 1 s, more than the allowed clock skew `0s`"
                .to_owned(),
            vec!["e1".to_owned()]
        )
    );
    // Observed at the instant itself, or later within the same whole second, is not later.
    for same in ["2026-10-04T12:00:00Z", "2026-10-04T12:00:00.999Z"] {
        let evidence = [record("e1", "fresh", "r1", Some(same))];
        let decision = decide(&ir, &evidence, Some(AT), Some("0s")).expect("decides");
        assert_eq!(
            decision,
            decide_by_default(&ir, &evidence, Some(AT)),
            "{same}"
        );
    }
}

#[test]
fn strict_allows_a_record_observed_after_the_instant_within_the_skew() {
    let ir = compiled();
    for (observed, skew) in [
        ("2026-10-04T12:00:30Z", "1m"),
        ("2026-10-04T12:01:00Z", "1m"),
        ("2026-10-04T12:00:30Z", "30s"),
    ] {
        let evidence = [record("e1", "fresh", "r1", Some(observed))];
        let decision = decide(&ir, &evidence, Some(AT), Some(skew)).expect("decides");
        assert!(applied(&decision), "{observed} {skew}: {decision}");
        assert_eq!(
            decision,
            decide_by_default(&ir, &evidence, Some(AT)),
            "{observed} {skew}"
        );
    }
    let evidence = [record("e1", "fresh", "r1", Some("2026-10-04T12:01:01Z"))];
    assert_eq!(
        refused(decide(&ir, &evidence, Some(AT), Some("1m"))),
        (
            "future-evidence",
            "evidence `e1` observed_at `2026-10-04T12:01:01Z` is later than the evaluation \
             instant `2026-10-04T12:00:00Z` by 61 s, more than the allowed clock skew `1m`"
                .to_owned(),
            vec!["e1".to_owned()]
        )
    );
}

#[test]
fn strict_refuses_a_missing_instant_for_a_protocol_with_a_kind_that_has_max_age() {
    let ir = compiled_with("{lasting: {}, fresh: {max_age: 1h}, stale: {max_age: 2d}}");
    let evidence = [record("e1", "fresh", "r1", Some("2026-10-04T11:30:00Z"))];
    assert_eq!(
        refused(decide(&ir, &evidence, None, Some("0s"))),
        (
            "missing-instant",
            "strict freshness needs the evaluation instant (`--at`): evidence kind `fresh` has \
             max_age `1h`"
                .to_owned(),
            Vec::new()
        )
    );
    // Without a record at all, too: the instant is what the protocol needs.
    assert_eq!(
        refused(decide(&ir, &[], None, Some("0s"))).0,
        "missing-instant"
    );
}

#[test]
fn strict_changes_nothing_where_no_kind_has_max_age_or_every_record_is_dated() {
    let lasting = compiled_with("{fresh: {}, lasting: {}}");
    let undated = [
        record("e1", "fresh", "r1", None),
        record("e2", "lasting", "r1", Some("2030-01-01T00:00:00Z")),
    ];
    for at in [None, Some(AT)] {
        assert_eq!(
            decide(&lasting, &undated, at, Some("0s")).expect("decides"),
            decide_by_default(&lasting, &undated, at)
        );
    }
    // An undated or future record of a kind without max_age is not read by the strict mode.
    let ir = compiled();
    let evidence = [
        record("e1", "fresh", "r1", Some("2026-10-04T11:30:00Z")),
        record("e2", "fresh", "r1", Some("2026-10-04T10:00:00Z")),
        record("e3", "lasting", "r1", None),
        record("e4", "lasting", "r1", Some("2030-01-01T00:00:00Z")),
    ];
    let decision = decide(&ir, &evidence, Some(AT), Some("0s")).expect("decides");
    assert_eq!(decision, decide_by_default(&ir, &evidence, Some(AT)));
    assert!(decision.contains("\"reason\": \"expired\""), "{decision}");
}

/// The strict mode reads the records the freshness stage sees: one revision binding already
/// excluded does not apply, so it is not refused.
#[test]
fn strict_does_not_read_a_record_revision_binding_excluded() {
    let ir = compiled();
    let evidence = [
        record("e1", "fresh", "r0", None),
        record("e2", "fresh", "r0", Some("2030-01-01T00:00:00Z")),
    ];
    let decision = decide(&ir, &evidence, Some(AT), Some("0s")).expect("decides");
    assert_eq!(decision, decide_by_default(&ir, &evidence, Some(AT)));
    assert!(
        decision.contains("\"reason\": \"revision_mismatch\""),
        "{decision}"
    );
}

#[test]
fn strict_refuses_a_skew_that_is_not_an_age() {
    let ir = compiled();
    let evidence = [record("e1", "fresh", "r1", Some("2026-10-04T11:30:00Z"))];
    for skew in ["", "1 minute", "01m", "-1s", "99999999999999999999s"] {
        assert_eq!(
            refused(decide(&ir, &evidence, Some(AT), Some(skew))),
            (
                "invalid-skew",
                format!(
                    "the allowed clock skew `{skew}` is not an age the evaluator can read: a \
                     whole number of seconds, minutes, hours or days, written with its unit s, m, \
                     h or d"
                ),
                Vec::new()
            ),
            "{skew:?}"
        );
    }
}
