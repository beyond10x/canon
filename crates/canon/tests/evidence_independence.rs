//! story:evidence-independence beyond CANON-INDEPENDENCE-001: the validator's refusals of an
//! `independent_on`, the IR of a match that demands independence and its round trip through
//! `read_ir`, the refusal of a record whose producer is not an identifier, the explanation of a
//! record that fails a demand, and `canon diff` of a changed demand under `not`, beside other
//! changes and with no change.

use b10x_canon::diff::{self, Category};
use b10x_canon::eval;
use b10x_canon::ir::{self, Ir};
use b10x_canon::model::{self, EvidenceRecord};
use serde_json::{Value, json};

const HEADER: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {change: {}}\n\
    independence_dimensions: {different_principal: {}, different_agent_run: {}}\n\
    evidence_kinds: {review: {}}\n";

fn protocol(claims: &str) -> String {
    format!("{HEADER}claims:\n{claims}")
}

fn problems(text: &str) -> Vec<(String, String)> {
    let parsed = model::parse(text).expect("parses");
    match ir::compile(&parsed) {
        Ok(_) => Vec::new(),
        Err(problems) => problems
            .iter()
            .map(|problem| (problem.code().to_owned(), problem.to_string()))
            .collect(),
    }
}

fn compiled(text: &str) -> Ir {
    ir::compile(&model::parse(text).expect("parses")).expect("compiles")
}

/// An empty `independent_on`, a dimension listed twice and an undeclared dimension are each
/// refused, naming the declaration, and a malformed or repeated dimension declaration is refused
/// like every other identifier.
#[test]
fn the_validator_refuses_a_malformed_demand() {
    for (independent_on, expected) in [
        (
            "[]",
            vec![(
                "empty-independent-on",
                "claim `c` demands independence on no dimension of evidence kind `review`",
            )],
        ),
        (
            "[different_principal, different_principal]",
            vec![(
                "duplicate-independent-on",
                "claim `c` demands independence on dimension `different_principal` more than once of evidence kind `review`",
            )],
        ),
        (
            "[different_team, different_principal, different_team]",
            vec![
                (
                    "duplicate-independent-on",
                    "claim `c` demands independence on dimension `different_team` more than once of evidence kind `review`",
                ),
                (
                    "undeclared-dimension",
                    "claim `c` references independence dimension `different_team`, which is not declared",
                ),
            ],
        ),
    ] {
        let text = protocol(&format!(
            "  c: {{true_when: {{evidence: {{kind: review, independent_on: {independent_on}}}}}}}\n"
        ));
        let expected: Vec<(String, String)> = expected
            .into_iter()
            .map(|(code, message)| (code.to_owned(), message.to_owned()))
            .collect();
        assert_eq!(problems(&text), expected, "{independent_on}");
    }
    let malformed = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        independence_dimensions: {'a b': {}}\n";
    assert_eq!(
        problems(malformed),
        [(
            "invalid-identifier".to_owned(),
            "independence dimension identifier `a b` is empty or contains whitespace or a control character"
                .to_owned()
        )]
    );
}

/// `independent_on` compiles to its dimensions in identifier order, written only when named; a
/// protocol that declares no dimension has no `independence_dimensions` key; and the IR reads back
/// to the same bytes.
#[test]
fn the_ir_writes_the_demand_sorted_and_reads_it_back() {
    let ir = compiled(&protocol(
        "  c: {true_when: {evidence: {kind: review, independent_on: [different_principal, different_agent_run]}}}\n\
         \x20\x20d: {true_when: {evidence: {kind: review}}}\n",
    ));
    let text = ir.canonical_json();
    let value: Value = serde_json::from_str(&text).expect("JSON");
    assert_eq!(
        value["claims"]["c"]["true_when"]["evidence"]["independent_on"],
        json!(["different_agent_run", "different_principal"])
    );
    assert_eq!(
        value["claims"]["d"]["true_when"]["evidence"].get("independent_on"),
        None
    );
    assert_eq!(
        value["independence_dimensions"],
        json!({"different_agent_run": {"description": null}, "different_principal": {"description": null}})
    );
    assert_eq!(eval::read_ir(&text).expect("reads back"), ir);

    let plain = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nevidence_kinds: {review: {}}\n",
    );
    let value: Value = serde_json::from_str(&plain.canonical_json()).expect("JSON");
    assert_eq!(value.get("independence_dimensions"), None);
}

/// Two matches that differ only in their demand are two members of an `all`, the one demanding
/// nothing first.
#[test]
fn matches_differing_only_in_their_demand_are_two_members() {
    let ir = compiled(&protocol(
        "  c: {true_when: {all: [{evidence: {kind: review, independent_on: [different_principal]}}, {evidence: {kind: review}}]}}\n",
    ));
    let value: Value = serde_json::from_str(&ir.canonical_json()).expect("JSON");
    assert_eq!(
        value["claims"]["c"]["true_when"]["all"],
        json!([
            {"evidence": {"kind": "review", "result": null}},
            {"evidence": {"independent_on": ["different_principal"], "kind": "review", "result": null}}
        ])
    );
}

fn record(id: &str, result: &str, producer: Option<&str>) -> EvidenceRecord {
    let producer = producer.map_or(String::new(), |producer| format!("producer: {producer}\n"));
    eval::read_evidence(&format!(
        "format: canon-evidence/1\nid: {id}\nkind: review\nresult: {result}\nsubject: change\n\
         subject_revision: r1\n{producer}"
    ))
    .expect("a canon-evidence/1 record")
}

const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {change: {revision: r1}}\n\
    independence: {reviewer: [different_principal], author: []}\n";

/// A producer that is not an identifier is refused, naming the record.
#[test]
fn a_producer_that_is_not_an_identifier_is_refused() {
    let ir = compiled(&protocol(
        "  c: {true_when: {evidence: {kind: review, independent_on: [different_principal]}}}\n",
    ));
    let case = eval::read_case(CASE).expect("a case");
    let refusal = eval::evaluate(&ir, &case, &[record("v1", "approved", Some("'a b'"))])
        .expect_err("refused");
    assert_eq!(refusal.code(), "invalid-identifier");
    assert_eq!(
        refusal.to_string(),
        "evidence `v1` producer identifier `a b` is empty or contains whitespace or a control character"
    );
}

/// The explanation lists a record that fails the demand as excluded with the reason
/// not_independent under the claim, and under the blocked action's evidence reason, which names
/// the demand; a claim whose other match accepts the record lists it as applied; and the case's
/// independence map is recorded in computed_from.
#[test]
fn the_explanation_names_a_record_that_fails_the_demand() {
    let text = format!(
        "{}actions:\n  merge: {{precondition: {{evidence: {{kind: review, result: approved, independent_on: [different_principal]}}}}}}\n",
        protocol(
            "  strict: {true_when: {evidence: {kind: review, result: approved, independent_on: [different_principal]}}}\n\
             \x20\x20either: {true_when: {any: [{evidence: {kind: review, result: approved, independent_on: [different_principal]}}, {evidence: {kind: review, result: approved}}]}}\n\
             \x20\x20neither: {true_when: {not: {claim: either}}}\n",
        )
    );
    let ir = compiled(&text);
    let case = eval::read_case(CASE).expect("a case");
    let decision =
        eval::evaluate(&ir, &case, &[record("v1", "approved", Some("author"))]).expect("decides");
    let rendered: Value = serde_json::from_str(&eval::render(&decision)).expect("JSON");
    let explanation = &rendered["explanation"];
    assert_eq!(
        explanation["claims"]["strict"],
        json!({"because": [{"evidence": "v1", "reason": "not_independent", "status": "excluded"}], "value": "unknown"})
    );
    assert_eq!(rendered["claims"]["either"], json!({"value": "true"}));
    assert_eq!(
        explanation["claims"]["either"]["because"],
        json!([{"evidence": "v1", "status": "applied"}])
    );
    assert_eq!(
        explanation["actions"]["merge"],
        json!({"because": [
            {"evidence": "review", "independent_on": ["different_principal"], "present": false},
            {"evidence": "v1", "reason": "not_independent", "status": "excluded"}
        ], "status": "blocked"})
    );
    assert_eq!(
        explanation["computed_from"]["case"]["independence"],
        json!({"author": [], "reviewer": ["different_principal"]})
    );
}

fn changes(from: &str, to: &str) -> Vec<(Category, String)> {
    let to = to.replace("revision: 1}", "revision: 2}");
    diff::diff(&compiled(from), &compiled(&to))
        .expect("one protocol")
        .changes
        .into_iter()
        .map(|change| (change.category, change.what))
        .collect()
}

/// A demand added to a positive match tightens and one removed relaxes; one added or removed under
/// `not`, or in a claim read in reverse, is `BREAKING`, its line saying what the dimensions did; a
/// demand changed along with another member is `BREAKING`; one dimension
/// swapped for another is `BREAKING`; the same demand in both is no change; and a dimension
/// declared is `EXPANSION`, one removed `BREAKING`.
#[test]
fn diff_classifies_a_changed_demand_by_its_polarity() {
    let plain = "{evidence: {kind: review}}";
    let one = "{evidence: {kind: review, independent_on: [different_principal]}}";
    let other = "{evidence: {kind: review, independent_on: [different_agent_run]}}";
    let claim = |predicate: &str| protocol(&format!("  c: {{true_when: {predicate}}}\n"));
    let negated = |predicate: &str| claim(&format!("{{not: {predicate}}}"));
    let read_in_reverse = |predicate: &str| {
        format!(
            "{}actions:\n  a: {{precondition: {{claim: c, is: false}}}}\n",
            claim(predicate)
        )
    };
    use Category::{Breaking, Relaxation, Tightening};
    for (from, to, expected) in [
        (
            claim(plain),
            claim(one),
            vec![(Tightening, "predicate demands more independence")],
        ),
        (
            claim(one),
            claim(plain),
            vec![(Relaxation, "predicate demands less independence")],
        ),
        (
            negated(plain),
            negated(one),
            vec![(Breaking, "predicate demands more independence")],
        ),
        (
            negated(one),
            negated(plain),
            vec![(Breaking, "predicate demands less independence")],
        ),
        (
            read_in_reverse(plain),
            read_in_reverse(one),
            vec![(
                Breaking,
                "predicate demands more independence, read in reverse",
            )],
        ),
        (
            read_in_reverse(one),
            read_in_reverse(plain),
            vec![(
                Breaking,
                "predicate demands less independence, read in reverse",
            )],
        ),
        (
            claim(&format!("{{all: [{plain}]}}")),
            claim(&format!(
                "{{all: [{one}, {{evidence: {{kind: review, result: ok}}}}]}}"
            )),
            vec![(Breaking, "predicate changed")],
        ),
        (
            claim(one),
            claim(other),
            vec![(Breaking, "predicate changed")],
        ),
        (claim(one), claim(one), vec![]),
    ] {
        let found: Vec<(Category, String)> = changes(&from, &to)
            .into_iter()
            .filter(|(_, what)| what.starts_with("predicate"))
            .collect();
        let expected: Vec<(Category, String)> = expected
            .into_iter()
            .map(|(category, what)| (category, what.to_owned()))
            .collect();
        assert_eq!(found, expected, "{from}\n->\n{to}");
    }

    let fewer = HEADER.replace(", different_agent_run: {}", "");
    assert_eq!(
        changes(&fewer, HEADER),
        [(Category::Expansion, "added".to_owned())]
    );
    assert_eq!(
        changes(HEADER, &fewer),
        [(Category::Breaking, "removed".to_owned())]
    );
}

/// A claim read in reverse is the `not` of the adversary's case one claim away: a demand added to
/// `{evidence: review, result: approved}` turns the author's rejection from deciding the claim
/// `false` to deciding nothing, so `{claim: c, is: false}` goes from admitting `merge` to blocking
/// it. Less work is legitimate and the report must not say `RELAXATION`.
#[test]
fn a_demand_added_to_a_claim_read_in_reverse_that_blocks_an_admitted_case_is_breaking() {
    let source = |predicate: &str| {
        protocol(&format!(
            "  c: {{true_when: {predicate}}}\n\
             actions:\n  merge: {{precondition: {{claim: c, is: false}}}}\n"
        ))
    };
    let from = source("{evidence: {kind: review, result: approved}}");
    let to = source(
        "{evidence: {kind: review, result: approved, independent_on: [different_principal]}}",
    )
    .replace("revision: 1}", "revision: 2}");
    let merge = |text: &str| {
        let case = eval::read_case(
            "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {change: {revision: r1}}\n\
             independence: {author: []}\n",
        )
        .expect("a case");
        let record: EvidenceRecord = eval::read_evidence(
            "format: canon-evidence/1\nid: v1\nkind: review\nresult: rejected\nsubject: change\n\
             subject_revision: r1\nproducer: author\n",
        )
        .expect("a record");
        let decision = eval::evaluate(&compiled(text), &case, &[record]).expect("decides");
        let rendered: Value = serde_json::from_str(&eval::render(&decision)).expect("JSON");
        rendered["actions"]["merge"]["status"]
            .as_str()
            .expect("a status")
            .to_owned()
    };
    assert_eq!(merge(&from), "admissible", "before the change");
    assert_eq!(merge(&to), "blocked", "after the change");
    assert_eq!(
        changes(&from, &to),
        [(
            Category::Breaking,
            "predicate demands more independence, read in reverse".to_owned()
        )]
    );
}
