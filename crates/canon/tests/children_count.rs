//! story:children-form-counts-unopened-children against the library: what `canon validate`
//! refuses of a `children` form's `count`, the canon-ir/1 form and its read-back, the values over
//! the records the form reads, the reasons and the explanation, the semantic diff, `canon check`
//! and `canon simulate`. CANON-COMPOSE-004 (conformance/scenarios/children-count.yaml) states the
//! values over the fixture; this file states the rest. The fixture is the five-reader protocol of
//! `fixtures/investigation/simulate/children-count/`, `sources` at `s1` and `merged` at `m1`.

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
        .join("../../fixtures/investigation/simulate/children-count/protocol.yaml");
    std::fs::read_to_string(&path).expect("the children-count fixture reads")
}

fn compiled(source: &str) -> Ir {
    let parsed = model::parse(source).unwrap_or_else(|error| panic!("parses: {error}"));
    ir::compile(&parsed).unwrap_or_else(|problems| panic!("compiles: {problems:?}"))
}

/// Every problem `canon validate` reports of `source`, as `code: message`.
fn problems(source: &str) -> Vec<String> {
    match validate(&model::parse(source).unwrap_or_else(|error| panic!("parses: {error}"))) {
        Ok(()) => Vec::new(),
        Err(problems) => problems
            .iter()
            .map(|problem| format!("{}: {problem}", problem.code()))
            .collect(),
    }
}

fn record(id: &str, kind: &str, result: Option<&str>, child: Option<&str>) -> EvidenceRecord {
    record_at(id, kind, result, child, "s1")
}

fn record_at(
    id: &str,
    kind: &str,
    result: Option<&str>,
    child: Option<&str>,
    revision: &str,
) -> EvidenceRecord {
    let mut text = format!("format: canon-evidence/1\nid: {id}\nkind: {kind}\n");
    if let Some(result) = result {
        text.push_str(&format!("result: {result}\n"));
    }
    text.push_str(&format!("subject: sources\nsubject_revision: {revision}\n"));
    if let Some(child) = child {
        text.push_str(&format!("child_case: {child}\n"));
    }
    eval::read_evidence(&text).expect("a record")
}

/// The `case_outcome` record of reader child `n` with `result`, about `sources` at `s1`.
fn reader(n: usize, result: &str) -> EvidenceRecord {
    record(
        &format!("r{n}"),
        "case_outcome",
        Some(result),
        Some(&format!("READER-{n}")),
    )
}

/// The `*_opened` records of the readers `letters`.
fn opened(letters: &str) -> Vec<EvidenceRecord> {
    letters
        .chars()
        .map(|letter| {
            record(
                &format!("o{letter}"),
                &format!("{letter}_opened"),
                None,
                None,
            )
        })
        .collect()
}

/// `sources` at `revision`, `merged` at `m1`.
fn case_at(revision: &str) -> model::Case {
    eval::read_case(&format!(
        "format: canon-case/1\nid: READ-1\nprotocol: simulate.children-count\n\
         artifacts: {{sources: {{revision: {revision}}}, merged: {{revision: m1}}}}\n"
    ))
    .expect("case reads")
}

fn claim(decision: &model::Decision, id: &str) -> Truth {
    decision
        .claims
        .get(&ClaimId::new(id))
        .unwrap_or_else(|| panic!("claim {id}"))
        .value
}

/// Expectation 8: a count of 0 is `zero-child-count`, naming the declaration the form is written
/// in, after `open-child-outcome` and before the kind `case_outcome` when that is undeclared; a
/// negative count is not an integer of the field's type and does not parse. A count of 1 is
/// accepted, and so is the fixture.
#[test]
fn validate_refuses_a_count_of_zero_naming_the_declaration() {
    let zero = fixture().replacen(
        "{in: [complete], subject: sources, count: 5}}\n  merged.none",
        "{in: [complete], subject: sources, count: 0}}\n  merged.none",
        1,
    );
    assert_ne!(zero, fixture(), "the claim's count was replaced");
    assert_eq!(
        problems(&zero),
        [
            "zero-child-count: claim `readers.complete` waits for 0 child cases; a count is 1 or more"
        ]
    );
    let in_action = fixture().replacen(
        "- {children: {in: [complete], subject: sources, count: 5}}",
        "- {children: {in: [complete], subject: sources, count: 0}}",
        1,
    );
    assert_ne!(in_action, fixture(), "the action's count was replaced");
    assert_eq!(
        problems(&in_action),
        ["zero-child-count: action `merge.open` waits for 0 child cases; a count is 1 or more"]
    );
    let undeclared_kind = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {sources: {}}\n\
        claims: {c: {true_when: {children: {in: [open], count: 0}}}}\n";
    assert_eq!(
        problems(undeclared_kind),
        [
            "open-child-outcome: claim `c` tests the child cases for outcome `open`, which a running child case has, never a terminal one",
            "zero-child-count: claim `c` waits for 0 child cases; a count is 1 or more",
            "undeclared-evidence-kind: claim `c` references evidence kind `case_outcome`, which is not declared",
        ]
    );
    let negative = fixture().replacen("count: 5}}\n  merged.none", "count: -1}}\n  merged.none", 1);
    assert_ne!(negative, fixture(), "the count was made negative");
    assert!(model::parse(&negative).is_err(), "a negative count parses");
    let one = fixture().replace("count: 5", "count: 1");
    assert_eq!(problems(&one), Vec::<String>::new());
    assert_eq!(problems(&fixture()), Vec::<String>::new());
}

/// Expectation 1: `"count"` is written only when the form declares it, first among the form's
/// keys as canonical JSON orders them, and reads back; the read-back refuses every other spelling
/// of it: `null`, text, 0, a negative number, a fraction, and the key out of its canonical place.
/// Without the key the IR is the one the protocol had before the key existed.
#[test]
fn the_ir_writes_count_only_when_declared_and_reads_it_back() {
    let ir = compiled(&fixture());
    let text = ir.canonical_json();
    let value: Value = serde_json::from_str(&text).expect("JSON");
    assert_eq!(
        value["claims"]["readers.complete"]["true_when"],
        json!({"children": {"count": 5, "in": ["complete"], "subject": "sources"}})
    );
    assert_eq!(eval::read_ir(&text).expect("reads"), ir);

    let uncounted = compiled(&fixture().replace(", count: 5}}", "}}"));
    let plain = uncounted.canonical_json();
    assert!(!plain.contains("\"count\""), "{plain}");
    assert_eq!(
        text.replace("\"count\": 5,\n", "").replace(' ', ""),
        plain.replace(' ', "")
    );

    let written = "\"count\": 5,\n          \"in\": [\n            \"complete\"\n          ],";
    assert!(text.contains(written), "{text}");
    for (other, why) in [
        ("\"count\": null,", "a null count"),
        ("\"count\": \"5\",", "a count as text"),
        ("\"count\": 0,", "a count of 0"),
        ("\"count\": -5,", "a negative count"),
        ("\"count\": 5.5,", "a fraction"),
    ] {
        let changed = text.replacen(written, &written.replacen("\"count\": 5,", other, 1), 1);
        assert_ne!(changed, text, "{why}: the text was changed");
        assert_eq!(
            eval::read_ir(&changed).expect_err(why).code(),
            "malformed-input",
            "{why}"
        );
    }
    let after_in = text.replacen(
        written,
        "\"in\": [\n            \"complete\"\n          ],\n          \"count\": 5,",
        1,
    );
    assert_ne!(after_in, text, "the key was moved");
    assert_eq!(
        eval::read_ir(&after_in)
            .expect_err("the key after `in`")
            .code(),
        "malformed-input"
    );
}

/// Two forms that differ only in `count` are two members of an `all`, no count first and then
/// the smaller first; one written twice is one member.
#[test]
fn forms_that_differ_only_in_count_are_two_members_no_count_first() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {sources: {}}\nevidence_kinds: {case_outcome: {}}\n\
        claims: {c: {true_when: {all: [{children: {in: [passed], count: 12}}, \
        {children: {in: [passed], count: 3}}, {children: {in: [passed]}}, \
        {children: {in: [passed], count: 12}}]}}}\n";
    let value: Value = serde_json::from_str(&compiled(source).canonical_json()).expect("JSON");
    assert_eq!(
        value["claims"]["c"]["true_when"],
        json!({"all": [
            {"children": {"in": ["passed"]}},
            {"children": {"count": 3, "in": ["passed"]}},
            {"children": {"count": 12, "in": ["passed"]}},
        ]})
    );
}

/// The values over the records the form reads: UNKNOWN with fewer records than the count, a
/// reader not yet opened having none; TRUE at the count and above it; FALSE once one ended
/// outside the set, however few were read; UNKNOWN while one is open. With `revision: current`
/// only the records about the current revision count. No record is listed in
/// `excluded_evidence`.
#[test]
fn the_form_waits_for_count_records() {
    let ir = compiled(&fixture());
    let complete = |n: usize| (1..=n).map(|n| reader(n, "complete")).collect::<Vec<_>>();
    let with = |mut records: Vec<EvidenceRecord>, more: Vec<EvidenceRecord>| {
        records.extend(more);
        records
    };
    for (records, value, why) in [
        (opened("abcd"), Truth::Unknown, "four opened, none ended"),
        (
            with(opened("abcd"), complete(4)),
            Truth::Unknown,
            "four ended, e not opened",
        ),
        (
            with(opened("abcde"), complete(5)),
            Truth::True,
            "five ended",
        ),
        (
            with(opened("abcde"), complete(6)),
            Truth::True,
            "six ended, more than the count",
        ),
        (
            with(opened("abcde"), with(complete(4), vec![reader(5, "open")])),
            Truth::Unknown,
            "the fifth open",
        ),
        (
            with(opened("ab"), vec![reader(1, "complete"), reader(2, "lost")]),
            Truth::False,
            "one outside the set, three not opened",
        ),
        (
            with(opened("abcde"), with(complete(5), vec![reader(6, "open")])),
            Truth::Unknown,
            "five ended, a sixth open",
        ),
    ] {
        let decision = eval::evaluate(&ir, &case_at("s1"), &records).expect("decides");
        assert_eq!(claim(&decision, "readers.complete"), value, "{why}");
        let excluded = &decision
            .claims
            .get(&ClaimId::new("readers.complete"))
            .expect("the claim")
            .excluded_evidence;
        assert!(excluded.is_empty(), "{why}");
    }
    let current = compiled(&fixture().replacen(
        "{in: [complete], subject: sources, count: 5}}\n  merged.none",
        "{in: [complete], subject: sources, count: 2, revision: current}}\n  merged.none",
        1,
    ));
    let earlier = record_at(
        "r1",
        "case_outcome",
        Some("complete"),
        Some("READER-1"),
        "s0",
    );
    for (records, value, why) in [
        (
            vec![earlier.clone(), reader(2, "complete")],
            Truth::Unknown,
            "one current, one earlier",
        ),
        (
            vec![earlier, reader(2, "complete"), reader(3, "complete")],
            Truth::True,
            "two current",
        ),
    ] {
        let decision = eval::evaluate(&current, &case_at("s1"), &records).expect("decides");
        assert_eq!(claim(&decision, "readers.complete"), value, "{why}");
    }
}

/// Reasons and the explanation name the form with `"count"`; two forms that differ only in
/// `count` are two reasons, no count first, each with its own value. The same inputs give the
/// same bytes.
#[test]
fn reasons_and_the_explanation_name_the_form_with_its_count() {
    let source = fixture().replacen(
        "        - {children: {in: [complete], subject: sources, count: 5}}\n",
        "        - {children: {in: [complete], subject: sources, count: 5}}\n\
         \x20       - {children: {in: [complete], subject: sources}}\n",
        1,
    );
    assert_ne!(source, fixture(), "the action was extended");
    let ir = compiled(&source);
    let mut records = opened("abcd");
    records.extend((1..=4).map(|n| reader(n, "complete")));
    let decision = eval::evaluate(&ir, &case_at("s1"), &records).expect("decides");
    let actions = decision.actions.as_ref().expect("actions");
    assert_eq!(actions["merge.open"]["status"], json!("blocked"));
    assert_eq!(
        actions["merge.open"]["reasons"],
        json!([
            {"children": ["complete"], "count": 5, "subject": "sources", "value": "unknown"},
        ])
    );
    let explanation = decision.explanation.as_ref().expect("an explanation");
    assert_eq!(
        explanation["claims"]["readers.complete"],
        json!({
            "because": [{"children": ["complete"], "count": 5, "subject": "sources", "value": "unknown"}],
            "value": "unknown",
        })
    );
    records.push(reader(5, "lost"));
    let decision = eval::evaluate(&ir, &case_at("s1"), &records).expect("decides");
    let actions = decision.actions.as_ref().expect("actions");
    assert_eq!(
        actions["merge.open"]["reasons"],
        json!([
            {"children": ["complete"], "subject": "sources", "value": "false"},
            {"children": ["complete"], "count": 5, "subject": "sources", "value": "false"},
        ])
    );
    let again = eval::evaluate(&ir, &case_at("s1"), &records).expect("decides");
    assert_eq!(eval::render(&again), eval::render(&decision));
}

/// Adding, removing or changing `count` on a form is `BREAKING`, as any change to a form is.
#[test]
fn the_diff_takes_a_changed_count_as_breaking() {
    let from = compiled(&fixture());
    let later = |text: String| compiled(&text.replacen("revision: 1", "revision: 2", 1));
    for (to, why) in [
        (
            later(fixture().replacen(
                "subject: sources, count: 5}}\n  merged.none",
                "subject: sources, count: 4}}\n  merged.none",
                1,
            )),
            "a changed count",
        ),
        (
            later(fixture().replacen(
                "subject: sources, count: 5}}\n  merged.none",
                "subject: sources}}\n  merged.none",
                1,
            )),
            "a removed count",
        ),
    ] {
        let changes = diff(&from, &to).expect("diffs").changes;
        assert_eq!(changes.len(), 1, "{why}: {changes:?}");
        assert_eq!(changes[0].category, Category::Breaking, "{why}");
        assert!(
            changes[0]
                .to_string()
                .starts_with("BREAKING: claim `readers.complete` "),
            "{why}: {}",
            changes[0]
        );
    }
}

fn checked(source: &str) -> String {
    check(&compiled(source), None)
        .unwrap_or_else(|refusal| panic!("{}: {refusal}", refusal.code()))
        .to_string()
}

/// One opening action about `sources`, which requires `requires` when given, and an outcome that
/// requires `requirement`.
fn counted(requirement: &str, requires: &str) -> String {
    format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\n\
         artifacts: {{sources: {{}}}}\nevidence_kinds: {{case_outcome: {{}}}}\n\
         actions:\n\x20\x20open: {{{requires}case_open: {{protocol: reader, subject: sources}}}}\n\
         outcomes:\n\x20\x20done: {{requires: {requirement}}}\n"
    )
}

/// `canon check` gives the `case_outcome` dimension a counting form reads a dimension of more
/// records: three classes (`open`, `passed`, another) make 8 values, the history 9, and `count: 3`
/// 3 values of more records (none, one, two), so 216 states, and `done` is reachable. Without
/// that dimension a state holds at most one current and one earlier `passed` record, and `done`
/// would be unreachable. `count: 1` asks no more than a form without the key, and adds no
/// dimension: 72 states, as without it.
#[test]
fn check_counts_records_beyond_one_per_class() {
    assert_eq!(
        checked(&counted(
            "{children: {in: [passed], subject: sources, count: 3}}",
            ""
        )),
        "checked: protocol `p` revision 1: 216 states, 0 properties, 0 findings\n"
    );
    assert_eq!(
        checked(&counted(
            "{children: {in: [passed], subject: sources, count: 1}}",
            ""
        )),
        "checked: protocol `p` revision 1: 72 states, 0 properties, 0 findings\n"
    );
    assert_eq!(
        checked(&counted("{children: {in: [passed], subject: sources}}", "")),
        "checked: protocol `p` revision 1: 72 states, 0 properties, 0 findings\n"
    );
}

/// The witness of a finding writes the more records after the class they repeat: with a second,
/// free opener about `sources`, the least state in which `done` is legitimate without authority
/// holds one `passed` record and two more of it, weight 3. Of the states of that weight, one
/// earlier record and two more renders first (`` earlier `` before `` , ``), ahead of one current
/// record and two more: a form without `revision: current` reads both alike.
#[test]
fn check_writes_the_more_records_of_a_witness() {
    let source = counted(
        "{children: {in: [passed], subject: sources, count: 3}}",
        "requires: [{capability: dispatch}], ",
    )
    .replacen(
        "outcomes:\n",
        "\x20\x20free: {case_open: {protocol: reader, subject: sources}}\noutcomes:\n",
        1,
    );
    assert_eq!(
        checked(&source),
        "authority-bypass: outcome `done` is legitimate without authority in state {evidence \
         `case_outcome` about `sources` result `passed` earlier, evidence `case_outcome` about \
         `sources` result `passed` earlier 2 more times}\n\
         checked: protocol `p` revision 1: 648 states, 0 properties, 1 finding\n"
    );
}

/// A count the bound cannot hold refuses the check, naming the dimension of more records with
/// its values.
#[test]
fn check_refuses_a_count_beyond_the_bound_naming_its_dimension() {
    let refusal = check(
        &compiled(&counted(
            "{children: {in: [passed], subject: sources, count: 70000}}",
            "",
        )),
        None,
    )
    .expect_err("refused");
    assert_eq!(refusal.code(), "state-space-bound");
    assert_eq!(
        refusal.to_string(),
        "protocol `p` revision 1: outcome `done` reads 5040000 states, more than the bound of \
         65536: evidence kind `case_outcome` about `sources` 8, history of evidence kind \
         `case_outcome` about `sources` 9, more records of evidence kind `case_outcome` about \
         `sources` 70000"
    );
}

fn simulated(source: &str) -> String {
    simulate(&compiled(source))
        .unwrap_or_else(|refusal| panic!("{}: {refusal}", refusal.code()))
        .to_string()
}

/// Expectation 2 of the story's acceptance in `canon simulate`: with `count: 5` the claim is never
/// true while a reader is unopened (`early`), and the merge never finds nothing after a read that
/// was not complete (the gap-8 probe); without the key both are reached, as gap 2 said.
#[test]
fn simulate_never_takes_the_count_as_reached_before_every_reader_opened() {
    let counted = simulated(&fixture());
    for probe in ["early", "nobody-after-incomplete-read"] {
        assert!(
            counted.contains(&format!(
                "unreached-outcome: outcome `{probe}` is legitimate in no reachable state\n"
            )),
            "{probe}: {counted}"
        );
    }
    let uncounted = simulated(&fixture().replace(", count: 5}}", "}}"));
    assert!(!uncounted.contains("unreached-outcome"), "{uncounted}");
}

/// The doc duty: the count is of records, not of opening actions. An opener whose precondition
/// stays true opens a second child, whose record counts, so `done` with `count: 2` is reached
/// through one opener. A child that ends outside the set leaves the form false for good, and the
/// printed cycle state writes how many children the walk counts in the dimension, two being the
/// count and so written `2 or more`.
#[test]
fn simulate_counts_every_child_an_unguarded_opener_opens() {
    let report = simulated(&counted(
        "{children: {in: [passed], subject: sources, count: 2}}",
        "",
    ));
    assert!(!report.contains("unreached-outcome"), "{report}");
    assert!(
        report.contains("2 or more children in evidence `case_outcome` about `sources`"),
        "{report}"
    );
}

/// With `revision: current` the walk counts the children at the current revision apart: two
/// children that pass after the last revision make `done` legitimate, and a printed state writes
/// that count beside the other. `redo` revises `sources` only while every child passed, so a
/// child that ends outside the set leaves a cycle state to print.
#[test]
fn simulate_counts_the_children_at_the_current_revision_apart() {
    let report = simulated(
        &counted(
            "{children: {in: [passed], subject: sources, revision: current, count: 2}}",
            "",
        )
        .replacen(
            "outcomes:\n",
            "\x20\x20redo: {precondition: {children: {in: [passed], subject: sources}}, \
             revises: [sources]}\noutcomes:\n",
            1,
        ),
    );
    assert!(!report.contains("unreached-outcome"), "{report}");
    assert!(report.contains(" at the current revision)"), "{report}");
}
