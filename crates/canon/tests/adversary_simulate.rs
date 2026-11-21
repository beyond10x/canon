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
/// with an empty path and no state waits.
#[test]
fn a_supplied_kind_nothing_reads_does_not_turn_a_stuck_state_into_waiting() {
    let report = simulate::simulate(&compiled(MUTUAL_WITH_NOTE)).expect("walked");
    let first = report.findings.first().map(ToString::to_string);
    let expected = "stuck-state: state {}: no outcome is legitimate and no action is admissible \
                    or approval-required; path: empty";
    assert!(
        first.as_deref() == Some(expected) && report.waiting == Some(0),
        "expected the first finding {expected:?} and 0 waiting, got {:?} and {:?}:\n{report}",
        first,
        report.waiting
    );
}
