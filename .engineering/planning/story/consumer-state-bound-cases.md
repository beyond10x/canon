---
format: aep.planning-md/3
id: story:consumer-state-bound-cases
kind: story
status: active
title: canon check decides the runbook consumer's state-bound outcomes within its gate budget
tags:
- from-consumer
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T00:44:55Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-24T00:44:56Z", actor: "human:timo", revision: 3}
---
## Outcome

`canon check` decides the runbook consumer's heaviest outcomes within the gate budget: the consumer's protocol check step ends well inside its 2-minute CI budget (today 86 s of it), so outcomes whose slices read billions of states (refresh.checks `clean` 2.4e9, release-review 1.9e16, refresh 1.1e12) are checked without the step dominating the budget.

## Why

- Upstream ask A14 of the runbook consumer (its `story:upstream-asks-filed`), relayed by the operator session on 2026-11-24: state-bound cases. Its check script spends 86 s of a 2-minute CI budget on these three protocols.
- Canon 0.15.0's `count: N` on the `children` form may cover part of it (five readers in one claim instead of a chain of openers). The consumer measures after it pins 0.15.0 and reports what is left.

## Scope

- **First:** wait for the consumer's measurement on 0.15.0 (which outcomes and how many states remain); do not design before it.
- **Then:** for each outcome that remains above the budget, find what multiplies the slice (independent dimensions that the outcome does not read together, counters, history modes) and the smallest change to `canon check` that decides it without enumerating the product, or a protocol rewrite the consumer can make, written in the docs.
- **Out:** raising the state bound; a new format version.

## Acceptance

- On copies of the consumer's three protocols (read-only originals), `canon check` time and state counts are reported before and after; the sum over the consumer's check step is under 30 s at the load of a gate run, or the remaining gap is named with its cause.
- Every existing fixture's `canon check` output is byte-identical unless the change names it.

## Source

- Request: upstream ask A14 of the runbook consumer, relayed by the operator session, 2026-11-24.
