---
format: aep.planning-md/3
id: story:ess-command-surface
kind: story
status: draft
title: Declare the canon command surface in ESS
refs:
- provider: atlas
  reference: adr:0076
relations:
- decomposes: epic:canon-kernel
- depends_on: story:ess-hard-gate
- depends_on: story:semantic-diff
- serves: vision:O2
- serves: vision:governed-autonomy
revision: 1
---
## Outcome

The `canon` command surface (`validate`, `compile`, `evaluate`, `conform run`, `diff`) is declared in
`ess/` as commands over the `canon.protocol` model, completing ADR 0076's "data model and command
surface". Protocol semantics stay in Canon's own conformance suite.

## Acceptance

`ess_gate` passes with the commands declared, and `ess verify conform synthesize --path ess` reports at
least one scenario per command with 0 refusals.

## Source

Atlas ADR 0076; gap named while drafting story:ess-hard-gate (2026-10-04).
