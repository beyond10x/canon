---
format: aep.planning-md/3
id: story:authority-lookup-by-capability
kind: story
status: active
title: authority lookup reads only the grants of the requested capability
tags:
- from-finding
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon/src/eval/authority.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T21:20:32Z", actor: "human:timo", revision: 3, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
- {from: "proposed", to: "active", at: "2026-11-22T21:20:33Z", actor: "human:timo", revision: 4, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
---
## Outcome

An authority lookup reads only the grants of the requested capability, so the time of `canon evaluate` grows with the grants a requirement can match, not with all grants times all actions.

## Why

- Review of tag 0.10.0: 192 actions with 192 grants made 36,864 comparisons; doubling both from 1,024 to 2,048 raised the median CLI time from 46.7 ms to 120.3 ms (`canon-evaluation/timings.txt`, inputs `grants4` to `grants2048`).
- On main (0.12.0, `crates/canon/src/eval/authority.rs`, byte-identical to 0.10.0) `Authority::decide` (line 126) filters every held decision by capability and coverage per call, and the evaluator calls it once per requirement.

## Scope

- **In:** index the decisions by capability when the authority is read (`authority::read` / `applying_at`), so `decide` and the stale-grounds lookup visit only that capability's entries; a test that counts comparisons for 192 actions and 192 grants on 192 capabilities and for 192 grants on one capability (the second stays linear in that capability's grants); decisions byte-identical, including the precedence rule (`denied` over `require_approval` over `granted`) and the order of stale grounds.
- **Out:** a change to what covers a scope, to the precedence, or to the default deny.
- **Surfaces:** `crates/canon/src/eval/authority.rs`.

## Acceptance

- The comparison-count test is red before and green after, output shown.
- Decisions byte-identical on the review's `grants4` to `grants2048` inputs (the coordinator gives their path in the unit's brief; copy what is needed into the unit's scratch directory) and on every conformance scenario; median CLI time at 1,024 and 2,048 grants before and after, stated.

## Source

- **Request:** Timo, 2026-11-22 about 22:10 CET, typed (see story:conform-compiles-the-fixture-once).
