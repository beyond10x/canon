//! Adversary pass 2 on story:review-hardening-w7: the effective-depth bound of `eval/depth.rs`.
//!
//! `depth.rs` computes each claim's effective depth once, after the claims it tests, and reuses
//! it when a claim is reached again (a diamond). These cases hold that reuse to the bound the
//! module docs state ("a claim test at level `n` reaches `n` plus the effective depth of the claim
//! it tests"), whichever way a claim is reached first, and check that evaluation of a diamond and
//! of claims kept apart by invalidation rules ends.

use std::time::Instant;

use b10x_canon::eval;
use b10x_canon::ir::{self, Ir};
use b10x_canon::model::{self, ClaimId, Predicate};

fn compiled(source: &str) -> Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

fn case(artifacts: &str) -> model::Case {
    eval::read_case(&format!(
        "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {{{artifacts}}}\n"
    ))
    .expect("case reads")
}

fn claim(at: usize) -> String {
    format!("c{at:05}")
}

/// `r: any [d, not … not d]`: `d` reached first by a test at level 2, then again by a test at
/// level `2 + nots` (the order `canon compile` gives too: `claim` sorts before `not`). `d` tests a
/// chain of `links` claims ending at evidence, so its effective depth is `links + 1`. The second
/// test decides `r`'s effective depth: `2 + nots + links + 1`.
fn reached_again(links: usize, nots: usize) -> Ir {
    let mut source = String::from(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {}}\nclaims:\n  r: {true_when: {claim: d}}\n  \
         d: {true_when: {claim: c00000}}\n",
    );
    for at in 0..links - 1 {
        source.push_str(&format!(
            "  {}: {{true_when: {{claim: {}}}}}\n",
            claim(at),
            claim(at + 1)
        ));
    }
    source.push_str(&format!(
        "  {}: {{true_when: {{evidence: {{kind: k}}}}}}\n",
        claim(links - 1)
    ));
    let mut built = compiled(&source);
    let r = built.claims.get_mut(&ClaimId::new("r")).expect("r");
    let test = r.true_when.clone();
    let mut deeper = test.clone();
    for _ in 0..nots {
        deeper = Predicate::Not(Box::new(deeper));
    }
    r.true_when = Predicate::Any(vec![test, deeper]);
    built
}

/// `r` tests `d` first at level 2 and again under `nots` `not`s. The first visit measures `d`;
/// the second must use that measure at its own, deeper level, so `r` reaches
/// `2 + nots + links + 1`: at 4096 it is evaluated, at 4097 refused. A check that measures `d`
/// only where it was first reached sees `2 + links + 1` and lets both through.
#[test]
fn a_claim_reached_again_through_a_deeper_test_is_measured_through_it() {
    let links = 3000;
    let at_bound = eval::MAX_IR_DEPTH - (2 + links + 1);
    let within = reached_again(links, at_bound);
    eval::evaluate(&within, &case("a: {revision: r}"), &[])
        .unwrap_or_else(|refusal| panic!("4096 levels through the second test: {refusal}"));
    let beyond = reached_again(links, at_bound + 1);
    let refused = eval::evaluate(&beyond, &case("a: {revision: r}"), &[])
        .expect_err("4097 levels through the second test are refused");
    assert_eq!(refused.code(), "predicate-too-deep", "{refused}");
    assert!(
        refused
            .to_string()
            .starts_with("claim `r` true_when nests predicates deeper than 4096 through the claims it tests: r -> d -> c00000 -> c00001 -> "),
        "{refused}"
    );
}

/// A diamond within the bound: 2000 claims, each testing the next two, compiled from a protocol.
/// There are about 2^2000 paths from the first claim to the last; evaluation and its explanation
/// must visit each claim a bounded number of times, so the evaluation ends with every claim decided.
#[test]
fn a_diamond_within_the_bound_is_evaluated() {
    let claims = 2000;
    let mut source = String::from(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {}}\nclaims:\n",
    );
    for at in 0..claims {
        let predicate = if at + 2 <= claims {
            format!(
                "{{all: [{{claim: {}}}, {{claim: {}}}]}}",
                claim(at + 1),
                claim((at + 2).min(claims))
            )
        } else {
            format!("{{claim: {}}}", claim(claims))
        };
        source.push_str(&format!("  {}: {{true_when: {predicate}}}\n", claim(at)));
    }
    source.push_str(&format!(
        "  {}: {{true_when: {{evidence: {{kind: k}}}}}}\n",
        claim(claims)
    ));
    let built = compiled(&source);
    let started = Instant::now();
    let decision =
        eval::evaluate(&built, &case("a: {revision: r}"), &[]).expect("within the bound");
    assert_eq!(decision.claims.iter().count(), claims + 1);
    assert!(
        decision
            .claims
            .iter()
            .all(|(_, entry)| entry.value == model::Truth::Unknown),
        "no evidence: every claim is unknown"
    );
    eprintln!("diamond of {claims}: {:?}", started.elapsed());
}

// claims_kept_apart_by_invalidation_rules_are_evaluated_in_time_linear_in_the_chain and its helper invalidated_chain moved to story:invalidation-evaluation-cost.
