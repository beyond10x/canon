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
