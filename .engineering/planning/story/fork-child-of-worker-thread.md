---
format: aep.planning-md/3
id: story:fork-child-of-worker-thread
kind: story
status: draft
title: A forked child of a thread mid-call on the deep-stack worker evaluates without waiting
tags:
- from-finding
refs:
- provider: review
  reference: wave-2026-11-22-w10-F1
relations:
- decomposes: epic:canon-kernel
scope:
- confidence: cited
  path: crates/canon/src/eval/worker.rs
- confidence: cited
  path: crates/canon/tests/adversary_eval_worker.rs
revision: 2
---
## Outcome

A forked child of a thread that is itself running a call on the deep-stack worker evaluates without waiting and without the parent's marker.

## Why

- Review of wave 2026-11-22-w10 (finding F1, fixed for the thread that called from outside the worker): `crates/canon/src/eval/worker.rs` keeps `ON_WORKER` (set on the worker thread) and `WORKER` (the sender, now with the process id). A process that forks while one of its worker threads is mid-call leaves the child with a thread whose `ON_WORKER` is set and whose work runs inline on that thread's stack; the worker thread's own stack is the deep one, so an inline call is correct there, but the child has no other worker. The correction unit named it and did not handle it; the review did not raise it.
- Nothing in `canon-cli` forks. The consumer that embeds `b10x-canon` as a library (runbook, through loom) is the one that could.

## Scope

- **In:** a test that forks from a worker thread mid-call and evaluates in the child (returns within 3 s); the change that makes it pass if it is red.
- **Out:** any API change.
- **Surfaces:** `crates/canon/src/eval/worker.rs`, `crates/canon/tests/adversary_eval_worker.rs`.

## Acceptance

The test exists, is red or is shown green on the unchanged worker with its output, takes under 5 s, and the `unsafe_confined` scan stays green.

## Source

- **Request:** the correction unit's report of 2026-11-22 (what is left), the review of the same wave (F1).
