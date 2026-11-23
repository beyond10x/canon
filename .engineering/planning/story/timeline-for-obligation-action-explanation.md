---
format: aep.planning-md/3
id: story:timeline-for-obligation-action-explanation
kind: story
status: draft
title: The obligation, action and explanation paths read each observed_at once per history
relations:
- decomposes: epic:canon-kernel
revision: 1
---
## Outcome

The obligation, action and explanation paths of `canon evaluate` read each `observed_at` once per history, as the claim walk does since wave 12.

## Why

- Unit 3 of wave 12 (`story:order-timeline-built-once`) built `order::Timeline` once per history for the claim walk: 10,240 timestamp reads became 160 on the review's 32-claim input (order256: 66,955,833 to 34,039,526 instructions).
- The obligation, action and explanation paths (`order::reasons` among them) still build one timeline per call.

## Scope

- **In:** share one `Timeline` per history across those paths; decisions stay byte-identical.
- **Out:** the claim walk, `canon check`.
- **Surfaces:** `crates/canon/src/eval/order.rs`, `crates/canon/src/eval/claims.rs`, `crates/canon/src/eval/mod.rs`.

## Acceptance

A count test shows at most one `observed_at` read per record for an evaluation that has obligations, actions and explanations, red before and green after, and `canon evaluate` output on every fixture is byte-identical.

## Source

- **Request:** wave 2026-11-22-w12, unit 3 report.
