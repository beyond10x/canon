//! Adversary cases for story:obligations (wave 2026-10-04-w7, pass 1): the published statements
//! about the `obligations` section, driven against what the library now writes for it.

use std::path::PathBuf;

use b10x_canon::{eval, ir, model};

fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir).join("../..")
}

fn read(path: &str) -> String {
    let full = repository_root().join(path);
    std::fs::read_to_string(&full).unwrap_or_else(|error| panic!("{}: {error}", full.display()))
}

const CASE: &str = "format: canon-case/1\nid: INV-18\nprotocol: investigation\nartifacts:\n  explanation: {revision: r1}\n";

/// The decision a fixture gives `INV-18` with no evidence.
fn decision(fixture: &str) -> model::Decision {
    let protocol = model::parse(&read(fixture)).expect("fixture parses");
    let compiled = ir::compile(&protocol).expect("fixture compiles");
    let case = eval::read_case(CASE).expect("case reads");
    eval::evaluate(&compiled, &case, &[]).expect("evaluates")
}

/// Every line of `path` holding one of `phrases`, as `path:line: text`.
fn holding(path: &str, phrases: &[&str]) -> Vec<String> {
    read(path)
        .lines()
        .enumerate()
        .filter(|(_, line)| phrases.iter().any(|phrase| line.contains(phrase)))
        .map(|(at, line)| format!("{path}:{}: {}", at + 1, line.trim()))
        .collect()
}

/// `crates/canon/src/eval/mod.rs` module docs, published as `website/docs/reference/evaluation.md`:
/// "The stages, sections and inputs of steps 2, 3 and 5 are not built yet: ... each section is
/// absent ... So the decision is the one three-valued claim evaluation gives, byte for byte." The
/// obligations section of step 5 is built now, and present for every protocol that declares an
/// obligation.
#[test]
fn the_evaluation_reference_no_longer_says_every_section_is_absent() {
    let written = decision("fixtures/investigation/obligations.yaml");
    assert!(
        written.obligations.is_some(),
        "control: a protocol declaring an obligation gets the obligations section"
    );
    // Adapted in the wave-w7 docs pass: the binding and freshness merges had already rewritten the
    // "each section is absent" sentence, so on the merged tree this case passed before any doc was
    // fixed. The other built-section statements the outcomes adversary (pass 1, F4) found are
    // checked beside it, in the same two pages and in the comment on `claims::predicate`.
    let stale: Vec<String> = [
        "crates/canon/src/eval/mod.rs",
        "crates/canon/src/eval/claims.rs",
        "website/docs/reference/evaluation.md",
    ]
    .into_iter()
    .flat_map(|path| {
        holding(
            path,
            &[
                "each section is absent",
                "each other section is absent",
                "story:outcomes); it refuses nothing yet",
                "section stubs do not evaluate anything yet",
            ],
        )
    })
    .collect();
    assert_eq!(stale, Vec::<String>::new(), "stale statements");
}

/// The generated worked example (`crates/canon-docs/src/pages.rs`) and the hand-written pages
/// still say obligations are not evaluated, or that their evaluation is planned. AGENTS.md
/// § Public documentation: "Mark every feature a hand-written page describes as shipped or
/// planned, and update the status page when a story lands."
#[test]
fn no_page_still_says_obligations_are_not_evaluated() {
    let written = decision("fixtures/investigation/obligations.yaml");
    assert!(
        written.obligations.is_some(),
        "control: obligations are evaluated"
    );
    let stale: Vec<String> = [
        ("crates/canon-docs/src/pages.rs", "obligations, actions \\"),
        (
            "website/docs/reference/investigation-example.md",
            "obligations, actions and outcomes are not",
        ),
        (
            "website/docs/status/where-this-stands.md",
            "| Obligations | Planned |",
        ),
        (
            "website/docs/concepts/evaluation.md",
            "which obligations are open, which actions",
        ),
        (
            "website/docs/concepts/protocols.md",
            "Evaluating obligations as open or discharged is planned.",
        ),
    ]
    .into_iter()
    .flat_map(|(path, phrase)| holding(path, &[phrase]))
    .collect();
    assert_eq!(stale, Vec::<String>::new(), "stale statements");
}

/// `website/docs/reference/documents.md` (generated from `model/decision.rs`) lists the
/// `canon-decision/1` key `obligations` as present "always". The decision of a protocol that
/// declares no obligation has no `obligations` key, as story:obligations requires.
#[test]
fn the_documents_reference_says_when_the_obligations_key_is_present() {
    let row: Vec<String> = holding(
        "website/docs/reference/documents.md",
        &["| `obligations` | any JSON value |"],
    );
    assert_eq!(row.len(), 1, "one row for the key: {row:?}");
    let without = eval::render(&decision("fixtures/investigation/protocol.yaml"));
    let with = eval::render(&decision("fixtures/investigation/obligations.yaml"));
    assert!(
        with.contains("\"obligations\""),
        "control: written for a protocol declaring an obligation"
    );
    assert!(
        !without.contains("\"obligations\""),
        "control: not written for a protocol declaring none"
    );
    assert!(
        !row[0].contains("| always |"),
        "the reference says the key is always present, and it is absent here:\n{}\n{without}",
        row[0]
    );
}
