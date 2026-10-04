---
format: aep.planning-md/3
id: decision-blocker:commissions-per-case
kind: decision-blocker
status: open
title: Nobody has decided whether one case may hold several commissions
refs:
- provider: atlas
  reference: adr:0081
relations:
- blocks: story:case-composition
revision: 1
---
## Question

May one case hold several commissions at once (for example an implementor and a reviewer)?

Atlas ADR 0081 § Open: "Whether Commission allows several commissions on one case (an implementor
and a reviewer). Not yet checked against commission `docs/design/commission-design.md`."

## Options seen in the sources

1. One commission per case: a second party works through a child case (`case.open`).
2. Several commissions per case, each acting under its own authority.

## What it stops

story:case-composition: whether splitting work between parties always goes through `case.open`,
which decides whether the child-case primitive is the only way a case is shared.

## Source

Atlas ADR 0081 § Open; commission `docs/design/commission-design.md` (not yet checked).
