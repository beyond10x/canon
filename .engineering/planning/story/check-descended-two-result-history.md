---
format: aep.planning-md/3
id: story:check-descended-two-result-history
kind: story
status: draft
title: canon check reaches descended beside first with earlier records of two results
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

`canon check` reaches every combination of a `descended` form and a `first` form over the same kind, the new history modes are tested beside `after` and `since`, and the `canon check` help names the modes.

## Finding

Left by correction unit 2 of wave 9 (finding E1 of `review-result:review-wave-9-eval`, fixed in `f903df4b0`, merged in `2f3d14b14`):

- `crates/canon/src/check/space.rs`: a history value got a no-item mode and a last-earlier mode. The reviewer's protocol went from 72 to 216 states with no finding; protocols without the mix are unchanged.
- One combination is still not reached: a `descended` true beside a `first` true with earlier records of two results at two instants. It needs another history mode.
- `after` and `since` beside the new modes are untested.
- The `canon check` help does not mention the new modes.

## Acceptance

A soundness test that builds the combination above is green (red before the change); a test runs `after` and `since` beside each new mode; the `canon check` help names the modes; no test added runs over 60 s (the sibling soundness test `ddc337201` takes 29.0 s and is in the slow lane).

## Source

Close-out of wave 2026-11-20-w9, from the report of correction unit 2. Not measured: the state count with another history mode.
