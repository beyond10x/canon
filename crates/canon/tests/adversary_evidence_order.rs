//! Adversary pass 1 for story:evidence-order-predicate, against `canon check`'s `authority-bypass`
//! finding over the order forms.
//!
//! The contract read is the module docs of `check`: `authority-bypass` is "an outcome that reads
//! governed evidence where its presence helps it hold, and that is legitimate without authority
//! because of evidence present". An evidence match helps only in a positive position: under a
//! `not`, a record it reads makes it `true` and so the `not` `false`. An order form is not like that.
//! `{first: {kind: K, result: R}}` is `unknown` with no record of `K`, and so is its negation: a
//! record of `K` (whose result is not `R`) is what makes `{not: {first: ...}}` `true`. So a record
//! of the kind an order form reads helps the requirement hold under a `not` too, and an outcome
//! that holds that way on governed evidence produced without authority is a bypass. The
//! implementation counts an order form's kinds only "in a positive position"
//! (`check/space.rs`, `positive_reads`), so it reports no bypass for it.

use b10x_canon::check::check;
use b10x_canon::ir;
use b10x_canon::model;

fn report(source: &str) -> String {
    let compiled = ir::compile(&model::parse(source).expect("parses")).expect("compiles");
    match check(&compiled, None) {
        Ok(report) => report.to_string(),
        Err(refusal) => format!("refused {}: {refusal}", refusal.code()),
    }
}

/// `review` is governed (`review` requires a capability) and also produced by `self_review`, which
/// requires none: the seeded defect of `fixtures/investigation/check/authority-bypass`. `approved`
/// requires the first review to have been an approval; `not_rejected` requires the first review not
/// to have been a rejection. Each is `unknown`, so blocked, with no review at all, and legitimate
/// with one review produced without authority (`approved` with result `approved`, `not_rejected`
/// with any result but `rejected`); taking that review away blocks each. Both are bypasses.
#[test]
fn an_order_form_under_not_still_reads_governed_evidence_for_a_bypass() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {change: {}}\n\
        evidence_kinds: {review: {}}\n\
        actions:\n\
        \x20\x20review: {requires: [{capability: review.give}], may_produce: [{evidence: review}]}\n\
        \x20\x20self_review: {may_produce: [{evidence: review}]}\n\
        outcomes:\n\
        \x20\x20approved: {requires: {first: {kind: review, result: approved}}}\n\
        \x20\x20not_rejected: {requires: {not: {first: {kind: review, result: rejected}}}}\n";
    let report = report(source);
    // The positive control: the same outcome with the form in a positive position is reported.
    assert!(
        report.contains("authority-bypass: outcome `approved` is legitimate without authority"),
        "{report}"
    );
    assert!(
        report.contains("authority-bypass: outcome `not_rejected` is legitimate without authority"),
        "a review produced without authority makes `not_rejected` legitimate, and taking it away \
         blocks it, so it is a bypass:\n{report}"
    );
}

/// The same through a claim test `is: false`, which the docs count as a flip like `not`: the claim
/// `rejected_first` is `false` only when a review exists and the first one was not a rejection.
#[test]
fn an_order_form_behind_a_claim_test_is_false_still_reads_governed_evidence_for_a_bypass() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {change: {}}\n\
        evidence_kinds: {review: {}}\n\
        claims:\n\
        \x20\x20rejected_first: {true_when: {first: {kind: review, result: rejected}}}\n\
        actions:\n\
        \x20\x20review: {requires: [{capability: review.give}], may_produce: [{evidence: review}]}\n\
        \x20\x20self_review: {may_produce: [{evidence: review}]}\n\
        outcomes:\n\
        \x20\x20released: {requires: {claim: rejected_first, is: false}}\n";
    let report = report(source);
    assert!(
        report.contains("authority-bypass: outcome `released` is legitimate without authority"),
        "`released` is legitimate on one review produced without authority and blocked without \
         it, so it is a bypass:\n{report}"
    );
}
