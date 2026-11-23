---
format: aep.planning-md/3
id: story:children-form-counts-unopened-children
kind: story
status: active
title: A children form can wait for all children of a set, including those not yet opened
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T03:55:41Z", actor: "human:timo", revision: 2, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w13"}
- {from: "proposed", to: "active", at: "2026-11-23T03:55:41Z", actor: "human:timo", revision: 3, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w13"}
---
## Outcome

A claim over the children of a case can say that every child of a named set ended, or that N children ended, without listing every outcome combination and without becoming true while a child is still unopened.

## Why

- Gap 2: {children: {in: [...]}} is true once every opened child ended; the consumer chained its openers and listed four reader outcomes (64 values, 65 history values per subject).
- Reported by the session `bf-runbook` (dev-operator), 2026-11-22, from `canon-gaps.md` in the dev-operator spec directory (22 composed protocols, `runbook` 0.5.0 linking Canon 0.11.0). Canon has not reproduced it; the file quotes the exact error texts.

## Scope

- **In:** the outcome above.
- **Out:** any change to the bytes of existing `protocol/1` inputs; a format change waits for Timo.
- **Surfaces:** to be scoped before a wave.

## Acceptance

A protocol with five parallel readers expresses the wait in one claim, within the state bound, and the claim is false while any reader is unopened.

## Source

- **Request:** `bf-runbook`, gap list of 2026-11-22.
