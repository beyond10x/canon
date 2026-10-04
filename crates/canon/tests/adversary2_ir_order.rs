//! Adversary cases, second pass, for story:canon-ir (wave 2026-10-04-w2).
//!
//! `ir/mod.rs` documents the canonical order of predicate members (`canonical_order`) and says the
//! members of every `all` and `any` are sorted by it and kept once. These cases read that document
//! independently: a reference order written from the doc comment alone, run against generated
//! predicates, and pinned cases for the lines of `canonical_order` the unit's suite never compares.

use std::cmp::Ordering;

use b10x_canon::ir;
use b10x_canon::model::{
    Artifact, Claim, ClaimId, ClaimTest, Declarations, EvidenceKind, EvidenceKindId, EvidenceMatch,
    Outcome, OutcomeId, Predicate, Protocol, ProtocolHeader, ProtocolId, Truth,
};

/// Identifiers chosen where code-point order is easy to get wrong: case, a Latin-1 letter, a BMP
/// character above the surrogates and an astral one (UTF-16 order disagrees on those two), and the
/// two characters JSON escapes.
const IDS: [&str; 8] = [
    "a",
    "B",
    "b",
    "\u{e9}",
    "\u{FF61}",
    "\u{1F600}",
    "q\"t",
    "b\\s",
];
const RESULTS: [Option<&str>; 6] = [
    None,
    Some(""),
    Some("x"),
    Some("y"),
    Some("\u{FF61}"),
    Some("\u{1F600}"),
];
const TRUTHS: [Truth; 3] = [Truth::True, Truth::False, Truth::Unknown];

/// xorshift64*, fixed seeds: the same cases on every run and every platform.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn claim(id: &str, is: Truth) -> Predicate {
    Predicate::Claim(ClaimTest {
        claim: ClaimId::new(id),
        is,
    })
}

fn evidence(kind: &str, result: Option<&str>) -> Predicate {
    Predicate::Evidence(EvidenceMatch {
        kind: EvidenceKindId::new(kind),
        result: result.map(str::to_owned),
    })
}

fn not(inner: Predicate) -> Predicate {
    Predicate::Not(Box::new(inner))
}

fn generate(rng: &mut Rng, depth: u32) -> Predicate {
    let leaf_only = depth == 0;
    match if leaf_only {
        rng.below(2)
    } else {
        rng.below(5)
    } {
        0 => claim(IDS[rng.below(IDS.len())], TRUTHS[rng.below(3)]),
        1 => evidence(IDS[rng.below(IDS.len())], RESULTS[rng.below(RESULTS.len())]),
        2 => not(generate(rng, depth - 1)),
        form => {
            let mut members: Vec<Predicate> = (0..rng.below(4))
                .map(|_| generate(rng, depth - 1))
                .collect();
            if !members.is_empty() && rng.below(3) == 0 {
                let copy = members[rng.below(members.len())].clone();
                members.push(copy);
            }
            if form == 3 {
                Predicate::All(members)
            } else {
                Predicate::Any(members)
            }
        }
    }
}

/// Reorders every `all` and `any` and repeats some members: by the module doc, a document that
/// differs only in this way compiles to the same bytes.
fn scramble(rng: &mut Rng, predicate: &Predicate) -> Predicate {
    let list = |rng: &mut Rng, members: &[Predicate]| {
        let mut members: Vec<Predicate> = members.iter().map(|m| scramble(rng, m)).collect();
        if !members.is_empty() && rng.below(2) == 0 {
            let copy = members[rng.below(members.len())].clone();
            members.push(copy);
        }
        for index in (1..members.len()).rev() {
            members.swap(index, rng.below(index + 1));
        }
        members
    };
    match predicate {
        Predicate::All(members) => Predicate::All(list(rng, members)),
        Predicate::Any(members) => Predicate::Any(list(rng, members)),
        Predicate::Not(inner) => not(scramble(rng, inner)),
        leaf => leaf.clone(),
    }
}

/// The order `canonical_order` documents, written from its doc comment and nothing else.
fn documented_order(left: &Predicate, right: &Predicate) -> Ordering {
    fn form(predicate: &Predicate) -> u8 {
        match predicate {
            Predicate::All(_) => 0,
            Predicate::Any(_) => 1,
            Predicate::Claim(_) => 2,
            Predicate::Evidence(_) => 3,
            Predicate::Not(_) => 4,
        }
    }
    fn tested(is: Truth) -> u8 {
        match is {
            Truth::False => 0,
            Truth::True => 1,
            Truth::Unknown => 2,
        }
    }
    fn code_points(text: &str) -> Vec<u32> {
        text.chars().map(u32::from).collect()
    }
    match (left, right) {
        (Predicate::All(l), Predicate::All(r)) | (Predicate::Any(l), Predicate::Any(r)) => l
            .iter()
            .zip(r)
            .map(|(l, r)| documented_order(l, r))
            .find(|order| *order != Ordering::Equal)
            .unwrap_or_else(|| l.len().cmp(&r.len())),
        (Predicate::Not(l), Predicate::Not(r)) => documented_order(l, r),
        (Predicate::Evidence(l), Predicate::Evidence(r)) => code_points(l.kind.as_str())
            .cmp(&code_points(r.kind.as_str()))
            .then_with(|| match (&l.result, &r.result) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (Some(l), Some(r)) => code_points(l).cmp(&code_points(r)),
            }),
        (Predicate::Claim(l), Predicate::Claim(r)) => code_points(l.claim.as_str())
            .cmp(&code_points(r.claim.as_str()))
            .then_with(|| tested(l.is).cmp(&tested(r.is))),
        _ => form(left).cmp(&form(right)),
    }
}

/// What the module doc says the IR holds for a predicate: members normalized, sorted by the
/// documented order, and only members that are *equal* (`==`) removed.
fn documented_normal_form(predicate: &Predicate) -> Predicate {
    let list = |members: &[Predicate]| {
        let mut members: Vec<Predicate> = members.iter().map(documented_normal_form).collect();
        members.sort_by(documented_order);
        members.dedup_by(|later, earlier| later == earlier);
        members
    };
    match predicate {
        Predicate::All(members) => Predicate::All(list(members)),
        Predicate::Any(members) => Predicate::Any(list(members)),
        Predicate::Not(inner) => not(documented_normal_form(inner)),
        leaf => leaf.clone(),
    }
}

/// A valid protocol whose one outcome `o` requires `requires`; every id in [`IDS`] is declared as
/// a claim (true when `all: []`, so there are no cycles) and as an evidence kind.
fn protocol(requires: Predicate) -> Protocol {
    Protocol {
        format: "protocol/1".to_owned(),
        protocol: ProtocolHeader {
            id: ProtocolId::new("p"),
            revision: 1,
            description: None,
        },
        artifacts: Declarations::new(Vec::<(_, Artifact)>::new()),
        evidence_kinds: Declarations::new(
            IDS.iter()
                .map(|id| (EvidenceKindId::new(*id), EvidenceKind::default()))
                .collect(),
        ),
        claims: Declarations::new(
            IDS.iter()
                .map(|id| {
                    (
                        ClaimId::new(*id),
                        Claim {
                            description: None,
                            true_when: Predicate::All(Vec::new()),
                        },
                    )
                })
                .collect(),
        ),
        obligations: Declarations::default(),
        actions: Declarations::default(),
        outcomes: Declarations::new(vec![(
            OutcomeId::new("o"),
            Outcome {
                description: None,
                requires: requires.into(),
            },
        )]),
    }
}

fn compiled(requires: Predicate) -> ir::Ir {
    let source = protocol(requires.clone());
    ir::compile(&source).unwrap_or_else(|problems| panic!("{requires:?}: {problems:?}"))
}

fn required(ir: &ir::Ir) -> &Predicate {
    ir.outcomes[&OutcomeId::new("o")]
        .requires
        .predicate()
        .expect("the requirement is a predicate")
}

/// Property: for 4000 generated predicates, the compiled predicate is exactly the documented
/// normal form (sorted by the documented order, and only equal members removed), and a scrambled
/// copy (members reordered, some repeated) compiles to the same bytes.
#[test]
fn compiled_predicates_are_the_documented_normal_form_and_ignore_member_order() {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for case in 0..4000 {
        let predicate = generate(&mut rng, 3);
        let ir = compiled(predicate.clone());
        assert_eq!(
            required(&ir),
            &documented_normal_form(&predicate),
            "case {case}: {predicate:?}"
        );
        let scrambled = scramble(&mut rng, &predicate);
        assert_eq!(
            compiled(scrambled.clone()).canonical_json(),
            ir.canonical_json(),
            "case {case}: {predicate:?}\nscrambled: {scrambled:?}"
        );
    }
}

/// Property: two members that are not equal are never merged, and their order does not depend on
/// the order they are written in (antisymmetry, and consistency with equality).
#[test]
fn distinct_members_are_both_kept_in_one_order_whichever_is_written_first() {
    let mut rng = Rng(0xD1B5_4A32_D192_ED03);
    for case in 0..4000 {
        let left = documented_normal_form(&generate(&mut rng, 2));
        let right = documented_normal_form(&generate(&mut rng, 2));
        let forward = compiled(Predicate::Any(vec![left.clone(), right.clone()]));
        let backward = compiled(Predicate::Any(vec![right.clone(), left.clone()]));
        let expected = if left == right { 1 } else { 2 };
        match required(&forward) {
            Predicate::Any(members) => assert_eq!(
                members.len(),
                expected,
                "case {case}: {left:?} / {right:?} -> {members:?}"
            ),
            other => panic!("case {case}: {other:?}"),
        }
        assert_eq!(
            forward.canonical_json(),
            backward.canonical_json(),
            "case {case}: {left:?} / {right:?}"
        );
    }
}

/// Property: three members sort to one order from all six input orders (transitivity: an order that
/// is not transitive sorts differently depending on where the members start).
#[test]
fn three_members_sort_to_one_order_from_every_permutation() {
    let mut rng = Rng(0x1234_5678_9ABC_DEF1);
    let permutations = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    for case in 0..1500 {
        let three: Vec<Predicate> = (0..3).map(|_| generate(&mut rng, 2)).collect();
        let first = compiled(Predicate::All(three.clone())).canonical_json();
        for order in &permutations {
            let members = order.iter().map(|index| three[*index].clone()).collect();
            assert_eq!(
                compiled(Predicate::All(members)).canonical_json(),
                first,
                "case {case}: {three:?} in order {order:?}"
            );
        }
    }
}

/// Two `any` members that differ are two members. The unit's suite only ever compares an `any` with
/// a copy of itself, so a `canonical_order` that compared every pair of `any`s as equal (dropping
/// the `(Any, Any)` arm) would merge `any: [a]` and `any: [b]` and stay green.
#[test]
fn two_different_anys_are_both_kept_and_ordered_by_their_members() {
    let ir = compiled(Predicate::All(vec![
        Predicate::Any(vec![claim("b", Truth::True)]),
        Predicate::Any(vec![claim("a", Truth::True)]),
    ]));
    assert_eq!(
        required(&ir),
        &Predicate::All(vec![
            Predicate::Any(vec![claim("a", Truth::True)]),
            Predicate::Any(vec![claim("b", Truth::True)]),
        ])
    );
}

/// Two lists whose first members differ in the *greater* direction are still different. The unit's
/// suite compares lists only where a prefix is equal, so a `members` comparison that returned early
/// only on `Less` would call `all: [b]` and `all: [a]` equal and drop one.
#[test]
fn lists_whose_first_members_differ_either_way_are_both_kept() {
    let ir = compiled(Predicate::Any(vec![
        Predicate::All(vec![evidence("b", None)]),
        Predicate::All(vec![evidence("a", None)]),
    ]));
    assert_eq!(
        required(&ir),
        &Predicate::Any(vec![
            Predicate::All(vec![evidence("a", None)]),
            Predicate::All(vec![evidence("b", None)]),
        ])
    );
}

/// Deduplication removes equal members only: a `not` and what it negates, and evidence matches that
/// differ only in result (none, empty, some), are all kept.
#[test]
fn deduplication_keeps_members_that_differ_only_in_negation_or_result() {
    let members = vec![
        evidence("a", Some("")),
        not(evidence("a", None)),
        evidence("a", None),
        evidence("a", Some("x")),
        not(not(evidence("a", None))),
        evidence("a", None),
        not(evidence("a", None)),
    ];
    let ir = compiled(Predicate::Any(members));
    assert_eq!(
        required(&ir),
        &Predicate::Any(vec![
            evidence("a", None),
            evidence("a", Some("")),
            evidence("a", Some("x")),
            not(evidence("a", None)),
            not(not(evidence("a", None))),
        ])
    );
}

/// The module doc lists equivalences the IR does *not* normalize; each pair compiles to different
/// bytes, as documented.
#[test]
fn the_documented_non_normalizations_stay_distinct() {
    let x = claim("a", Truth::True);
    let y = evidence("b", None);
    let pairs = [
        ("one-member all", Predicate::All(vec![x.clone()]), x.clone()),
        ("one-member any", Predicate::Any(vec![x.clone()]), x.clone()),
        (
            "nested all",
            Predicate::All(vec![Predicate::All(vec![x.clone(), y.clone()])]),
            Predicate::All(vec![x.clone(), y.clone()]),
        ),
        ("not not", not(not(x.clone())), x.clone()),
    ];
    for (name, written, equivalent) in pairs {
        let written_ir = compiled(written.clone());
        assert_eq!(required(&written_ir), &written, "{name}");
        assert_ne!(
            written_ir.canonical_json(),
            compiled(equivalent).canonical_json(),
            "{name}"
        );
    }
}

/// An identifier is written the same way as an object key (a declaration) and as a string value (a
/// reference to it): `"` and `\` escaped. Every key in the unit's suite is plain ASCII, so a
/// renderer that wrote keys unescaped would stay green and print `"q"t": {`, which is not JSON.
#[test]
fn an_identifier_is_escaped_alike_as_a_key_and_as_a_value() {
    let json = compiled(Predicate::All(vec![
        claim("q\"t", Truth::True),
        evidence("b\\s", None),
    ]))
    .canonical_json();
    for (key, section) in [("q\\\"t", "claims"), ("b\\\\s", "evidence_kinds")] {
        let start = json
            .find(&format!("  \"{section}\": {{\n"))
            .unwrap_or_else(|| panic!("{section} missing:\n{json}"));
        assert!(
            json[start..].contains(&format!("\n    \"{key}\": {{\n")),
            "{section} key {key} not escaped:\n{json}"
        );
    }
    assert!(json.contains("\"id\": \"q\\\"t\",\n"), "{json}");
    assert!(json.contains("\"kind\": \"b\\\\s\",\n"), "{json}");
    assert!(!json.contains("\"q\"t\""), "{json}");
    assert!(!json.contains("\"b\\s\""), "{json}");
}
