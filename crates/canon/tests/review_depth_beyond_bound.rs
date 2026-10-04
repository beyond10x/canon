//! Independent review of wave 2026-10-04-w7: no input reaches a process abort through the library
//! API with a caller-built IR.
//!
//! `eval::evaluate` takes any `Ir` a caller builds, and fails closed on the other states only a
//! caller can build: a claim cycle is refused as `claim-cycle` (`eval/claims.rs`), an unreadable
//! `max_age` as `invalid-max-age` (`eval/freshness.rs`). The predicate nesting bound,
//! `MAX_IR_DEPTH`, is enforced only by `read_ir`, on text. A caller-built IR whose predicate
//! nests far beyond that bound is evaluated on the 64 MiB `DEEP_STACK` thread anyway, and the
//! recursive claim walk overflows it: the process aborts with `stack overflow` instead of
//! returning a decision or a refusal.
//!
//! An abort cannot be caught inside one test, so the parent test runs the evaluation in a child
//! process (this same test binary, one ignored test selected by name) and asserts that the child
//! exits normally.

use std::process::Command;

use b10x_canon::eval;
use b10x_canon::ir;
use b10x_canon::model::{self, ClaimId, Predicate};

/// Far beyond `MAX_IR_DEPTH` (4096), and far beyond what a 64 MiB stack holds at one frame per
/// level.
const NOTS: usize = 2_000_000;

const CHILD_ENV: &str = "CANON_REVIEW_DEPTH_CHILD";

/// The evaluation itself. Ignored so the ordinary suite runs it only through the parent below.
#[test]
#[ignore = "run as a child process by a_caller_built_ir_nested_beyond_the_bound_does_not_abort"]
fn child_evaluates_a_caller_built_ir_nested_beyond_the_bound() {
    if std::env::var_os(CHILD_ENV).is_none() {
        return;
    }
    let mut built = ir::compile(
        &model::parse(
            "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
             evidence_kinds: {k: {}}\nclaims: {c: {true_when: {evidence: {kind: k}}}}\n",
        )
        .expect("parses"),
    )
    .expect("compiles");
    let mut deep = built.claims[&ClaimId::new("c")].true_when.clone();
    for _ in 0..NOTS {
        deep = Predicate::Not(Box::new(deep));
    }
    built
        .claims
        .get_mut(&ClaimId::new("c"))
        .expect("c is compiled")
        .true_when = deep;
    let case = eval::read_case(
        "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r}}\n",
    )
    .expect("case reads");
    // A decision or a refusal are both acceptable; reaching the next line is the point.
    let result = eval::evaluate(&built, &case, &[]);
    println!("evaluated: {}", result.is_ok());
    // Dropping the predicate recurses too; the question here is the evaluator, not `Drop`.
    std::mem::forget(built);
}

#[test]
fn a_caller_built_ir_nested_beyond_the_bound_does_not_abort() {
    let child = Command::new(std::env::current_exe().expect("test binary path"))
        .args([
            "--ignored",
            "--exact",
            "child_evaluates_a_caller_built_ir_nested_beyond_the_bound",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD_ENV, "1")
        .output()
        .expect("child test process runs");
    let stdout = String::from_utf8_lossy(&child.stdout);
    let stderr = String::from_utf8_lossy(&child.stderr);
    assert!(
        child.status.success() && stdout.contains("evaluated: "),
        "evaluating a caller-built IR {NOTS} `not`s deep must return a decision or a refusal; \
         the process ended with {:?}\nstderr:\n{}",
        child.status,
        stderr.lines().take(5).collect::<Vec<_>>().join("\n")
    );
}
