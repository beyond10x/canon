//! story:child-outcome-subject against the library: what `canon validate` refuses of a `children`
//! form's `subject`, the canon-ir/1 form and its read-back, the reasons and the explanation, the
//! semantic diff, `canon check` and `canon simulate`. CANON-COMPOSE-003
//! (conformance/scenarios/child-outcome-subject.yaml) states the values; this file states the
//! rest. Every case gives the two artifacts two different revisions (`intake` at `n1`, `wave` at
//! `w2`) and holds children about both.

use std::path::Path;

use b10x_canon::check::check;
use b10x_canon::diff::{Category, diff};
use b10x_canon::eval;
use b10x_canon::ir::{self, Ir};
use b10x_canon::model::{self, ClaimId, EvidenceRecord, Truth};
use b10x_canon::simulate::simulate;
use b10x_canon::validate::validate;
use serde_json::{Value, json};

fn fixture() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/investigation/child-outcome-subject.yaml");
    std::fs::read_to_string(&path).expect("the child-outcome-subject fixture reads")
}

fn compiled(source: &str) -> Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

/// Every problem `canon validate` reports of `source`, as `code: message`.
fn problems(source: &str) -> Vec<String> {
    match validate(&model::parse(source).expect("parses")) {
        Ok(()) => Vec::new(),
        Err(problems) => problems
            .iter()
            .map(|problem| format!("{}: {problem}", problem.code()))
            .collect(),
    }
}

fn outcome(id: &str, result: &str, subject: &str, revision: &str) -> EvidenceRecord {
    eval::read_evidence(&format!(
        "format: canon-evidence/1\nid: {id}\nkind: case_outcome\nresult: {result}\n\
         subject: {subject}\nsubject_revision: {revision}\nchild_case: CHILD-{id}\n"
    ))
    .expect("a record")
}

/// `intake` at `n1`, `wave` at `w2`.
fn case() -> model::Case {
    eval::read_case(
        "format: canon-case/1\nid: ORCH-1\nprotocol: orchestrate\n\
         artifacts: {intake: {revision: n1}, wave: {revision: w2}}\n",
    )
    .expect("case reads")
}

/// Expectation 7: a `subject` the protocol does not declare is `undeclared-artifact`, naming the
/// declaration the form is written in, wherever the form is written; it comes after the kind
/// `case_outcome` when that is undeclared too, as an evidence match's subject comes after its
/// kind. No other code is added.
#[test]
fn validate_refuses_an_undeclared_subject_naming_the_declaration() {
    let nope = fixture().replacen("        subject: wave\n", "        subject: nope\n", 1);
    assert_ne!(nope, fixture(), "the claim's subject was replaced");
    assert_eq!(
        problems(&nope),
        [
            "undeclared-artifact: claim `wave.passed` references artifact `nope`, which is not declared"
        ]
    );
    let in_action = fixture().replacen(
        "{in: [passed], subject: wave, revision: current}",
        "{in: [passed], subject: nope, revision: current}",
        1,
    );
    assert_ne!(in_action, fixture(), "the action's subject was replaced");
    assert_eq!(
        problems(&in_action),
        [
            "undeclared-artifact: action `wave.close` references artifact `nope`, which is not declared"
        ]
    );
    let in_outcome = fixture().replacen(
        "    requires:\n      claim: wave.passed\n",
        "    requires:\n      children: {in: [passed], subject: nope}\n",
        1,
    );
    assert_ne!(
        in_outcome,
        fixture(),
        "the outcome's requirement was replaced"
    );
    assert_eq!(
        problems(&in_outcome),
        [
            "undeclared-artifact: outcome `wave.done` references artifact `nope`, which is not declared"
        ]
    );
    let undeclared_kind = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {wave: {}}\n\
        claims: {c: {true_when: {children: {in: [passed], subject: nope}}}}\n";
    assert_eq!(
        problems(undeclared_kind),
        [
            "undeclared-evidence-kind: claim `c` references evidence kind `case_outcome`, which is not declared",
            "undeclared-artifact: claim `c` references artifact `nope`, which is not declared",
        ]
    );
    assert_eq!(problems(&fixture()), Vec::<String>::new());
}

/// Expectation 1: `"subject"` is written only when the form names one, and reads back; the
/// read-back refuses every other spelling of it: `null`, a number, an artifact the protocol does
/// not declare, an empty one, and the key out of its canonical place.
#[test]
fn the_ir_writes_subject_only_when_named_and_reads_it_back() {
    let ir = compiled(&fixture());
    let text = ir.canonical_json();
    let value: Value = serde_json::from_str(&text).expect("JSON");
    assert_eq!(
        value["claims"]["wave.passed"]["true_when"],
        json!({"children": {"in": ["passed"], "revision": "current", "subject": "wave"}})
    );
    assert_eq!(
        value["claims"]["all.passed"]["true_when"],
        json!({"children": {"in": ["passed"], "revision": "current"}})
    );
    assert_eq!(eval::read_ir(&text).expect("reads"), ir);

    let written = "\"revision\": \"current\",\n          \"subject\": \"wave\"";
    assert!(text.contains(written), "{text}");
    for (other, why) in [
        (
            "\"revision\": \"current\",\n          \"subject\": null",
            "a null subject",
        ),
        (
            "\"revision\": \"current\",\n          \"subject\": 7",
            "a number",
        ),
        (
            "\"revision\": \"current\",\n          \"subject\": \"nope\"",
            "an undeclared artifact",
        ),
        (
            "\"revision\": \"current\",\n          \"subject\": \"\"",
            "an empty artifact",
        ),
        (
            "\"subject\": \"wave\",\n          \"revision\": \"current\"",
            "the key before `revision`",
        ),
    ] {
        let changed = text.replacen(written, other, 1);
        assert_ne!(changed, text, "{why}: the text was changed");
        assert_eq!(
            eval::read_ir(&changed).expect_err(why).code(),
            "malformed-input",
            "{why}"
        );
    }
}

/// Two forms that differ only in `subject` are two members of an `all` or an `any`, no subject
/// first and then in subject order; one written twice is one member.
#[test]
fn forms_that_differ_only_in_subject_are_two_members_no_subject_first() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {intake: {}, wave: {}}\nevidence_kinds: {case_outcome: {}}\n\
        claims: {c: {true_when: {all: [{children: {in: [passed], subject: wave}}, \
        {children: {in: [passed], subject: intake}}, {children: {in: [passed]}}, \
        {children: {in: [passed], subject: wave}}]}}}\n";
    let value: Value = serde_json::from_str(&compiled(source).canonical_json()).expect("JSON");
    assert_eq!(
        value["claims"]["c"]["true_when"],
        json!({"all": [
            {"children": {"in": ["passed"]}},
            {"children": {"in": ["passed"], "subject": "intake"}},
            {"children": {"in": ["passed"], "subject": "wave"}},
        ]})
    );
}

/// A form with `subject` and no `revision` reads every record about its artifact, at whatever
/// revision, and none about another; with `revision: current` too, only those at the current
/// revision of its artifact. No record is listed in `excluded_evidence`.
#[test]
fn a_subject_selects_the_records_about_its_artifact() {
    let source = format!(
        "{}  wave.ever:\n    true_when:\n      children: {{in: [passed], subject: wave}}\n",
        fixture()
            .replacen("\nactions:\n", "\n", 1)
            .split("\n  ideas.open:")
            .next()
            .expect("head")
    );
    let ir = compiled(&source);
    for (records, ever, now, why) in [
        (
            vec![outcome("c1", "dropped", "wave", "w1")],
            Truth::False,
            Truth::Unknown,
            "an earlier wave story dropped",
        ),
        (
            vec![
                outcome("c1", "passed", "wave", "w1"),
                outcome("i1", "dropped", "intake", "n1"),
            ],
            Truth::True,
            Truth::Unknown,
            "an idea dropped at the current intake",
        ),
        (
            vec![outcome("i1", "passed", "intake", "w2")],
            Truth::Unknown,
            Truth::Unknown,
            "an idea about the intake at the wave's current revision",
        ),
        (
            vec![outcome("c3", "passed", "wave", "w2")],
            Truth::True,
            Truth::True,
            "a wave story at the current revision",
        ),
    ] {
        let decision = eval::evaluate(&ir, &case(), &records).expect("decides");
        let claim = |id: &str| {
            decision
                .claims
                .get(&ClaimId::new(id))
                .expect("the claim")
                .clone()
        };
        assert_eq!(claim("wave.ever").value, ever, "wave.ever: {why}");
        assert_eq!(claim("wave.passed").value, now, "wave.passed: {why}");
        assert!(claim("wave.ever").excluded_evidence.is_empty(), "{why}");
        assert!(claim("wave.passed").excluded_evidence.is_empty(), "{why}");
    }
}

/// Reasons and the explanation name the form with `"subject"`; three forms that differ only in
/// `subject` are three reasons, no subject first and then in subject order, each with its own
/// value. The same inputs give the same bytes.
#[test]
fn reasons_and_the_explanation_name_the_form_with_its_subject() {
    let source = fixture().replacen(
        "        - children: {in: [passed], subject: wave, revision: current}\n",
        "        - children: {in: [passed], subject: wave, revision: current}\n\
         \x20       - children: {in: [passed], subject: intake, revision: current}\n",
        1,
    );
    assert_ne!(source, fixture(), "the action was extended");
    let ir = compiled(&source);
    let records = [
        outcome("c4", "dropped", "wave", "w2"),
        outcome("i1", "open", "intake", "n1"),
    ];
    let decision = eval::evaluate(&ir, &case(), &records).expect("decides");
    let actions = decision.actions.as_ref().expect("actions");
    assert_eq!(
        actions["wave.close"]["reasons"],
        json!([
            {"children": ["passed"], "revision": "current", "value": "false"},
            {"children": ["passed"], "revision": "current", "subject": "intake", "value": "unknown"},
            {"children": ["passed"], "revision": "current", "subject": "wave", "value": "false"},
        ])
    );
    let explanation = decision.explanation.as_ref().expect("an explanation");
    assert_eq!(
        explanation["claims"]["wave.passed"],
        json!({
            "because": [{"children": ["passed"], "revision": "current", "subject": "wave", "value": "false"}],
            "value": "false",
        })
    );
    assert_eq!(
        explanation["claims"]["all.passed"],
        json!({
            "because": [{"children": ["passed"], "revision": "current", "value": "false"}],
            "value": "false",
        })
    );
    assert_eq!(
        explanation["actions"]["wave.close"]["because"],
        actions["wave.close"]["reasons"]
    );
    let again = eval::evaluate(&ir, &case(), &records).expect("decides");
    assert_eq!(eval::render(&again), eval::render(&decision));
}

/// Adding, removing or changing `subject` on a form is `BREAKING`, as any change to a form is.
#[test]
fn the_diff_takes_a_changed_subject_as_breaking() {
    let from = compiled(&fixture());
    let later = |text: String| compiled(&text.replacen("revision: 1", "revision: 2", 1));
    let moved =
        later(fixture().replacen("        subject: wave\n", "        subject: intake\n", 1));
    let changes = diff(&from, &moved).expect("diffs").changes;
    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(changes[0].category, Category::Breaking);
    assert!(
        changes[0]
            .to_string()
            .starts_with("BREAKING: claim `wave.passed` "),
        "{}",
        changes[0]
    );
    let unbound = later(fixture().replacen("        subject: wave\n", "", 1));
    let changes = diff(&from, &unbound).expect("diffs").changes;
    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(changes[0].category, Category::Breaking);
}

/// `canon check` over the fixture: a form with `subject` reads the `case_outcome` dimension about
/// its artifact, beside the one about another artifact that the form without `subject` reads, so
/// `wave.done` is reachable and nothing is reported. Two dimensions of three classes (`passed`,
/// `open`, another), each 8 values and a history dimension of 9.
#[test]
fn check_reads_the_dimension_about_the_forms_subject() {
    let report = check(&compiled(&fixture()), None).expect("checks");
    assert_eq!(
        report.to_string(),
        "checked: protocol `orchestrate` revision 1: 5184 states, 0 properties, 0 findings\n"
    );
}

/// With only forms that name `wave`, the space holds the one `case_outcome` dimension about
/// `wave` and none about `intake`, the first artifact, which no form reads.
#[test]
fn check_reads_only_the_subject_when_every_form_names_it() {
    let bound = fixture()
        .replacen("        - children: {in: [passed], revision: current}\n", "", 1)
        .replacen(
            "  all.passed:\n    description: Every child case about the current revision of its artifact ended passed, whatever the artifact.\n    true_when:\n      children:\n        in: [passed]\n        revision: current\n",
            "",
            1,
        );
    assert!(!bound.contains("all.passed"), "{bound}");
    let report = check(&compiled(&bound), None).expect("checks");
    assert_eq!(
        report.to_string(),
        "checked: protocol `orchestrate` revision 1: 72 states, 0 properties, 0 findings\n"
    );
}

// ---------------------------------------------------------------------------------------------
// Correction 1, after adversary pass 1: D1, a child's record is about the artifact its opening
// action's `case_open` names, so `canon check` binds each opening action to the `case_outcome`
// dimension about that artifact, or to the one without a subject; D3; and the cases behind the
// adversary's mutants m2, m3 and m4.

/// A protocol of two artifacts, `intake` and `wave`: `ideas.open` opens children about
/// `ideas_about` and needs no authority, `stories.open` opens children about `wave` and requires
/// `dispatch`, and `wave.done` requires `requirement`.
fn openers(ideas_about: &str, requirement: &str) -> String {
    format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\n\
         artifacts: {{intake: {{}}, wave: {{}}}}\nevidence_kinds: {{case_outcome: {{}}}}\n\
         actions:\n\
         \x20\x20ideas.open: {{case_open: {{protocol: idea, subject: {ideas_about}}}}}\n\
         \x20\x20stories.open: {{requires: [{{capability: dispatch}}], \
         case_open: {{protocol: story, subject: wave}}}}\n\
         outcomes:\n\x20\x20wave.done: {{requires: {requirement}}}\n"
    )
}

fn checked(source: &str) -> String {
    check(&compiled(source), None)
        .unwrap_or_else(|refusal| panic!("{}: {refusal}", refusal.code()))
        .to_string()
}

/// D1: only `stories.open`, which requires `dispatch`, opens children about `wave`, so no state
/// that takes no authority decision holds a record `wave.done` reads that an action could have
/// produced: no `authority-bypass`. When `ideas.open` opens children about `wave` too, the record
/// is produced without authority and the bypass is reported; and with a form without `subject`,
/// which reads the one dimension every opening action produces, as before the key existed.
#[test]
fn check_binds_a_childs_record_to_the_artifact_it_was_opened_about() {
    let wave_form = "{children: {in: [passed], subject: wave}}";
    assert_eq!(
        checked(&openers("intake", wave_form)),
        "checked: protocol `p` revision 1: 216 states, 0 properties, 0 findings\n"
    );
    for (source, why) in [
        (openers("wave", wave_form), "a free opening about wave"),
        (
            openers("intake", "{children: {in: [passed]}}"),
            "a form without subject",
        ),
    ] {
        let report = checked(&source);
        assert!(
            report.starts_with(
                "authority-bypass: outcome `wave.done` is legitimate without authority in state {"
            ) && report.ends_with("216 states, 0 properties, 1 finding\n"),
            "{why}: {report}"
        );
    }
}

/// D3: no action opens a child about `wave`, yet `canon check` takes `wave.done` as reachable: a
/// state holds any combination of records, and production decides only which states witness a
/// bypass. `unproduced-evidence` is decided per kind, and `ideas.open` produces `case_outcome`, so
/// nothing is reported. 72 states: the dimension about `wave` and its history.
#[test]
fn check_reports_nothing_for_a_subject_no_child_is_opened_about() {
    let source = openers("intake", "{children: {in: [passed], subject: wave}}").replacen(
        "  stories.open: {requires: [{capability: dispatch}], case_open: {protocol: story, subject: wave}}\n",
        "",
        1,
    );
    assert!(!source.contains("stories.open"), "{source}");
    assert_eq!(
        checked(&source),
        "checked: protocol `p` revision 1: 72 states, 0 properties, 0 findings\n"
    );
}

/// Behind m2 (`reads_in` giving every form of a predicate the first form's subject): two forms of
/// two subjects in one requirement read two dimensions, 72 values each with its history, and the
/// outcome is reachable; one form also says `revision: current`.
#[test]
fn check_reads_each_forms_own_subject_in_one_predicate() {
    let source = openers(
        "intake",
        "{all: [{children: {in: [passed], subject: intake}}, \
         {children: {in: [passed], subject: wave, revision: current}}]}",
    );
    assert_eq!(
        checked(&source),
        "checked: protocol `p` revision 1: 15552 states, 0 properties, 0 findings\n"
    );
}

/// Behind m3 (`reads_in` keeping the subject only with `revision: current`): a form naming
/// `wave` without `revision` reads only the dimension about `wave`, and the outcome is reachable.
#[test]
fn check_reads_the_subject_of_a_form_without_revision() {
    let source = openers("intake", "{children: {in: [passed], subject: wave}}").replacen(
        "requires: [{capability: dispatch}], ",
        "",
        1,
    );
    assert_eq!(
        checked(&source),
        "checked: protocol `p` revision 1: 72 states, 0 properties, 0 findings\n"
    );
}

/// Behind m4 (`normalize` dropping the subject of a form under `not`): the compiled requirement
/// keeps it, and `canon check` reads the dimensions about both artifacts.
#[test]
fn check_and_compile_keep_the_subject_of_a_form_under_not() {
    let source = openers(
        "intake",
        "{all: [{not: {children: {in: [passed], subject: wave}}}, \
         {children: {in: [passed], subject: intake}}]}",
    )
    .replacen("requires: [{capability: dispatch}], ", "", 1);
    let value: Value = serde_json::from_str(&compiled(&source).canonical_json()).expect("JSON");
    assert_eq!(
        value["outcomes"]["wave.done"]["requires"],
        json!({"all": [
            {"children": {"in": ["passed"], "subject": "intake"}},
            {"not": {"children": {"in": ["passed"], "subject": "wave"}}},
        ]})
    );
    assert_eq!(
        checked(&source),
        "checked: protocol `p` revision 1: 5184 states, 0 properties, 0 findings\n"
    );
}

// ---------------------------------------------------------------------------------------------
// Correction 2, after story:simulate-child-outcome-step merged: D1 in `canon simulate`. A child
// of an opening action records only in the `case_outcome` dimension that action produces, the
// replacing successor replaces `open` in that dimension only, and an `open` record keeps a mark
// above none on an action that produces its dimension (D5 narrowed).

fn simulated(source: &str) -> String {
    simulate(&compiled(source))
        .unwrap_or_else(|refusal| panic!("{}: {refusal}", refusal.code()))
        .to_string()
}

/// The fixture: `ideas.open`'s children record in the dimension without a subject, standing for
/// `intake`, and `stories.open`'s in the one about `wave`. Unbound, the walk reached 528 states, 68
/// of them terminable, some through an idea child recorded about `wave`.
#[test]
fn simulate_over_the_fixture_records_each_child_about_its_opening() {
    assert_eq!(
        simulated(&fixture()),
        "cycle: state {evidence `case_outcome` about `wave`, action `ideas.open` opened 2 or more \
         children, action `stories.open` opened 2 or more children}: no outcome is legitimate, no \
         action is approval-required, and every step of action `ideas.open`, action \
         `stories.open` returns this state; path: action `ideas.open` produces nothing, then \
         action `ideas.open` produces nothing, then action `stories.open` produces nothing, then \
         action `stories.open` produces nothing, then a child of action `stories.open` records \
         evidence `case_outcome` about `wave`; 31 other cycle states contain it\n\
         simulated: protocol `orchestrate` revision 1: 400 reachable states, 60 terminable \
         (outcome `wave.done` 60), 24 waiting on supplied evidence, a child outcome or expiry, 1 \
         finding (32 stuck or cycle states)\n"
    );
}

/// Only `stories.open`, which requires `dispatch`, opens children about `wave`. Unbound, a child
/// of the free `ideas.open` recorded about `wave` too: 108 states, 15 terminable, among them
/// `{evidence `case_outcome` about `wave` result `passed`}` with no capability decided, which no
/// case reaches. Bound, `wave.done` is legitimate only after `dispatch` is granted.
#[test]
fn simulate_reaches_an_outcome_only_through_a_child_about_its_subject() {
    let text = simulated(&openers(
        "intake",
        "{children: {in: [passed], subject: wave}}",
    ));
    assert!(
        !text.contains("a child of action `ideas.open` records"),
        "{text}"
    );
    assert_eq!(
        text,
        "cycle: state {capability `dispatch` denied, action `ideas.open` opened 2 or more \
         children}: no outcome is legitimate, no action is approval-required, and every step of \
         action `ideas.open` returns this state; path: action `ideas.open` produces nothing, then \
         action `ideas.open` produces nothing, then capability `dispatch` denied\n\
         cycle: state {evidence `case_outcome` about `wave`, capability `dispatch` granted, action \
         `ideas.open` opened 2 or more children, action `stories.open` opened 2 or more \
         children}: no outcome is legitimate, no action is approval-required, and every step of \
         action `ideas.open`, action `stories.open` returns this state; path: action `ideas.open` \
         produces nothing, then action `ideas.open` produces nothing, then capability `dispatch` \
         granted, then action `stories.open` produces nothing, then action `stories.open` \
         produces nothing, then a child of action `stories.open` records evidence `case_outcome` \
         about `wave`; 3 other cycle states contain it\n\
         simulated: protocol `p` revision 1: 66 reachable states, 9 terminable (outcome \
         `wave.done` 9), 3 waiting on supplied evidence, a child outcome or expiry, 2 findings (5 \
         stuck or cycle states)\n"
    );
}

/// Two opening actions about two artifacts, each read by a form naming it: `beta.open` is
/// admissible once the children about `alpha` passed, and `alpha.open` while they are unknown.
/// An `open` record about `alpha` needs a mark on `alpha.open`: a state holding it with a mark
/// only on `beta.open` (an `alpha` child that ended beside its own `open` record) is no state.
const PAIR: &str = "format: protocol/1\nprotocol: {id: pair, revision: 1}\n\
    artifacts: {alpha: {}, beta: {}}\nevidence_kinds: {case_outcome: {}}\n\
    claims:\n  \
    alpha.passed: {true_when: {children: {in: [passed], subject: alpha}}}\n  \
    beta.passed: {true_when: {children: {in: [passed], subject: beta}}}\n\
    actions:\n  \
    alpha.open:\n    precondition: {claim: alpha.passed, is: unknown}\n    \
    case_open: {protocol: q, subject: alpha}\n  \
    beta.open:\n    precondition: {claim: alpha.passed}\n    \
    case_open: {protocol: q, subject: beta}\n\
    outcomes:\n  done:\n    requires: {all: [{claim: alpha.passed}, {claim: beta.passed}]}\n";

#[test]
fn simulate_keeps_an_open_record_only_with_a_mark_on_its_producer() {
    assert_eq!(
        simulated(PAIR),
        "stuck-state: state {evidence `case_outcome` about `alpha`}: no outcome is legitimate and \
         no action is admissible or approval-required; path: action `alpha.open` produces \
         nothing, then a child of action `alpha.open` records evidence `case_outcome` about \
         `alpha`; 104 other stuck states contain it\n\
         cycle: state {evidence `case_outcome` about `alpha` result `passed`, evidence \
         `case_outcome` about `beta`, action `beta.open` opened 2 or more children}: no outcome \
         is legitimate, no action is approval-required, and every step of action `beta.open` \
         returns this state; path: action `alpha.open` produces nothing, then a child of action \
         `alpha.open` records evidence `case_outcome` about `alpha` result `passed`, then action \
         `beta.open` produces nothing, then action `beta.open` produces nothing, then a child of \
         action `beta.open` records evidence `case_outcome` about `beta`; 15 other cycle states \
         contain it\n\
         cycle: state {evidence `case_outcome` about `alpha` result `open`, evidence \
         `case_outcome` about `alpha` result `passed`, evidence `case_outcome` about `beta`, \
         action `alpha.open` opened 2 or more children}: no outcome is legitimate, no action is \
         approval-required, and every step of action `alpha.open` returns this state; path: \
         action `alpha.open` produces nothing, then action `alpha.open` produces nothing, then a \
         child of action `alpha.open` records evidence `case_outcome` about `alpha` result \
         `passed`, then action `beta.open` produces nothing, then a child of action `alpha.open` \
         records evidence `case_outcome` about `alpha` result `open`, then a child of action \
         `beta.open` records evidence `case_outcome` about `beta`; 9 other cycle states contain \
         it\n\
         simulated: protocol `pair` revision 1: 210 reachable states, 9 terminable (outcome \
         `done` 9), 21 waiting on supplied evidence, a child outcome or expiry, 3 findings (127 \
         stuck or cycle states)\n"
    );
}

/// D3 in the walk: no action opens a child about `wave`, and the form names it, so no dimension
/// `ideas.open` produces is read: its children add no record. Every finding is true: `wave.done`
/// is legitimate in no case, and a case that took `ideas.open` twice has nothing left to change.
#[test]
fn simulate_reports_an_outcome_over_a_subject_no_child_is_opened_about() {
    let source = openers("intake", "{children: {in: [passed], subject: wave}}").replacen(
        "  stories.open: {requires: [{capability: dispatch}], case_open: {protocol: story, subject: wave}}\n",
        "",
        1,
    );
    assert_eq!(
        simulated(&source),
        "cycle: state {action `ideas.open` opened 2 or more children}: no outcome is legitimate, \
         no action is approval-required, and every step of action `ideas.open` returns this \
         state; path: action `ideas.open` produces nothing, then action `ideas.open` produces \
         nothing\n\
         unreached-outcome: outcome `wave.done` is legitimate in no reachable state\n\
         no-outcome-reachable: protocol `p` revision 1 reaches no state in which an outcome is \
         legitimate\n\
         simulated: protocol `p` revision 1: 3 reachable states, 0 terminable (outcome \
         `wave.done` 0), 0 waiting on supplied evidence, a child outcome or expiry, 3 findings (1 \
         stuck or cycle state)\n"
    );
}
