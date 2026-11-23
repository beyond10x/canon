//! story:freshness-needs-trusted-timestamps, the order forms (review of wave 2026-11-22-w12).
//!
//! The order forms (`first`, `before`, `after`, `since`) read the evidence history: every record
//! the evidence check accepted, the ones revision binding excluded included (`eval/order.rs`). So
//! the strict mode reads that whole set: a record of a kind with a `max_age`, observed long after
//! `--at` and bound to an earlier revision of its subject, is refused as `future-evidence`, as it
//! is when it applies. Without the refusal its untrusted timestamp decides the order form.

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model;

const AT: &str = "2026-10-04T12:00:00Z";

const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n";

/// A protocol whose one claim `c` is the order form `form`, over the kinds `review` (no
/// `max_age`) and `fresh` (`max_age: 1h`).
fn protocol(form: &str) -> ir::Ir {
    let source = format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nartifacts: {{a: {{}}}}\n\
         evidence_kinds: {{review: {{}}, fresh: {{max_age: 1h}}}}\nclaims:\n  \
         c: {{true_when: {form}}}\n"
    );
    ir::compile(&model::parse(&source).expect("parses")).expect("compiles")
}

fn record(id: &str, kind: &str, revision: &str, observed_at: &str) -> String {
    format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\nresult: pass\nsubject: a\n\
         subject_revision: {revision}\nobserved_at: {observed_at}\n"
    )
}

fn decide(
    ir: &ir::Ir,
    evidence: &[String],
    strict: Option<&str>,
) -> Result<model::Decision, eval::Refusal> {
    let case = eval::read_case(CASE).expect("case reads");
    let evidence: Vec<_> = evidence
        .iter()
        .map(|text| eval::read_evidence(text).expect("record reads"))
        .collect();
    let supplied = Supplied {
        at: Some(AT),
        ..Supplied::default()
    };
    eval::evaluate_with_freshness(ir, &case, &evidence, supplied, strict)
}

/// The value of the claim `c`, as `canon-decision/1` writes it.
fn value(decision: &model::Decision) -> String {
    let rendered: serde_json::Value =
        serde_json::from_str(&eval::render(decision)).expect("the decision is JSON");
    rendered["claims"]["c"]["value"]
        .as_str()
        .expect("c has a value")
        .to_owned()
}

/// The review at the case revision, and `e1` of kind `fresh` (max_age 1h), observed 73 years
/// after the instant and bound to revision r0, which revision binding excludes.
fn evidence() -> [String; 2] {
    [
        record("e0", "review", "r1", "2026-10-04T11:30:00Z"),
        record("e1", "fresh", "r0", "2099-01-01T00:00:00Z"),
    ]
}

#[test]
fn strict_refuses_a_future_record_every_order_form_reads_though_revision_binding_excluded_it() {
    // (form, value without e1, value with e1 by default): where the two differ, the form reads e1,
    // and only its timestamp (or its being there) decides it.
    let forms = [
        (
            "{first: {kind: fresh, result: pass}}",
            Some(("unknown", "true")),
        ),
        (
            "{before: {first: review, then: fresh}}",
            Some(("unknown", "true")),
        ),
        (
            "{after: {first: review, then: {kind: fresh, result: pass}}}",
            Some(("false", "true")),
        ),
        (
            "{since: {last: review, then: {kind: fresh, result: pass}}}",
            None,
        ),
    ];
    let [review, future] = evidence();
    for (form, values) in forms {
        let ir = protocol(form);
        if let Some((alone, with_future)) = values {
            let without = decide(&ir, std::slice::from_ref(&review), Some("0s")).expect("decides");
            assert_eq!(value(&without), alone, "{form}");
            let by_default =
                decide(&ir, &[review.clone(), future.clone()], None).expect("decides");
            assert_eq!(value(&by_default), with_future, "{form}");
        }
        match decide(&ir, &[review.clone(), future.clone()], Some("0s")) {
            Err(refusal) => {
                assert_eq!(refusal.code(), "future-evidence", "{form}: {refusal}");
                assert_eq!(
                    refusal.to_string(),
                    "evidence `e1` observed_at `2099-01-01T00:00:00Z` is later than the \
                     evaluation instant `2026-10-04T12:00:00Z` by 2279793600 s, more than the \
                     allowed clock skew `0s`",
                    "{form}"
                );
                let cited: Vec<&str> = refusal.evidence().iter().map(|id| id.as_str()).collect();
                assert_eq!(cited, ["e1"], "{form}");
            }
            Ok(decision) => panic!(
                "{form}: --strict 0s at {AT} accepted e1, observed_at 2099-01-01T00:00:00Z of \
                 kind `fresh` (max_age 1h), and the order form read it: claim `c` is {}",
                value(&decision)
            ),
        }
    }
}

/// Where the strict mode refuses nothing, the order forms decide as the default does.
#[test]
fn strict_refusing_nothing_leaves_every_order_form_the_default_bytes() {
    let evidence = [
        record("e0", "review", "r1", "2026-10-04T11:30:00Z"),
        record("e1", "fresh", "r0", "2026-10-04T11:45:00Z"),
        record("e2", "fresh", "r1", "2026-10-04T11:50:00Z"),
    ];
    for form in [
        "{first: {kind: fresh, result: pass}}",
        "{before: {first: review, then: fresh}}",
        "{after: {first: review, then: {kind: fresh, result: pass}}}",
        "{since: {last: review, then: {kind: fresh, result: pass}}}",
    ] {
        let ir = protocol(form);
        let strict = decide(&ir, &evidence, Some("0s")).expect("decides");
        let by_default = decide(&ir, &evidence, None).expect("decides");
        assert_eq!(eval::render(&strict), eval::render(&by_default), "{form}");
    }
}
