//! Review of wave 2026-11-22-w12, story:freshness-needs-trusted-timestamps.
//!
//! The strict mode refuses a record of a kind with a `max_age` that is dated after the evaluation
//! instant, but reads only the records revision binding left. The order forms read every record
//! the evidence check accepted, the ones revision binding excluded included (`eval/order.rs`). So
//! a record observed long after `--at`, of a kind with a `max_age`, bound to an earlier revision
//! of its subject, is not refused and still decides an order form under `--strict`: its untrusted
//! timestamp is what makes the claim true.

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model;

const AT: &str = "2026-10-04T12:00:00Z";

const PROTOCOL: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
     evidence_kinds: {review: {}, fresh: {max_age: 1h}}\nclaims:\n  \
     later: {true_when: {after: {first: review, then: {kind: fresh, result: pass}}}}\n";

const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n";

fn record(id: &str, kind: &str, revision: &str, observed_at: &str) -> String {
    format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\nresult: pass\nsubject: a\n\
         subject_revision: {revision}\nobserved_at: {observed_at}\n"
    )
}

fn decide(evidence: &[String], strict: Option<&str>) -> Result<String, eval::Refusal> {
    let ir = ir::compile(&model::parse(PROTOCOL).expect("parses")).expect("compiles");
    let case = eval::read_case(CASE).expect("case reads");
    let evidence: Vec<_> = evidence
        .iter()
        .map(|text| eval::read_evidence(text).expect("record reads"))
        .collect();
    let supplied = Supplied {
        at: Some(AT),
        ..Supplied::default()
    };
    eval::evaluate_with_freshness(&ir, &case, &evidence, supplied, strict)
        .map(|decision| eval::render(&decision))
}

fn later(decision: &str) -> &'static str {
    if decision.contains("\"later\": {\n      \"value\": \"true\"") {
        "true"
    } else {
        "not true"
    }
}

#[test]
fn strict_refuses_a_future_record_an_order_form_reads_though_revision_binding_excluded_it() {
    let review = record("e0", "review", "r1", "2026-10-04T11:30:00Z");
    // Of a kind with max_age 1h, observed 73 years after the instant, bound to revision r0.
    let future = record("e1", "fresh", "r0", "2099-01-01T00:00:00Z");

    // Control: without the future record the claim is not true; with it, by default, it is, so
    // the order form reads it, and only its timestamp places it after the review.
    let alone = decide(std::slice::from_ref(&review), Some("0s")).expect("decides");
    assert_eq!(later(&alone), "not true", "{alone}");
    let by_default = decide(&[review.clone(), future.clone()], None).expect("decides");
    assert_eq!(later(&by_default), "true", "{by_default}");

    match decide(&[review, future], Some("0s")) {
        Err(refusal) => assert_eq!(refusal.code(), "future-evidence", "{refusal}"),
        Ok(decision) => panic!(
            "--strict 0s at {AT} accepted e1, observed_at 2099-01-01T00:00:00Z of kind `fresh` \
             (max_age 1h), and the order form read it: claim `later` is {}\n{decision}",
            later(&decision)
        ),
    }
}
