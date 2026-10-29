//! Adversary pass for story:evidence-order-subsecond (CANON-ORDER-003), against the library: the
//! fraction of an instant at its parsing edges, the intervals at their boundaries, the order forms
//! over fractional records (the `after` self-rule, several records that may be first, `before`
//! over one kind, records without `observed_at`), expiry in whole seconds, the evaluation
//! instant, order independence, and `canon check`'s claim that every evidence set maps to a state
//! that evaluates the same way.

use b10x_canon::check::check;
use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir::{self, Ir};
use b10x_canon::model::{self, ClaimId, EvidenceRecord, Instant, Truth};

const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n";

fn compiled(source: &str) -> Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

/// One record about `a` at `r1`, its instant written unquoted, as a consumer's YAML would.
fn record(id: &str, kind: &str, result: Option<&str>, at: Option<&str>) -> EvidenceRecord {
    let mut text = format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\nsubject: a\nsubject_revision: r1\n"
    );
    if let Some(result) = result {
        text.push_str(&format!("result: {result}\n"));
    }
    if let Some(at) = at {
        text.push_str(&format!("observed_at: {at}\n"));
    }
    eval::read_evidence(&text).expect("a record")
}

/// Order forms over `k` and `m`, and expiry over `fresh` (a one-second `max_age`).
const ORDER: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {a: {}}\n\
    evidence_kinds: {k: {}, m: {}, fresh: {max_age: 1s}}\n\
    claims:\n\
    \x20\x20after: {true_when: {after: {first: k, then: {kind: m, result: failed}}}}\n\
    \x20\x20again: {true_when: {after: {first: m, then: {kind: m, result: failed}}}}\n\
    \x20\x20before: {true_when: {before: {first: k, then: m}}}\n\
    \x20\x20first: {true_when: {first: {kind: m, result: failed}}}\n\
    \x20\x20applies: {true_when: {evidence: {kind: fresh, result: passed}}}\n";

fn decide(evidence: &[EvidenceRecord], at: Option<&str>) -> model::Decision {
    let case = eval::read_case(CASE).expect("a case");
    let supplied = Supplied {
        at,
        ..Supplied::default()
    };
    eval::evaluate_with(&compiled(ORDER), &case, evidence, supplied).expect("decides")
}

fn claim(evidence: &[EvidenceRecord], name: &str) -> Truth {
    decide(evidence, None)
        .claims
        .get(&ClaimId::new(name))
        .expect("declared")
        .value
}

const S: &str = "2026-10-01T12:00:00";

fn at(fraction: &str) -> String {
    format!("{S}{fraction}Z")
}

/// The parsing edges the story names: leap days with a fraction, the last nanosecond of a day,
/// and every way a fraction can be written wrong, including non-ASCII digits, signs, case,
/// surrounding junk, ten or more digits and digit strings that would overflow.
#[test]
fn the_fraction_is_read_exactly_at_its_edges() {
    let second = |text: &str| Instant::new(text).seconds();
    let interval = |text: &str| Instant::new(text).interval();
    assert_eq!(
        second("2024-02-29T23:59:59.999999999Z"),
        Some(1_709_251_199)
    );
    assert_eq!(
        interval("2024-02-29T23:59:59.999999999Z"),
        Some((
            1_709_251_199_i128 * 1_000_000_000 + 999_999_999,
            1_709_251_200_i128 * 1_000_000_000
        ))
    );
    assert_eq!(second("2000-02-29T00:00:00.5Z"), Some(951_782_400));
    assert_eq!(
        interval("2026-10-04T12:00:00.00000000Z"),
        Some((
            1_791_115_200_i128 * 1_000_000_000,
            1_791_115_200_i128 * 1_000_000_000 + 10
        ))
    );
    for refused in [
        "2023-02-29T00:00:00.5Z",
        "2100-02-29T00:00:00.1Z",
        "2026-10-04T23:59:60.0Z",
        "2026-10-04T24:00:00.0Z",
        "2026-10-04T12:00:00.\u{ff15}Z",
        "2026-10-04T12:00:00.\u{0665}Z",
        "2026-10-04T12:00:00.+5Z",
        "2026-10-04T12:00:00.-0Z",
        "+2026-10-04T12:00:00.5Z",
        " 2026-10-04T12:00:00.5Z",
        "2026-10-04T12:00:00.5Z ",
        "2026-10-04T12:00:00.5Z\n",
        "2026-10-04T12:00:00.5 Z",
        "2026-10-04T12:00:00. 5Z",
        "2026-10-04T12:00:00.5e1Z",
        "2026-10-04T12:00:00.5\u{0}Z",
        "2026-10-04T12:00:00.5z",
        "2026-10-04t12:00:00.5Z",
        "2026-10-04T12:00:00.1234567890Z",
        "2026-10-04T12:00:00.99999999999999999999Z",
        "2026-10-04T12:00:00.5.5Z",
        "2026-10-04T12:00.5Z",
        "2026-10-04T12:00:0.5Z",
    ] {
        assert_eq!(second(refused), None, "{refused:?} is read");
        assert_eq!(interval(refused), None, "{refused:?} names an interval");
    }
}

/// A fractional `observed_at` written unquoted in YAML keeps its text: the parser does not turn
/// it into a timestamp and back.
#[test]
fn an_unquoted_fraction_keeps_its_text() {
    for fraction in [".1", ".100", ".000000001", ".999999999", ".50"] {
        let written = at(fraction);
        let read = record("e", "k", None, Some(&written));
        assert_eq!(
            read.observed_at.as_ref().map(Instant::as_str),
            Some(written.as_str())
        );
    }
}

/// The boundaries the story names: `.9` against the next whole second, `.99` within `.9`, nine
/// digits against a whole second, and an interval ending exactly where the other begins.
#[test]
fn intervals_order_at_their_boundaries() {
    use Truth::{False as F, True as T, Unknown as U};
    let next = "2026-10-01T12:00:01Z";
    for (k, m, expected, why) in [
        (
            at(".9"),
            next.to_owned(),
            T,
            ".9 ends where the next second begins",
        ),
        (next.to_owned(), at(".9"), F, "the next second after .9"),
        (at(".9"), at(".99"), U, ".99 within .9"),
        (at(".99"), at(".9"), U, ".9 around .99"),
        (at(".98"), at(".99"), T, "neighbouring hundredths"),
        (at(".999999999"), next.to_owned(), T, "the last nanosecond"),
        (
            at(""),
            at(".999999999"),
            U,
            "the last nanosecond within the second",
        ),
        (
            at(".999999999"),
            at(""),
            U,
            "the second around its last nanosecond",
        ),
        (at(".5"), at(".499999999"), F, "a nanosecond before .5"),
        (
            at(".5"),
            at(".500000000"),
            U,
            "one instant at two precisions",
        ),
    ] {
        let evidence = [
            record("k", "k", None, Some(&k)),
            record("m", "m", Some("failed"), Some(&m)),
        ];
        assert_eq!(claim(&evidence, "after"), expected, "after: {why}");
        assert_eq!(claim(&evidence, "before"), expected, "before: {why}");
    }
}

/// `after` over one kind with fractions: a record is not later than itself, but a different record
/// at an overlapping instant is not ordered against it, whatever their precisions.
#[test]
fn after_over_one_kind_keeps_the_self_rule_with_fractions() {
    use Truth::{False as F, True as T, Unknown as U};
    let red = |id: &str, time: &str| record(id, "m", Some("failed"), Some(time));
    let green = |id: &str, time: &str| record(id, "m", Some("passed"), Some(time));
    for (evidence, expected, why) in [
        (vec![red("r", &at(".5"))], F, "one record"),
        (
            vec![red("r", &at(".500000000"))],
            F,
            "one record, nine digits",
        ),
        (
            vec![red("r", &at(".5")), red("s", &at(".50"))],
            U,
            "two records, one inside the other",
        ),
        (
            vec![red("r", &at(".5")), green("g", &at(""))],
            U,
            "a pass in the whole second around it",
        ),
        (
            vec![red("r", &at("")), green("g", &at(".5"))],
            U,
            "a pass inside its whole second",
        ),
        (
            vec![red("r", &at(".4")), green("g", &at(".5"))],
            F,
            "only a pass later",
        ),
        (
            vec![green("g", &at(".4")), red("r", &at(".5"))],
            T,
            "a failure after a pass",
        ),
        (
            vec![red("r", &at(".4")), red("s", &at(".5"))],
            T,
            "a failure after a failure",
        ),
        (
            vec![red("r", &at(".4")), record("n", "m", None, None)],
            U,
            "another record of the kind without an instant",
        ),
    ] {
        assert_eq!(claim(&evidence, "again"), expected, "{why}");
    }
}

/// `first` with several records that may be first: those no record of the kind is observed
/// before, including a coarse record overlapping finer ones that are ordered among themselves.
#[test]
fn first_reads_every_record_that_may_be_first() {
    use Truth::{False as F, True as T, Unknown as U};
    let red = |id: &str, time: &str| record(id, "m", Some("failed"), Some(time));
    let green = |id: &str, time: &str| record(id, "m", Some("passed"), Some(time));
    for (evidence, expected, why) in [
        (
            vec![
                red("r", &at("")),
                green("g", &at(".2")),
                red("s", &at(".5")),
            ],
            U,
            "the whole second and .2 may be first",
        ),
        // story:evidence-order-subsecond, order settled at one resolution per evaluation: the
        // whole-second record makes the evaluation compare seconds, so .2 and .5 tie with it and
        // the pass may be first too. Was `true` under pairwise interval comparison.
        (
            vec![
                red("r", &at("")),
                red("s", &at(".2")),
                green("g", &at(".5")),
            ],
            U,
            "the whole second makes .2 and .5 one instant, so the pass may be first",
        ),
        (
            vec![
                green("g", &at("")),
                red("r", &at(".2")),
                green("h", &at(".5")),
            ],
            U,
            "a pass and a failure may be first",
        ),
        (
            vec![
                red("r", &at(".2")),
                green("g", &at(".25")),
                red("s", &at(".3")),
            ],
            U,
            ".25 lies within .2",
        ),
        (
            vec![
                green("g", &at(".1")),
                red("r", &at(".2")),
                red("s", &at(".2")),
            ],
            F,
            "a pass alone before two failures",
        ),
        (
            vec![
                red("r", &at(".1")),
                green("g", &at(".2")),
                record("s", "m", Some("failed"), None),
            ],
            U,
            "a record without an instant",
        ),
    ] {
        assert_eq!(claim(&evidence, "first"), expected, "{why}");
    }
}

/// `before` over one kind never reaches the order rules: the protocol is refused when compiled,
/// so fractions cannot make it `true` or `false`.
#[test]
fn before_over_one_kind_is_refused_when_compiled() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\nevidence_kinds: {m: {}}\n\
        claims: {same: {true_when: {before: {first: m, then: m}}}}\n";
    let problems = ir::compile(&model::parse(source).expect("parses")).expect_err("refused");
    let codes: Vec<&str> = problems.iter().map(|problem| problem.code()).collect();
    assert_eq!(codes, ["self-ordered-evidence"]);
}

/// Records without `observed_at` beside fractional ones: a compared record without one leaves the
/// order unknown, and a record of another result takes no part in `after`.
#[test]
fn records_without_an_instant_beside_fractions() {
    use Truth::{True as T, Unknown as U};
    let k = |id: &str, time: Option<&str>| record(id, "k", None, time);
    let red = |id: &str, time: Option<&str>| record(id, "m", Some("failed"), time);
    let green = |id: &str, time: Option<&str>| record(id, "m", Some("passed"), time);
    let (one, five) = (at(".1"), at(".5"));
    let cases = [
        (
            vec![k("k", Some(&one)), red("r", Some(&five)), green("g", None)],
            "after",
            T,
        ),
        (
            vec![k("k", Some(&one)), red("r", Some(&five)), green("g", None)],
            "before",
            U,
        ),
        (
            vec![k("k", Some(&one)), k("j", None), red("r", Some(&five))],
            "after",
            U,
        ),
        (
            vec![k("k", Some(&one)), red("r", Some(&five)), red("s", None)],
            "after",
            U,
        ),
    ];
    for (evidence, name, expected) in cases {
        assert_eq!(claim(&evidence, name), expected, "{name} over {evidence:?}");
    }
}

/// Expiry counts whole seconds on both sides: `max_age: 1s`, observed at `.9`, is not expired at
/// `.1` one second later, nor at `.9` one second later, and is at `.0` two seconds later.
#[test]
fn expiry_with_fractions_counts_whole_seconds() {
    let expired = |observed: &str, instant: &str| {
        let evidence = [record("f", "fresh", Some("passed"), Some(observed))];
        let decision = decide(&evidence, Some(instant));
        let found = decision
            .claims
            .get(&ClaimId::new("applies"))
            .expect("declared");
        (found.value, found.excluded_evidence.len())
    };
    assert_eq!(
        expired(&at(".9"), "2026-10-01T12:00:01.1Z"),
        (Truth::True, 0)
    );
    assert_eq!(
        expired(&at(".1"), "2026-10-01T12:00:01.9Z"),
        (Truth::True, 0)
    );
    assert_eq!(
        expired(&at(".9"), "2026-10-01T12:00:02Z"),
        (Truth::Unknown, 1)
    );
    assert_eq!(
        expired(&at(".9"), "2026-10-01T12:00:02.0Z"),
        (Truth::Unknown, 1)
    );
    assert_eq!(expired(&at(".9"), &at(".1")), (Truth::True, 0));
}

/// The evaluation instant is held to the same grammar as `observed_at`.
#[test]
fn the_evaluation_instant_refuses_what_observed_at_refuses() {
    let case = eval::read_case(CASE).expect("a case");
    for refused in [
        "2026-10-01T12:00:00.5z",
        "2026-10-01T12:00:00.Z",
        "2026-10-01T12:00:00.1234567890Z",
        "2026-10-01T12:00:00,5Z",
        "2026-10-01T12:00:00.\u{ff15}Z",
        "2026-10-01T12:00:00.5Z ",
    ] {
        let supplied = Supplied {
            at: Some(refused),
            ..Supplied::default()
        };
        let refusal =
            eval::evaluate_with(&compiled(ORDER), &case, &[], supplied).expect_err(refused);
        assert_eq!(refusal.code(), "invalid-instant", "{refused:?}");
    }
}

/// The decision over fractional records does not depend on the order the records are given in.
#[test]
fn fractional_records_decide_alike_in_any_order() {
    let records = [
        record("k", "k", None, Some(&at(""))),
        record("r", "m", Some("failed"), Some(&at(".2"))),
        record("g", "m", Some("passed"), Some(&at(".25"))),
        record("s", "m", Some("failed"), Some(&at(".7"))),
    ];
    let rendered = eval::render(&decide(&records, None));
    let orders = [[0, 1, 2, 3], [3, 2, 1, 0], [1, 3, 0, 2], [2, 0, 3, 1]];
    for order in orders {
        let permuted: Vec<EvidenceRecord> = order.iter().map(|&i| records[i].clone()).collect();
        assert_eq!(
            eval::render(&decide(&permuted, None)),
            rendered,
            "{order:?}"
        );
    }
}

/// `canon check` says every evidence set maps to a state that evaluates the same way, because a
/// state orders the first observations of the dimensions as a weak order (earlier, later or at one
/// instant). Intervals of mixed precision are not a weak order: `ka` at the whole second overlaps
/// both `kb` at `.2` and `kc` at `.5`, while `kb` is before `kc`. Outcome `interval` needs exactly
/// that shape, so no state of the check reaches it and the check reports it unreachable; no
/// evidence set may then make it legitimate.
///
/// story:evidence-order-subsecond, order settled at one resolution per evaluation: the evaluator
/// now reads all three at the coarsest precision written, so the evidence above ties all three and
/// leaves `bc` unknown. The property is unchanged and is held over every combination of
/// precisions in the grid below instead of the one evidence set that broke it under pairwise
/// interval comparison.
#[test]
fn check_reaches_what_mixed_precision_evidence_makes_legitimate() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\nevidence_kinds: {ka: {}, kb: {}, kc: {}}\n\
        actions: {produce: {may_produce: [{evidence: ka}, {evidence: kb}, {evidence: kc}]}}\n\
        claims:\n\
        \x20\x20fa: {true_when: {first: {kind: ka, result: passed}}}\n\
        \x20\x20ab: {true_when: {before: {first: ka, then: kb}}}\n\
        \x20\x20bc: {true_when: {before: {first: kb, then: kc}}}\n\
        \x20\x20ac: {true_when: {before: {first: ka, then: kc}}}\n\
        outcomes:\n\
        \x20\x20interval: {requires: {all: [{claim: fa}, {claim: bc}, \
        {claim: ab, is: unknown}, {claim: ac, is: unknown}]}}\n";
    let protocol = compiled(source);
    let case = eval::read_case(CASE).expect("a case");
    let report = check(&protocol, None).expect("checks").to_string();
    let unreachable = report.contains("unreachable-outcome: outcome `interval`");
    assert!(unreachable, "the check reaches `interval`:\n{report}");
    let next = "2026-10-01T12:00:01Z".to_owned();
    let instants = [
        at(""),
        at(".2"),
        at(".20"),
        at(".25"),
        at(".5"),
        at(".50"),
        at(".500000000"),
        next,
    ];
    for a in &instants {
        for b in &instants {
            for c in &instants {
                let evidence = [
                    record("a1", "ka", Some("passed"), Some(a)),
                    record("b1", "kb", None, Some(b)),
                    record("c1", "kc", None, Some(c)),
                ];
                let outcomes = eval::evaluate(&protocol, &case, &evidence)
                    .expect("decides")
                    .outcomes
                    .expect("outcomes");
                assert_ne!(
                    outcomes["interval"]["status"], "legitimate",
                    "check calls an outcome unreachable that the evaluator makes legitimate on \
                     {evidence:?}:\n{report}"
                );
            }
        }
    }
}
