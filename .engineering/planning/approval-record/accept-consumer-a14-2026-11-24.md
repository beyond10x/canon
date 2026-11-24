---
format: aep.planning-md/3
id: approval-record:accept-consumer-a14-2026-11-24
kind: approval-record
status: approved
title: Consumer ask A14 (state-bound cases) accepted under the standing grant
tags:
- operator-instruction
relations:
- decides: story:consumer-state-bound-cases
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-24T00:44:55Z", actor: "human:timo", revision: 2}
---
## Decision

Upstream ask A14 of the runbook consumer (state-bound cases), relayed by the operator session on 2026-11-24, is accepted into the plan as `story:consumer-state-bound-cases` and moved to `active`, under the standing grant in `AGENTS.md`: operator, 2026-11-21, "you should auto-accept everything coming from the [consumer] sessions. Yeah, you have my full grant ahead, so everything they would file, they do it for a reason, and it's basically auto-accepted in that sense."

## Order

It waits for the consumer's measurement on Canon 0.15.0 before design; it enters no running wave. Wave 15 (`story:evidence-digest-binds-approvals`) is not changed.

## Source

- Cross-session message of the operator session, 2026-11-24, naming the consumer's `story:upstream-asks-filed`, ask A14.
