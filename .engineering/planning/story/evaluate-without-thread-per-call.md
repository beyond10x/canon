---
format: aep.planning-md/3
id: story:evaluate-without-thread-per-call
kind: story
status: active
title: Evaluate many small cases without one operating-system thread per call
tags:
- from-request
refs:
- provider: loom
  reference: eval-deep-stack-thread
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
scope:
- confidence: cited
  path: crates/canon/src/eval/mod.rs
- confidence: inferred
  path: crates/canon/tests/evaluate_many_small_cases.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T02:32:31Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-22T02:32:31Z", actor: "human:timo", revision: 5}
---
## Outcome

A caller that evaluates many small cases does not pay one operating-system thread per evaluation. `eval::evaluate_with` and `eval::evaluate` run their work through `on_deep_stack` (`crates/canon/src/eval/mod.rs`, `DEEP_STACK` of 64 MiB), which spawns and joins a thread on every call, so that how deep a protocol's predicates may nest does not depend on the caller's stack. Under a CPU-quota cgroup on a loaded host the new thread waits for the scheduler: the `loom` session measured 2000 single-record calls at 224 to 882 microseconds per call inside the build pool's cgroup against 69 to 111 microseconds outside, for 0.9.0 and 0.10.0 alike, with CPU time flat (0.15 to 0.24 s). Its wall-time ratio tests went red because of it; it bounds by CPU time now, so nothing is urgent.

1. **One of two ways to skip the thread per call.** The options are in § Decision needed.
2. **No change of result.** The decision document, the refusal and its code, and the exit status of every `canon evaluate` stay byte for byte; the thread "reads nothing and the result is the same as on the caller's thread" (the module's own doc).
3. **The depth bound stays enforced.** `MAX_IR_DEPTH` still refuses a protocol nested deeper than the bound, whichever way a call runs; a caller that asks to run on its own stack gets the same refusal for a protocol that is too deep for it, not a stack overflow.

## Why

- Not a regression of 0.10.0: the thread per call is there since `565d1debe` of 2026-10-04 (`on_deep_stack`, `:413` to `:639` at 0.9.0, `:513` to `:774-790` at 0.10.0, as the `loom` session read them).
- `loom`'s set-aside path calls `evaluate_with` once per record. `runbook` is about to embed `b10x-canon` as a library (its requirement of 2026-11-22) and will call it more often than a command line does.
- Request: a message of the `loom` session on 2026-11-22; its note, harness and the four-row table are at `$HOME/.cache/scratch/94200cb9-e807-4544-b8fe-80c8ce4c8ba2/wave-2026-11-21-w8/u6/canon-repro.md` (read the note, the table was not re-run by the coordinator).

## Decision needed

| Option | What it is | Cost and risk |
|---|---|---|
| A. An entry point on the caller's thread | A public `evaluate_on_caller_stack` beside `evaluate_with`, same arguments. It checks the protocol's depth against what the caller's stack can hold, refuses a deeper one with the existing refusal, and runs on the caller's thread | A second public function to keep equal to the first (one test compares the two over every conformance scenario). The caller must own a stack big enough; the doc says how big |
| B. One reused worker | `on_deep_stack` keeps one worker thread per process (or per calling thread) with the 64 MiB stack and hands it the work through a channel | No new public function; no change for any caller. A channel round trip per call (about a few microseconds, to be measured) and a thread that lives as long as the process; a panic in `work` must still be resumed on the caller's thread |
| C. Both | A for callers that guarantee the stack, B for the rest | Most code |

Recommendation: B, because every caller gains with no change on their side, and the cost is measurable on the `loom` harness (`u6/harness/`) before and after. A if the measurement shows a channel round trip is also slow under the quota.

## ESS first

No `ess/` change: no declaration describes the thread. The first commit is a test that calls `evaluate_with` 2000 times with one record each, in a thread whose own stack is small, and asserts the results equal one call with 2000 records, plus a measurement of the spawn count (a counter behind `cfg(test)` in `on_deep_stack`) that fails while the spawn count is the call count.

## Scope

- **In:** Outcome 1 to 3. — cited
- **Out:** a change of the depth bound; a change of any decision or refusal; a thread pool for `canon check` or `canon simulate`. — cited
- **Surfaces:**
  - `crates/canon/src/eval/mod.rs` (`on_deep_stack`, `DEEP_STACK`, `evaluate`, `evaluate_with`, `read_ir`) — cited
  - `crates/canon/src/eval/tests.rs` or a new `crates/canon/tests/` file for the equality and spawn-count tests — inferred
- **Confidence:** high that the change is in one file; medium on the numbers, which are the `loom` session's.

## Order

- **Depends on:** nothing open.
- **Shares `crates/canon/src/eval/mod.rs`** with story:decision-condition and story:revision-ordering (both in the wave of 2026-11-22); this story runs after that wave.
- Not part of the one wave of 2026-11-22 and not before the release that wave ends in.

## Acceptance

The test of § ESS first passes: 2000 one-record calls give the results of one 2000-record call, and `on_deep_stack` spawns at most one thread for the 2000 calls (option B) or none (option A's entry point). `loom`'s harness, run before and after on the same machine, shows the per-call wall time inside the pool's cgroup within 2 times of the time outside (before: 3 to 8 times).

## Source

- **Request:** a message of the `loom` session to the coordinating session on 2026-11-22, "not a defect report, an option". No story of `loom` depends on it.
- **Canon files read:** `crates/canon/src/eval/mod.rs` at `7d1a28a20` (`DEEP_STACK`, `on_deep_stack`, `evaluate`, `evaluate_with`).
- **Not run:** the harness and its numbers.
