---
format: aep.planning-md/3
id: story:refused-approval-is-not-stuck
kind: story
status: draft
title: A refused approval ends in a legitimate state, not a stuck one
relations:
- decomposes: epic:canon-kernel
revision: 1
---
## Outcome

When a required capability is denied, canon simulate and canon check have a way to say the case ends or lapses without a workaround action.

## Why

- Gap 5: predicates cannot read authority decisions; with requires owner.approve denied, simulate prints stuck-state with no legitimate outcome; the consumer added an approval.lapse action in two protocols.
- Reported by the session `bf-runbook` (dev-operator), 2026-11-22, from `canon-gaps.md` in the dev-operator spec directory (22 composed protocols, `runbook` 0.5.0 linking Canon 0.11.0). Canon has not reproduced it; the file quotes the exact error texts.

## Scope

- **In:** the outcome above.
- **Out:** any change to the bytes of existing `protocol/1` inputs; a format change waits for Timo.
- **Surfaces:** to be scoped before a wave.

## Acceptance

A protocol whose approval is denied reaches a declared outcome in simulate with no extra action, or the stuck-state text names the supported way to declare one.

## Source

- **Request:** `bf-runbook`, gap list of 2026-11-22.
