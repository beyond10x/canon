---
format: aep.planning-md/3
id: story:order-timeline-built-once
kind: story
status: implemented
title: canon evaluate builds the record timeline once per evaluation, not per chronology predicate
tags:
- from-finding
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon/src/eval/order.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T21:20:32Z", actor: "human:timo", revision: 3, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
- {from: "proposed", to: "active", at: "2026-11-22T21:20:32Z", actor: "human:timo", revision: 4, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
- {from: "active", to: "implemented", at: "2026-11-23T03:03:15Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
---
## Outcome

`canon evaluate` builds the timeline of a record history once per evaluation, however many chronology claims read it.

## Why

- Review of tag 0.10.0: `canon evaluate` parsed 160 timestamps 10,240 times across 32 chronology claims; rebuilding the same timeline took 19 % of the CLI's instructions (`canon-evaluation/counters.txt`, `profile.txt`, with the inputs `order32`, `order64`, `order256`).
- On main (0.12.0) `crates/canon/src/eval/order.rs` `value` (lines 85 to 100) builds `ticks(history)` through a closure for each predicate it evaluates; `ticks` parses every record's timestamp (line 112 on). `order.rs` changed since 0.10.0 (revision ordering) but the per-predicate build is still there.

## Scope

- **In:** compute the ticks of a history once for an evaluation (or once per distinct history, with the `revised_by`/`since` histories kept apart) and share them across claims; a test that counts timestamp parses for 32 chronology claims over 160 records (at most 160, not 10,240); decisions byte-identical.
- **Out:** a change of what an order form means; the sub-second order semantics.
- **Surfaces:** `crates/canon/src/eval/order.rs`, and the call sites in `crates/canon/src/eval/` that pass the history.

## Acceptance

- The parse-count test is red before and green after, output shown.
- `canon evaluate` on the review's `order32`, `order64`, `order256` inputs (in the review's `canon-evaluation/` scratch directory of session `01a201b4`; the coordinator gives the path in the unit's brief; copy what is needed into the unit's scratch directory) gives byte-identical decisions before and after, and the time before and after is stated.
- The conformance scenarios and the `since`/`after`/`before` tests stay green.

## Source

- **Request:** Timo, 2026-11-22 about 22:10 CET, typed (see story:conform-compiles-the-fixture-once).
