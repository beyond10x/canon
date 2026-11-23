---
format: aep.planning-md/3
id: story:conform-compiles-the-fixture-once
kind: story
status: implemented
title: canon conform compiles a scenario's fixture once, not once per step
tags:
- from-finding
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon/src/conform/mod.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T21:20:31Z", actor: "human:timo", revision: 3, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
- {from: "proposed", to: "active", at: "2026-11-22T21:20:31Z", actor: "human:timo", revision: 4, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
- {from: "active", to: "implemented", at: "2026-11-23T03:03:14Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
---
## Outcome

`canon conform` compiles a scenario's fixture once, however many steps the scenario has, and every step still gets the bytes it got before.

## Why

- Review of tag 0.10.0 (Timo pasted it 2026-11-22): `canon conform` compiled one unchanged fixture 23 times in a passing 22-step scenario; compilation took 31.5 % of the CLI's instructions. Profile: `conform.cg`, `conform-result.txt`, `conform-valgrind.log` in the review's scratch directory `codex-canon-review.TTcRkdo1`.
- Source on main (0.12.0, `crates/canon/src/conform/mod.rs`, byte-identical to 0.10.0): the step loop (about line 528 in the review) and the extra compile (about line 1027).

## Scope

- **In:** reuse the compiled protocol across the steps of one scenario; a test that counts compiles in a multi-step scenario (a `cfg(test)` counter) and fails at more than one; the bytes of `canon conform run` on `conformance/scenarios` unchanged.
- **Out:** a change of what a step expects, of the scenario format, or of the conformance requirements.
- **Surfaces:** `crates/canon/src/conform/mod.rs`, `crates/canon/tests/`.

## Acceptance

- A 22-step scenario compiles its fixture once (test red before, green after, output shown).
- `canon conform run --requirements conformance/requirements.yaml` gives the same output as at 0.12.0 (30 passed, 31 of 31 covered).
- Instructions of the CLI on the review's scenario before and after, read with the review's method or with a count of compile calls; the share of compilation is named.

## Source

- **Request:** Timo, 2026-11-22 about 22:10 CET, typed: take the findings that are still present and implement them in one wave, then release.
- **Read:** the review's profile files; the source on main.
