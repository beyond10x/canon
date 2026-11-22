//! A caller that evaluates many small cases gets the results of one big evaluation, from a thread
//! with a small stack (story:evaluate-without-thread-per-call).
//!
//! `eval::evaluate_with` runs its recursive steps on a thread with a stack of its own (the
//! module's `DEEP_STACK`) so the depth a protocol may nest does not depend on the caller's stack.
//! The number of threads that takes for many calls is counted by the unit test
//! `eval::tests::two_thousand_calls_spawn_at_most_one_thread`; this file holds what a caller sees:
//! the same decisions, from a small stack, and a protocol nested deep still evaluates from it.

use std::thread;

use b10x_canon::eval::{self, Supplied};
use b10x_canon::ir::{self, Ir};
use b10x_canon::model::{
    self, Claim, ClaimId, Decision, Declarations, EvidenceRecord, Predicate, Truth,
};

/// The calls each test makes, and the claims the evidence is spread over.
const CALLS: usize = 2000;
const CLAIMS: usize = 50;

/// The stack of the caller's thread: far below the 2 MiB of a spawned thread.
const SMALL_STACK: usize = 256 << 10;

fn protocol() -> Ir {
    let mut source =
        String::from("format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n");
    source.push_str("evidence_kinds:\n");
    for at in 0..CLAIMS {
        source.push_str(&format!("  k{at:02}: {{}}\n"));
    }
    source.push_str("claims:\n");
    for at in 0..CLAIMS {
        source.push_str(&format!(
            "  c{at:02}: {{true_when: {{evidence: {{kind: k{at:02}, result: pass}}}}}}\n"
        ));
    }
    ir::compile(&model::parse(&source).expect("parses")).expect("compiles")
}

fn case() -> model::Case {
    eval::read_case("format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r1}}\n")
        .expect("case reads")
}

/// Record `at` passes the claim `c(at % CLAIMS)`.
fn records() -> Vec<EvidenceRecord> {
    (0..CALLS)
        .map(|at| {
            eval::read_evidence(&format!(
                "format: canon-evidence/1\nid: e{at:04}\nkind: k{:02}\nresult: pass\n\
                 subject: a\nsubject_revision: r1\n",
                at % CLAIMS
            ))
            .expect("evidence reads")
        })
        .collect()
}

/// Every record on its own, one call each, from a thread whose stack is `stack` bytes. The
/// decisions come back as the canonical documents `canon evaluate` prints.
fn one_call_per_record(
    ir: &Ir,
    case: &model::Case,
    records: &[EvidenceRecord],
    stack: usize,
) -> Vec<(Decision, String)> {
    thread::scope(|scope| {
        thread::Builder::new()
            .stack_size(stack)
            .spawn_scoped(scope, || {
                records
                    .iter()
                    .map(|record| {
                        let decision = eval::evaluate_with(
                            ir,
                            case,
                            std::slice::from_ref(record),
                            Supplied::default(),
                        )
                        .expect("decides");
                        let text = eval::render(&decision);
                        (decision, text)
                    })
                    .collect()
            })
            .expect("the caller's thread starts")
            .join()
            .expect("the caller's thread ends normally")
    })
}

fn value(decision: &Decision, claim: &ClaimId) -> Truth {
    decision
        .claims
        .get(claim)
        .expect("the claim is decided")
        .value
}

/// 2000 one-record calls from a 256 KiB thread give what the same calls give from a thread with
/// a large stack, and each says about its claim what one 2000-record evaluation says.
#[test]
fn two_thousand_one_record_calls_equal_one_two_thousand_record_call() {
    let ir = protocol();
    let case = case();
    let records = records();

    let at_once = eval::evaluate(&ir, &case, &records).expect("decides");
    let small = one_call_per_record(&ir, &case, &records, SMALL_STACK);
    let large = one_call_per_record(&ir, &case, &records, 64 << 20);

    assert_eq!(small.len(), CALLS);
    for (at, ((decision, text), (_, large_text))) in small.iter().zip(&large).enumerate() {
        assert_eq!(
            text, large_text,
            "call {at}: small and large caller stack differ"
        );
        let claim = ClaimId::new(format!("c{:02}", at % CLAIMS));
        assert_eq!(
            value(&decision, &claim),
            value(&at_once, &claim),
            "call {at}: the value of {claim:?}"
        );
        let other = ClaimId::new(format!("c{:02}", (at + 1) % CLAIMS));
        assert_ne!(
            value(&decision, &other),
            value(&at_once, &other),
            "call {at}: a record of another claim is not evidence for {other:?}"
        );
    }
}

/// A protocol nested deep still evaluates from a thread with a small stack: the stack the
/// recursion needs is the worker's, not the caller's.
#[test]
fn a_deep_protocol_evaluates_from_a_small_stack() {
    let deep = thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(|| {
            let mut protocol = model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
                 evidence_kinds: {k: {}}\n\
                 claims: {c: {true_when: {evidence: {kind: k, result: pass}}}}\n",
            )
            .expect("parses");
            let (_, claim) = protocol.claims.iter().next().expect("c");
            let mut predicate = claim.true_when.clone();
            for _ in 0..1500 {
                predicate = Predicate::Not(Box::new(predicate));
            }
            protocol.claims = Declarations::new(vec![(
                ClaimId::new("c"),
                Claim {
                    description: None,
                    true_when: predicate,
                },
            )]);
            ir::compile(&protocol).expect("compiles")
        })
        .expect("thread")
        .join()
        .expect("built");
    let case = case();
    let (deep, decided) = thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(move || {
            let decided = (0..20)
                .map(|_| {
                    eval::evaluate(&deep, &case, &[])
                        .map(|decision| decision.claims.len())
                        .map_err(|refusal| refusal.to_string())
                })
                .collect::<Vec<_>>();
            (deep, decided)
        })
        .expect("the caller's thread starts")
        .join()
        .expect("the caller's thread ends normally");
    assert!(decided.iter().all(|claims| *claims == Ok(1)), "{decided:?}");
    // Dropped here: the test thread's stack holds the drop of this depth; the small one might not.
    drop(deep);
}
