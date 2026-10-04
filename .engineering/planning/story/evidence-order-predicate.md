---
format: aep.planning-md/3
id: story:evidence-order-predicate
kind: story
status: draft
title: 'Predicate over evidence order: first result of a kind, one kind before another'
refs:
- provider: atlas
  reference: adr:0084
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:three-valued-claims
- depends_on: story:evaluator-skeleton
- depends_on: story:evidence-freshness
- depends_on: story:invalidation-rules
scope:
- confidence: cited
  path: conformance/scenarios/evidence-order.yaml
- confidence: cited
  path: crates/canon/src/eval/claims.rs
- confidence: cited
  path: crates/canon/src/eval/order.rs
- confidence: cited
  path: crates/canon/src/ir/
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/validate/
- confidence: cited
  path: ess/
- confidence: cited
  path: fixtures/investigation/evidence-order.yaml
revision: 3
---
## Outcome

`protocol/1` gains a predicate over evidence order (Atlas ADR 0084 § Decision 2): the first record of
a kind and its result (`first: {kind: test_result, result: failed}`), and one kind's first record
before another's (`before: {first: test_result, then: diff}`). Three-valued: UNKNOWN while either
kind has no record. Required for the ADR 0077 parity switch: it carries AEP's `test-driven`
principle, `test.first_result == failed` and `evidence.first_seq.test_result <
evidence.first_seq.diff` (aep `principles/development/test-driven.yaml:26,35` at `c0d5f852`).

## First step: the predicate syntax

ADR 0084 § Open leaves "the syntax of the evidence-order predicate" open. This story settles it in
its first step, before the ESS-first commit, and records it in this body: the two variants' field
names and shapes (the `first` / `before` forms above are the draft), and the order key records are
compared on — the `observed_at` instant story:evidence-freshness adds to `canon-evidence/1`, or a
per-record sequence number. The evaluator reads no clock either way.

## ESS first

- Specification change, first commit: in `ess/domains/protocol.yaml`, the `canon.protocol.Predicate`
  union gains the two variants as settled in the first step; with the fixture
  `fixtures/investigation/evidence-order.yaml` and the scenario file
  `conformance/scenarios/evidence-order.yaml`.
- Red on that commit: `crates/canon/tests/ess_model_matches.rs` fails naming the new variants, because
  the Rust `Predicate` (`crates/canon/src/model/predicate.rs`) does not have them yet; then
  `CANON-ORDER-001` fails under `canon conform run`.

## Scope

- In: the two variants in the source model, validator (kind references resolve), IR and evaluator.
- Out: revision-based ordering (story:revision-ordering); independence of the records
  (story:evidence-independence); classifying the variants in `canon diff`.
- Surfaces: `ess/`, `crates/canon/src/model/`, `crates/canon/src/validate/`, `crates/canon/src/ir/`,
  `crates/canon/src/eval/claims.rs` (the predicate dispatch), `crates/canon/src/eval/order.rs` (new),
  `fixtures/investigation/evidence-order.yaml`, `conformance/scenarios/evidence-order.yaml`.

## Order

depends_on story:three-valued-claims (the draft's edge: the predicate is three-valued over its
claim evaluation), story:evaluator-skeleton (it adds `eval/order.rs` beside the per-concept files)
and story:evidence-freshness (`observed_at`, the candidate order key).

depends_on story:invalidation-rules is an ordering edge, stated as one: every story that changes
`protocol/1` shares `ess/`, `model/`, `validate/` and `ir/` and runs one per wave regardless; the
edge keeps the proposed chain that story:semantic-diff is written against in front (reasoning in
story:case-composition § Order).

## Acceptance

Conformance scenario `CANON-ORDER-001` passes under `canon conform run` over
`fixtures/investigation/evidence-order.yaml`, for a claim requiring a failed first `test_result`
before the first `diff`, with four expectations: red then green then diff is TRUE; green only is
FALSE; diff before any `test_result` is FALSE; no records is UNKNOWN.
`crates/canon/tests/ess_model_matches.rs` passes.

## Source

Atlas ADR 0084 (accepted 2026-10-04; on Atlas branch `plan/ga-adrs-0084-0085` at `38266148`, not yet
on `main`) § Decision 2 and § Open; aep `principles/development/test-driven.yaml:26,35`; draft
`canon-stories-step-order.md`; Atlas ADR 0080.
