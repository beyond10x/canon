//! Adversarial cases for `canon diff` over a changed independence demand
//! (story:evidence-independence). Each case states the change, what the report says about it, and
//! a case snapshot evaluated under both revisions that shows what the change does to legitimate
//! work.

use b10x_canon::diff::{self, Category};
use b10x_canon::eval;
use b10x_canon::ir::{self, Ir};
use b10x_canon::model::{self, EvidenceRecord};
use serde_json::Value;

const HEADER: &str = "format: protocol/1\nprotocol: {id: p, revision: REV}\n\
    artifacts: {change: {}}\n\
    independence_dimensions: {different_principal: {}}\n\
    evidence_kinds: {review: {}}\n";

fn compiled(revision: u32, body: &str) -> Ir {
    let text = format!("{}{body}", HEADER.replace("REV", &revision.to_string()));
    ir::compile(&model::parse(&text).expect("parses")).expect("compiles")
}

/// The merge gate before: no review rejected the change, whoever produced it.
const GATE_PLAIN: &str =
    "actions:\n  merge: {precondition: {not: {evidence: {kind: review, result: rejected}}}}\n";

/// The merge gate after: `independent_on: [different_principal]` added to the match under `not`.
const GATE_DEMANDING: &str = "actions:\n  merge: {precondition: {not: {evidence: {kind: review, \
    result: rejected, independent_on: [different_principal]}}}}\n";

fn merge_status(ir: &Ir) -> String {
    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {change: {revision: r1}}\n\
         independence: {author: []}\n",
    )
    .expect("a case");
    let record: EvidenceRecord = eval::read_evidence(
        "format: canon-evidence/1\nid: v1\nkind: review\nresult: approved\nsubject: change\n\
         subject_revision: r1\nproducer: author\n",
    )
    .expect("a record");
    let decision = eval::evaluate(ir, &case, &[record]).expect("decides");
    let rendered: Value = serde_json::from_str(&eval::render(&decision)).expect("JSON");
    rendered["actions"]["merge"]["status"]
        .as_str()
        .expect("a status")
        .to_owned()
}

/// Adding a dimension to an evidence match's `independent_on` is a match that demands *more*
/// independence, wherever the match sits. Under `not` the report says the predicate "demands less
/// independence", which is false of the documents: the later revision names a dimension the earlier
/// did not, and no dimension was removed. (The read-in-reverse case already words the same
/// situation truthfully: "demands more independence, read in reverse".)
#[test]
fn a_dimension_added_under_not_is_not_reported_as_demanding_less_independence() {
    let report =
        diff::diff(&compiled(1, GATE_PLAIN), &compiled(2, GATE_DEMANDING)).expect("one protocol");
    let changes: Vec<(Category, String)> = report
        .changes
        .into_iter()
        .map(|change| (change.category, change.what))
        .collect();
    assert!(
        changes
            .iter()
            .all(|(_, what)| !what.contains("less independence")),
        "a dimension was added to `independent_on`, yet the report says: {changes:?}"
    );
}

/// `RELAXATION` means more work is legitimate, or less is required of it (`crate::diff`). Adding
/// `independent_on` under `not` is reported as `RELAXATION`, yet a case whose only review is the
/// author's approval had `merge` admissible before the change and has it blocked after: the
/// author's approval made `{evidence: review, result: rejected}` FALSE, so the gate held, and after
/// the change the match reads no record, is UNKNOWN, and `not` keeps it UNKNOWN. Less work is
/// legitimate in that case, so the change cannot be a relaxation.
#[test]
fn a_demand_added_under_not_that_blocks_an_admitted_case_is_not_a_relaxation() {
    let from = compiled(1, GATE_PLAIN);
    let to = compiled(2, GATE_DEMANDING);
    assert_eq!(merge_status(&from), "admissible", "before the change");
    assert_eq!(merge_status(&to), "blocked", "after the change");
    let report = diff::diff(&from, &to).expect("one protocol");
    let relaxations: Vec<String> = report
        .changes
        .iter()
        .filter(|change| change.category == Category::Relaxation)
        .map(|change| format!("{} `{}` {}", change.section, change.id, change.what))
        .collect();
    assert_eq!(
        relaxations,
        Vec::<String>::new(),
        "reported as RELAXATION although the author-only case went from admissible to blocked"
    );
}
