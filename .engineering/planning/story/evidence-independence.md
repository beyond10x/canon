---
format: aep.planning-md/3
id: story:evidence-independence
kind: story
status: draft
title: Evidence requirements that demand an independent producer
refs:
- provider: atlas
  reference: adr:0084
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:three-valued-claims
- depends_on: story:evaluator-skeleton
- depends_on: story:invalidation-rules
scope:
- confidence: cited
  path: conformance/scenarios/evidence-independence.yaml
- confidence: cited
  path: crates/canon/src/eval/claims.rs
- confidence: cited
  path: crates/canon/src/eval/independence.rs
- confidence: cited
  path: crates/canon/src/ir/
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/validate/
- confidence: cited
  path: ess/
- confidence: cited
  path: fixtures/investigation/evidence-independence.yaml
revision: 3
---
## Outcome

An evidence requirement can demand independence: a record produced by the agent working the case
does not satisfy it alone; a record produced by an independent producer (the test runner) does.
This carries AEP's `EvidenceRequirement.independent` (aep
`crates/govern/aep-domain/src/requirement.rs:291`; `principles/development/test-driven.yaml:42`)
into Canon, as Atlas ADR 0084 § "Also needed" requires. A requirement left unsatisfied this way is
UNKNOWN, not FALSE: the evidence is inapplicable, not contrary.

## First step: where independence is declared

ADR 0084 § Open leaves "where independence is declared in `protocol/1` (on the evidence kind, or on
the requirement)" open. This story settles it in its first step, before the ESS-first commit, and
records the choice in this body, together with the field on `canon-evidence/1` that says who
produced a record.

## ESS first

- Specification change, first commit: in `ess/domains/protocol.yaml`, the independence flag where
  the first step put it (on `canon.protocol.EvidenceKind` or on the evidence match of
  `canon.protocol.Predicate`) and the producer field on the evidence record; with the fixture
  `fixtures/investigation/evidence-independence.yaml` and the scenario file
  `conformance/scenarios/evidence-independence.yaml`.
- Red on that commit: `crates/canon/tests/ess_model_matches.rs` fails naming the new fields; then
  `CANON-INDEPENDENT-001` fails under `canon conform run`.

## Scope

- In: the flag and the producer field in model, validator and IR; excluding non-independent
  records for an independent requirement.
- Out: proving who produced a record (signing, attestation), which Canon takes as input;
  classifying independence changes in `canon diff` (story:semantic-diff § Scope already excludes
  independence).
- Surfaces: `ess/`, `crates/canon/src/model/`, `crates/canon/src/validate/`, `crates/canon/src/ir/`,
  `crates/canon/src/eval/claims.rs` (the match evaluation), `crates/canon/src/eval/independence.rs`
  (new), `fixtures/investigation/evidence-independence.yaml`,
  `conformance/scenarios/evidence-independence.yaml`.

## Order

depends_on story:three-valued-claims (it changes which records satisfy a match) and
story:evaluator-skeleton (it adds `eval/independence.rs` beside the per-concept files).

depends_on story:invalidation-rules is an ordering edge, stated as one: every story that changes
`protocol/1` shares `ess/`, `model/`, `validate/` and `ir/` and runs one per wave regardless; the
edge keeps the proposed chain that story:semantic-diff is written against in front (reasoning in
story:case-composition § Order).

## Acceptance

Conformance scenario `CANON-INDEPENDENT-001` passes under `canon conform run` over
`fixtures/investigation/evidence-independence.yaml`, for a claim over an independent `test_result`,
with three expectations: a passing record produced by the agent alone leaves the claim UNKNOWN; a
passing record produced by the test runner makes it TRUE; both together make it TRUE.
`crates/canon/tests/ess_model_matches.rs` passes.

## Source

Atlas ADR 0084 (accepted 2026-10-04; on Atlas branch `plan/ga-adrs-0084-0085` at `38266148`, not yet
on `main`) § Also needed and § Open; aep `crates/govern/aep-domain/src/requirement.rs:291`; draft
`canon-stories-step-order.md`; Atlas ADR 0080.
