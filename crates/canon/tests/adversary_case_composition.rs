//! Adversary pass 1 on story:case-composition: cases the unit's own suite does not state.
//!
//! - `canon check` must reach every combination of claim values a real history gives, when a
//!   `children` form shares `case_outcome` with subject-bound and result-less evidence matches, and
//!   when a form or a match names the result `other` that the space uses for a record without one.
//! - The evaluation does not depend on the order the records are given in.
//! - `check_openings` (D206) refuses exactly the sets with a cycle, names a real cycle from the least
//!   protocol id on any cycle, and names the same one whatever order the set is given in.
//! - canon-ir/1 round-trips a `children` form whose outcomes are not ASCII, and refuses a
//!   hand-edited IR that opens a case under its own protocol.
//! - `canon diff` takes any change to the set of `children` forms as `BREAKING`, as its module docs
//!   say, even where implication alone would call it an expansion.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use b10x_canon::check::check;
use b10x_canon::diff::{Category, diff};
use b10x_canon::eval;
use b10x_canon::ir::{self, Ir, check_openings};
use b10x_canon::model::{self, EvidenceRecord, Truth};
use b10x_canon::validate::validate;
use serde_json::Value;

fn fixture() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/investigation/case-composition.yaml");
    std::fs::read_to_string(&path).expect("the case-composition fixture reads")
}

fn compiled(source: &str) -> Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

fn problems(source: &str) -> Vec<String> {
    match validate(&model::parse(source).expect("parses")) {
        Ok(()) => Vec::new(),
        Err(problems) => problems
            .iter()
            .map(|problem| format!("{}: {problem}", problem.code()))
            .collect(),
    }
}

/// A `case_outcome` record for child `child`, written as a consumer's YAML would.
fn outcome(id: &str, child: &str, result: &str, subject: &str, revision: &str) -> EvidenceRecord {
    eval::read_evidence(&format!(
        "format: canon-evidence/1\nid: '{id}'\nkind: case_outcome\nsubject: {subject}\n\
         subject_revision: {revision}\nchild_case: '{child}'\nresult: '{result}'\n"
    ))
    .expect("a record")
}

fn case(protocol: &str, artifacts: &[&str]) -> model::Case {
    let listed: Vec<String> = artifacts
        .iter()
        .map(|artifact| format!("{artifact}: {{revision: r1}}"))
        .collect();
    eval::read_case(&format!(
        "format: canon-case/1\nid: C-1\nprotocol: {protocol}\nartifacts: {{{}}}\n",
        listed.join(", ")
    ))
    .expect("a case")
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

/// Up to four `case_outcome` records, one per child, each with a result of `results`, about an
/// artifact of `subjects`, at `r1` (current) or `r0` (superseded).
fn history(rng: &mut Rng, results: &[&str], subjects: &[&str]) -> Vec<EvidenceRecord> {
    (0..rng.below(5))
        .map(|n| {
            outcome(
                &format!("h{n}"),
                &format!("c{n}"),
                results[rng.below(results.len())],
                subjects[rng.below(subjects.len())],
                ["r1", "r0"][rng.below(2)],
            )
        })
        .collect()
}

/// Every combination of claim values the histories of `results` about `subjects` give under
/// `head` becomes an outcome requiring exactly it; each such outcome is legitimate on the history
/// that gave it, so `canon check` must not call it unreachable. Returns the combinations found.
fn assert_every_combination_reachable(
    head: &str,
    results: &[&str],
    subjects: &[&str],
) -> Vec<Vec<(String, Truth)>> {
    let ir = compiled(head);
    let case = case("p", subjects);
    let mut rng = Rng(0x00AD_5EED);
    let mut found: BTreeMap<Vec<(String, Truth)>, Vec<EvidenceRecord>> = BTreeMap::new();
    for _ in 0..3000 {
        let records = history(&mut rng, results, subjects);
        let decision = eval::evaluate(&ir, &case, &records).expect("decides");
        let values: Vec<(String, Truth)> = decision
            .claims
            .iter()
            .map(|(id, claim)| (id.as_str().to_owned(), claim.value))
            .collect();
        found.entry(values).or_insert(records);
    }
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
    for (index, (combination, records)) in found.iter().enumerate() {
        let status = &eval::evaluate(&protocol, &case, records)
            .expect("decides")
            .outcomes
            .expect("outcomes")[format!("o{index}")]["status"];
        assert_eq!(status, "legitimate", "o{index} on its own history");
        assert!(
            !report.contains(&format!("unreachable-outcome: outcome `o{index}` ")),
            "{combination:?} is legitimate on {records:?}, but the check calls it unreachable:\n{report}"
        );
    }
    found.into_keys().collect()
}

const OPENS: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n";

/// A `children` form beside a subject-bound match and a match about any artifact, over the same
/// `case_outcome` records about two artifacts at two revisions.
#[test]
fn check_reaches_children_jointly_with_subject_bound_matches() {
    let head = format!(
        "{OPENS}artifacts: {{findings: {{}}, report: {{}}}}\n\
         evidence_kinds: {{case_outcome: {{}}}}\n\
         actions: {{open: {{case_open: {{protocol: q, subject: findings}}}}}}\n\
         claims:\n\
         \x20\x20done: {{true_when: {{children: {{in: [accepted]}}}}}}\n\
         \x20\x20acc_f: {{true_when: {{evidence: {{kind: case_outcome, subject: findings, result: accepted}}}}}}\n\
         \x20\x20open_any: {{true_when: {{evidence: {{kind: case_outcome, result: open}}}}}}\n"
    );
    let found = assert_every_combination_reachable(
        &head,
        &["accepted", "open", "declined"],
        &["findings", "report"],
    );
    assert!(found.len() >= 10, "{} combinations", found.len());
}

/// The space gives a `case_outcome` record of the class without a result an outcome no predicate
/// names, `other` primed past every named one. A `children` form that lists `other` must still be
/// `false` in some state: a child that ended outside the set.
#[test]
fn check_reaches_a_child_outside_a_set_that_lists_other() {
    let head = format!(
        "{OPENS}artifacts: {{findings: {{}}}}\n\
         evidence_kinds: {{case_outcome: {{}}}}\n\
         actions: {{open: {{case_open: {{protocol: q, subject: findings}}}}}}\n\
         claims:\n\
         \x20\x20c: {{true_when: {{children: {{in: [other]}}}}}}\n"
    );
    let found =
        assert_every_combination_reachable(&head, &["other", "open", "abandoned"], &["findings"]);
    assert_eq!(found.len(), 3, "{found:?}");
}

/// Likewise when an evidence match, not a `children` form, names the result `other`: a child that
/// ended outside the set without that result is a state of its own.
#[test]
fn check_reaches_a_child_outside_the_set_beside_a_match_on_other() {
    let head = format!(
        "{OPENS}artifacts: {{findings: {{}}}}\n\
         evidence_kinds: {{case_outcome: {{}}}}\n\
         actions: {{open: {{case_open: {{protocol: q, subject: findings}}}}}}\n\
         claims:\n\
         \x20\x20c: {{true_when: {{children: {{in: [accepted]}}}}}}\n\
         \x20\x20m: {{true_when: {{evidence: {{kind: case_outcome, result: other}}}}}}\n"
    );
    let found = assert_every_combination_reachable(
        &head,
        &["accepted", "other", "open", "abandoned"],
        &["findings"],
    );
    assert!(
        found.contains(&vec![
            ("c".to_owned(), Truth::False),
            ("m".to_owned(), Truth::False)
        ]),
        "{found:?}"
    );
}

/// A child still `open` while a match that names no result holds: the form is `unknown` with a
/// record present, which only an `open` child gives.
#[test]
fn check_reaches_an_open_child_beside_a_match_without_result() {
    let head = format!(
        "{OPENS}artifacts: {{findings: {{}}}}\n\
         evidence_kinds: {{case_outcome: {{}}}}\n\
         actions: {{open: {{case_open: {{protocol: q, subject: findings}}}}}}\n\
         claims:\n\
         \x20\x20c: {{true_when: {{children: {{in: [accepted]}}}}}}\n\
         \x20\x20any: {{true_when: {{evidence: {{kind: case_outcome}}}}}}\n"
    );
    let found = assert_every_combination_reachable(
        &head,
        &["accepted", "open", "abandoned"],
        &["findings"],
    );
    assert!(
        found.contains(&vec![
            ("any".to_owned(), Truth::True),
            ("c".to_owned(), Truth::Unknown)
        ]),
        "{found:?}"
    );
}

/// The same records in any order give the same decision, byte for byte, and the same refusal.
#[test]
fn the_evaluation_does_not_depend_on_record_order() {
    let ir = compiled(&fixture());
    let case = case("audit", &["findings"]);
    let mut rng = Rng(0x0BAD_C0DE);
    for _ in 0..300 {
        let mut records = history(
            &mut rng,
            &["accepted", "declined", "open", "abandoned"],
            &["findings"],
        );
        let first = eval::evaluate(&ir, &case, &records).map(|decision| eval::render(&decision));
        for turn in 0..records.len() {
            records.rotate_left(1);
            if turn % 2 == 1 {
                records.reverse();
            }
            let again =
                eval::evaluate(&ir, &case, &records).map(|decision| eval::render(&decision));
            assert_eq!(first, again, "{records:?}");
        }
    }
    let duplicated = [
        outcome("o5", "CHG-3", "open", "findings", "r1"),
        outcome("o2", "CHG-9", "accepted", "findings", "r1"),
        outcome("o4", "CHG-3", "declined", "findings", "r0"),
        outcome("o1", "CHG-9", "open", "findings", "r1"),
        outcome("o3", "CHG-1", "open", "findings", "r1"),
    ];
    let mut given = duplicated.to_vec();
    for _ in 0..given.len() {
        given.rotate_left(1);
        let refusal = eval::evaluate(&ir, &case, &given).expect_err("refused");
        assert_eq!(refusal.code(), "duplicate-child-case");
        assert_eq!(
            refusal.to_string(),
            "child case `CHG-3` has more than one case_outcome record: `o4`, `o5`"
        );
    }
}

/// `not` over the form keeps three values: `unknown` with no record and with an open child,
/// `true` once a child ended outside the set, `false` once every child ended inside it.
#[test]
fn a_negated_form_keeps_unknown_apart_from_false() {
    let source = fixture().replacen(
        "      children:\n        in: [declined, accepted]",
        "      not: {children: {in: [declined, accepted]}}",
        1,
    );
    let ir = compiled(&source);
    let case = case("audit", &["findings"]);
    let value = |records: &[EvidenceRecord]| {
        eval::evaluate(&ir, &case, records)
            .expect("decides")
            .claims
            .get(&model::ClaimId::new("findings.resolved"))
            .expect("the claim")
            .value
    };
    assert_eq!(value(&[]), Truth::Unknown);
    assert_eq!(
        value(&[outcome("o1", "c1", "open", "findings", "r1")]),
        Truth::Unknown
    );
    assert_eq!(
        value(&[
            outcome("o1", "c1", "open", "findings", "r1"),
            outcome("o2", "c2", "abandoned", "findings", "r1"),
        ]),
        Truth::True
    );
    assert_eq!(
        value(&[outcome("o1", "c1", "accepted", "findings", "r0")]),
        Truth::False
    );
}

/// The ids of `ids` with random openings between them and to a protocol outside the set, no
/// protocol opening itself (which `canon validate` refuses).
fn random_set(rng: &mut Rng, ids: &[&str]) -> BTreeMap<String, BTreeSet<String>> {
    let mut graph = BTreeMap::new();
    for id in ids {
        let mut opens = BTreeSet::new();
        for other in ids.iter().chain(&["outside"]) {
            if other != id && rng.below(4) == 0 {
                opens.insert((*other).to_owned());
            }
        }
        graph.insert((*id).to_owned(), opens);
    }
    graph
}

fn opener(id: &str, opens: &BTreeSet<String>) -> Ir {
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

/// Whether `from` reaches `to` through one or more openings.
fn reaches(graph: &BTreeMap<String, BTreeSet<String>>, from: &str, to: &str) -> bool {
    let mut seen = BTreeSet::new();
    let mut pending: Vec<&str> = graph
        .get(from)
        .into_iter()
        .flatten()
        .map(String::as_str)
        .collect();
    while let Some(next) = pending.pop() {
        if next == to {
            return true;
        }
        if seen.insert(next) {
            pending.extend(graph.get(next).into_iter().flatten().map(String::as_str));
        }
    }
    false
}

/// D206 as a property: a set is refused exactly when its openings form a cycle; the cycle named
/// is a cycle of the set's openings, from the least id of any protocol on any cycle, through
/// greater ids only, each protocol once; and the same set given in any order names the same one.
#[test]
fn check_openings_names_a_real_least_cycle_whatever_the_order() {
    let ids = ["audit", "review", "software.change", "Zeta", "alpha", "b"];
    let mut rng = Rng(0x0D20_6D20);
    let mut refused = 0;
    for _ in 0..400 {
        let graph = random_set(&mut rng, &ids);
        let mut set: Vec<Ir> = graph.iter().map(|(id, opens)| opener(id, opens)).collect();
        let on_cycle: BTreeSet<&str> = graph
            .keys()
            .map(String::as_str)
            .filter(|id| reaches(&graph, id, id))
            .collect();
        let result = check_openings(&set);
        let Some(least) = on_cycle.first() else {
            assert_eq!(result, Ok(()), "{graph:?}");
            continue;
        };
        refused += 1;
        let cycle = result.expect_err("a cycle");
        let path: Vec<&str> = cycle.protocols.iter().map(|id| id.as_str()).collect();
        assert_eq!(path.first(), Some(least), "{graph:?}: {path:?}");
        assert_eq!(path.last(), Some(least), "{graph:?}: {path:?}");
        for pair in path.windows(2) {
            assert!(graph[pair[0]].contains(pair[1]), "{graph:?}: {path:?}");
        }
        let inner: BTreeSet<&str> = path[1..path.len() - 1].iter().copied().collect();
        assert_eq!(inner.len(), path.len() - 2, "{graph:?}: {path:?}");
        assert!(inner.iter().all(|id| id > least), "{graph:?}: {path:?}");
        assert_eq!(
            cycle.to_string(),
            format!(
                "protocols open cases under each other in a cycle: {}",
                path.join(" -> ")
            )
        );
        for _ in 0..4 {
            let at = rng.below(set.len());
            set.swap(0, at);
            set.reverse();
            assert_eq!(check_openings(&set), Err(cycle.clone()), "{graph:?}");
        }
    }
    assert!(refused >= 50, "{refused} cyclic sets");
}

/// A cycle through a protocol's second action is found; protocol ids differ by case, so `audit`
/// opening `Audit` is no self-opening, and the two opening each other are a cycle named from
/// `Audit`, the lesser in code-point order.
#[test]
fn a_cycle_through_a_second_action_and_ids_that_differ_by_case() {
    let opens = |ids: &[&str]| ids.iter().map(|id| (*id).to_owned()).collect();
    let set = [
        opener("audit", &opens(&["review", "software.change"])),
        opener("software.change", &opens(&["audit"])),
        opener("review", &opens(&[])),
    ];
    assert_eq!(
        check_openings(&set).expect_err("a cycle").to_string(),
        "protocols open cases under each other in a cycle: audit -> software.change -> audit"
    );
    let source = "format: protocol/1\nprotocol: {id: audit, revision: 1}\n\
        artifacts: {a: {}}\nactions: {o: {case_open: {protocol: Audit, subject: a}}}\n";
    assert_eq!(problems(source), Vec::<String>::new());
    let set = [compiled(source), opener("Audit", &opens(&["audit"]))];
    assert_eq!(
        check_openings(&set).expect_err("a cycle").to_string(),
        "protocols open cases under each other in a cycle: Audit -> audit -> Audit"
    );
}

/// canon-ir/1 writes a `children` form's outcomes once each in code-point order, whatever they
/// are, and reads the text back to the IR it was written from.
#[test]
fn the_ir_round_trips_outcomes_that_are_not_ascii() {
    let source = format!(
        "{OPENS}artifacts: {{findings: {{}}}}\nevidence_kinds: {{case_outcome: {{}}}}\n\
         claims:\n\
         \x20\x20c: {{true_when: {{children: {{in: ['z', 'é', 'Z', 'e\u{301}', '𝔸', 'a', 'ä']}}}}}}\n"
    );
    let ir = compiled(&source);
    let text = ir.canonical_json();
    let value: Value = serde_json::from_str(&text).expect("JSON");
    let mut expected = vec!["z", "é", "Z", "e\u{301}", "𝔸", "a", "ä"];
    expected.sort_by(|left, right| left.chars().cmp(right.chars()));
    assert_eq!(
        value["claims"]["c"]["true_when"]["children"]["in"],
        serde_json::json!(expected)
    );
    assert_eq!(eval::read_ir(&text), Ok(ir));
}

/// A hand-edited IR whose action opens a case under the protocol's own id does not read: the
/// reader compiles what it rebuilt, and the compiler refuses the self-opening.
#[test]
fn a_hand_edited_ir_that_opens_its_own_protocol_does_not_read() {
    let text = compiled(&fixture()).canonical_json();
    let edited = text.replacen(
        "\"protocol\": \"software.change\"",
        "\"protocol\": \"audit\"",
        1,
    );
    assert_ne!(edited, text, "the IR names the opened protocol as expected");
    let refusal = eval::read_ir(&edited).expect_err("refused");
    assert_eq!(refusal.code(), "malformed-input");
    assert!(
        refusal.to_string().contains("self-opened-case"),
        "{refusal}"
    );
}

/// The diff's module docs: a predicate whose `children` forms, taken as a set, differ between the
/// revisions is changed (`BREAKING`), whatever else it implies. A form joined by a second one
/// under an `any` is a different set, though implication alone would call it an expansion.
#[test]
fn the_diff_takes_a_changed_set_of_children_forms_as_breaking() {
    let from = compiled(&fixture());
    let to = compiled(
        &fixture()
            .replacen("revision: 1", "revision: 2", 1)
            .replacen(
                "      children:\n        in: [declined, accepted]",
                "      any: [{children: {in: [declined, accepted]}}, {children: {in: [inconclusive]}}]",
                1,
            ),
    );
    let changes = diff(&from, &to).expect("diffs").changes;
    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(changes[0].category, Category::Breaking, "{}", changes[0]);
    assert!(
        changes[0]
            .to_string()
            .starts_with("BREAKING: claim `findings.resolved` "),
        "{}",
        changes[0]
    );
}

/// Negating a `children` form keeps the set of forms and reverses the claim: `BREAKING`.
#[test]
fn the_diff_takes_a_negated_children_form_as_breaking() {
    let from = compiled(&fixture());
    let to = compiled(
        &fixture()
            .replacen("revision: 1", "revision: 2", 1)
            .replacen(
                "      children:\n        in: [declined, accepted]",
                "      not: {children: {in: [declined, accepted]}}",
                1,
            ),
    );
    let changes = diff(&from, &to).expect("diffs").changes;
    assert_eq!(changes.len(), 1, "{changes:?}");
    assert_eq!(changes[0].category, Category::Breaking, "{}", changes[0]);
}

/// An invalidation rule that invalidates a claim reading only a `children` form keeps no record
/// from anything: `inert-invalidation`, as the validator's module docs say.
#[test]
fn an_invalidation_of_a_claim_over_children_only_is_inert() {
    let source = fixture().replacen(
        "outcomes:\n",
        "invalidation:\n  replanned:\n    upstream: findings\n    invalidates: [findings.resolved]\n\noutcomes:\n",
        1,
    );
    let found = problems(&source);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].starts_with("inert-invalidation: "), "{found:?}");
}

/// A `children` form written directly in an action's precondition or an outcome's requirement
/// reads every accepted record too (the module docs of `eval`, step 5: "their order forms and forms
/// over the child cases over every record accepted"): a child's outcome recorded about an earlier
/// revision of its subject, or expired at the evaluation instant, still counts there.
#[test]
fn a_precondition_and_a_requirement_read_every_child_outcome() {
    let source = fixture()
        .replacen(
            "    description: The outcome of one child case the audit opened, one record per child; open while the child runs.\n",
            "    description: The outcome of one child case the audit opened.\n    max_age: 1h\n",
            1,
        )
        .replacen(
            "    requires:\n      claim: findings.resolved\n",
            "    requires:\n      children:\n        in: [accepted]\n",
            1,
        );
    let ir = compiled(&source);
    let case = case("audit", &["findings"]);
    let records = [
        outcome("o1", "c1", "accepted", "findings", "r0"),
        eval::read_evidence(
            "format: canon-evidence/1\nid: o2\nkind: case_outcome\nsubject: findings\n\
             subject_revision: r1\nchild_case: c2\nresult: accepted\n\
             observed_at: 2026-01-01T00:00:00Z\n",
        )
        .expect("a record"),
    ];
    let decision = eval::evaluate_with(
        &ir,
        &case,
        &records,
        eval::Supplied {
            at: Some("2026-06-01T00:00:00Z"),
            ..Default::default()
        },
    )
    .expect("decides");
    let actions = decision.actions.expect("actions");
    let outcomes = decision.outcomes.expect("outcomes");
    assert_eq!(actions["close"]["status"], "admissible", "{actions}");
    assert_eq!(outcomes["resolved"]["status"], "legitimate", "{outcomes}");
}

/// A `children` form beside a match demanding independence over the same `case_outcome` records:
/// a record from a producer that is not independent counts as a child for the form and neither
/// establishes nor contradicts the match. Every combination a history gives is reached.
#[test]
fn check_reaches_children_jointly_with_a_match_demanding_independence() {
    let head = format!(
        "{OPENS}artifacts: {{findings: {{}}}}\n\
         evidence_kinds: {{case_outcome: {{}}}}\n\
         independence_dimensions: {{team: {{}}}}\n\
         actions: {{open: {{case_open: {{protocol: q, subject: findings}}}}}}\n\
         claims:\n\
         \x20\x20done: {{true_when: {{children: {{in: [accepted]}}}}}}\n\
         \x20\x20acc: {{true_when: {{evidence: {{kind: case_outcome, result: accepted, independent_on: [team]}}}}}}\n"
    );
    let ir = compiled(&head);
    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {findings: {revision: r1}}\n\
         independence: {indep: [team], dep: []}\n",
    )
    .expect("a case");
    let producers = ["", "indep", "dep"];
    let results = ["accepted", "open", "abandoned"];
    let mut rng = Rng(0x1D1D_1D1D);
    let mut found: BTreeMap<Vec<(String, Truth)>, Vec<EvidenceRecord>> = BTreeMap::new();
    for _ in 0..3000 {
        let records: Vec<EvidenceRecord> = (0..rng.below(5))
            .map(|n| {
                let producer = producers[rng.below(3)];
                let producer = if producer.is_empty() {
                    String::new()
                } else {
                    format!("producer: {producer}\n")
                };
                eval::read_evidence(&format!(
                    "format: canon-evidence/1\nid: h{n}\nkind: case_outcome\nsubject: findings\n\
                     subject_revision: {}\nchild_case: c{n}\nresult: {}\n{producer}",
                    ["r1", "r0"][rng.below(2)],
                    results[rng.below(3)],
                ))
                .expect("a record")
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
    assert!(found.len() >= 7, "{} combinations", found.len());
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
    for (index, (combination, records)) in found.iter().enumerate() {
        assert!(
            !report.contains(&format!("unreachable-outcome: outcome `o{index}` ")),
            "{combination:?} is legitimate on {records:?}, but the check calls it unreachable:\n{report}"
        );
    }
}

/// A `children` form's value does not depend on when a record was observed, and no order form
/// reads here, so the space needs no order of first observations (`check/space.rs`, `Space`: the
/// order dimension comes "only when an order form reads evidence"). Two `case_outcome` dimensions
/// (one bound by a match about `findings`, one for `report`) have 16 and 8 values and histories of
/// 17 and 9, beside the capability's 3: 58752 states, within the bound. An order dimension of 3
/// over the two histories triples that past the bound, and `canon check` refuses a protocol it
/// can check.
#[test]
fn check_adds_no_order_dimension_for_children_forms_alone() {
    let source = format!(
        "{OPENS}artifacts: {{findings: {{}}, report: {{}}}}\n\
         evidence_kinds: {{case_outcome: {{}}}}\n\
         actions: {{open: {{requires: [{{capability: case.open}}], case_open: {{protocol: q, subject: findings}}}}}}\n\
         claims:\n\
         \x20\x20done: {{true_when: {{children: {{in: [accepted]}}}}}}\n\
         \x20\x20declined_f: {{true_when: {{evidence: {{kind: case_outcome, subject: findings, result: declined}}}}}}\n"
    );
    match check(&compiled(&source), None) {
        Ok(report) => assert!(
            report
                .to_string()
                .starts_with("checked: protocol `p` revision 1: 58752 states, "),
            "{report}"
        ),
        Err(refusal) => panic!("{}: {refusal}", refusal.code()),
    }
}
