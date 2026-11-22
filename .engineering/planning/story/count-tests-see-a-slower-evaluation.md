---
format: aep.planning-md/3
id: story:count-tests-see-a-slower-evaluation
kind: story
status: draft
title: The invalidation cost tests fail for an evaluation made slower, without a second place for unsafe
tags:
- from-finding
- test-budget
refs:
- provider: review
  reference: wave-2026-11-22-w11-F7
relations:
- decomposes: epic:canon-kernel
scope:
- confidence: cited
  path: crates/canon/src/eval/claims.rs
- confidence: cited
  path: crates/canon/src/eval/invalidation_cost.rs
- confidence: cited
  path: crates/canon/tests/unsafe_confined.rs
revision: 2
---
## Outcome

The invalidation cost tests fail for a change that makes one claim evaluation slower (a scan of the walk's `in_progress` list, a copy of `kept_out` per evaluation), not only for a change that evaluates more often.

## Why

- Review of wave 2026-11-22-w11 (finding F7): the count tests of `story:wall-clock-ratio-tests-under-load` (`crates/canon/src/eval/invalidation_cost.rs`) count claim evaluations and marks. A mutant that restores the linear `in_progress` scan and the `kept_out` copy of the old cubic evaluator (commit `93f017a58`) passes both with identical counts (1275, 80200, 125250). Scratch diff of the reviewer: `mutant-scan.diff`.
- Unit 1's correction caught the copy half with a counting global allocator (bytes per evaluation: 596 against 314 for a chain of 400 against 100) and could not catch the scan half (it allocates nothing). It failed `unsafe_confined::unsafe_is_confined_to_the_eval_worker`: the allocator is an `unsafe impl GlobalAlloc` and the crate keeps its one `unsafe` in `eval/worker.rs` (Timo's H12). The correction was reverted.
- A CPU-time bound could not separate the scan from noise below about 1600 claims (652 ns against 733 ns per evaluation at 400 claims).

## Scope

- **In:** a way to see a slower evaluation that adds no `unsafe` outside `eval/worker.rs`: counters in proportion to cost at the sites (entries scanned in `in_progress`, entries copied of `kept_out`) that the production code increments under `cfg(test)`, or a decision by Timo to allow a test-only `unsafe` in a named file with the `unsafe_confined` scan extended to say so.
- **Out:** the evaluator's algorithm.
- **Surfaces:** `crates/canon/src/eval/claims.rs`, `crates/canon/src/eval/invalidation_cost.rs`, `crates/canon/tests/unsafe_confined.rs`.

## Acceptance

The reviewer's mutant (both halves) fails the chain test and the diamond test with its output shown, the mutant is restored byte for byte, and `unsafe_confined` is green.

## Source

- **Request:** review of wave 2026-11-22-w11, F7; unit 1's correction report.
