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
scope:
- confidence: inferred
  path: ess/domains/cli.yaml
- confidence: inferred
  path: ess/system.yaml
revision: 5
---
## Outcome

The `canon` command surface (`validate`, `compile`, `evaluate`, `check`, `generate`, `conform run`,
`diff`: every subcommand on `main` when this story lands) is declared in `ess/` as commands over
the `canon.protocol` model, completing ADR 0076's "data model and command surface". Protocol
semantics stay in Canon's own conformance suite.

From then on a story that changes a command declares the change in `ess/` in its first commit:
story:protocol-imports (`--import` on `validate`, `compile` and `check`) and story:protocol-floor
(`canon floor`) depend on this story for that reason (2026-10-07, order recorded for
https://github.com/beyond10x/canon/issues/5).

## ESS first

- Specification change, first commit: one command declaration per subcommand in `ess/`.
- Red on that commit: not named yet. The acceptance below counts synthesized scenarios, and no
  existing test fails when a command is declared before anything conforms to it; this story is a
  draft without scope, and naming the red test is part of scoping it.

## Acceptance

`ess_gate` passes with the commands declared, and `ess verify conform synthesize --path ess` reports at
least one scenario per command with 0 refusals.

## Source

Atlas ADR 0076; gap named while drafting story:ess-hard-gate (2026-10-04); Atlas ADR 0080 (draft).

## Scope

- Surfaces (inferred, 2026-10-07; the file does not exist yet): `ess/domains/cli.yaml`, a new domain
  `canon.cli` holding one command declaration per subcommand, and the domain list in
  `ess/system.yaml`. Neither is `ess/domains/protocol.yaml`, which story:case-inputs edits, so the two
  may share a wave.
- Order: after story:semantic-diff (existing edge); before story:protocol-imports and
  story:protocol-floor, which depend on this story and declare their command changes here.
- Shared files: `ess/system.yaml` with any other story that adds a domain; none in the 2026-10-07
  order does.
