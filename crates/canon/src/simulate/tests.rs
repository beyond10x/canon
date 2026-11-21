use super::*;
use crate::{ir, model};

fn compiled(source: &str) -> Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

fn report(source: &str) -> String {
    simulate(&compiled(source)).expect("walked").to_string()
}

const HEAD: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1, description: A protocol.}\n\
    artifacts: {doc: {description: The document.}}\n";

/// A record from a producer of a profile is a step of its own, written with its producer; stuck
/// states of one path length are ordered by their rendering.
#[test]
fn a_producer_profile_is_a_step_of_its_own() {
    let source = format!(
        "{HEAD}\
         independence_dimensions: {{other: {{description: Another principal.}}}}\n\
         evidence_kinds:\n\
         \x20 review: {{description: A review.}}\n\
         \x20 ship: {{description: A shipment.}}\n\
         claims:\n\
         \x20 approved: {{description: Approved independently., true_when: {{evidence: {{kind: \
         review, result: approved, independent_on: [other]}}}}}}\n\
         \x20 shipped: {{description: Shipped., true_when: {{evidence: {{kind: ship}}}}}}\n\
         actions:\n\
         \x20 review: {{description: Review., precondition: {{claim: approved, is: unknown}}, \
         may_produce: [{{evidence: review}}]}}\n\
         outcomes:\n\
         \x20 shipped: {{description: Shipped., requires: {{claim: shipped}}}}\n"
    );
    assert_eq!(
        report(&source),
        "stuck-state: state {evidence `review` by a producer independent on `other`}: no outcome \
         is legitimate and no action is admissible or approval-required; path: action `review` \
         produces evidence `review` by a producer independent on `other`\n\
         stuck-state: state {evidence `review` result `approved` by a producer independent on \
         `other`}: no outcome is legitimate and no action is admissible or approval-required; \
         path: action `review` produces evidence `review` result `approved` by a producer \
         independent on `other`\n\
         unreached-outcome: outcome `shipped` is legitimate in no reachable state\n\
         no-outcome-reachable: protocol `p` revision 1 reaches no state in which an outcome is \
         legitimate\n\
         simulated: protocol `p` revision 1: 3 reachable states, 0 terminable (outcome `shipped` \
         0), 4 findings\n"
    );
}

/// A supplied record arrives by a step of its own, naming its supplier; a state a supplied
/// record can still leave waits, and the state no supplied record leaves is stuck.
#[test]
fn a_supplied_record_is_a_step_naming_its_supplier() {
    let source = format!(
        "{HEAD}\
         evidence_kinds:\n\
         \x20 peer_review: {{description: A peer review., supplied_by: q}}\n\
         \x20 publication: {{description: A publication.}}\n\
         \x20 veto: {{description: A veto., supplied_by: q}}\n\
         claims:\n\
         \x20 reviewed: {{description: Reviewed., true_when: {{evidence: {{kind: \
         peer_review}}}}}}\n\
         \x20 vetoed: {{description: Vetoed., true_when: {{evidence: {{kind: veto}}}}}}\n\
         \x20 published: {{description: Published., true_when: {{evidence: {{kind: \
         publication}}}}}}\n\
         actions:\n\
         \x20 publish: {{description: Publish., precondition: {{all: [{{claim: reviewed}}, \
         {{claim: vetoed, is: unknown}}]}}, may_produce: [{{evidence: publication}}]}}\n\
         outcomes:\n\
         \x20 published: {{description: Published., requires: {{claim: published}}}}\n"
    );
    assert_eq!(
        report(&source),
        "stuck-state: state {evidence `peer_review`, evidence `veto`}: no outcome is legitimate \
         and no action is admissible or approval-required; path: protocol `q` supplies evidence \
         `peer_review`, then protocol `q` supplies evidence `veto`\n\
         simulated: protocol `p` revision 1: 6 reachable states, 2 terminable (outcome \
         `published` 2), 2 waiting on supplied evidence, 1 finding\n"
    );
}

/// A decision is a step: an outcome that requires one is legitimate after it, and a state where
/// no action is admissible is stuck though a decision could still be taken there.
#[test]
fn a_decision_is_a_step() {
    let source = format!(
        "{HEAD}\
         evidence_kinds:\n\
         \x20 draft: {{description: A draft.}}\n\
         claims:\n\
         \x20 drafted: {{description: Drafted., true_when: {{evidence: {{kind: draft}}}}}}\n\
         actions:\n\
         \x20 write: {{description: Write., precondition: {{claim: drafted}}, may_produce: \
         [{{evidence: draft}}]}}\n\
         outcomes:\n\
         \x20 abandoned: {{description: Abandoned., requires: {{decision: abandon}}}}\n\
         \x20 done: {{description: Done., requires: {{claim: drafted}}}}\n"
    );
    assert_eq!(
        report(&source),
        "stuck-state: state {}: no outcome is legitimate and no action is admissible or \
         approval-required; path: empty\n\
         dead-action: action `write` is admissible in no reachable state\n\
         unreached-outcome: outcome `done` is legitimate in no reachable state\n\
         simulated: protocol `p` revision 1: 2 reachable states, 1 terminable (outcome \
         `abandoned` 1, outcome `done` 0), 3 findings\n"
    );
}

/// A dimension with more values than a state holds is refused, naming it, before any state is
/// walked: five classes in eight upstream variants each are `2^40` values.
#[test]
fn a_dimension_a_state_cannot_hold_is_refused() {
    let mut source = String::from(
        "format: protocol/1\n\
         protocol: {id: p, revision: 1, description: A protocol.}\n\
         artifacts:\n\
         \x20 doc: {description: The document.}\n\
         \x20 u1: {description: An upstream.}\n\
         \x20 u2: {description: An upstream.}\n\
         \x20 u3: {description: An upstream.}\n\
         evidence_kinds: {review: {description: A review.}}\n\
         claims:\n",
    );
    for result in ["a", "b", "c", "d"] {
        source.push_str(&format!(
            "  {result}: {{description: Result {result}., true_when: {{evidence: {{kind: review, \
             result: {result}}}}}}}\n"
        ));
    }
    source.push_str("invalidation:\n");
    for upstream in ["u1", "u2", "u3"] {
        source.push_str(&format!(
            "  {upstream}.moved: {{description: Moved., upstream: {upstream}, invalidates: [a]}}\n"
        ));
    }
    source.push_str(
        "actions: {review: {description: Review., may_produce: [{evidence: review}]}}\n\
         outcomes: {a: {description: A., requires: {claim: a}}}\n",
    );
    let refused = simulate(&compiled(&source)).expect_err("refused");
    assert_eq!(refused.code(), "reachable-state-bound");
    assert_eq!(
        refused.to_string(),
        "protocol `p` revision 1: evidence kind `review` with upstream `u1`, `u2`, `u3` \
         1099511627776 values, more than a state of the walk holds"
    );
}

/// The walk is a pure function: the same IR gives the same report.
#[test]
fn the_same_protocol_gives_the_same_report() {
    let source = format!(
        "{HEAD}\
         evidence_kinds: {{note: {{description: A note.}}}}\n\
         claims: {{noted: {{description: Noted., true_when: {{evidence: {{kind: note}}}}}}}}\n\
         actions: {{note: {{description: Note., requires: [{{capability: c}}], may_produce: \
         [{{evidence: note}}]}}}}\n\
         outcomes: {{noted: {{description: Noted., requires: {{claim: noted}}}}}}\n"
    );
    let ir = compiled(&source);
    assert_eq!(simulate(&ir), simulate(&ir));
}
