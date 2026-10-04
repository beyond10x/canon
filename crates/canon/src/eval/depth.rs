//! The depth bound on a compiled protocol a caller hands the evaluator (story:review-hardening-w7).
//!
//! Evaluation recurses along the nesting of a predicate and, through each claim test, into the
//! predicate of the claim it tests, so the stack it needs follows the effective depth of a claim:
//! its predicate's levels, where a claim test at level `n` reaches `n` plus the effective depth of
//! the claim it tests. A predicate is one level, and each `all`, `any` or `not` around it one more.
//! [`check`] refuses, as `predicate-too-deep`, a predicate that nests more than [`MAX_IR_DEPTH`]
//! levels itself (`` claim `c` true_when nests predicates deeper than 4096 ``) and a claim whose
//! effective depth is more than [`MAX_IR_DEPTH`], naming the claims of the chain that reaches it,
//! from that claim to the last one tested (`` claim `c1` true_when nests predicates deeper than
//! 4096 through the claims it tests: c1 -> c2 -> c3 ``). Claims that test each other in a cycle
//! have no effective depth; they are refused as `claim-cycle`, naming the claims from the first
//! one reached again (`claims test each other in a cycle: c -> d -> c`), in the order evaluation
//! would reach them. Everything here is walked with stacks of its own, never by recursion, and
//! each claim's effective depth is computed once, after the claims it tests.

use std::collections::BTreeMap;

use super::Refusal;
use super::read::MAX_IR_DEPTH;
use crate::ir::Ir;
use crate::model::{ClaimId, OutcomeRequirement, Predicate, one_line};

/// Checks every predicate of `ir`, then the claims' effective depths, as the module docs say.
pub(crate) fn check(ir: &Ir) -> Result<(), Refusal> {
    let too_deep = |owner: &str, id: &str, field: &str| {
        Refusal::new(
            "predicate-too-deep",
            format!(
                "{owner} `{}` {field} nests predicates deeper than {MAX_IR_DEPTH}",
                one_line(id)
            ),
        )
    };
    let mut claims = BTreeMap::new();
    for (id, claim) in &ir.claims {
        let own =
            own(&claim.true_when).ok_or_else(|| too_deep("claim", id.as_str(), "true_when"))?;
        claims.insert(id, own);
    }
    let others = ir
        .obligations
        .iter()
        .map(|(id, obligation)| {
            (
                "obligation",
                id.as_str(),
                "discharged_when",
                &obligation.discharged_when,
            )
        })
        .chain(
            ir.actions
                .iter()
                .map(|(id, action)| ("action", id.as_str(), "precondition", &action.precondition)),
        )
        .chain(
            ir.outcomes
                .iter()
                .filter_map(|(id, outcome)| match &outcome.requires {
                    OutcomeRequirement::Predicate(requires) => {
                        Some(("outcome", id.as_str(), "requires", requires))
                    }
                    OutcomeRequirement::Decision(_) => None,
                }),
        );
    for (owner, id, field, predicate) in others {
        own(predicate).ok_or_else(|| too_deep(owner, id, field))?;
    }
    effective_depths(&claims)
}

/// One claim's own predicate: its deepest level, and each claim test in it with the level the test
/// stands at, in the order evaluation reaches them.
struct Own<'a> {
    deepest: usize,
    tests: Vec<(usize, &'a ClaimId)>,
}

/// `predicate` walked in evaluation order (each member of `all` and `any` in turn, depth first), or
/// `None` once a level beyond [`MAX_IR_DEPTH`] is reached.
fn own(predicate: &Predicate) -> Option<Own<'_>> {
    let mut own = Own {
        deepest: 0,
        tests: Vec::new(),
    };
    let mut pending = vec![(predicate, 1usize)];
    while let Some((node, level)) = pending.pop() {
        if level > MAX_IR_DEPTH {
            return None;
        }
        own.deepest = own.deepest.max(level);
        match node {
            Predicate::All(members) | Predicate::Any(members) => {
                pending.extend(members.iter().rev().map(|member| (member, level + 1)));
            }
            Predicate::Not(inner) => pending.push((inner, level + 1)),
            Predicate::Claim(test) => own.tests.push((level, &test.claim)),
            Predicate::Evidence(_) => {}
        }
    }
    Some(own)
}

/// A claim being walked: the next of its tests to follow, the level of the test that reached it,
/// and the deepest it reaches so far with the claim tested on that way.
struct Frame<'a> {
    id: &'a ClaimId,
    next: usize,
    entered_at: usize,
    depth: usize,
    via: Option<&'a ClaimId>,
}

/// Each claim's effective depth, computed once, after the claims it tests, from the claims in
/// identifier order; refuses a cycle or a depth beyond the bound.
fn effective_depths(claims: &BTreeMap<&ClaimId, Own<'_>>) -> Result<(), Refusal> {
    let mut done: BTreeMap<&ClaimId, (usize, Option<&ClaimId>)> = BTreeMap::new();
    for (&root, own) in claims {
        if done.contains_key(root) {
            continue;
        }
        let mut frames = vec![Frame {
            id: root,
            next: 0,
            entered_at: 0,
            depth: own.deepest,
            via: None,
        }];
        // Where each claim being walked stands in `frames`, so a cycle is found without a scan.
        let mut open: BTreeMap<&ClaimId, usize> = BTreeMap::from([(root, 0)]);
        while let Some(top) = frames.len().checked_sub(1) {
            let next_test = claims[frames[top].id].tests.get(frames[top].next).copied();
            if let Some((level, tested)) = next_test {
                frames[top].next += 1;
                // A claim the IR does not declare is `unknown`; nothing is evaluated below it.
                let Some(tested_own) = claims.get(tested) else {
                    continue;
                };
                if let Some(&(below, _)) = done.get(tested) {
                    if level + below > frames[top].depth {
                        frames[top].depth = level + below;
                        frames[top].via = Some(tested);
                    }
                    continue;
                }
                if let Some(&start) = open.get(tested) {
                    let path: Vec<String> = frames[start..]
                        .iter()
                        .map(|frame| one_line(frame.id.as_str()))
                        .chain([one_line(tested.as_str())])
                        .collect();
                    return Err(Refusal::new(
                        "claim-cycle",
                        format!("claims test each other in a cycle: {}", path.join(" -> ")),
                    ));
                }
                let deepest = tested_own.deepest;
                open.insert(tested, frames.len());
                frames.push(Frame {
                    id: tested,
                    next: 0,
                    entered_at: level,
                    depth: deepest,
                    via: None,
                });
                continue;
            }
            let finished = frames.pop().expect("the frame just read");
            open.remove(finished.id);
            if finished.depth > MAX_IR_DEPTH {
                let mut chain = vec![one_line(finished.id.as_str())];
                let mut next = finished.via;
                while let Some(id) = next {
                    chain.push(one_line(id.as_str()));
                    next = done.get(id).and_then(|(_, via)| *via);
                }
                return Err(Refusal::new(
                    "predicate-too-deep",
                    format!(
                        "claim `{}` true_when nests predicates deeper than {MAX_IR_DEPTH} through \
                         the claims it tests: {}",
                        one_line(finished.id.as_str()),
                        chain.join(" -> ")
                    ),
                ));
            }
            done.insert(finished.id, (finished.depth, finished.via));
            if let Some(parent) = frames.last_mut() {
                let through = finished.entered_at + finished.depth;
                if through > parent.depth {
                    parent.depth = through;
                    parent.via = Some(finished.id);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::{evaluate, read_case};
    use crate::model::{Case, ClaimTest, EvidenceKindId, EvidenceMatch, Truth};

    fn claim(at: usize) -> ClaimId {
        ClaimId::new(format!("c{at:04}"))
    }

    /// `claims` claims, each testing the next inside `nots` `not`s, the last of them reading
    /// evidence: the first claim's effective depth is `claims * (nots + 1) + 1`.
    fn chain(claims: usize, nots: usize) -> Ir {
        let mut ir = crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
                 evidence_kinds: {k: {}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles");
        let template = crate::ir::Claim {
            description: None,
            true_when: Predicate::Evidence(EvidenceMatch {
                kind: EvidenceKindId::new("k"),
                result: None,
                subject: None,
            }),
        };
        ir.claims.insert(claim(claims), template.clone());
        for at in 0..claims {
            let mut tests = Predicate::Claim(ClaimTest {
                claim: claim(at + 1),
                is: Truth::True,
            });
            for _ in 0..nots {
                tests = Predicate::Not(Box::new(tests));
            }
            let mut declared = template.clone();
            declared.true_when = tests;
            ir.claims.insert(claim(at), declared);
        }
        ir
    }

    fn case() -> Case {
        read_case("format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r}}\n")
            .expect("case reads")
    }

    /// At the bound, a chain of claims is evaluated: 4095 plain claim tests and 5 tests each 819
    /// levels deep both reach exactly 4096. One level more is refused, naming the chain.
    #[test]
    fn the_bound_holds_for_the_effective_depth_along_claim_references() {
        for (claims, nots) in [(4095, 0), (5, 818)] {
            assert_eq!(claims * (nots + 1) + 1, MAX_IR_DEPTH);
            evaluate(&chain(claims, nots), &case(), &[])
                .unwrap_or_else(|refusal| panic!("{claims} x {nots}: {refusal}"));
        }
        let refused = evaluate(&chain(5, 819), &case(), &[]).expect_err("4101 levels");
        assert_eq!(refused.code(), "predicate-too-deep");
        assert_eq!(
            refused.to_string(),
            "claim `c0000` true_when nests predicates deeper than 4096 through the claims it \
             tests: c0000 -> c0001 -> c0002 -> c0003 -> c0004 -> c0005"
        );
        let refused = evaluate(&chain(4096, 0), &case(), &[]).expect_err("4097 levels");
        assert_eq!(refused.code(), "predicate-too-deep");
        let message = refused.to_string();
        assert!(
            message.starts_with("claim `c0000` true_when nests predicates deeper than 4096 through the claims it tests: c0000 -> c0001 -> ")
                && message.ends_with(" -> c4095 -> c4096"),
            "{message}"
        );
    }

    /// A cycle of 100 000 claims, each within the bound, is refused as `claim-cycle` before any
    /// step recurses; walked by recursion it would overflow the evaluation stack.
    #[test]
    fn a_long_cycle_is_refused_without_recursion() {
        let mut ir = chain(100_000, 0);
        ir.claims
            .get_mut(&claim(100_000))
            .expect("declared")
            .true_when = Predicate::Claim(ClaimTest {
            claim: claim(0),
            is: Truth::True,
        });
        let refused = evaluate(&ir, &case(), &[]).expect_err("a cycle");
        assert_eq!(refused.code(), "claim-cycle");
        let message = refused.to_string();
        assert!(
            message.starts_with("claims test each other in a cycle: c0000 -> c0001 -> ")
                && message.ends_with(" -> c99999 -> c100000 -> c0000"),
            "{}",
            &message[..200]
        );
    }

    /// Each claim's effective depth is computed once: 3000 claims, each testing the next two, have
    /// 2^3000 paths, and the check (and the evaluation after it) still ends.
    #[test]
    fn each_claim_is_measured_once() {
        let mut ir = chain(3000, 0);
        for at in 0..2999 {
            ir.claims.get_mut(&claim(at)).expect("declared").true_when = Predicate::All(vec![
                Predicate::Claim(ClaimTest {
                    claim: claim(at + 1),
                    is: Truth::True,
                }),
                Predicate::Claim(ClaimTest {
                    claim: claim(at + 2),
                    is: Truth::True,
                }),
            ]);
        }
        // Two levels a claim, so 3000 claims reach about 6000: refused, and quickly.
        let refused = evaluate(&ir, &case(), &[]).expect_err("beyond the bound");
        assert_eq!(refused.code(), "predicate-too-deep", "{refused}");
        let within = chain(2000, 0);
        evaluate(&within, &case(), &[]).expect("within the bound");
    }
}
