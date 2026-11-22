//! Adversary pass on story:child-outcome-recovery, against the library (review of wave
//! 2026-11-21).
//!
//! § Acceptance 6 of the story: every earlier fixture keeps its bytes. A protocol in which no
//! `case_open` subject is an artifact some action `revises` or another protocol revises
//! (`revised_by`) has no revision that could set a child's outcome aside, so the walk keeps the
//! space's one `case_outcome` dimension (`recovery::split`).

use b10x_canon::ir::{self, Ir};
use b10x_canon::model;
use b10x_canon::simulate::simulate;

fn compiled(source: &str) -> Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

fn report(source: &str) -> String {
    simulate(&compiled(source)).expect("walked").to_string()
}

/// Two actions open child cases about two artifacts, neither the least; no action declares
/// `revises` and no artifact `revised_by`, so no revision is walked and the subject of a child's
/// record changes no evaluation. `simulate/mod.rs` ("a protocol in which no action declares one
/// walks as before"), the `canon simulate` help ("An action without `revises` has no revise
/// step") and `ess/domains/cli.yaml` ("a protocol without revises walks as before") say it walks
/// as before; story:child-outcome-recovery § Acceptance 6 says every earlier fixture keeps its
/// bytes. Found in review: the walk split the `case_outcome` dimension per subject anyway
/// (`recovery::split` had no condition on a revisable subject): 400 states and 2 findings where
/// canon 0.10.0 gives the bytes below (68 states, 1 finding). With four such subjects 0.10.0
/// walks 644 states and that head refused `reachable-state-bound`; the fixtures `festival` and
/// `event` of `fixtures/child-outcome-large/` (no `revises`) changed the same way.
#[test]
fn a_protocol_that_revises_nothing_walks_child_outcomes_as_before() {
    let source = "format: protocol/1\n\
        protocol: {id: probe.subjects.2, revision: 1, description: Two subjects.}\n\
        artifacts: {a0: {description: Least.}, a1: {description: One.}, a2: {description: Two.}}\n\
        evidence_kinds: {case_outcome: {description: A child's outcome.}}\n\
        claims:\n\
        \x20 done.current: {description: Done., true_when: {children: {in: [done], revision: current}}}\n\
        actions:\n\
        \x20 open1: {description: Open., precondition: {claim: done.current, is: unknown}, \
        case_open: {protocol: probe.child, subject: a1}}\n\
        \x20 open2: {description: Open., precondition: {claim: done.current, is: unknown}, \
        case_open: {protocol: probe.child, subject: a2}}\n\
        outcomes: {built: {description: Built., requires: {claim: done.current}}}\n";
    let expected = "stuck-state: state {evidence `case_outcome`}: no outcome is legitimate and no \
        action is admissible or approval-required; path: action `open1` produces nothing, then a \
        child of action `open1` records evidence `case_outcome`; 33 other stuck states contain it\n\
        simulated: protocol `probe.subjects.2` revision 1: 68 reachable states, 9 terminable \
        (outcome `built` 9), 3 waiting on supplied evidence, a child outcome or expiry, 1 finding \
        (34 stuck or cycle states)";
    assert_eq!(report(source).trim_end(), expected);
}
