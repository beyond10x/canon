//! Adversary cases for story:obligations (wave 2026-10-04-w7, pass 2): `discharged_when` through
//! `read_ir`, hand-written IR, identifiers that need escaping, determinism, termination, nesting
//! to the IR bound, and the obligation statuses against the claim values the decision reports.

use std::collections::BTreeMap;
use std::path::PathBuf;

use b10x_canon::model::{ClaimId, ClaimTest, EvidenceKindId, ObligationId, Predicate, Truth};
use b10x_canon::{eval, ir, model};

const CASE: &str = "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r1}}\n";

const HEADER: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
    evidence_kinds: {k: {}, j: {}}\n\
    claims:\n  c: {true_when: {evidence: {kind: k}}}\n  d: {true_when: {evidence: {kind: j, result: pass}}}\n";

fn compiled(obligations: &str) -> ir::Ir {
    let source = format!(
        "{HEADER}outcomes: {{done: {{requires: {{claim: c}}}}}}\nobligations:\n{obligations}"
    );
    ir::compile(&model::parse(&source).expect("parses")).expect("compiles")
}

fn record(id: &str, kind: &str, result: Option<&str>, revision: &str) -> model::EvidenceRecord {
    let result = result.map_or(String::new(), |r| format!("result: {r}\n"));
    eval::read_evidence(&format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\n{result}subject: a\nsubject_revision: {revision}\n"
    ))
    .expect("evidence reads")
}

fn case(text: &str) -> model::Case {
    eval::read_case(text).expect("case reads")
}

/// `(id, status)` of every entry of the rendered decision, parsed back as JSON.
fn rendered_entries(decision: &model::Decision) -> Vec<(String, String)> {
    let text = eval::render(decision);
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("the decision is JSON");
    parsed["obligations"]
        .as_array()
        .expect("obligations array")
        .iter()
        .map(|entry| {
            (
                entry["id"].as_str().expect("id").to_owned(),
                entry["status"].as_str().expect("status").to_owned(),
            )
        })
        .collect()
}

fn pairs(rows: &[(&str, &str)]) -> Vec<(String, String)> {
    rows.iter()
        .map(|(id, status)| ((*id).to_owned(), (*status).to_owned()))
        .collect()
}

const EVERY_FORM: &str = concat!(
    "  o1.claim: {discharged_when: {claim: c}}\n",
    "  o2.false: {discharged_when: {claim: c, is: false}}\n",
    "  o3.unknown: {discharged_when: {claim: c, is: unknown}}\n",
    "  o4.not: {discharged_when: {not: {claim: c}}}\n",
    "  o5.all: {discharged_when: {all: [{claim: c}, {claim: d, is: false}]}}\n",
    "  o6.any-empty: {discharged_when: {any: []}}\n",
    "  o7.all-empty: {discharged_when: {all: []}}\n",
    "  o8.any: {discharged_when: {any: [{claim: d}, {claim: c, is: unknown}]}}\n",
);

/// The IR `canon compile` prints reads back equal, every discharge predicate included, and the
/// read IR decides each obligation as the compiled one does, to the literal statuses the three-
/// valued rules give.
#[test]
fn every_discharge_predicate_survives_read_ir_and_decides_the_same() {
    let compiled = compiled(EVERY_FORM);
    let read = eval::read_ir(&compiled.canonical_json()).expect("reads back");
    assert_eq!(read, compiled);
    let case = case(CASE);
    let sets = [
        (
            vec![],
            pairs(&[
                ("o1.claim", "open"),
                ("o2.false", "open"),
                ("o3.unknown", "discharged"),
                ("o4.not", "open"),
                ("o5.all", "open"),
                ("o6.any-empty", "open"),
                ("o7.all-empty", "discharged"),
                ("o8.any", "discharged"),
            ]),
        ),
        (
            vec![
                record("e1", "k", None, "r1"),
                record("e2", "j", Some("fail"), "r1"),
            ],
            pairs(&[
                ("o1.claim", "discharged"),
                ("o2.false", "open"),
                ("o3.unknown", "open"),
                ("o4.not", "open"),
                ("o5.all", "discharged"),
                ("o6.any-empty", "open"),
                ("o7.all-empty", "discharged"),
                ("o8.any", "open"),
            ]),
        ),
        (
            vec![
                record("e1", "k", None, "r1"),
                record("e2", "j", Some("pass"), "r1"),
            ],
            pairs(&[
                ("o1.claim", "discharged"),
                ("o2.false", "open"),
                ("o3.unknown", "open"),
                ("o4.not", "open"),
                ("o5.all", "open"),
                ("o6.any-empty", "open"),
                ("o7.all-empty", "discharged"),
                ("o8.any", "discharged"),
            ]),
        ),
    ];
    for (evidence, expected) in sets {
        let from_compiled = eval::evaluate(&compiled, &case, &evidence).expect("evaluates");
        let from_read = eval::evaluate(&read, &case, &evidence).expect("evaluates");
        assert_eq!(eval::render(&from_read), eval::render(&from_compiled));
        assert_eq!(rendered_entries(&from_read), expected);
    }
}

/// A hand-written `canon-ir/1` whose discharge predicate names an undeclared claim, or tests
/// evidence, is refused by `read_ir`; it never reaches the obligations section.
#[test]
fn read_ir_refuses_a_hand_written_discharge_predicate_the_validator_refuses() {
    let mut ghost = compiled("  o: {discharged_when: {claim: c}}\n");
    ghost
        .obligations
        .get_mut(&ObligationId::new("o"))
        .expect("o")
        .discharged_when = Predicate::Claim(ClaimTest {
        claim: ClaimId::new("ghost"),
        is: Truth::Unknown,
    });
    let refusal = eval::read_ir(&ghost.canonical_json()).expect_err("undeclared claim");
    assert_eq!(refusal.code(), "malformed-input");
    assert!(
        refusal
            .to_string()
            .contains("undeclared-claim: obligation `o` references claim `ghost`"),
        "{refusal}"
    );

    let mut evidence = compiled("  o: {discharged_when: {claim: c}}\n");
    evidence
        .obligations
        .get_mut(&ObligationId::new("o"))
        .expect("o")
        .discharged_when = Predicate::Not(Box::new(Predicate::Evidence(model::EvidenceMatch {
        kind: EvidenceKindId::new("k"),
        result: None,
    })));
    let refusal = eval::read_ir(&evidence.canonical_json()).expect_err("evidence in discharge");
    assert!(
        refusal.to_string().contains("evidence-in-discharge"),
        "{refusal}"
    );
}

/// In an IR a caller builds, an undeclared claim is `unknown` to the obligations section exactly
/// as it is to a claim: the two evaluators agree.
#[test]
fn an_undeclared_claim_is_unknown_to_obligations_as_to_claims() {
    let mut ir =
        compiled("  o.u: {discharged_when: {claim: c}}\n  o.t: {discharged_when: {claim: c}}\n");
    let ghost = |is| {
        Predicate::Claim(ClaimTest {
            claim: ClaimId::new("ghost"),
            is,
        })
    };
    ir.obligations
        .get_mut(&ObligationId::new("o.u"))
        .unwrap()
        .discharged_when = ghost(Truth::Unknown);
    ir.obligations
        .get_mut(&ObligationId::new("o.t"))
        .unwrap()
        .discharged_when = ghost(Truth::True);
    ir.claims.get_mut(&ClaimId::new("d")).unwrap().true_when = ghost(Truth::Unknown);
    let decision = eval::evaluate(&ir, &case(CASE), &[]).expect("evaluates");
    let d = decision.claims.get(&ClaimId::new("d")).expect("d").value;
    assert_eq!(d, Truth::True);
    assert_eq!(
        rendered_entries(&decision),
        pairs(&[("o.t", "open"), ("o.u", "discharged")])
    );
}

/// Identifiers that need escaping in JSON: the rendered decision parses as JSON and gives each id
/// back unchanged, in code-point order. Quote and backslash compile; the control and separator
/// characters only a caller can put in an IR.
#[test]
fn obligation_ids_that_need_escaping_render_as_json_and_read_back() {
    let mut ir = compiled(
        "  'q\"uote': {discharged_when: {claim: c}}\n  'back\\slash': {discharged_when: {claim: c, is: unknown}}\n  '}],': {discharged_when: {claim: c}}\n",
    );
    for id in ["nl\nline", "nul\u{0}", "ls\u{2028}", "del\u{7f}", "tab\t"] {
        ir.obligations.insert(
            ObligationId::new(id),
            ir::Obligation {
                description: None,
                discharged_when: Predicate::Claim(ClaimTest {
                    claim: ClaimId::new("c"),
                    is: Truth::Unknown,
                }),
            },
        );
    }
    let decision = eval::evaluate(&ir, &case(CASE), &[]).expect("evaluates");
    let mut expected: Vec<(String, String)> = vec![
        ("q\"uote".to_owned(), "open".to_owned()),
        ("back\\slash".to_owned(), "discharged".to_owned()),
        ("}],".to_owned(), "open".to_owned()),
    ];
    for id in ["nl\nline", "nul\u{0}", "ls\u{2028}", "del\u{7f}", "tab\t"] {
        expected.push((id.to_owned(), "discharged".to_owned()));
    }
    expected.sort_by(|a, b| a.0.chars().cmp(b.0.chars()));
    assert_eq!(rendered_entries(&decision), expected);
    let text = eval::render(&decision);
    assert!(text.contains("\"id\": \"nl\\nline\""), "{text}");
    assert!(text.contains("\"id\": \"nul\\u0000\""), "{text}");
    assert!(text.contains("\"id\": \"q\\\"uote\""), "{text}");
}

/// The same inputs give the same bytes: repeated runs, other threads, evidence in any order.
#[test]
fn the_obligations_section_is_the_same_bytes_whatever_the_run_thread_or_evidence_order() {
    let ir = compiled(EVERY_FORM);
    let case = case(CASE);
    let evidence = vec![
        record("e3", "j", Some("fail"), "r1"),
        record("e1", "k", None, "r1"),
        record("e2", "j", Some("pass"), "r1"),
    ];
    let first = eval::render(&eval::evaluate(&ir, &case, &evidence).expect("evaluates"));
    for rotation in 0..evidence.len() {
        let mut turned = evidence.clone();
        turned.rotate_left(rotation);
        turned.reverse();
        let again = std::thread::scope(|scope| {
            scope
                .spawn(|| eval::render(&eval::evaluate(&ir, &case, &turned).expect("evaluates")))
                .join()
                .expect("thread")
        });
        assert_eq!(again, first);
    }
}

/// A recorded termination does not change the obligations section: the outcomes stub accepts
/// it, and an open obligation stays open in a terminated case.
#[test]
fn a_recorded_termination_leaves_the_obligations_section_unchanged() {
    let manifest = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("cargo"));
    let fixture = manifest.join("../../fixtures/investigation/obligations.yaml");
    let text = std::fs::read_to_string(&fixture).expect("fixture reads");
    let ir = ir::compile(&model::parse(&text).expect("parses")).expect("compiles");
    let open = "format: canon-case/1\nid: INV-18\nprotocol: investigation\nartifacts: {explanation: {revision: r1}}\n";
    let ended = format!("{open}termination: supported\n");
    let observed = |id: &str, kind: &str, result: Option<&str>| {
        let result = result.map_or(String::new(), |r| format!("result: {r}\n"));
        eval::read_evidence(&format!(
            "format: canon-evidence/1\nid: {id}\nkind: {kind}\n{result}subject: explanation\nsubject_revision: r1\n"
        ))
        .expect("evidence")
    };
    let sets = [
        (vec![], "open"),
        (
            vec![
                observed("o", "supporting_observation", None),
                observed("f", "falsification_attempt", Some("refuted")),
            ],
            "open",
        ),
        (
            vec![
                observed("o", "supporting_observation", None),
                observed("f", "falsification_attempt", Some("survived")),
            ],
            "discharged",
        ),
    ];
    for (evidence, status) in sets {
        let a = eval::evaluate(&ir, &case(open), &evidence).expect("open case");
        let b = eval::evaluate(&ir, &case(&ended), &evidence).expect("terminated case");
        let expected = pairs(&[("establish.explanation", status)]);
        assert_eq!(rendered_entries(&a), expected);
        assert_eq!(rendered_entries(&b), expected);
    }
}

/// Each obligation's status is its discharge predicate over the claim values the decision itself
/// reports (after every exclusion stage), whatever the evidence: a record bound to another
/// revision, conflicting records, a result nobody tests. Holds once the exclusion stages exclude.
#[test]
fn every_status_follows_the_claim_values_the_decision_reports() {
    let ir = compiled(EVERY_FORM);
    let case = case(CASE);
    let pool = [
        record("e1", "k", None, "r0"),
        record("e2", "k", None, "r1"),
        record("e3", "j", Some("pass"), "r1"),
        record("e4", "j", Some("fail"), "r0"),
        record("e5", "j", Some("other"), "r1"),
    ];
    let truth = |t: Truth| match t {
        Truth::True => 2u8,
        Truth::Unknown => 1,
        Truth::False => 0,
    };
    for mask in 0u32..(1 << pool.len()) {
        let evidence: Vec<_> = pool
            .iter()
            .enumerate()
            .filter(|(n, _)| mask & (1 << n) != 0)
            .map(|(_, r)| r.clone())
            .collect();
        let decision = eval::evaluate(&ir, &case, &evidence).expect("evaluates");
        let value = |id: &str| truth(decision.claims.get(&ClaimId::new(id)).expect(id).value);
        let (c, d) = (value("c"), value("d"));
        let is = |v: u8, t: Truth| match t {
            Truth::True => v,
            Truth::False => 2 - v,
            Truth::Unknown => {
                if v == 1 {
                    2
                } else {
                    0
                }
            }
        };
        let expected: BTreeMap<&str, u8> = BTreeMap::from([
            ("o1.claim", c),
            ("o2.false", is(c, Truth::False)),
            ("o3.unknown", is(c, Truth::Unknown)),
            ("o4.not", 2 - c),
            ("o5.all", c.min(is(d, Truth::False))),
            ("o6.any-empty", 0),
            ("o7.all-empty", 2),
            ("o8.any", d.max(is(c, Truth::Unknown))),
        ]);
        let expected: Vec<(String, String)> = expected
            .into_iter()
            .map(|(id, v)| {
                (
                    id.to_owned(),
                    if v == 2 { "discharged" } else { "open" }.to_owned(),
                )
            })
            .collect();
        assert_eq!(
            rendered_entries(&decision),
            expected,
            "evidence mask {mask:05b}"
        );
    }
}

/// Bracket nesting outside strings, as `read_ir` counts it.
fn depth(text: &str) -> usize {
    let (mut now, mut deepest, mut in_string, mut escaped) = (0usize, 0usize, false, false);
    for byte in text.bytes() {
        if in_string {
            match byte {
                _ if escaped => escaped = false,
                b'\\' => escaped = true,
                b'"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'[' | b'{' => {
                now += 1;
                deepest = deepest.max(now);
            }
            b']' | b'}' => now = now.saturating_sub(1),
            _ => {}
        }
    }
    deepest
}

/// A discharge predicate nested to `MAX_IR_DEPTH` reads back and evaluates from this test thread,
/// and its status is the one the parity of the `not`s gives.
#[test]
fn a_discharge_predicate_nested_to_the_ir_bound_reads_and_evaluates() {
    let build = |nots: usize| {
        std::thread::Builder::new()
            .stack_size(256 << 20)
            .spawn(move || {
                let mut ir = compiled("  o: {discharged_when: {claim: c}}\n");
                let mut predicate = Predicate::Claim(ClaimTest {
                    claim: ClaimId::new("c"),
                    is: Truth::True,
                });
                for _ in 0..nots {
                    predicate = Predicate::Not(Box::new(predicate));
                }
                ir.obligations
                    .get_mut(&ObligationId::new("o"))
                    .unwrap()
                    .discharged_when = predicate;
                ir.canonical_json()
            })
            .expect("thread")
            .join()
            .expect("built")
    };
    let base = depth(&build(0));
    let nots = eval::MAX_IR_DEPTH - base;
    let text = build(nots);
    assert_eq!(depth(&text), eval::MAX_IR_DEPTH);
    let read = eval::read_ir(&text).expect("reads at the bound");
    let decision =
        eval::evaluate(&read, &case(CASE), &[record("e1", "k", None, "r1")]).expect("evaluates");
    let expected = if nots.is_multiple_of(2) {
        "discharged"
    } else {
        "open"
    };
    assert_eq!(rendered_entries(&decision), pairs(&[("o", expected)]));
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || drop(read))
        .expect("thread")
        .join()
        .expect("dropped");
}
