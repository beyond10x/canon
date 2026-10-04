---
format: aep.planning-md/3
id: story:invalidation-evaluation-cost
kind: story
status: draft
title: Evaluation under invalidation rules is at most quadratic in the claim count
relations:
- decomposes: epic:canon-kernel
- depends_on: story:invalidation-rules
- serves: vision:O2
- serves: vision:governed-autonomy
revision: 1
---
## Outcome

Evaluating a protocol whose claims each carry an invalidation rule takes time no worse than
quadratic in the number of claims, within the depth bound (about 4000 chained claims).

## Found by

Adversary pass 2 of story:review-hardening-w7 (wave 2026-10-04-w14): a chain of claims with one
invalidation rule each took 0.37 s at 50 claims and 84 s at 400 (the same on a0cc691), about cubic.
Suspected, not attributed: per-context memo keys that clone the `EvidenceId` set
(`eval/claims.rs:66`), the `invalidated_claims` fixpoint rescanning every claim per round
(`eval/invalidation.rs:122`), and `Invalidated::lists` walking reached matches per record.

## Acceptance

`claims_kept_apart_by_invalidation_rules_are_evaluated_in_time_linear_in_the_chain` (kept at
`ga-wave-2026-10-04-w14/canon-review-hardening-w7/scratch/`): a chain eight times longer takes at
most 64 times as long; and a 4000-claim chain evaluates in under 10 s in a release build.
