---
format: aep.planning-md/3
id: story:single-child-outcome-read-in-simulate
kind: story
status: draft
title: canon simulate gives a child's outcome the running state, not only the classes the matches name
relations:
- decomposes: epic:canon-kernel
revision: 1
---
## Outcome

canon simulate walks a dimension for one sequential child's outcome with the running record (result open) as one of its classes.

## Why

- Gap 3: simulate gives a case_outcome match only the named classes plus other, so a failure outcome written as every named result false holds while the child runs; the consumer added a running claim per child, which doubles the dimension.
- Reported by the session `bf-runbook` (dev-operator), 2026-11-22, from `canon-gaps.md` in the dev-operator spec directory (22 composed protocols, `runbook` 0.5.0 linking Canon 0.11.0). Canon has not reproduced it; the file quotes the exact error texts.

## Scope

- **In:** the outcome above.
- **Out:** any change to the bytes of existing `protocol/1` inputs; a format change waits for Timo.
- **Surfaces:** to be scoped before a wave.

## Acceptance

A probe reading found and none walks the running state, and the workaround claim is no longer needed to stay sound.

## Source

- **Request:** `bf-runbook`, gap list of 2026-11-22.
