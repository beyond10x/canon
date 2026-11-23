---
format: aep.planning-md/3
id: story:ordering-guards-provable-by-check
kind: story
status: draft
title: canon check proves an ordering guard (an action never runs before another)
relations:
- decomposes: epic:canon-kernel
revision: 1
---
## Outcome

A guard such as the digest is never posted before the commit is provable by canon check, not only by a simulate probe.

## Why

- Gap 7: check evaluates every combination of records and does not follow preconditions; the consumer proves each guard with a probe outcome that simulate must report as unreached-outcome.
- Reported by the session `bf-runbook` (dev-operator), 2026-11-22, from `canon-gaps.md` in the dev-operator spec directory (22 composed protocols, `runbook` 0.5.0 linking Canon 0.11.0). Canon has not reproduced it; the file quotes the exact error texts.

## Scope

- **In:** the outcome above.
- **Out:** any change to the bytes of existing `protocol/1` inputs; a format change waits for Timo.
- **Surfaces:** to be scoped before a wave.

## Acceptance

A property or form for an ordering guard fails on a protocol that breaks it, with the counterexample, and passes on one that keeps it.

## Source

- **Request:** `bf-runbook`, gap list of 2026-11-22.
