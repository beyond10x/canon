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

/// A kind with 127 distinct results has 2^128 values: neither it nor the space fits a `u128`, and
/// the refusal still names both.
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
            "protocol `p` revision 2 has more than {} states, more than the bound of 65536: \
             evidence kind `k` 2^128",
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

/// Adversary pass 1, finding 7: the bound refusal the module docs show is one `check` gives, so
/// its count is the product of its dimensions. Reads the module source at run time.
#[test]
fn the_module_docs_show_a_bound_refusal_check_gives() {
    let results = |kind: &str, prefix: &str, n: usize| {
        (0..n)
            .map(|i| format!("{{evidence: {{kind: {kind}, result: {prefix}{i}}}}}"))
            .collect::<Vec<_>>()
    };
    let mut matches = results("k", "r", 9);
    matches.extend(results("m", "s", 3));
    let source = format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nartifacts: {{a: {{}}}}\n\
         evidence_kinds: {{k: {{}}, m: {{}}}}\n\
         claims: {{x: {{true_when: {{any: [{}]}}}}}}\n\
         actions: {{act: {{requires: [{{capability: c}}], may_produce: [{{evidence: k}}, {{evidence: m}}]}}}}\n\
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
         actions: {{probe: {{may_produce: [{{evidence: k}}]}}}}\n",
        matches.join(", ")
    );
    let refusal = check(&compiled(&large), None).expect_err("refused");
    assert_eq!(
        refusal.to_string(),
        "protocol `p` revision 1 has 131072 states, more than the bound of 65536: evidence kind \
         `k` about `b` 131072"
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
            "protocol `p` revision 2 has 4294967296 states, more than the bound of 65536: evidence \
             kind `k` with upstream `a` 4294967296"
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
    let case = Case {
        format: CASE_FORMAT.to_owned(),
        independence: crate::model::Declarations::default(),
        id: CaseId::new("check"),
        protocol: ir.protocol.id.clone(),
        artifacts: Declarations::new(
            ir.artifacts
                .keys()
                .map(|id| {
                    let current = CaseArtifact {
                        revision: Revision::new(REVISION),
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
    let reached: BTreeSet<Vec<crate::model::Truth>> = (0..size)
        .map(|index| claim_values(&ir, &space.evidence(&space.state(index))))
        .collect();
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let instants = [
        Some("2026-10-01T09:00:00Z"),
        Some("2026-10-01T09:00:01Z"),
        Some("2026-10-01T09:00:02Z"),
        None,
    ];
    let mut histories = 0;
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
                }
            })
            .collect();
        let values = claim_values(&ir, &records);
        assert!(
            reached.contains(&values),
            "no state gives {values:?}, which {records:?} gives"
        );
        histories += 1;
    }
    assert_eq!(histories, 20_000);
    // The space is not trivially everything: `first_k_a` true and `first_k` false needs `b`'s
    // first record before `a`'s, so both orders are reached.
    assert!(reached.len() > 100, "{}", reached.len());
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
    let refusal = check(
        &compiled(&source.replace(
            "{claim: red_first}, {claim: passes}",
            &format!("{{any: [{}]}}, {{claim: passes}}", wide.join(", ")),
        )),
        None,
    )
    .expect_err("refused");
    assert_eq!(refusal.code(), "state-space-bound", "{refusal}");
    assert!(
        refusal.to_string().ends_with(
            "evidence kind `d` 2, evidence kind `t` 32768, history of evidence kind `d` 3, \
             history of evidence kind `t` 32769, order of first observations 3"
        ),
        "{refusal}"
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
