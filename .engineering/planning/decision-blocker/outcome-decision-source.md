---
format: aep.planning-md/3
id: decision-blocker:outcome-decision-source
kind: decision-blocker
status: cleared
title: Nobody has decided what supplies the explicit decision an outcome may require
relations:
- blocks: epic:canon-kernel
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-04T00:00:35Z", actor: "human:timo", revision: 3}
---
## Question

What supplies the explicit decision an outcome may require instead of a claim, as in the design
§ 12 example `inconclusive: requires: decision: explicitly_inconclusive`?

Nothing settles it. The contract sketch lists no decision construct; the evaluation inputs of design
§ 13 are protocol IR, case, evidence set, authority decisions and evaluation instant. Canon opts out
of ESS for language semantics, so no ess/1 document answers it, and no code does.

## Options seen in the sources

1. An authority decision (an input that already exists) granting a declared capability.
2. A field of the case snapshot recorded by whoever governs the case.
3. A new decision construct and evaluation input.

## What it stops

Outcomes that rest on a decision rather than a claim, such as `inconclusive`, `declined` or
`abandoned` (design § 4.6), within TASKBOARD C-007. `story:outcomes` covers claim-based outcomes
only.

## Source

`docs/design/canon-protocol-calculus-design.md` § 4.6, § 12, § 13; `docs/contracts/protocol-core.md`.

## Decision (operator, 2026-10-04)

A new decision construct and evaluation input (canon-decisions/1). Built by story:decision-outcomes.
