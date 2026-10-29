//! story:case-composition against the library: what `canon validate` refuses of an action's
//! `case_open` and of a `children` form, what the evaluator refuses of a `case_outcome` record, the
//! canon-ir/1 round trip, the static check that a set of protocols opens no cycle of cases (D206),
//! `canon check` over the form jointly with evidence matches, the explanation and the semantic diff.
//! CANON-COMPOSE-001 (conformance/scenarios/case-composition.yaml) states the values; this file
//! states the rest.

use std::collections::BTreeMap;
use std::path::Path;

use b10x_canon::check::check;
use b10x_canon::diff::{Category, diff};
use b10x_canon::eval;
use b10x_canon::ir::{self, Ir, check_openings};
use b10x_canon::model::{self, EvidenceRecord, Truth};
use b10x_canon::validate::validate;
use serde_json::{Value, json};

fn fixture() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/investigation/case-composition.yaml");
    std::fs::read_to_string(&path).expect("the case-composition fixture reads")
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

/// One record, its fields written as a consumer's YAML would.
fn record(
    id: &str,
    kind: &str,
    child: Option<&str>,
    result: Option<&str>,
    subject: &str,
    revision: &str,
) -> EvidenceRecord {
    let mut text = format!(
        "format: canon-evidence/1\nid: '{id}'\nkind: {kind}\nsubject: {subject}\nsubject_revision: {revision}\n"
    );
    if let Some(child) = child {
        text.push_str(&format!("child_case: '{child}'\n"));
    }
    if let Some(result) = result {
        text.push_str(&format!("result: {result}\n"));
    }
    eval::read_evidence(&text).expect("a record")
}

fn outcome(id: &str, child: &str, result: &str) -> EvidenceRecord {
    record(
        id,
        "case_outcome",
        Some(child),
        Some(result),
        "findings",
        "f1",
    )
}

/// A case with every artifact of `artifacts` at revision `revision`.
fn case(protocol: &str, artifacts: &[&str], revision: &str) -> model::Case {
    let listed: Vec<String> = artifacts
        .iter()
        .map(|artifact| format!("{artifact}: {{revision: {revision}}}"))
        .collect();
    eval::read_case(&format!(
        "format: canon-case/1\nid: C-1\nprotocol: {protocol}\nartifacts: {{{}}}\n",
        listed.join(", ")
    ))
    .expect("a case")
}

/// The evaluator's refusal of `evidence` against the fixture, as `code: message`.
fn refused(evidence: &[EvidenceRecord]) -> String {
    let ir = compiled(&SECOND_KIND);
    let refusal =
        eval::evaluate(&ir, &case("audit", &["findings"], "f1"), evidence).expect_err("refused");
    format!("{}: {refusal}", refusal.code())
}

/// The fixture's protocol with a second evidence kind, `review`, beside `case_outcome`.
static SECOND_KIND: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    fixture().replacen("evidence_kinds:\n", "evidence_kinds:\n  review: {}\n", 1)
});

/// D206: a malformed protocol id, an undeclared subject and an opening under the protocol's own
/// id are each refused; the last two name the action.
#[test]
fn validate_refuses_each_malformed_opening() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {findings: {}}\n\
        actions:\n\
        \x20\x20a: {case_open: {protocol: 'bad id', subject: findings}}\n\
        \x20\x20b: {case_open: {protocol: q, subject: missing}}\n\
        \x20\x20c: {case_open: {protocol: p, subject: findings}}\n";
    assert_eq!(
        problems(source),
        [
            "invalid-identifier: protocol identifier `bad id` is empty or contains whitespace or a control character",
            "undeclared-artifact: action `b` references artifact `missing`, which is not declared",
            "self-opened-case: action `c` opens a case under protocol `p`, the protocol's own id",
        ]
    );
    assert_eq!(problems(&fixture()), Vec::<String>::new());
}

/// A `children` form that lists nothing, an outcome twice or `open`, or is written in a protocol
/// that does not declare `case_outcome`, is refused, naming where it is written; in a discharge or
/// an applicability predicate it is evidence, which neither may test.
#[test]
fn validate_refuses_each_malformed_children_form() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {findings: {}}\n\
        claims:\n\
        \x20\x20c: {true_when: {children: {in: []}}}\n\
        \x20\x20d: {true_when: {all: [{children: {in: [accepted, accepted, open]}}]}}\n";
    assert_eq!(
        problems(source),
        [
            "empty-child-outcomes: claim `c` tests the child cases for no outcome",
            "undeclared-evidence-kind: claim `c` references evidence kind `case_outcome`, which is not declared",
            "duplicate-child-outcome: claim `d` tests the child cases for outcome `accepted` more than once",
            "open-child-outcome: claim `d` tests the child cases for outcome `open`, which a running child case has, never a terminal one",
            "undeclared-evidence-kind: claim `d` references evidence kind `case_outcome`, which is not declared",
        ]
    );
    let guarded = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {findings: {}}\nevidence_kinds: {case_outcome: {}}\n\
        obligations:\n\
        \x20\x20o: {discharged_when: {not: {children: {in: [accepted]}}}, applicable_when: {children: {in: [declined]}}}\n";
    assert_eq!(
        problems(guarded),
        [
            "evidence-in-discharge: obligation `o` tests evidence kind `case_outcome` at `discharged_when.not.children`; a discharge predicate tests only claim values",
            "evidence-in-applicability: obligation `o` tests evidence kind `case_outcome` at `applicable_when.children`; an applicability predicate tests only claim values and case inputs",
        ]
    );
}

/// A `case_outcome` record names its child and gives its outcome; a record of another kind names
/// no child; a child id is an identifier; two records for one child are refused naming both.
#[test]
fn the_evaluator_refuses_each_malformed_case_outcome_record() {
    let cases = [
        (
            record(
                "o1",
                "case_outcome",
                None,
                Some("accepted"),
                "findings",
                "f1",
            ),
            "missing-child-case: evidence `o1` is a case_outcome record and names no child case",
        ),
        (
            record("o1", "case_outcome", Some("CHG-1"), None, "findings", "f1"),
            "missing-child-outcome: evidence `o1` is a case_outcome record and gives no outcome",
        ),
        (
            record(
                "r1",
                "review",
                Some("CHG-1"),
                Some("pass"),
                "findings",
                "f1",
            ),
            "misplaced-child-case: evidence `r1` names a child case, which only a case_outcome record does; it is of kind `review`",
        ),
        (
            record(
                "o1",
                "case_outcome",
                Some("CHG 1"),
                Some("open"),
                "findings",
                "f1",
            ),
            "invalid-identifier: evidence `o1` child case identifier `CHG 1` is empty or contains whitespace or a control character",
        ),
    ];
    for (given, expected) in cases {
        assert_eq!(refused(&[given]), expected);
    }
    assert_eq!(
        refused(&[
            outcome("o3", "CHG-2", "open"),
            outcome("o2", "CHG-1", "accepted"),
            outcome("o1", "CHG-2", "declined"),
        ]),
        "duplicate-child-case: child case `CHG-2` has more than one case_outcome record: `o1`, `o3`"
    );
}

/// The IR writes `case_open` and the `children` form, each outcome once in code-point order, and
/// reads back to the protocol it was written from.
#[test]
fn the_ir_writes_the_opening_and_the_form_and_reads_them_back() {
    let compiled = compiled(&fixture());
    let text = compiled.canonical_json();
    let value: Value = serde_json::from_str(&text).expect("JSON");
    assert_eq!(
        value["actions"]["open_change"]["case_open"],
        json!({"protocol": "software.change", "subject": "findings"})
    );
    assert_eq!(
        value["claims"]["findings.resolved"]["true_when"],
        json!({"children": {"in": ["accepted", "declined"]}})
    );
    assert!(value["actions"]["close"].get("case_open").is_none());
    assert_eq!(eval::read_ir(&text).expect("reads"), compiled);

    let reversed = compiled_with_claim("{children: {in: [b, a]}}");
    let value: Value = serde_json::from_str(&reversed.canonical_json()).expect("JSON");
    assert_eq!(
        value["claims"]["c"]["true_when"],
        json!({"children": {"in": ["a", "b"]}})
    );
    // Between `before` and `claim` in canonical order; two forms by their lists, shorter first.
    let ordered = compiled_with_claim(
        "{all: [{claim: d}, {children: {in: [b]}}, {children: {in: [a, b]}}, {children: {in: [a]}}]}",
    );
    let value: Value = serde_json::from_str(&ordered.canonical_json()).expect("JSON");
    assert_eq!(
        value["claims"]["c"]["true_when"],
        json!({"all": [
            {"children": {"in": ["a"]}},
            {"children": {"in": ["a", "b"]}},
            {"children": {"in": ["b"]}},
            {"claim": {"id": "d", "is": "true"}},
        ]})
    );
}

fn compiled_with_claim(true_when: &str) -> Ir {
    compiled(&format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\n\
         artifacts: {{findings: {{}}}}\nevidence_kinds: {{case_outcome: {{}}}}\n\
         claims:\n\x20\x20c: {{true_when: {true_when}}}\n\x20\x20d: {{true_when: {{all: []}}}}\n"
    ))
}

/// A protocol `id` that opens a case under each of `opens`.
fn opener(id: &str, opens: &[&str]) -> Ir {
    let actions: String = opens
        .iter()
        .enumerate()
        .map(|(n, opened)| {
            format!("  open{n}: {{case_open: {{protocol: '{opened}', subject: a}}}}\n")
        })
        .collect();
    let actions = if actions.is_empty() {
        String::new()
    } else {
        format!("actions:\n{actions}")
    };
    compiled(&format!(
        "format: protocol/1\nprotocol: {{id: '{id}', revision: 1}}\nartifacts: {{a: {{}}}}\n{actions}"
    ))
}

/// D206: protocols whose openings form a cycle are refused, the cycle named from its least id; a
/// chain of openings is accepted.
#[test]
fn a_set_of_protocols_that_open_each_other_is_refused_and_a_chain_accepted() {
    let audit = compiled(&fixture());
    let cycle =
        check_openings([&audit, &opener("software.change", &["audit"])]).expect_err("a cycle");
    assert_eq!(cycle.code(), "case-open-cycle");
    assert_eq!(
        cycle.to_string(),
        "protocols open cases under each other in a cycle: audit -> software.change -> audit"
    );
    let chain = [
        audit,
        opener("software.change", &["review"]),
        opener("review", &[]),
    ];
    assert_eq!(check_openings(&chain), Ok(()));
}

/// `canon check` over the fixture: an action that opens a child case produces `case_outcome`,
/// so nothing reads evidence no action produces. Without the opening, the kind is unproduced.
/// The kind's one dimension has four classes (`accepted`, `declined`, `open`, another), so 16
/// values, and a history dimension of 17, beside the capability's 3.
#[test]
fn check_takes_an_opening_action_as_producing_case_outcome() {
    let report = check(&compiled(&fixture()), None).expect("checks");
    assert_eq!(
        report.to_string(),
        "checked: protocol `audit` revision 1: 816 states, 0 properties, 0 findings\n"
    );
    let unopened = fixture().replacen(
        "    case_open:\n      protocol: software.change\n      subject: findings\n",
        "",
        1,
    );
    let report = check(&compiled(&unopened), None)
        .expect("checks")
        .to_string();
    assert!(
        report.contains("unproduced-evidence: claim `findings.resolved`"),
        "{report}"
    );
}

/// A protocol in which an order form reads `case_outcome` beside a `children` form is refused:
/// the space does not cover the two together. Either alone is checked.
#[test]
fn check_refuses_case_outcome_read_by_children_and_an_order_form() {
    let ordered = fixture().replacen(
        "outcomes:\n",
        "  first_accepted:\n    precondition: {first: {kind: case_outcome, result: accepted}}\n\noutcomes:\n",
        1,
    );
    let refusal = check(&compiled(&ordered), None).expect_err("refused");
    assert_eq!(refusal.code(), "ordered-child-outcomes");
    assert_eq!(
        refusal.to_string(),
        "protocol `audit` revision 1 reads evidence kind `case_outcome` with a `children` form and \
         with an order form, which canon check does not cover together"
    );
    let order_alone = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {findings: {}}\nevidence_kinds: {case_outcome: {}}\n\
        actions: {a: {precondition: {first: {kind: case_outcome, result: accepted}}}}\n";
    assert!(check(&compiled(order_alone), None).is_ok());
}

/// xorshift64*, a fixed seed: the same histories on every run.
struct Rng(u64);

impl Rng {
    fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        (self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) % n as u64) as usize
    }
}

/// `canon check` reaches every combination of claim values a history gives: two `children` forms
/// with overlapping lists, beside evidence matches over the same records, about two artifacts.
/// Each combination becomes an outcome requiring exactly it; the check must not call one
/// unreachable that a history makes legitimate.
#[test]
fn check_reaches_every_combination_of_children_forms_and_matches() {
    let head = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {findings: {}, report: {}}\nevidence_kinds: {case_outcome: {}}\n\
        actions: {open: {case_open: {protocol: q, subject: findings}}}\n\
        claims:\n\
        \x20\x20resolved: {true_when: {children: {in: [accepted, declined]}}}\n\
        \x20\x20accepted: {true_when: {children: {in: [accepted]}}}\n\
        \x20\x20any_declined: {true_when: {evidence: {kind: case_outcome, result: declined}}}\n\
        \x20\x20any_open: {true_when: {evidence: {kind: case_outcome, result: open}}}\n";
    let ir = compiled(head);
    let case = case("p", &["findings", "report"], "r1");
    let results = ["accepted", "declined", "open", "abandoned"];
    let mut rng = Rng(0x00C0_FFEE);
    let mut found: BTreeMap<Vec<(String, Truth)>, Vec<EvidenceRecord>> = BTreeMap::new();
    for _ in 0..4000 {
        let records: Vec<EvidenceRecord> = (0..rng.below(5))
            .map(|n| {
                record(
                    &format!("h{n}"),
                    "case_outcome",
                    Some(&format!("c{n}")),
                    Some(results[rng.below(results.len())]),
                    ["findings", "report"][rng.below(2)],
                    ["r1", "r0"][rng.below(2)],
                )
            })
            .collect();
        let decision = eval::evaluate(&ir, &case, &records).expect("decides");
        let values: Vec<(String, Truth)> = decision
            .claims
            .iter()
            .map(|(id, claim)| (id.as_str().to_owned(), claim.value))
            .collect();
        found.entry(values).or_insert(records);
    }
    assert!(found.len() >= 12, "{} combinations", found.len());
    let mut outcomes = String::from("outcomes:\n");
    for (index, combination) in found.keys().enumerate() {
        let tests: Vec<String> = combination
            .iter()
            .map(|(claim, value)| format!("{{claim: {claim}, is: {value}}}"))
            .collect();
        outcomes.push_str(&format!(
            "\x20\x20o{index}: {{requires: {{all: [{}]}}}}\n",
            tests.join(", ")
        ));
    }
    let protocol = compiled(&format!("{head}{outcomes}"));
    let report = check(&protocol, None).expect("checks").to_string();
    for (index, (combination, history)) in found.iter().enumerate() {
        let status = &eval::evaluate(&protocol, &case, history)
            .expect("decides")
            .outcomes
            .expect("outcomes")[format!("o{index}")]["status"];
        assert_eq!(status, "legitimate", "o{index} on its own history");
        assert!(
            !report.contains(&format!("unreachable-outcome: outcome `o{index}` ")),
            "{combination:?} is legitimate on {history:?}, but the check calls it unreachable"
        );
    }
}

/// The explanation gives a claim over the child cases its form's value, as a blocked action names
/// it, and lists no record under it.
#[test]
fn the_explanation_names_the_form_with_its_value() {
    let explanation = eval::evaluate(
        &compiled(&fixture()),
        &case("audit", &["findings"], "f1"),
        &[
            outcome("o1", "CHG-1", "accepted"),
            outcome("o2", "CHG-2", "open"),
        ],
    )
    .expect("decides")
    .explanation
    .expect("an explanation");
    assert_eq!(
        explanation["claims"]["findings.resolved"],
        json!({
            "because": [{"children": ["accepted", "declined"], "value": "unknown"}],
            "value": "unknown",
        })
    );
    assert_eq!(
        explanation["actions"]["close"]["because"],
        json!([{"children": ["accepted", "declined"], "value": "unknown"}])
    );
}

/// The semantic diff compares a `children` form structurally, and a changed opening is
/// `BREAKING`.
#[test]
fn the_diff_takes_a_changed_form_or_opening_as_breaking() {
    let from = compiled(&fixture());
    let later = |text: String| compiled(&text.replacen("revision: 1", "revision: 2", 1));
    let widened = later(fixture().replacen(
        "        in: [declined, accepted]",
        "        in: [declined, accepted, inconclusive]",
        1,
    ));
    let lines: Vec<String> = diff(&from, &widened)
        .expect("diffs")
        .changes
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(
        lines[0].starts_with("BREAKING: claim `findings.resolved` "),
        "{lines:?}"
    );
    let reopened = later(fixture().replacen("protocol: software.change", "protocol: review", 1));
    let changes = diff(&from, &reopened).expect("diffs").changes;
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].category, Category::Breaking);
    assert_eq!(
        changes[0].to_string(),
        "BREAKING: action `open_change` case it opens changed from protocol `software.change` about `findings` to protocol `review` about `findings`"
    );
    let closed = later(fixture().replacen(
        "    case_open:\n      protocol: software.change\n      subject: findings\n",
        "",
        1,
    ));
    assert_eq!(
        diff(&from, &closed).expect("diffs").changes[0].to_string(),
        "BREAKING: action `open_change` case it opens changed from protocol `software.change` about `findings` to none"
    );
}
