//! What evaluation costs when every claim of a chain carries an invalidation rule
//! (story:wall-clock-ratio-tests-under-load), held as a count of work done, not as time: a count
//! does not move with the load of the machine.
//!
//! Adversary pass 2 of story:review-hardening-w7 timed a chain of claims with one invalidation
//! rule each: 0.37 s at 50 claims, 84 s at 400, about cubic. The chain here is that shape: claim
//! `c_i` tests `c_{i+1}`, the last claim reads the one evidence kind `k`, rule `r_i` invalidates
//! `c_i`, and one record of kind `k` is moved per rule. Every claim has a context of its own, so a
//! walk starts in each, and the walk from `c_i` decides the `n - i` claims from `c_i` down: `n(n+1)/2`
//! claim evaluations in all, and as many marks of a moved record as kept out. That is the
//! bound asserted, for the chain and for a diamond of the same rules (`c_i` tests `c_{i+1}` and
//! `c_{i+2}`), which holds the walk to reusing a claim's value within it: not reused, the last
//! claims of the diamond are reached a Fibonacci number of times.
//!
//! The counters are bumped by the walk (`claims.rs`) on the thread that evaluates, which is the
//! worker of `on_deep_stack`, so each test thread has its own and tests running side by side do
//! not count each other's work. A test sets a limit before it evaluates; a count past it panics,
//! so an evaluation that does not end ends there.

use std::cell::Cell;

use super::{Decision, on_deep_stack};
use crate::ir::{self, Ir};
use crate::model::{self, EvidenceRecord, Truth};

#[derive(Clone, Copy)]
struct Work {
    /// Claim predicates evaluated, one per claim and context the walk reaches.
    evaluated: usize,
    /// Moved records marked as kept out of a walk, or marked as no longer kept out.
    marked: usize,
    /// The most of either the walk may do before it panics.
    limit: usize,
}

thread_local! {
    static WORK: Cell<Work> = const {
        Cell::new(Work { evaluated: 0, marked: 0, limit: usize::MAX })
    };
}

/// A claim predicate is evaluated.
pub(super) fn evaluated() {
    WORK.with(|work| {
        let mut now = work.get();
        now.evaluated += 1;
        work.set(now);
        assert!(
            now.evaluated <= now.limit,
            "more than {} claim evaluations",
            now.limit
        );
    });
}

/// `places` moved records are marked as kept out of a walk, or as no longer kept out.
pub(super) fn marked(places: usize) {
    WORK.with(|work| {
        let mut now = work.get();
        now.marked += places;
        work.set(now);
        assert!(
            now.marked <= now.limit,
            "more than {} marks of a moved record",
            now.limit
        );
    });
}

/// The claim evaluations and marks of one evaluation of `ir`, which panics when either passes
/// `limit`.
fn work_of(
    ir: &Ir,
    case: &model::Case,
    evidence: &[EvidenceRecord],
    limit: usize,
) -> (usize, usize, Decision) {
    start(limit);
    let decision = super::evaluate(ir, case, evidence).expect("decides");
    let counted = on_deep_stack(|| WORK.with(Cell::get));
    start(usize::MAX);
    (counted.evaluated, counted.marked, decision)
}

/// Counts from nothing on this thread's worker, which panics at `limit`.
fn start(limit: usize) {
    let fresh = Work {
        evaluated: 0,
        marked: 0,
        limit,
    };
    on_deep_stack(|| WORK.with(|work| work.set(fresh)));
}

fn claim(at: usize) -> String {
    format!("c{at:05}")
}

fn upstream(at: usize) -> String {
    format!("u{at:05}")
}

/// A chain or a diamond of `claims` claims kept apart by one invalidation rule each: claim `c_i`
/// is true when `tests(i)` holds, the last reads kind `k`, and one record of kind `k` is moved per
/// rule (it records an earlier revision of the rule's upstream artifact). One more record of kind
/// `k` records no upstream revision, so every claim is `true`.
fn invalidated(
    claims: usize,
    tests: impl Fn(usize) -> String,
) -> (Ir, model::Case, Vec<EvidenceRecord>) {
    let mut source =
        String::from("format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts:\n  a: {}\n");
    for at in 0..claims {
        source.push_str(&format!("  {}: {{}}\n", upstream(at)));
    }
    source.push_str("evidence_kinds: {k: {}}\nclaims:\n");
    for at in 0..claims - 1 {
        source.push_str(&format!("  {}: {{true_when: {}}}\n", claim(at), tests(at)));
    }
    source.push_str(&format!(
        "  {}: {{true_when: {{evidence: {{kind: k}}}}}}\n",
        claim(claims - 1)
    ));
    source.push_str("invalidation:\n");
    for at in 0..claims {
        source.push_str(&format!(
            "  r{at:05}: {{upstream: {}, invalidates: [{}]}}\n",
            upstream(at),
            claim(at)
        ));
    }
    let ir = ir::compile(&model::parse(&source).expect("parses")).expect("compiles");

    let mut artifacts = String::from("a: {revision: r1}");
    for at in 0..claims {
        artifacts.push_str(&format!(", {}: {{revision: new}}", upstream(at)));
    }
    let case = super::read_case(&format!(
        "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {{{artifacts}}}\n"
    ))
    .expect("case reads");

    let record = |id: &str, upstream: Option<String>| {
        let upstream = upstream
            .map(|artifact| format!("upstream_revisions: {{{artifact}: old}}\n"))
            .unwrap_or_default();
        super::read_evidence(&format!(
            "format: canon-evidence/1\nid: {id}\nkind: k\nsubject: a\nsubject_revision: r1\n\
             {upstream}"
        ))
        .expect("evidence reads")
    };
    let mut evidence = vec![record("base", None)];
    for at in 0..claims {
        evidence.push(record(&format!("e{at:05}"), Some(upstream(at))));
    }
    (ir, case, evidence)
}

fn chain(claims: usize) -> (Ir, model::Case, Vec<EvidenceRecord>) {
    invalidated(claims, |at| format!("{{claim: {}}}", claim(at + 1)))
}

fn diamond(claims: usize) -> (Ir, model::Case, Vec<EvidenceRecord>) {
    invalidated(claims, |at| {
        if at + 2 < claims {
            format!(
                "{{all: [{{claim: {}}}, {{claim: {}}}]}}",
                claim(at + 1),
                claim(at + 2)
            )
        } else {
            format!("{{claim: {}}}", claim(at + 1))
        }
    })
}

/// The work of evaluating a chain of `claims` claims, with the limit of one more than the bound:
/// `n(n+1)/2` claim evaluations and as many marks as there are places kept out and put back.
fn chain_work(claims: usize) -> (usize, usize) {
    let (ir, case, evidence) = chain(claims);
    let bound = claims * (claims + 1) / 2;
    let (evaluated, marked, decision) = work_of(&ir, &case, &evidence, 2 * bound);
    assert_eq!(decision.claims.iter().count(), claims);
    assert!(
        decision
            .claims
            .iter()
            .all(|(_, entry)| entry.value == Truth::True),
        "the record that recorded no upstream revision decides every claim"
    );
    (evaluated, marked)
}

/// The name is the adversary's and is kept so it can be traced; the bound is quadratic, since the
/// excluded evidence the decision lists has `n(n+1)/2` entries by itself. A chain of 400
/// claims is held to the claim evaluations and marks of a chain of 50 claims scaled by 64 (the
/// square of the ratio of the lengths, rounded up), and each chain to `n(n+1)/2` evaluations.
#[test]
fn claims_kept_apart_by_invalidation_rules_are_evaluated_in_work_quadratic_in_the_chain() {
    let (short, short_marks) = chain_work(50);
    let (long, long_marks) = chain_work(400);
    eprintln!("invalidated chain: 50 claims {short} evaluations, 400 claims {long}");
    assert!(
        short <= 50 * 51 / 2,
        "a chain of 50 claims took {short} claim evaluations"
    );
    assert!(
        long <= 400 * 401 / 2,
        "a chain of 400 claims took {long} claim evaluations"
    );
    assert!(
        long <= short * 64,
        "a chain eight times as long took {long} claim evaluations, against {short}: more than \
         quadratic growth"
    );
    assert!(
        long_marks <= short_marks * 64,
        "a chain eight times as long made {long_marks} marks, against {short_marks}: more than \
         quadratic growth"
    );
}

/// A diamond of 500 claims, each with a rule of its own, so that every walk has a context of its
/// own and reuses values only through its per-walk memo. Reused, the claims evaluated are `n(n+1)/2`;
/// not reused, the last claims are reached a Fibonacci number of times (about 10^104 for
/// 500 claims) and the evaluation would never end, so the limit ends it.
#[test]
fn a_diamond_of_claims_kept_apart_by_invalidation_rules_reuses_each_claims_value_within_a_walk() {
    let claims = 500;
    let bound = claims * (claims + 1) / 2;
    let (ir, case, evidence) = diamond(claims);
    let (evaluated, _, decision) = work_of(&ir, &case, &evidence, 2 * bound);
    eprintln!("invalidated diamond: {claims} claims {evaluated} evaluations");
    assert_eq!(decision.claims.iter().count(), claims);
    assert!(
        evaluated <= bound,
        "a diamond of {claims} claims took {evaluated} claim evaluations, over {bound}"
    );
}
