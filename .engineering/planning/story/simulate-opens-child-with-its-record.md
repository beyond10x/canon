---
format: aep.planning-md/3
id: story:simulate-opens-child-with-its-record
kind: story
status: draft
title: canon simulate opens a child together with its running record
relations:
- decomposes: epic:canon-kernel
revision: 1
---
## Outcome

In the walk, a child is opened and its running case_outcome record exists in the same step, as the runner writes it.

## Why

- Gap 8: simulate adds the child's record in a later step, so a children form over four parallel readers on one subject reads 'three ended complete, the fourth opened with no record' as true; a probe on needs-you reaches a state a real run cannot. With one reader it holds.
- Reported by the session `bf-runbook` (dev-operator), 2026-11-22, from `canon-gaps.md` in the dev-operator spec directory (22 composed protocols, `runbook` 0.5.0 linking Canon 0.11.0). Canon has not reproduced it; the file quotes the exact error texts.

## Scope

- **In:** the outcome above.
- **Out:** any change to the bytes of existing `protocol/1` inputs; a format change waits for Timo.
- **Surfaces:** to be scoped before a wave.

## Acceptance

The needs-you probe nobody-after-incomplete-read is unreached with four readers, as with one; a case the walk could not reach before is not reachable now either way, shown on the fixtures.

## Source

- **Request:** `bf-runbook`, gap list of 2026-11-22.
