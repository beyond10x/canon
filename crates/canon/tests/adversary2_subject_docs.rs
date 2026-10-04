//! Adversary pass 2 for story:subject-bound-evidence-match: the evaluation reference against what
//! the evaluator does with a subject-bound match.
//!
//! `website/docs/reference/evaluation.md` is generated from the `crate::eval` module docs. Its
//! `## Sections` bullet on `actions` gives the evidence reason as `{"evidence": <kind>, "present":
//! <whether a record of the kind applies>}`, and the paragraph after the evaluation steps says a
//! record an exclusion stage keeps out is "listed as excluded under each claim that reaches its
//! kind". Each case first observes the behaviour, then reads the page at run time and checks it
//! says what was observed.

use std::path::PathBuf;

use b10x_canon::eval;
use b10x_canon::ir;
use b10x_canon::model;
use serde_json::json;

fn page(relative: &str) -> String {
    let root = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set by cargo"),
    )
    .join("../..");
    std::fs::read_to_string(root.join(relative)).unwrap_or_else(|e| panic!("{relative}: {e}"))
}

/// The markdown bullet that starts with `start`, up to the next top-level bullet or blank line.
fn bullet<'a>(text: &'a str, start: &str) -> &'a str {
    let at = text
        .find(start)
        .unwrap_or_else(|| panic!("no bullet `{start}`"));
    let rest = &text[at..];
    let end = rest[1..]
        .find("\n- ")
        .map(|i| i + 1)
        .into_iter()
        .chain(rest.find("\n\n"))
        .min()
        .unwrap_or(rest.len());
    &rest[..end]
}

fn compile(sections: &str) -> ir::Ir {
    let source = format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\n\
         artifacts: {{change: {{}}, docs: {{}}}}\nevidence_kinds: {{test_result: {{}}}}\n{sections}"
    );
    ir::compile(&model::parse(&source).expect("parses")).expect("compiles")
}

fn case() -> model::Case {
    eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\n\
         artifacts: {change: {revision: c1}, docs: {revision: d1}}\n",
    )
    .expect("case reads")
}

fn record(id: &str, subject: &str, revision: &str) -> model::EvidenceRecord {
    eval::read_evidence(&format!(
        "format: canon-evidence/1\nid: {id}\nkind: test_result\nresult: pass\n\
         subject: {subject}\nsubject_revision: {revision}\n"
    ))
    .expect("evidence reads")
}

/// Behaviour: a test result about the docs applies (it is bound to the docs' current revision and
/// nothing excludes it), so a record of the kind applies; the action's reason nonetheless says
/// `present: false` and carries `subject`. The reference's `actions` bullet must give the reason
/// shape the evaluator writes: it names no `subject` key and defines `present` as "whether a
/// record of the kind applies", which here would be `true`.
#[test]
fn the_evaluation_reference_gives_the_evidence_reason_the_evaluator_writes() {
    let ir = compile(
        "actions:\n  merge: {precondition: {evidence: {kind: test_result, result: pass, subject: change}}}\n",
    );
    let decision =
        eval::evaluate(&ir, &case(), &[record("t-docs", "docs", "d1")]).expect("decides");
    let reasons = &decision.actions.as_ref().expect("actions")["merge"]["reasons"];
    assert_eq!(
        reasons,
        &json!([{"evidence": "test_result", "present": false, "subject": "change"}]),
        "the observed behaviour"
    );

    let reference = page("website/docs/reference/evaluation.md");
    let actions = bullet(&reference, "- `actions` maps each declared action");
    assert!(
        actions.contains("\"subject\""),
        "the reference's action reason has no `subject` key, which the evaluator writes:\n{actions}"
    );
    assert!(
        !actions.contains("whether a record of the kind applies"),
        "the reference defines `present` as whether a record of the kind applies; one does here \
         and the evaluator says `false`:\n{actions}"
    );
}

/// Behaviour: a test result about the docs, bound to a superseded docs revision, is excluded as
/// `revision_mismatch`, and `tests.pass` reaches its kind; it is not listed under `tests.pass`,
/// whose only match is bound to the change. The reference's summary of the exclusion stages must
/// not say every excluded record is listed under each claim that reaches its kind.
#[test]
fn the_evaluation_reference_says_which_claims_list_an_excluded_record() {
    let ir = compile(
        "claims:\n  tests.pass: {true_when: {evidence: {kind: test_result, result: pass, subject: change}}}\n",
    );
    let decision =
        eval::evaluate(&ir, &case(), &[record("t-docs", "docs", "d0")]).expect("decides");
    let (_, entry) = decision.claims.iter().next().expect("one claim");
    assert!(
        entry.excluded_evidence.is_empty(),
        "the observed behaviour: nothing listed"
    );

    let reference = page("website/docs/reference/evaluation.md");
    let summary = reference
        .split("\n\n")
        .find(|paragraph| paragraph.starts_with("Revision binding excludes a record"))
        .expect("the summary paragraph");
    assert!(
        !summary.contains("each listed as excluded under each claim that reaches its kind."),
        "the reference says an excluded record is listed under each claim that reaches its kind; \
         `tests.pass` reaches `test_result` and lists nothing:\n{summary}"
    );
}
