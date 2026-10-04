//! Adversary pass 2 for story:invalidation-rules (wave 2026-10-04-w11), after the pass-1 fixes.
//!
//! Contracts read: the `check` module docs (a class "can be present observed before the move,
//! after it, or both"; `2^(c * 2^m)` values; the witness rule), the `eval` and `explain` module
//! docs (a record invalidated for a claim is listed under it only when an evidence match of the
//! claim's own reads it), coordinator decision 2 of pass 1 (a record listed as excluded under a
//! claim does not decide that claim), the status row ("excluded (`invalidated`) from the claims the
//! rule names and every claim built on them, which become `UNKNOWN`, never `FALSE`, for want of
//! other evidence") and the validator docs (step 6, `inert-invalidation`).

use std::collections::BTreeSet;

use b10x_canon::check::{Finding, check};
use b10x_canon::eval;
use b10x_canon::ir;
use b10x_canon::model::{self, ClaimId, EvidenceRecord, ExclusionReason, Truth};

fn compiled(source: &str) -> ir::Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

fn problems(source: &str) -> Vec<String> {
    match b10x_canon::validate::validate(&model::parse(source).expect("parses")) {
        Ok(()) => Vec::new(),
        Err(problems) => problems
            .iter()
            .map(|problem| format!("{}: {problem}", problem.code()))
            .collect(),
    }
}

/// A fixed-seed xorshift generator: the same cases on every run.
struct Rng(u64);

impl Rng {
    fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}

const VALUES: [&str; 3] = ["true", "false", "unknown"];

/// One outcome per combination of the values of `claims`, as a `protocol/1` `outcomes:` section,
/// and the outcome ids in identifier order.
fn grid(claims: &[&str]) -> (String, Vec<String>) {
    let mut combinations: Vec<Vec<&str>> = vec![Vec::new()];
    for _ in claims {
        combinations = combinations
            .into_iter()
            .flat_map(|prefix| {
                VALUES.iter().map(move |value| {
                    let mut next = prefix.clone();
                    next.push(value);
                    next
                })
            })
            .collect();
    }
    let mut section = String::from("outcomes:\n");
    let mut ids = Vec::new();
    for values in combinations {
        let id = format!("o_{}", values.join("_"));
        let tests: Vec<String> = claims
            .iter()
            .zip(&values)
            .map(|(claim, value)| format!("{{claim: {claim}, is: {value}}}"))
            .collect();
        section.push_str(&format!(
            "  {id}: {{requires: {{all: [{}]}}}}\n",
            tests.join(", ")
        ));
        ids.push(id);
    }
    ids.sort();
    (section, ids)
}

/// A protocol of the family: its source (claims, rules, artifacts), the claims its outcome grid
/// reads, and the artifacts and upstream artifacts the random cases use.
struct Member {
    name: &'static str,
    body: &'static str,
    claims: &'static [&'static str],
    artifacts: &'static [&'static str],
    upstreams: &'static [&'static str],
}

const FAMILY: [Member; 3] = [
    // Two rules on two upstream artifacts, one dimension: 2 classes, 4 vectors, 256 states.
    Member {
        name: "two-upstreams",
        body: "artifacts: {a: {}, u1: {}, u2: {}}\nevidence_kinds: {k: {}}\n\
               claims:\n\
               \x20\x20x: {true_when: {evidence: {kind: k, result: pass}}}\n\
               \x20\x20y: {true_when: {evidence: {kind: k, result: pass}}}\n\
               \x20\x20z: {true_when: {evidence: {kind: k, result: pass}}}\n\
               invalidation: {r1: {upstream: u1, invalidates: [x]}, r2: {upstream: u2, invalidates: [y]}}\n",
        claims: &["x", "y", "z"],
        artifacts: &["a", "u1", "u2"],
        upstreams: &["u1", "u2"],
    },
    // A named claim reading `k` itself and through a helper, a claim built on it with a match
    // about `b` under `not`, bound and unbound dimensions: 4096 states.
    Member {
        name: "helper-and-subjects",
        body: "artifacts: {a: {}, b: {}, up: {}}\nevidence_kinds: {k: {}}\n\
               claims:\n\
               \x20\x20h: {true_when: {evidence: {kind: k, subject: a, result: pass}}}\n\
               \x20\x20n: {true_when: {any: [{claim: h}, {evidence: {kind: k, result: pass}}]}}\n\
               \x20\x20built: {true_when: {all: [{claim: n}, {not: {evidence: {kind: k, subject: b}}}]}}\n\
               invalidation: {r: {upstream: up, invalidates: [n]}}\n",
        claims: &["n", "built", "h"],
        artifacts: &["a", "b", "up"],
        upstreams: &["up"],
    },
    // Two rules on one upstream artifact, a claim built on a named one through `is: unknown`:
    // 3 classes, 2 vectors, 64 states.
    Member {
        name: "shared-upstream",
        body: "artifacts: {a: {}, up: {}}\nevidence_kinds: {k: {}}\n\
               claims:\n\
               \x20\x20x: {true_when: {evidence: {kind: k, result: pass}}}\n\
               \x20\x20w: {true_when: {all: [{claim: x, is: unknown}, {evidence: {kind: k, result: fail}}]}}\n\
               \x20\x20y: {true_when: {evidence: {kind: k, result: fail}}}\n\
               invalidation: {r1: {upstream: up, invalidates: [x]}, r2: {upstream: up, invalidates: [y]}}\n",
        claims: &["x", "w", "y"],
        artifacts: &["a", "up"],
        upstreams: &["up"],
    },
];

fn source(member: &Member) -> String {
    format!(
        "format: protocol/1\nprotocol: {{id: p, revision: 1}}\n{}{}",
        member.body,
        grid(member.claims).0
    )
}

/// Every artifact at `r`, as `canon check` builds its cases.
fn case(member: &Member) -> model::Case {
    let artifacts: Vec<String> = member
        .artifacts
        .iter()
        .map(|a| format!("{a}: {{revision: r}}"))
        .collect();
    eval::read_case(&format!(
        "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {{{}}}\n",
        artifacts.join(", ")
    ))
    .expect("case reads")
}

/// Zero to three records of kind `k`: any result, any subject, mostly at the current subject
/// revision, each upstream revision absent, current, or one of two earlier ones.
fn random_evidence(member: &Member, rng: &mut Rng) -> Vec<EvidenceRecord> {
    let results = [Some("pass"), Some("fail"), Some("other"), None];
    (0..rng.below(4))
        .map(|n| {
            let result = results[rng.below(results.len())]
                .map(|r| format!("result: {r}\n"))
                .unwrap_or_default();
            let subject = member.artifacts[rng.below(member.artifacts.len())];
            let subject_revision = if rng.below(8) == 0 { "x" } else { "r" };
            let upstream: Vec<String> = member
                .upstreams
                .iter()
                .filter_map(|u| match rng.below(4) {
                    0 => None,
                    1 => Some(format!("{u}: r")),
                    2 => Some(format!("{u}: r0")),
                    _ => Some(format!("{u}: r9")),
                })
                .collect();
            let upstream = if upstream.is_empty() {
                String::new()
            } else {
                format!("upstream_revisions: {{{}}}\n", upstream.join(", "))
            };
            eval::read_evidence(&format!(
                "format: canon-evidence/1\nid: e{n}\nkind: k\n{result}subject: {subject}\n\
                 subject_revision: {subject_revision}\n{upstream}"
            ))
            .expect("evidence reads")
        })
        .collect()
}

fn legitimate(decision: &model::Decision) -> BTreeSet<String> {
    decision
        .outcomes
        .as_ref()
        .expect("outcomes")
        .as_object()
        .expect("object")
        .iter()
        .filter(|(_, entry)| entry["status"] == "legitimate")
        .map(|(id, _)| id.clone())
        .collect()
}

/// `canon check` and `canon evaluate` agree on which combinations of claim values a protocol
/// reaches, over a family that uses two upstream artifacts on one dimension, a named claim with a
/// helper, bound and unbound dimensions, `not`, `is: unknown` and two rules on one upstream: no
/// combination evaluate reaches on 4000 seeded random cases is reported unreachable by check, and
/// every combination check reaches is reached by some random case.
#[test]
fn check_and_evaluate_reach_the_same_claim_combinations_across_a_family() {
    for member in &FAMILY {
        let ir = compiled(&source(member));
        let report = check(&ir, None).expect("within the bound");
        let unreachable: BTreeSet<String> = report
            .findings
            .iter()
            .filter_map(|finding| match finding {
                Finding::UnreachableOutcome { outcome } => Some(outcome.as_str().to_owned()),
                _ => None,
            })
            .collect();
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
        let case = case(member);
        let mut reached = BTreeSet::new();
        for _ in 0..4000 {
            let evidence = random_evidence(member, &mut rng);
            let decision = eval::evaluate(&ir, &case, &evidence).expect("decides");
            for outcome in legitimate(&decision) {
                assert!(
                    !unreachable.contains(&outcome),
                    "{}: check calls `{outcome}` unreachable, evaluate reaches it on {evidence:?}",
                    member.name
                );
                reached.insert(outcome);
            }
        }
        let all: BTreeSet<String> = grid(member.claims).1.into_iter().collect();
        let by_check: BTreeSet<String> = all.difference(&unreachable).cloned().collect();
        assert_eq!(
            reached, by_check,
            "{}: {} states",
            member.name, report.states
        );
    }
}

/// The state counts the module docs give: `2^(c * 2^m)` values per dimension.
#[test]
fn the_family_has_the_state_counts_the_formula_gives() {
    let states: Vec<u128> = FAMILY
        .iter()
        .map(|member| {
            check(&compiled(&source(member)), None)
                .expect("within the bound")
                .states
        })
        .collect();
    // 2^(2 * 2^2); (2^(2 * 2^1))^3; 2^(3 * 2^1).
    assert_eq!(states, [256, 4096, 64]);
}

/// Under every claim the explanation writes, the records `because` gives as `excluded`, with their
/// reasons, are exactly the claim's `excluded_evidence`, and no record `excluded_evidence` lists is
/// `applied` there; over the same seeded random cases.
#[test]
fn excluded_evidence_and_the_explanation_agree_under_every_claim() {
    for member in &FAMILY {
        let ir = compiled(&source(member));
        let case = case(member);
        let mut rng = Rng(0x2545_f491_4f6c_dd1d);
        for _ in 0..2000 {
            let evidence = random_evidence(member, &mut rng);
            let decision = eval::evaluate(&ir, &case, &evidence).expect("decides");
            let explanation = decision.explanation.as_ref().expect("explained");
            for (claim, entry) in decision.claims.iter() {
                let Some(because) = explanation["claims"][claim.as_str()]["because"].as_array()
                else {
                    continue;
                };
                let listed: BTreeSet<(String, String)> = entry
                    .excluded_evidence
                    .iter()
                    .map(|e| (e.evidence.as_str().to_owned(), e.reason.as_str().to_owned()))
                    .collect();
                let explained: BTreeSet<(String, String)> = because
                    .iter()
                    .filter(|r| r["status"] == "excluded")
                    .map(|r| {
                        (
                            r["evidence"].as_str().expect("id").to_owned(),
                            r["reason"].as_str().expect("reason").to_owned(),
                        )
                    })
                    .collect();
                let applied: BTreeSet<String> = because
                    .iter()
                    .filter(|r| r["status"] == "applied")
                    .map(|r| r["evidence"].as_str().expect("id").to_owned())
                    .collect();
                assert_eq!(
                    listed,
                    explained,
                    "{}: claim `{}` on {evidence:?}",
                    member.name,
                    claim.as_str()
                );
                assert!(
                    listed.iter().all(|(id, _)| !applied.contains(id)),
                    "{}: claim `{}` lists a record it applies",
                    member.name,
                    claim.as_str()
                );
            }
        }
    }
}

/// Pass-1 decision 2 kept the property "a record listed as excluded from a claim does not decide
/// that claim" and fixed it by listing only under a claim whose own match reads the record, and
/// refusing a named claim with no match of its own. `named` has a match of its own and also tests
/// `helper`, which reads the same kind and which the rule does not name. `named` lists `e1` as
/// `invalidated`, and is `TRUE` only because of `e1`: without it, `UNKNOWN`.
#[test]
fn a_named_claim_that_also_reads_the_record_through_a_helper_lists_it_yet_rests_on_it() {
    assert_eq!(
        problems(HELPER),
        Vec::<String>::new(),
        "`named` is not inert"
    );
    let ir = compiled(HELPER);
    let case = eval::read_case(
        "format: canon-case/1\nid: C-1\nprotocol: p\n\
         artifacts: {a: {revision: r1}, up: {revision: u1}}\n",
    )
    .expect("case reads");
    let moved = eval::read_evidence(
        "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r1\n\
         upstream_revisions: {up: u0}\n",
    )
    .expect("evidence reads");
    let with = eval::evaluate(&ir, &case, std::slice::from_ref(&moved)).expect("decides");
    let without = eval::evaluate(&ir, &case, &[]).expect("decides");
    let named = &with.claims.get(&ClaimId::new("named")).expect("declared");
    let listed: Vec<(&str, ExclusionReason)> = named
        .excluded_evidence
        .iter()
        .map(|e| (e.evidence.as_str(), e.reason))
        .collect();
    let without_value = without
        .claims
        .get(&ClaimId::new("named"))
        .expect("declared")
        .value;
    if listed == [("e1", ExclusionReason::Invalidated)] {
        assert_eq!(
            named.value, without_value,
            "`named` lists `e1` as invalidated yet its value changes when `e1` is removed"
        );
    }
}

/// `named` has a match of its own and tests `helper`, which reads the same kind.
const HELPER: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
    artifacts: {a: {}, up: {}}\nevidence_kinds: {k: {}}\n\
    claims:\n\
    \x20\x20helper: {true_when: {evidence: {kind: k}}}\n\
    \x20\x20named: {true_when: {any: [{claim: helper}, {evidence: {kind: k}}]}}\n\
    invalidation: {r: {upstream: up, invalidates: [named]}}\n";

/// Pass-2 decision 2: the named claim loses the support, and a claim built on it follows its
/// predicate. `z` is built on `x` through `is: unknown`: with one record and only the upstream
/// revision moving, `x` becomes `UNKNOWN`, so `z` goes from `TRUE` to `FALSE`, as its predicate
/// says. (Before decision 2 this case held the status row to "never `FALSE`".)
#[test]
fn a_claim_built_on_a_named_one_follows_its_predicate_when_only_the_upstream_moves() {
    let ir = compiled(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
         artifacts: {a: {}, up: {}}\nevidence_kinds: {k: {}}\n\
         claims:\n\
         \x20\x20x: {true_when: {evidence: {kind: k}}}\n\
         \x20\x20z: {true_when: {not: {claim: x, is: unknown}}}\n\
         invalidation: {r: {upstream: up, invalidates: [x]}}\n",
    );
    let record = eval::read_evidence(
        "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r1\n\
         upstream_revisions: {up: u0}\n",
    )
    .expect("evidence reads");
    let value = |up: &str| {
        let case = eval::read_case(&format!(
            "format: canon-case/1\nid: C-1\nprotocol: p\n\
             artifacts: {{a: {{revision: r1}}, up: {{revision: {up}}}}}\n"
        ))
        .expect("case reads");
        eval::evaluate(&ir, &case, std::slice::from_ref(&record))
            .expect("decides")
            .claims
            .get(&ClaimId::new("z"))
            .expect("declared")
            .value
    };
    assert_eq!(value("u0"), Truth::True);
    assert_eq!(value("u1"), Truth::False, "`z` follows its predicate");
}

/// `done` reads `k` directly, so the rule on `x` changes nothing about it. Without the rule the
/// bypass witness is `{evidence `k`}`; adding the rule must not make the reported witness a record
/// observed before an upstream move, which `done` does not need. The two states weigh the same, and
/// the rendering tie-break puts ` observed…` (a space) before `}`.
#[test]
fn an_unrelated_rule_does_not_change_a_bypass_witness() {
    let base = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}, up: {}}\nevidence_kinds: {k: {}}\n\
        claims:\n\
        \x20\x20x: {true_when: {evidence: {kind: k}}}\n\
        actions:\n\
        \x20\x20observe: {may_produce: [{evidence: k}]}\n\
        \x20\x20approve: {requires: [{capability: c}], may_produce: [{evidence: k}]}\n\
        outcomes:\n\
        \x20\x20done: {requires: {evidence: {kind: k}}}\n";
    let bypass = |source: &str| -> Vec<String> {
        check(&compiled(source), None)
            .expect("checked")
            .findings
            .iter()
            .filter(|f| f.code() == "authority-bypass")
            .map(ToString::to_string)
            .collect()
    };
    let expected = "authority-bypass: outcome `done` is legitimate without authority in state \
        {evidence `k`}";
    assert_eq!(bypass(base), [expected]);
    assert_eq!(
        bypass(&format!(
            "{base}invalidation: {{r: {{upstream: up, invalidates: [x]}}}}\n"
        )),
        [expected]
    );
}

/// A claim whose only evidence match sits under `not` is not inert: a moved record takes it from
/// `FALSE` to `UNKNOWN`, and it lists the record.
#[test]
fn a_named_claim_whose_only_match_is_under_not_is_not_inert() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}, up: {}}\nevidence_kinds: {k: {}}\n\
        claims: {absent: {true_when: {not: {evidence: {kind: k}}}}}\n\
        invalidation: {r: {upstream: up, invalidates: [absent]}}\n";
    assert_eq!(problems(source), Vec::<String>::new());
    let ir = compiled(source);
    let record = eval::read_evidence(
        "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r1\n\
         upstream_revisions: {up: u0}\n",
    )
    .expect("evidence reads");
    let entry = |up: &str| {
        let case = eval::read_case(&format!(
            "format: canon-case/1\nid: C-1\nprotocol: p\n\
             artifacts: {{a: {{revision: r1}}, up: {{revision: {up}}}}}\n"
        ))
        .expect("case reads");
        let decision = eval::evaluate(&ir, &case, std::slice::from_ref(&record)).expect("decides");
        let entry = decision
            .claims
            .get(&ClaimId::new("absent"))
            .expect("declared")
            .clone();
        let listed: Vec<String> = entry
            .excluded_evidence
            .iter()
            .map(|e| format!("{}:{}", e.evidence.as_str(), e.reason.as_str()))
            .collect();
        (entry.value, listed)
    };
    assert_eq!(entry("u0"), (Truth::False, vec![]));
    assert_eq!(
        entry("u1"),
        (Truth::Unknown, vec!["e1:invalidated".to_owned()])
    );
}

/// The refusal order the validator docs give: unresolved references (step 5), then
/// `inert-invalidation` (step 6), then claim cycles (step 7). Pass-2 decision 1: a claim is inert
/// only when it reaches no evidence match, directly or through the claims it tests, so `on`, built
/// on `base`, is not; `c1`, whose cycle reaches none, is.
#[test]
fn inert_invalidation_comes_between_references_and_cycles() {
    let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\nevidence_kinds: {k: {}}\n\
        claims:\n\
        \x20\x20base: {true_when: {evidence: {kind: k}}}\n\
        \x20\x20on: {true_when: {claim: base}}\n\
        \x20\x20c1: {true_when: {claim: c2}}\n\
        \x20\x20c2: {true_when: {claim: c1}}\n\
        outcomes: {o: {requires: {claim: gone}}}\n\
        invalidation: {r: {upstream: a, invalidates: [base, on, c1]}}\n";
    assert_eq!(
        problems(source),
        [
            "undeclared-claim: outcome `o` references claim `gone`, which is not declared",
            "inert-invalidation: invalidation rule `r` invalidates claim `c1`, which reaches no evidence match",
            "claim-cycle: claims test each other in a cycle: c1 -> c2 -> c1",
        ]
    );
}

/// A dimension with five upstream artifacts and one class has `2^(1 * 2^5)` values, named with its
/// upstream artifacts in identifier order; one with 32 has a vector count no `u32` holds, and the
/// refusal still names it.
#[test]
fn the_bound_refusal_names_many_upstream_artifacts() {
    let protocol = |n: usize| {
        let artifacts: Vec<String> = (0..n).map(|i| format!("u{i:02}: {{}}")).collect();
        let claims: Vec<String> = (0..n)
            .map(|i| format!("c{i:02}: {{true_when: {{evidence: {{kind: k}}}}}}"))
            .collect();
        let rules: Vec<String> = (0..n)
            .rev()
            .map(|i| format!("r{i:02}: {{upstream: u{i:02}, invalidates: [c{i:02}]}}"))
            .collect();
        format!(
            "format: protocol/1\nprotocol: {{id: p, revision: 1}}\n\
             artifacts: {{a: {{}}, {}}}\nevidence_kinds: {{k: {{}}}}\n\
             claims: {{{}}}\ninvalidation: {{{}}}\n",
            artifacts.join(", "),
            claims.join(", "),
            rules.join(", ")
        )
    };
    assert_eq!(
        check(&compiled(&protocol(4)), None)
            .expect("2^(1 * 2^4) is the bound")
            .states,
        65536
    );
    let refusal = check(&compiled(&protocol(5)), None).expect_err("refused");
    assert_eq!(
        refusal.to_string(),
        "protocol `p` revision 1 has 4294967296 states, more than the bound of 65536: evidence \
         kind `k` with upstream `u00`, `u01`, `u02`, `u03`, `u04` 4294967296"
    );
    let refusal = check(&compiled(&protocol(32)), None).expect_err("refused");
    assert_eq!(refusal.code(), "state-space-bound");
    assert!(refusal.to_string().ends_with(" 2^(1 * 2^32)"), "{refusal}");
}
