//! Adversary cases for the deep-stack worker of `eval::evaluate` (story:evaluate-without-thread-per-call,
//! `crates/canon/src/eval/worker.rs`), driven through the public API only.
//!
//! The worker is kept in a thread-local of the calling thread. These cases call evaluation where
//! that thread-local is being destroyed, in each of the three orders a thread's destructors can
//! take, with a protocol too deep for the caller's small stack: a call that ran on the caller's
//! stack there would overflow it, and a call that touched the destroyed worker handle would panic
//! inside a destructor and abort. Each fails, rather than hangs, when its call does not end in
//! 20 s. The unit's own tests reach neither the fallback of `worker::run` for a destroyed worker
//! handle nor the end of a worker with its caller; on a copy of `worker.rs` these cases go red
//! when the fallback is dropped (abort: "thread local panicked on drop"), when it runs on the
//! caller's stack (SIGSEGV), and when a worker outlives its caller.

use std::cell::RefCell;
use std::sync::Arc;
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};

use b10x_canon::eval;
use b10x_canon::ir::{self, Ir};
use b10x_canon::model::{
    self, Case, Claim, ClaimId, Declarations, EvidenceRecord, Predicate, Truth,
};

/// The caller's stack: far below what 1500 nested predicates need on any stack but the worker's.
const SMALL_STACK: usize = 256 << 10;
/// How long a case waits for a call before it counts as hung.
const WAIT: Duration = Duration::from_secs(20);
/// The nesting of the deep protocol: an even number of `not`, so `c` is what its evidence says.
const LEVELS: usize = 1500;

/// A protocol whose one claim `c` is `not` nested `levels` times around one evidence match, built
/// on a thread with a large stack (the compiler recurses along the nesting too).
fn protocol(levels: usize) -> Arc<Ir> {
    thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let mut protocol = model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
                 evidence_kinds: {k: {}}\n\
                 claims: {c: {true_when: {evidence: {kind: k, result: pass}}}}\n",
            )
            .expect("parses");
            let (_, claim) = protocol.claims.iter().next().expect("c");
            let mut predicate = claim.true_when.clone();
            for _ in 0..levels {
                predicate = Predicate::Not(Box::new(predicate));
            }
            protocol.claims = Declarations::new(vec![(
                ClaimId::new("c"),
                Claim {
                    description: None,
                    true_when: predicate,
                },
            )]);
            Arc::new(ir::compile(&protocol).expect("compiles"))
        })
        .expect("thread")
        .join()
        .expect("built")
}

fn case() -> Case {
    eval::read_case("format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r1}}\n")
        .expect("case reads")
}

fn passing() -> EvidenceRecord {
    eval::read_evidence(
        "format: canon-evidence/1\nid: e1\nkind: k\nresult: pass\nsubject: a\nsubject_revision: r1\n",
    )
    .expect("evidence reads")
}

/// The value of `c` for one passing record, or the refusal as text.
fn value_of_c(ir: &Ir) -> Result<Truth, String> {
    eval::evaluate(ir, &case(), &[passing()])
        .map(|decision| {
            decision
                .claims
                .get(&ClaimId::new("c"))
                .expect("c is decided")
                .value
        })
        .map_err(|refusal| refusal.to_string())
}

/// Evaluates in its `Drop`, which runs while the thread's locals are destroyed, and sends the value.
struct EvaluatesOnDrop {
    ir: Arc<Ir>,
    report: Sender<Result<Truth, String>>,
}

impl Drop for EvaluatesOnDrop {
    fn drop(&mut self) {
        let _ = self.report.send(value_of_c(&self.ir));
    }
}

thread_local! {
    static AT_EXIT: RefCell<Option<EvaluatesOnDrop>> = const { RefCell::new(None) };
}

/// When the caller's thread evaluates before its exit, relative to setting [`AT_EXIT`].
#[derive(Clone, Copy, Debug)]
enum Order {
    /// `AT_EXIT` first, then a call: the worker's handle is destroyed before `AT_EXIT`.
    LocalThenCall,
    /// A call first, then `AT_EXIT`: `AT_EXIT` is destroyed while the worker's handle lives.
    CallThenLocal,
    /// No call before exit: the destructor makes the thread's first call.
    LocalOnly,
}

/// Runs a small-stack thread that sets [`AT_EXIT`] and calls evaluation in `order`, and returns
/// what the destructor's call gave, or why there is none.
fn evaluate_at_exit(order: Order) -> Result<Truth, String> {
    let ir = protocol(LEVELS);
    let (report, reported) = mpsc::channel();
    let (done, finished) = mpsc::channel();
    let at_exit = EvaluatesOnDrop {
        ir: Arc::clone(&ir),
        report,
    };
    let caller_ir = Arc::clone(&ir);
    let caller = thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(move || {
            let set = |at_exit| AT_EXIT.with(|slot| *slot.borrow_mut() = Some(at_exit));
            let before = match order {
                Order::LocalThenCall => {
                    set(at_exit);
                    Some(value_of_c(&caller_ir))
                }
                Order::CallThenLocal => {
                    let value = value_of_c(&caller_ir);
                    set(at_exit);
                    Some(value)
                }
                Order::LocalOnly => {
                    set(at_exit);
                    None
                }
            };
            drop(caller_ir);
            let _ = done.send(before);
        })
        .expect("the caller's thread starts");
    let before = finished
        .recv_timeout(WAIT)
        .map_err(|error| format!("{order:?}: the caller's body did not end: {error}"))?;
    if let Some(before) = before {
        assert_eq!(before, Ok(Truth::True), "{order:?}: the call before exit");
    }
    let at_exit = reported
        .recv_timeout(WAIT)
        .map_err(|error| format!("{order:?}: the destructor's call gave nothing: {error}"))?;
    // The caller's clone of the deep protocol is dropped by the destructor; joined first, the last
    // clone is this thread's, whose stack holds the drop of this depth.
    caller
        .join()
        .map_err(|_| format!("{order:?}: the caller's thread panicked"))?;
    drop(ir);
    at_exit
}

/// The thread-local was set before the first call, so it is destroyed after the worker's handle:
/// the call in its destructor must run on a deep stack of its own, not on the 256 KiB caller's.
#[test]
fn a_call_from_a_destructor_after_the_worker_handle_is_gone_evaluates_a_deep_protocol() {
    assert_eq!(evaluate_at_exit(Order::LocalThenCall), Ok(Truth::True));
}

/// The thread-local was set after the first call, so its destructor runs while the worker's handle
/// still lives and hands the call to the worker.
#[test]
fn a_call_from_a_destructor_while_the_worker_lives_evaluates_a_deep_protocol() {
    assert_eq!(evaluate_at_exit(Order::CallThenLocal), Ok(Truth::True));
}

/// The thread never evaluated before its exit: the destructor's call is the thread's first and
/// starts the worker while the thread's locals are destroyed.
#[test]
fn a_first_call_from_a_destructor_evaluates_a_deep_protocol() {
    assert_eq!(evaluate_at_exit(Order::LocalOnly), Ok(Truth::True));
}

/// The threads of this process, from `/proc/self/task`.
fn threads() -> usize {
    std::fs::read_dir("/proc/self/task")
        .expect("/proc/self/task reads")
        .count()
}

/// 1000 short-lived callers, 50 at a time, each evaluate once; after they are joined, the workers
/// they started end too (each with a 64 MiB stack reserved): within 10 s the process has no more
/// threads than before them.
#[test]
fn a_thousand_short_lived_callers_leave_no_worker_behind() {
    let ir = protocol(0);
    let baseline = threads();
    let started = Instant::now();
    for _ in 0..20 {
        let callers: Vec<_> = (0..50)
            .map(|_| {
                let ir = Arc::clone(&ir);
                thread::Builder::new()
                    .stack_size(SMALL_STACK)
                    .spawn(move || value_of_c(&ir))
                    .expect("the caller's thread starts")
            })
            .collect();
        for caller in callers {
            assert_eq!(caller.join().expect("the caller ends"), Ok(Truth::True));
        }
    }
    let joined = threads();
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut now = joined;
    while now > baseline && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(20));
        now = threads();
    }
    eprintln!(
        "baseline {baseline}, right after the joins {joined}, after waiting {now}; 1000 calls in {:?}",
        started.elapsed()
    );
    assert!(
        now <= baseline,
        "{} threads outlive their callers 10 s after the joins (baseline {baseline}, right after \
         the joins {joined})",
        now - baseline
    );
}

/// The calls of the C library a forked child needs; this file declares them, so the test needs
/// no new dependency.
#[cfg(target_os = "linux")]
mod libc_calls {
    unsafe extern "C" {
        pub fn fork() -> i32;
        pub fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
        pub fn kill(pid: i32, signal: i32) -> i32;
        pub fn _exit(code: i32) -> !;
    }
    pub const WNOHANG: i32 = 1;
    pub const SIGKILL: i32 = 9;
}

/// A thread that evaluated, forks; the child evaluates on the same thread and exits 0 when it has
/// the value. Before the worker, the child's call started a thread of its own and returned. Now
/// the child's thread-local still holds the parent's worker handle, a worker no thread of the
/// child runs: the job is queued, nothing takes it, and the call waits forever. The child is
/// killed after 3 s.
#[cfg(target_os = "linux")]
#[test]
#[allow(unsafe_code)]
fn a_call_in_a_forked_child_of_a_thread_that_evaluated_returns() {
    use libc_calls::{_exit, SIGKILL, WNOHANG, fork, kill, waitpid};
    let ir = protocol(0);
    assert_eq!(value_of_c(&ir), Ok(Truth::True), "the parent's call");
    // SAFETY: the child calls only evaluation and `_exit`; the parent only waits and kills.
    let child = unsafe { fork() };
    assert!(child >= 0, "fork failed");
    if child == 0 {
        let code = if value_of_c(&ir) == Ok(Truth::True) {
            0
        } else {
            3
        };
        // SAFETY: ends the child without running the parent's test harness.
        unsafe { _exit(code) }
    }
    let started = Instant::now();
    let mut status = 0;
    let ended = loop {
        // SAFETY: `child` is this process's child and `status` lives through the call.
        if unsafe { waitpid(child, &mut status, WNOHANG) } == child {
            break true;
        }
        if started.elapsed() > Duration::from_secs(3) {
            // SAFETY: as above; the child is killed and reaped.
            unsafe {
                kill(child, SIGKILL);
                waitpid(child, &mut status, 0);
            }
            break false;
        }
        thread::sleep(Duration::from_millis(10));
    };
    assert!(
        ended,
        "the forked child's evaluation did not return in 3 s: it waits on a worker that exists \
         only in the parent"
    );
    assert_eq!(
        status, 0,
        "the forked child's evaluation gave another value"
    );
}
