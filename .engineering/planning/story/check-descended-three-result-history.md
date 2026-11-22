---
format: aep.planning-md/3
id: story:check-descended-three-result-history
kind: story
status: draft
title: canon check reaches a descended true beside a first true when a kind has three or more results
tags:
- from-finding
relations:
- decomposes: epic:canon-kernel
scope:
- confidence: cited
  path: crates/canon-cli/src/lib.rs
- confidence: cited
  path: crates/canon/src/check/space.rs
- confidence: cited
  path: crates/canon/src/check/tests.rs
revision: 2
---
## Outcome

`canon check` reaches a `descended` true beside a `first` true when the kind has three or more results and the result the `first` names was observed alone first among earlier records that include the middle result.

## Why

- Wave 2026-11-22-w11, unit 3 (`story:check-descended-two-result-history`): the two new history modes (`LOW_FIRST`, `HIGH_FIRST`) put the earlier records of the lowest, or the highest, result at the rank's instant and the others 15 seconds after. With three or more results the middle one is never observed alone first. Two-result sets are covered, which is what that story names.
- The state count grows with each added mode (reviewer's protocol: 53760 states with two modes, bound 65536), so a third mode may need a smaller space elsewhere.

## Scope

- **In:** a test that builds the three-result combination and is red on the current head; the history mode or modes that reach it; the state counts of the reviewer's protocol and of a protocol without the mix before and after; `canon check --help` names the modes.
- **Out:** a change of the state-space bound.
- **Surfaces:** `crates/canon/src/check/space.rs`, `crates/canon/src/check/tests.rs`, `crates/canon-cli/src/lib.rs`.

## Acceptance

The combination is reached (test red before, green after); protocols without the mix keep their state counts; none of the existing `check` tests is slower than 5 s.

## Source

- **Request:** unit 3's report of 2026-11-22 ("decided by the unit": middle result).
