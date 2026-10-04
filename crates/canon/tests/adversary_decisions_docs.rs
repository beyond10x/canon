//! Adversary pass 1 for story:decision-outcomes (wave 2026-10-04-w8): the generated reference page
//! of evaluation documents, read against the model this unit added.
//!
//! `website/docs/reference/documents.md` says of itself: "Generated from the model types in
//! `crates/canon/src/model`. `canon evaluate` reads a compiled protocol, one case snapshot and a set
//! of evidence records, and writes a decision. Every key is listed". This unit adds a model type
//! `canon evaluate` reads, `ExplicitDecision` (`model/explicit.rs`, `DECISIONS_FORMAT =
//! "canon-decisions/1"`), and lists it in `ess_model_matches::DOCUMENTS` beside `EvidenceRecord`
//! and `Decision`, but the page generator's `EVALUATION_DOCUMENTS`
//! (`crates/canon-docs/src/pages.rs`) still names only `Case`, `EvidenceRecord` and `Decision`.
//! So the keys of a `canon-decisions/1` entry, and the `Principal` identifier, appear on no
//! reference page that lists keys; only the prose of `evaluation.md` names them.
//!
//! Read at run time from `CARGO_MANIFEST_DIR`, so a binary reused from a shared build directory
//! reads this tree's page.

use std::path::PathBuf;

fn documents_page() -> String {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    let path = PathBuf::from(manifest_dir).join("../../website/docs/reference/documents.md");
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// The page that lists every key of every document `canon evaluate` reads lists the
/// `canon-decisions/1` explicit decision and its four keys.
#[test]
fn the_documents_page_lists_the_canon_decisions_1_entry_and_its_keys() {
    let page = documents_page();
    assert!(
        page.contains("## `canon-decisions/1`"),
        "documents.md has no `canon-decisions/1` section, though `canon evaluate --decisions` reads \
         one and its model type is in crates/canon/src/model/explicit.rs"
    );
    for key in ["decision", "outcome", "principal", "case_revision"] {
        assert!(
            page.contains(&format!("| `{key}` |")),
            "documents.md lists no key `{key}`"
        );
    }
    assert!(
        page.contains("| `Principal` |"),
        "documents.md's identifier table has no `Principal`"
    );
}

/// `Revision` now also identifies the case's own revision (`Case.revision`) and the case revision
/// a decision was taken at (`ExplicitDecision.case_revision`); the identifier table still says it
/// identifies only an artifact's revision.
#[test]
fn the_revision_identifier_is_not_described_as_an_artifacts_only() {
    let page = documents_page();
    let row = page
        .lines()
        .find(|line| line.starts_with("| `Revision` |"))
        .expect("documents.md has a `Revision` identifier row");
    assert_ne!(
        row, "| `Revision` | Identifies one revision of an artifact. |",
        "the `Revision` row describes only artifacts, but the case snapshot's `revision` is one"
    );
}
