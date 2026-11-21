//! Adversary pass 1 for story:check-subject-slices (wave 2026-11-20-w2), against `canon check`
//! deciding each outcome, action and property over its slice (`crates/canon/src/check/mod.rs`,
//! `space.rs`).
//!
//! The contract read is the story's Outcome and the module docs of `check`: the report over slices
//! is byte-identical to the whole product's, and "An evaluation the evaluator refuses is refused
//! with the evaluator's code".
//!
//! The `probe_*` cases compare a protocol the generator in `check/tests.rs` does not write (claims
//! three levels deep under `not` with an invalidation rule built on, three case inputs whose
//! first-rendered value is not the first declared, scoped and repeated capabilities, child cases
//! opened under authority, shared producer profiles, and a `since` or `after` whose whole product
//! is above the generator's 2048 states) over its slices and over its whole product. The whole
//! product is reached from outside the crate by adding one action, `zz`, whose slice is every
//! dimension and which finds nothing, so `canon check` enumerates the whole product; without it,
//! the slices hold fewer states than the whole product and are enumerated instead. They pass.
//!
//! `an_unreadable_max_age_is_refused_when_no_subject_reads_a_state` fails: a protocol with no
//! outcome, action or property has no slice, so no state is evaluated and the evaluator's refusal
//! of a caller-built IR is lost, where the whole-product enumeration at 557e75b refused it.

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

/// `source` holds a line `ZZ`: without it the check slices; with `every` in its place, an action
/// whose slice is the whole product, `canon check` enumerates the whole product.
fn agree(source: &str, every: &str, properties: Option<&str>) -> String {
    let sliced = report(&source.replace("  ZZ\n", ""), properties);
    let whole = report(
        &source.replace("  ZZ\n", &format!("  {every}\n")),
        properties,
    );
    eprintln!("sliced:\n{sliced}\nwhole:\n{whole}");
    assert_eq!(sliced, whole);
    sliced
}

const CHAIN: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
artifacts: {a: {}, b: {}}\n\
evidence_kinds: {k: {}, m: {}, n: {}, u: {}, v: {}}\n\
claims:\n\
\x20\x20c1: {true_when: {evidence: {kind: k, result: ok}}}\n\
\x20\x20c2: {true_when: {any: [{claim: c1}, {evidence: {kind: m, subject: a}}]}}\n\
\x20\x20c3: {true_when: {not: {claim: c2}}}\n\
\x20\x20c4: {true_when: {all: [{claim: c3, is: false}, {evidence: {kind: n, result: x}}]}}\n\
\x20\x20cu: {true_when: {evidence: {kind: u, result: p}}}\n\
\x20\x20cv: {true_when: {evidence: {kind: v}}}\n\
invalidation: {r: {upstream: b, invalidates: [c1]}}\n\
actions:\n\
\x20\x20free_k: {may_produce: [{evidence: k}, {evidence: n}]}\n\
\x20\x20gov: {requires: [{capability: g}], may_produce: [{evidence: m}, {evidence: u}]}\n\
\x20\x20act4: {precondition: {claim: c4}, may_produce: [{evidence: v}]}\n\
\x20\x20actu: {precondition: {claim: cu, is: unknown}}\n\
\x20\x20ZZ\n\
outcomes:\n\
\x20\x20o4: {requires: {claim: c4}}\n\
\x20\x20ou: {requires: {all: [{claim: cu}, {claim: cv}]}}\n\
\x20\x20ov: {requires: {claim: cv}}\n";

const CHAIN_EVERY: &str = "zz: {precondition: {any: [{claim: c4, is: true}, {claim: c4, is: false}, \
{claim: c4, is: unknown}, {claim: cu}, {claim: cv}]}, requires: [{capability: g}]}";

const CHAIN_PROPERTIES: &str = "format: canon-properties/1\nprotocol: p\nproperties:\n\
\x20\x20x1: {subject: {outcome: o4}, independent_of: {claim: cu}}\n\
\x20\x20x2: {subject: {action: act4}, independent_of: {claim: c1}}\n\
\x20\x20x3: {subject: {outcome: ou}, independent_of: {claim: cv}}\n\
\x20\x20x4: {subject: {outcome: o4}, independent_of: {claim: c2}}\n";

#[test]
fn probe_chain() {
    agree(CHAIN, CHAIN_EVERY, Some(CHAIN_PROPERTIES));
}

const ORDER: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
artifacts: {a: {}}\n\
evidence_kinds: {k: {}, m: {}, j: {}}\n\
claims:\n\
\x20\x20s1: {true_when: {since: {last: k, then: {kind: m, result: fail}}}}\n\
\x20\x20f1: {true_when: {first: {kind: k, result: x}}}\n\
\x20\x20cj: {true_when: {evidence: {kind: j}}}\n\
actions:\n\
\x20\x20run: {may_produce: [{evidence: k}, {evidence: j}]}\n\
\x20\x20gate: {precondition: {claim: f1}, may_produce: [{evidence: m}]}\n\
\x20\x20ZZ\n\
outcomes:\n\
\x20\x20early: {requires: {claim: f1}}\n\
\x20\x20late: {requires: {claim: s1}}\n\
\x20\x20nl: {requires: {not: {claim: s1}}}\n\
\x20\x20seen: {requires: {claim: cj}}\n";

const ORDER_EVERY: &str = "zz: {precondition: {any: [{claim: s1}, {claim: f1}, {claim: cj, is: true}, \
{claim: cj, is: false}, {claim: cj, is: unknown}]}}";

const ORDER_PROPERTIES: &str = "format: canon-properties/1\nprotocol: p\nproperties:\n\
\x20\x20p1: {subject: {outcome: early}, independent_of: {claim: s1}}\n\
\x20\x20p2: {subject: {outcome: late}, independent_of: {claim: f1}}\n\
\x20\x20p3: {subject: {outcome: nl}, independent_of: {claim: cj}}\n\
\x20\x20p4: {subject: {action: gate}, independent_of: {claim: s1}}\n";

#[test]
fn probe_order() {
    agree(ORDER, ORDER_EVERY, Some(ORDER_PROPERTIES));
}

const BEFORE: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
artifacts: {a: {}}\n\
evidence_kinds: {k: {}, m: {}, n: {}, j: {}}\n\
claims:\n\
\x20\x20f1: {true_when: {first: {kind: k, result: x}}}\n\
\x20\x20b1: {true_when: {before: {first: n, then: m}}}\n\
\x20\x20cj: {true_when: {evidence: {kind: j}}}\n\
actions:\n\
\x20\x20run: {may_produce: [{evidence: k}, {evidence: n}, {evidence: j}]}\n\
\x20\x20gate: {precondition: {claim: f1}, requires: [{capability: g}], may_produce: [{evidence: m}]}\n\
\x20\x20ZZ\n\
outcomes:\n\
\x20\x20early: {requires: {claim: f1}}\n\
\x20\x20nb: {requires: {not: {claim: b1}}}\n\
\x20\x20yb: {requires: {all: [{claim: b1}, {claim: cj, is: false}]}}\n";

const BEFORE_EVERY: &str = "zz: {precondition: {any: [{claim: f1}, {claim: b1}, {claim: cj, is: true}, \
{claim: cj, is: false}, {claim: cj, is: unknown}]}, requires: [{capability: g}]}";

const BEFORE_PROPERTIES: &str = "format: canon-properties/1\nprotocol: p\nproperties:\n\
\x20\x20b1p: {subject: {outcome: early}, independent_of: {claim: b1}}\n\
\x20\x20b2p: {subject: {outcome: nb}, independent_of: {claim: f1}}\n\
\x20\x20b3p: {subject: {outcome: yb}, independent_of: {claim: cj}}\n\
\x20\x20b4p: {subject: {action: gate}, independent_of: {claim: b1}}\n";

#[test]
fn probe_before() {
    agree(BEFORE, BEFORE_EVERY, Some(BEFORE_PROPERTIES));
}

const AFTER: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
artifacts: {a: {}}\n\
evidence_kinds: {k: {}, m: {}, j: {}}\n\
claims:\n\
\x20\x20a1: {true_when: {after: {first: m, then: {kind: k, result: x}}}}\n\
\x20\x20ck: {true_when: {evidence: {kind: k, result: x}}}\n\
\x20\x20cj: {true_when: {evidence: {kind: j}}}\n\
actions:\n\
\x20\x20run: {requires: [{capability: g}], may_produce: [{evidence: k}, {evidence: j}]}\n\
\x20\x20free: {may_produce: [{evidence: m}, {evidence: j}]}\n\
\x20\x20gate: {precondition: {claim: ck}, may_produce: [{evidence: m}]}\n\
\x20\x20ZZ\n\
outcomes:\n\
\x20\x20aft: {requires: {claim: a1}}\n\
\x20\x20naft: {requires: {not: {claim: a1}}}\n\
\x20\x20passed: {requires: {all: [{claim: ck}, {claim: cj}]}}\n";

const AFTER_EVERY: &str = "zz: {precondition: {any: [{claim: a1}, {claim: ck}, {claim: cj, is: true}, \
{claim: cj, is: false}, {claim: cj, is: unknown}]}, requires: [{capability: g}]}";

const AFTER_PROPERTIES: &str = "format: canon-properties/1\nprotocol: p\nproperties:\n\
\x20\x20t1: {subject: {outcome: aft}, independent_of: {claim: ck}}\n\
\x20\x20t2: {subject: {outcome: passed}, independent_of: {claim: a1}}\n\
\x20\x20t3: {subject: {action: gate}, independent_of: {claim: a1}}\n";

#[test]
fn probe_after() {
    agree(AFTER, AFTER_EVERY, Some(AFTER_PROPERTIES));
}

const INPUTS: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
case: {inputs: {s: {values: [x, x-y, z]}, t: {values: [b, B, _a]}, u: {values: [q, P]}}}\n\
artifacts: {a: {}}\n\
evidence_kinds: {k: {}, m: {}, n: {}}\n\
claims:\n\
\x20\x20ck: {true_when: {evidence: {kind: k}}}\n\
\x20\x20ct: {true_when: {all: [{input: t, in: [b]}, {evidence: {kind: m}}]}}\n\
\x20\x20cn: {true_when: {evidence: {kind: n, result: r}}}\n\
actions:\n\
\x20\x20approve: {requires: [{capability: c}], may_produce: [{evidence: k}]}\n\
\x20\x20self: {may_produce: [{evidence: k}, {evidence: m}, {evidence: n}]}\n\
\x20\x20note: {precondition: {any: [{input: s, in: [x]}, {claim: ct}]}, may_produce: [{evidence: m}]}\n\
\x20\x20ZZ\n\
outcomes:\n\
\x20\x20granted: {requires: {claim: ck}}\n\
\x20\x20noted: {requires: {all: [{claim: cn}, {not: {claim: ct}}]}}\n";

const INPUTS_EVERY: &str = "zz: {precondition: {any: [{claim: ck}, {claim: ct}, {claim: cn}, \
{input: s, in: [x]}, {input: u, in: [q]}]}, requires: [{capability: c}]}";

const INPUTS_PROPERTIES: &str = "format: canon-properties/1\nprotocol: p\nproperties:\n\
\x20\x20i1: {subject: {action: note}, independent_of: {claim: ct}}\n\
\x20\x20i2: {subject: {outcome: granted}, independent_of: {claim: ct}}\n\
\x20\x20i3: {subject: {outcome: noted}, independent_of: {claim: cn}}\n";

#[test]
fn probe_inputs() {
    agree(INPUTS, INPUTS_EVERY, Some(INPUTS_PROPERTIES));
}

const SCOPED: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
artifacts: {a: {}}\n\
evidence_kinds: {k: {}, m: {}, n: {}}\n\
claims:\n\
\x20\x20ck: {true_when: {evidence: {kind: k}}}\n\
\x20\x20cm: {true_when: {evidence: {kind: m, result: ok}}}\n\
\x20\x20cn: {true_when: {evidence: {kind: n}}}\n\
actions:\n\
\x20\x20wide: {precondition: {claim: ck}, requires: [{capability: c, scope: {x: '1'}}, {capability: c, scope: {x: '2'}}, {capability: d}], may_produce: [{evidence: m}]}\n\
\x20\x20narrow: {requires: [{capability: d}], may_produce: [{evidence: n}]}\n\
\x20\x20free: {may_produce: [{evidence: k}, {evidence: n}]}\n\
\x20\x20ZZ\n\
outcomes:\n\
\x20\x20done: {requires: {all: [{claim: cm}, {claim: cn}]}}\n\
\x20\x20started: {requires: {claim: ck}}\n";

const SCOPED_EVERY: &str = "zz: {precondition: {any: [{claim: ck}, {claim: cm}, {claim: cn}, \
{claim: ck, is: false}, {claim: ck, is: unknown}]}, requires: [{capability: c}, {capability: d}]}";

const SCOPED_PROPERTIES: &str = "format: canon-properties/1\nprotocol: p\nproperties:\n\
\x20\x20w1: {subject: {action: wide}, independent_of: {claim: cm}}\n\
\x20\x20w2: {subject: {action: narrow}, independent_of: {claim: ck}}\n\
\x20\x20w3: {subject: {outcome: done}, independent_of: {claim: cn}}\n";

#[test]
fn probe_scoped() {
    agree(SCOPED, SCOPED_EVERY, Some(SCOPED_PROPERTIES));
}

const CHILDREN: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
artifacts: {a: {}}\n\
evidence_kinds: {k: {}, m: {}, case_outcome: {}}\n\
claims:\n\
\x20\x20ok: {true_when: {children: {in: [accepted]}}}\n\
\x20\x20ck: {true_when: {evidence: {kind: k, result: y}}}\n\
\x20\x20cm: {true_when: {evidence: {kind: m}}}\n\
actions:\n\
\x20\x20open: {requires: [{capability: g}], case_open: {protocol: q, subject: a}}\n\
\x20\x20free: {may_produce: [{evidence: k}, {evidence: m}]}\n\
\x20\x20follow: {precondition: {not: {claim: ok}}, may_produce: [{evidence: m}]}\n\
\x20\x20ZZ\n\
outcomes:\n\
\x20\x20closed: {requires: {all: [{claim: ok}, {claim: ck, is: unknown}]}}\n\
\x20\x20seen: {requires: {claim: cm}}\n\
\x20\x20graded: {requires: {claim: ck}}\n";

const CHILDREN_EVERY: &str = "zz: {precondition: {any: [{claim: ok}, {claim: ck}, {claim: cm}, \
{claim: cm, is: false}, {claim: cm, is: unknown}]}, requires: [{capability: g}]}";

const CHILDREN_PROPERTIES: &str = "format: canon-properties/1\nprotocol: p\nproperties:\n\
\x20\x20h1: {subject: {outcome: closed}, independent_of: {claim: ck}}\n\
\x20\x20h2: {subject: {action: follow}, independent_of: {claim: cm}}\n";

#[test]
fn probe_children() {
    agree(CHILDREN, CHILDREN_EVERY, Some(CHILDREN_PROPERTIES));
}

const PROFILES: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
independence_dimensions: {d: {}, e: {}}\n\
artifacts: {a: {}}\n\
evidence_kinds: {k: {}, m: {}, n: {}}\n\
claims:\n\
\x20\x20c1: {true_when: {evidence: {kind: k, independent_on: [d]}}}\n\
\x20\x20c2: {true_when: {evidence: {kind: m, independent_on: [d, e]}}}\n\
\x20\x20c3: {true_when: {evidence: {kind: k, independent_on: [e, d]}}}\n\
\x20\x20c4: {true_when: {evidence: {kind: n, independent_on: [e]}}}\n\
actions:\n\
\x20\x20gov: {requires: [{capability: g}], may_produce: [{evidence: k}, {evidence: m}]}\n\
\x20\x20free: {may_produce: [{evidence: k}, {evidence: n}]}\n\
\x20\x20ZZ\n\
outcomes:\n\
\x20\x20o1: {requires: {claim: c1}}\n\
\x20\x20o2: {requires: {claim: c2}}\n\
\x20\x20o3: {requires: {all: [{claim: c3}, {claim: c4}]}}\n";

const PROFILES_EVERY: &str = "zz: {precondition: {any: [{claim: c1}, {claim: c2}, {claim: c3}, \
{claim: c4}, {claim: c1, is: false}, {claim: c1, is: unknown}]}, requires: [{capability: g}]}";

const PROFILES_PROPERTIES: &str = "format: canon-properties/1\nprotocol: p\nproperties:\n\
\x20\x20q1: {subject: {outcome: o1}, independent_of: {claim: c3}}\n\
\x20\x20q2: {subject: {outcome: o3}, independent_of: {claim: c2}}\n";

#[test]
fn probe_profiles() {
    agree(PROFILES, PROFILES_EVERY, Some(PROFILES_PROPERTIES));
}

/// `source` compiled, with `k`'s `max_age` replaced by text the evaluator cannot read, as a
/// caller that builds an IR can (`ir::EvidenceKind::max_age`, `eval/freshness.rs`: refused as
/// `invalid-max-age` "whether or not an instant is given. Such an age never silently disables
/// expiry").
fn unreadable_age(source: &str) -> String {
    let mut compiled = ir::compile(&model::parse(source).expect("parses")).expect("compiles");
    compiled
        .evidence_kinds
        .get_mut(&model::EvidenceKindId::new("k"))
        .expect("k is declared")
        .max_age = Some(model::Age::new("1 hour"));
    match check(&compiled, None) {
        Ok(report) => report.to_string(),
        Err(refusal) => format!("refused {}: {refusal}", refusal.code()),
    }
}

/// Every evaluation of an IR whose kind has an unreadable `max_age` is refused, whatever the state.
/// The whole-product enumeration evaluated every state, so `canon check` at 557e75b refused this
/// protocol as `invalid-max-age`, with or without an outcome. Over slices, a protocol with no
/// outcome, action or property has no subject and no slice: no state is evaluated, the refusal is
/// lost, and the check reports the protocol checked (`unproduced-evidence` only), where it still
/// refuses the same protocol with one outcome added.
#[test]
fn an_unreadable_max_age_is_refused_when_no_subject_reads_a_state() {
    let claims_only = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
        evidence_kinds: {k: {max_age: 1h}}\n\
        claims: {seen: {true_when: {evidence: {kind: k}}}}\n\
        obligations: {settle: {discharged_when: {claim: seen}}}\n";
    let refused = "refused invalid-max-age: evidence kind `k` has max_age `1 hour`, which the \
                   evaluator cannot read";
    let with_outcome = format!("{claims_only}outcomes: {{done: {{requires: {{claim: seen}}}}}}\n");
    assert_eq!(unreadable_age(&with_outcome), refused);
    assert_eq!(unreadable_age(claims_only), refused);
}

// Pass 2: the probes `check` evaluates over the whole product before it enumerates slices.

/// `k`, `n` (through a `first`, with a history dimension) and `case_outcome` (through a `children`
/// form, with a history dimension) are read only by obligation `settle`, input `t` only by its
/// `applicable_when`; `done` reads `j` and `work` requires `c`. The whole product is 34560 states;
/// without `zz` the slices hold 5, so `check` evaluates the probes and then the slices; with it,
/// `check` enumerates the whole product, as before slices existed.
const PROBED: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
case: {inputs: {t: {values: [x, y]}}}\n\
artifacts: {a: {}, b: {}}\n\
independence_dimensions: {d: {}}\n\
evidence_kinds: {k: {}, n: {}, j: {}, case_outcome: {}}\n\
claims:\n\
\x20\x20seen: {true_when: {evidence: {kind: k}}}\n\
\x20\x20early: {true_when: {first: {kind: n, result: r}}}\n\
\x20\x20kids: {true_when: {children: {in: [accepted]}}}\n\
\x20\x20noted: {true_when: {evidence: {kind: j}}}\n\
obligations: {settle: {discharged_when: {all: [{claim: seen}, {claim: early}, {claim: kids}]}, \
applicable_when: {input: t, in: [x]}}}\n\
actions:\n\
\x20\x20work: {requires: [{capability: c}], may_produce: [{evidence: k}, {evidence: n}, \
{evidence: j}], case_open: {protocol: q, subject: a}}\n\
\x20\x20ZZ\n\
outcomes: {done: {requires: {claim: noted}}}\n";

const PROBED_EVERY: &str = "zz: {precondition: {any: [{claim: seen}, {claim: early}, {claim: kids}, \
{claim: noted}, {input: t, in: [x]}]}, requires: [{capability: c}]}";

/// `PROBED` with and without `zz`, each compiled and then changed by `change`, as a caller that
/// builds an IR can; returns both results.
fn probed(change: &dyn Fn(&mut ir::Ir)) -> (String, String) {
    let run = |source: String| {
        let mut compiled = ir::compile(&model::parse(&source).expect("parses")).expect("compiles");
        change(&mut compiled);
        match check(&compiled, None) {
            Ok(report) => report.to_string(),
            Err(refusal) => format!("refused {}: {refusal}", refusal.code()),
        }
    };
    let sliced = run(PROBED.replace("  ZZ\n", ""));
    let whole = run(PROBED.replace("  ZZ\n", &format!("  {PROBED_EVERY}\n")));
    eprintln!("sliced: {sliced}\nwhole:  {whole}");
    (sliced, whole)
}

fn seen_reads(compiled: &mut ir::Ir, matching: model::EvidenceMatch) {
    compiled
        .claims
        .get_mut(&model::ClaimId::new("seen"))
        .expect("declared")
        .true_when = model::Predicate::Evidence(matching);
}

fn k_match() -> model::EvidenceMatch {
    model::EvidenceMatch {
        kind: model::EvidenceKindId::new("k"),
        result: None,
        subject: None,
        independent_on: None,
    }
}

fn bad_input(compiled: &mut ir::Ir) {
    compiled
        .inputs
        .get_mut(&model::InputId::new("t"))
        .expect("declared")
        .values = vec![model::InputValue::new("x"), model::InputValue::new("y z")];
}

fn bad_capability(compiled: &mut ir::Ir) {
    for action in compiled.actions.values_mut() {
        action.requires = vec![model::CapabilityId::new("c c")];
    }
}

/// Three offenders at once, a producer dimension no protocol declares, a capability and an input
/// value named with a space: the whole product refuses first at the record, the first of them in
/// its order.
#[test]
fn probe_three_offenders_refuse_as_the_whole_product() {
    let (sliced, whole) = probed(&|compiled| {
        let mut demand = k_match();
        demand.independent_on = Some(vec![model::IndependenceDimensionId::new("ghost")]);
        seen_reads(compiled, demand);
        bad_input(compiled);
        bad_capability(compiled);
    });
    assert_eq!(sliced, whole);
}

/// A capability and an input value: the capability comes first in the whole product's order,
/// though the evaluator checks the case's inputs before the authority.
#[test]
fn probe_capability_before_input_refuses_as_the_whole_product() {
    let (sliced, whole) = probed(&|compiled| {
        bad_input(compiled);
        bad_capability(compiled);
    });
    assert_eq!(sliced, whole);
}

/// A record about an undeclared artifact, refused at binding, after the evaluator's case check,
/// beside an input value refused by the case check: the record's state comes first.
#[test]
fn probe_binding_before_case_check_refuses_as_the_whole_product() {
    let (sliced, whole) = probed(&|compiled| {
        let mut about = k_match();
        about.subject = Some(model::ArtifactId::new("ghost"));
        seen_reads(compiled, about);
        bad_input(compiled);
    });
    assert_eq!(sliced, whole);
}

/// An unreadable `max_age` on `n`, which only an order form reads, beside a capability: refused at
/// the least state.
#[test]
fn probe_max_age_on_an_ordered_kind_refuses_as_the_whole_product() {
    let (sliced, whole) = probed(&|compiled| {
        compiled
            .evidence_kinds
            .get_mut(&model::EvidenceKindId::new("n"))
            .expect("declared")
            .max_age = Some(model::Age::new("1 hour"));
        bad_capability(compiled);
    });
    assert_eq!(sliced, whole);
}

/// Unchanged, the probes refuse nothing and the report is the whole product's.
#[test]
fn probe_unchanged_reports_as_the_whole_product() {
    let (sliced, whole) = probed(&|_| {});
    assert!(!sliced.starts_with("refused"), "{sliced}");
    assert_eq!(sliced, whole);
}
