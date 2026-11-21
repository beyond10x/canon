//! Adversary pass 1 for story:canon-simulate, against the library.
//!
//! - The story: the walk "is refused with `reachable-state-bound` when it would visit more than
//!   `check::STATE_BOUND` (65,536) states", and "so can [be simulated] a protocol the check refuses
//!   whose reachable part fits". A kind read by a claim five upstream artifacts can invalidate has
//!   2^64 values in the check's space, and the walk reaches three states; it refuses the protocol
//!   as `reachable-state-bound` because the dimension does not fit a `u32`. Four upstream
//!   artifacts (2^32 values) walk.
//! - Every finding is sound: an action reported dead is admissible in no state a real case
//!   reaches. A record of a kind with `max_age` lapses, and the story does not exclude that. The
//!   walk never lets a record lapse, so an action admissible once an observation has expired is
//!   reported `dead-action`, while the evaluator, at an instant two hours on, finds it admissible.
//! - A state is counted as waiting on supplied evidence only when a supplied step leads out of
//!   it. A supplied kind nothing reads changes the state and nothing else: the empty state of
//!   `mutual` stays stuck, with an empty path, and no state waits.

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir::{self, Ir};
use b10x_canon::model;
use b10x_canon::simulate::{self, Finding};

fn compiled(source: &str) -> Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

/// A review accepted once, with `upstreams` upstream artifacts each invalidating the claim.
fn wide(upstreams: usize) -> String {
    let mut source = String::from(
        "format: protocol/1\n\
         protocol: {id: p, revision: 1, description: A protocol.}\n\
         artifacts:\n\
         \x20 analysis: {description: The analysis.}\n",
    );
    for at in 1..=upstreams {
        source.push_str(&format!("  up_{at}: {{description: An input.}}\n"));
    }
    source.push_str(
        "evidence_kinds: {review: {description: A review.}}\n\
         claims:\n\
         \x20 accepted: {description: Accepted., true_when: {evidence: {kind: review, result: \
         accepted}}}\n\
         invalidation:\n",
    );
    for at in 1..=upstreams {
        source.push_str(&format!(
            "  up_{at}.revised: {{description: Moved., upstream: up_{at}, invalidates: \
             [accepted]}}\n"
        ));
    }
    source.push_str(
        "actions:\n\
         \x20 review: {description: Review once., precondition: {claim: accepted, is: unknown}, \
         may_produce: [{evidence: review}]}\n\
         outcomes:\n\
         \x20 accepted: {description: Accepted., requires: {claim: accepted}}\n",
    );
    source
}

/// Reachable states: `{}`, `{accepted}` (terminable) and `{another result}` (stuck). The
/// reachable part fits the bound whatever the number of upstream artifacts, since a walk's record
/// is always observed after every move.
#[test]
fn a_dimension_wider_than_a_u32_is_walked_when_its_reachable_part_fits() {
    let mut failures = Vec::new();
    for upstreams in [4, 5] {
        match simulate::simulate(&compiled(&wide(upstreams))) {
            Ok(report) if report.states == 3 && report.terminable == 1 => {}
            Ok(report) => failures.push(format!("{upstreams} upstreams: walked\n{report}")),
            Err(refusal) => failures.push(format!(
                "{upstreams} upstreams: refused {}: {refusal}",
                refusal.code()
            )),
        }
    }
    assert!(
        failures.is_empty(),
        "a protocol whose reachable part is three states is not simulated:\n{}",
        failures.join("\n")
    );
}

const EXPIRY: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1, description: A protocol.}\n\
    artifacts: {doc: {description: The document.}}\n\
    evidence_kinds:\n\
    \x20 observation: {description: An observation valid for an hour., max_age: 1h}\n\
    \x20 seal: {description: A seal.}\n\
    \x20 renewal: {description: A renewal.}\n\
    claims:\n\
    \x20 observed: {description: Observed., true_when: {evidence: {kind: observation}}}\n\
    \x20 sealed: {description: Sealed., true_when: {evidence: {kind: seal}}}\n\
    \x20 renewed: {description: Renewed., true_when: {evidence: {kind: renewal}}}\n\
    actions:\n\
    \x20 observe: {description: Observe., may_produce: [{evidence: observation}]}\n\
    \x20 place_seal: {description: Seal while observed., precondition: {claim: observed}, \
    may_produce: [{evidence: seal}]}\n\
    \x20 renew: {description: Renew once the observation expired., precondition: {all: [{claim: \
    sealed}, {claim: observed, is: unknown}]}, may_produce: [{evidence: renewal}]}\n\
    outcomes:\n\
    \x20 renewed: {description: Renewed., requires: {claim: renewed}}\n";

fn record(id: &str, kind: &str, observed_at: &str) -> String {
    format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\nsubject: doc\nsubject_revision: r1\n\
         observed_at: {observed_at}\n"
    )
}

/// The case `observe`, then `place_seal`, then two hours: the evaluator finds `renew`
/// admissible. The walk must not report it dead, nor `renewed` unreached.
#[test]
fn an_action_admissible_once_a_record_expires_is_not_reported_dead() {
    let ir = compiled(EXPIRY);
    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {doc: {revision: r1}}\n",
    )
    .expect("a case");
    let evidence = [
        record("e1", "observation", "2026-01-01T00:00:00Z"),
        record("e2", "seal", "2026-01-01T00:10:00Z"),
    ]
    .iter()
    .map(|text| eval::read_evidence(text).expect("a record"))
    .collect::<Vec<_>>();
    let decision = eval::evaluate_with(
        &ir,
        &case,
        &evidence,
        Supplied {
            at: Some("2026-01-01T02:00:00Z"),
            ..Supplied::default()
        },
    )
    .expect("evaluates");
    let actions = decision.actions.expect("actions");
    assert_eq!(
        actions["renew"]["status"], "admissible",
        "the real case reaches `renew`"
    );

    let report = simulate::simulate(&ir).expect("walked");
    let false_findings: Vec<String> = report
        .findings
        .iter()
        .filter(|finding| {
            matches!(finding, Finding::DeadAction { action } if action.as_str() == "renew")
                || matches!(finding, Finding::UnreachedOutcome { outcome } if outcome.as_str() == "renewed")
        })
        .map(ToString::to_string)
        .collect();
    assert!(
        false_findings.is_empty(),
        "findings a real case refutes (renew is admissible once the observation expired):\n{}\n\
         whole report:\n{report}",
        false_findings.join("\n")
    );
}

const MUTUAL_WITH_NOTE: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1, description: A protocol.}\n\
    artifacts: {paper: {description: The paper.}}\n\
    evidence_kinds:\n\
    \x20 draft: {description: A draft.}\n\
    \x20 note: {description: A note nothing reads., supplied_by: q}\n\
    \x20 review: {description: A review.}\n\
    claims:\n\
    \x20 drafted: {description: Drafted., true_when: {evidence: {kind: draft}}}\n\
    \x20 reviewed: {description: Reviewed., true_when: {evidence: {kind: review}}}\n\
    actions:\n\
    \x20 review_draft: {description: Review., precondition: {claim: drafted}, may_produce: \
    [{evidence: review}]}\n\
    \x20 write_draft: {description: Write., precondition: {claim: reviewed}, may_produce: \
    [{evidence: draft}]}\n\
    outcomes:\n\
    \x20 done: {description: Done., requires: {all: [{claim: drafted}, {claim: reviewed}]}}\n";

/// `{}` is stuck: the supplied `note` leads to `{note}`, which is stuck too. So `{}` is reported
/// with an empty path and no state waits. Pass 2: `{note}` contains `{}`, so R5 (one finding per
/// smallest state) ends the finding with the count of the other stuck state.
#[test]
fn a_supplied_kind_nothing_reads_does_not_turn_a_stuck_state_into_waiting() {
    let report = simulate::simulate(&compiled(MUTUAL_WITH_NOTE)).expect("walked");
    let first = report.findings.first().map(ToString::to_string);
    let expected = "stuck-state: state {}: no outcome is legitimate and no action is admissible \
                    or approval-required; path: empty; 1 other stuck state contains it";
    assert!(
        first.as_deref() == Some(expected) && report.waiting == Some(0),
        "expected the first finding {expected:?} and 0 waiting, got {:?} and {:?}:\n{report}",
        first,
        report.waiting
    );
}

/// Pass 2. A `case_outcome` kind that declares `max_age`: open a child case while unlocked, lock
/// once every child ended accepted, and the outcome needs both.
const CHILD_EXPIRY: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1, description: A protocol.}\n\
    artifacts: {findings: {description: The findings.}}\n\
    evidence_kinds:\n\
    \x20 case_outcome: {description: The outcome of a child case., max_age: 1h}\n\
    \x20 lock: {description: A lock.}\n\
    claims:\n\
    \x20 resolved: {description: Every child ended accepted., true_when: {children: {in: \
    [accepted]}}}\n\
    \x20 locked: {description: Locked., true_when: {evidence: {kind: lock}}}\n\
    actions:\n\
    \x20 open_change: {description: Open a child while unlocked., precondition: {claim: locked, \
    is: unknown}, case_open: {protocol: q, subject: findings}}\n\
    \x20 lock: {description: Lock once resolved., precondition: {claim: resolved}, may_produce: \
    [{evidence: lock}]}\n\
    outcomes:\n\
    \x20 done: {description: Resolved and locked., requires: {all: [{claim: resolved}, {claim: \
    locked}]}}\n";

/// R2: the state after an expiry step evaluates as a case in which that record is past its
/// `max_age`. The `children` form reads every record the evidence check accepted, expired ones
/// included (`eval/composition.rs`: "an expiry ... does not undo" a child's outcome). The real
/// case: the child ended accepted, the case locked, two hours pass; `done` stays legitimate. At
/// 4b36d52 the walk dropped the `case_outcome` record and reported `{evidence lock}` stuck.
#[test]
fn an_expired_child_outcome_still_counts_for_the_children_form() {
    let ir = compiled(CHILD_EXPIRY);
    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {findings: {revision: r1}}\n",
    )
    .expect("a case");
    let evidence = [
        "format: canon-evidence/1\nid: e1\nkind: case_outcome\nsubject: findings\n\
         subject_revision: r1\nchild_case: C-2\nresult: accepted\n\
         observed_at: 2026-01-01T00:00:00Z\n"
            .to_owned(),
        "format: canon-evidence/1\nid: e2\nkind: lock\nsubject: findings\nsubject_revision: r1\n\
         observed_at: 2026-01-01T00:10:00Z\n"
            .to_owned(),
    ]
    .iter()
    .map(|text| eval::read_evidence(text).expect("a record"))
    .collect::<Vec<_>>();
    let decision = eval::evaluate_with(
        &ir,
        &case,
        &evidence,
        Supplied {
            at: Some("2026-01-01T02:00:00Z"),
            ..Supplied::default()
        },
    )
    .expect("evaluates");
    let outcomes = decision.outcomes.expect("outcomes");
    assert_eq!(
        outcomes["done"]["status"], "legitimate",
        "the real case keeps `done` once the child outcome has expired"
    );

    // Re-pinned after the R2a correction: what this case means is that no finding rests on an
    // expired child outcome. The findings left are true: a child that ended outside the listed
    // outcomes keeps the `children` form false for good, past every `max_age`. Re-pinned again by
    // story:simulate-child-outcome-step, under which a child's outcome arrives after its opening
    // whether `open_change` is admissible or not: the smallest such states are now the `cycle`
    // with the unlisted outcome alone, where `open_change` only opens another child, and the
    // `stuck-state` after `lock`, where nothing is admissible. Re-pinned under the count of
    // children (D1): the stuck state is reached by two children, one accepted and one unlisted,
    // with none left; the cycle holds two or more children not ended, where a step of
    // `open_change` no longer changes the count.
    let report = simulate::simulate(&ir).expect("walked");
    let dead: Vec<(&simulate::Dead, bool)> = report
        .findings
        .iter()
        .filter_map(|finding| match finding {
            Finding::StuckState(dead) => Some((dead, false)),
            Finding::Cycle { dead, .. } => Some((dead, true)),
            _ => None,
        })
        .collect();
    let expired_child = dead.iter().any(|(dead, _)| {
        dead.path
            .iter()
            .any(|step| step.contains("evidence `case_outcome`") && step.ends_with(" expires"))
    });
    let lock_reported = dead
        .iter()
        .any(|(dead, _)| dead.state == "{evidence `lock`}");
    let only_the_unlisted = matches!(
        dead.as_slice(),
        [(stuck, false), (cycle, true)] if stuck.state
            == "{evidence `case_outcome` result `accepted`, evidence `case_outcome`, evidence \
                `lock`}"
            && cycle.state
                == "{evidence `case_outcome`, action `open_change` opened 2 or more children}"
    );
    assert!(
        !expired_child && !lock_reported && only_the_unlisted,
        "expected no finding whose path has an expiry of `case_outcome`, no finding at \
         `{{evidence lock}}`, and two findings, both at a state holding an unlisted child \
         outcome: the stuck state after `lock` and the cycle before it; whole report:\n{report}"
    );
}
