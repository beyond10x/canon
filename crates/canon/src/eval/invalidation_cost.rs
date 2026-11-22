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
//!
//! A count of evaluations does not see an evaluation made slower: a copy of the kept-out marks, a
//! scan of the claims being decided, each made for every evaluation, leave the count as it is
//! while the cost goes cubic (wave 11 review, F7). Copies are held by the bytes the evaluating
//! thread asks of the allocator, which are the same on every run: per evaluation they do not grow
//! with the chain. A scan allocates nothing and no counter sees it, since the code that scans is
//! not code that bumps one; a time bound that would see it is not held here, since at the sizes a
//! test can afford (the scan costs about half an evaluation at 1600 claims) the processor time
//! of the evaluating thread moves about as much with the machine as the scan adds.

#![allow(unsafe_code)]

use std::alloc::{GlobalAlloc, Layout, System};
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

thread_local! {
    /// The bytes this thread has asked the allocator for, never reduced by a free: what the
    /// evaluation copies is counted, not what it holds at the end.
    static ALLOCATED: Cell<usize> = const { Cell::new(0) };
}

fn allocated(bytes: usize) {
    let _ = ALLOCATED.try_with(|total| total.set(total.get() + bytes));
}

/// The system allocator, counting the bytes each thread asks of it. The lib's test binary runs on
/// it, which only adds one thread-local add to an allocation; nothing reads the count but
/// [`work_of`].
struct Counting;

// SAFETY: every method does what `System` does and only adds to a thread-local count first.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        allocated(layout.size());
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        allocated(layout.size());
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        allocated(new_size);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static COUNTING: Counting = Counting;

/// What one evaluation of `ir` did on the evaluating thread.
#[derive(Clone, Copy, Debug)]
struct Measured {
    /// Claim predicates evaluated.
    evaluated: usize,
    /// Moved records marked.
    marked: usize,
    /// Bytes asked of the allocator: a copy made for each evaluation, such as a copy of the
    /// kept-out marks, shows here in proportion to its size, which no count of evaluations sees.
    bytes: usize,
}

/// One evaluation of `ir`, measured on the worker thread that runs it, which panics when the
/// evaluations or the marks pass `limit`.
fn work_of(
    ir: &Ir,
    case: &model::Case,
    evidence: &[EvidenceRecord],
    limit: usize,
) -> (Measured, Decision) {
    start(limit);
    let bytes_before = on_deep_stack(|| ALLOCATED.with(Cell::get));
    let decision = super::evaluate(ir, case, evidence).expect("decides");
    let bytes_after = on_deep_stack(|| ALLOCATED.with(Cell::get));
    let counted = on_deep_stack(|| WORK.with(Cell::get));
    start(usize::MAX);
    let measured = Measured {
        evaluated: counted.evaluated,
        marked: counted.marked,
        bytes: bytes_after - bytes_before,
    };
    (measured, decision)
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

/// The work of evaluating a chain of `claims` claims, with a limit of twice the bound: `n(n+1)/2`
/// claim evaluations and as many marks as there are places kept out and put back.
fn chain_work(claims: usize) -> Measured {
    let (ir, case, evidence) = chain(claims);
    let bound = claims * (claims + 1) / 2;
    let (measured, decision) = work_of(&ir, &case, &evidence, 2 * bound);
    assert_eq!(decision.claims.iter().count(), claims);
    assert!(
        decision
            .claims
            .iter()
            .all(|(_, entry)| entry.value == Truth::True),
        "the record that recorded no upstream revision decides every claim"
    );
    measured
}

/// Whether `large` asked the allocator for no more than half as much again per claim evaluation as
/// `small`: each evaluation costs the same whatever the length of the walk, and a copy made for
/// each evaluation (of the marks of the moved records, of the claims being decided) grows with it.
/// The bytes are the same on every run, so the bound is exact and no load moves it.
fn bytes_per_evaluation_held(small: Measured, large: Measured) -> bool {
    2 * large.bytes * small.evaluated <= 3 * small.bytes * large.evaluated
}

/// The name is the adversary's and is kept so it can be traced; the bound is quadratic, since the
/// excluded evidence the decision lists has `n(n+1)/2` entries by itself. A chain of 400
/// claims is held to the claim evaluations and marks of a chain of 50 claims scaled by 64 (the
/// square of the ratio of the lengths, rounded up), and each chain to `n(n+1)/2` evaluations.
/// What each evaluation costs is held by the bytes it asks of the allocator, per evaluation, of a
/// chain of 100 claims against one of 400: a count of evaluations does not see an evaluation made
/// slower (a copy of the kept-out marks for each, which cost an evaluation `n` bytes more).
#[test]
fn claims_kept_apart_by_invalidation_rules_are_evaluated_in_work_quadratic_in_the_chain() {
    let (short, middle, long) = (chain_work(50), chain_work(100), chain_work(400));
    eprintln!(
        "invalidated chain: 50 claims {} evaluations, 400 claims {}; bytes per evaluation: 100 \
         claims {}, 400 claims {}",
        short.evaluated,
        long.evaluated,
        middle.bytes / middle.evaluated,
        long.bytes / long.evaluated
    );
    assert!(
        short.evaluated <= 50 * 51 / 2,
        "a chain of 50 claims took {} claim evaluations",
        short.evaluated
    );
    assert!(
        long.evaluated <= 400 * 401 / 2,
        "a chain of 400 claims took {} claim evaluations",
        long.evaluated
    );
    assert!(
        long.evaluated <= short.evaluated * 64,
        "a chain eight times as long took {} claim evaluations, against {}: more than quadratic \
         growth",
        long.evaluated,
        short.evaluated
    );
    assert!(
        long.marked <= short.marked * 64,
        "a chain eight times as long made {} marks, against {}: more than quadratic growth",
        long.marked,
        short.marked
    );
    assert!(
        bytes_per_evaluation_held(middle, long),
        "a chain of 400 claims asked the allocator for {} bytes per claim evaluation, a chain of \
         100 claims for {}: an evaluation grows with the chain",
        long.bytes / long.evaluated,
        middle.bytes / middle.evaluated
    );
}

/// A diamond of 500 claims, each with a rule of its own, so that every walk has a context of its
/// own and reuses values only through its per-walk memo. Reused, the claims evaluated are
/// `n(n+1)/2`; not reused, the last claims are reached a Fibonacci number of times (about 10^104
/// for 500 claims) and the evaluation would never end, so the limit ends it. Each evaluation asks
/// the allocator for no more bytes than one of a diamond of 100 claims, as the chain test holds.
#[test]
fn a_diamond_of_claims_kept_apart_by_invalidation_rules_reuses_each_claims_value_within_a_walk() {
    let measure = |claims: usize| {
        let (ir, case, evidence) = diamond(claims);
        let bound = claims * (claims + 1) / 2;
        let (measured, decision) = work_of(&ir, &case, &evidence, 2 * bound);
        assert_eq!(decision.claims.iter().count(), claims);
        assert!(
            measured.evaluated <= bound,
            "a diamond of {claims} claims took {} claim evaluations, over {bound}",
            measured.evaluated
        );
        measured
    };
    let (small, large) = (measure(100), measure(500));
    eprintln!(
        "invalidated diamond: 500 claims {} evaluations; bytes per evaluation: 100 claims {}, 500 \
         claims {}",
        large.evaluated,
        small.bytes / small.evaluated,
        large.bytes / large.evaluated
    );
    assert!(
        bytes_per_evaluation_held(small, large),
        "a diamond of 500 claims asked the allocator for {} bytes per claim evaluation, one of 100 \
         claims for {}: an evaluation grows with the walk",
        large.bytes / large.evaluated,
        small.bytes / small.evaluated
    );
}
