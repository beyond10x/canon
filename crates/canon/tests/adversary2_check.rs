//! Adversary pass 2 for story:canon-check: the per-(kind, subject) evidence dimensions, erasure per
//! dimension in properties and authority bypasses, and the documents that describe them.

use b10x_canon::check::{check, read_properties};
use b10x_canon::ir;
use b10x_canon::model;

fn report(source: &str, properties: Option<&str>) -> String {
    let compiled = ir::compile(&model::parse(source).expect("parses")).expect("compiles");
    let properties = properties.map(|body| read_properties(body).expect("properties read"));
    match check(&compiled, properties.as_ref()) {
        Ok(report) => report.to_string(),
        Err(refusal) => format!("refused {}: {refusal}", refusal.code()),
    }
}

fn repository_file(relative: &str) -> String {
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR").expect("run through cargo");
    let path = std::path::PathBuf::from(manifest)
        .join("../..")
        .join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{relative}: {error}"))
}

/// Every declared artifact is named by a bound match and an unbound match also reads `k`: there
/// is no artifact left for an unbound dimension, so the space is `k` about `a` and `k` about `b`
/// only (4 states). `elsewhere` needs a record of `k` about neither artifact, which no case can
/// hold, so it is unreachable.
#[test]
fn with_every_artifact_named_there_is_no_unbound_dimension() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}, b: {}}\n\
        evidence_kinds: {k: {}}\n\
        claims:\n\
        \x20\x20a_seen: {true_when: {evidence: {kind: k, subject: a}}}\n\
        \x20\x20b_seen: {true_when: {evidence: {kind: k, subject: b}}}\n\
        actions: {probe: {may_produce: [{evidence: k}]}}\n\
        outcomes:\n\
        \x20\x20elsewhere: {requires: {all: [{claim: a_seen, is: unknown}, \
        {claim: b_seen, is: unknown}, {evidence: {kind: k}}]}}\n";
    assert_eq!(
        report(source, None),
        "unreachable-outcome: outcome `elsewhere` is legitimate in no state\n\
         checked: protocol `p` revision 1: 4 states, 0 properties, 1 finding\n"
    );
}

/// The first artifact, `a`, is named; `b` is not. The unbound dimension must be about `b`: a
/// record of `k` about `a` would make `a_seen` true, so `other` holds only on a record about `b`.
#[test]
fn the_unbound_dimension_is_about_an_artifact_no_match_names_even_when_the_first_is_named() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}, b: {}}\n\
        evidence_kinds: {k: {}}\n\
        claims: {a_seen: {true_when: {evidence: {kind: k, subject: a}}}}\n\
        actions: {probe: {may_produce: [{evidence: k}]}}\n\
        outcomes:\n\
        \x20\x20other: {requires: {all: [{claim: a_seen, is: unknown}, {evidence: {kind: k}}]}}\n";
    assert_eq!(
        report(source, None),
        "checked: protocol `p` revision 1: 4 states, 0 properties, 0 findings\n"
    );
}

/// The bound refusal names each dimension of a kind: the bound ones in subject order, then the
/// unbound one, without a subject. `k` about `a` and about `b` are read by their own match and by
/// the unbound one (2 + 2 results and no result: 32 values each); the unbound dimension only by the
/// unbound match (2 + 1: 8 values). 32 * 32 * 8 * 3 * 3 = 73728.
#[test]
fn the_bound_refusal_names_bound_dimensions_then_the_unbound_one() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}, b: {}, c: {}}\n\
        evidence_kinds: {k: {}}\n\
        claims:\n\
        \x20\x20x: {true_when: {any: [\
        {evidence: {kind: k, subject: b, result: s0}}, {evidence: {kind: k, subject: b, result: s1}}, \
        {evidence: {kind: k, result: u0}}, {evidence: {kind: k, result: u1}}, \
        {evidence: {kind: k, subject: a, result: r0}}, {evidence: {kind: k, subject: a, result: r1}}]}}\n\
        actions:\n\
        \x20\x20one: {requires: [{capability: c2}], may_produce: [{evidence: k}]}\n\
        \x20\x20two: {requires: [{capability: c1}]}\n";
    assert_eq!(
        report(source, None),
        "refused state-space-bound: protocol `p` revision 1 has 73728 states, more than the bound \
         of 65536: evidence kind `k` about `a` 32, evidence kind `k` about `b` 32, evidence kind \
         `k` 8, capability `c1` 3, capability `c2` 3"
    );
}

/// Declaration order is not an input: artifacts, kinds, claims and the members of an `all` written
/// in another order give the same bytes.
#[test]
fn declaration_order_does_not_change_the_report() {
    let first = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}, b: {}, c: {}}\n\
        evidence_kinds: {k: {}, m: {}}\n\
        claims:\n\
        \x20\x20a_pass: {true_when: {evidence: {kind: k, subject: a, result: pass}}}\n\
        \x20\x20c_fail: {true_when: {evidence: {kind: k, subject: c, result: fail}}}\n\
        actions:\n\
        \x20\x20approve: {requires: [{capability: c}], may_produce: [{evidence: k}]}\n\
        \x20\x20self: {may_produce: [{evidence: k}, {evidence: m}]}\n\
        outcomes:\n\
        \x20\x20done: {requires: {all: [{claim: a_pass}, {claim: c_fail}, {evidence: {kind: m}}]}}\n";
    let second = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {c: {}, b: {}, a: {}}\n\
        evidence_kinds: {m: {}, k: {}}\n\
        claims:\n\
        \x20\x20c_fail: {true_when: {evidence: {kind: k, subject: c, result: fail}}}\n\
        \x20\x20a_pass: {true_when: {evidence: {kind: k, subject: a, result: pass}}}\n\
        actions:\n\
        \x20\x20self: {may_produce: [{evidence: m}, {evidence: k}]}\n\
        \x20\x20approve: {requires: [{capability: c}], may_produce: [{evidence: k}]}\n\
        outcomes:\n\
        \x20\x20done: {requires: {all: [{evidence: {kind: m}}, {claim: c_fail}, {claim: a_pass}]}}\n";
    let expected = "authority-bypass: outcome `done` is legitimate without authority in state \
        {evidence `k` about `a` result `pass`, evidence `k` about `c` result `fail`, evidence `m`}\n\
        checked: protocol `p` revision 1: 96 states, 0 properties, 1 finding\n";
    assert_eq!(report(first, None), expected);
    assert_eq!(report(second, None), expected);
}

/// `both` is built on `a_seen` and adds a match about `b`. A property independent of `both`
/// erases both dimensions; one independent of `a_seen` only the dimension about `a`, and `done`
/// (which needs `both`) still depends on it, with the pair of states that shows it.
#[test]
fn a_claim_built_on_a_claim_about_another_artifact_erases_both_dimensions() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}, b: {}}\n\
        evidence_kinds: {k: {}}\n\
        claims:\n\
        \x20\x20a_seen: {true_when: {evidence: {kind: k, subject: a}}}\n\
        \x20\x20both: {true_when: {all: [{claim: a_seen}, {evidence: {kind: k, subject: b}}]}}\n\
        actions: {probe: {may_produce: [{evidence: k}]}}\n\
        outcomes: {done: {requires: {claim: both}}}\n";
    let on_a = "format: canon-properties/1\nprotocol: p\nproperties:\n\
        \x20\x20x: {subject: {outcome: done}, independent_of: {claim: a_seen}}\n";
    assert_eq!(
        report(source, Some(on_a)),
        "property-failed: property `x`: outcome `done` is blocked in state {evidence `k` about \
         `b`} and legitimate in state {evidence `k` about `a`, evidence `k` about `b`}\n\
         checked: protocol `p` revision 1: 4 states, 1 property, 1 finding\n"
    );
    let on_both = "format: canon-properties/1\nprotocol: p\nproperties:\n\
        \x20\x20y: {subject: {outcome: done}, independent_of: {claim: both}}\n";
    assert_eq!(
        report(source, Some(on_both)),
        "property-failed: property `y`: outcome `done` is blocked in state {} and legitimate in \
         state {evidence `k` about `a`, evidence `k` about `b`}\n\
         checked: protocol `p` revision 1: 4 states, 1 property, 1 finding\n"
    );
}

/// `granted` reads the governed `g` about `a` only; a record of `g` about `b` is beside the point.
/// The witness is the one record about `a`.
#[test]
fn a_bypass_on_evidence_about_one_artifact_names_only_that_record() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}, b: {}}\n\
        evidence_kinds: {g: {}}\n\
        claims: {b_seen: {true_when: {evidence: {kind: g, subject: b}}}}\n\
        actions:\n\
        \x20\x20approve: {requires: [{capability: c}], may_produce: [{evidence: g}]}\n\
        \x20\x20self_approve: {may_produce: [{evidence: g}]}\n\
        outcomes:\n\
        \x20\x20granted: {requires: {all: [{evidence: {kind: g, subject: a}}, \
        {claim: b_seen, is: unknown}]}}\n";
    assert_eq!(
        report(source, None),
        "authority-bypass: outcome `granted` is legitimate without authority in state \
         {evidence `g` about `a`}\n\
         checked: protocol `p` revision 1: 12 states, 0 properties, 1 finding\n"
    );
}

/// The story's acceptance: "a variant whose outcome is reachable without any authority-requiring
/// action reports the bypass". `released` reads the governed `approval`, and is reachable through
/// `run_check` alone, which needs no authority.
#[test]
fn an_outcome_reachable_through_a_free_alternative_to_governed_evidence_is_reported() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\n\
        evidence_kinds: {approval: {}, self_check: {}}\n\
        actions:\n\
        \x20\x20approve: {requires: [{capability: c}], may_produce: [{evidence: approval}]}\n\
        \x20\x20run_check: {may_produce: [{evidence: self_check}]}\n\
        outcomes:\n\
        \x20\x20released: {requires: {any: [{evidence: {kind: approval}}, \
        {evidence: {kind: self_check}}]}}\n";
    let text = report(source, None);
    assert!(
        text.lines()
            .any(|line| line.starts_with("authority-bypass: outcome `released`")),
        "an outcome reachable without any authority-requiring action was not reported:\n{text}"
    );
}

/// `closed` holds when `g` about `a` is present, or when there is no record about either artifact.
/// In {`g` about `a`, `g` about `b`} it holds only because of the record about `a`, which
/// `self_approve` produces without authority: take that record away and `closed` is blocked.
/// Erasing every governed dimension at once instead reaches the empty state, where `closed` holds
/// by absence, so the presence the outcome rests on goes unreported.
#[test]
fn a_bypass_is_not_hidden_by_erasing_every_governed_dimension_at_once() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}, b: {}}\n\
        evidence_kinds: {g: {}}\n\
        claims:\n\
        \x20\x20a_ok: {true_when: {evidence: {kind: g, subject: a}}}\n\
        \x20\x20b_ok: {true_when: {evidence: {kind: g, subject: b}}}\n\
        actions:\n\
        \x20\x20approve: {requires: [{capability: c}], may_produce: [{evidence: g}]}\n\
        \x20\x20self_approve: {may_produce: [{evidence: g}]}\n\
        outcomes:\n\
        \x20\x20closed: {requires: {any: [{claim: a_ok}, \
        {all: [{claim: a_ok, is: unknown}, {claim: b_ok, is: unknown}]}]}}\n";
    assert_eq!(
        report(source, None),
        "authority-bypass: outcome `closed` is legitimate without authority in state \
         {evidence `g` about `a`, evidence `g` about `b`}\n\
         checked: protocol `p` revision 1: 12 states, 0 properties, 1 finding\n"
    );
}

/// A property independent of a claim about `b` holds for an outcome that reads only `k` about `a`
/// (the check erases the dimension, not the kind). The published meaning of `independent_of` must
/// say so: "the evidence kinds it reads" would make this property fail, since `done` reads `k`.
#[test]
fn the_published_meaning_of_independent_of_matches_what_check_erases() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}, b: {}}\n\
        evidence_kinds: {k: {}}\n\
        claims:\n\
        \x20\x20a_seen: {true_when: {evidence: {kind: k, subject: a}}}\n\
        \x20\x20b_seen: {true_when: {evidence: {kind: k, subject: b}}}\n\
        actions: {probe: {may_produce: [{evidence: k}]}}\n\
        outcomes: {done: {requires: {claim: a_seen}}}\n";
    let body = "format: canon-properties/1\nprotocol: p\nproperties:\n\
        \x20\x20x: {subject: {outcome: done}, independent_of: {claim: b_seen}}\n";
    assert_eq!(
        report(source, Some(body)),
        "checked: protocol `p` revision 1: 4 states, 1 property, 0 findings\n"
    );
    for file in [
        "crates/canon/src/model/properties.rs",
        "ess/domains/check.yaml",
        "website/docs/reference/documents.md",
    ] {
        let text = repository_file(file)
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            !text.contains("evidence kinds it reads"),
            "{file} says a property's claim stands for the evidence kinds it reads; check erases \
             only the records the claim reads (kind and subject)"
        );
    }
}
