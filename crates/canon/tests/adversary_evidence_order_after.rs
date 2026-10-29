//! Adversary pass for story:evidence-order-after (CANON-ORDER-002), against the library: the
//! `after` order form at equal instants, without `observed_at`, over records the exclusion stages
//! set aside, with and without a subject, over one kind, for a result no record has, nested under
//! `not` and `any`, in `canon check`, through `canon-ir/1` and its canonical order, and in
//! `canon diff`.

use b10x_canon::check::check;
use b10x_canon::diff::{self, Category};
use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir::{self, Ir};
use b10x_canon::model::{self, ClaimId, EvidenceRecord, Truth};
use serde_json::Value;

const PROTOCOL: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {change: {}, notes: {}, dataset: {}}\n\
    evidence_kinds: {tests_change: {}, test_result: {max_age: 1h}}\n\
    claims:\n\
    \x20\x20red: {true_when: {after: {first: tests_change, then: {kind: test_result, result: failed}}}}\n\
    \x20\x20red_change: {true_when: {after: {first: tests_change, then: {kind: test_result, \
    result: failed}, subject: change}}}\n\
    \x20\x20again: {true_when: {after: {first: test_result, then: {kind: test_result, result: failed}}}}\n\
    \x20\x20never: {true_when: {after: {first: tests_change, then: {kind: test_result, result: flaky}}}}\n\
    \x20\x20not_red: {true_when: {not: {after: {first: tests_change, then: {kind: test_result, \
    result: failed}}}}}\n\
    \x20\x20red_or_pass: {true_when: {any: [{after: {first: tests_change, then: {kind: test_result, \
    result: failed}}}, {evidence: {kind: test_result, result: passed}}]}}\n\
    \x20\x20passes: {true_when: {evidence: {kind: test_result, result: passed}}}\n\
    invalidation:\n\
    \x20\x20dataset.revised: {upstream: dataset, invalidates: [red_or_pass, passes]}\n";

const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\n\
    artifacts: {change: {revision: r2}, notes: {revision: n1}, dataset: {revision: d2}}\n";

const T0: &str = "2026-10-01T08:59:59Z";
const T1: &str = "2026-10-01T09:00:00Z";
const T2: &str = "2026-10-01T09:00:01Z";
const T3: &str = "2026-10-01T09:00:02Z";

/// One record: its id, kind, result, subject and revision, instant, and upstream dataset revision.
struct R<'a> {
    id: &'a str,
    kind: &'a str,
    result: Option<&'a str>,
    subject: &'a str,
    revision: &'a str,
    at: Option<&'a str>,
    dataset: Option<&'a str>,
}

impl R<'_> {
    fn read(&self) -> EvidenceRecord {
        let mut text = format!(
            "format: canon-evidence/1\nid: {}\nkind: {}\nsubject: {}\nsubject_revision: {}\n",
            self.id, self.kind, self.subject, self.revision
        );
        if let Some(result) = self.result {
            text.push_str(&format!("result: {result}\n"));
        }
        if let Some(at) = self.at {
            text.push_str(&format!("observed_at: \"{at}\"\n"));
        }
        if let Some(dataset) = self.dataset {
            text.push_str(&format!("upstream_revisions: {{dataset: {dataset}}}\n"));
        }
        eval::read_evidence(&text).expect("a record")
    }
}

/// A tests_change about the change at its current revision.
fn tc<'a>(id: &'a str, at: Option<&'a str>) -> R<'a> {
    R {
        id,
        kind: "tests_change",
        result: None,
        subject: "change",
        revision: "r2",
        at,
        dataset: None,
    }
}

/// A test_result about the change at its current revision.
fn run<'a>(id: &'a str, result: &'a str, at: Option<&'a str>) -> R<'a> {
    R {
        id,
        kind: "test_result",
        result: Some(result),
        subject: "change",
        revision: "r2",
        at,
        dataset: None,
    }
}

fn compiled(source: &str) -> Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

fn values_at(records: &[R<'_>], at: Option<&str>) -> Vec<(String, Truth, usize)> {
    let ir = compiled(PROTOCOL);
    let case = eval::read_case(CASE).expect("a case");
    let evidence: Vec<EvidenceRecord> = records.iter().map(R::read).collect();
    let supplied = Supplied {
        at,
        ..Supplied::default()
    };
    let decision = eval::evaluate_with(&ir, &case, &evidence, supplied).expect("decides");
    [
        "red",
        "red_change",
        "again",
        "never",
        "not_red",
        "red_or_pass",
        "passes",
    ]
    .into_iter()
    .map(|claim| {
        let found = decision.claims.get(&ClaimId::new(claim)).expect("declared");
        (claim.to_owned(), found.value, found.excluded_evidence.len())
    })
    .collect()
}

fn value(records: &[R<'_>], claim: &str) -> Truth {
    value_at(records, claim, None)
}

fn value_at(records: &[R<'_>], claim: &str, at: Option<&str>) -> Truth {
    values_at(records, at)
        .into_iter()
        .find(|(name, _, _)| name == claim)
        .expect("a claim of the protocol")
        .1
}

use Truth::{False as F, True as T, Unknown as U};

/// Equal instants: a record at the earliest instant of the first kind is not later than it,
/// whether it is of another kind or of the same; a record is not later than itself, but two
/// records at that instant leave which came first unknown.
#[test]
fn a_record_at_the_first_instant_is_not_later() {
    let cases: [(Vec<R<'_>>, &str, Truth, &str); 7] = [
        (
            vec![tc("c", Some(T1)), run("f", "failed", Some(T1))],
            "red",
            U,
            "two kinds at one instant",
        ),
        (
            vec![run("f", "failed", Some(T1))],
            "again",
            F,
            "one record is not later than itself",
        ),
        (
            vec![run("f", "failed", Some(T1)), run("g", "failed", Some(T1))],
            "again",
            U,
            "two failed runs at the first instant",
        ),
        (
            vec![run("p", "passed", Some(T1)), run("f", "failed", Some(T1))],
            "again",
            U,
            "a passed and a failed run at the first instant",
        ),
        (
            vec![run("p", "passed", Some(T1)), run("f", "failed", Some(T2))],
            "again",
            T,
            "a failed run a second after the first run",
        ),
        (
            vec![run("f", "failed", Some(T1)), run("p", "passed", Some(T2))],
            "again",
            F,
            "the only failed run is the first",
        ),
        (
            vec![
                tc("c", Some(T1)),
                run("f", "failed", Some(T1)),
                run("g", "failed", Some(T2)),
            ],
            "red",
            T,
            "one at the instant and one strictly later",
        ),
    ];
    for (records, claim, expected, why) in cases {
        assert_eq!(value(&records, claim), expected, "{claim}: {why}");
    }
}

/// A record without `observed_at` leaves the form unknown when it is of the first kind or of the
/// later kind with the result, and takes no part when it is of the later kind with another result.
/// With no record of the later kind with the result, the form is false whatever the instants.
#[test]
fn a_record_without_an_instant_counts_only_when_compared() {
    let cases: [(Vec<R<'_>>, &str, Truth, &str); 6] = [
        (
            vec![tc("c", None)],
            "red",
            F,
            "no failed run, the first unplaced",
        ),
        (
            vec![tc("c", Some(T1)), run("f", "failed", None)],
            "red",
            U,
            "the failed run unplaced",
        ),
        (
            vec![
                tc("c", Some(T1)),
                run("p", "passed", None),
                run("f", "failed", Some(T2)),
            ],
            "red",
            T,
            "a passed run unplaced takes no part",
        ),
        (
            vec![
                tc("c", Some(T1)),
                run("p", "passed", None),
                run("f", "failed", Some(T2)),
            ],
            "again",
            U,
            "over one kind the passed run is a first record, unplaced",
        ),
        (
            vec![
                tc("c", Some(T1)),
                tc("d", None),
                run("f", "failed", Some(T2)),
            ],
            "red",
            U,
            "a second tests_change unplaced",
        ),
        (
            vec![run("f", "failed", Some(T2))],
            "red",
            U,
            "no tests_change",
        ),
    ];
    for (records, claim, expected, why) in cases {
        assert_eq!(value(&records, claim), expected, "{claim}: {why}");
    }
}

/// `after` reads the history: a failed run at a superseded revision, one past its `max_age`, and
/// one invalidated for the claim (`red_or_pass`, whose `after` reads it while its evidence match excludes it) are each read, though the evidence match `passes` excludes the
/// same kind of record. No record is listed as excluded because of an order form.
#[test]
fn records_the_exclusion_stages_set_aside_are_read() {
    let superseded = R {
        revision: "r1",
        ..run("f", "failed", Some(T2))
    };
    let invalidated = R {
        dataset: Some("d1"),
        ..run("f", "failed", Some(T2))
    };
    for (record, at, why) in [
        (superseded, None, "revision_mismatch"),
        (
            run("f", "failed", Some(T2)),
            Some("2026-10-02T09:00:00Z"),
            "expired",
        ),
        (invalidated, None, "invalidated"),
    ] {
        let values = values_at(&[tc("c", Some(T1)), record], at);
        let red = values
            .iter()
            .find(|(name, _, _)| name == "red")
            .expect("red");
        assert_eq!((red.1, red.2), (T, 0), "{why}: {values:?}");
        let either = values
            .iter()
            .find(|(name, _, _)| name == "red_or_pass")
            .expect("red_or_pass");
        assert_eq!(either.1, T, "{why}: {values:?}");
        if why == "invalidated" {
            assert_eq!(either.2, 1, "invalidated for red_or_pass: {values:?}");
        }
    }
    let passed_and_invalidated = R {
        dataset: Some("d1"),
        ..run("p", "passed", Some(T2))
    };
    let values = values_at(&[tc("c", Some(T1)), passed_and_invalidated], None);
    let passes = values
        .iter()
        .find(|(name, _, _)| name == "passes")
        .expect("passes");
    assert_eq!(
        (passes.1, passes.2),
        (U, 1),
        "the control: the evidence match excludes an invalidated record"
    );
}

/// A subject reads only the records about it, of both kinds: the first tests_change about the
/// notes does not open the change's window, and a failed run about the notes does not fill it.
#[test]
fn a_subject_filters_both_kinds() {
    let notes_change = R {
        subject: "notes",
        revision: "n1",
        ..tc("n", Some(T0))
    };
    let notes_failed = R {
        subject: "notes",
        revision: "n1",
        ..run("g", "failed", Some(T3))
    };
    let records = [
        notes_change,
        run("f", "failed", Some(T1)),
        tc("c", Some(T2)),
        notes_failed,
    ];
    assert_eq!(
        value(&records, "red"),
        T,
        "unbound: the notes change opens it"
    );
    assert_eq!(
        value(&records, "red_change"),
        F,
        "bound: the change's tests changed after its only failed run"
    );
    let only_notes = [R {
        subject: "notes",
        revision: "n1",
        ..tc("n", Some(T0))
    }];
    assert_eq!(
        value(&only_notes, "red_change"),
        U,
        "bound: no tests_change about the change"
    );
}

/// A result no record has: false once the first kind has a record, unknown before.
#[test]
fn a_result_no_record_has_is_false_once_the_first_kind_has_a_record() {
    assert_eq!(
        value(&[tc("c", Some(T1)), run("f", "failed", Some(T2))], "never"),
        F
    );
    assert_eq!(value(&[run("f", "failed", Some(T2))], "never"), U);
    assert_eq!(value(&[], "never"), U);
}

/// Under `not` the three values swap as Kleene's; under `any` a true member decides.
#[test]
fn nested_under_not_and_any() {
    for (records, red, not_red) in [
        (vec![tc("c", Some(T1)), run("f", "failed", Some(T2))], T, F),
        (vec![tc("c", Some(T1)), run("f", "failed", Some(T0))], F, T),
        (vec![tc("c", Some(T1)), run("f", "failed", Some(T1))], U, U),
    ] {
        assert_eq!(value(&records, "red"), red);
        assert_eq!(value(&records, "not_red"), not_red);
    }
    let records = [run("p", "passed", Some(T2))];
    assert_eq!(value(&records, "red"), U);
    assert_eq!(value(&records, "red_or_pass"), T);
}

/// The order of the records given does not change any value.
#[test]
fn the_order_records_are_given_in_does_not_matter() {
    let records = [
        tc("c", Some(T1)),
        run("f", "failed", Some(T1)),
        run("g", "failed", Some(T2)),
        run("p", "passed", None),
    ];
    let forward = values_at(&records, None);
    let reversed: Vec<R<'_>> = records.into_iter().rev().collect();
    assert_eq!(values_at(&reversed, None), forward);
}

const KINDS: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {a: {}, b: {}}\nevidence_kinds: {k: {}, m: {}}\n";

/// Two `after` forms that differ in any one field are two members of an `all`: the compiler
/// deduplicates members its canonical order calls equal, so an order that ignored a field would
/// drop a requirement.
#[test]
fn afters_that_differ_in_one_field_are_kept_apart() {
    let members = [
        "{after: {first: k, then: {kind: m, result: r}}}",
        "{after: {first: m, then: {kind: m, result: r}}}",
        "{after: {first: k, then: {kind: k, result: r}}}",
        "{after: {first: k, then: {kind: m, result: s}}}",
        "{after: {first: k, then: {kind: m, result: r}, subject: a}}",
        "{after: {first: k, then: {kind: m, result: r}, subject: b}}",
        "{before: {first: k, then: m}}",
        "{first: {kind: k, result: r}}",
        "{evidence: {kind: m, result: r}}",
    ];
    let source = format!(
        "{KINDS}claims: {{c: {{true_when: {{all: [{}]}}}}}}\n",
        members.join(", ")
    );
    let json: Value = serde_json::from_str(&compiled(&source).canonical_json()).expect("JSON");
    let all = json["claims"]["c"]["true_when"]["all"]
        .as_array()
        .expect("an all");
    assert_eq!(all.len(), members.len(), "{all:#?}");
    let afters: Vec<String> = all
        .iter()
        .take(6)
        .map(|member| member["after"].to_string())
        .collect();
    assert_eq!(
        afters,
        [
            r#"{"first":"k","then":{"kind":"k","result":"r"}}"#,
            r#"{"first":"k","then":{"kind":"m","result":"r"}}"#,
            r#"{"first":"k","subject":"a","then":{"kind":"m","result":"r"}}"#,
            r#"{"first":"k","subject":"b","then":{"kind":"m","result":"r"}}"#,
            r#"{"first":"k","then":{"kind":"m","result":"s"}}"#,
            r#"{"first":"m","then":{"kind":"m","result":"r"}}"#,
        ],
        "afters first, by first kind, later kind, result, then subject (none before any)"
    );
    let repeated = format!(
        "{KINDS}claims: {{c: {{true_when: {{all: [{}, {}]}}}}}}\n",
        members[0], members[0]
    );
    let single = format!(
        "{KINDS}claims: {{c: {{true_when: {{all: [{}]}}}}}}\n",
        members[0]
    );
    assert_eq!(
        compiled(&repeated).canonical_json(),
        compiled(&single).canonical_json(),
        "one `after` written twice is one member"
    );
}

/// The IR `canon compile` prints reads back to the same protocol, whatever order the members were
/// written in; a hand-written `after` with a missing or extra key, or a `then` that is not a map,
/// is refused.
#[test]
fn the_ir_reads_back_and_refuses_a_malformed_after() {
    let forward = format!(
        "{KINDS}claims: {{c: {{true_when: {{any: [{{not: {{after: {{first: k, then: {{kind: k, \
         result: r}}}}}}}}, {{after: {{first: m, then: {{kind: k, result: r}}, subject: b}}}}, \
         {{claim: d}}]}}}}, d: {{true_when: {{after: {{first: k, then: {{kind: m, result: r}}}}}}}}}}\n"
    );
    let ir = compiled(&forward);
    let text = ir.canonical_json();
    assert_eq!(eval::read_ir(&text), Ok(ir));
    let mut json: Value = serde_json::from_str(&text).expect("JSON");
    let good = r#"{"after":{"first":"k","then":{"kind":"m","result":"r"}}}"#;
    assert_eq!(json["claims"]["d"]["true_when"].to_string(), good);
    let canonical =
        |json: &Value| format!("{}\n", serde_json::to_string_pretty(json).expect("JSON"));
    assert_eq!(canonical(&json), text, "the control is canonical");
    for bad in [
        r#"{"after":{"first":"k","then":{"kind":"m"}}}"#,
        r#"{"after":{"first":"k","then":"m"}}"#,
        r#"{"after":{"then":{"kind":"m","result":"r"}}}"#,
        r#"{"after":{"first":"k","then":{"kind":"m","result":"r"},"extra":1}}"#,
        r#"{"after":{"first":"k","then":{"kind":"m","result":"r","subject":"a"}}}"#,
        r#"{"after":{"first":"k","then":{"kind":"m","result":"r"},"subject":"z"}}"#,
        r#"{"after":{"first":"x","then":{"kind":"m","result":"r"}}}"#,
    ] {
        json["claims"]["d"]["true_when"] = serde_json::from_str(bad).expect("JSON");
        let refused = eval::read_ir(&canonical(&json));
        assert!(refused.is_err(), "{bad} is read");
    }
}

fn check_report(source: &str) -> String {
    match check(&compiled(source), None) {
        Ok(report) => report.to_string(),
        Err(refusal) => format!("refused {}: {refusal}", refusal.code()),
    }
}

/// `canon check` reaches every value: an outcome needing a failed run after the first run, and one
/// needing none, are both reachable; one needing both is not; a governed review produced without
/// authority that makes `not after` hold is a bypass.
#[test]
fn check_reaches_after_and_reports_what_it_cannot_reach() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {change: {}}\nevidence_kinds: {run: {}}\n\
        actions: {test: {may_produce: [{evidence: run}]}}\n\
        outcomes:\n\
        \x20\x20again: {requires: {after: {first: run, then: {kind: run, result: failed}}}}\n\
        \x20\x20not_again: {requires: {not: {after: {first: run, then: {kind: run, result: failed}}}}}\n\
        \x20\x20both: {requires: {all: [{after: {first: run, then: {kind: run, result: failed}}}, \
        {not: {after: {first: run, then: {kind: run, result: failed}}}}]}}\n";
    let report = check_report(source);
    assert!(
        report.contains("unreachable-outcome: outcome `both`"),
        "{report}"
    );
    assert!(
        !report.contains("outcome `again` is legitimate in no state")
            && !report.contains("outcome `not_again` is legitimate in no state"),
        "{report}"
    );
    let bypass = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {change: {}}\nevidence_kinds: {review: {}}\n\
        actions:\n\
        \x20\x20review: {requires: [{capability: review.give}], may_produce: [{evidence: review}]}\n\
        \x20\x20self_review: {may_produce: [{evidence: review}]}\n\
        outcomes:\n\
        \x20\x20not_rejected: {requires: {not: {after: {first: review, then: {kind: review, \
        result: rejected}}}}}\n";
    let bypassed = check_report(bypass);
    assert!(
        bypassed
            .contains("authority-bypass: outcome `not_rejected` is legitimate without authority"),
        "one review produced without authority makes `not after` hold:\n{bypassed}"
    );
}

/// `canon diff`: any change to an `after` is BREAKING, the same `after` written in another member
/// order is no semantic change, and an `after` added beside a requirement is classed as an evidence
/// match, a `first` or a `before` added there is.
#[test]
fn diff_classifies_a_changed_after() {
    let with = |revision: u32, requires: &str| {
        compiled(&format!(
            "format: protocol/1\nprotocol: {{id: p, revision: {revision}}}\n\
             artifacts: {{a: {{}}, b: {{}}}}\nevidence_kinds: {{k: {{}}, m: {{}}}}\n\
             outcomes: {{o: {{requires: {requires}}}}}\n"
        ))
    };
    let base = "{after: {first: k, then: {kind: m, result: r}}}";
    let categories = |to: &str| -> Vec<Category> {
        diff::diff(&with(1, base), &with(2, to))
            .expect("one protocol")
            .changes
            .into_iter()
            .filter(|change| change.section == "outcome")
            .map(|change| change.category)
            .collect()
    };
    for to in [
        "{after: {first: m, then: {kind: k, result: r}}}",
        "{after: {first: k, then: {kind: m, result: r}, subject: a}}",
        "{after: {first: k, then: {kind: k, result: r}}}",
        "{after: {first: k, then: {kind: m, result: s}}}",
        "{before: {first: k, then: m}}",
        "{not: {after: {first: k, then: {kind: m, result: r}}}}",
    ] {
        assert_eq!(categories(to), [Category::Breaking], "{to}");
    }
    let both = "{all: [{evidence: {kind: k}}, {after: {first: k, then: {kind: m, result: r}}}]}";
    let both_reordered =
        "{all: [{after: {first: k, then: {kind: m, result: r}}}, {evidence: {kind: k}}]}";
    let report = diff::diff(&with(1, both), &with(2, both_reordered)).expect("one protocol");
    assert!(
        report
            .changes
            .iter()
            .all(|change| change.section != "outcome"),
        "{:?}",
        report.changes
    );
    for from in ["{evidence: {kind: k}}", "{all: [{evidence: {kind: k}}]}"] {
        let mut categories = Vec::new();
        for added in [
            "{after: {first: k, then: {kind: m, result: r}}}",
            "{first: {kind: k, result: r}}",
            "{before: {first: k, then: m}}",
            "{evidence: {kind: m}}",
        ] {
            let to = format!("{{all: [{{evidence: {{kind: k}}}}, {added}]}}");
            let got: Vec<Category> = diff::diff(&with(1, from), &with(2, &to))
                .expect("one protocol")
                .changes
                .into_iter()
                .filter(|change| change.section == "outcome")
                .map(|change| change.category)
                .collect();
            categories.push((added, got));
        }
        let evidence = categories.last().expect("four").1.clone();
        for (added, got) in &categories {
            assert_eq!(got, &evidence, "{from} + {added}: {categories:?}");
        }
    }
}
