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
    assert_eq!(layout.held(&expired), [(at("lock"), 0, vec![0])]);
    assert_eq!(layout.expired(&expired), [(outcome, accepted)]);
    assert_eq!(
        space.render_walk(&base, &layout.held(&expired), &layout.expired(&expired)),
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

/// R7: a protocol with an action that opens a child case prints the limit on child outcomes
/// after the findings and before the summary; one without such an action does not.
#[test]
fn a_protocol_that_opens_child_cases_states_the_limit() {
    let text = report(HELD);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[lines.len() - 2], CHILD_OUTCOME_LIMIT);
    assert_eq!(
        text.matches(CHILD_OUTCOME_LIMIT).count(),
        1,
        "one limit line:\n{text}"
    );
    let without = report(&HELD.replace(", case_open: {protocol: q, subject: findings}", ""));
    assert!(!without.contains("limit:"), "{without}");
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
