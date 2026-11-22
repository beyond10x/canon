---
format: aep.planning-md/3
id: story:wall-clock-ratio-tests-under-load
kind: story
status: active
title: The invalidation chain test passes or fails by what the code does, not by the machine's load
tags:
- from-finding
refs:
- provider: decision
  reference: D131
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon/src/eval/mod.rs
- confidence: cited
  path: crates/canon/tests/invalidation_evaluation_cost.rs
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T17:21:51Z", actor: "human:timo", revision: 3, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w11"}
- {from: "proposed", to: "active", at: "2026-11-22T17:21:51Z", actor: "human:timo", revision: 4, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w11"}
---
## Outcome

The test that compares the time of a long chain with a short one (`invalidation_evaluation_cost::claims_kept_apart_by_invalidation_rules_are_evaluated_in_time_linear_in_the_chain`) passes or fails by what the code does, not by the load of the machine.

## Why

- Wave 2026-11-22-w10, 2026-11-22: at the base `e81f970de` it failed 4 of 6 runs at load 71 to 140 (50 claims 2.0 ms, 400 claims 236 ms against a bound of 8x length); at the wave head it failed 1 of 6 at load 117 to 139. The failures predate the wave's change.
- It sits in the slow lane (D123: the slow lane may fail), so it is not a gate defect today; a timing assertion that fails on load teaches the reader to ignore red.
- `AGENTS.md` (workspace) § Tests stay fast: tests make no timing claim a loaded host can break.

## Scope

- **In:** replace the wall-clock ratio with a count of evaluations or visited nodes (a measure that does not move with load) for the same claim, or keep the time with a repeated minimum and a stated bound; the three other wall-clock-budget tests of the same file get the same check.
- **Out:** a change of the evaluation algorithm.
- **Surfaces:** `crates/canon/tests/invalidation_evaluation_cost.rs`, `crates/canon/src/eval/mod.rs` (a counter behind `cfg(test)` if needed).

## Acceptance

The test passes 20 runs in a row at load above 70 and fails when the algorithm is made quadratic (a mutant, output shown).

## Source

- **Request:** the wave 10 gate (`w11/gate3`, `w11/base-inval.out` in the coordinator's scratch), the measured runs above.
