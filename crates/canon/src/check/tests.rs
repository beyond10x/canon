//! What the acceptance fixtures do not reach: every properties refusal, a protocol that declares no
//! artifact, and a space too large to count.

use super::*;
use crate::model::parse;

const PROTOCOL: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 2}\n\
    artifacts: {a: {}}\n\
    evidence_kinds: {k: {}}\n\
    claims: {c: {true_when: {evidence: {kind: k}}}}\n\
    actions: {act: {may_produce: [{evidence: k}]}}\n\
    outcomes: {done: {requires: {claim: c}}}\n";

fn compiled(source: &str) -> Ir {
    crate::ir::compile(&parse(source).expect("parses")).expect("compiles")
}

fn properties(body: &str) -> Properties {
    read_properties(body).expect("a canon-properties/1 document")
}

fn refused(source: &str, body: &str) -> (&'static str, String) {
    let refusal = check(&compiled(source), Some(&properties(body))).expect_err("refused");
    (refusal.code(), refusal.to_string())
}

const HEAD: &str = "format: canon-properties/1\nprotocol: p\n";

#[test]
fn a_protocol_without_findings_reports_its_summary_only() {
    let report = check(&compiled(PROTOCOL), None).expect("checked");
    assert_eq!(
        report.to_string(),
        "checked: protocol `p` revision 2: 2 states, 0 properties, 0 findings\n"
    );
    assert!(report.is_clean());
}

#[test]
fn properties_in_another_format_are_refused() {
    let (code, message) = refused(PROTOCOL, "format: canon-properties/2\nprotocol: p\n");
    assert_eq!(code, "unsupported-format");
    assert_eq!(
        message,
        "format is `canon-properties/2`, expected `canon-properties/1`"
    );
}

#[test]
fn properties_about_another_protocol_are_refused() {
    let (code, message) = refused(PROTOCOL, "format: canon-properties/1\nprotocol: q\n");
    assert_eq!(code, "protocol-mismatch");
    assert_eq!(message, "the properties are about protocol `q`, not `p`");
}

#[test]
fn a_property_about_an_undeclared_action_outcome_or_claim_is_refused() {
    let cases = [
        (
            "{x: {subject: {action: nope}, independent_of: {claim: c}}}",
            "undeclared-action",
            "property `x` is about action `nope`, which is not declared",
        ),
        (
            "{x: {subject: {outcome: nope}, independent_of: {claim: c}}}",
            "undeclared-outcome",
            "property `x` is about outcome `nope`, which is not declared",
        ),
        (
            "{x: {subject: {outcome: done}, independent_of: {claim: nope}}}",
            "undeclared-claim",
            "property `x` is independent of claim `nope`, which is not declared",
        ),
    ];
    for (declared, code, message) in cases {
        let body = format!("{HEAD}properties: {declared}\n");
        assert_eq!(refused(PROTOCOL, &body), (code, message.to_owned()));
    }
}

#[test]
fn a_property_declared_twice_is_refused() {
    let one = "{subject: {outcome: done}, independent_of: {claim: c}}";
    let body = format!("{HEAD}properties:\n  x: {one}\n  x: {one}\n");
    assert_eq!(
        refused(PROTOCOL, &body),
        (
            "duplicate-identifier",
            "property `x` is declared more than once".to_owned()
        )
    );
}

#[test]
fn a_subject_with_both_or_neither_form_is_malformed() {
    for subject in ["{action: act, outcome: done}", "{}"] {
        let body = format!(
            "{HEAD}properties: {{x: {{subject: {subject}, independent_of: {{claim: c}}}}}}\n"
        );
        let refusal = read_properties(&body).expect_err("malformed");
        assert_eq!(refusal.code(), "malformed-input", "{subject}");
    }
}

#[test]
fn a_property_that_holds_is_counted_and_not_reported() {
    let body = format!(
        "{HEAD}properties: {{x: {{subject: {{action: act}}, independent_of: {{claim: c}}}}}}\n"
    );
    let report = check(&compiled(PROTOCOL), Some(&properties(&body))).expect("checked");
    assert_eq!(
        report.to_string(),
        "checked: protocol `p` revision 2: 2 states, 1 property, 0 findings\n"
    );
}

/// Without an artifact no record is about anything, so each kind has the one value absent and an
/// outcome that needs evidence is unreachable.
#[test]
fn a_protocol_without_artifacts_has_no_evidence_states() {
    let source = PROTOCOL.replace("artifacts: {a: {}}\n", "");
    let report = check(&compiled(&source), None).expect("checked");
    assert_eq!(
        report.to_string(),
        "unreachable-outcome: outcome `done` is legitimate in no state\n\
         checked: protocol `p` revision 2: 1 state, 0 properties, 1 finding\n"
    );
}

/// A kind with 127 distinct results has 2^128 values: neither it nor the slice of `done`, which
/// reads it, fits a `u128`, and the refusal still names both.
#[test]
fn a_space_too_large_to_count_is_refused_naming_it() {
    let matches: Vec<String> = (0..127)
        .map(|n| format!("{{evidence: {{kind: k, result: r{n}}}}}"))
        .collect();
    let source = PROTOCOL.replace(
        "claims: {c: {true_when: {evidence: {kind: k}}}}",
        &format!(
            "claims: {{c: {{true_when: {{any: [{}]}}}}}}",
            matches.join(", ")
        ),
    );
    let refusal = check(&compiled(&source), None).expect_err("refused");
    assert_eq!(refusal.code(), "state-space-bound");
    assert_eq!(
        refusal.to_string(),
        format!(
            "protocol `p` revision 2: outcome `done` reads more than {} states, more than the \
             bound of 65536: evidence kind `k` 2^128",
            u128::MAX
        )
    );
}

/// Adversary pass 1, finding 4: an action precondition and an outcome requirement that read an
/// evidence kind directly, with no claim between, each need it produced, as a claim does. An
/// obligation reads evidence only through claims (`protocol/1` refuses evidence in a discharge
/// condition), and is reported through the claim it tests.
#[test]
fn every_reader_of_an_unproduced_kind_is_reported_not_only_claims() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
        evidence_kinds: {k: {}, m: {}}\n\
        claims: {seen: {true_when: {evidence: {kind: m}}}}\n\
        obligations: {o: {discharged_when: {claim: seen}}}\n\
        actions:\n\
        \x20\x20probe: {may_produce: [{evidence: k}]}\n\
        \x20\x20gate: {precondition: {evidence: {kind: m}}}\n\
        outcomes:\n\
        \x20\x20done: {requires: {evidence: {kind: m}}}\n";
    let report = check(&compiled(source), None).expect("checked");
    assert_eq!(
        report.to_string(),
        "unproduced-evidence: claim `seen` reads evidence kind `m`, which no action may produce\n\
         unproduced-evidence: obligation `o` reads evidence kind `m`, which no action may produce\n\
         unproduced-evidence: action `gate` reads evidence kind `m`, which no action may produce\n\
         unproduced-evidence: outcome `done` reads evidence kind `m`, which no action may produce\n\
         checked: protocol `p` revision 1: 4 states, 0 properties, 4 findings\n"
    );
}

/// Adversary pass 1, finding 6: `withdrawn` holds because the governed `approved` evidence is
/// absent, which needs no authority, so it is no bypass. `granted` holds because that evidence is
/// present, and `self_approve` produces it without authority, so it is one.
#[test]
fn an_outcome_that_holds_on_absent_governed_evidence_is_no_bypass() {
    let governed = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
        evidence_kinds: {k: {}}\n\
        claims: {approved: {true_when: {evidence: {kind: k}}}}\n\
        actions:\n\
        \x20\x20approve: {requires: [{capability: c}], may_produce: [{evidence: k}]}\n\
        outcomes:\n\
        \x20\x20withdrawn: {requires: {claim: approved, is: unknown}}\n";
    assert_eq!(
        check(&compiled(governed), None)
            .expect("checked")
            .to_string(),
        "checked: protocol `p` revision 1: 6 states, 0 properties, 0 findings\n"
    );
    let bypassed = governed
        .replace(
            "outcomes:\n",
            "\x20\x20self_approve: {may_produce: [{evidence: k}]}\noutcomes:\n",
        )
        .replace(
            "\x20\x20withdrawn:",
            "\x20\x20granted: {requires: {claim: approved}}\n\x20\x20withdrawn:",
        );
    assert_eq!(
        check(&compiled(&bypassed), None)
            .expect("checked")
            .to_string(),
        "authority-bypass: outcome `granted` is legitimate without authority in state \
         {evidence `k`}\n\
         checked: protocol `p` revision 1: 6 states, 0 properties, 1 finding\n"
    );
}

/// story:supplied-evidence: a kind that declares `supplied_by` is produced, so no
/// `unproduced-evidence` names it, and produced without authority, so a state where it is present
/// witnesses the bypass of `granted` through the governed `k` that `self_approve` also produces.
/// The kind `w`, which only `other` reads, makes the whole product four times `granted`'s slice,
/// so `canon check` decides each subject over its slice, and the three strategies agree. Without
/// `supplied_by`, `s` is unproduced, a state holding it is no witness and the bypass is hidden.
/// The state count does not change: production reads no state.
#[test]
fn a_supplied_kind_is_produced_without_authority_in_every_slice() {
    let supplied = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
        evidence_kinds: {k: {}, s: {supplied_by: q}, w: {}}\n\
        claims: {approved: {true_when: {evidence: {kind: k}}}, \
        reviewed: {true_when: {evidence: {kind: s}}}}\n\
        actions:\n\
        \x20\x20approve: {requires: [{capability: c}], may_produce: [{evidence: k}]}\n\
        \x20\x20self_approve: {may_produce: [{evidence: k}]}\n\
        \x20\x20watch: {may_produce: [{evidence: w}]}\n\
        outcomes:\n\
        \x20\x20granted: {requires: {all: [{claim: approved}, {claim: reviewed}]}}\n\
        \x20\x20other: {requires: {evidence: {kind: w}}}\n";
    let ir = compiled(supplied);
    assert_eq!(
        check(&ir, None).expect("checked").to_string(),
        "authority-bypass: outcome `granted` is legitimate without authority in state \
         {evidence `k`, evidence `s`}\n\
         checked: protocol `p` revision 1: 24 states, 0 properties, 1 finding\n"
    );
    let (widest, whole) = assert_slices_agree(&ir, None, supplied);
    assert!(widest * 2 < whole, "{widest} of {whole}");

    let unsupplied = supplied.replace("s: {supplied_by: q}", "s: {}");
    let ir = compiled(&unsupplied);
    assert_eq!(
        check(&ir, None).expect("checked").to_string(),
        "unproduced-evidence: claim `reviewed` reads evidence kind `s`, which no action may produce\n\
         unproduced-evidence: outcome `granted` reads evidence kind `s`, which no action may produce\n\
         checked: protocol `p` revision 1: 24 states, 0 properties, 2 findings\n"
    );
    assert_slices_agree(&ir, None, &unsupplied);
}

/// story:supplied-evidence, decided in review on 2026-11-21: a kind that declares `supplied_by`
/// counts as produced without authority and stays governed when an action that requires a
/// capability may also produce it, so `canon check` reports `authority-bypass` on an outcome a
/// supplied record alone makes legitimate. Without `supplied_by` only the governed action produces
/// the kind, so nothing is found; with a capability-free action producing it too, the bypass is
/// reported as it was before the field existed.
#[test]
fn a_supplied_kind_an_authority_action_produces_stays_governed() {
    let supplied = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
        evidence_kinds: {s: {supplied_by: q}}\n\
        claims: {ok: {true_when: {evidence: {kind: s}}}}\n\
        actions:\n\
        \x20\x20approve: {requires: [{capability: c}], may_produce: [{evidence: s}]}\n\
        outcomes:\n\
        \x20\x20done: {requires: {claim: ok}}\n";
    let bypass = "authority-bypass: outcome `done` is legitimate without authority in state \
        {evidence `s`}\n\
        checked: protocol `p` revision 1: 6 states, 0 properties, 1 finding\n";
    let ir = compiled(supplied);
    assert_eq!(check(&ir, None).expect("checked").to_string(), bypass);
    assert_slices_agree(&ir, None, supplied);

    let unsupplied = supplied.replace("s: {supplied_by: q}", "s: {}");
    assert_eq!(
        check(&compiled(&unsupplied), None)
            .expect("checked")
            .to_string(),
        "checked: protocol `p` revision 1: 6 states, 0 properties, 0 findings\n"
    );

    let free = unsupplied.replace(
        "outcomes:\n",
        "\x20\x20self_approve: {may_produce: [{evidence: s}]}\noutcomes:\n",
    );
    assert_eq!(
        check(&compiled(&free), None).expect("checked").to_string(),
        bypass
    );
}

/// Adversary pass 1, finding 7: the bound refusal the module docs show is one `check` gives, so
/// its count is the product of its dimensions. Reads the module source at run time. The slice of
/// `act` holds `k`, `m` and `c`, not the decision `d` only `o` reads (story:check-subject-slices).
#[test]
fn the_module_docs_show_a_bound_refusal_check_gives() {
    let results = |kind: &str, prefix: &str, n: usize| {
        (0..n)
            .map(|i| format!("{{evidence: {{kind: {kind}, result: {prefix}{i}}}}}"))
            .collect::<Vec<_>>()
    };
    let mut matches = results("k", "r", 10);
    matches.extend(results("m", "s", 3));
    let source = format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nartifacts: {{a: {{}}}}\n\
         evidence_kinds: {{k: {{}}, m: {{}}}}\n\
         claims: {{x: {{true_when: {{any: [{}]}}}}}}\n\
         actions: {{act: {{precondition: {{claim: x}}, requires: [{{capability: c}}], may_produce: [{{evidence: k}}, {{evidence: m}}]}}}}\n\
         outcomes: {{o: {{requires: {{decision: d}}}}}}\n",
        matches.join(", ")
    );
    let refusal = check(&compiled(&source), None).expect_err("refused");
    assert_eq!(refusal.code(), "state-space-bound");
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR").expect("run through cargo");
    let path = std::path::PathBuf::from(manifest).join("src/check/mod.rs");
    let text = std::fs::read_to_string(&path).expect("the module source is readable");
    let docs: Vec<&str> = text
        .lines()
        .filter_map(|line| line.strip_prefix("//!"))
        .map(str::trim)
        .collect();
    let docs = docs.join(" ");
    assert!(
        docs.contains(&format!("`` {refusal} ``")),
        "the module docs do not show `{refusal}`"
    );
}

/// Adversary pass 1, finding 1: a match that names a subject reads only records about it, so the
/// space has a dimension of `k` about `b`. The unbound `any_seen` also reads records about `a`,
/// which no match names, and `elsewhere` needs one with none about `b`: without that separate
/// dimension `elsewhere` would be unreachable. `k` about `b` has 4 values (`pass` and no result),
/// `k` about another artifact 2.
#[test]
fn an_unbound_match_keeps_its_own_dimension_beside_a_bound_one() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}, b: {}}\n\
        evidence_kinds: {k: {}}\n\
        claims:\n\
        \x20\x20any_seen: {true_when: {evidence: {kind: k}}}\n\
        \x20\x20b_passed: {true_when: {evidence: {kind: k, subject: b, result: pass}}}\n\
        actions: {probe: {may_produce: [{evidence: k}]}}\n\
        outcomes:\n\
        \x20\x20elsewhere: {requires: {all: [{claim: any_seen}, {claim: b_passed, is: unknown}]}}\n";
    assert_eq!(
        check(&compiled(source), None).expect("checked").to_string(),
        "checked: protocol `p` revision 1: 8 states, 0 properties, 0 findings\n"
    );
}

/// A bound dimension is written with its subject, in a witness state and in the bound refusal.
#[test]
fn a_bound_dimension_is_written_with_its_subject() {
    let bypassed = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}, b: {}}\n\
        evidence_kinds: {k: {}}\n\
        actions:\n\
        \x20\x20approve: {requires: [{capability: c}], may_produce: [{evidence: k}]}\n\
        \x20\x20self_approve: {may_produce: [{evidence: k}]}\n\
        outcomes:\n\
        \x20\x20granted: {requires: {evidence: {kind: k, subject: b, result: 'yes'}}}\n";
    assert_eq!(
        check(&compiled(bypassed), None)
            .expect("checked")
            .to_string(),
        "authority-bypass: outcome `granted` is legitimate without authority in state \
         {evidence `k` about `b` result `yes`}\n\
         checked: protocol `p` revision 1: 12 states, 0 properties, 1 finding\n"
    );
    let matches: Vec<String> = (0..16)
        .map(|n| format!("{{evidence: {{kind: k, subject: b, result: r{n}}}}}"))
        .collect();
    let large = format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nartifacts: {{a: {{}}, b: {{}}}}\n\
         evidence_kinds: {{k: {{}}}}\n\
         claims: {{x: {{true_when: {{any: [{}]}}}}}}\n\
         actions: {{probe: {{may_produce: [{{evidence: k}}]}}}}\n\
         outcomes: {{graded: {{requires: {{claim: x}}}}}}\n",
        matches.join(", ")
    );
    let refusal = check(&compiled(&large), None).expect_err("refused");
    assert_eq!(
        refusal.to_string(),
        "protocol `p` revision 1: outcome `graded` reads 131072 states, more than the bound of \
         65536: evidence kind `k` about `b` 131072"
    );
}

/// A property independent of a claim about `b` erases only the dimension of `k` about `b`: states
/// that differ in records about `a` are not partners, so `done`, which reads only those, holds it.
#[test]
fn a_property_erases_only_the_dimensions_its_claim_reads() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}, b: {}}\n\
        evidence_kinds: {k: {}}\n\
        claims:\n\
        \x20\x20a_seen: {true_when: {evidence: {kind: k, subject: a}}}\n\
        \x20\x20b_seen: {true_when: {evidence: {kind: k, subject: b}}}\n\
        actions: {probe: {may_produce: [{evidence: k}]}}\n\
        outcomes: {done: {requires: {claim: a_seen}}}\n";
    let body = format!(
        "{HEAD}properties: {{x: {{subject: {{outcome: done}}, independent_of: {{claim: b_seen}}}}}}\n"
    );
    assert_eq!(
        check(&compiled(source), Some(&properties(&body)))
            .expect("checked")
            .to_string(),
        "checked: protocol `p` revision 1: 4 states, 1 property, 0 findings\n"
    );
}

/// Adversary pass 2, findings 2 and 3: the bypass reads polarity. `clean` reads the governed `g`
/// under `not`, and `refuted` through `is: false`: both hold on what the evidence does not
/// establish, so neither is a bypass. `known` reads it under `not` of `is: unknown`, two flips, so
/// a record helps it hold, and `self_approve` gives one without authority.
#[test]
fn a_bypass_reads_governed_evidence_only_where_its_presence_helps() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
        evidence_kinds: {g: {}}\n\
        claims: {approved: {true_when: {evidence: {kind: g, result: 'yes'}}}}\n\
        actions:\n\
        \x20\x20approve: {requires: [{capability: c}], may_produce: [{evidence: g}]}\n\
        \x20\x20self_approve: {may_produce: [{evidence: g}]}\n\
        outcomes:\n\
        \x20\x20clean: {requires: {not: {evidence: {kind: g, result: bad}}}}\n\
        \x20\x20refuted: {requires: {claim: approved, is: false}}\n\
        \x20\x20known: {requires: {not: {claim: approved, is: unknown}}}\n";
    assert_eq!(
        check(&compiled(source), None).expect("checked").to_string(),
        "authority-bypass: outcome `known` is legitimate without authority in state \
         {evidence `g` result `bad`}\n\
         checked: protocol `p` revision 1: 24 states, 0 properties, 1 finding\n"
    );
}

/// A protocol whose invalidation rule makes states reachable that no state without the rule
/// reaches. `named` and `recorded` read the same dimension `b`; the rule invalidates `named`
/// alone, so once the dataset moves, `recorded` keeps the record and `named` does not. `stale` and
/// `revalidate` hold only there, and `settled` then depends on `ca`, which it does not otherwise.
const INVALIDATING: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1}\n\
    artifacts: {dataset: {}, explanation: {}}\n\
    evidence_kinds: {a: {}, b: {}}\n\
    claims: {ca: {true_when: {evidence: {kind: a}}}, named: {true_when: {evidence: {kind: b}}}, \
    recorded: {true_when: {evidence: {kind: b}}}}\n\
    actions:\n\
    \x20\x20observe: {may_produce: [{evidence: a}, {evidence: b}]}\n\
    \x20\x20revalidate: {precondition: {all: [{claim: recorded}, {claim: named, is: unknown}]}}\n\
    outcomes:\n\
    \x20\x20stale: {requires: {all: [{claim: recorded}, {claim: named, is: unknown}]}}\n\
    \x20\x20settled: {requires: {all: [{claim: recorded}, {any: [{claim: named}, {claim: ca}]}]}}\n\
    invalidation: {dataset.revised: {upstream: dataset, invalidates: [named]}}\n";

/// The probe that found the gap: with the dataset moved, `canon evaluate` finds `stale`
/// legitimate and `revalidate` approval-free, so `canon check` must not report either as
/// unreachable. The rule gives `b`, which `named` reads, the upstream `dataset`, and `a`, which no
/// claim the rule invalidates reads, none: `a` has 2 values, `b` 4 (no record, one observed at the
/// current dataset revision, one observed before it moved, both).
#[test]
fn a_state_with_the_upstream_moved_is_checked() {
    assert_eq!(
        check(&compiled(INVALIDATING), None)
            .expect("checked")
            .to_string(),
        "checked: protocol `p` revision 1: 8 states, 0 properties, 0 findings\n"
    );
}

/// `settled` is independent of `ca` in every state without a moved upstream: with `b` present
/// `named` holds it, with `b` absent `recorded` blocks it. With the dataset moved, `named` is
/// unknown while `recorded` holds, and `ca` decides it: the counterexample names the record observed
/// before the move.
#[test]
fn a_property_sees_the_states_with_the_upstream_moved() {
    let body = format!(
        "{HEAD}properties: {{p: {{subject: {{outcome: settled}}, independent_of: {{claim: ca}}}}}}\n"
    );
    let checked = check(&compiled(INVALIDATING), Some(&properties(&body))).expect("checked");
    assert_eq!(
        checked.to_string(),
        "property-failed: property `p`: outcome `settled` is blocked in state {evidence `b` \
         observed before upstream `dataset` moved} and legitimate in state {evidence `a`, \
         evidence `b` observed before upstream `dataset` moved}\n\
         checked: protocol `p` revision 1: 8 states, 1 property, 1 finding\n"
    );
}

/// The upstream counts toward the bound and is named in the refusal: `k` has 16 classes, so 65536
/// values without an upstream artifact, and 2^(16 * 2) with one, each class present observed
/// before the move, after it, both or neither. The listed numbers multiply to the total.
#[test]
fn a_rule_dimension_counts_toward_the_bound_and_is_named() {
    let matches: Vec<String> = (0..15)
        .map(|n| format!("{{evidence: {{kind: k, result: r{n}}}}}"))
        .collect();
    let source = format!(
        "{}invalidation: {{r: {{upstream: a, invalidates: [c]}}}}\n",
        PROTOCOL.replace(
            "claims: {c: {true_when: {evidence: {kind: k}}}}",
            &format!(
                "claims: {{c: {{true_when: {{any: [{}]}}}}}}",
                matches.join(", ")
            ),
        )
    );
    let refusal = check(&compiled(&source), None).expect_err("refused");
    assert_eq!(
        (refusal.code(), refusal.to_string().as_str()),
        (
            "state-space-bound",
            "protocol `p` revision 2: outcome `done` reads 4294967296 states, more than the bound \
             of 65536: evidence kind `k` with upstream `a` 4294967296"
        )
    );
    let without_rule = check(
        &compiled(&source.replace("invalidation: {r: {upstream: a, invalidates: [c]}}\n", "")),
        None,
    )
    .expect("65536 states are within the bound");
    assert_eq!(without_rule.states, 65536);
}

// ---------------------------------------------------------------------------------------------
// Order forms (story:evidence-order-predicate).

/// Order forms over two artifacts: `k` read with and without a subject, so it has a dimension bound
/// to `a` and an unbound one about `b`; `m` read only about `a`, so one dimension bound to `a`.
/// Three dimensions an order form reads, so 13 orders of first observations.
const ORDERED: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1}\n\
    artifacts: {a: {}, b: {}}\n\
    evidence_kinds: {k: {}, m: {}}\n\
    claims:\n\
    \x20\x20first_k: {true_when: {first: {kind: k, result: pass}}}\n\
    \x20\x20first_k_a: {true_when: {first: {kind: k, result: pass, subject: a}}}\n\
    \x20\x20k_before_m_a: {true_when: {before: {first: k, then: m, subject: a}}}\n\
    \x20\x20m_before_k_a: {true_when: {before: {first: m, then: k, subject: a}}}\n\
    \x20\x20pass_k: {true_when: {evidence: {kind: k, result: pass}}}\n\
    \x20\x20pass_k_a: {true_when: {evidence: {kind: k, result: pass, subject: a}}}\n\
    \x20\x20some_m_a: {true_when: {evidence: {kind: m, subject: a}}}\n\
    actions: {act: {may_produce: [{evidence: k}, {evidence: m}]}}\n";

/// The value of every claim of `ir`, in claim order, for `evidence` about the current revision `r`
/// of every artifact.
fn claim_values(ir: &Ir, evidence: &[crate::model::EvidenceRecord]) -> Vec<crate::model::Truth> {
    claim_values_with(ir, evidence, Declarations::default())
}

/// [`claim_values`] for a case that lists `independence`.
fn claim_values_with(
    ir: &Ir,
    evidence: &[crate::model::EvidenceRecord],
    independence: Declarations<crate::model::Principal, Vec<crate::model::IndependenceDimensionId>>,
) -> Vec<crate::model::Truth> {
    let case = Case {
        format: CASE_FORMAT.to_owned(),
        independence,
        id: CaseId::new("check"),
        protocol: ir.protocol.id.clone(),
        artifacts: Declarations::new(
            ir.artifacts
                .keys()
                .map(|id| {
                    let current = CaseArtifact {
                        revision: Revision::new(REVISION),
                        descends_from: None,
                    };
                    (id.clone(), current)
                })
                .collect(),
        ),
        termination: None,
        revision: None,
        inputs: Declarations::default(),
    };
    eval::evaluate(ir, &case, evidence)
        .expect("evaluates")
        .claims
        .iter()
        .map(|(_, entry)| entry.value)
        .collect()
}

/// The threads [`par_map`] runs on: the `threads-required` that `.config/nextest.toml` gives the
/// tests of the `slow` lane that use it (story:fast-test-step).
const WORKERS: usize = 4;

/// `f` of every index below `n`, in index order, computed on [`WORKERS`] threads: the same cases
/// and the same assertions as a sequential map, in less wall time. A panic in `f` is the caller's.
fn par_map<R: Send>(n: usize, f: impl Fn(usize) -> R + Sync) -> Vec<R> {
    let chunk = n.div_ceil(WORKERS).max(1);
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..n)
            .step_by(chunk)
            .map(|start| {
                let f = &f;
                scope.spawn(move || (start..(start + chunk).min(n)).map(f).collect::<Vec<R>>())
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|worker| {
                worker
                    .join()
                    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
            })
            .collect()
    })
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

/// Soundness of the space for order forms: every combination of claim values that an evidence set
/// gives — records of either kind, about either artifact, at the current or a superseded revision,
/// with any result, at one of three instants or none — is the combination of some state. The
/// claims hold every order form jointly with evidence matches over the same records.
#[test]
fn every_value_of_every_order_form_is_reached_jointly_with_the_evidence_matches() {
    let ir = compiled(ORDERED);
    let space = Space::new(&ir);
    let size = space.size().expect("fits");
    assert_eq!(size, 31200, "{}", space.dimensions());
    let reached: BTreeSet<Vec<crate::model::Truth>> = par_map(size as usize, |index| {
        claim_values(&ir, &space.evidence(&space.state(index as u128)))
    })
    .into_iter()
    .collect();
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let instants = [
        Some("2026-10-01T09:00:00Z"),
        Some("2026-10-01T09:00:01Z"),
        Some("2026-10-01T09:00:02Z"),
        None,
    ];
    let mut histories = Vec::new();
    for _ in 0..20_000 {
        let records: Vec<crate::model::EvidenceRecord> = (0..rng.below(6))
            .map(|n| {
                let kind = ["k", "m"][rng.below(2)];
                let result = [Some("pass"), Some("fail"), None][rng.below(3)];
                let at = instants[rng.below(4)];
                crate::model::EvidenceRecord {
                    format: crate::model::EVIDENCE_FORMAT.to_owned(),
                    producer: None,
                    id: crate::model::EvidenceId::new(format!("h{n}")),
                    kind: EvidenceKindId::new(kind),
                    result: result.map(str::to_owned),
                    subject: crate::model::ArtifactId::new(["a", "b"][rng.below(2)]),
                    subject_revision: Revision::new(["r", "r0"][rng.below(2)]),
                    observed_at: at.map(crate::model::Instant::new),
                    upstream_revisions: Declarations::default(),
                    child_case: None,
                    item: None,
                }
            })
            .collect();
        histories.push(records);
    }
    let checked = par_map(histories.len(), |at| {
        let records = &histories[at];
        let values = claim_values(&ir, records);
        assert!(
            reached.contains(&values),
            "no state gives {values:?}, which {records:?} gives"
        );
    });
    assert_eq!(checked.len(), 20_000);
    // The space is not trivially everything: `first_k_a` true and `first_k` false needs `b`'s
    // first record before `a`'s, so both orders are reached.
    assert!(reached.len() > 100, "{}", reached.len());
}

/// A protocol with a `descended` form (story:revision-ordering) over one kind, beside evidence
/// matches on its two results and a `first` on another kind.
const DESCENDED: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1}\n\
    artifacts: {a: {}}\n\
    evidence_kinds: {k: {}, m: {}}\n\
    claims:\n\
    \x20\x20red_then_green: {true_when: {descended: {first: {kind: k, result: failed}, \
    then: {kind: k, result: passed}}}}\n\
    \x20\x20passes: {true_when: {evidence: {kind: k, result: passed}}}\n\
    \x20\x20fails: {true_when: {evidence: {kind: k, result: failed}}}\n\
    \x20\x20first_m: {true_when: {first: {kind: m, result: passed}}}\n\
    actions: {run: {may_produce: [{evidence: k}, {evidence: m}]}}\n\
    outcomes: {done: {requires: {all: [{claim: red_then_green}, {claim: passes}]}}}\n";

/// The value of every claim of `ir`, in claim order, for `evidence` and a case whose every artifact
/// is at `r` and descends from `ancestors`.
fn claim_values_descending(
    ir: &Ir,
    evidence: &[crate::model::EvidenceRecord],
    ancestors: Option<Vec<Revision>>,
) -> Vec<crate::model::Truth> {
    let case = Case {
        format: CASE_FORMAT.to_owned(),
        independence: Declarations::default(),
        id: CaseId::new("check"),
        protocol: ir.protocol.id.clone(),
        artifacts: Declarations::new(
            ir.artifacts
                .keys()
                .map(|id| {
                    let current = CaseArtifact {
                        revision: Revision::new(REVISION),
                        descends_from: ancestors.clone(),
                    };
                    (id.clone(), current)
                })
                .collect(),
        ),
        termination: None,
        revision: None,
        inputs: Declarations::default(),
    };
    eval::evaluate(ir, &case, evidence)
        .expect("evaluates")
        .claims
        .iter()
        .map(|(_, entry)| entry.value)
        .collect()
}

/// Soundness of the space for `descended` (story:revision-ordering): the space's case says every
/// artifact descends from `r0`, and every combination of claim values an evidence set and a
/// snapshot give, records of either kind, with any result, at the current revision, at `r0` or
/// at `r1`, naming the space's item, another or none, at one of two instants or none, under a
/// `descends_from` absent, empty, `[r0]` or `[r1, r0]`, is the combination of some state; and
/// the form takes each of its three values in some state.
#[test]
fn every_value_of_descended_is_reached_jointly_with_the_evidence_matches() {
    let ir = compiled(DESCENDED);
    let space = Space::new(&ir);
    assert_eq!(space.descends_from(), Some(vec![Revision::new("r0")]));
    let size = space.size().expect("fits");
    let reached: BTreeSet<Vec<crate::model::Truth>> = (0..size)
        .map(|index| {
            let evidence = space.evidence(&space.state(index));
            claim_values_descending(&ir, &evidence, space.descends_from())
        })
        .collect();
    let forms: BTreeSet<crate::model::Truth> = reached.iter().map(|values| values[3]).collect();
    assert_eq!(forms.len(), 3, "{reached:?}");
    let mut rng = Rng(0x5EC0_4D5A_F7E2_0005);
    let instants = [
        Some("2026-10-01T09:00:00Z"),
        Some("2026-10-01T09:00:01Z"),
        None,
    ];
    for _ in 0..20_000 {
        let records: Vec<crate::model::EvidenceRecord> = (0..rng.below(6))
            .map(|n| crate::model::EvidenceRecord {
                format: crate::model::EVIDENCE_FORMAT.to_owned(),
                producer: None,
                id: crate::model::EvidenceId::new(format!("h{n}")),
                kind: EvidenceKindId::new(["k", "m"][rng.below(2)]),
                result: [Some("passed"), Some("failed"), None][rng.below(3)].map(str::to_owned),
                subject: crate::model::ArtifactId::new("a"),
                subject_revision: Revision::new(["r", "r0", "r1"][rng.below(3)]),
                observed_at: instants[rng.below(3)].map(crate::model::Instant::new),
                upstream_revisions: Declarations::default(),
                child_case: None,
                item: [Some("item"), Some("other"), None][rng.below(3)]
                    .map(crate::model::ItemId::new),
            })
            .collect();
        let ancestors = [
            None,
            Some(vec![]),
            Some(vec![Revision::new("r0")]),
            Some(vec![Revision::new("r1"), Revision::new("r0")]),
        ][rng.below(4)]
        .clone();
        let values = claim_values_descending(&ir, &records, ancestors.clone());
        assert!(
            reached.contains(&values),
            "no state gives {values:?}, which {records:?} under {ancestors:?} gives"
        );
    }
}

/// `canon check` of a protocol with a `descended` form reaches the outcome that needs it: a
/// failure about `r0` and a pass about the current revision, one item.
#[test]
fn descended_is_checked_over_the_history_dimension() {
    let ir = compiled(DESCENDED);
    let report = check(&ir, None).expect("checked");
    assert!(
        !report.to_string().contains("unreachable-outcome"),
        "{report}"
    );
    assert_slices_agree(&ir, None, DESCENDED);
}

/// A protocol with order forms is checked over its history and order dimensions: an outcome no
/// order of first observations can satisfy is unreachable, and one that needs a red record no
/// longer at the current revision is reached. The bound refusal names the new dimensions.
#[test]
fn order_forms_are_checked_over_the_history_and_order_dimensions() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\n\
        evidence_kinds: {t: {}, d: {}}\n\
        claims:\n\
        \x20\x20red_first: {true_when: {first: {kind: t, result: failed}}}\n\
        \x20\x20green_first: {true_when: {first: {kind: t, result: passed}}}\n\
        \x20\x20passes: {true_when: {evidence: {kind: t, result: passed}}}\n\
        actions: {run: {may_produce: [{evidence: t}, {evidence: d}]}}\n\
        outcomes:\n\
        \x20\x20both: {requires: {all: [{claim: red_first}, {claim: green_first}]}}\n\
        \x20\x20fixed: {requires: {all: [{claim: red_first}, {claim: passes}, \
        {before: {first: t, then: d}}]}}\n";
    let report = check(&compiled(source), None).expect("checked");
    assert_eq!(
        report.to_string(),
        "unreachable-outcome: outcome `both` is legitimate in no state\n\
         checked: protocol `p` revision 1: 1296 states, 0 properties, 1 finding\n"
    );
    let ir = compiled(source);
    assert_slices_agree(&ir, None, source);
    let space = Space::new(&ir);
    // `t` has classes failed, passed and none: 8 values and 9 histories; `d` 2 and 3; 3 orders.
    assert_eq!(
        space.dimensions(),
        "evidence kind `d` 2, evidence kind `t` 8, history of evidence kind `d` 3, history of \
         evidence kind `t` 9, order of first observations 3"
    );
    let wide: Vec<String> = (0..12)
        .map(|n| format!("{{first: {{kind: t, result: r{n}}}}}"))
        .collect();
    let widened = compiled(&source.replace(
        "{claim: red_first}, {claim: passes}",
        &format!("{{any: [{}]}}, {{claim: passes}}", wide.join(", ")),
    ));
    let refusal = check(&widened, None).expect_err("refused");
    // `both`, first in finding order, reads `t` alone, whose classes are the whole space's: the
    // twelve results the widened `fixed` matches too (story:check-subject-slices).
    assert_eq!(
        (refusal.code(), refusal.to_string().as_str()),
        (
            "state-space-bound",
            "protocol `p` revision 1: outcome `both` reads 1073774592 states, more than the bound \
             of 65536: evidence kind `t` 32768, history of evidence kind `t` 32769"
        )
    );
    let space = Space::new(&widened);
    let fixed = space.outcome_slice(&widened, &widened.outcomes[&OutcomeId::new("fixed")]);
    assert_eq!(
        space.slice(&fixed).dimensions(),
        "evidence kind `d` 2, evidence kind `t` 32768, history of evidence kind `d` 3, history of \
         evidence kind `t` 32769, order of first observations 3"
    );
}

/// A state writes its earlier records, an unknown order and the order of first observations after
/// its case inputs, and builds the records it stands for: earlier ones at the superseded revision,
/// every record of an ordered dimension at its rank's instant.
#[test]
fn a_state_writes_and_builds_its_history() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {t: {}, d: {}}\n\
         claims: {c: {true_when: {before: {first: t, then: d}}}, \
         r: {true_when: {first: {kind: t, result: failed}}}}\n",
    );
    let space = Space::new(&ir);
    // Dimensions: d (2), t (4: failed, none), history d (3), history t (5), order (3).
    let find = |wanted: &str| {
        (0..space.size().expect("fits"))
            .map(|index| space.state(index))
            .find(|state| space.render(state) == wanted)
            .unwrap_or_else(|| panic!("no state renders {wanted}"))
    };
    let state = find("{evidence `d`, evidence `t` result `failed` earlier, order `t` < `d`}");
    let records: Vec<(String, Option<String>, String, Option<String>)> = space
        .evidence(&state)
        .into_iter()
        .map(|record| {
            (
                record.kind.as_str().to_owned(),
                record.result,
                record.subject_revision.as_str().to_owned(),
                record.observed_at.map(|at| at.as_str().to_owned()),
            )
        })
        .collect();
    assert_eq!(
        records,
        [
            (
                "d".to_owned(),
                None,
                "r".to_owned(),
                Some("2000-01-01T00:01:00Z".to_owned())
            ),
            (
                "t".to_owned(),
                Some("failed".to_owned()),
                "r0".to_owned(),
                Some("2000-01-01T00:00:00Z".to_owned())
            ),
        ]
    );
    assert_eq!(space.weight(&state), 2);
    let unknown = find("{evidence `t` earlier, observed at no known instant}");
    let records = space.evidence(&unknown);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].observed_at, None);
    assert_eq!(records[0].subject_revision.as_str(), "r0");
    // Erasing `t` empties its dimension and history and leaves `d` alone in the order.
    let erased = space.erase(&state, &BTreeSet::from([1]));
    assert_eq!(space.render(&erased), "{evidence `d`}");
}

/// A protocol whose matches demand independence on `[d]`, `[e]`, `[d, e]` (twice, written in
/// either order) and nothing.
const DEMANDING: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1}\n\
    artifacts: {a: {}}\n\
    independence_dimensions: {d: {}, e: {}}\n\
    evidence_kinds: {k: {}}\n\
    claims:\n\
    \x20\x20on_d: {true_when: {evidence: {kind: k, result: ok, independent_on: [d]}}}\n\
    \x20\x20on_de: {true_when: {evidence: {kind: k, result: ok, independent_on: [d, e]}}}\n\
    \x20\x20on_e: {true_when: {evidence: {kind: k, result: ok, independent_on: [e]}}}\n\
    \x20\x20on_ed_any: {true_when: {evidence: {kind: k, independent_on: [e, d]}}}\n\
    \x20\x20ok: {true_when: {evidence: {kind: k, result: ok}}}\n\
    actions: {act: {may_produce: [{evidence: k}]}}\n";

/// Soundness of the space for independence demands: every combination of claim values an evidence
/// set gives, its records from producers independent on any dimensions or none, or naming no
/// producer, is the combination of some state. Records keep the space's one producer per distinct
/// demand, and the case lists each producer they name on exactly its demand.
#[test]
fn every_combination_of_demands_met_is_reached_by_a_state() {
    let ir = compiled(DEMANDING);
    let space = Space::new(&ir);
    // Classes ok and none; profiles none, [d], [d, e], [e]; per class one value per down-closed
    // set of them: no record, {none}, {none, [d]}, {none, [e]}, {none, [d], [e]} and all four, so
    // 6^2.
    assert_eq!(
        space.dimensions(),
        "evidence kind `k` with producers independent on [`d`], [`d`, `e`], [`e`] 36"
    );
    let size = space.size().expect("fits");
    let reached: BTreeSet<Vec<crate::model::Truth>> = (0..size)
        .map(|index| {
            let evidence = space.evidence(&space.state(index));
            claim_values_with(&ir, &evidence, space.independence(&evidence))
        })
        .collect();
    let listed = |ids: &[&str]| -> Vec<crate::model::IndependenceDimensionId> {
        ids.iter()
            .map(|id| crate::model::IndependenceDimensionId::new(*id))
            .collect()
    };
    let independence = Declarations::new(vec![
        (crate::model::Principal::new("x1"), listed(&["d"])),
        (crate::model::Principal::new("x2"), listed(&["e"])),
        (crate::model::Principal::new("x3"), listed(&["e", "d"])),
        (crate::model::Principal::new("x4"), listed(&[])),
    ]);
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);
    let mut sets = 0;
    for _ in 0..20_000 {
        let records: Vec<crate::model::EvidenceRecord> = (0..rng.below(5))
            .map(|n| crate::model::EvidenceRecord {
                format: crate::model::EVIDENCE_FORMAT.to_owned(),
                producer: [
                    None,
                    Some("x1"),
                    Some("x2"),
                    Some("x3"),
                    Some("x4"),
                    Some("x5"),
                ][rng.below(6)]
                .map(crate::model::Principal::new),
                id: crate::model::EvidenceId::new(format!("h{n}")),
                kind: EvidenceKindId::new("k"),
                result: [Some("ok"), Some("bad"), None][rng.below(3)].map(str::to_owned),
                subject: crate::model::ArtifactId::new("a"),
                subject_revision: Revision::new(["r", "r0"][rng.below(2)]),
                observed_at: None,
                upstream_revisions: Declarations::default(),
                child_case: None,
                item: None,
            })
            .collect();
        let values = claim_values_with(&ir, &records, independence.clone());
        assert!(
            reached.contains(&values),
            "no state gives {values:?}, which {records:?} gives"
        );
        sets += 1;
    }
    assert_eq!(sets, 20_000);
    // Not trivially everything: approvals independent on `d` alone and on `e` alone, and none on
    // both, need two producers no other profile stands for.
    use crate::model::Truth::{True, Unknown};
    assert!(reached.contains(&vec![True, True, Unknown, True, Unknown]));
    // A state names its producers and builds a case listing each on exactly its demand.
    let state = (0..size)
        .map(|index| space.state(index))
        .find(|state| {
            space.render(state)
                == "{evidence `k` result `ok` by a producer independent on `d`, `e`}"
        })
        .expect("a state with an approval independent on both dimensions");
    let evidence = space.evidence(&state);
    assert_eq!(
        evidence[0].producer.as_ref().map(|p| p.as_str()),
        Some("producer-2")
    );
    let case_map: Vec<(String, Vec<String>)> = space
        .independence(&evidence)
        .iter()
        .map(|(producer, dimensions)| {
            (
                producer.as_str().to_owned(),
                dimensions.iter().map(|d| d.as_str().to_owned()).collect(),
            )
        })
        .collect();
    assert_eq!(
        case_map,
        [(
            "producer-2".to_owned(),
            vec!["d".to_owned(), "e".to_owned()]
        )]
    );
    assert_eq!(space.weight(&state), 1);
}

/// A dimension every reader of which demands independence: a record without a producer is read by
/// none, so the space leaves it out, and each class takes one value per down-closed set of the
/// demands `[d]` and `[d, e]` (none, `[d]` alone, both): 3^2 values. Every combination of claim
/// values an evidence set gives is still the combination of some state, records without a producer
/// or from a producer independent on nothing included.
#[test]
fn a_dimension_only_demanding_matches_read_has_no_record_without_a_producer() {
    let ir = compiled(
        "format: protocol/1\n\
         protocol: {id: p, revision: 1}\n\
         artifacts: {a: {}}\n\
         independence_dimensions: {d: {}, e: {}}\n\
         evidence_kinds: {k: {}}\n\
         claims:\n\
         \x20\x20on_d: {true_when: {evidence: {kind: k, result: ok, independent_on: [d]}}}\n\
         \x20\x20on_de: {true_when: {not: {evidence: {kind: k, result: ok, \
         independent_on: [e, d]}}}}\n\
         actions: {act: {may_produce: [{evidence: k}]}}\n",
    );
    let space = Space::new(&ir);
    assert_eq!(
        space.dimensions(),
        "evidence kind `k` with producers independent on [`d`], [`d`, `e`] 9"
    );
    let size = space.size().expect("fits");
    let mut rendered = Vec::new();
    let reached: BTreeSet<Vec<crate::model::Truth>> = (0..size)
        .map(|index| {
            let state = space.state(index);
            rendered.push(space.render(&state));
            let evidence = space.evidence(&state);
            assert!(evidence.iter().all(|record| record.producer.is_some()));
            claim_values_with(&ir, &evidence, space.independence(&evidence))
        })
        .collect();
    assert!(
        rendered.contains(
            &"{evidence `k` result `ok` by a producer independent on `d`, \
          evidence `k` by a producer independent on `d`, `e`}"
                .to_owned()
        )
    );
    let independence = Declarations::new(vec![
        (
            crate::model::Principal::new("x1"),
            vec![crate::model::IndependenceDimensionId::new("d")],
        ),
        (crate::model::Principal::new("x2"), Vec::new()),
        (
            crate::model::Principal::new("x3"),
            vec![
                crate::model::IndependenceDimensionId::new("e"),
                crate::model::IndependenceDimensionId::new("d"),
            ],
        ),
    ]);
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for _ in 0..5_000 {
        let records: Vec<crate::model::EvidenceRecord> = (0..rng.below(5))
            .map(|n| crate::model::EvidenceRecord {
                format: crate::model::EVIDENCE_FORMAT.to_owned(),
                producer: [None, Some("x1"), Some("x2"), Some("x3"), Some("x4")][rng.below(5)]
                    .map(crate::model::Principal::new),
                id: crate::model::EvidenceId::new(format!("h{n}")),
                kind: EvidenceKindId::new("k"),
                result: [Some("ok"), Some("bad"), None][rng.below(3)].map(str::to_owned),
                subject: crate::model::ArtifactId::new("a"),
                subject_revision: Revision::new("r"),
                observed_at: None,
                upstream_revisions: Declarations::default(),
                child_case: None,
                item: None,
            })
            .collect();
        let values = claim_values_with(&ir, &records, independence.clone());
        assert!(
            reached.contains(&values),
            "no state gives {values:?}, which {records:?} gives"
        );
    }
}

/// Soundness of the space of `source` for order forms over random histories: every combination of
/// claim values that records of `kinds`, about one of `artifacts`, at the current or a superseded
/// revision, with one of `results`, at one of three instants or none, give is the combination of
/// some state. Returns the combinations the states reach.
fn every_history_is_reached(
    source: &str,
    kinds: &[&str],
    artifacts: &[&str],
    results: &[Option<&str>],
    seed: u64,
) -> BTreeSet<Vec<crate::model::Truth>> {
    let (reached, unreached) =
        unreached_histories(source, kinds, artifacts, &["r", "r0"], results, seed);
    if let Some((values, records)) = unreached.first() {
        panic!(
            "no state gives {values:?}, which {records:?} gives ({} histories unreached)",
            unreached.len()
        );
    }
    reached
}

/// [`every_history_is_reached`] over records at one of `revisions`, the first the current one:
/// the combinations the states reach, and each random history whose combination no state gives,
/// with that combination.
#[allow(clippy::type_complexity)]
fn unreached_histories(
    source: &str,
    kinds: &[&str],
    artifacts: &[&str],
    revisions: &[&str],
    results: &[Option<&str>],
    seed: u64,
) -> (
    BTreeSet<Vec<crate::model::Truth>>,
    Vec<(Vec<crate::model::Truth>, Vec<crate::model::EvidenceRecord>)>,
) {
    let ir = compiled(source);
    let space = Space::new(&ir);
    let size = space.size().expect("fits");
    assert!(size <= STATE_BOUND, "{size} states: {}", space.dimensions());
    let reached: BTreeSet<Vec<crate::model::Truth>> = par_map(size as usize, |index| {
        claim_values(&ir, &space.evidence(&space.state(index as u128)))
    })
    .into_iter()
    .collect();
    let mut histories = Vec::new();
    let mut rng = Rng(seed);
    let instants = [
        Some("2026-10-01T09:00:00Z"),
        Some("2026-10-01T09:00:01Z"),
        Some("2026-10-01T09:00:02Z"),
        None,
    ];
    for _ in 0..20_000 {
        let records: Vec<crate::model::EvidenceRecord> = (0..rng.below(6))
            .map(|n| crate::model::EvidenceRecord {
                format: crate::model::EVIDENCE_FORMAT.to_owned(),
                producer: None,
                id: crate::model::EvidenceId::new(format!("h{n}")),
                kind: EvidenceKindId::new(kinds[rng.below(kinds.len())]),
                result: results[rng.below(results.len())].map(str::to_owned),
                subject: crate::model::ArtifactId::new(artifacts[rng.below(artifacts.len())]),
                subject_revision: Revision::new(revisions[rng.below(revisions.len())]),
                observed_at: instants[rng.below(4)].map(crate::model::Instant::new),
                upstream_revisions: Declarations::default(),
                child_case: None,
                item: None,
            })
            .collect();
        histories.push(records);
    }
    let values = par_map(histories.len(), |at| claim_values(&ir, &histories[at]));
    let unreached = values
        .into_iter()
        .zip(histories)
        .filter(|(values, _)| !reached.contains(values))
        .collect();
    (reached, unreached)
}

/// Whether claim `at` of the combinations takes each of `true`, `false` and `unknown`.
fn takes_every_value(reached: &BTreeSet<Vec<crate::model::Truth>>, at: usize) -> bool {
    use crate::model::Truth;
    [Truth::True, Truth::False, Truth::Unknown]
        .iter()
        .all(|value| reached.iter().any(|values| values[at] == *value))
}

/// Soundness of the space for `after` (story:evidence-order-after) when its result is a class of
/// the dimension it compares: over two kinds and one kind, jointly with `first` and `before` over
/// the same kinds and evidence matches over the same records, every combination of claim values a
/// history gives is reached, and each `after` takes every value.
#[test]
fn every_value_of_after_is_reached_jointly_with_the_other_forms() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\n\
        evidence_kinds: {k: {}, m: {}}\n\
        claims:\n\
        \x20\x20after_k_m: {true_when: {after: {first: k, then: {kind: m, result: fail}}}}\n\
        \x20\x20after_m_m: {true_when: {after: {first: m, then: {kind: m, result: fail}}}}\n\
        \x20\x20before_k_m: {true_when: {before: {first: k, then: m}}}\n\
        \x20\x20fail_m: {true_when: {evidence: {kind: m, result: fail}}}\n\
        \x20\x20first_m: {true_when: {first: {kind: m, result: fail}}}\n\
        \x20\x20some_k: {true_when: {evidence: {kind: k}}}\n\
        actions: {act: {may_produce: [{evidence: k}, {evidence: m}]}}\n";
    let space_of = compiled(source);
    let space = Space::new(&space_of);
    assert_eq!(
        space.dimensions(),
        "evidence kind `k` 2, evidence kind `m` 4, history of evidence kind `k` 3, history of \
         evidence kind `m` 5, later evidence kind `m` result `fail` 4, order of first \
         observations and later records 13"
    );
    let reached = every_history_is_reached(
        source,
        &["k", "m"],
        &["a"],
        &[Some("fail"), Some("pass"), None],
        0x51A2_7C3B_9D4E_6F80,
    );
    assert!(takes_every_value(&reached, 0), "after_k_m");
    assert!(takes_every_value(&reached, 1), "after_m_m");
}

/// Soundness of the space for `after` when no evidence match or `first` reads its result: the
/// later record alone stands for every record of the result, over two kinds and one.
#[test]
fn every_value_of_after_is_reached_when_its_result_is_no_class() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\n\
        evidence_kinds: {k: {}, m: {}}\n\
        claims:\n\
        \x20\x20after_k_m: {true_when: {after: {first: k, then: {kind: m, result: pass}}}}\n\
        \x20\x20after_m_m: {true_when: {after: {first: m, then: {kind: m, result: pass}}}}\n\
        \x20\x20fail_m: {true_when: {evidence: {kind: m, result: fail}}}\n\
        \x20\x20first_m: {true_when: {first: {kind: m, result: fail}}}\n\
        actions: {act: {may_produce: [{evidence: k}, {evidence: m}]}}\n";
    let reached = every_history_is_reached(
        source,
        &["k", "m"],
        &["a"],
        &[Some("fail"), Some("pass"), None],
        0x0F1E_2D3C_4B5A_6978,
    );
    assert!(takes_every_value(&reached, 0), "after_k_m");
    assert!(takes_every_value(&reached, 1), "after_m_m");
}

/// Soundness of the space for an `after` that names a subject: records about another artifact
/// take no part in it, while an evidence match reads them.
#[test]
fn every_value_of_an_after_naming_a_subject_is_reached() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        artifacts: {a: {}, b: {}}\n\
        evidence_kinds: {k: {}, m: {}}\n\
        claims:\n\
        \x20\x20after_a: {true_when: {after: {first: k, then: {kind: m, result: fail}, subject: a}}}\n\
        \x20\x20fail_m: {true_when: {evidence: {kind: m, result: fail}}}\n\
        actions: {act: {may_produce: [{evidence: k}, {evidence: m}]}}\n";
    let reached = every_history_is_reached(
        source,
        &["k", "m"],
        &["a", "b"],
        &[Some("fail"), Some("pass"), None],
        0x3C6E_F372_FE94_F82B,
    );
    assert!(takes_every_value(&reached, 0), "after_a");
}

/// Soundness of the space of `source` for `since` (story:evidence-order-since): over records at
/// the current revision and two superseded ones, so that two latest anchors can be about different
/// revisions, every combination of claim values a history gives is reached. Names each unreached
/// combination with one history that gives it.
fn every_since_history_is_reached(
    source: &str,
    kinds: &[&str],
    artifacts: &[&str],
    results: &[Option<&str>],
    seed: u64,
) -> BTreeSet<Vec<crate::model::Truth>> {
    let (reached, unreached) =
        unreached_histories(source, kinds, artifacts, &["r", "r0", "r1"], results, seed);
    let mut distinct: BTreeMap<Vec<crate::model::Truth>, Vec<crate::model::EvidenceRecord>> =
        BTreeMap::new();
    for (values, records) in &unreached {
        distinct
            .entry(values.clone())
            .or_insert_with(|| records.clone());
    }
    assert!(
        distinct.is_empty(),
        "{} histories unreached, {} combinations: {:#?}",
        unreached.len(),
        distinct.len(),
        distinct
            .iter()
            .map(|(values, records)| {
                let records: Vec<String> = records
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
                (values.clone(), records)
            })
            .collect::<Vec<_>>()
    );
    reached
}

/// Soundness of the space for a two-kind `since` with evidence matches over the same records:
/// every combination of claim values a history gives is reached, and the `since` takes every value.
#[test]
fn every_value_of_since_is_reached_with_evidence_matches() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\n\
        evidence_kinds: {k: {}, m: {}}\n\
        claims:\n\
        \x20\x20since_k_m: {true_when: {since: {last: k, then: {kind: m, result: fail}}}}\n\
        \x20\x20fail_m: {true_when: {evidence: {kind: m, result: fail}}}\n\
        \x20\x20some_k: {true_when: {evidence: {kind: k}}}\n\
        actions: {act: {may_produce: [{evidence: k}, {evidence: m}]}}\n";
    let reached = every_since_history_is_reached(
        source,
        &["k", "m"],
        &["a"],
        &[Some("fail"), Some("pass"), None],
        0x2545_F491_4F6C_DD1D,
    );
    assert!(takes_every_value(&reached, 1), "since_k_m");
}

/// Soundness of the space for `since` jointly with `after`, `before` and `first` over the same
/// kinds: every combination of claim values a history gives is reached.
#[test]
fn every_value_of_since_is_reached_jointly_with_the_other_forms() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\n\
        evidence_kinds: {k: {}, m: {}}\n\
        claims:\n\
        \x20\x20since_k_m: {true_when: {since: {last: k, then: {kind: m, result: fail}}}}\n\
        \x20\x20after_k_m: {true_when: {after: {first: k, then: {kind: m, result: fail}}}}\n\
        \x20\x20before_k_m: {true_when: {before: {first: k, then: m}}}\n\
        \x20\x20first_m: {true_when: {first: {kind: m, result: fail}}}\n\
        actions: {act: {may_produce: [{evidence: k}, {evidence: m}]}}\n";
    let reached = every_since_history_is_reached(
        source,
        &["k", "m"],
        &["a"],
        &[Some("fail"), Some("pass"), None],
        0x9E37_79B9_7F4A_7C15,
    );
    assert!(takes_every_value(&reached, 3), "since_k_m");
}

/// Soundness of the space for a one-kind `since` jointly with `after`, `first` and an evidence
/// match over its kind. Such a `since` is never `true`, by the story's values: a latest record of
/// the kind is one of its own anchors, and no record of the kind is observed after it; it takes
/// `false` and `unknown`.
#[test]
fn every_value_of_a_one_kind_since_is_reached_jointly_with_the_other_forms() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\n\
        evidence_kinds: {m: {}}\n\
        claims:\n\
        \x20\x20since_m_m: {true_when: {since: {last: m, then: {kind: m, result: fail}}}}\n\
        \x20\x20after_m_m: {true_when: {after: {first: m, then: {kind: m, result: fail}}}}\n\
        \x20\x20first_m: {true_when: {first: {kind: m, result: fail}}}\n\
        \x20\x20fail_m: {true_when: {evidence: {kind: m, result: fail}}}\n\
        actions: {act: {may_produce: [{evidence: m}]}}\n";
    let reached = every_since_history_is_reached(
        source,
        &["m"],
        &["a"],
        &[Some("fail"), Some("pass"), None],
        0x6A09_E667_F3BC_C908,
    );
    use crate::model::Truth;
    assert!(reached.iter().all(|values| values[3] != Truth::True));
    assert!(reached.iter().any(|values| values[3] == Truth::False));
    assert!(reached.iter().any(|values| values[3] == Truth::Unknown));
}

/// Soundness of the space for `since` when its result is no class of the dimension it compares,
/// over two kinds and over one.
#[test]
fn every_value_of_since_is_reached_when_its_result_is_no_class() {
    let two_kinds = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\n\
        evidence_kinds: {k: {}, m: {}}\n\
        claims:\n\
        \x20\x20since_k_m: {true_when: {since: {last: k, then: {kind: m, result: pass}}}}\n\
        \x20\x20fail_m: {true_when: {evidence: {kind: m, result: fail}}}\n\
        actions: {act: {may_produce: [{evidence: k}, {evidence: m}]}}\n";
    let reached = every_since_history_is_reached(
        two_kinds,
        &["k", "m"],
        &["a"],
        &[Some("fail"), Some("pass"), None],
        0xBB67_AE85_84CA_A73B,
    );
    assert!(takes_every_value(&reached, 1), "since_k_m");
    let one_kind = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\n\
        evidence_kinds: {m: {}}\n\
        claims:\n\
        \x20\x20since_m_m: {true_when: {since: {last: m, then: {kind: m, result: pass}}}}\n\
        \x20\x20fail_m: {true_when: {evidence: {kind: m, result: fail}}}\n\
        actions: {act: {may_produce: [{evidence: m}]}}\n";
    every_since_history_is_reached(
        one_kind,
        &["m"],
        &["a"],
        &[Some("fail"), Some("pass"), None],
        0x510E_527F_ADE6_82D1,
    );
}

/// Soundness of the space for a `since` that names a subject: records about another artifact take
/// no part in it, while an evidence match reads them.
#[test]
fn every_value_of_a_since_naming_a_subject_is_reached() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        artifacts: {a: {}, b: {}}\n\
        evidence_kinds: {k: {}, m: {}}\n\
        claims:\n\
        \x20\x20since_a: {true_when: {since: {last: k, then: {kind: m, result: fail}, subject: a}}}\n\
        \x20\x20some_k: {true_when: {evidence: {kind: k}}}\n\
        actions: {act: {may_produce: [{evidence: k}, {evidence: m}]}}\n";
    let reached = every_since_history_is_reached(
        source,
        &["k", "m"],
        &["a", "b"],
        &[Some("fail"), Some("pass"), None],
        0x3C6E_F372_FE94_F82B,
    );
    assert!(takes_every_value(&reached, 0), "since_a");
}

/// A protocol with a `since` writes the last record of the kind it reads as `last` and builds it,
/// at the superseded revision: a thing in the order of its own, at or forty-five seconds past the
/// instant of its dimension's rank or of the later record the `since` compares, so it can stand
/// before, with or after the records the `since` judges by, and after its dimension's current
/// records. Erasing the compared kind moves a last record placed against its later record to a
/// place that stands the same against every record that stays; erasing the last kind takes it.
#[test]
fn a_state_writes_and_builds_its_last_records() {
    use crate::model::Truth;
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {t: {}, d: {}}\n\
         claims: {c: {true_when: {since: {last: d, then: {kind: t, result: failed}}}}}\n",
    );
    let space = Space::new(&ir);
    assert_eq!(
        space.dimensions(),
        "evidence kind `d` 2, evidence kind `t` 2, history of evidence kind `d` 3, history of \
         evidence kind `t` 3, later evidence kind `t` result `failed` 3, last evidence kind `d` 5, \
         order of first observations and later records 13"
    );
    let find = |wanted: &str| {
        (0..space.size().expect("fits"))
            .map(|index| space.state(index))
            .find(|state| space.render(state) == wanted)
            .unwrap_or_else(|| panic!("no state renders {wanted}"))
    };
    let built = |state: &State| -> Vec<(String, Option<String>, String, Option<String>)> {
        space
            .evidence(state)
            .into_iter()
            .map(|record| {
                (
                    record.kind.as_str().to_owned(),
                    record.result,
                    record.subject_revision.as_str().to_owned(),
                    record.observed_at.map(|at| at.as_str().to_owned()),
                )
            })
            .collect()
    };
    let record = |kind: &str, result: Option<&str>, revision: &str, at: &str| {
        (
            kind.to_owned(),
            result.map(str::to_owned),
            revision.to_owned(),
            Some(at.to_owned()),
        )
    };
    let red_after = find(
        "{evidence `t` result `failed` later, evidence `d` last, order `d` last < `t` result \
         `failed` later}",
    );
    assert_eq!(
        built(&red_after),
        [
            record("t", Some("failed"), "r0", "2000-01-01T00:01:00Z"),
            record("d", None, "r0", "2000-01-01T00:00:00Z"),
        ]
    );
    assert_eq!(
        claim_values(&ir, &space.evidence(&red_after)),
        [Truth::True]
    );
    assert_eq!(space.weight(&red_after), 2);
    let red_before = find(
        "{evidence `t` result `failed` later, evidence `d` last, order `t` result `failed` later \
         < `d` last}",
    );
    assert_eq!(
        built(&red_before),
        [
            record("t", Some("failed"), "r0", "2000-01-01T00:00:00Z"),
            record("d", None, "r0", "2000-01-01T00:00:45Z"),
        ]
    );
    assert_eq!(
        claim_values(&ir, &space.evidence(&red_before)),
        [Truth::False]
    );
    let tied = find(
        "{evidence `t` result `failed` later, evidence `d` last, order `t` result `failed` later \
         = `d` last}",
    );
    assert_eq!(
        built(&tied),
        [
            record("t", Some("failed"), "r0", "2000-01-01T00:00:00Z"),
            record("d", None, "r0", "2000-01-01T00:00:00Z"),
        ]
    );
    assert_eq!(claim_values(&ir, &space.evidence(&tied)), [Truth::Unknown]);
    let with_current = find("{evidence `d`, evidence `d` last, order `d` = `d` last}");
    assert_eq!(
        built(&with_current),
        [
            record("d", None, "r", "2000-01-01T00:00:00Z"),
            record("d", None, "r0", "2000-01-01T00:00:00Z"),
        ]
    );
    // A change at the current revision, then a failed run and a change at a superseded one: the
    // latest change is about the older revision, and the run before it, at that revision, is not
    // since it.
    let superseded = find(
        "{evidence `d`, evidence `t` result `failed` later, evidence `d` last, order `d` < `t` \
         result `failed` later < `d` last}",
    );
    assert_eq!(
        built(&superseded),
        [
            record("d", None, "r", "2000-01-01T00:00:00Z"),
            record("t", Some("failed"), "r0", "2000-01-01T00:01:00Z"),
            record("d", None, "r0", "2000-01-01T00:01:45Z"),
        ]
    );
    assert_eq!(
        claim_values(&ir, &space.evidence(&superseded)),
        [Truth::False]
    );
    let erased = space.erase(&superseded, &BTreeSet::from([1]));
    assert_eq!(
        space.render(&erased),
        "{evidence `d`, evidence `d` last, order `d` < `d` last}"
    );
    assert_eq!(
        built(&erased),
        [
            record("d", None, "r", "2000-01-01T00:01:00Z"),
            record("d", None, "r0", "2000-01-01T00:01:45Z"),
        ]
    );
    let erased = space.erase(&red_after, &BTreeSet::from([0]));
    assert_eq!(
        space.render(&erased),
        "{evidence `t` result `failed` later}"
    );
}

/// Erasing dimensions from a state of a protocol with `since` evaluates every claim as the state's
/// own records without the erased dimensions' records do: a last record placed against a thing
/// that goes moves to a place that stands the same against every record that stays, or that thing
/// keeps its rank. Over every third state of a protocol with a `since` and a `first` over its
/// `last` kind, erasing each dimension and every dimension a claim does not read. Leaving a last
/// record where its place was when its compared kind goes puts it before the current records it
/// was observed with, and changes `first_k_x`.
#[test]
fn erasing_from_a_since_state_keeps_every_claim_value() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {}, m: {}}\n\
         claims:\n\
         \x20\x20since_k_m: {true_when: {since: {last: k, then: {kind: m, result: fail}}}}\n\
         \x20\x20first_k_x: {true_when: {first: {kind: k, result: x}}}\n",
    );
    let space = Space::new(&ir);
    let size = space.size().expect("fits");
    assert!(size <= STATE_BOUND, "{size} states: {}", space.dimensions());
    let about = space.evidence_dimensions();
    let mut sets: Vec<BTreeSet<usize>> = (0..about.len()).map(|at| BTreeSet::from([at])).collect();
    let all: BTreeSet<usize> = (0..about.len()).collect();
    for claim in ir.claims.values() {
        let read: BTreeSet<usize> = space
            .dimensions_read(&ir, &claim.true_when)
            .into_keys()
            .collect();
        sets.push(all.difference(&read).copied().collect());
    }
    let mut wrong = Vec::new();
    for index in (0..size).step_by(3) {
        let state = space.state(index);
        let records = space.evidence(&state);
        for set in &sets {
            let kept: Vec<crate::model::EvidenceRecord> = records
                .iter()
                .filter(|record| {
                    !set.iter().any(|at| {
                        about[*at].is_some_and(|(kind, subject)| {
                            *kind == record.kind && *subject == record.subject
                        })
                    })
                })
                .cloned()
                .collect();
            let erased = space.erase(&state, set);
            let want = claim_values(&ir, &kept);
            let got = claim_values(&ir, &space.evidence(&erased));
            if want != got && wrong.len() < 5 {
                wrong.push(format!(
                    "erasing {set:?} from {} gives {}: {want:?}, not {got:?}",
                    space.render(&state),
                    space.render(&erased)
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// Soundness of the space for a `since` jointly with an `after` over the same kinds and evidence
/// matches over the same records, the adversary's protocol for story:evidence-order-since: a latest
/// change at a superseded revision with a failed run at another and no current failure, and a
/// failed run between two changes at one superseded revision, are reached with every other value.
#[test]
fn every_value_of_since_is_reached_jointly_with_after_and_matches() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\n\
        evidence_kinds: {k: {}, m: {}}\n\
        claims:\n\
        \x20\x20since_k_m: {true_when: {since: {last: k, then: {kind: m, result: fail}}}}\n\
        \x20\x20after_k_m: {true_when: {after: {first: k, then: {kind: m, result: fail}}}}\n\
        \x20\x20fail_m: {true_when: {evidence: {kind: m, result: fail}}}\n\
        \x20\x20some_k: {true_when: {evidence: {kind: k}}}\n\
        actions: {act: {may_produce: [{evidence: k}, {evidence: m}]}}\n";
    let reached = every_since_history_is_reached(
        source,
        &["k", "m"],
        &["a"],
        &[Some("fail"), Some("pass"), None],
        0x1F83_D9AB_FB41_BD6B,
    );
    assert!(takes_every_value(&reached, 0), "since_k_m");
    assert!(takes_every_value(&reached, 1), "after_k_m");
}

/// A protocol with an `after` writes its later records and the order they take part in, and builds
/// them: a later record at the superseded revision with its result at its rank's instant, current
/// records after earlier ones thirty seconds past their rank's instant, and the current records of
/// the result at the rank's instant when they are observed first. A protocol without one is
/// unchanged (see [`a_state_writes_and_builds_its_history`]).
#[test]
fn a_state_writes_and_builds_its_later_records() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {t: {}, d: {}}\n\
         claims: {c: {true_when: {after: {first: d, then: {kind: t, result: failed}}}}, \
         r: {true_when: {first: {kind: t, result: failed}}}}\n",
    );
    let space = Space::new(&ir);
    let find = |wanted: &str| {
        (0..space.size().expect("fits"))
            .map(|index| space.state(index))
            .find(|state| space.render(state) == wanted)
            .unwrap_or_else(|| panic!("no state renders {wanted}"))
    };
    let built = |state: &State| -> Vec<(String, Option<String>, String, Option<String>)> {
        space
            .evidence(state)
            .into_iter()
            .map(|record| {
                (
                    record.kind.as_str().to_owned(),
                    record.result,
                    record.subject_revision.as_str().to_owned(),
                    record.observed_at.map(|at| at.as_str().to_owned()),
                )
            })
            .collect()
    };
    let record = |kind: &str, result: Option<&str>, revision: &str, at: Option<&str>| {
        (
            kind.to_owned(),
            result.map(str::to_owned),
            revision.to_owned(),
            at.map(str::to_owned),
        )
    };
    let later = find(
        "{evidence `d`, evidence `t` result `failed` later, order `d` < `t` result `failed` later}",
    );
    assert_eq!(
        built(&later),
        [
            record("d", None, "r", Some("2000-01-01T00:00:00Z")),
            record("t", Some("failed"), "r0", Some("2000-01-01T00:01:00Z")),
        ]
    );
    assert_eq!(space.weight(&later), 2);
    let behind =
        find("{evidence `d`, evidence `t` result `failed`, evidence `t` earlier, order `t` < `d`}");
    assert_eq!(
        built(&behind),
        [
            record("d", None, "r", Some("2000-01-01T00:01:00Z")),
            record("t", Some("failed"), "r", Some("2000-01-01T00:00:30Z")),
            record("t", None, "r0", Some("2000-01-01T00:00:00Z")),
        ]
    );
    let first = find(
        "{evidence `d`, evidence `t` result `failed`, evidence `t` result `failed` observed \
         first, order `d` = `t`}",
    );
    assert_eq!(
        built(&first),
        [
            record("d", None, "r", Some("2000-01-01T00:00:00Z")),
            record("t", Some("failed"), "r", Some("2000-01-01T00:00:00Z")),
        ]
    );
    let unknown =
        find("{evidence `t` result `failed`, evidence `t` earlier, observed at no known instant}");
    assert_eq!(
        built(&unknown),
        [
            record("t", Some("failed"), "r", Some("2000-01-01T00:00:00Z")),
            record("t", None, "r0", None),
        ]
    );
    let unplaced = find("{evidence `t` result `failed` later, observed at no known instant}");
    assert_eq!(built(&unplaced), [record("t", Some("failed"), "r0", None)]);
    // Erasing `t` empties its dimension, its history and its later record, which stand first.
    let erased = space.erase(&later, &BTreeSet::from([1]));
    assert_eq!(space.render(&erased), "{evidence `d`}");
    assert_eq!(
        space.dimensions(),
        "evidence kind `d` 2, evidence kind `t` 4, history of evidence kind `d` 3, history of \
         evidence kind `t` 5, later evidence kind `t` result `failed` 4, order of first \
         observations and later records 13"
    );
}

/// `canon check` over an `after`: an outcome that needs a failed run after the tests changed and
/// no such run at once is unreachable, while one that needs either is reached. The `after` counts
/// for `authority-bypass` under a `not` too: `run` comes only from an action that requires
/// authority, so `not_red` is examined, and it holds on a change whose instant is unknown and no
/// run, which taking the change away blocks. `red` needs a run, so it rests on authority.
#[test]
fn after_is_checked_over_later_records_and_counts_for_authority_bypass() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\n\
        evidence_kinds: {change: {}, run: {}}\n\
        claims:\n\
        \x20\x20red_after: {true_when: {after: {first: change, then: {kind: run, result: failed}}}}\n\
        \x20\x20only_before: {true_when: {not: {after: {first: change, then: {kind: run, result: failed}}}}}\n\
        actions:\n\
        \x20\x20edit: {may_produce: [{evidence: change}]}\n\
        \x20\x20test: {requires: [{capability: ci}], may_produce: [{evidence: run}]}\n\
        outcomes:\n\
        \x20\x20both: {requires: {all: [{claim: red_after}, {claim: only_before}]}}\n\
        \x20\x20red: {requires: {claim: red_after}}\n\
        \x20\x20not_red: {requires: {claim: only_before}}\n";
    let report = check(&compiled(source), None).expect("checked");
    assert_eq!(
        report.to_string(),
        "unreachable-outcome: outcome `both` is legitimate in no state\n\
         authority-bypass: outcome `not_red` is legitimate without authority in state \
         {evidence `change` earlier, observed at no known instant}\n\
         checked: protocol `p` revision 1: 4212 states, 0 properties, 2 findings\n"
    );
    let ir = compiled(source);
    assert_slices_agree(&ir, None, source);
    let space = Space::new(&ir);
    assert_eq!(
        space.dimensions(),
        "evidence kind `change` 2, evidence kind `run` 2, capability `ci` 3, history of evidence \
         kind `change` 3, history of evidence kind `run` 3, later evidence kind `run` result \
         `failed` 3, order of first observations and later records 13"
    );
}

/// `canon check` over a `since` (story:evidence-order-since): an outcome that needs a failed run
/// since the last change and no such run at once is unreachable, while one that needs either is
/// reached. `clean` holds on a change and no run, which needs no authority; `red` needs a run,
/// which only an action that requires authority produces.
#[test]
fn since_is_checked_over_last_records_and_counts_for_authority_bypass() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\n\
        evidence_kinds: {change: {}, run: {}}\n\
        claims:\n\
        \x20\x20red_since: {true_when: {since: {last: change, then: {kind: run, result: failed}}}}\n\
        \x20\x20not_red: {true_when: {not: {since: {last: change, then: {kind: run, result: failed}}}}}\n\
        actions:\n\
        \x20\x20edit: {may_produce: [{evidence: change}]}\n\
        \x20\x20test: {requires: [{capability: ci}], may_produce: [{evidence: run}]}\n\
        outcomes:\n\
        \x20\x20both: {requires: {all: [{claim: red_since}, {claim: not_red}]}}\n\
        \x20\x20red: {requires: {claim: red_since}}\n\
        \x20\x20clean: {requires: {claim: not_red}}\n";
    let ir = compiled(source);
    // The report and the slices (below) are computed side by side.
    let report = std::thread::scope(|scope| {
        let report = scope.spawn(|| check(&ir, None).expect("checked"));
        // Over slices too: `test`'s capability is in no outcome's slice (story:check-subject-slices).
        assert_slices_agree(&ir, None, source);
        report
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    });
    assert_eq!(
        report.to_string(),
        "unreachable-outcome: outcome `both` is legitimate in no state\n\
         authority-bypass: outcome `clean` is legitimate without authority in state \
         {evidence `change` earlier, observed at no known instant}\n\
         checked: protocol `p` revision 1: 21060 states, 0 properties, 2 findings\n"
    );
}

/// The refusal names the first subject in finding order whose slice is above the bound: outcomes,
/// then actions, then properties. `done` reads `k` (1024 values) and `gate` reads `k` and `m`
/// (262144); the property's slice is its outcome's and its claim's together, so it is refused
/// alone when each part fits.
#[test]
fn the_bound_refusal_names_the_first_subject_above_the_bound() {
    let results = |kind: &str, n: usize| {
        (0..n)
            .map(|i| format!("{{evidence: {{kind: {kind}, result: r{i}}}}}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let source = format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nartifacts: {{a: {{}}}}\n\
         evidence_kinds: {{k: {{}}, m: {{}}}}\n\
         claims: {{x: {{true_when: {{any: [{}]}}}}, y: {{true_when: {{any: [{}]}}}}}}\n\
         actions: {{probe: {{may_produce: [{{evidence: k}}, {{evidence: m}}]}}, \
         gate: {{precondition: {{all: [{{claim: x}}, {{claim: y}}]}}}}}}\n\
         outcomes: {{done: {{requires: {{claim: x}}}}}}\n",
        results("k", 9),
        results("m", 7)
    );
    let refusal = check(&compiled(&source), None).expect_err("refused");
    assert_eq!(
        (refusal.code(), refusal.to_string().as_str()),
        (
            "state-space-bound",
            "protocol `p` revision 1: action `gate` reads 262144 states, more than the bound of \
             65536: evidence kind `k` 1024, evidence kind `m` 256"
        )
    );
    let ungated = source.replace(
        "gate: {precondition: {all: [{claim: x}, {claim: y}]}}",
        "gate: {precondition: {claim: y}}",
    );
    let report = check(&compiled(&ungated), None).expect("every slice fits");
    assert_eq!(
        report.to_string(),
        "checked: protocol `p` revision 1: 262144 states, 0 properties, 0 findings\n"
    );
    let body = format!(
        "{HEAD}properties: {{z: {{subject: {{outcome: done}}, independent_of: {{claim: y}}}}}}\n"
    );
    let refusal = check(&compiled(&ungated), Some(&properties(&body))).expect_err("refused");
    assert_eq!(
        refusal.to_string(),
        "protocol `p` revision 1: property `z` reads 262144 states, more than the bound of \
         65536: evidence kind `k` 1024, evidence kind `m` 256"
    );
}

/// A case input a subject does not read holds the value a witness of the whole product holds: the
/// one whose rendering comes first, `x-y` before `x` (`-` before the closing backtick), not the
/// first declared. An input whose one value's text begins another's (`` a` `` begins `` a`b` ``)
/// stays in the slice: what follows decides, and here `` a`b` `` comes first, because `b` comes
/// before the `}` that follows `` a` ``, so holding `a` would change the witness.
#[test]
fn an_unread_input_holds_the_value_a_whole_product_witness_holds() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        case: {inputs: {s: {values: [x, x-y]}, t: {values: [a, 'a`b']}}}\n\
        artifacts: {a: {}}\n\
        evidence_kinds: {k: {}, m: {}}\n\
        claims: {approved: {true_when: {evidence: {kind: k}}}}\n\
        actions:\n\
        \x20\x20approve: {requires: [{capability: c}], may_produce: [{evidence: k}]}\n\
        \x20\x20self_approve: {may_produce: [{evidence: k}]}\n\
        \x20\x20note: {precondition: {input: s, in: [x]}, may_produce: [{evidence: m}]}\n\
        outcomes: {granted: {requires: {claim: approved}}}\n";
    let ir = compiled(source);
    assert_slices_agree(&ir, None, source);
    assert_eq!(
        check_by(&ir, None, Strategy::Own)
            .expect("checked")
            .to_string(),
        "authority-bypass: outcome `granted` is legitimate without authority in state \
         {evidence `k`, input `s` `x-y`, input `t` `a`b`}\n\
         checked: protocol `p` revision 1: 48 states (input `s` 2 values, input `t` 2 values), 0 \
         properties, 1 finding\n"
    );
    let space = Space::new(&ir);
    let granted = space.outcome_slice(&ir, ir.outcomes.values().next().expect("one outcome"));
    assert_eq!(
        space.slice(&granted).dimensions(),
        "evidence kind `k` 2, input `t` 2"
    );
}

/// Erasing the last record's comparison kind from a slice would leave the record nowhere in the
/// order: a slice that holds a dimension a `since` reads as `last` holds every dimension its last
/// record is placed against, so a subject that reads `k` only through a `first` still decides
/// over `m`, which the `since` anchored on `k` compares.
#[test]
fn a_slice_holds_what_its_last_records_are_placed_against() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
        evidence_kinds: {k: {}, m: {}, n: {}}\n\
        claims:\n\
        \x20\x20since_k_m: {true_when: {since: {last: k, then: {kind: m, result: fail}}}}\n\
        \x20\x20first_k_x: {true_when: {first: {kind: k, result: x}}}\n\
        \x20\x20seen_n: {true_when: {evidence: {kind: n}}}\n\
        actions: {run: {may_produce: [{evidence: k}, {evidence: m}, {evidence: n}]}}\n\
        outcomes:\n\
        \x20\x20early: {requires: {claim: first_k_x}}\n\
        \x20\x20late: {requires: {claim: since_k_m}}\n\
        \x20\x20seen: {requires: {claim: seen_n}}\n";
    let ir = compiled(source);
    let space = Space::new(&ir);
    let early = space.outcome_slice(&ir, &ir.outcomes[&OutcomeId::new("early")]);
    let dimensions = space.slice(&early).dimensions();
    assert!(
        dimensions.contains("evidence kind `m`") && !dimensions.contains("evidence kind `n`"),
        "{dimensions}"
    );
    assert_slices_agree(&ir, None, source);
}

// ---------------------------------------------------------------------------------------------
// Slices (story:check-subject-slices).

/// The three strategies give one result: each subject over its own slice, each over the widest
/// slice holding its own (or the whole product) as `canon check` does, and the whole product as
/// before slices existed. Returns how many states the slices of `ir` hold at most, against the
/// whole product's. The three run side by side.
fn assert_slices_agree(ir: &Ir, properties: Option<&Properties>, source: &str) -> (u128, u128) {
    slices_agree(ir, properties, source, true)
}

/// [`assert_slices_agree`], the three strategies one after another when `side_by_side` is false:
/// for a caller that already runs protocols on [`WORKERS`] threads.
fn slices_agree(
    ir: &Ir,
    properties: Option<&Properties>,
    source: &str,
    side_by_side: bool,
) -> (u128, u128) {
    let (whole, own, chosen) = if side_by_side {
        std::thread::scope(|scope| {
            let own = scope.spawn(|| check_by(ir, properties, Strategy::Own));
            let chosen = scope.spawn(|| check(ir, properties));
            let whole = check_by(ir, properties, Strategy::Whole);
            let joined = |worker: std::thread::ScopedJoinHandle<'_, _>| {
                worker
                    .join()
                    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
            };
            (whole, joined(own), joined(chosen))
        })
    } else {
        (
            check_by(ir, properties, Strategy::Whole),
            check_by(ir, properties, Strategy::Own),
            check(ir, properties),
        )
    };
    assert_eq!(
        own, whole,
        "own slices differ from the whole product:\n{source}"
    );
    assert_eq!(
        chosen, whole,
        "canon check differs from the whole product:\n{source}"
    );
    let space = Space::new(ir);
    let widest = subjects(ir, &space, properties)
        .iter()
        .filter_map(|(_, slice)| space.slice(slice).size())
        .max()
        .unwrap_or(1);
    (widest, space.size().unwrap_or(u128::MAX))
}

/// The protocols a generator writes: two or three evidence kinds about one or two artifacts, an
/// optional case input, up to four claims, three actions and three outcomes over every predicate
/// form, an optional invalidation rule and up to two properties. One in four is small around one
/// order form, whose history and order dimensions make a space large (an `after` or a `since`
/// over one kind keeps it within [`GENERATED_STATES`]), and one in four reads child cases, half of
/// its `children` forms with `revision: current`.
fn generated(rng: &mut Rng) -> (String, Option<String>) {
    let family = rng.below(4);
    let ordered = family == 0;
    let children = family == 1;
    let two = !ordered && rng.below(2) == 0;
    let artifacts: &[&str] = if two { &["a", "b"] } else { &["a"] };
    let mut kinds: Vec<&str> = vec!["k", "m"];
    if !ordered && rng.below(2) == 0 {
        kinds.push("n");
    }
    if children {
        kinds.push("case_outcome");
    }
    let input = match rng.below(if ordered { 8 } else { 4 }) {
        0 => Some(vec!["x", "y"]),
        1 => Some(vec!["x", "x-y", "z"]),
        _ => None,
    };
    let context = Context {
        kinds: kinds.clone(),
        artifacts: artifacts.to_vec(),
        input: input.is_some(),
        children,
        orders: !ordered,
    };
    let depth = if ordered { 1 } else { 2 };
    let claims = 1 + rng.below(if ordered { 3 } else { 4 });
    let mut source = String::from(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
         independence_dimensions: {d: {}, e: {}}\n",
    );
    if let Some(values) = &input {
        source.push_str(&format!(
            "case: {{inputs: {{s: {{values: [{}]}}}}}}\n",
            values.join(", ")
        ));
    }
    let listed = |ids: &[&str]| {
        ids.iter()
            .map(|id| format!("{id}: {{}}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    source.push_str(&format!("artifacts: {{{}}}\n", listed(artifacts)));
    source.push_str(&format!("evidence_kinds: {{{}}}\n", listed(&kinds)));
    source.push_str("claims:\n");
    for claim in 0..claims {
        let predicate = match (claim, ordered, children) {
            (0, true, _) => order_form(rng),
            (0, _, true) => children_form(rng, artifacts),
            _ => context.predicate(rng, depth, claim),
        };
        source.push_str(&format!("  c{claim}: {{true_when: {predicate}}}\n"));
    }
    source.push_str("actions:\n");
    let actions = 1 + rng.below(3);
    for action in 0..actions {
        let mut fields = Vec::new();
        if rng.below(3) != 0 {
            fields.push(format!(
                "precondition: {}",
                context.predicate(rng, depth, claims)
            ));
        }
        match rng.below(if ordered { 8 } else { 4 }) {
            0 => fields.push("requires: [{capability: g0}]".to_owned()),
            1 => fields.push("requires: [{capability: g1}]".to_owned()),
            _ => {}
        }
        let produced: Vec<String> = kinds
            .iter()
            .filter(|kind| **kind != "case_outcome" && rng.below(2) == 0)
            .map(|kind| format!("{{evidence: {kind}}}"))
            .collect();
        if !produced.is_empty() {
            fields.push(format!("may_produce: [{}]", produced.join(", ")));
        }
        if children && action == 0 {
            fields.push("case_open: {protocol: q, subject: a}".to_owned());
        }
        if children && two && action == 1 {
            fields.push("case_open: {protocol: q, subject: b}".to_owned());
        }
        source.push_str(&format!("  a{action}: {{{}}}\n", fields.join(", ")));
    }
    source.push_str("outcomes:\n");
    let outcomes = 1 + rng.below(3);
    for outcome in 0..outcomes {
        let requires = match rng.below(5) {
            0 => format!("{{decision: d{}}}", rng.below(2)),
            _ => context.predicate(rng, depth, claims),
        };
        source.push_str(&format!("  o{outcome}: {{requires: {requires}}}\n"));
    }
    if two && rng.below(3) == 0 {
        source.push_str(&format!(
            "invalidation: {{r: {{upstream: b, invalidates: [c{}]}}}}\n",
            rng.below(claims)
        ));
    }
    let count = rng.below(3);
    let properties = (count > 0).then(|| {
        let mut body = String::from("format: canon-properties/1\nprotocol: p\nproperties:\n");
        for property in 0..count {
            let subject = if rng.below(2) == 0 {
                format!("{{action: a{}}}", rng.below(actions))
            } else {
                format!("{{outcome: o{}}}", rng.below(outcomes))
            };
            body.push_str(&format!(
                "  x{property}: {{subject: {subject}, independent_of: {{claim: c{}}}}}\n",
                rng.below(claims)
            ));
        }
        body
    });
    (source, properties)
}

/// One `children` form listing `accepted` or `p`, half of them with `revision: current`
/// (story:child-outcome-revision) and, independently, half of them with `subject` naming one of
/// `artifacts` (story:child-outcome-subject). One draw decides all three: its remainder by 2 the
/// outcome as a draw below 2 did before the keys existed, its remainder by 4 the revision as a
/// draw below 4 did before `subject` existed, so every later draw, and every protocol a seed
/// generates but for the keys, is what it was.
fn children_form(rng: &mut Rng, artifacts: &[&str]) -> String {
    let draw = rng.below(16);
    let revision = if draw % 4 >= 2 {
        ", revision: current"
    } else {
        ""
    };
    let subject = if (draw / 4) % 2 == 1 {
        format!(", subject: {}", artifacts[(draw / 8) % artifacts.len()])
    } else {
        String::new()
    };
    format!(
        "{{children: {{in: [{}]{subject}{revision}}}}}",
        ["accepted", "p"][draw % 2]
    )
}

/// One order form over `k` and `m`, or over `k` alone for half of the `after` and `since` forms.
fn order_form(rng: &mut Rng) -> String {
    let then = if rng.below(2) == 0 { "k" } else { "m" };
    let result = ["p", "q"][rng.below(2)];
    match rng.below(4) {
        0 => format!("{{first: {{kind: k, result: {result}}}}}"),
        1 => "{before: {first: k, then: m}}".to_owned(),
        2 => format!("{{after: {{first: k, then: {{kind: {then}, result: {result}}}}}}}"),
        _ => format!("{{since: {{last: k, then: {{kind: {then}, result: {result}}}}}}}"),
    }
}

/// What a generated predicate may name.
struct Context<'a> {
    kinds: Vec<&'a str>,
    artifacts: Vec<&'a str>,
    input: bool,
    children: bool,
    /// Whether it may hold an order form.
    orders: bool,
}

impl Context<'_> {
    /// A predicate of at most `depth` levels of `all`, `any` and `not`, testing only the claims
    /// before `claims`, so no claims test each other in a cycle.
    fn predicate(&self, rng: &mut Rng, depth: usize, claims: usize) -> String {
        let kind = |rng: &mut Rng| {
            let plain: Vec<&&str> = self
                .kinds
                .iter()
                .filter(|k| **k != "case_outcome")
                .collect();
            *plain[rng.below(plain.len())]
        };
        let subject = |rng: &mut Rng| match rng.below(3) {
            0 => format!(
                ", subject: {}",
                self.artifacts[rng.below(self.artifacts.len())]
            ),
            _ => String::new(),
        };
        let result = |rng: &mut Rng| ["p", "q"][rng.below(2)];
        match rng.below(if depth == 0 { 7 } else { 10 }) {
            0 | 1 => {
                let result = match rng.below(3) {
                    0 => String::new(),
                    _ => format!(", result: {}", result(rng)),
                };
                let independent = match rng.below(6) {
                    0 => ", independent_on: [d]",
                    1 => ", independent_on: [d, e]",
                    _ => "",
                };
                format!(
                    "{{evidence: {{kind: {}{}{result}{independent}}}}}",
                    kind(rng),
                    subject(rng)
                )
            }
            2 if claims > 0 => format!(
                "{{claim: c{}, is: {}}}",
                rng.below(claims),
                ["true", "false", "unknown"][rng.below(3)]
            ),
            3 if self.input => format!("{{input: s, in: [{}]}}", ["x", "x-y", "y"][rng.below(3)]),
            4 if self.orders => match rng.below(4) {
                0 => format!(
                    "{{first: {{kind: {}, result: {}{}}}}}",
                    kind(rng),
                    result(rng),
                    subject(rng)
                ),
                1 => format!(
                    "{{before: {{first: {}, then: {}{}}}}}",
                    kind(rng),
                    kind(rng),
                    subject(rng)
                ),
                2 => format!(
                    "{{after: {{first: {}, then: {{kind: {}, result: {}}}{}}}}}",
                    kind(rng),
                    kind(rng),
                    result(rng),
                    subject(rng)
                ),
                _ => format!(
                    "{{since: {{last: {}, then: {{kind: {}, result: {}}}{}}}}}",
                    kind(rng),
                    kind(rng),
                    result(rng),
                    subject(rng)
                ),
            },
            5 if self.children => children_form(rng, &self.artifacts),
            7 => format!("{{not: {}}}", self.predicate(rng, depth - 1, claims)),
            8 | 9 => {
                let members: Vec<String> = (0..1 + rng.below(3))
                    .map(|_| self.predicate(rng, depth - 1, claims))
                    .collect();
                let form = if rng.below(2) == 0 { "all" } else { "any" };
                format!("{{{form}: [{}]}}", members.join(", "))
            }
            _ => format!("{{evidence: {{kind: {}}}}}", kind(rng)),
        }
    }
}

/// The largest whole product a generated protocol is enumerated three ways over.
const GENERATED_STATES: u128 = 2048;

/// Generates protocols from `seed` until `wanted` of them compile and have a whole product within
/// [`GENERATED_STATES`], checks each three ways and asserts every report and refusal agrees.
/// Returns how many were compared and how many have a subject whose slice is smaller than the
/// whole product.
fn generated_protocols_agree(seed: u64, wanted: usize) -> (usize, usize) {
    let mut rng = Rng(seed);
    let (mut compared, mut attempts) = (0, 0);
    let mut chosen = Vec::new();
    while compared < wanted {
        attempts += 1;
        assert!(
            attempts < wanted * 50,
            "{compared} of {attempts} generated protocols compile"
        );
        let (source, body) = generated(&mut rng);
        let Ok(parsed) = parse(&source) else {
            continue;
        };
        let Ok(ir) = crate::ir::compile(&parsed) else {
            continue;
        };
        let Some(properties) = body.as_deref().map(read_properties).transpose().ok() else {
            continue;
        };
        if Space::new(&ir)
            .size()
            .is_none_or(|size| size > GENERATED_STATES)
        {
            continue;
        }
        if properties
            .as_ref()
            .is_some_and(|p| properties::validate(&ir, p).is_err())
        {
            continue;
        }
        compared += 1;
        chosen.push((ir, properties, source));
    }
    // The protocols are chosen above in generation order; each is then compared on its own.
    let results = par_map(chosen.len(), |at| {
        let (ir, properties, source) = &chosen[at];
        let (widest, whole) = slices_agree(ir, properties.as_ref(), source, false);
        let report = check(ir, properties.as_ref()).map(|report| report.to_string());
        (
            widest < whole,
            report.unwrap_or_else(|_| "refused".to_owned()),
        )
    });
    let mut sliced = 0;
    let mut features: BTreeMap<&str, usize> = BTreeMap::new();
    for ((_, _, source), (narrower, text)) in chosen.iter().zip(&results) {
        if *narrower {
            sliced += 1;
        }
        for feature in GENERATED_FEATURES {
            if source.contains(feature) || text.contains(feature) {
                *features.entry(feature).or_default() += 1;
            }
        }
    }
    eprintln!("seed {seed:#x}: {compared} compared, {sliced} sliced, {features:?}");
    for feature in GENERATED_FEATURES {
        assert!(
            features.get(feature).is_some_and(|n| *n >= 2),
            "seed {seed:#x}: {feature} in fewer than two compared protocols: {features:?}"
        );
    }
    (compared, sliced)
}

/// What the generated protocols must reach, in the source or the report: every order form, child
/// cases, a `children` form bound to the current revision (story:child-outcome-revision), one
/// naming a subject (story:child-outcome-subject, written `], subject:` only by a `children` form),
/// invalidation, independence, inputs, and each finding that reads states.
const GENERATED_FEATURES: [&str; 13] = [
    "inputs:",
    "first:",
    "since:",
    "after:",
    "children:",
    "revision: current",
    "], subject:",
    "invalidation:",
    "independent_on",
    "unreachable-outcome",
    "unsatisfiable-precondition",
    "authority-bypass",
    "property-failed",
];

/// Option A's claim, tested: an action's status depends only on what its precondition reads and
/// the capabilities it requires, an outcome's on its requirement or its decision, a property's on
/// its subject's slice and its `independent_of` claim. Over generated protocols with every
/// predicate form, the report of each subject over its own slice, and `canon check`'s, is the
/// whole product's, byte for byte. Four seeds, run side by side.
#[test]
fn generated_protocols_check_alike_sliced_and_whole_0() {
    let (compared, sliced) = generated_protocols_agree(0x9E37_79B9_7F4A_7C15, 150);
    assert!(sliced * 2 >= compared, "{sliced} of {compared} sliced");
}

#[test]
fn generated_protocols_check_alike_sliced_and_whole_1() {
    let (compared, sliced) = generated_protocols_agree(0xD1B5_4A32_D192_ED03, 150);
    assert!(sliced * 2 >= compared, "{sliced} of {compared} sliced");
}

#[test]
fn generated_protocols_check_alike_sliced_and_whole_2() {
    let (compared, sliced) = generated_protocols_agree(0x8CB9_2BA7_2F3D_8DD7, 150);
    assert!(sliced * 2 >= compared, "{sliced} of {compared} sliced");
}

#[test]
fn generated_protocols_check_alike_sliced_and_whole_3() {
    let (compared, sliced) = generated_protocols_agree(0x2545_F491_4F6C_DD1D, 150);
    assert!(sliced * 2 >= compared, "{sliced} of {compared} sliced");
}

/// The largest whole product a fixture protocol is enumerated three ways over, which keeps the
/// suite's time; the check fixtures hold at most 288 states.
const FIXTURE_STATES: u128 = 8192;

/// Every `protocol/1` document under `fixtures/` whose whole product is within
/// [`FIXTURE_STATES`] checks alike over slices and over the whole product, with its
/// `properties.yaml` when it has one.
#[test]
fn every_fixture_protocol_checks_alike_sliced_and_whole() {
    let fixtures: Vec<_> = fixture_protocols()
        .into_iter()
        .filter(|(_, ir)| {
            Space::new(ir)
                .size()
                .is_some_and(|size| size <= FIXTURE_STATES)
        })
        .collect();
    let narrower = par_map(fixtures.len(), |at| {
        let (path, ir) = &fixtures[at];
        let declared = path.with_file_name("properties.yaml");
        let properties = declared
            .exists()
            .then(|| properties(&std::fs::read_to_string(&declared).expect("readable")));
        let (widest, whole) =
            slices_agree(ir, properties.as_ref(), &path.display().to_string(), false);
        widest < whole
    });
    let compared = narrower.len();
    let sliced = narrower.iter().filter(|narrower| **narrower).count();
    eprintln!("{compared} fixture protocols compared, {sliced} sliced");
    assert!(
        compared >= 20 && sliced >= 10,
        "{sliced} of {compared} fixtures sliced"
    );
}

/// Every `protocol/1` document under `fixtures/` that compiles, in path order, with its path.
fn fixture_protocols() -> Vec<(std::path::PathBuf, Ir)> {
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR").expect("run through cargo");
    let root = std::path::PathBuf::from(manifest).join("../../fixtures");
    let mut pending = vec![root];
    let mut files = Vec::new();
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("readable") {
            let path = entry.expect("readable").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|e| e == "yaml") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
        .into_iter()
        .filter_map(|path| {
            let source = std::fs::read_to_string(&path).expect("readable");
            if !source.starts_with("format: protocol/1") || path.ends_with("properties.yaml") {
                return None;
            }
            let ir = crate::ir::compile(&parse(&source).ok()?).ok()?;
            Some((path, ir))
        })
        .collect()
}

/// Every probe of a whole product is one of its states: each value below the number of values its
/// dimension has. Over every fixture protocol whose whole product can be counted, 600 generated
/// protocols, and protocols that declare no artifact, whose evidence dimensions have one value but
/// two producer sets.
#[test]
fn every_probe_is_a_state_of_the_whole_product() {
    let mut spaces: Vec<(String, Ir)> = fixture_protocols()
        .into_iter()
        .map(|(path, ir)| (path.display().to_string(), ir))
        .collect();
    let mut rng = Rng(7);
    let mut generated_count = 0;
    while generated_count < 600 {
        let (source, _) = generated(&mut rng);
        if let Some(ir) = parse(&source)
            .ok()
            .and_then(|m| crate::ir::compile(&m).ok())
        {
            spaces.push((source, ir));
            generated_count += 1;
        }
    }
    for source in [
        PROTOCOL.replace("artifacts: {a: {}}\n", ""),
        UNREAD.replace("artifacts: {a: {}}\n", ""),
    ] {
        spaces.push((source.clone(), compiled(&source)));
    }
    let mut probed = 0;
    for (name, ir) in &spaces {
        let space = Space::new(ir);
        if space.size().is_none() {
            continue;
        }
        for probe in space.probes() {
            assert!(space.is_state(&probe), "{name}: {probe:?}");
        }
        probed += 1;
    }
    assert!(probed > 600, "{probed} spaces probed");
}

/// A slice keeps the classes and producer profiles its evidence dimensions have in the whole
/// space, not only those its subject's matches name: `granted` reads `k` with no result, yet the
/// whole product's witness is a record of `fail`, a class only `failed` names, because
/// `` result `fail` `` is written before the `}` that ends a bare record; and `seen`'s is a record
/// from the producer only `reviewed` demands. Classes derived from the subject's own matches would
/// write `{evidence `k`}` and `{evidence `m`}` (story:check-subject-slices, Outcome item 1).
#[test]
fn a_slice_keeps_the_classes_and_profiles_of_the_whole_space() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        independence_dimensions: {d: {}}\n\
        artifacts: {a: {}}\n\
        evidence_kinds: {k: {}, m: {}}\n\
        claims:\n\
        \x20\x20failed: {true_when: {evidence: {kind: k, result: fail}}}\n\
        \x20\x20reviewed: {true_when: {evidence: {kind: m, independent_on: [d]}}}\n\
        actions:\n\
        \x20\x20approve: {requires: [{capability: c}], may_produce: [{evidence: k}, {evidence: m}]}\n\
        \x20\x20self_approve: {may_produce: [{evidence: k}, {evidence: m}]}\n\
        outcomes:\n\
        \x20\x20granted: {requires: {evidence: {kind: k}}}\n\
        \x20\x20seen: {requires: {evidence: {kind: m}}}\n";
    let ir = compiled(source);
    assert_slices_agree(&ir, None, source);
    assert_eq!(
        check_by(&ir, None, Strategy::Own)
            .expect("checked")
            .to_string(),
        "authority-bypass: outcome `granted` is legitimate without authority in state \
         {evidence `k` result `fail`}\n\
         authority-bypass: outcome `seen` is legitimate without authority in state \
         {evidence `m` by a producer independent on `d`}\n\
         checked: protocol `p` revision 1: 36 states, 0 properties, 2 findings\n"
    );
}

/// A protocol in which evidence kind `k` and case input `t` are read only by obligation `settle`,
/// which is no subject: no outcome, action or property reads them, so no slice holds them.
const UNREAD: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    case: {inputs: {t: {values: [x, y]}}}\n\
    artifacts: {a: {}}\n\
    evidence_kinds: {k: {}, m: {}}\n\
    independence_dimensions: {d: {}}\n\
    claims:\n\
    \x20\x20seen: {true_when: {evidence: {kind: k}}}\n\
    \x20\x20noted: {true_when: {evidence: {kind: m}}}\n\
    obligations: {settle: {discharged_when: {claim: seen}, applicable_when: {input: t, in: [x]}}}\n\
    actions: {work: {may_produce: [{evidence: k}, {evidence: m}]}}\n\
    outcomes: {done: {requires: {claim: noted}}}\n";

/// `UNREAD` compiled and then changed by `change`, as a caller that builds an IR can: the refusal
/// the whole product gives, which each subject over its own slice and `canon check` give too,
/// both with the protocol's outcome and action and, unless `held` (the declaration is a subject's
/// own), with neither, so no state is enumerated but the probes. No slice names `outside`, and the
/// slices hold fewer states than the whole product.
fn refused_alike(change: impl Fn(&mut Ir), outside: &str, held: bool) -> (&'static str, String) {
    let mut ir = compiled(UNREAD);
    change(&mut ir);
    let refusal = |ir: &Ir| {
        let whole = check_by(ir, None, Strategy::Whole).expect_err("the whole product refuses");
        for strategy in [Strategy::Own, Strategy::Chosen] {
            let sliced = check_by(ir, None, strategy).expect_err("the slices refuse");
            assert_eq!(
                (sliced.code(), sliced.to_string()),
                (whole.code(), whole.to_string())
            );
        }
        (whole.code(), whole.to_string())
    };
    let space = Space::new(&ir);
    let slices = subjects(&ir, &space, None);
    let widest = slices
        .iter()
        .filter_map(|(_, slice)| space.slice(slice).size())
        .sum::<u128>();
    assert!(widest < space.size().expect("fits"), "{widest}");
    if !outside.is_empty() {
        for (subject, slice) in &slices {
            let dimensions = space.slice(slice).dimensions();
            assert!(!dimensions.contains(outside), "{subject}: {dimensions}");
        }
    }
    let with_subjects = refusal(&ir);
    if !held {
        ir.outcomes.clear();
        ir.actions.clear();
        assert_eq!(refusal(&ir), with_subjects, "without a subject");
    }
    with_subjects
}

/// A refusal as `refused_alike` gives it.
fn refusal(code: &'static str, message: &str) -> (&'static str, String) {
    (code, message.to_owned())
}

/// `seen` reads `matching` in place of a bare match of `k`.
fn seen_reads(ir: &mut Ir, matching: crate::model::EvidenceMatch) {
    ir.claims
        .get_mut(&ClaimId::new("seen"))
        .expect("declared")
        .true_when = crate::model::Predicate::Evidence(matching);
}

fn matching(kind: &str) -> crate::model::EvidenceMatch {
    crate::model::EvidenceMatch {
        kind: EvidenceKindId::new(kind),
        result: None,
        subject: None,
        independent_on: None,
    }
}

#[test]
fn an_unreadable_max_age_no_subject_reads_is_refused() {
    let refused = refused_alike(
        |ir| {
            ir.evidence_kinds
                .get_mut(&EvidenceKindId::new("k"))
                .expect("declared")
                .max_age = Some(crate::model::Age::new("1 hour"));
        },
        "evidence kind `k`",
        false,
    );
    assert_eq!(
        refused,
        refusal(
            "invalid-max-age",
            "evidence kind `k` has max_age `1 hour`, which the evaluator cannot read"
        )
    );
}

#[test]
fn an_invalidation_rule_over_an_undeclared_artifact_no_subject_reads_is_refused() {
    let refused = refused_alike(
        |ir| {
            let rule = crate::ir::InvalidationRule {
                description: None,
                upstream: crate::model::ArtifactId::new("ghost"),
                invalidates: vec![ClaimId::new("seen")],
            };
            ir.invalidation
                .insert(crate::model::InvalidationRuleId::new("r"), rule);
        },
        "evidence kind `k`",
        false,
    );
    assert_eq!(
        refused,
        refusal(
            "undeclared-artifact",
            "invalidation rule `r` watches artifact `ghost`, which the protocol does not declare"
        )
    );
}

#[test]
fn a_protocol_artifact_or_input_named_with_a_space_is_refused() {
    let protocol = refused_alike(|ir| ir.protocol.id = ProtocolId::new("p q"), "", false);
    let artifact = refused_alike(
        |ir| {
            ir.artifacts.insert(
                crate::model::ArtifactId::new("b b"),
                crate::ir::Artifact {
                    description: None,
                    revised_by: None,
                },
            );
        },
        "",
        false,
    );
    let input = refused_alike(
        |ir| {
            let input = crate::ir::CaseInput {
                description: None,
                values: vec![crate::model::InputValue::new("x")],
            };
            ir.inputs.insert(InputId::new("u u"), input);
        },
        "",
        false,
    );
    assert_eq!(
        [protocol, artifact, input],
        [
            refusal(
                "invalid-identifier",
                "case protocol identifier `p q` is empty or contains whitespace or a control character"
            ),
            refusal(
                "invalid-identifier",
                "case artifact identifier `b b` is empty or contains whitespace or a control character"
            ),
            refusal(
                "invalid-identifier",
                "case input identifier `u u` is empty or contains whitespace or a control character"
            )
        ]
    );
}

#[test]
fn an_input_value_no_subject_reads_named_with_a_space_is_refused() {
    let refused = refused_alike(
        |ir| {
            ir.inputs
                .get_mut(&InputId::new("t"))
                .expect("declared")
                .values = vec![
                crate::model::InputValue::new("x"),
                crate::model::InputValue::new("y z"),
            ];
        },
        "input `t`",
        false,
    );
    assert_eq!(
        refused,
        refusal(
            "invalid-identifier",
            "input value `y z` is empty or contains whitespace or a control character"
        )
    );
}

#[test]
fn an_evidence_kind_no_subject_reads_named_with_a_space_is_refused() {
    let refused = refused_alike(
        |ir| {
            let kind = ir
                .evidence_kinds
                .remove(&EvidenceKindId::new("k"))
                .expect("declared");
            ir.evidence_kinds.insert(EvidenceKindId::new("k k"), kind);
            seen_reads(ir, matching("k k"));
        },
        "evidence kind `k k`",
        false,
    );
    assert_eq!(
        refused,
        refusal(
            "invalid-identifier",
            "evidence `e0` kind identifier `k k` is empty or contains whitespace or a control character"
        )
    );
}

/// The space builds a dimension for each declared evidence kind only, so no evaluation `check`
/// makes holds a record of an undeclared kind, and `undeclared-evidence-kind` is raised neither
/// over the whole product nor over slices.
#[test]
fn an_undeclared_evidence_kind_has_no_dimension_and_is_never_refused() {
    let mut ir = compiled(UNREAD);
    seen_reads(&mut ir, matching("ghost"));
    assert_slices_agree(&ir, None, "UNREAD, `seen` reading undeclared `ghost`");
    let mut none = ir.clone();
    none.outcomes.clear();
    none.actions.clear();
    assert_slices_agree(&none, None, "the same, without a subject");
    assert_eq!(
        Space::new(&ir).dimensions(),
        "evidence kind `k` 2, evidence kind `m` 2, input `t` 2"
    );
}

#[test]
fn a_record_about_an_undeclared_or_misnamed_artifact_no_subject_reads_is_refused() {
    let about = |artifact: &str| {
        let artifact = artifact.to_owned();
        refused_alike(
            move |ir| {
                let mut about = matching("k");
                about.subject = Some(crate::model::ArtifactId::new(artifact.as_str()));
                seen_reads(ir, about);
            },
            "evidence kind `k`",
            false,
        )
    };
    assert_eq!(
        [about("ghost"), about("gh ost")],
        [
            refusal(
                "undeclared-artifact",
                "evidence `e0` is about artifact `ghost`, which the protocol does not declare"
            ),
            refusal(
                "invalid-identifier",
                "evidence `e0` subject identifier `gh ost` is empty or contains whitespace or a control character"
            )
        ]
    );
}

#[test]
fn a_producer_dimension_no_subject_reads_undeclared_or_misnamed_is_refused() {
    let demanding = |dimension: &str| {
        let dimension = dimension.to_owned();
        refused_alike(
            move |ir| {
                let mut demand = matching("k");
                demand.independent_on = Some(vec![crate::model::IndependenceDimensionId::new(
                    dimension.as_str(),
                )]);
                seen_reads(ir, demand);
            },
            "evidence kind `k`",
            false,
        )
    };
    assert_eq!(
        [demanding("ghost"), demanding("d d")],
        [
            refusal(
                "undeclared-dimension",
                "case `check` lists producer `producer-1` as independent on dimension `ghost`, which the protocol does not declare"
            ),
            refusal(
                "invalid-identifier",
                "independence dimension identifier `d d` is empty or contains whitespace or a control character"
            )
        ]
    );
}

#[test]
fn a_capability_or_decision_named_with_a_space_is_refused() {
    let capability = refused_alike(
        |ir| {
            ir.actions
                .get_mut(&ActionId::new("work"))
                .expect("declared")
                .requires = vec![crate::model::CapabilityId::new("c c")];
        },
        "",
        true,
    );
    let decision = refused_alike(
        |ir| {
            ir.outcomes
                .get_mut(&OutcomeId::new("done"))
                .expect("declared")
                .requires =
                crate::model::OutcomeRequirement::Decision(crate::model::DecisionRequirement {
                    decision: crate::model::DecisionName::new("d d"),
                    when: None,
                });
        },
        "",
        true,
    );
    assert_eq!(
        [capability, decision],
        [
            refusal(
                "invalid-identifier",
                "authority capability `c c` is empty or contains whitespace or a control character"
            ),
            refusal(
                "invalid-identifier",
                "decision name `d d` is empty or contains whitespace or a control character"
            )
        ]
    );
}

/// An outcome that requires a decision with a condition (story:decision-condition) is decided
/// over the decision's dimension and what the condition reads, so a slice that leaves the
/// condition's evidence out would decide it over states that cannot tell `when` apart. The check
/// gains no new kind of dimension, and the three strategies agree.
#[test]
fn a_decision_outcome_slice_holds_the_dimensions_its_condition_reads() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
        evidence_kinds: {k: {}, m: {}}\n\
        claims:\n\
        \x20\x20refused: {true_when: {evidence: {kind: k}}}\n\
        \x20\x20noted: {true_when: {evidence: {kind: m}}}\n\
        actions: {act: {may_produce: [{evidence: k}, {evidence: m}]}}\n\
        outcomes:\n\
        \x20\x20withdrawn: {requires: {decision: d, when: {claim: refused}}}\n\
        \x20\x20plain: {requires: {decision: e}}\n";
    let ir = compiled(source);
    let space = Space::new(&ir);
    let withdrawn = space.outcome_slice(&ir, &ir.outcomes[&OutcomeId::new("withdrawn")]);
    let plain = space.outcome_slice(&ir, &ir.outcomes[&OutcomeId::new("plain")]);
    assert_eq!(
        space.slice(&withdrawn).dimensions(),
        "evidence kind `k` 2, decision `d` 2"
    );
    assert_eq!(space.slice(&plain).dimensions(), "decision `e` 2");
    assert_slices_agree(&ir, None, source);
}
