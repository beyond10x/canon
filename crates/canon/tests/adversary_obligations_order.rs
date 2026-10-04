//! Adversary cases for story:obligations (wave 2026-10-04-w7, pass 1): the order and number of
//! entries in the `obligations` section, and a discharge predicate holding evidence in an IR a
//! caller builds.

use b10x_canon::model::{EvidenceKindId, EvidenceMatch, ObligationId, Predicate};
use b10x_canon::{eval, ir, model};

const CASE: &str = "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {}\n";

fn compiled(obligations: &str) -> ir::Ir {
    let source = format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nevidence_kinds: {{k: {{}}}}\n\
         claims:\n  c: {{true_when: {{evidence: {{kind: k}}}}}}\nobligations:\n{obligations}"
    );
    ir::compile(&model::parse(&source).expect("parses")).expect("compiles")
}

fn evidence(kind: &str) -> model::EvidenceRecord {
    eval::read_evidence(&format!(
        "format: canon-evidence/1\nid: e-{kind}\nkind: {kind}\nsubject: x\nsubject_revision: r1\n"
    ))
    .expect("evidence reads")
}

/// `(id, status)` of every entry, in the order written.
fn entries(ir: &ir::Ir, evidence: &[model::EvidenceRecord]) -> Vec<(String, String)> {
    let case = eval::read_case(CASE).expect("case reads");
    let decision = eval::evaluate(ir, &case, evidence).expect("evaluates");
    let section = decision.obligations.expect("section present");
    section
        .as_array()
        .expect("array")
        .iter()
        .map(|entry| {
            assert_eq!(entry.as_object().expect("object").len(), 2, "{entry}");
            (
                entry["id"].as_str().expect("id").to_owned(),
                entry["status"].as_str().expect("status").to_owned(),
            )
        })
        .collect()
}

/// "In identifier order": the order `canon-decision/1` sorts object keys in, by Unicode code
/// point (`eval/json.rs`), which is what `claims` uses too. Upper case before lower, a BMP
/// character above the surrogates (U+FF5E) before a supplementary one (U+1F600), unlike UTF-16
/// order.
#[test]
fn entries_are_in_code_point_order_whatever_the_declaration_order() {
    let ids = [
        "\u{1F600}",
        "a-b",
        "\u{FF5E}",
        "a.b",
        "é",
        "a",
        "Z",
        "A",
        "_",
    ];
    let declared: String = ids
        .iter()
        .map(|id| format!("  \"{id}\": {{discharged_when: {{claim: c}}}}\n"))
        .collect();
    let mut expected: Vec<String> = ids.iter().map(|id| (*id).to_owned()).collect();
    expected.sort_by(|a, b| a.chars().cmp(b.chars()));
    let ir = compiled(&declared);
    let got: Vec<String> = entries(&ir, &[]).into_iter().map(|(id, _)| id).collect();
    assert_eq!(got, expected);
    let rendered =
        eval::render(&eval::evaluate(&ir, &eval::read_case(CASE).unwrap(), &[]).unwrap());
    let positions: Vec<usize> = expected
        .iter()
        .map(|id| rendered.find(&format!("\"id\": \"{id}\"")).expect(id))
        .collect();
    assert!(
        positions.windows(2).all(|w| w[0] < w[1]),
        "rendered order:\n{rendered}"
    );
}

/// Many obligations: one entry each, none merged or dropped, each with its own status.
#[test]
fn every_one_of_many_obligations_has_one_entry() {
    let declared: String = (0..2000)
        .map(|n| {
            let when = if n % 2 == 0 {
                "{claim: c}"
            } else {
                "{claim: c, is: unknown}"
            };
            format!("  o{n:05}: {{discharged_when: {when}}}\n")
        })
        .collect();
    let ir = compiled(&declared);
    let got = entries(&ir, &[evidence("k")]);
    assert_eq!(got.len(), 2000);
    for (n, (id, status)) in got.iter().enumerate() {
        assert_eq!(id, &format!("o{n:05}"));
        let expected = if n % 2 == 0 { "discharged" } else { "open" };
        assert_eq!(status, expected, "{id}");
    }
}

/// obligations.rs: "an evidence match in an IR a caller builds is `unknown`, and leaves its
/// obligation open", even when a record of the kind is in the set (validate refuses one, so only
/// a caller-built IR has it).
#[test]
fn an_evidence_match_a_caller_puts_in_a_discharge_predicate_never_discharges() {
    let mut ir = compiled("  o: {discharged_when: {claim: c}}\n");
    ir.obligations
        .get_mut(&ObligationId::new("o"))
        .expect("declared")
        .discharged_when = Predicate::Evidence(EvidenceMatch {
        kind: EvidenceKindId::new("k"),
        result: None,
    });
    assert_eq!(
        entries(&ir, &[evidence("k")]),
        vec![("o".to_owned(), "open".to_owned())]
    );
}
