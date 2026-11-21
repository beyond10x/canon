//! story:child-outcome-subject against the library: what `canon validate` refuses of a `children`
//! form's `subject`, the canon-ir/1 form and its read-back, the reasons and the explanation, the
//! semantic diff and `canon check`. CANON-COMPOSE-003
//! (conformance/scenarios/child-outcome-subject.yaml) states the values; this file states the
//! rest. Every case gives the two artifacts two different revisions (`intake` at `n1`, `wave` at
//! `w2`) and holds children about both.

use std::path::Path;

use b10x_canon::check::check;
use b10x_canon::diff::{Category, diff};
use b10x_canon::eval;
use b10x_canon::ir::{self, Ir};
use b10x_canon::model::{self, ClaimId, EvidenceRecord, Truth};
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
