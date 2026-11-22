//! The worker thread of [`super::on_deep_stack`]: one thread per calling thread, started by that
//! thread's first call, with a stack of `DEEP_STACK` bytes, kept until the calling thread ends.
//!
//! This is the one place of the crate that needs `unsafe` (the crate otherwise denies it): the
//! work a caller hands over borrows from the caller's frame (the protocol, the evidence), so it
//! is not `'static`, and a thread that outlives the call can only be handed a `'static` job. The
//! job's lifetime is erased once, in [`run`], and that is sound for the reason written there.

#![allow(unsafe_code)]

use std::any::Any;
use std::cell::{Cell, RefCell};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::mpsc::{self, Sender};

type Job = Box<dyn FnOnce() + Send + 'static>;

thread_local! {
    /// The worker this thread hands its work to, started by its first call. Dropped with the
    /// thread, which ends the worker.
    static WORKER: RefCell<Option<Sender<Job>>> = const { RefCell::new(None) };
    /// Set on a worker thread: a call made there runs where it is, on the deep stack it has.
    static ON_WORKER: Cell<bool> = const { Cell::new(false) };
}

/// Runs `work` on this thread's worker and returns its result, or resumes its panic here.
///
/// Called on the worker itself (`work` that calls back into evaluation), it runs `work` inline:
/// the worker is blocked on this very call and could not take another job. Called while this
/// thread's own locals are being destroyed, when the worker handle is gone, it runs `work` on a
/// thread of its own, as every call did before the worker existed.
pub(super) fn run<T: Send>(work: impl FnOnce() -> T + Send) -> T {
    if ON_WORKER.with(Cell::get) {
        return work();
    }
    if WORKER.try_with(|_| ()).is_err() {
        return super::on_a_thread_of_its_own(work);
    }
    let (done, finished) = mpsc::sync_channel::<Result<T, Box<dyn Any + Send>>>(1);
    let job: Box<dyn FnOnce() + Send + '_> = Box::new(move || {
        let result = catch_unwind(AssertUnwindSafe(work));
        let _ = done.send(result);
    });
    // SAFETY: the job borrows from this frame through `work`. This function returns, or unwinds,
    // only after `finished.recv()` returns, and that happens only after the job has run `work` to
    // its end (the result is sent after it), or after the job has been dropped, which drops `work`.
    // Before `submit` hands the job to the worker, a panic drops it with this frame. Once the
    // worker has it, the worker runs it: the worker ends only when this thread's sender is
    // dropped, which is when this thread ends, and `catch_unwind` keeps a panic of `work` from
    // ending the worker. A `recv` error would mean the job was dropped unrun by some path this
    // comment does not know; then the borrow cannot be shown over, and the process aborts rather
    // than return.
    let job: Job = unsafe { std::mem::transmute::<Box<dyn FnOnce() + Send + '_>, Job>(job) };
    WORKER.with(|worker| submit(&mut worker.borrow_mut(), job));
    match finished.recv() {
        Ok(Ok(result)) => result,
        Ok(Err(panic)) => resume_unwind(panic),
        Err(_) => std::process::abort(),
    }
}

/// Sends `job` to the worker in `slot`, starting a worker first when there is none or the last one
/// has gone.
fn submit(slot: &mut Option<Sender<Job>>, mut job: Job) {
    loop {
        if let Some(worker) = slot.as_ref() {
            match worker.send(job) {
                Ok(()) => return,
                Err(mpsc::SendError(returned)) => job = returned,
            }
        }
        *slot = Some(start());
    }
}

fn start() -> Sender<Job> {
    let (jobs, queue) = mpsc::channel::<Job>();
    std::thread::Builder::new()
        .name("canon-eval".to_owned())
        .stack_size(super::DEEP_STACK)
        .spawn(move || {
            ON_WORKER.with(|on| on.set(true));
            for job in queue {
                job();
            }
        })
        .expect("the operating system starts a thread for evaluation");
    #[cfg(test)]
    super::SPAWNED.with(|spawned| spawned.set(spawned.get() + 1));
    jobs
}
