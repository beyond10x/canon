//! Adversary pass 1 on story:simulate-revise-step, against the library.
//!
//! § Outcome 1: "In the successor, every artifact of S is at a new revision, and then the record
//! is added at the current revision." No test of the unit tells that order from the reverse one:
//! with the record added first and the revision applied after, a record about a revised artifact
//! is added and set aside in one step, and every unit and command-line test of the unit still
//! passes. The case below reaches a state only through a revise step that adds a record about the
//! artifact it revises.

use b10x_canon::ir::{self, Ir};
use b10x_canon::model;
use b10x_canon::simulate::simulate;

fn compiled(source: &str) -> Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

fn report(source: &str) -> String {
    simulate(&compiled(source)).expect("walked").to_string()
}

/// `fix` is admissible after a test of `doc` that did not pass; it revises `doc` and may record a
/// `note` about `doc`. Its revise step with the note sets the test aside and then adds the note
/// at the new revision, so the state holding the note alone is reached, and it is stuck: `check`
/// waits for no note and `fix` for a failed test. Adding the note before the revision would set
/// the note aside too, and the state would not be reached.
#[test]
fn a_revise_step_adds_its_record_after_the_revision() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1, description: A protocol.}\n\
        artifacts: {doc: {description: The document.}}\n\
        evidence_kinds: {note: {description: A note.}, test: {description: A test.}}\n\
        claims:\n\
        \x20 noted: {description: Noted., true_when: {evidence: {kind: note, subject: doc}}}\n\
        \x20 passed: {description: Passed., true_when: {evidence: {kind: test, subject: doc, \
        result: pass}}}\n\
        actions:\n\
        \x20 check: {description: Check., precondition: {all: [{claim: passed, is: unknown}, \
        {claim: noted, is: unknown}]}, may_produce: [{evidence: test}]}\n\
        \x20 fix: {description: Fix., precondition: {claim: passed, is: false}, revises: [doc], \
        may_produce: [{evidence: note}]}\n\
        outcomes:\n\
        \x20 done: {description: Done., requires: {all: [{claim: passed}, {claim: noted}]}}\n";
    assert_eq!(
        report(source),
        "stuck-state: state {evidence `test` about `doc` result `pass`}: no outcome is \
         legitimate and no action is admissible or approval-required; path: action `check` \
         produces evidence `test` about `doc` result `pass`\n\
         stuck-state: state {evidence `note` about `doc`}: no outcome is legitimate and no \
         action is admissible or approval-required; path: action `check` produces evidence \
         `test` about `doc`, then action `fix` revises `doc` and produces evidence `note` about \
         `doc`\n\
         cycle: state {evidence `note` about `doc`, evidence `test` about `doc`}: no outcome is \
         legitimate, no action is approval-required, and every record step of action `fix` \
         returns this state; path: action `check` produces evidence `test` about `doc`, then \
         action `fix` produces evidence `note` about `doc`; every revision by action `fix` leads \
         to a state no outcome is reachable from\n\
         unreached-outcome: outcome `done` is legitimate in no reachable state\n\
         no-outcome-reachable: protocol `p` revision 1 reaches no state in which an outcome is \
         legitimate\n\
         simulated: protocol `p` revision 1: 5 reachable states, 0 terminable (outcome `done` \
         0), 5 findings (3 stuck or cycle states)\n"
    );
}
