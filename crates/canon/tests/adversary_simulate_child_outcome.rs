//! Adversary pass 1 for story:simulate-child-outcome-step, against the library.
//!
//! The story claims, under "Why both claims still hold", that every printed finding is true of a
//! real case and that no dead end goes unreported. Both rest on what a child-outcome step may
//! add. The step is offered from every state in which a mark is set, for as long as the walk
//! runs, so a child that already ended can "end again" with another outcome, standing for a
//! second child. When the opening action's own guard closes after its one step, a real case has
//! exactly one child: `duplicate-child-case` refuses a second record for it, and no step of the
//! opening action can open another.
//!
//! - `ended-once`: the child ended `accepted` and the capability `licence`, which `publish`
//!   requires, is denied. The evaluator finds nothing admissible, nothing approval-required and
//!   no outcome legitimate, and no record can arrive any more. The walk counts that state as
//!   waiting, because a second, unlisted outcome would make `declined` legitimate, and prints no
//!   finding at all: a dead end goes unreported (claim 2). The base printed a finding for it
//!   (with its limit line).
//! - `either-once`: the walk prints a `stuck-state` holding `alpha` and `beta`, reached by two
//!   child-outcome steps after one step of `open`, whose guard then closes. A real case cannot
//!   hold both (claim 1); the reference page of `canon simulate` states no limit that covers it.
//!
//! Two cases pin § Outcome 4 ("R5 containment also requires every mark of A to be set in B.
//! `size` counts set marks"), which no test of the unit reads: with the marks left out of
//! `records_within`, or out of `size`, in `simulate/mod.rs`, every test of the unit stays green.

use std::path::PathBuf;

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir::{self, Ir};
use b10x_canon::model;
use b10x_canon::simulate::{self, Finding};

fn fixture(name: &str) -> Ir {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/child-outcome-adversary")
        .join(name)
        .join("protocol.yaml");
    let source = std::fs::read_to_string(&path).expect("the fixture reads");
    ir::compile(&model::parse(&source).expect("parses")).expect("compiles")
}

fn case(protocol: &str) -> model::Case {
    eval::read_case(&format!(
        "format: canon-case/1\nid: C-1\nprotocol: {protocol}\nartifacts: {{thing: {{revision: r1}}}}\n"
    ))
    .expect("a case")
}

/// A record about `thing` at `r1`: `kind`, with `result` and `child` when given.
fn record(
    id: &str,
    kind: &str,
    result: Option<&str>,
    child: Option<&str>,
) -> model::EvidenceRecord {
    let mut text = format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\nsubject: thing\nsubject_revision: r1\n\
         observed_at: 2026-01-01T00:00:00Z\n"
    );
    if let Some(result) = result {
        text.push_str(&format!("result: {result}\n"));
    }
    if let Some(child) = child {
        text.push_str(&format!("child_case: {child}\n"));
    }
    eval::read_evidence(&text).expect("a record")
}

fn statuses(section: &serde_json::Value) -> Vec<(String, String)> {
    section
        .as_object()
        .expect("a section")
        .iter()
        .map(|(id, entry)| {
            (
                id.clone(),
                entry["status"].as_str().unwrap_or("").to_owned(),
            )
        })
        .collect()
}

/// Claim 2: the real case `open`, its one child ends `accepted`, `publish` is denied. Nothing
/// can happen any more: `open` is blocked, a second record for the child is refused, and no
/// outcome is legitimate. The walk must report at least one finding.
#[test]
fn a_dead_end_after_the_only_child_ended_is_reported() {
    let ir = fixture("ended-once");
    let case = case("adversary.ended_once");
    let evidence = vec![
        record("e1", "opened", None, None),
        record("e2", "case_outcome", Some("accepted"), Some("C-2")),
    ];
    let denied = "- capability: licence\n  decision: denied\n";
    let decision = eval::evaluate_with(
        &ir,
        &case,
        &evidence,
        Supplied {
            authority: Some(denied),
            ..Supplied::default()
        },
    )
    .expect("evaluates");
    let actions = statuses(decision.actions.as_ref().expect("actions"));
    let outcomes = statuses(decision.outcomes.as_ref().expect("outcomes"));
    assert!(
        actions.iter().all(|(_, status)| status == "blocked")
            && outcomes.iter().all(|(_, status)| status == "blocked"),
        "the real case is a dead end: actions {actions:?}, outcomes {outcomes:?}"
    );
    // The child cannot end a second time.
    let mut again = evidence.clone();
    again.push(record("e3", "case_outcome", Some("declined"), Some("C-2")));
    let refused = eval::evaluate_with(
        &ir,
        &case,
        &again,
        Supplied {
            authority: Some(denied),
            ..Supplied::default()
        },
    )
    .expect_err("a second record for one child is refused");
    assert_eq!(refused.code(), "duplicate-child-case");

    let report = simulate::simulate(&ir).expect("walked");
    assert!(
        !report.is_clean(),
        "a real case reaches {{opened, child accepted, licence denied}} and cannot leave it, \
         and the walk reports no finding (it waits on a second outcome of the one child):\n\
         {report}"
    );
}

/// Claim 1: `open` takes one step and its guard closes, so a real case has one child. A printed
/// stuck state or cycle must not hold two terminal outcomes of it.
#[test]
fn no_printed_finding_holds_two_outcomes_of_the_only_child() {
    let ir = fixture("either-once");
    let case = case("adversary.either_once");
    let opened = vec![record("e1", "opened", None, None)];
    let decision = eval::evaluate(&ir, &case, &opened).expect("evaluates");
    assert_eq!(
        decision.actions.as_ref().expect("actions")["open"]["status"],
        "blocked",
        "`open` runs once"
    );
    let mut both = opened.clone();
    both.push(record("e2", "case_outcome", Some("alpha"), Some("C-2")));
    both.push(record("e3", "case_outcome", Some("beta"), Some("C-2")));
    assert_eq!(
        eval::evaluate(&ir, &case, &both)
            .expect_err("one child, two outcomes")
            .code(),
        "duplicate-child-case"
    );

    let report = simulate::simulate(&ir).expect("walked");
    let false_findings: Vec<String> = report
        .findings
        .iter()
        .filter_map(|finding| match finding {
            Finding::StuckState(dead) | Finding::Cycle { dead, .. } => Some(dead),
            _ => None,
        })
        .filter(|dead| {
            dead.state.contains("result `alpha`") && dead.state.contains("result `beta`")
        })
        .map(|dead| format!("{} after {}", dead.state, dead.path.join(", then ")))
        .collect();
    assert!(
        false_findings.is_empty(),
        "printed findings no real case reaches (two outcomes of the one child):\n{}\n\
         whole report:\n{report}",
        false_findings.join("\n")
    );
}

fn compiled(source: &str) -> Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

/// The stuck states and cycles of a walk, as printed.
fn dead_lines(report: &simulate::Report) -> Vec<String> {
    report
        .findings
        .iter()
        .filter(|finding| matches!(finding, Finding::StuckState(_) | Finding::Cycle { .. }))
        .map(ToString::to_string)
        .collect()
}

/// `{evidence mark}` is reached by `other`, `{evidence mark, action open opened a child}` by
/// `open`; both are stuck and the first is contained in the second, which holds one more thing,
/// its mark. One finding, standing for both.
const MARK_COUNTS_IN_SIZE: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1, description: A record from an opening action or another.}\n\
    artifacts: {thing: {description: The thing.}}\n\
    evidence_kinds:\n\
    \x20 mark: {description: A record either action makes.}\n\
    \x20 seal: {description: A record nothing makes.}\n\
    claims:\n\
    \x20 marked: {description: Marked., true_when: {evidence: {kind: mark}}}\n\
    \x20 sealed: {description: Sealed., true_when: {evidence: {kind: seal}}}\n\
    actions:\n\
    \x20 open: {description: Open a child once., precondition: {claim: marked, is: unknown}, \
    case_open: {protocol: q, subject: thing}, may_produce: [{evidence: mark}]}\n\
    \x20 other: {description: Mark without a child., precondition: {claim: marked, is: \
    unknown}, may_produce: [{evidence: mark}]}\n\
    outcomes:\n\
    \x20 done: {description: Sealed., requires: {claim: sealed}}\n";

#[test]
fn a_set_mark_counts_in_the_size_of_a_state() {
    let report = simulate::simulate(&compiled(MARK_COUNTS_IN_SIZE)).expect("walked");
    assert_eq!(
        dead_lines(&report),
        vec![
            "stuck-state: state {evidence `mark`}: no outcome is legitimate and no action is \
             admissible or approval-required; path: action `other` produces evidence `mark`; 1 \
             other stuck state contains it"
                .to_owned()
        ],
        "whole report:\n{report}"
    );
}

/// `{evidence first, action open opened a child}` is stuck, and so is `{evidence first,
/// evidence second, evidence third}`, reached without `open`. The second holds every record of
/// the first but not its mark, so it is not contained in it: two findings.
const MARK_COUNTS_IN_CONTAINMENT: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1, description: An opening state and a larger one without its \
    mark.}\n\
    artifacts: {thing: {description: The thing.}}\n\
    evidence_kinds:\n\
    \x20 first: {description: First.}\n\
    \x20 second: {description: Second.}\n\
    \x20 third: {description: Third.}\n\
    \x20 seal: {description: A record nothing makes.}\n\
    claims:\n\
    \x20 has_first: {description: First., true_when: {evidence: {kind: first}}}\n\
    \x20 has_second: {description: Second., true_when: {evidence: {kind: second}}}\n\
    \x20 has_third: {description: Third., true_when: {evidence: {kind: third}}}\n\
    \x20 sealed: {description: Sealed., true_when: {evidence: {kind: seal}}}\n\
    actions:\n\
    \x20 open: {description: Open a child before anything., precondition: {all: [{claim: \
    has_first, is: unknown}, {claim: has_second, is: unknown}]}, case_open: {protocol: q, \
    subject: thing}, may_produce: [{evidence: first}]}\n\
    \x20 begin: {description: Begin without a child., precondition: {all: [{claim: has_first, \
    is: unknown}, {claim: has_second, is: unknown}]}, may_produce: [{evidence: second}]}\n\
    \x20 follow: {description: Follow a beginning., precondition: {all: [{claim: has_second}, \
    {claim: has_first, is: unknown}]}, may_produce: [{evidence: first}]}\n\
    \x20 finish: {description: Finish once both are there., precondition: {all: [{claim: \
    has_first}, {claim: has_second}, {claim: has_third, is: unknown}]}, may_produce: \
    [{evidence: third}]}\n\
    outcomes:\n\
    \x20 done: {description: Sealed., requires: {claim: sealed}}\n";

#[test]
fn a_state_without_a_mark_does_not_contain_one_with_it() {
    let report = simulate::simulate(&compiled(MARK_COUNTS_IN_CONTAINMENT)).expect("walked");
    assert_eq!(
        dead_lines(&report),
        vec![
            "stuck-state: state {evidence `first`, action `open` opened a child}: no outcome is \
             legitimate and no action is admissible or approval-required; path: action `open` \
             produces evidence `first`"
                .to_owned(),
            "stuck-state: state {evidence `first`, evidence `second`, evidence `third`}: no \
             outcome is legitimate and no action is admissible or approval-required; path: \
             action `begin` produces evidence `second`, then action `follow` produces evidence \
             `first`, then action `finish` produces evidence `third`"
                .to_owned(),
        ],
        "whole report:\n{report}"
    );
}
