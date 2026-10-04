//! Adversary pass 1 for story:evidence-revision-binding: published pages that still say a
//! record's subject and subject revision do not affect evaluation, after this story made them
//! decide the result.

use std::path::PathBuf;

use b10x_canon::eval;
use b10x_canon::ir;
use b10x_canon::model::{self, Truth};

/// Read at run time: a shared build directory may reuse this binary across worktrees.
fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

/// The subject revision alone moves the claim from TRUE to UNKNOWN.
fn subject_revision_affects_the_result() -> bool {
    let ir = ir::compile(
        &model::parse(
            "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
             evidence_kinds: {k: {}}\nclaims:\n  c: {true_when: {evidence: {kind: k}}}\n",
        )
        .expect("parses"),
    )
    .expect("compiles");
    let record = eval::read_evidence(
        "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r1\n",
    )
    .expect("reads");
    let at = |revision: &str| {
        let case = eval::read_case(&format!(
            "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {{a: {{revision: {revision}}}}}\n"
        ))
        .expect("case reads");
        eval::evaluate(&ir, &case, std::slice::from_ref(&record))
            .expect("decides")
            .claims
            .iter()
            .next()
            .expect("one claim")
            .1
            .value
    };
    at("r1") == Truth::True && at("r2") == Truth::Unknown
}

#[test]
fn no_published_page_says_subject_revision_does_not_affect_evaluation() {
    assert!(subject_revision_affects_the_result(), "precondition");
    let root = repository_root();
    let stale = [
        (
            "website/docs/reference/documents.md",
            "they do not yet affect evaluation",
        ),
        (
            "website/docs/reference/investigation-example.md",
            "subject revision are read but do not yet affect the result",
        ),
        (
            "website/docs/concepts/evidence-and-revisions.md",
            "do not affect evaluation yet",
        ),
        (
            "website/docs/status/where-this-stands.md",
            "Records name a revision today, but it does not affect the result",
        ),
    ];
    let found: Vec<String> = stale
        .iter()
        .filter_map(|(page, phrase)| {
            let text = std::fs::read_to_string(root.join(page))
                .unwrap_or_else(|error| panic!("{page}: {error}"));
            let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
            flat.contains(phrase)
                .then(|| format!("{page}: \"{phrase}\""))
        })
        .collect();
    assert!(
        found.is_empty(),
        "pages still say the binding fields do not affect evaluation:\n{}",
        found.join("\n")
    );
}
