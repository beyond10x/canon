//! story:simulate-opens-child-with-its-record, against the library: a child's `open` record that
//! meets no demand of its readers (found by the review of wave 2026-11-23-w13).
//!
//! The opening step adds the child's `open` record "from a producer of the dimension's first
//! profile (the empty one when some reader demands no independence)" (`simulate/mod.rs`, module
//! docs, Steps). When every reader of the `case_outcome` dimension demands independence, the
//! first profile is a demand, so the walk records every child as opened by a producer that meets
//! it. The same module docs say a producer profile is a free choice of each step, "as a runner
//! may use any producer", and `canon check`'s space holds a record "without a producer when
//! something that demands nothing reads it" and otherwise none. A runner whose `open` record does
//! not meet the demand (the runner itself, or the child's own principal) gives a case in which the
//! child runs and no record of it meets the demand. The walk before the story reached that state;
//! the walk after it does not, and reports an outcome that holds there as legitimate in no
//! reachable state.

use b10x_canon::ir::{self, Ir};
use b10x_canon::model;
use b10x_canon::simulate;

fn compiled(source: &str) -> Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

/// One child about `doc`; `case_outcome` is read only through `independent_on: [other_team]`.
const INDEPENDENT_READERS: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1, description: A child read only through an independence demand.}\n\
    artifacts: {doc: {description: The document.}}\n\
    independence_dimensions:\n\
    \x20 other_team: {description: A producer of another team.}\n\
    evidence_kinds:\n\
    \x20 case_outcome: {description: The outcome of one child case.}\n\
    \x20 opened: {description: The child was opened.}\n\
    claims:\n\
    \x20 opened: {description: Opened., true_when: {evidence: {kind: opened}}}\n\
    \x20 running:\n\
    \x20   description: The child runs, as a producer of another team recorded.\n\
    \x20   true_when: {evidence: {kind: case_outcome, subject: doc, result: open, independent_on: [other_team]}}\n\
    \x20 recorded:\n\
    \x20   description: Some record of the child, from a producer of another team, is there.\n\
    \x20   true_when: {evidence: {kind: case_outcome, subject: doc, independent_on: [other_team]}}\n\
    actions:\n\
    \x20 open:\n\
    \x20   description: Open the child once.\n\
    \x20   precondition: {claim: opened, is: unknown}\n\
    \x20   case_open: {protocol: child, subject: doc}\n\
    \x20   may_produce: [{evidence: opened}]\n\
    outcomes:\n\
    \x20 unseen:\n\
    \x20   description: The child was opened and no record of it from a producer of another team is there.\n\
    \x20   requires:\n\
    \x20     all:\n\
    \x20       - {claim: opened}\n\
    \x20       - {claim: recorded, is: unknown}\n\
    \x20 ran:\n\
    \x20   description: The child was seen running.\n\
    \x20   requires: {claim: running}\n";

/// The state "opened, the child runs, its `open` record from a producer that does not meet the
/// demand" is one a runner reaches, and `unseen` holds in it.
#[test]
fn an_open_record_that_meets_no_demand_is_reachable() {
    let report = simulate::simulate(&compiled(INDEPENDENT_READERS))
        .expect("walked")
        .to_string();
    assert!(
        !report
            .lines()
            .any(|line| line.starts_with("unreached-outcome: outcome `unseen`")),
        "{report}"
    );
}
