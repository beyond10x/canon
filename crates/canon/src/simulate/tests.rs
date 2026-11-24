use super::*;
use crate::eval::Supplied;
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
         0), 4 findings (2 stuck or cycle states)\n"
    );
}

/// A supplied record arrives by a step of its own, naming its supplier. A state a supplied
/// record leads out of waits; one only a supplied veto has reached is stuck, and its finding is
/// printed at its smallest state.
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
        "stuck-state: state {evidence `veto`}: no outcome is legitimate and no action is \
         admissible or approval-required; path: protocol `q` supplies evidence `veto`; 1 other \
         stuck state contains it\n\
         simulated: protocol `p` revision 1: 6 reachable states, 2 terminable (outcome \
         `published` 2), 1 waiting on supplied evidence or expiry, 1 finding (2 stuck or cycle \
         states)\n"
    );
}

/// story:supplied-evidence-required-at-open: a kind required at open starts the walk with one of
/// its records, one initial state per record the supplied step could add, so `{}` is not reached
/// and does not wait. A review handed in with another result than the one the action reads is a
/// stuck state with an empty path; the supplied step still adds the other class beside it.
#[test]
fn a_kind_required_at_open_starts_the_walk_with_each_of_its_records() {
    let source = format!(
        "{HEAD}\
         evidence_kinds:\n\
         \x20 review: {{description: A review., supplied_by: q, required_at_open: true}}\n\
         \x20 publication: {{description: A publication.}}\n\
         claims:\n\
         \x20 approved: {{description: Approved., true_when: {{evidence: {{kind: review, result: \
         approved}}}}}}\n\
         \x20 published: {{description: Published., true_when: {{evidence: {{kind: \
         publication}}}}}}\n\
         actions:\n\
         \x20 publish: {{description: Publish., precondition: {{claim: approved}}, may_produce: \
         [{{evidence: publication}}]}}\n\
         outcomes:\n\
         \x20 published: {{description: Published., requires: {{claim: published}}}}\n"
    );
    assert_eq!(
        report(&source),
        "stuck-state: state {evidence `review`}: no outcome is legitimate and no action is \
         admissible or approval-required; path: empty; 1 other stuck state contains it\n\
         simulated: protocol `p` revision 1: 5 reachable states, 2 terminable (outcome \
         `published` 2), 0 waiting on supplied evidence or expiry, 1 finding (2 stuck or cycle \
         states)\n"
    );
    let without = report(&source.replace(", required_at_open: true", ""));
    assert!(
        without.ends_with(
            "simulated: protocol `p` revision 1: 6 reachable states, 2 terminable (outcome \
             `published` 2), 1 waiting on supplied evidence or expiry, 1 finding (2 stuck or \
             cycle states)\n"
        ),
        "{without}"
    );
}

/// A decision is a step but no way out: a state from which only a decision leads to a
/// legitimate outcome is stuck, and its finding names the decision and the outcome.
#[test]
fn a_decision_is_named_on_the_finding_it_would_end() {
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
         \x20 dropped: {{description: Dropped., requires: {{decision: abandon}}}}\n\
         \x20 done: {{description: Done., requires: {{claim: drafted}}}}\n\
         \x20 halted: {{description: Halted., requires: {{decision: halt}}}}\n"
    );
    assert_eq!(
        report(&source),
        "stuck-state: state {}: no outcome is legitimate and no action is admissible or \
         approval-required; path: empty; decision `abandon` taken makes outcome `abandoned`, \
         outcome `dropped` legitimate, decision `halt` taken makes outcome `halted` legitimate\n\
         dead-action: action `write` is admissible in no reachable state\n\
         unreached-outcome: outcome `done` is legitimate in no reachable state\n\
         simulated: protocol `p` revision 1: 4 reachable states, 3 terminable (outcome \
         `abandoned` 2, outcome `done` 0, outcome `dropped` 2, outcome `halted` 2), 3 findings \
         (1 stuck or cycle state)\n"
    );
}

/// The number of values of a dimension is no reason to refuse (R1): five classes in eight
/// upstream variants each are `2^40` values in the check's space, and the walk reaches six
/// states.
#[test]
fn a_dimension_wider_than_a_state_value_is_walked() {
    let mut source = String::from(
        "format: protocol/1\n\
         protocol: {id: p, revision: 1, description: A protocol.}\n\
         artifacts:\n\
         \x20 doc: {description: The document.}\n\
         \x20 u1: {description: An upstream.}\n\
         \x20 u2: {description: An upstream.}\n\
         \x20 u3: {description: An upstream.}\n\
         evidence_kinds: {review: {description: A review.}}\n\
         claims:\n\
         \x20 reviewed: {description: Reviewed., true_when: {evidence: {kind: review}}}\n",
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
        "actions: {review: {description: Review once., precondition: {claim: reviewed, is: \
         unknown}, may_produce: [{evidence: review}]}}\n\
         outcomes: {a: {description: A., requires: {claim: a}}}\n",
    );
    let walked = simulate(&compiled(&source)).expect("walked");
    assert_eq!((walked.states, walked.terminable), (6, 1), "{walked}");
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
    \x20 place_seal: {description: Seal., precondition: {claim: observed}, may_produce: \
    [{evidence: seal}]}\n\
    \x20 renew: {description: Renew., precondition: {all: [{claim: sealed}, {claim: observed, \
    is: unknown}]}, may_produce: [{evidence: renewal}]}\n\
    outcomes:\n\
    \x20 renewed: {description: Renewed., requires: {claim: renewed}}\n";

/// The state after an expiry step evaluates as a case in which the record is past its `max_age`
/// (R2): the walk's `{seal}`, reached from `{observation, seal}` by `evidence `observation`
/// expires`, gives every action and outcome the status the evaluator gives the case with both
/// records, evaluated two hours after the observation.
#[test]
fn the_state_after_an_expiry_evaluates_as_a_case_past_its_max_age() {
    let ir = compiled(EXPIRY);
    let space = Space::new(&ir);
    let dimensions = space.walked();
    let base = space.initial_states().remove(0);
    let layout = Layout::new(&dimensions, base.len());
    let mut empty = base.clone();
    empty.resize(layout.length(), 0);
    let at = |kind: &str| {
        dimensions
            .iter()
            .position(|walked| walked.kind.as_str() == kind)
            .expect("a dimension of the kind")
    };
    let both = layout.add(
        &layout.add(&empty, at("observation"), 0, 0),
        at("seal"),
        0,
        0,
    );
    let expired = layout.expire(&both, at("observation"), 0);
    assert_eq!(expired, layout.add(&empty, at("seal"), 0, 0));
    let walked = check::evaluate_records(
        &ir,
        &space,
        &base,
        space.walk_evidence(&layout.held(&expired), &layout.expired(&expired)),
    )
    .expect("evaluates");

    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {doc: {revision: r1}}\n",
    )
    .expect("a case");
    let evidence: Vec<_> = [
        ("e1", "observation", "2026-01-01T00:00:00Z"),
        ("e2", "seal", "2026-01-01T00:10:00Z"),
    ]
    .iter()
    .map(|(id, kind, at)| {
        eval::read_evidence(&format!(
            "format: canon-evidence/1\nid: {id}\nkind: {kind}\nsubject: doc\n\
             subject_revision: r1\nobserved_at: {at}\n"
        ))
        .expect("a record")
    })
    .collect();
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
    let status = |section: &Option<model::Json>, id: &str| {
        section.as_ref().expect("a section")[id]["status"]
            .as_str()
            .expect("a status")
            .to_owned()
    };
    let real: Vec<String> = ir
        .actions
        .keys()
        .map(|id| status(&decision.actions, id.as_str()))
        .chain(
            ir.outcomes
                .keys()
                .map(|id| status(&decision.outcomes, id.as_str())),
        )
        .collect();
    let simulated: Vec<String> = walked.actions.into_iter().chain(walked.outcomes).collect();
    assert_eq!(simulated, real);
    assert_eq!(real, ["admissible", "blocked", "admissible", "blocked"]);
}

/// A `case_outcome` kind with a `max_age` that a `children` form and an evidence match read.
const HELD: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1, description: A protocol.}\n\
    artifacts: {findings: {description: The findings.}}\n\
    evidence_kinds:\n\
    \x20 case_outcome: {description: The outcome of a child case., max_age: 1h}\n\
    \x20 lock: {description: A lock.}\n\
    claims:\n\
    \x20 resolved: {description: Every child ended accepted., true_when: {children: {in: \
    [accepted]}}}\n\
    \x20 opened: {description: A fresh child outcome., true_when: {evidence: {kind: \
    case_outcome}}}\n\
    \x20 locked: {description: Locked., true_when: {evidence: {kind: lock}}}\n\
    actions:\n\
    \x20 open_change: {description: Open a child., precondition: {all: [{claim: opened, is: \
    unknown}, {claim: locked, is: unknown}]}, case_open: {protocol: q, subject: findings}}\n\
    \x20 lock: {description: Lock once resolved., precondition: {claim: resolved}, may_produce: \
    [{evidence: lock}]}\n\
    outcomes:\n\
    \x20 done: {description: Resolved and locked., requires: {all: [{claim: resolved}, {claim: \
    locked}]}}\n";

/// R2a: a `case_outcome` record held as expired evaluates as the real case two hours after the
/// child's outcome was recorded: the evidence match loses it, the `children` form still reads it.
#[test]
fn a_child_outcome_held_as_expired_evaluates_as_a_case_past_its_max_age() {
    let ir = compiled(HELD);
    let space = Space::new(&ir);
    let dimensions = space.walked();
    let base = space.initial_states().remove(0);
    let holds: Vec<bool> = dimensions
        .iter()
        .map(|walked| walked.kind.as_str() == CASE_OUTCOME)
        .collect();
    let layout = Layout::holding(&dimensions, base.len(), &holds);
    let mut empty = base.clone();
    empty.resize(layout.length(), 0);
    let at = |kind: &str| {
        dimensions
            .iter()
            .position(|walked| walked.kind.as_str() == kind)
            .expect("a dimension of the kind")
    };
    let outcome = at("case_outcome");
    let accepted = dimensions[outcome]
        .classes
        .iter()
        .position(|class| *class == Some("accepted"))
        .expect("an accepted class");
    let both = layout.add(&layout.add(&empty, outcome, accepted, 0), at("lock"), 0, 0);
    let expired = layout.expire(&both, outcome, accepted);
    assert_eq!(layout.held(&expired), [(at("lock"), 0, 0, vec![0])]);
    assert_eq!(layout.expired(&expired), [(outcome, accepted)]);
    assert_eq!(
        space.render_walk(
            &base,
            &layout.held(&expired),
            &layout.expired(&expired),
            &[]
        ),
        "{evidence `case_outcome` result `accepted` expired, evidence `lock`}"
    );
    let walked = check::evaluate_records(
        &ir,
        &space,
        &base,
        space.walk_evidence(&layout.held(&expired), &layout.expired(&expired)),
    )
    .expect("evaluates");

    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {findings: {revision: r1}}\n",
    )
    .expect("a case");
    let evidence: Vec<_> = [
        "format: canon-evidence/1\nid: e1\nkind: case_outcome\nsubject: findings\n\
         subject_revision: r1\nchild_case: C-2\nresult: accepted\n\
         observed_at: 2026-01-01T00:00:00Z\n",
        "format: canon-evidence/1\nid: e2\nkind: lock\nsubject: findings\nsubject_revision: r1\n\
         observed_at: 2026-01-01T00:10:00Z\n",
    ]
    .iter()
    .map(|text| eval::read_evidence(text).expect("a record"))
    .collect();
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
    let status = |section: &Option<model::Json>, id: &str| {
        section.as_ref().expect("a section")[id]["status"]
            .as_str()
            .expect("a status")
            .to_owned()
    };
    let real: Vec<String> = ir
        .actions
        .keys()
        .map(|id| status(&decision.actions, id.as_str()))
        .chain(
            ir.outcomes
                .keys()
                .map(|id| status(&decision.outcomes, id.as_str())),
        )
        .collect();
    let simulated: Vec<String> = walked.actions.into_iter().chain(walked.outcomes).collect();
    assert_eq!(simulated, real);
    assert_eq!(real, ["admissible", "blocked", "legitimate"]);
}

/// R2a, the `cycle` left when only a `children` form reads `case_outcome`: in it one child ended
/// with an outcome the form does not list, which no revision, expiry or invalidation undoes, so
/// the form stays `false` and the outcome that needs it is illegitimate in every later case. The
/// real case, with the walk's three child outcomes, at an instant past every `max_age`.
#[test]
fn a_child_that_ended_outside_the_list_keeps_the_children_form_false() {
    let ir = compiled(
        &HELD
            .replace(
                "\x20 opened: {description: A fresh child outcome., true_when: {evidence: {kind: \
         case_outcome}}}\n",
                "",
            )
            .replace(
                "{all: [{claim: opened, is: unknown}, {claim: locked, is: unknown}]}",
                "{claim: locked, is: unknown}",
            ),
    );
    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {findings: {revision: r1}}\n",
    )
    .expect("a case");
    let evidence: Vec<_> = [
        ("e1", "C-2", "accepted"),
        ("e2", "C-3", "open"),
        ("e3", "C-4", "other"),
    ]
    .iter()
    .map(|(id, child, result)| {
        eval::read_evidence(&format!(
            "format: canon-evidence/1\nid: {id}\nkind: case_outcome\nsubject: findings\n\
                 subject_revision: r1\nchild_case: {child}\nresult: {result}\n\
                 observed_at: 2026-01-01T00:00:00Z\n"
        ))
        .expect("a record")
    })
    .collect();
    for at in [None, Some("2026-01-02T00:00:00Z")] {
        let decision = eval::evaluate_with(
            &ir,
            &case,
            &evidence,
            Supplied {
                at,
                ..Supplied::default()
            },
        )
        .expect("evaluates");
        let resolved = decision
            .claims
            .get(&model::ClaimId::new("resolved"))
            .expect("the claim");
        assert_eq!(resolved.value, model::Truth::False, "at {at:?}");
        assert_eq!(
            decision.outcomes.expect("outcomes")["done"]["status"],
            "blocked",
            "at {at:?}"
        );
    }
}

/// story:simulate-child-outcome-step, which replaces R7: no line names a limit on child outcomes.
/// With an action that opens a child case the summary counts the states waiting on supplied
/// evidence, a child outcome or expiry; without one it is the summary of story:canon-simulate.
#[test]
fn a_protocol_that_opens_child_cases_states_no_limit() {
    let text = report(HELD);
    assert!(!text.contains("limit:"), "{text}");
    assert!(
        text.lines().last().is_some_and(|summary| summary
            .contains(" waiting on supplied evidence, a child outcome or expiry, ")),
        "{text}"
    );
    let without = report(&HELD.replace(", case_open: {protocol: q, subject: findings}", ""));
    assert!(!without.contains("limit:"), "{without}");
    assert!(
        without
            .lines()
            .last()
            .is_some_and(|summary| summary.contains(" waiting on supplied evidence or expiry, ")),
        "{without}"
    );
}

/// An artifact that declares `revised_by` adds the revision to what the waiting states wait on,
/// whatever else the summary names, and a protocol without one keeps the phrase it had.
#[test]
fn a_revision_by_another_protocol_is_named_in_the_summary() {
    let summary = |source: &str| report(source).lines().last().expect("a summary").to_owned();
    let revised = HELD.replace(
        "findings: {description: The findings.}",
        "findings: {description: The findings., revised_by: q}",
    );
    assert_ne!(revised, HELD);
    assert!(
        summary(&revised).contains(
            " waiting on supplied evidence, a child outcome, expiry or a revision by another \
             protocol, "
        ),
        "{}",
        summary(&revised)
    );
    let without_opening = revised.replace(", case_open: {protocol: q, subject: findings}", "");
    assert!(
        summary(&without_opening)
            .contains(" waiting on supplied evidence, expiry or a revision by another protocol, "),
        "{}",
        summary(&without_opening)
    );
    assert!(summary(HELD).contains(" waiting on supplied evidence, a child outcome or expiry, "));
}

/// story:simulate-child-outcome-step: a record of the class `open` held as expired (R2a) is an
/// open child, and a child that ends replaces it as it replaces a current one. In `HELD`, the
/// state in which the only child is open and its record expired lets `open_change` open another
/// child and nothing else, and leads out only when that open child ends accepted, replacing its
/// expired record: `lock`, then `done`. Without that successor it would be printed as a `cycle`
/// a real case does not have. `replace` drops exactly the records it names.
#[test]
fn a_child_that_ends_replaces_its_open_record_held_as_expired() {
    let text = report(HELD);
    let dead: Vec<&str> = text
        .lines()
        .filter(|line| line.starts_with("stuck-state: ") || line.starts_with("cycle: "))
        .collect();
    assert!(
        !dead.is_empty()
            && dead
                .iter()
                .all(|line| !line.contains("result `open` expired")),
        "{text}"
    );

    let ir = compiled(HELD);
    let space = Space::new(&ir);
    let dimensions = space.walked();
    let base = space.initial_states().remove(0);
    let holds: Vec<bool> = dimensions
        .iter()
        .map(|walked| walked.kind.as_str() == CASE_OUTCOME)
        .collect();
    let layout = Layout::holding(&dimensions, base.len(), &holds);
    let mut empty = base.clone();
    empty.resize(layout.length(), 0);
    let outcome = dimensions
        .iter()
        .position(|walked| walked.kind.as_str() == CASE_OUTCOME)
        .expect("a case_outcome dimension");
    let class = |name: &str| {
        dimensions[outcome]
            .classes
            .iter()
            .position(|class| *class == Some(name))
            .expect("the class")
    };
    let open = class(OPEN_OUTCOME);
    let held = layout.expire(&layout.add(&empty, outcome, open, 0), outcome, open);
    let both = layout.add(&held, outcome, open, 0);
    assert!(layout.held_expired(&both, outcome, open) && layout.present(&both, outcome, open));
    let replaced =
        |current, expired| layout.replace(&both, outcome, open, Replacing { current, expired });
    assert_eq!(replaced(false, true), layout.add(&empty, outcome, open, 0));
    assert_eq!(replaced(true, false), held);
    assert_eq!(replaced(true, true), empty);
}

/// A mark is a part of the state (R1 counts it): three opening actions admissible everywhere,
/// which record nothing, reach the 27 combinations of their marks (none, one, two or more each),
/// and each again with the one class of `case_outcome` a child may record, which a child's
/// outcome adds while lowering a mark or leaving one at two or more, 54 states, within the bound
/// of 3^k times the states without marks; without `case_open` the one empty state, each action's
/// step returning it. A step of an action at two or more children returns the state, so the
/// `cycle` is the state with every mark at two or more; the child-outcome step is no action step,
/// so that state with a child's outcome is a `cycle` too.
#[test]
fn every_combination_of_marks_is_a_state_of_its_own() {
    let source = format!(
        "{HEAD}\
         evidence_kinds: {{case_outcome: {{description: The outcome of a child case.}}}}\n\
         actions:\n\
         \x20 a: {{description: Open one., case_open: {{protocol: q, subject: doc}}}}\n\
         \x20 b: {{description: Open one., case_open: {{protocol: q, subject: doc}}}}\n\
         \x20 c: {{description: Open one., case_open: {{protocol: q, subject: doc}}}}\n"
    );
    let walked = simulate(&compiled(&source)).expect("walked");
    assert_eq!(walked.states, 54, "{walked}");
    assert_eq!(
        walked.to_string(),
        "cycle: state {action `a` opened 2 or more children, action `b` opened 2 or more \
         children, action `c` opened 2 or more children}: no outcome is legitimate, no action is \
         approval-required, and every step of action `a`, action `b`, action `c` returns this \
         state; path: action `a` produces nothing, then action `a` produces nothing, then action \
         `b` produces nothing, then action `b` produces nothing, then action `c` produces \
         nothing, then action `c` produces nothing; 1 other cycle state contains it\n\
         simulated: protocol `p` revision 1: 54 reachable states, 0 terminable, 0 waiting on \
         supplied evidence, a child outcome or expiry, 1 finding (2 stuck or cycle states)\n"
    );
    let without = simulate(&compiled(
        &source.replace(", case_open: {protocol: q, subject: doc}", ""),
    ))
    .expect("walked");
    assert_eq!(without.states, 1, "{without}");
}

/// R5 with the count of children (D1): a mark is in a state's containment and size, none below
/// one below two or more, one thing per child it counts. A mark left out of `records_within`
/// would let a state with a child not ended be contained in one without it; left out of `size`,
/// it would make a state with a child as small as the state without.
#[test]
fn a_mark_counts_in_containment_and_size_by_its_children() {
    let ir = compiled(&format!(
        "{HEAD}\
         evidence_kinds: {{case_outcome: {{description: The outcome of a child case.}}}}\n\
         actions: {{a: {{description: Open one., case_open: {{protocol: q, subject: doc}}}}}}\n"
    ));
    let space = Space::new(&ir);
    let dimensions = space.walked();
    let base = space.initial_states().remove(0);
    let layout = Layout::new(&dimensions, base.len()).marking(vec![0]);
    let mut none = base.clone();
    none.resize(layout.length(), 0);
    let one = layout.raise(&none, 0);
    let two = layout.raise(&one, 0);
    assert_eq!(
        [&none, &one, &two].map(|state| layout.mark(state, 0)),
        [0, 1, 2]
    );
    assert_eq!(layout.raise(&two, 0), two);
    assert_eq!(layout.marked(&two, 0, 1), one);
    assert_eq!(layout.marked(&one, 0, 0), none);
    assert!(layout.records_within(&none, &one) && layout.records_within(&one, &two));
    assert!(
        !layout.records_within(&one, &none),
        "a child is not within no child"
    );
    assert!(
        !layout.records_within(&two, &one),
        "two children are not within one"
    );
    assert_eq!(
        [&none, &one, &two].map(|state| layout.size(state) - layout.size(&none)),
        [0, 1, 2]
    );
}

/// D2: a child that ends lowers its action's mark by one child, so a second child's outcome still
/// arrives after the first child ended. The opening action is taken twice before `close`; one
/// child ends accepted, the other declined, and with both results neither outcome is legitimate:
/// a stuck state a real case with two children reaches, which a mark cleared by the first
/// child's end would never print. With one child, its one outcome makes an outcome legitimate.
#[test]
fn a_second_child_ends_after_the_first_one_ended() {
    let source = format!(
        "{HEAD}\
         evidence_kinds:\n\
         \x20 case_outcome: {{description: The outcome of a child case.}}\n\
         \x20 closure: {{description: No more children.}}\n\
         \x20 opened: {{description: A child was opened.}}\n\
         claims:\n\
         \x20 closed: {{description: Closed., true_when: {{evidence: {{kind: closure}}}}}}\n\
         \x20 started: {{description: Started., true_when: {{evidence: {{kind: opened}}}}}}\n\
         \x20 passed: {{description: A child passed., true_when: {{evidence: {{kind: \
         case_outcome, result: accepted}}}}}}\n\
         \x20 failed: {{description: A child failed., true_when: {{evidence: {{kind: \
         case_outcome, result: declined}}}}}}\n\
         actions:\n\
         \x20 open: {{description: Open one., precondition: {{claim: closed, is: unknown}}, \
         case_open: {{protocol: q, subject: doc}}, may_produce: [{{evidence: opened}}]}}\n\
         \x20 close: {{description: Close., precondition: {{all: [{{claim: started}}, {{claim: \
         closed, is: unknown}}]}}, may_produce: [{{evidence: closure}}]}}\n\
         outcomes:\n\
         \x20 passed: {{description: Closed and a child passed., requires: {{all: [{{claim: \
         closed}}, {{claim: passed}}, {{claim: failed, is: false}}]}}}}\n\
         \x20 failed: {{description: Closed and a child failed., requires: {{all: [{{claim: \
         closed}}, {{claim: failed}}, {{claim: passed, is: false}}]}}}}\n"
    );
    assert_eq!(
        report(&source),
        "stuck-state: state {evidence `case_outcome`, evidence `closure`, evidence `opened`}: no \
         outcome is legitimate and no action is admissible or approval-required; path: action \
         `open` produces evidence `opened`, then action `close` produces evidence `closure`, then \
         a child of action `open` records evidence `case_outcome`; 11 other stuck states contain \
         it\n\
         stuck-state: state {evidence `case_outcome` result `accepted`, evidence `case_outcome` \
         result `declined`, evidence `closure`, evidence `opened`}: no outcome is legitimate and \
         no action is admissible or approval-required; path: action `open` produces evidence \
         `opened`, then action `open` produces evidence `opened`, then action `close` produces \
         evidence `closure`, then a child of action `open` records evidence `case_outcome` result \
         `accepted`, then a child of action `open` records evidence `case_outcome` result \
         `declined`; 5 other stuck states contain it\n\
         simulated: protocol `p` revision 1: 47 reachable states, 6 terminable (outcome `failed` \
         3, outcome `passed` 3), 2 waiting on supplied evidence, a child outcome or expiry, 2 \
         findings (15 stuck or cycle states)\n"
    );
}

/// D5: a state that holds an `open` record, current or held as expired (R2a), holds a mark above
/// none, since an `open` record is a child that has not ended. In `HELD` a second child may
/// record `open`, the record expire, and that child end accepted: the successor that keeps the
/// expired `open` record beside the new outcome with every mark at none would stand for a third
/// child, and is not one. No printed state holds `open` without a mark, and, since the opening
/// step adds the child's `open` record (story:simulate-opens-child-with-its-record), none holds a
/// mark without `open`: 114 states, against 252 when the record arrived by a later step.
#[test]
fn an_open_record_keeps_a_mark_above_none() {
    let text = report(HELD);
    for line in text
        .lines()
        .filter(|line| line.starts_with("stuck-state: ") || line.starts_with("cycle: "))
    {
        let state = line.split("}: ").next().expect("a state");
        assert_eq!(
            state.contains("result `open`"),
            state.contains(" opened "),
            "{line}"
        );
    }
    assert_eq!(
        text.lines().last(),
        Some(
            "simulated: protocol `p` revision 1: 114 reachable states, 3 terminable (outcome \
             `done` 3), 19 waiting on supplied evidence, a child outcome or expiry, 3 findings \
             (78 stuck or cycle states)"
        ),
        "{text}"
    );
}

/// The walk is a pure function: the same IR gives the same report, and a protocol with its
/// declarations in another order compiles to the same IR and gives the same bytes.
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
    let renewal = "\x20 renewal: {description: A renewal.}\n";
    let reordered: String = EXPIRY
        .replace(renewal, "")
        .replace("evidence_kinds:\n", &format!("evidence_kinds:\n{renewal}"));
    assert_ne!(reordered, EXPIRY);
    assert_eq!(report(&reordered), report(EXPIRY));
}

/// story:simulate-revise-step, § Outcome 2, third change: a revision of `a` turns a record of a
/// dimension with upstream `a` into the one observed before `a` moved, which the invalidation
/// rule keeps from `passed` but not from `recorded`. The stuck state is reached only by the revise
/// step and is written with the variant; the state before it is a `cycle` under the restated rule
/// (every record step of `fix` returns it, its revise step does not), with the suffix.
#[test]
fn a_revision_of_an_upstream_artifact_keeps_the_record_from_its_claim() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1, description: A protocol.}\n\
        artifacts: {a: {description: A.}, b: {description: B.}}\n\
        evidence_kinds: {test: {description: A test.}}\n\
        claims:\n\
        \x20 passed: {description: Passed., true_when: {evidence: {kind: test, subject: b, \
        result: pass}}}\n\
        \x20 recorded: {description: Recorded., true_when: {evidence: {kind: test, subject: b}}}\n\
        actions:\n\
        \x20 check: {description: Check., precondition: {claim: recorded, is: unknown}, \
        may_produce: [{evidence: test}]}\n\
        \x20 fix: {description: Fix., precondition: {claim: passed, is: false}, revises: [a]}\n\
        outcomes:\n\
        \x20 passed: {description: Passed., requires: {claim: passed}}\n\
        invalidation:\n\
        \x20 a.revised: {description: A moved., upstream: a, invalidates: [passed]}\n";
    assert_eq!(
        report(source),
        "stuck-state: state {evidence `test` about `b` observed before upstream `a` moved}: no \
         outcome is legitimate and no action is admissible or approval-required; path: action \
         `check` produces evidence `test` about `b`, then action `fix` revises `a`\n\
         cycle: state {evidence `test` about `b`}: no outcome is legitimate, no action is \
         approval-required, and every record step of action `fix` returns this state; path: \
         action `check` produces evidence `test` about `b`; every revision by action `fix` leads \
         to a state no outcome is reachable from\n\
         simulated: protocol `p` revision 1: 4 reachable states, 1 terminable (outcome `passed` \
         1), 2 findings (2 stuck or cycle states)\n"
    );
}

/// story:simulate-revise-step, § Outcome 2, second change: a revision of `doc` turns a child's
/// record about `doc` into an earlier record, written `… earlier`. A `children` form with
/// `revision: current` does not read it, so the outcome it requires is lost after the revision:
/// the three states that hold the child's record, current with an unlisted outcome or earlier,
/// are `cycle`s, the one with a current record with the revision suffix. The state with `open`
/// earlier waits on the child's outcome; the one with `open` current does not wait, since its
/// revise step changes it. The child is about `doc`, the artifact the space's dimension is about,
/// and the form says `revision: current`, so a child that ends after the revision may end against
/// the earlier one (story:simulate-child-about-own-artifact): the two earlier states are first
/// reached that way, and the walk holds two states more, the open child after the revision, than
/// when only a child that ended before it counted (9 before, 11 after; the same three findings).
/// The opening step adds the child's `open` record (story:simulate-opens-child-with-its-record),
/// so each child's end replaces it, current or earlier, and the states of an opened child with no
/// record are gone: 8 states, one waiting, the same three findings.
#[test]
fn a_record_a_children_form_reads_becomes_earlier() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1, description: A protocol.}\n\
        artifacts: {b: {description: B.}, doc: {description: The document.}}\n\
        evidence_kinds:\n\
        \x20 case_outcome: {description: The outcome of a child case.}\n\
        \x20 start: {description: A start.}\n\
        claims:\n\
        \x20 reviewed: {description: Reviewed., true_when: {children: {in: [accepted], subject: \
        doc, revision: current}}}\n\
        \x20 started: {description: Started., true_when: {evidence: {kind: start, subject: b}}}\n\
        actions:\n\
        \x20 fix: {description: Fix., revises: [doc]}\n\
        \x20 open_review: {description: Open., precondition: {claim: started, is: unknown}, \
        may_produce: [{evidence: start}], case_open: {protocol: q, subject: doc}}\n\
        outcomes:\n\
        \x20 accepted: {description: Accepted., requires: {claim: reviewed}}\n";
    assert_eq!(
        report(source),
        "cycle: state {evidence `case_outcome` about `doc`, evidence `start` about `b`}: no \
         outcome is legitimate, no action is approval-required, and every record step of action \
         `fix` returns this state; path: action `open_review` produces evidence `start` about \
         `b`, then a child of action `open_review` records evidence `case_outcome` about `doc`, \
         replacing result `open`; every revision by action `fix` leads to a state no outcome is \
         reachable from\n\
         cycle: state {evidence `case_outcome` about `doc` earlier, evidence `start` about `b`}: \
         no outcome is legitimate, no action is approval-required, and every step of action \
         `fix` returns this state; path: action `open_review` produces evidence `start` about \
         `b`, then action `fix` revises `doc`, then a child of action `open_review` records \
         evidence `case_outcome` about `doc` earlier, replacing result `open` expired\n\
         cycle: state {evidence `case_outcome` about `doc` result `accepted` earlier, evidence \
         `start` about `b`}: no outcome is legitimate, no action is approval-required, and every \
         step of action `fix` returns this state; path: action `open_review` produces evidence \
         `start` about `b`, then action `fix` revises `doc`, then a child of action \
         `open_review` records evidence `case_outcome` about `doc` result `accepted` earlier, \
         replacing result `open` expired\n\
         simulated: protocol `p` revision 1: 8 reachable states, 1 terminable (outcome \
         `accepted` 1), 1 waiting on supplied evidence, a child outcome or expiry, 3 findings (3 \
         stuck or cycle states)\n"
    );
}

/// story:simulate-revise-step, § Scope: a revise step adds no record, one record or one group
/// the action offers (story:simulate-record-groups), and a grouped kind is not added alone. The
/// stuck state `{k, l}` is reached only by revising `doc`, which sets `m` aside, and adding the
/// group in the same step, written on one line.
#[test]
fn a_revise_step_adds_a_group_in_the_same_step() {
    let source = format!(
        "{HEAD}\
         evidence_kinds: {{k: {{description: K.}}, l: {{description: L.}}, m: {{description: \
         M.}}}}\n\
         claims:\n\
         \x20 hk: {{description: K., true_when: {{evidence: {{kind: k}}}}}}\n\
         \x20 hl: {{description: L., true_when: {{evidence: {{kind: l}}}}}}\n\
         \x20 hm: {{description: M., true_when: {{evidence: {{kind: m}}}}}}\n\
         actions:\n\
         \x20 x: {{description: X., precondition: {{claim: hm}}, revises: [doc], may_produce: \
         [{{evidence: k}}, {{evidence: l}}], records_together: [{{kinds: [k, l]}}]}}\n\
         \x20 y: {{description: Y., precondition: {{claim: hk, is: unknown}}, may_produce: \
         [{{evidence: m}}]}}\n\
         outcomes:\n\
         \x20 done: {{description: Done., requires: {{all: [{{claim: hk}}, {{claim: hl}}, \
         {{claim: hm}}]}}}}\n"
    );
    assert_eq!(
        report(&source),
        "stuck-state: state {evidence `k`, evidence `l`}: no outcome is legitimate and no action \
         is admissible or approval-required; path: action `y` produces evidence `m`, then action \
         `x` revises `doc` and produces evidence `k` and evidence `l`\n\
         simulated: protocol `p` revision 1: 4 reachable states, 1 terminable (outcome `done` \
         1), 1 finding (1 stuck or cycle state)\n"
    );
}

/// story:simulate-revise-step, § Outcome 1: the subsets of an action's `revises` are tried in
/// order of their sorted ids, `[a]`, `[a, b]`, `[b]`. From `{l, m}` revising `a` changes nothing,
/// and `[a, b]` and `[b]` both set `l` aside; the path names the first, `a, b`.
#[test]
fn revise_subsets_come_in_order_of_their_sorted_ids() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1, description: A protocol.}\n\
        artifacts: {a: {description: A.}, b: {description: B.}, c: {description: C.}}\n\
        evidence_kinds: {l: {description: L.}, m: {description: M.}}\n\
        claims:\n\
        \x20 hl: {description: L., true_when: {evidence: {kind: l, subject: b}}}\n\
        \x20 hm: {description: M., true_when: {evidence: {kind: m, subject: c}}}\n\
        actions:\n\
        \x20 x: {description: X., precondition: {claim: hl}, revises: [b, a]}\n\
        \x20 y: {description: Y., precondition: {claim: hm, is: unknown}, may_produce: \
        [{evidence: l}, {evidence: m}], records_together: [{kinds: [l, m]}]}\n\
        outcomes:\n\
        \x20 done: {description: Done., requires: {all: [{claim: hl}, {claim: hm}]}}\n";
    assert_eq!(
        report(source),
        "stuck-state: state {evidence `m` about `c`}: no outcome is legitimate and no action is \
         admissible or approval-required; path: action `y` produces evidence `l` about `b` and \
         evidence `m` about `c`, then action `x` revises `a`, `b`\n\
         simulated: protocol `p` revision 1: 3 reachable states, 1 terminable (outcome `done` \
         1), 1 finding (1 stuck or cycle state)\n"
    );
}

/// story:simulate-revise-step, § Outcome 1: an action's record steps are tried before its revise
/// steps, so a state both reach from one state is reached by the record step; a revise step that
/// changes nothing is no step, and a `cycle` whose revise steps all return the state is written
/// as before, without the suffix.
#[test]
fn a_record_step_is_tried_before_a_revise_step() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1, description: A protocol.}\n\
        artifacts: {b: {description: B.}, doc: {description: The document.}}\n\
        evidence_kinds: {k: {description: K.}, z: {description: Z.}}\n\
        claims:\n\
        \x20 hk: {description: K., true_when: {evidence: {kind: k, subject: b}}}\n\
        \x20 hz: {description: Z., true_when: {evidence: {kind: z}}}\n\
        actions:\n\
        \x20 x: {description: X., revises: [doc], may_produce: [{evidence: k}]}\n\
        outcomes:\n\
        \x20 done: {description: Done., requires: {claim: hz}}\n";
    assert_eq!(
        report(source),
        "cycle: state {evidence `k` about `b`}: no outcome is legitimate, no action is \
         approval-required, and every step of action `x` returns this state; path: action `x` \
         produces evidence `k` about `b`\n\
         unreached-outcome: outcome `done` is legitimate in no reachable state\n\
         no-outcome-reachable: protocol `p` revision 1 reaches no state in which an outcome is \
         legitimate\n\
         simulated: protocol `p` revision 1: 2 reachable states, 0 terminable (outcome `done` \
         0), 3 findings (1 stuck or cycle state)\n"
    );
}

/// story:simulate-revise-step, § Outcome 2: a state holds no revision number, so a walk with
/// revise steps reaches at most the states of `canon check`'s whole product, times three per
/// opening action for its mark. Each `revise*` fixture is within that bound.
#[test]
fn a_walk_with_revise_steps_stays_within_the_check_product() {
    let fixtures = [
        include_str!("../../../../fixtures/investigation/simulate/revise/protocol.yaml"),
        include_str!("../../../../fixtures/investigation/simulate/revise-upstream/protocol.yaml"),
        include_str!("../../../../fixtures/investigation/simulate/revise-denied/protocol.yaml"),
        include_str!("../../../../fixtures/investigation/simulate/revise-mixed/protocol.yaml"),
        include_str!("../../../../fixtures/investigation/simulate/revise-child/protocol.yaml"),
    ];
    for source in fixtures {
        let ir = compiled(source);
        let product = Space::new(&ir).size().expect("a product that fits");
        let openers = ir
            .actions
            .values()
            .filter(|action| action.case_open.is_some())
            .count();
        let walked = simulate(&ir).expect("walked");
        let states = u128::try_from(walked.states).expect("fits");
        assert!(
            states <= product * 3u128.pow(u32::try_from(openers).expect("few")),
            "{}: {states} states, product {product}",
            ir.protocol.id.as_str()
        );
    }
}

/// An external revision is a step of its own, written with the protocol that revises: it is tried
/// after the supplied steps and before the expiry steps. The state `{u}` is reached from `{t, u}`
/// both by `t` expiring and by `q` revising `a`, which `t` is about; the revise step comes first,
/// so the shortest path to it ends with it.
#[test]
fn an_external_revision_is_written_as_protocol_revises() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {revised_by: q}, b: {}}\n\
        evidence_kinds: {t: {max_age: 1h}, u: {}, n: {}}\n\
        claims:\n\
        \x20 c: {true_when: {evidence: {kind: t, subject: a}}}\n\
        \x20 g: {true_when: {any: [{evidence: {kind: t, subject: a}}, {evidence: {kind: u, subject: b}}]}}\n\
        \x20 never: {true_when: {evidence: {kind: n}}}\n\
        actions:\n\
        \x20 produce_t: {precondition: {claim: g, is: unknown}, may_produce: [{evidence: t}]}\n\
        \x20 produce_u: {precondition: {claim: c, is: true}, may_produce: [{evidence: u}]}\n\
        outcomes: {o: {requires: {claim: never}}}\n";
    let text = report(source);
    assert!(
        text.contains(
            "stuck-state: state {evidence `u` about `b`}: no outcome is legitimate and no action is \
             admissible or approval-required; path: action `produce_t` produces evidence `t` \
             about `a`, then action `produce_u` produces evidence `u` about `b`, then protocol `q` \
             revises `a`"
        ),
        "{text}"
    );
}

/// story:artifact-revised-by § Acceptance 7: an external revision is tried after the supplied
/// steps. The state `{u, s}` (`s` supplied) is reached at depth 4 from `{t, u, s}` by `q` revising
/// `a`, which `t` is about, and from `{u}` by `s` arriving; both parents are first reached from
/// `{t, u}` at depth 3, the supplied step first, so the shortest path to `{u, s}` ends with the
/// revision. `produce_x` is open only while `s` is unknown, so `{u}` is no stuck state that
/// would contain `{u, s}` and keep it from being printed.
#[test]
fn an_external_revision_is_tried_after_the_supplied_steps() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {revised_by: q}, b: {}}\n\
        evidence_kinds: {t: {}, u: {}, x: {}, s: {supplied_by: r}, n: {}}\n\
        claims:\n\
        \x20 c: {true_when: {evidence: {kind: t, subject: a}}}\n\
        \x20 g: {true_when: {any: [{evidence: {kind: t, subject: a}}, {evidence: {kind: u, subject: b}}]}}\n\
        \x20 sup: {true_when: {evidence: {kind: s, subject: b}}}\n\
        \x20 never: {true_when: {evidence: {kind: n}}}\n\
        actions:\n\
        \x20 produce_t: {precondition: {claim: g, is: unknown}, may_produce: [{evidence: t}]}\n\
        \x20 produce_u: {precondition: {claim: c, is: true}, may_produce: [{evidence: u}]}\n\
        \x20 produce_x: {precondition: {all: [{claim: g, is: true}, {claim: sup, is: unknown}]}, \
        may_produce: [{evidence: x}]}\n\
        outcomes: {o: {requires: {claim: never}}}\n";
    let text = report(source);
    let line = text
        .lines()
        .find(|line| {
            line.starts_with("stuck-state: state {evidence `s` about `b`, evidence `u` about `b`}")
        })
        .unwrap_or_else(|| panic!("no stuck state {{s, u}} printed:\n{text}"));
    assert!(
        line.contains(
            "path: action `produce_t` produces evidence `t` about `a`, then action `produce_u` \
             produces evidence `u` about `b`, then protocol `r` supplies evidence `s` about `b`, then \
             protocol `q` revises `a`;"
        ),
        "{line}"
    );
}

/// story:child-outcome-recovery, § Outcome 5: an action whose `case_open` subject is the artifact
/// the space's `case_outcome` dimension is about (here `a`, the least id) is walked as before: no
/// record is written about an artifact, and a revision of `b` leaves the failed outcome current.
#[test]
fn the_spaces_own_subject_is_not_split() {
    let source = include_str!(
        "../../../../fixtures/investigation/simulate/revise-child-current/protocol.yaml"
    )
    .replace("      subject: b\n", "      subject: a\n");
    let text = report(&source);
    assert!(text.contains("state {evidence `case_outcome`}"), "{text}");
    assert!(!text.contains(" about "), "{text}");
}

/// A protocol with a `descended` form (story:revision-ordering) is refused as `ordered-evidence`,
/// as one with any other order form is, naming the first declaration that has one.
#[test]
fn a_descended_form_is_refused_as_ordered_evidence() {
    let source = format!(
        "{HEAD}evidence_kinds: {{run: {{}}}}\nactions: {{go: {{may_produce: [{{evidence: run}}]}}}}\n\
         outcomes: {{done: {{requires: {{descended: {{first: {{kind: run, result: failed}}, \
         then: {{kind: run, result: passed}}, subject: doc}}}}}}}}\n"
    );
    let refusal = simulate(&compiled(&source)).expect_err("refused");
    assert_eq!(refusal.code(), "ordered-evidence");
    assert!(refusal.to_string().contains("outcome `done`"), "{refusal}");
}

/// Every fixture protocol that declares `case_open` and validates, by path: the 15 fixtures
/// story:simulate-opens-child-with-its-record names, the six others that open a child case, and
/// its two own.
const OPENING: [(&str, &str); 23] = [
    (
        "child-outcome/closed",
        include_str!("../../../../fixtures/investigation/child-outcome/closed/protocol.yaml"),
    ),
    (
        "child-outcome/denied",
        include_str!("../../../../fixtures/investigation/child-outcome/denied/protocol.yaml"),
    ),
    (
        "child-outcome/ends-open",
        include_str!("../../../../fixtures/investigation/child-outcome/ends-open/protocol.yaml"),
    ),
    (
        "child-outcome/repeat",
        include_str!("../../../../fixtures/investigation/child-outcome/repeat/protocol.yaml"),
    ),
    (
        "child-outcome/twice",
        include_str!("../../../../fixtures/investigation/child-outcome/twice/protocol.yaml"),
    ),
    (
        "child-outcome/two-records",
        include_str!("../../../../fixtures/investigation/child-outcome/two-records/protocol.yaml"),
    ),
    (
        "child-outcome-large/event",
        include_str!("../../../../fixtures/child-outcome-large/event/protocol.yaml"),
    ),
    (
        "child-outcome-large/festival",
        include_str!("../../../../fixtures/child-outcome-large/festival/protocol.yaml"),
    ),
    (
        "simulate/child-expiry",
        include_str!("../../../../fixtures/investigation/simulate/child-expiry/protocol.yaml"),
    ),
    (
        "simulate/child-expiry-held",
        include_str!("../../../../fixtures/investigation/simulate/child-expiry-held/protocol.yaml"),
    ),
    (
        "simulate/revise-child",
        include_str!("../../../../fixtures/investigation/simulate/revise-child/protocol.yaml"),
    ),
    (
        "simulate/revise-child-current",
        include_str!(
            "../../../../fixtures/investigation/simulate/revise-child-current/protocol.yaml"
        ),
    ),
    (
        "simulate/revise-child-late",
        include_str!("../../../../fixtures/investigation/simulate/revise-child-late/protocol.yaml"),
    ),
    (
        "simulate/revise-child-other-subject",
        include_str!(
            "../../../../fixtures/investigation/simulate/revise-child-other-subject/protocol.yaml"
        ),
    ),
    (
        "simulate/revise-child-two-subjects",
        include_str!(
            "../../../../fixtures/investigation/simulate/revise-child-two-subjects/protocol.yaml"
        ),
    ),
    (
        "child-outcome-adversary/either-once",
        include_str!("../../../../fixtures/child-outcome-adversary/either-once/protocol.yaml"),
    ),
    (
        "child-outcome-adversary/ended-once",
        include_str!("../../../../fixtures/child-outcome-adversary/ended-once/protocol.yaml"),
    ),
    (
        "case-composition",
        include_str!("../../../../fixtures/investigation/case-composition.yaml"),
    ),
    (
        "child-outcome-revision",
        include_str!("../../../../fixtures/investigation/child-outcome-revision.yaml"),
    ),
    (
        "child-outcome-subject",
        include_str!("../../../../fixtures/investigation/child-outcome-subject.yaml"),
    ),
    (
        "evidence-redelivery",
        include_str!("../../../../fixtures/investigation/evidence-redelivery.yaml"),
    ),
    (
        "simulate/children-open-record",
        include_str!(
            "../../../../fixtures/investigation/simulate/children-open-record/protocol.yaml"
        ),
    ),
    (
        "simulate/children-open-record-one-reader",
        include_str!(
            "../../../../fixtures/investigation/simulate/children-open-record-one-reader/protocol.yaml"
        ),
    ),
];

/// story:simulate-opens-child-with-its-record, acceptance 1: a runner writes a child's `open`
/// record in the step that opens it, so with four readers on one subject no state holds "three
/// ended complete, the fourth opened with no record", and the probe that needs the merge's `none`
/// while the readers are not all complete is legitimate in no reachable state. With one reader it
/// was unreached before and is after.
#[test]
fn a_child_opens_with_its_open_record() {
    let unreached = "unreached-outcome: outcome `probe` is legitimate in no reachable state";
    for (case, source) in &OPENING[21..] {
        let text = report(source);
        assert!(text.lines().any(|line| line == unreached), "{case}: {text}");
    }
    let ir = compiled(OPENING[21].1);
    let before = walk(&ir, false).expect("walked").report;
    assert!(
        !before.to_string().lines().any(|line| line == unreached),
        "the walk without the open record reaches the probe: {before}"
    );
}

/// Acceptance 2: every state the walk reaches is one the walk without the `open` record at the
/// opening reached, on every fixture that opens a child case: the opening step's successor is
/// that walk's opening step followed by the child's `open` record, and the only successor left
/// out is a child's end that drops the `open` record of a child that has not ended. Each of the
/// fixtures that read the class `open` reaches fewer.
#[test]
fn the_walk_reaches_only_states_the_walk_without_the_open_record_reached() {
    for (case, source) in OPENING {
        let ir = compiled(source);
        let with = walk(&ir, true).expect("walked");
        let without = walk(&ir, false).expect("walked");
        let old: std::collections::HashSet<&State> = without.states.iter().collect();
        let new = with
            .states
            .iter()
            .filter(|state| !old.contains(state))
            .count();
        assert_eq!(new, 0, "{case}: {new} states the old walk never reached");
        assert!(
            with.states.len() < without.states.len(),
            "{case}: {} states, {} without the open record",
            with.states.len(),
            without.states.len()
        );
    }
}

/// Acceptance 3: in no reachable state, printed or not, is an action's count of children above
/// none without an `open` record in the dimension its children record in, where some reader of
/// it demands no independence, or an `open` record held with every such count at none; the walk
/// without the `open` record at the opening has such states in each of these fixtures.
#[test]
fn every_count_above_none_holds_its_open_record_and_the_reverse() {
    for (case, source) in OPENING {
        let ir = compiled(source);
        let with = walk(&ir, true).expect("walked");
        assert_eq!(with.unpaired, Vec::<usize>::new(), "{case}");
        let without = walk(&ir, false).expect("walked");
        assert!(!without.unpaired.is_empty(), "{case}");
    }
}

/// A child read only through `independent_on: [other_team]`: no reader demands nothing, so a
/// runner's `open` record may meet no demand, which the state without it stands for.
const DEMANDING_READERS: &str = "format: protocol/1\n\
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

/// The review of wave 2026-11-23-w13, F1: where every reader of the dimension demands
/// independence, the opening step adds no `open` record and a child that ends may drop the last
/// one while a child is left, so the walk is the walk without the `open` record at the opening,
/// state for state; it reaches the running child whose record meets no demand (outcome `unseen`
/// legitimate), and a count above none without an `open` record is no unpaired state there.
#[test]
fn a_dimension_read_only_with_a_demand_walks_as_without_the_open_record() {
    let ir = compiled(DEMANDING_READERS);
    let with = walk(&ir, true).expect("walked");
    let without = walk(&ir, false).expect("walked");
    assert_eq!(with.states, without.states);
    assert_eq!(with.unpaired, Vec::<usize>::new());
    assert_eq!(
        with.report.to_string(),
        without.report.to_string(),
        "{}",
        with.report
    );
    assert!(
        with.report
            .outcomes
            .iter()
            .any(|(outcome, count)| outcome.as_str() == "unseen" && *count > 0),
        "{}",
        with.report
    );
}

/// The ending clause of story:refused-approval-is-not-stuck, option A.
const FORMS: &str = "to end the case after the denial, declare an outcome with `requires: \
    {decision: ..., when: ...}` or an action without capability `sign` whose record an outcome \
    reads";

/// Option A: one denial that blocks two actions whose preconditions are true names both after
/// the capability, once, with the two forms that end the case; the finding stays a finding.
#[test]
fn a_denial_names_each_action_it_blocks_and_the_forms_that_end_the_case() {
    let source = format!(
        "{HEAD}\
         evidence_kinds: {{post: {{description: A post.}}}}\n\
         claims:\n\
         \x20 posted: {{description: Posted., true_when: {{evidence: {{kind: post}}}}}}\n\
         actions:\n\
         \x20 publish: {{description: Publish., requires: [{{capability: sign}}], may_produce: \
         [{{evidence: post}}]}}\n\
         \x20 send: {{description: Send., requires: [{{capability: sign}}], may_produce: \
         [{{evidence: post}}]}}\n\
         outcomes:\n\
         \x20 posted: {{description: Posted., requires: {{claim: posted}}}}\n"
    );
    assert_eq!(
        report(&source),
        format!(
            "stuck-state: state {{capability `sign` denied}}: no outcome is legitimate and no \
             action is admissible or approval-required; path: capability `sign` denied; \
             capability `sign` denied blocks action `publish`, action `send`, whose preconditions \
             are true: {FORMS}\n\
             simulated: protocol `p` revision 1: 4 reachable states, 1 terminable (outcome \
             `posted` 1), 1 finding (1 stuck or cycle state)\n"
        )
    );
}

/// Option A's clause over several denials: the actions one set of denied capabilities blocks are
/// named together, in action order, and the forms name every denied capability.
#[test]
fn several_denials_are_named_by_the_actions_they_block() {
    let dead = Dead {
        state: "{capability `review` denied, capability `sign` denied}".to_owned(),
        path: vec![
            "capability `review` denied".to_owned(),
            "capability `sign` denied".to_owned(),
        ],
        blocks: vec![
            (
                ActionId::new("publish"),
                vec![CapabilityId::new("review"), CapabilityId::new("sign")],
            ),
            (ActionId::new("send"), vec![CapabilityId::new("sign")]),
        ],
        decisions: Vec::new(),
        containing: 0,
    };
    assert_eq!(
        Finding::StuckState(dead).to_string(),
        "stuck-state: state {capability `review` denied, capability `sign` denied}: no outcome is \
         legitimate and no action is admissible or approval-required; path: capability `review` \
         denied, then capability `sign` denied; capability `review` and capability `sign` denied \
         block action `publish`, capability `sign` denied blocks action `send`, whose \
         preconditions are true: to end the case after the denials, declare an outcome with \
         `requires: {decision: ..., when: ...}` or an action without capability `review` or \
         capability `sign` whose record an outcome reads"
    );
}

/// Option B: a state that is not open, in which a denial blocks an action and a decision step
/// leads to a legitimate outcome, is no finding, and the summary counts it; a decision that
/// counts only after an approval leaves the denial a finding, with option A's clause.
#[test]
fn a_decision_after_a_denial_ends_the_case() {
    let protocol = |requires: &str| {
        format!(
            "{HEAD}\
             evidence_kinds: {{approval: {{description: An approval.}}}}\n\
             claims:\n\
             \x20 approved: {{description: Approved., true_when: {{evidence: {{kind: \
             approval}}}}}}\n\
             actions:\n\
             \x20 approve: {{description: Approve., requires: [{{capability: sign}}], \
             may_produce: [{{evidence: approval}}]}}\n\
             outcomes:\n\
             \x20 approved: {{description: Approved., requires: {{claim: approved}}}}\n\
             \x20 closed: {{description: Closed., requires: {requires}}}\n"
        )
    };
    let walked = simulate(&compiled(&protocol("{decision: close}"))).expect("walked");
    assert_eq!((walked.ending, walked.dead), (1, 0));
    assert_eq!(
        walked.to_string(),
        "simulated: protocol `p` revision 1: 8 reachable states, 5 terminable (outcome `approved` \
         2, outcome `closed` 4), 1 ending by a decision after a denial, 0 findings (0 stuck or \
         cycle states)\n"
    );
    let walked = simulate(&compiled(&protocol(
        "{decision: close, when: {claim: approved}}",
    )))
    .expect("walked");
    assert_eq!((walked.ending, walked.dead), (0, 2));
    assert_eq!(
        walked.to_string(),
        format!(
            "stuck-state: state {{capability `sign` denied}}: no outcome is legitimate and no \
             action is admissible or approval-required; path: capability `sign` denied; \
             capability `sign` denied blocks action `approve`, whose precondition is true: \
             {FORMS}; 1 other stuck state contains it\n\
             simulated: protocol `p` revision 1: 8 reachable states, 2 terminable (outcome \
             `approved` 2, outcome `closed` 1), 1 finding (2 stuck or cycle states)\n"
        )
    );
}
