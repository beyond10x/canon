---
format: aep.planning-md/3
id: story:state-bound-names-the-split
kind: story
status: draft
title: The state-space errors name which kinds to split and what each costs
relations:
- decomposes: epic:canon-kernel
revision: 1
---
## Outcome

state-space-bound and reachable-state-bound say which dimension to reduce, and canon check reports the walk bound before it passes a protocol that the walk then refuses.

## Why

- Gap 4: needs-you passed canon check (4,362,076,160 states in the product) and was then refused by the walk with reachable-state-bound at 65,536; one outcome reading 1,131,520 states was refused with a list of dimensions and no advice.
- Reported by the session `bf-runbook` (dev-operator), 2026-11-22, from `canon-gaps.md` in the dev-operator spec directory (22 composed protocols, `runbook` 0.5.0 linking Canon 0.11.0). Canon has not reproduced it; the file quotes the exact error texts.

## Scope

- **In:** the outcome above.
- **Out:** any change to the bytes of existing `protocol/1` inputs; a format change waits for Timo.
- **Surfaces:** to be scoped before a wave.

## Acceptance

The two errors quote the largest contributing dimensions with their counts; a protocol the check passes and the walk refuses gets a warning in the check.

## Source

- **Request:** `bf-runbook`, gap list of 2026-11-22.
