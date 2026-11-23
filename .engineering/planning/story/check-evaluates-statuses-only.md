---
format: aep.planning-md/3
id: story:check-evaluates-statuses-only
kind: story
status: implemented
title: canon check evaluates statuses only and builds no explanation per state
tags:
- from-finding
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon/src/check/mod.rs
- confidence: cited
  path: crates/canon/src/eval/mod.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T21:20:31Z", actor: "human:timo", revision: 4, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
- {from: "proposed", to: "active", at: "2026-11-22T21:20:31Z", actor: "human:timo", revision: 5, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
- {from: "active", to: "implemented", at: "2026-11-23T03:03:15Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
---
## Outcome

`canon check` evaluates each state for the statuses of actions and outcomes only, and builds no explanation for a state.

## Why

- Review of tag 0.10.0: on the shipped 48-state fixture `canon check` built 46 explanations and discarded them, 22.6 % of the CLI's instructions (`check-profiled.txt`, `check.cg`). The checker reads only action and outcome statuses.
- On main (0.12.0) `check::evaluate` (`crates/canon/src/check/mod.rs`, from about line 1040) still calls `eval::evaluate_with`, builds the whole `Decision`, and then reads `status` from its `actions` and `outcomes` sections.

## Scope

- **In:** an evaluation entry point (or a mode of `evaluate_with`) that returns the action and outcome statuses without building explanations, used by the checker; the statuses it returns equal those of the full decision for every state of every fixture; `canon check` and `canon generate` output bytes unchanged.
- **Out:** any change of the decision's bytes for `canon evaluate`; a change of the state space.
- **Surfaces:** `crates/canon/src/check/mod.rs`, `crates/canon/src/eval/mod.rs` (and the explanation code it calls).

## Acceptance

- A test over every protocol fixture compares the statuses of the new path with those of the full decision for every state of each (equal), and a count test holds that the checker builds no explanation.
- `canon check` and `canon generate` give byte-identical output on every fixture of `fixtures/` (compared with a build of main, 173 `protocol/1` fixtures as in the wave 11 review).
- The time of `canon check` on the 48-state fixture, before and after, and the state-evaluation share of a profile, stated.

## Source

- **Request:** Timo, 2026-11-22 about 22:10 CET, typed (see story:conform-compiles-the-fixture-once).
- **Read:** the review's `check-profiled.txt`; `check/mod.rs` on main.
