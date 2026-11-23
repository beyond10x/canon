---
format: aep.planning-md/3
id: story:supplied-evidence-required-at-open
kind: story
status: active
title: A protocol can require that its opener supplies a record when it opens the case
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T03:55:42Z", actor: "human:timo", revision: 2, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w13"}
- {from: "proposed", to: "active", at: "2026-11-23T03:55:42Z", actor: "human:timo", revision: 3, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w13"}
---
## Outcome

A protocol declares that a supplied evidence kind must be present at case open, and the check refuses an opener that cannot supply it.

## Why

- Gap 6: a case whose supplied record never arrives has no admissible action and waits for good; simulate counts it as waiting on supplied evidence or expiry, not as a finding.
- Reported by the session `bf-runbook` (dev-operator), 2026-11-22, from `canon-gaps.md` in the dev-operator spec directory (22 composed protocols, `runbook` 0.5.0 linking Canon 0.11.0). Canon has not reproduced it; the file quotes the exact error texts.

## Scope

- **In:** the outcome above.
- **Out:** any change to the bytes of existing `protocol/1` inputs; a format change waits for Timo.
- **Surfaces:** to be scoped before a wave.

## Acceptance

A fixture whose opener never supplies the record is a finding; one that does supply it passes.

## Source

- **Request:** `bf-runbook`, gap list of 2026-11-22.
