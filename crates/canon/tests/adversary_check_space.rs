//! Adversary pass 1 for story:canon-check (wave 2026-10-04-w10), against the state space
//! `crates/canon/src/check/space.rs` builds.
//!
//! The contract read is the module docs of `check`: "An evidence kind's classes are each distinct
//! result the protocol's predicates match for the kind", where the predicates are the claims,
//! obligations, action preconditions and outcome requirements; and "A space of more than
//! `STATE_BOUND` states (65 536) is refused", so a space of exactly 65 536 is checked.
//!
//! The acceptance fixtures match results only inside claims, and their bound fixture has 98 304
//! states, so a class collector that reads claims alone, or a bound compared with `<`, passes every
//! case the unit wrote. Each case here fails on one of those mutants.

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

/// The result `ready` is matched only by `gate`'s precondition. Without it as a class, `gate`'s
/// precondition is never `true` and the check would report a defect the protocol does not have.
#[test]
fn a_result_only_an_action_precondition_matches_is_a_class() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
        evidence_kinds: {k: {}}\n\
        actions:\n\
        \x20\x20probe: {may_produce: [{evidence: k}]}\n\
        \x20\x20gate: {precondition: {evidence: {kind: k, result: ready}}}\n";
    assert_eq!(
        report(source),
        "checked: protocol `p` revision 1: 4 states, 0 properties, 0 findings\n"
    );
}

/// The result `passed` is matched only by `done`'s requirement, and `bad` only under a `not` in
/// `clean`'s. Without them as classes, `done` would be reported unreachable.
#[test]
fn a_result_only_an_outcome_requirement_matches_is_a_class() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
        evidence_kinds: {k: {}, m: {}}\n\
        actions:\n\
        \x20\x20probe: {may_produce: [{evidence: k}, {evidence: m}]}\n\
        outcomes:\n\
        \x20\x20done: {requires: {evidence: {kind: k, result: passed}}}\n\
        \x20\x20clean: {requires: {not: {evidence: {kind: m, result: bad}}}}\n";
    assert_eq!(
        report(source),
        "checked: protocol `p` revision 1: 16 states, 0 properties, 0 findings\n"
    );
}

/// Sixteen kinds, none matched with a result: two values each, 2^16 = 65 536 states, which is the
/// bound and not more than it, so the space is checked. Seventeen kinds are refused.
#[test]
fn a_space_of_exactly_the_bound_is_checked_and_one_past_it_refused() {
    let kinds = |n: usize| {
        (0..n)
            .map(|i| format!("k{i:02}: {{}}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let source = |n: usize| {
        format!(
            "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nartifacts: {{a: {{}}}}\n\
             evidence_kinds: {{{}}}\n",
            kinds(n)
        )
    };
    assert_eq!(
        report(&source(16)),
        "checked: protocol `p` revision 1: 65536 states, 0 properties, 0 findings\n"
    );
    let dimensions = (0..17)
        .map(|i| format!("evidence kind `k{i:02}` 2"))
        .collect::<Vec<_>>()
        .join(", ");
    assert_eq!(
        report(&source(17)),
        format!(
            "refused state-space-bound: protocol `p` revision 1 has 131072 states, more than the \
             bound of 65536: {dimensions}"
        )
    );
}
