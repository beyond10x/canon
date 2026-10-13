//! `canon check` and `canon generate` over evidence matches that demand independence
//! (story:evidence-independence): the state space they share holds, for each evidence dimension a
//! demanding match reads, records from a producer independent on what is demanded and from one
//! that is not, so no finding claims a demand can never be met, and every generated witness
//! passes when evaluated.

use std::path::PathBuf;

use b10x_canon::check::{self, Finding};
use b10x_canon::conform::{self, Verdict};
use b10x_canon::generate;
use b10x_canon::{ir, model};

const FIXTURE: &str = "fixtures/investigation/evidence-independence.yaml";

fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn compiled(source: &str) -> ir::Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

/// Every scenario generated for `source` passes when run against it; returns their ids.
fn witnesses_pass(source: &str, fixture: &str) -> Vec<String> {
    let scenarios = generate::generate(&compiled(source), fixture, source)
        .unwrap_or_else(|refusal| panic!("{}: {refusal}", refusal.code()));
    for scenario in &scenarios {
        let parsed = conform::parse(&scenario.text).expect("a generated scenario parses");
        assert_eq!(
            conform::run(&parsed, Ok(source)),
            Verdict::Passed,
            "{}",
            scenario.text
        );
    }
    scenarios.into_iter().map(|scenario| scenario.id).collect()
}

/// The fixture's `merge` action demands an approving review from a producer independent on
/// `different_principal`: some evidence set satisfies it, so `canon check` reports no
/// unsatisfiable precondition, and the generated witnesses pass.
#[test]
fn check_over_the_independence_fixture_reports_no_unsatisfiable_precondition() {
    let source = std::fs::read_to_string(repository_root().join(FIXTURE)).expect("readable");
    let report = check::check(&compiled(&source), None).expect("checked");
    let unsatisfiable: Vec<String> = report
        .findings
        .iter()
        .filter(|finding| matches!(finding, Finding::UnsatisfiablePrecondition { .. }))
        .map(ToString::to_string)
        .collect();
    assert_eq!(unsatisfiable, Vec::<String>::new(), "{report}");
    assert!(!report.to_string().contains("unsatisfiable"), "{report}");
    assert_eq!(
        witnesses_pass(&source, FIXTURE),
        ["change-review.action.merge.blocked"]
    );
}

/// An outcome that needs an independent approval is reachable, and its generated witness holds
/// a record from a producer the case lists as independent on the demanded dimension.
const OUTCOME: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1}\n\
    artifacts: {change: {}}\n\
    independence_dimensions: {different_principal: {}}\n\
    evidence_kinds: {review: {}}\n\
    claims:\n  \
      independently_approved: {true_when: {evidence: {kind: review, result: approved, \
      independent_on: [different_principal]}}}\n\
    actions: {review: {may_produce: [{evidence: review}]}}\n\
    outcomes: {merged: {requires: {claim: independently_approved}}}\n";

#[test]
fn an_outcome_needing_an_independent_record_is_reached_and_its_witness_passes() {
    let report = check::check(&compiled(OUTCOME), None).expect("checked");
    assert!(report.is_clean(), "{report}");
    assert_eq!(
        witnesses_pass(OUTCOME, "p.yaml"),
        ["p.outcome.merged.legitimate"]
    );
}

/// Three demands on one kind, `[d]`, `[e]` and `[d, e]`: a record independent on `d` alone meets
/// the first and not the others, so every combination of demands met is reached, including one
/// that needs two producers each independent on one dimension only. `contradict` needs `on_de`
/// false beside `on_d` true, which no evidence set gives (every record `on_de` reads, `on_d` reads
/// too, and a match reading mixed results is unknown): that finding is true and stays.
const NESTED: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1}\n\
    artifacts: {a: {}}\n\
    independence_dimensions: {d: {}, e: {}}\n\
    evidence_kinds: {k: {}}\n\
    claims:\n  \
      on_d: {true_when: {evidence: {kind: k, result: ok, independent_on: [d]}}}\n  \
      on_de: {true_when: {evidence: {kind: k, result: ok, independent_on: [d, e]}}}\n  \
      on_e: {true_when: {evidence: {kind: k, result: ok, independent_on: [e]}}}\n\
    actions: {run: {may_produce: [{evidence: k}]}, \
    ship: {precondition: {all: [{claim: on_d}, {claim: on_e}, {claim: on_de, is: unknown}]}}, \
    contradict: {precondition: {all: [{claim: on_d}, {claim: on_de, is: false}]}}}\n\
    outcomes:\n  \
      d_only: {requires: {all: [{claim: on_d}, {claim: on_de, is: unknown}]}}\n  \
      both: {requires: {claim: on_de}}\n  \
      neither: {requires: {all: [{claim: on_d, is: unknown}, {claim: on_e, is: unknown}]}}\n";

#[test]
fn every_combination_of_demands_met_is_reached() {
    let report = check::check(&compiled(NESTED), None).expect("checked");
    let findings: Vec<String> = report.findings.iter().map(ToString::to_string).collect();
    assert_eq!(
        findings,
        ["unsatisfiable-precondition: action `contradict`: its precondition holds in no state"],
        "{report}"
    );
    assert_eq!(
        witnesses_pass(NESTED, "p.yaml"),
        [
            "p.action.contradict.blocked",
            "p.action.ship.blocked",
            "p.outcome.both.legitimate",
            "p.outcome.d_only.legitimate",
            "p.outcome.neither.legitimate",
        ]
    );
}
