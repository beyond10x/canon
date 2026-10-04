//! Adversary pass 2 for story:decision-outcomes (wave 2026-10-04-w8): the published documents and
//! the hand-written status rows, read against what the evaluator now writes and reads.
//!
//! What the evaluator writes, measured by the unit's own scenario `CANON-OUTCOME-002`
//! (`conformance/scenarios/decision-outcomes.yaml`) and `eval/outcomes.rs`: a decided outcome is
//! `{"status": "legitimate", "decided_by": {"decision": …, "principal": …}}`, and a blocked one
//! carries the reason `{"decision": …, "present": false}`. Neither shape is in CANON-OUTCOME-001.
//!
//! Read at run time from `CARGO_MANIFEST_DIR`, so a binary reused from a shared build directory
//! reads this tree's files.

use std::path::PathBuf;

fn read(relative: &str) -> String {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    let path = PathBuf::from(manifest_dir).join("../..").join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// The one table row of `page` whose first cell contains `first_cell`.
fn row<'a>(page: &'a str, first_cell: &str) -> &'a str {
    page.lines()
        .find(|line| {
            line.starts_with('|')
                && line
                    .split('|')
                    .nth(1)
                    .is_some_and(|cell| cell.contains(first_cell))
        })
        .unwrap_or_else(|| panic!("no table row whose first cell contains {first_cell:?}"))
}

/// `website/docs/reference/documents.md`, generated from `model/decision.rs`, says of the
/// `canon-decision/1` `outcomes` section: "Its shape is fixed by CANON-OUTCOME-001." The section
/// now has two entry shapes CANON-OUTCOME-001 does not hold (`decided_by`, and the reason
/// `{"decision", "present"}`), fixed by CANON-OUTCOME-002. A consumer that reads the page as the
/// contract is told the shape is CANON-OUTCOME-001's and meets a key it was never told about.
#[test]
fn the_documents_page_names_the_scenario_that_fixes_the_decided_outcome_shape() {
    let page = read("website/docs/reference/documents.md");
    let outcomes = row(&page, "`outcomes`");
    assert!(
        outcomes.contains("CANON-OUTCOME-002") || outcomes.contains("decided_by"),
        "the `outcomes` row of the canon-decision/1 table does not name CANON-OUTCOME-002 or \
         `decided_by`, while the evaluator writes both decided-outcome shapes:\n{outcomes}"
    );
}

/// The status rows are hand-written in `website/status.yaml` and rendered from
/// `website/data/status.json`; the brief asks that they stay true. The "Evaluation documents" row
/// links the documents page, which now lists four documents (`canon-case/1`, `canon-evidence/1`,
/// `canon-decisions/1`, `canon-decision/1`); the row still names three.
#[test]
fn the_status_row_for_evaluation_documents_names_canon_decisions_1() {
    let status: serde_json::Value =
        serde_json::from_str(&read("website/data/status.json")).expect("the status file is JSON");
    let documents = status["items"]
        .as_array()
        .expect("the status file has items")
        .iter()
        .find(|item| item["label"] == "Evaluation documents")
        .expect("a status row labelled `Evaluation documents`")["detail"]
        .as_str()
        .expect("the row has a detail");
    assert!(
        documents.contains("canon-decisions/1"),
        "the shipped `Evaluation documents` row does not name canon-decisions/1:\n{documents}"
    );
}

/// The published JSON Schema for `canon-decisions/1` accepts a list that repeats an entry
/// exactly, which `canon evaluate --decisions` refuses as `duplicate-identifier`
/// (`eval/decisions.rs`). Exact repetition is the one refusal JSON Schema can state
/// (`uniqueItems`); the `Identifier` definition in the same file shows the house style when the
/// schema cannot state a rule (its description says what Canon also refuses). This schema does
/// neither, so a document a validator passes is refused by Canon with nothing in the schema
/// saying why.
#[test]
fn the_decisions_schema_states_that_a_repeated_entry_is_refused() {
    let text = read("website/static/schemas/decisions-1.schema.json");
    let schema: serde_json::Value = serde_json::from_str(&text).expect("the schema is JSON");
    let unique = schema["uniqueItems"] == serde_json::Value::Bool(true);
    let described = schema["description"]
        .as_str()
        .is_some_and(|text| text.contains("more than once") || text.contains("repeat"));
    assert!(
        unique || described,
        "decisions-1.schema.json neither sets `uniqueItems` nor says a repeated entry is refused:\n\
         uniqueItems = {}\ndescription = {}",
        schema["uniqueItems"],
        schema["description"]
    );
}
