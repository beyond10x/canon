//! Adversary cases for story:missing-evidence (wave 2026-11-20-w4, pass 1), against the evaluator
//! library: the `missing_evidence` section of `canon-decision/1`.
//!
//! Driven from CANON-FRONTIER-001 (`conformance/requirements.yaml`) and the story's `## Decided`
//! rule: a record is applicable when the revision-binding and freshness stages leave it and no
//! claim lists it in `excluded_evidence`; each admissible action whose `may_produce` is not empty
//! maps to the kinds of that list with no applicable record, in kind-identifier order; a blocked or
//! approval-required action is not a key; the section is absent when no action qualifies. Each
//! case builds a ground the unit's own scenario and unit tests do not.

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir;
use b10x_canon::model::{self, Decision, Truth};

fn compiled(protocol: &str) -> ir::Ir {
    ir::compile(&model::parse(protocol).expect("protocol parses")).expect("protocol compiles")
}

fn record(id: &str, kind: &str, extra: &str) -> model::EvidenceRecord {
    eval::read_evidence(&format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\nsubject: a\nsubject_revision: r1\n{extra}"
    ))
    .expect("evidence reads")
}

fn decide(
    ir: &ir::Ir,
    case: &str,
    evidence: &[model::EvidenceRecord],
    supplied: Supplied<'_>,
) -> Decision {
    let case = eval::read_case(case).expect("case reads");
    eval::evaluate_with(ir, &case, evidence, supplied).expect("decides")
}

/// The `missing_evidence` section alone, as `canon conform run` compares it.
fn section(decision: &Decision) -> String {
    let listed = ["missing_evidence".to_owned()].into_iter().collect();
    eval::render_sections(decision, &listed)
}

fn missing(decision: &Decision) -> Vec<(String, Vec<String>)> {
    decision
        .missing_evidence
        .as_ref()
        .map(|section| {
            section
                .iter()
                .map(|(action, kinds)| {
                    (
                        action.as_str().to_owned(),
                        kinds.iter().map(|kind| kind.as_str().to_owned()).collect(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn entry(action: &str, kinds: &[&str]) -> (String, Vec<String>) {
    (
        action.to_owned(),
        kinds.iter().map(|kind| (*kind).to_owned()).collect(),
    )
}

fn value(decision: &Decision, claim: &str) -> Truth {
    decision
        .claims
        .iter()
        .find(|(id, _)| id.as_str() == claim)
        .map(|(_, entry)| entry.value)
        .expect("claim is in the decision")
}

const CASE: &str = "format: canon-case/1\nid: C\nprotocol: p\nrevision: c1\n\
    artifacts: {a: {revision: r1}, up: {revision: u1}}\n";

/// Claim `inv` is invalidated by rule `r` and reads kind `k`; claim `free` reads `k` too and is not
/// invalidated; nothing reads `m`. `probe` may produce `k` and `m`.
const SHARED: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {a: {}, up: {}}\n\
    independence_dimensions: {d: {}}\n\
    evidence_kinds: {k: {max_age: 1h}, m: {}, n: {}}\n\
    claims:\n\
    \x20\x20inv: {true_when: {evidence: {kind: k}}}\n\
    \x20\x20free: {true_when: {evidence: {kind: k}}}\n\
    \x20\x20strict: {true_when: {evidence: {kind: n, independent_on: [d]}}}\n\
    \x20\x20lax: {true_when: {evidence: {kind: n}}}\n\
    invalidation:\n\
    \x20\x20r: {upstream: up, invalidates: [inv]}\n\
    actions:\n\
    \x20\x20probe: {may_produce: [{evidence: m}, {evidence: k}, {evidence: n}]}\n";

/// A record invalidated for one claim only is not applicable for its kind, although another claim
/// reads it and is `true` on it (the decided rule: "no claim lists it").
#[test]
fn a_record_invalidated_for_one_claim_and_read_by_another_is_not_applicable() {
    let ir = compiled(SHARED);
    let evidence = [record("e1", "k", "upstream_revisions: {up: u0}\n")];
    let decision = decide(&ir, CASE, &evidence, Supplied::default());
    assert_eq!(value(&decision, "free"), Truth::True);
    assert_eq!(value(&decision, "inv"), Truth::Unknown);
    assert_eq!(missing(&decision), [entry("probe", &["k", "m", "n"])]);
}

/// A record of a kind no claim reads is in no claim's `excluded_evidence`, so it is applicable.
#[test]
fn a_record_of_a_kind_no_claim_reads_is_applicable() {
    let ir = compiled(SHARED);
    let decision = decide(&ir, CASE, &[record("e1", "m", "")], Supplied::default());
    assert_eq!(missing(&decision), [entry("probe", &["k", "n"])]);
}

/// Two records of one kind, one set aside by binding or freshness and one current: the current one
/// is applicable, whichever order the records come in.
#[test]
fn a_stale_record_beside_a_current_one_of_the_same_kind_does_not_list_the_kind() {
    let ir = compiled(SHARED);
    let current = record("e2", "k", "observed_at: 2026-01-01T01:30:00Z\n");
    let mismatch = eval::read_evidence(
        "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r0\n",
    )
    .expect("evidence reads");
    let expired = record("e3", "k", "observed_at: 2026-01-01T00:00:00Z\n");
    let at = Supplied {
        at: Some("2026-01-01T02:00:00Z"),
        ..Supplied::default()
    };
    for evidence in [
        vec![mismatch.clone(), current.clone(), expired.clone()],
        vec![expired, current, mismatch],
    ] {
        let decision = decide(&ir, CASE, &evidence, at);
        assert_eq!(missing(&decision), [entry("probe", &["m", "n"])]);
    }
}

/// A record one claim rejects as `not_independent` is not applicable, although a claim that demands
/// nothing reads it and is `true` on it.
#[test]
fn a_record_not_independent_for_one_claim_and_read_by_another_is_not_applicable() {
    let ir = compiled(SHARED);
    let case = "format: canon-case/1\nid: C\nprotocol: p\n\
        artifacts: {a: {revision: r1}, up: {revision: u1}}\nindependence: {author: []}\n";
    let evidence = [record("e1", "n", "producer: author\n")];
    let decision = decide(&ir, case, &evidence, Supplied::default());
    assert_eq!(value(&decision, "lax"), Truth::True);
    assert_eq!(value(&decision, "strict"), Truth::Unknown);
    assert_eq!(missing(&decision), [entry("probe", &["k", "m", "n"])]);
}

/// Kinds are listed in identifier order, which is code-point order: an upper-case kind before a
/// lower-case one and `_` between them, whatever order `may_produce` and `evidence_kinds` give;
/// a kind listed twice in `may_produce` is listed once. Actions and kinds declared in another
/// order give the same bytes.
#[test]
fn kinds_are_in_code_point_order_and_declaration_order_changes_no_byte() {
    let protocol = |kinds: &str, actions: &str| {
        format!(
            "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nartifacts: {{a: {{}}}}\n\
             evidence_kinds: {{{kinds}}}\nactions:\n{actions}"
        )
    };
    let one = compiled(&protocol(
        "b: {}, Z: {}, _y: {}, a: {}, B: {}",
        "  zeta: {may_produce: [{evidence: b}, {evidence: Z}, {evidence: b}, {evidence: _y}]}\n  \
         Alpha: {may_produce: [{evidence: a}, {evidence: B}]}\n",
    ));
    let two = compiled(&protocol(
        "B: {}, a: {}, _y: {}, Z: {}, b: {}",
        "  Alpha: {may_produce: [{evidence: B}, {evidence: a}]}\n  \
         zeta: {may_produce: [{evidence: _y}, {evidence: b}, {evidence: Z}]}\n",
    ));
    let case = "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r1}}\n";
    let first = decide(&one, case, &[], Supplied::default());
    let second = decide(&two, case, &[], Supplied::default());
    assert_eq!(
        missing(&first),
        [
            entry("Alpha", &["B", "a"]),
            entry("zeta", &["Z", "_y", "b"])
        ]
    );
    assert_eq!(section(&first), section(&second));
    assert_eq!(eval::render(&first), eval::render(&second));
}

/// Authority: an action admissible only through a grant is a key; one whose only covering entry
/// asks for approval, or was taken at another case revision (so it does not apply and approval
/// is required), or is denied (blocked), is not, and with no other action the section is absent.
#[test]
fn only_an_action_granted_at_the_case_revision_is_a_key() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {}}\n\
         actions:\n  gated: {requires: [{capability: cap}], may_produce: [{evidence: k}]}\n",
    );
    let case =
        "format: canon-case/1\nid: C\nprotocol: p\nrevision: c2\nartifacts: {a: {revision: r1}}\n";
    let with = |authority: &'static str| Supplied {
        authority: Some(authority),
        ..Supplied::default()
    };
    let granted = decide(
        &ir,
        case,
        &[],
        with("- {capability: cap, decision: granted, case_revision: c2}\n"),
    );
    assert_eq!(missing(&granted), [entry("gated", &["k"])]);
    for (what, authority) in [
        (
            "approval",
            "- {capability: cap, decision: require_approval}\n",
        ),
        (
            "lapsed grant",
            "- {capability: cap, decision: granted, case_revision: c1}\n",
        ),
        ("denied", "- {capability: cap, decision: denied}\n"),
    ] {
        let decision = decide(&ir, case, &[], with(authority));
        assert_eq!(decision.missing_evidence, None, "{what}");
        assert!(
            !eval::render(&decision).contains("\"missing_evidence\""),
            "{what}"
        );
    }
}

/// An admissible action that opens a child case and may produce evidence is a key like any other.
#[test]
fn an_admissible_action_with_case_open_is_a_key() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {}}\n\
         actions:\n  spawn: {case_open: {protocol: q, subject: a}, may_produce: [{evidence: k}]}\n",
    );
    let case = "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r1}}\n";
    let decision = decide(&ir, case, &[], Supplied::default());
    assert_eq!(missing(&decision), [entry("spawn", &["k"])]);
    let decision = decide(&ir, case, &[record("e1", "k", "")], Supplied::default());
    assert_eq!(missing(&decision), [entry("spawn", &[])]);
}
