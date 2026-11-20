//! Adversary cases, first pass, for story:protocol-imports (wave 2026-11-20-w2).
//!
//! § Outcome 4 refuses "an identifier two imports declare differently (two identical declarations
//! are one declaration)". `ir/mod.rs` says what makes two documents the same: "documents that differ
//! only in the ways listed above give the same bytes" (the order of `all` members, of `may_produce`,
//! of a case input's values, an absent precondition and `{"all": []}`, ...). `ir::compose` compares
//! the parsed source declarations with `!=` instead, so two parts whose declaration compiles to the
//! same `canon-ir/1` are refused as `conflicting-imports`.
//!
//! Each case builds two parts, `a` and `b`, that declare one identifier written two ways, and
//! asserts first that each way compiles to the same IR (so the two declarations are identical in
//! `canon-ir/1`'s own terms), then that a root importing both composes into the IR of the
//! flattened document that declares it once.

use b10x_canon::ir;
use b10x_canon::model::{self, Protocol};

fn parse(source: &str) -> Protocol {
    model::parse(source).unwrap_or_else(|error| panic!("{error}\n{source}"))
}

fn compiled(source: &str) -> String {
    ir::compile(&parse(source))
        .unwrap_or_else(|problems| panic!("{problems:?}\n{source}"))
        .canonical_json()
}

/// A part `id` holding `body` after its header.
fn part(id: &str, body: &str) -> String {
    format!("format: protocol/1\nprotocol:\n  id: {id}\n  revision: 1\n{body}")
}

/// Composes a root importing parts `a` (with `first`) and `b` (with `second`), and returns the
/// composed IR, or the refusal codes and messages.
fn compose_two(first: &str, second: &str) -> Result<String, String> {
    let a = parse(&part("a", first));
    let b = parse(&part("b", second));
    let pin = |document: &Protocol| {
        ir::digest(&ir::compile(document).unwrap_or_else(|problems| panic!("{problems:?}")))
    };
    let root = parse(&format!(
        "format: protocol/1\nprotocol:\n  id: root\n  revision: 1\nimports:\n  a:\n    revision: 1\n    digest: \"{}\"\n  b:\n    revision: 1\n    digest: \"{}\"\n",
        pin(&a),
        pin(&b)
    ));
    ir::compose(&root, &[a, b])
        .and_then(|composed| ir::compile(&composed))
        .map(|ir| ir.canonical_json())
        .map_err(|problems| {
            problems
                .iter()
                .map(|problem| format!("{}: {problem}", problem.code()))
                .collect::<Vec<_>>()
                .join("\n")
        })
}

/// The two ways compile to the same IR, and the composition of two parts holding them is the IR
/// of the root declaring the first way once.
fn assert_identical_declarations_compose(first: &str, second: &str) {
    assert_eq!(
        compiled(&part("same", first)),
        compiled(&part("same", second)),
        "the two ways are not identical in canon-ir/1"
    );
    let flattened = compiled(&part("root", first));
    match compose_two(first, second) {
        Ok(ir) => assert_eq!(ir, flattened),
        Err(refusal) => {
            panic!("two declarations that compile to the same canon-ir/1 are refused:\n{refusal}")
        }
    }
}

const KINDS: &str = "evidence_kinds:\n  e1: {}\n  e2: {}\n";

#[test]
fn claims_whose_all_members_are_written_in_another_order_are_one_declaration() {
    assert_identical_declarations_compose(
        &format!(
            "{KINDS}claims:\n  c:\n    true_when:\n      all:\n        - evidence: {{kind: e1}}\n        - evidence: {{kind: e2}}\n"
        ),
        &format!(
            "{KINDS}claims:\n  c:\n    true_when:\n      all:\n        - evidence: {{kind: e2}}\n        - evidence: {{kind: e1}}\n"
        ),
    );
}

#[test]
fn an_action_without_a_precondition_and_one_with_the_empty_all_are_one_declaration() {
    assert_identical_declarations_compose(
        "actions:\n  act:\n    description: x\n",
        "actions:\n  act:\n    description: x\n    precondition: {all: []}\n",
    );
}

#[test]
fn actions_whose_may_produce_is_written_in_another_order_are_one_declaration() {
    assert_identical_declarations_compose(
        &format!(
            "{KINDS}actions:\n  act:\n    may_produce: [{{evidence: e1}}, {{evidence: e2}}]\n"
        ),
        &format!(
            "{KINDS}actions:\n  act:\n    may_produce: [{{evidence: e2}}, {{evidence: e1}}]\n"
        ),
    );
}

#[test]
fn case_inputs_whose_values_are_written_in_another_order_are_one_declaration() {
    assert_identical_declarations_compose(
        "case:\n  inputs:\n    sev: {values: [low, high]}\n",
        "case:\n  inputs:\n    sev: {values: [high, low]}\n",
    );
}
