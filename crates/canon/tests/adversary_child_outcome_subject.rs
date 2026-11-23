//! Adversary pass 1 on story:child-outcome-subject: what `canon check` and `canon simulate` make of
//! a `children` form's `subject`. The unit's own reference states the runner duty these cases rest
//! on (`ess/domains/protocol.yaml`, `canon.protocol.ChildOutcomes`): a `case_outcome` record's
//! `subject` is the `case_open` subject of the action that opened the child. So an action that opens
//! children about one artifact never produces a record a form naming another artifact reads.

use std::path::Path;

use b10x_canon::check::check;
use b10x_canon::ir::{self, Ir};
use b10x_canon::model::{self, OutcomeId};
use b10x_canon::simulate::simulate;

fn compiled(source: &str) -> Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

/// `alpha.open` opens children about `alpha` and needs no authority; `beta.open` opens children
/// about `beta` and requires `dispatch`. `beta.done` reads only the children about `beta`.
const GOVERNED_BETA: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {alpha: {}, beta: {}}\nevidence_kinds: {case_outcome: {}}\n\
    actions:\n  alpha.open:\n    case_open: {protocol: q, subject: alpha}\n  \
    beta.open:\n    requires: [{capability: dispatch}]\n    case_open: {protocol: q, subject: beta}\n\
    outcomes:\n  beta.done:\n    requires:\n      children: {in: [passed], subject: beta}\n";

/// Only `alpha.open` opens children, about `alpha`; `beta.done` reads only the children about
/// `beta`, which no action opens.
const UNOPENED_BETA: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {alpha: {}, beta: {}}\nevidence_kinds: {case_outcome: {}}\n\
    actions:\n  alpha.open:\n    case_open: {protocol: q, subject: alpha}\n\
    outcomes:\n  beta.done:\n    requires:\n      children: {in: [passed], subject: beta}\n";

/// Three outcomes, each reading only the children about its own artifact at its current revision.
const THREE_SUBJECTS: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {alpha: {}, beta: {}, gamma: {}}\nevidence_kinds: {case_outcome: {}}\n\
    actions:\n  alpha.open: {case_open: {protocol: q, subject: alpha}}\n  \
    beta.open: {case_open: {protocol: q, subject: beta}}\n  \
    gamma.open: {case_open: {protocol: q, subject: gamma}}\n\
    outcomes:\n  \
    alpha.done: {requires: {children: {in: [passed], subject: alpha, revision: current}}}\n  \
    beta.done: {requires: {children: {in: [passed], subject: beta, revision: current}}}\n  \
    gamma.done: {requires: {children: {in: [passed], subject: gamma, revision: current}}}\n";

/// `authority-bypass` claims `beta.done` is legitimate without authority. Its witness holds a
/// record about `beta`, and only `beta.open`, which requires `dispatch`, opens a child about
/// `beta`; `alpha.open` produces records about `alpha` only. The finding is false: `check` takes
/// every `case_open` action as producing `case_outcome` about any artifact (`check::produces`).
#[test]
fn check_finds_no_bypass_through_a_child_opened_about_another_artifact() {
    let report = check(&compiled(GOVERNED_BETA), None).expect("checks");
    assert_eq!(
        report.to_string(),
        "checked: protocol `p` revision 1: 216 states, 0 properties, 0 findings\n"
    );
}

/// `alpha.done` reads only the `case_outcome` dimension about `alpha` and its history (72 states).
/// An evidence match or an order form naming a subject is sliced to that subject
/// (story:check-subject-slices); a `children` form naming one is taken as reading every
/// `case_outcome` dimension (`evidence_reached`), so the slice holds all three subjects, 373248
/// states, and `check` refuses a protocol whose every slice holds 72.
#[test]
fn check_slices_a_children_form_to_its_subject() {
    let report = check(&compiled(THREE_SUBJECTS), None)
        .unwrap_or_else(|refusal| panic!("{}: {refusal}", refusal.code()));
    assert_eq!(
        report.to_string(),
        "checked: protocol `p` revision 1: 373248 states, 0 properties, 0 findings\n"
    );
}

/// No action opens a child about `beta`, so no reachable state holds a record `beta.done` reads,
/// and `beta.done` is legitimate in no reachable state. `simulate` walks `alpha.open` producing
/// `case_outcome` about `beta` and reports `beta.done` terminable.
#[test]
fn simulate_does_not_reach_an_outcome_through_a_child_about_another_artifact() {
    let report = simulate(&compiled(UNOPENED_BETA)).expect("simulates");
    let text = report.to_string();
    assert!(
        !text.contains("action `alpha.open` produces evidence `case_outcome` about `beta`"),
        "{text}"
    );
    assert_eq!(
        report.outcomes,
        vec![(OutcomeId::new("beta.done"), 0)],
        "{text}"
    );
}

/// Two forms naming two subjects in one requirement: `both.done` reads the dimension about each.
const TWO_IN_ONE: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {alpha: {}, beta: {}}\nevidence_kinds: {case_outcome: {}}\n\
    actions:\n  alpha.open: {case_open: {protocol: q, subject: alpha}}\n  \
    beta.open: {case_open: {protocol: q, subject: beta}}\n\
    outcomes:\n  both.done:\n    requires:\n      all:\n        \
    - {children: {in: [passed], subject: alpha}}\n        \
    - {children: {in: [passed], subject: beta}}\n";

/// Gap behind mutant m3 (`reads_in` keeping a form's subject only with `revision: current`):
/// every `canon check` case of the unit gives its forms `revision: current`. A form naming `beta`
/// without it reads the dimension about `beta`: one dimension of three classes and its history,
/// 72 states, `beta.done` reachable. `unproduced-evidence` is decided per kind, so it is silent.
#[test]
fn check_reads_the_subject_of_a_form_without_revision() {
    let report = check(&compiled(UNOPENED_BETA), None).expect("checks");
    assert_eq!(
        report.to_string(),
        "checked: protocol `p` revision 1: 72 states, 0 properties, 0 findings\n"
    );
}

/// Gap behind mutant m2 (`reads_in` giving every `children` form of one predicate the subject of
/// its first): two forms naming two subjects in one requirement read two dimensions, 72 states
/// each, and `both.done` is reachable.
#[test]
fn check_reads_each_forms_own_subject_in_one_predicate() {
    let report = check(&compiled(TWO_IN_ONE), None).expect("checks");
    assert_eq!(
        report.to_string(),
        "checked: protocol `p` revision 1: 5184 states, 0 properties, 0 findings\n"
    );
}

/// Gap behind mutant m4 (`normalize` dropping the subject of a form under `not`): no case of the
/// unit compiles a form with `subject` under `not`.
#[test]
fn compile_keeps_the_subject_of_a_form_under_not() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {alpha: {}, beta: {}}\nevidence_kinds: {case_outcome: {}}\n\
        claims: {c: {true_when: {not: {children: {in: [passed], subject: beta}}}}}\n";
    let value: serde_json::Value =
        serde_json::from_str(&compiled(source).canonical_json()).expect("JSON");
    assert_eq!(
        value["claims"]["c"]["true_when"],
        serde_json::json!({"not": {"children": {"in": ["passed"], "subject": "beta"}}})
    );
}

/// The unit's own fixture: `ideas.open` opens children about `intake` only, yet the walk's path
/// has it produce `case_outcome` about `wave`, a record no idea child carries.
#[test]
fn simulate_over_the_fixture_opens_ideas_only_about_the_intake() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/investigation/child-outcome-subject.yaml");
    let source = std::fs::read_to_string(&path).expect("the fixture reads");
    let text = simulate(&compiled(&source)).expect("simulates").to_string();
    assert!(
        !text.contains("action `ideas.open` produces evidence `case_outcome` about `wave`"),
        "{text}"
    );
}

// ---------------------------------------------------------------------------------------------
// Pass 2, after `Space::produced_by` (step 1) and the walk bound to it (step 2). Each case below
// agrees with an enumeration of real cases judged by `canon evaluate` (one child per opening, its
// `open` record written at once, every artifact at one revision, two children per opener).

/// The last line of `canon simulate` over `source`.
fn summary(source: &str) -> String {
    let text = simulate(&compiled(source))
        .unwrap_or_else(|refusal| panic!("{}: {refusal}", refusal.code()))
        .to_string();
    text.lines().last().expect("a summary").to_owned()
}

const WITH_AND_WITHOUT: &str = "claims:\n  a.ok: {true_when: {children: {in: [passed], subject: alpha}}}\n  \
    all.ok: {true_when: {children: {in: [passed]}}}\n";

/// `gamma` is no form's subject, and the dimension without a subject stands for `beta`, the
/// first artifact nothing names. A child of `gamma.open` is still read by `all.ok`: `no.alpha` is
/// legitimate once one passed and no child about `alpha` exists (real cases: 2 states). Behind
/// `produced_by` binding the dimension without a subject to its stand-in artifact only.
/// Re-pinned by story:simulate-opens-child-with-its-record (each child opens with its `open`
/// record, as in the enumeration of real cases): 144 states, `no.alpha` still legitimate.
#[test]
fn simulate_records_a_child_about_an_unnamed_artifact_without_subject() {
    let source = format!(
        "format: protocol/1\nprotocol: {{id: gamma, revision: 1}}\n\
         artifacts: {{alpha: {{}}, beta: {{}}, gamma: {{}}}}\nevidence_kinds: {{case_outcome: {{}}}}\n\
         {WITH_AND_WITHOUT}actions:\n  alpha.open: {{case_open: {{protocol: q, subject: alpha}}}}\n  \
         gamma.open: {{case_open: {{protocol: q, subject: gamma}}}}\n\
         outcomes:\n  no.alpha: {{requires: {{all: [{{claim: all.ok}}, {{claim: a.ok, is: unknown}}]}}}}\n"
    );
    assert_eq!(
        summary(&source),
        "simulated: protocol `gamma` revision 1: 144 reachable states, 1 terminable (outcome \
         `no.alpha` 1), 0 waiting on supplied evidence, a child outcome or expiry, 1 finding (16 \
         stuck or cycle states)"
    );
}

/// Only `alpha.open` opens children, and a form names `alpha`: its children record about
/// `alpha` alone, so `all.ok` true beside `a.ok` unknown is no case (real cases: 0 states).
/// Behind `produced_by` giving an opener the dimension without a subject as well.
/// Re-pinned by story:simulate-opens-child-with-its-record: 12 states, `none.alpha` still in none.
#[test]
fn simulate_keeps_a_named_child_out_of_the_dimension_without_subject() {
    let source = format!(
        "format: protocol/1\nprotocol: {{id: alpha-only, revision: 1}}\n\
         artifacts: {{alpha: {{}}, beta: {{}}}}\nevidence_kinds: {{case_outcome: {{}}}}\n\
         {WITH_AND_WITHOUT}actions:\n  alpha.open: {{case_open: {{protocol: q, subject: alpha}}}}\n\
         outcomes:\n  none.alpha: {{requires: {{all: [{{claim: all.ok}}, {{claim: a.ok, is: unknown}}]}}}}\n  \
         with.alpha: {{requires: {{claim: a.ok}}}}\n"
    );
    assert_eq!(
        summary(&source),
        "simulated: protocol `alpha-only` revision 1: 12 reachable states, 1 terminable (outcome \
         `none.alpha` 0, outcome `with.alpha` 1), 2 waiting on supplied evidence, a child outcome \
         or expiry, 2 findings (2 stuck or cycle states)"
    );
}

/// `authority-bypass` decided per dimension. A free opener about `alpha` and a gated one about
/// `beta` with an outcome requiring both forms: no case reaches `done` without `dispatch`. A
/// gated opener about `beta` with a free one about `alpha` read only by a form without subject:
/// `done` rests on `beta` alone, no bypass. Two openers about `alpha`, one free: a bypass.
#[test]
fn check_decides_a_bypass_per_dimension() {
    let head = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {alpha: {}, beta: {}}\nevidence_kinds: {case_outcome: {}}\n";
    let gated = "  beta.open: {requires: [{capability: dispatch}], case_open: {protocol: q, subject: beta}}\n";
    let both = format!(
        "{head}actions:\n  alpha.open: {{case_open: {{protocol: q, subject: alpha}}}}\n{gated}\
         outcomes:\n  done: {{requires: {{all: [{{children: {{in: [passed], subject: alpha}}}}, \
         {{children: {{in: [passed], subject: beta}}}}]}}}}\n"
    );
    assert_eq!(
        check(&compiled(&both), None).expect("checks").to_string(),
        "checked: protocol `p` revision 1: 15552 states, 0 properties, 0 findings\n"
    );
    let unbound_free = format!(
        "{head}claims:\n  all.ok: {{true_when: {{children: {{in: [passed]}}}}}}\n\
         actions:\n  alpha.open: {{case_open: {{protocol: q, subject: alpha}}}}\n{gated}\
         \x20 close: {{precondition: {{claim: all.ok}}}}\n\
         outcomes:\n  done: {{requires: {{children: {{in: [passed], subject: beta}}}}}}\n"
    );
    assert_eq!(
        check(&compiled(&unbound_free), None)
            .expect("checks")
            .to_string(),
        "checked: protocol `p` revision 1: 15552 states, 0 properties, 0 findings\n"
    );
    let same = format!(
        "{head}actions:\n  free.open: {{case_open: {{protocol: q, subject: alpha}}}}\n  \
         gated.open: {{requires: [{{capability: dispatch}}], case_open: {{protocol: q, subject: alpha}}}}\n\
         outcomes:\n  done: {{requires: {{children: {{in: [passed], subject: alpha}}}}}}\n"
    );
    assert_eq!(
        check(&compiled(&same), None).expect("checks").to_string(),
        "authority-bypass: outcome `done` is legitimate without authority in state {evidence \
         `case_outcome` about `alpha` result `passed` earlier}\n\
         checked: protocol `p` revision 1: 216 states, 0 properties, 1 finding\n"
    );
}
