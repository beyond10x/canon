//! The direction `canon diff` gives a changed claim (story:case-inputs § Outcome 5): a claim is
//! classified by the direction of each place that reads it. A claim read positively, by a
//! precondition, an outcome requirement, a discharge predicate or another claim read positively,
//! is classified as its own predicate is; a claim read in reverse, under a `not`, as
//! `{claim: c, is: false}` or by an obligation's `applicable_when`, has the direction reversed; a
//! claim read in both directions is `BREAKING`; a claim nothing reads is classified as its own
//! predicate is.

use b10x_canon::diff::diff;
use b10x_canon::ir::{Ir, compile};
use b10x_canon::model::parse;

/// A protocol `p` at `revision` with the evidence kinds `k`, `h` and `e`, the claim `c` whose
/// predicate is `c`, the claim `e` (true on a record of kind `e`) and the declarations in `rest`.
fn protocol(revision: u32, c: &str, rest: &str) -> Ir {
    let source = format!(
        "format: protocol/1\nprotocol: {{id: p, revision: {revision}}}\n\
         evidence_kinds: {{k: {{}}, h: {{}}, e: {{}}}}\n\
         {rest}"
    )
    .replace("CLAIM_C", c);
    compile(&parse(&source).expect("parses")).expect("compiles")
}

/// `c` as one evidence match, and strengthened by a second under an `all`.
const WEAK: &str = "{evidence: {kind: k}}";
const STRONG: &str = "{all: [{evidence: {kind: k}}, {evidence: {kind: h}}]}";

/// The report's change lines, without the header, from `c` weak to `c` strong and back.
fn both_ways(rest: &str) -> (Vec<String>, Vec<String>) {
    let lines = |from: &Ir, to: &Ir| -> Vec<String> {
        diff(from, to)
            .expect("one protocol")
            .changes
            .iter()
            .map(ToString::to_string)
            .collect()
    };
    let weak = protocol(1, WEAK, rest);
    let strong = protocol(2, STRONG, rest);
    (lines(&weak, &strong), lines(&strong, &weak))
}

const CLAIMS: &str = "claims: {c: {true_when: CLAIM_C}, e: {true_when: {evidence: {kind: e}}}}\n";

#[test]
fn a_claim_read_under_a_not_in_a_precondition_reverses() {
    let (strengthened, weakened) = both_ways(&format!(
        "{CLAIMS}actions: {{a: {{precondition: {{not: {{claim: c}}}}}}}}\n"
    ));
    assert_eq!(
        strengthened,
        ["RELAXATION: claim `c` predicate strengthened, read in reverse"]
    );
    assert_eq!(
        weakened,
        ["TIGHTENING: claim `c` predicate weakened, read in reverse"]
    );
}

#[test]
fn a_claim_tested_false_reverses_and_one_tested_unknown_breaks() {
    let (strengthened, _) = both_ways(&format!(
        "{CLAIMS}actions: {{a: {{precondition: {{claim: c, is: false}}}}}}\n"
    ));
    assert_eq!(
        strengthened,
        ["RELAXATION: claim `c` predicate strengthened, read in reverse"]
    );
    let (strengthened, _) = both_ways(&format!(
        "{CLAIMS}actions: {{a: {{precondition: {{claim: c, is: unknown}}}}}}\n"
    ));
    assert_eq!(
        strengthened,
        ["BREAKING: claim `c` predicate strengthened, read in both directions"]
    );
}

#[test]
fn a_claim_read_positively_and_in_reverse_breaks() {
    let (strengthened, weakened) = both_ways(&format!(
        "{CLAIMS}actions: {{a: {{precondition: {{claim: c}}}}}}\n\
         obligations: {{o: {{applicable_when: {{claim: c}}, discharged_when: {{claim: e}}}}}}\n"
    ));
    assert_eq!(
        strengthened,
        ["BREAKING: claim `c` predicate strengthened, read in both directions"]
    );
    assert_eq!(
        weakened,
        ["BREAKING: claim `c` predicate weakened, read in both directions"]
    );
}

#[test]
fn a_claim_read_only_by_an_applicability_predicate_reverses() {
    let (strengthened, _) = both_ways(&format!(
        "{CLAIMS}obligations: {{o: {{applicable_when: {{claim: c}}, discharged_when: {{claim: e}}}}}}\n"
    ));
    assert_eq!(
        strengthened,
        ["RELAXATION: claim `c` predicate strengthened, read in reverse"]
    );
}

/// A claim read through another claim takes the direction that claim is read in: `c` is read
/// positively by `d`, and `d` under a `not` by an outcome, so `c` is read in reverse.
#[test]
fn a_claim_read_through_another_claim_takes_its_direction() {
    let (strengthened, _) = both_ways(
        "claims: {c: {true_when: CLAIM_C}, d: {true_when: {claim: c}}}\n\
         outcomes: {x: {requires: {not: {claim: d}}}}\n",
    );
    assert_eq!(
        strengthened,
        ["RELAXATION: claim `c` predicate strengthened, read in reverse"]
    );
}

/// Read positively everywhere, or by nothing at all, a claim is classified as its predicate is.
#[test]
fn a_claim_read_positively_or_by_nothing_keeps_its_direction() {
    for rest in [
        format!(
            "{CLAIMS}actions: {{a: {{precondition: {{claim: c}}}}}}\n\
             obligations: {{o: {{discharged_when: {{claim: c}}}}}}\n\
             outcomes: {{x: {{requires: {{not: {{not: {{claim: c}}}}}}}}}}\n"
        ),
        CLAIMS.to_owned(),
    ] {
        let (strengthened, weakened) = both_ways(&rest);
        assert_eq!(
            strengthened,
            ["TIGHTENING: claim `c` predicate strengthened"],
            "{rest}"
        );
        assert_eq!(
            weakened,
            ["RELAXATION: claim `c` predicate weakened"],
            "{rest}"
        );
    }
}
