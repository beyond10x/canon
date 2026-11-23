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
/// Re-pinned by story:simulate-opens-child-with-its-record, whose opening step adds the
/// child's `open` record: 36 states, the same one finding, and still one dimension without a
/// subject.
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
        child of action `open1` records evidence `case_outcome`, replacing result `open`; 17 other \
        stuck states contain it\n\
        simulated: protocol `probe.subjects.2` revision 1: 36 reachable states, 1 terminable \
        (outcome `built` 1), 2 waiting on supplied evidence, a child outcome or expiry, 1 finding \
        (18 stuck or cycle states)";
    assert_eq!(report(source).trim_end(), expected);
}

fn findings(report: &str) -> Vec<&str> {
    report
        .lines()
        .filter(|line| !line.starts_with("simulated:"))
        .collect()
}

const LATE: &str =
    include_str!("../../../fixtures/investigation/simulate/revise-child-late/protocol.yaml");

/// `revise-child-late` with artifact `a` renamed `c`: one child case about `b`, opened once;
/// `fix` revises `b` while no child has ended. A child that ends `done` against the revision it
/// was opened at leaves `done.current` unknown and `done.any` true, so neither action is
/// admissible again and the outcome is never legitimate: a stuck state. The fixture as named
/// reports it (`… result `done` earlier`); the renamed protocol must report it too, and the same
/// number of findings.
#[test]
fn a_child_about_the_spaces_own_subject_may_end_against_the_earlier_revision() {
    let renamed = LATE
        .replace("  a:\n", "  c:\n")
        .replace("        subject: a\n", "        subject: c\n");
    assert_ne!(renamed, LATE, "the rename applied");
    let named = report(LATE);
    let text = report(&renamed);
    assert_eq!(
        findings(&text).len(),
        findings(&named).len(),
        "renaming an artifact changed the dead ends reported\nas named:\n{named}\nrenamed:\n{text}"
    );
    assert!(
        findings(&text)
            .iter()
            .any(|line| line.contains("result `done` earlier")),
        "the child that ended `done` against the revision it was opened at is not reported:\n{text}"
    );
}

const CHILD: &str =
    include_str!("../../../fixtures/investigation/simulate/revise-child/protocol.yaml");

/// Decided with story:simulate-child-about-own-artifact: a child about the artifact the space
/// already uses ends against an earlier revision only where a predicate reads an earlier record
/// differently from a current one. `revise-child` reads its children with a form without
/// `revision: current`, which counts both alike, so the walk of the fixture keeps its 9 states;
/// with `revision: current` added the child's late outcome is walked as well.
///
/// Re-pinned by story:simulate-opens-child-with-its-record. The opening step adds the child's
/// `open` record, which the revision holds as earlier, so the two states of an opened child with
/// no record after the revision are gone and both walks reach 8 states: the count no longer tells
/// them apart. The step does: with `revision: current` a printed path holds the late step, a
/// child that records its outcome `… earlier`; the fixture as it is prints only its summary.
#[test]
fn a_child_about_the_spaces_own_subject_is_walked_late_only_where_a_claim_tells_the_difference() {
    let any = report(CHILD);
    let summary = |text: &str| text.lines().last().expect("a summary").to_owned();
    assert!(summary(&any).contains(": 8 reachable states"), "{any}");
    assert_eq!(any.lines().count(), 1, "{any}");
    let current = CHILD.replace(
        "        subject: doc\n",
        "        subject: doc\n        revision: current\n",
    );
    assert_ne!(current, CHILD, "the key was added");
    let current = report(&current);
    assert!(
        summary(&current).contains(": 8 reachable states"),
        "{current}"
    );
    assert!(
        current.lines().any(|line| line.contains(
            "then a child of action `open_review` records evidence `case_outcome` about `doc` \
             result `accepted` earlier"
        )),
        "{current}"
    );
}
