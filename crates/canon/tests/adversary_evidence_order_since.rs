//! Adversary pass for story:evidence-order-since (CANON-ORDER-004), against the library: `canon
//! check`'s claim that every evidence set maps to a state that evaluates the same way, for `since`
//! jointly with the other order forms and evidence matches; the values against the story over
//! excluded records, several subjects and revisions; and determinism under record order.

use std::collections::BTreeMap;

use b10x_canon::check::check;
use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir::{self, Ir};
use b10x_canon::model::{self, ClaimId, EvidenceRecord, Truth};

fn compiled(source: &str) -> Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

/// One record, its fields written as a consumer's YAML would.
fn record(
    id: &str,
    kind: &str,
    result: Option<&str>,
    subject: &str,
    revision: &str,
    at: Option<&str>,
) -> EvidenceRecord {
    let mut text = format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\nsubject: {subject}\nsubject_revision: {revision}\n"
    );
    if let Some(result) = result {
        text.push_str(&format!("result: {result}\n"));
    }
    if let Some(at) = at {
        text.push_str(&format!("observed_at: {at}\n"));
    }
    eval::read_evidence(&text).expect("a record")
}

/// A case with every artifact of `artifacts` at its current revision `r1`.
fn case(artifacts: &[&str]) -> model::Case {
    let listed: Vec<String> = artifacts
        .iter()
        .map(|artifact| format!("{artifact}: {{revision: r1}}"))
        .collect();
    eval::read_case(&format!(
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {{{}}}\n",
        listed.join(", ")
    ))
    .expect("a case")
}

/// The value of every claim, in claim-id order.
fn values(ir: &Ir, case: &model::Case, evidence: &[EvidenceRecord]) -> Vec<(String, Truth)> {
    let decision = eval::evaluate(ir, case, evidence).expect("decides");
    decision
        .claims
        .iter()
        .map(|(id, claim)| (id.as_str().to_owned(), claim.value))
        .collect()
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

/// Every combination of claim values that `histories` random histories give over `claims`, each
/// with the first history that gave it. Records are of `kinds`, about one of `artifacts`, at the
/// current revision `r1` or a superseded `r0` or `r2`, with one of `results`, at one of three whole
/// seconds or none.
#[allow(clippy::type_complexity)]
fn combinations(
    ir: &Ir,
    case: &model::Case,
    kinds: &[&str],
    artifacts: &[&str],
    results: &[Option<&str>],
    seed: u64,
    histories: usize,
) -> BTreeMap<Vec<(String, Truth)>, Vec<EvidenceRecord>> {
    let mut rng = Rng(seed);
    let instants = [
        Some("2026-10-01T09:00:00Z"),
        Some("2026-10-01T09:00:01Z"),
        Some("2026-10-01T09:00:02Z"),
        None,
    ];
    let mut found = BTreeMap::new();
    for _ in 0..histories {
        let records: Vec<EvidenceRecord> = (0..rng.below(6))
            .map(|n| {
                let at = instants[if rng.below(8) == 0 { 3 } else { rng.below(3) }];
                record(
                    &format!("h{n}"),
                    kinds[rng.below(kinds.len())],
                    results[rng.below(results.len())],
                    artifacts[rng.below(artifacts.len())],
                    ["r1", "r0", "r2"][rng.below(3)],
                    at,
                )
            })
            .collect();
        found.entry(values(ir, case, &records)).or_insert(records);
    }
    found
}

/// `canon check` over `claims` with one outcome per combination of claim values a random history
/// gives, requiring exactly that combination: every such outcome is legitimate on that history, so
/// the check must reach it. Returns each outcome the check calls unreachable, with its combination
/// and the history that makes it legitimate.
fn unreachable_but_legitimate(
    claims: &str,
    kinds: &[&str],
    artifacts: &[&str],
    results: &[Option<&str>],
    seed: u64,
) -> Vec<String> {
    let declared_artifacts: Vec<String> = artifacts.iter().map(|a| format!("{a}: {{}}")).collect();
    let declared_kinds: Vec<String> = kinds.iter().map(|k| format!("{k}: {{}}")).collect();
    let produced: Vec<String> = kinds.iter().map(|k| format!("{{evidence: {k}}}")).collect();
    let head = format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nartifacts: {{{}}}\n\
         evidence_kinds: {{{}}}\nactions: {{act: {{may_produce: [{}]}}}}\nclaims:\n{claims}",
        declared_artifacts.join(", "),
        declared_kinds.join(", "),
        produced.join(", "),
    );
    let case = case(artifacts);
    let found = combinations(
        &compiled(&head),
        &case,
        kinds,
        artifacts,
        results,
        seed,
        20_000,
    );
    let mut outcomes = String::from("outcomes:\n");
    let mut witnesses = Vec::new();
    for (index, (combination, history)) in found.iter().enumerate() {
        let tests: Vec<String> = combination
            .iter()
            .map(|(claim, value)| format!("{{claim: {claim}, is: {value}}}"))
            .collect();
        outcomes.push_str(&format!(
            "\x20\x20o{index}: {{requires: {{all: [{}]}}}}\n",
            tests.join(", ")
        ));
        witnesses.push((combination.clone(), history.clone()));
    }
    let protocol = compiled(&format!("{head}{outcomes}"));
    let report = check(&protocol, None).expect("checks").to_string();
    let mut contradicted = Vec::new();
    for (index, (combination, history)) in witnesses.iter().enumerate() {
        let line = format!("unreachable-outcome: outcome `o{index}` ");
        if report.contains(&line) {
            let outcome = eval::evaluate(&protocol, &case, history)
                .expect("decides")
                .outcomes
                .expect("outcomes");
            assert_eq!(
                outcome[format!("o{index}")]["status"],
                "legitimate",
                "o{index} is legitimate on its own history"
            );
            let written: Vec<String> = history
                .iter()
                .map(|record| {
                    format!(
                        "{} {:?} {}@{} {:?}",
                        record.kind.as_str(),
                        record.result,
                        record.subject.as_str(),
                        record.subject_revision.as_str(),
                        record.observed_at.as_ref().map(|at| at.as_str().to_owned()),
                    )
                })
                .collect();
            contradicted.push(format!("{combination:?} given by {written:?}"));
        }
    }
    contradicted
}

/// `since` jointly with `after` and evidence matches over the same two kinds: the check must reach
/// every combination a history gives. A latest anchor at a superseded revision followed by a
/// candidate at another superseded revision gives `since` false, `after` true, and no current
/// record of either kind; the space puts every non-current record at one revision.
#[test]
fn check_reaches_every_combination_of_since_with_after_and_matches() {
    let claims = "\x20\x20since_k_m: {true_when: {since: {last: k, then: {kind: m, result: fail}}}}\n\
        \x20\x20after_k_m: {true_when: {after: {first: k, then: {kind: m, result: fail}}}}\n\
        \x20\x20fail_m: {true_when: {evidence: {kind: m, result: fail}}}\n\
        \x20\x20some_k: {true_when: {evidence: {kind: k}}}\n";
    let contradicted = unreachable_but_legitimate(
        claims,
        &["k", "m"],
        &["a"],
        &[Some("fail"), Some("pass"), None],
        0x2545_F491_4F6C_DD1D,
    );
    assert!(
        contradicted.is_empty(),
        "check calls {} outcomes unreachable that a history makes legitimate:\n{}",
        contradicted.len(),
        contradicted.join("\n")
    );
}

/// A `since` naming a subject beside an evidence match that names none, over two artifacts.
#[test]
fn check_reaches_every_combination_of_since_over_two_subjects() {
    let claims = "\x20\x20since_a: {true_when: {since: {last: k, then: {kind: m, result: fail}, subject: a}}}\n\
        \x20\x20some_k: {true_when: {evidence: {kind: k}}}\n";
    let contradicted = unreachable_but_legitimate(
        claims,
        &["k", "m"],
        &["a", "b"],
        &[Some("fail"), None],
        0x9E37_79B9_7F4A_7C15,
    );
    assert!(
        contradicted.is_empty(),
        "check calls {} outcomes unreachable that a history makes legitimate:\n{}",
        contradicted.len(),
        contradicted.join("\n")
    );
}

/// A one-kind `since` beside a two-kind one and `before`.
#[test]
fn check_reaches_every_combination_of_one_kind_and_two_kind_since() {
    let claims = "\x20\x20since_m_m: {true_when: {since: {last: m, then: {kind: m, result: fail}}}}\n\
        \x20\x20since_k_m: {true_when: {since: {last: k, then: {kind: m, result: fail}}}}\n\
        \x20\x20before_k_m: {true_when: {before: {first: k, then: m}}}\n";
    let contradicted = unreachable_but_legitimate(
        claims,
        &["k", "m"],
        &["a"],
        &[Some("fail"), Some("pass")],
        0xBB67_AE85_84CA_A73B,
    );
    assert!(
        contradicted.is_empty(),
        "check calls {} outcomes unreachable that a history makes legitimate:\n{}",
        contradicted.len(),
        contradicted.join("\n")
    );
}

/// The test-first rule of the story, with `after` beside it and the evidence matches a consumer
/// writes for "the tests were changed at the current revision" and "a run failed at it". Outcome
/// `stale_red`: no `since`, an `after`, no change record at the current revision, and a run at the
/// current revision that did not fail.
const TEST_FIRST: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {a: {}}\n\
    evidence_kinds: {tests_change: {}, test_result: {}}\n\
    actions: {act: {may_produce: [{evidence: tests_change}, {evidence: test_result}]}}\n\
    claims:\n\
    \x20\x20red_since: {true_when: {since: {last: tests_change, then: {kind: test_result, result: failed}}}}\n\
    \x20\x20red_after: {true_when: {after: {first: tests_change, then: {kind: test_result, result: failed}}}}\n\
    \x20\x20change_now: {true_when: {evidence: {kind: tests_change}}}\n\
    \x20\x20red_now: {true_when: {evidence: {kind: test_result, result: failed}}}\n\
    outcomes:\n\
    \x20\x20stale_red: {requires: {all: [{claim: red_since, is: false}, {claim: red_after}, \
    {claim: change_now, is: unknown}, {claim: red_now, is: false}]}}\n";

/// Acceptance case 2 with the case moved on: the tests changed at `h2`, a failed run about `h1` was
/// observed later, and the change is now at `h3`, where a run passed; neither earlier record is
/// current. `since` is false
/// and `after` true, and outcome `stale_red` is legitimate. `canon check` calls it legitimate in no
/// state: every record a state adds that is not current is at the one superseded revision `r0`, so
/// a candidate observed after the latest anchor is either current or at the anchor's revision.
#[test]
fn check_reaches_a_failed_run_about_an_older_revision_observed_after_the_last_change() {
    let protocol = compiled(TEST_FIRST);
    let evidence = [
        record(
            "change",
            "tests_change",
            None,
            "a",
            "h2",
            Some("2026-10-01T09:00:01Z"),
        ),
        record(
            "run",
            "test_result",
            Some("failed"),
            "a",
            "h1",
            Some("2026-10-01T09:00:02Z"),
        ),
        record(
            "green",
            "test_result",
            Some("passed"),
            "a",
            "h3",
            Some("2026-10-01T09:00:02Z"),
        ),
    ];
    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: h3}}\n",
    )
    .expect("a case");
    let outcomes = eval::evaluate(&protocol, &case, &evidence)
        .expect("decides")
        .outcomes
        .expect("outcomes");
    assert_eq!(outcomes["stale_red"]["status"], "legitimate");
    let report = check(&protocol, None).expect("checks").to_string();
    assert!(
        !report.contains("unreachable-outcome: outcome `stale_red`"),
        "check calls `stale_red` unreachable, which {evidence:?} makes legitimate:\n{report}"
    );
}

/// The same outcome over two revisions only: the tests changed at the superseded `h1`, a failed
/// run at `h1` was observed after that, the tests changed again at `h1` (a second change record
/// about the same revision, as a re-observed change), and the case is at `h2`, where a run passed.
/// `since` is false
/// (the run comes before the latest change), `after` true (it comes after the first).
#[test]
fn check_reaches_a_failed_run_between_two_changes_at_one_superseded_revision() {
    let protocol = compiled(TEST_FIRST);
    let evidence = [
        record(
            "c1",
            "tests_change",
            None,
            "a",
            "h1",
            Some("2026-10-01T09:00:00Z"),
        ),
        record(
            "run",
            "test_result",
            Some("failed"),
            "a",
            "h1",
            Some("2026-10-01T09:00:01Z"),
        ),
        record(
            "c2",
            "tests_change",
            None,
            "a",
            "h1",
            Some("2026-10-01T09:00:02Z"),
        ),
        record(
            "green",
            "test_result",
            Some("passed"),
            "a",
            "h2",
            Some("2026-10-01T09:00:02Z"),
        ),
    ];
    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: h2}}\n",
    )
    .expect("a case");
    let outcomes = eval::evaluate(&protocol, &case, &evidence)
        .expect("decides")
        .outcomes
        .expect("outcomes");
    assert_eq!(outcomes["stale_red"]["status"], "legitimate");
    let report = check(&protocol, None).expect("checks").to_string();
    assert!(
        !report.contains("unreachable-outcome: outcome `stale_red`"),
        "check calls `stale_red` unreachable, which {evidence:?} makes legitimate:\n{report}"
    );
}

/// `since` over `k` and `m`, an expiring `k` and an invalidation rule over the claim, so that the
/// anchor is expired and the candidate invalidated for the claim.
/// Each claim is the `since` or a match on a kind no record has, so that the rule is not inert;
/// that match is `unknown`, so only a `true` `since` shows through it. The other rows use the same
/// protocol without the match and the rule.
const EXCLUDED: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {a: {}, up: {}}\n\
    evidence_kinds: {k: {max_age: 1s}, m: {}, never: {}}\n\
    claims:\n\
    \x20\x20since: {true_when: {any: [{since: {last: k, then: {kind: m, result: fail}}}, {evidence: {kind: never}}]}}\n\
    \x20\x20since_a: {true_when: {any: [{since: {last: k, then: {kind: m, result: fail}, subject: a}}, {evidence: {kind: never}}]}}\n\
    invalidation:\n\
    \x20\x20moved: {upstream: up, invalidates: [since, since_a]}\n";

/// The values the story gives, by the public evaluator: excluded records read (an expired anchor,
/// a candidate invalidated for the claim, records at a superseded revision), latest anchors at
/// several revisions and subjects, a candidate about another subject at the anchor's revision.
#[test]
fn since_values_over_excluded_records_revisions_and_subjects() {
    use Truth::{False as F, True as T, Unknown as U};
    let excluded = compiled(EXCLUDED);
    let plain = compiled(
        &EXCLUDED.replace(", {evidence: {kind: never}}", "").replace(
            "invalidation:\n\x20\x20moved: {upstream: up, invalidates: [since, since_a]}\n",
            "",
        ),
    );
    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}, up: {revision: u1}}\n",
    )
    .expect("a case");
    let t = |s: u32| format!("2026-10-01T09:00:0{s}Z");
    let k = |id: &str, subject: &str, revision: &str, s: u32| {
        record(id, "k", None, subject, revision, Some(&t(s)))
    };
    let red = |id: &str, subject: &str, revision: &str, s: Option<u32>| {
        let at = s.map(t);
        record(id, "m", Some("fail"), subject, revision, at.as_deref())
    };
    let invalidated = |mut record: EvidenceRecord| {
        let text = format!(
            "format: canon-evidence/1\nid: {}\nkind: {}\n{}subject: {}\nsubject_revision: {}\n{}upstream_revisions: {{up: u0}}\n",
            record.id.as_str(),
            record.kind.as_str(),
            record
                .result
                .as_ref()
                .map(|r| format!("result: {r}\n"))
                .unwrap_or_default(),
            record.subject.as_str(),
            record.subject_revision.as_str(),
            record
                .observed_at
                .as_ref()
                .map(|at| format!("observed_at: {}\n", at.as_str()))
                .unwrap_or_default(),
        );
        record = eval::read_evidence(&text).expect("a record");
        record
    };
    let cases: Vec<(Vec<EvidenceRecord>, Truth, Truth, &str)> = vec![
        (
            vec![
                k("k", "a", "r1", 1),
                invalidated(red("r", "a", "r1", Some(2))),
            ],
            T,
            T,
            "expired anchor, invalidated candidate after it",
        ),
        (
            vec![k("k", "a", "r0", 1), red("r", "a", "r0", Some(2))],
            T,
            T,
            "anchor and candidate at a superseded revision",
        ),
        (
            vec![
                k("k1", "a", "r1", 1),
                k("k0", "a", "r0", 1),
                red("r", "a", "r1", Some(2)),
            ],
            U,
            U,
            "two latest anchors, the superseded one has no candidate",
        ),
        (
            vec![
                k("k1", "a", "r1", 1),
                k("k0", "a", "r0", 1),
                red("r1", "a", "r1", Some(2)),
                red("r0", "a", "r0", Some(2)),
            ],
            T,
            T,
            "two latest anchors, each with a candidate after it",
        ),
        (
            vec![
                k("k1", "a", "r1", 1),
                k("k0", "a", "r0", 1),
                red("r1", "a", "r1", Some(0)),
                red("r0", "a", "r0", Some(0)),
            ],
            F,
            F,
            "two latest anchors, every candidate before them",
        ),
        (
            vec![k("ka", "a", "r1", 1), red("r", "up", "r1", Some(2))],
            F,
            F,
            "candidate about another subject at the anchor's revision",
        ),
        (
            vec![
                k("ka", "a", "r1", 1),
                k("kb", "up", "r1", 1),
                red("r", "a", "r1", Some(2)),
            ],
            U,
            T,
            "latest anchors about two subjects, a candidate for one",
        ),
        (
            vec![
                k("ka", "a", "r1", 1),
                red("r", "a", "r1", Some(2)),
                red("s", "up", "r9", None),
            ],
            U,
            T,
            "a candidate unplaced about another subject",
        ),
        (
            vec![
                k("old", "a", "r1", 0),
                red("r", "a", "r1", Some(1)),
                k("new", "a", "r1", 2),
            ],
            F,
            F,
            "a red between two changes at one revision",
        ),
    ];
    let at = Some("2026-10-01T09:05:00Z");
    for (evidence, expected, expected_a, why) in cases {
        let decision = eval::evaluate_with(
            if why.starts_with("expired") {
                &excluded
            } else {
                &plain
            },
            &case,
            &evidence,
            Supplied {
                at,
                ..Supplied::default()
            },
        )
        .expect("decides");
        let value = |name: &str| {
            decision
                .claims
                .get(&ClaimId::new(name))
                .expect("declared")
                .value
        };
        assert_eq!(value("since"), expected, "{why}");
        assert_eq!(value("since_a"), expected_a, "{why} (subject a)");
    }
}

/// The decision does not depend on the order the records are given in: every permutation of a
/// history with ties, two latest anchors at two revisions and one kind as both anchor and
/// candidate renders the same bytes.
#[test]
fn since_decides_alike_in_every_record_order() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\nevidence_kinds: {k: {}, m: {}}\n\
        claims:\n\
        \x20\x20two: {true_when: {since: {last: k, then: {kind: m, result: fail}}}}\n\
        \x20\x20one: {true_when: {since: {last: m, then: {kind: m, result: fail}}}}\n\
        actions: {go: {precondition: {all: [{claim: two}, {claim: one}, \
        {since: {last: m, then: {kind: m, result: pass}}}]}}}\n";
    let protocol = compiled(source);
    let case = case(&["a"]);
    let t = |s: u32| format!("2026-10-01T09:00:0{s}Z");
    let records = [
        record("k1", "k", None, "a", "r1", Some(&t(1))),
        record("k0", "k", None, "a", "r0", Some(&t(1))),
        record("m1", "m", Some("fail"), "a", "r1", Some(&t(2))),
        record("m2", "m", Some("pass"), "a", "r1", Some(&t(2))),
        record("m3", "m", Some("fail"), "a", "r0", Some(&t(0))),
    ];
    let rendered = eval::render(&eval::evaluate(&protocol, &case, &records).expect("decides"));
    let mut order: Vec<usize> = (0..records.len()).collect();
    let mut seen = 0;
    permute(&mut order, 0, &mut |order| {
        let permuted: Vec<EvidenceRecord> = order.iter().map(|&i| records[i].clone()).collect();
        let again = eval::render(&eval::evaluate(&protocol, &case, &permuted).expect("decides"));
        assert_eq!(again, rendered, "{order:?}");
        seen += 1;
    });
    assert_eq!(seen, 120);
}

fn permute(items: &mut Vec<usize>, from: usize, visit: &mut dyn FnMut(&[usize])) {
    if from == items.len() {
        visit(items);
        return;
    }
    for at in from..items.len() {
        items.swap(from, at);
        permute(items, from + 1, visit);
        items.swap(from, at);
    }
}
