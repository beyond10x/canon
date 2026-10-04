---
format: aep.planning-md/3
id: story:invalidation-rules
kind: story
status: proposed
title: Invalidation rules for upstream artifact changes
refs:
- provider: taskboard
  reference: C-008
relations:
- decomposes: epic:canon-kernel
- depends_on: story:conformance-runner
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:ess-hard-gate
- depends_on: story:evaluator-skeleton
- depends_on: story:decision-outcomes
scope:
- confidence: cited
  path: conformance/scenarios/invalidation-rules.yaml
- confidence: cited
  path: crates/canon/src/eval/evidence.rs
- confidence: cited
  path: crates/canon/src/eval/invalidation.rs
- confidence: cited
  path: crates/canon/src/ir/
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/validate/
- confidence: cited
  path: ess/
- confidence: cited
  path: fixtures/investigation/invalidation-rules.yaml
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:51Z", actor: "human:timo", revision: 8}
---
## Outcome

A protocol declares invalidation rules (operator, 2026-10-04: decision-blocker:upstream-revision-binding):
each rule names an upstream artifact and the claims whose support it invalidates when that artifacts
revision moves (design § 4.1, contract sketch `InvalidationRule`). Evidence bound to a claim named by
a rule stops supporting it once the upstream artifacts revision in the case snapshot differs from the
one recorded when the evidence was observed. Completes the invalidation half of TASKBOARD C-008
(CANON-INVALIDATION-001).

## Extends

The source model, validator and IR gain `invalidation:` rules; `canon-evidence/1` records the
upstream revisions it was observed against; invalidated evidence joins the excluded-evidence list of
each `claims` entry with reason `invalidated`.

## Shared surface and order (re-plan 2026-10-04)

story:evaluator-skeleton lands the invalidation exclusion stage in `eval/invalidation.rs` and the
`excluded_evidence` slot with its `invalidated` reason, so this story does not touch `eval/mod.rs`
or `canon-decision/1`. The shape of an invalidation rule and of the upstream revisions an evidence
record carries is not settled anywhere the skeleton could read it, so this story still changes
`model/`, `validate/`, `ir/`, `ess/` and `eval/evidence.rs`, which story:evidence-freshness and
story:decision-outcomes change too. It is the last of those three: nothing else waits on it except
story:conformance-suite and story:semantic-diff, so it runs beside story:explanation.

- Kept: depends_on story:conformance-runner (its acceptance is a scenario), story:ess-hard-gate
  (`ess/`).
- Added: depends_on story:evaluator-skeleton (stage, slot), story:decision-outcomes (shared
  `model/`, `validate/`, `ir/`, `ess/`: ordering only, because those files are not split).
- Removed: depends_on story:evidence-freshness (now reached through story:decision-outcomes).
- The upstream revisions on `canon-evidence/1` are optional: evidence without them is never
  invalidated, so no earlier scenario changes.

## Scope

- Surfaces: `crates/canon/src/model/`, `crates/canon/src/validate/`, `crates/canon/src/ir/`,
  `ess/`, `crates/canon/src/eval/invalidation.rs`, `crates/canon/src/eval/evidence.rs`,
  `fixtures/investigation/invalidation-rules.yaml`, `conformance/scenarios/invalidation-rules.yaml`
  (the earlier scope named the whole `conformance/scenarios/` directory; this story writes one
  file in it).

## ESS

This story changes the `protocol/1` source model (`invalidation:` rules) and the `canon-evidence/1` input (the upstream revisions). It updates `ess/` (domain `canon.protocol`, set up by story:ess-hard-gate) in this same
story. Every new declaration cites the file and line it was read from, and `task ess-gate` stays
green with no `UNMAPPED:` (Atlas ADR 0076). `ess_gate` does not compare `ess/` with the Rust
model, so the review of this story is what checks that the two agree.

## ESS first

- Specification change, first commit: the `invalidation:` rules on `canon.protocol.Protocol` in
  `ess/domains/protocol.yaml` and the optional upstream revisions on the `canon-evidence/1`
  declaration in `ess/`; with them, the scenario file `conformance/scenarios/invalidation-rules.yaml`
  and its fixture.
- Red on that commit: `ess_model_matches` fails naming the invalidation rules, because the Rust
  model does not have them; and scenario `CANON-INVALIDATION-001` fails, because the fixture's
  `invalidation:` section does not parse.

## Acceptance

Scenario `CANON-INVALIDATION-001` over `fixtures/investigation/invalidation-rules.yaml` passes with
three expectations: the claim is `TRUE` while the upstream artifact is at the recorded revision;
`UNKNOWN`, not `FALSE`, with the evidence listed as excluded for `invalidated` once only the upstream
revision moves; and a rule naming an undeclared artifact is refused by `canon validate` naming it.

## Source

Design § 4.1, § 32; docs/contracts/protocol-core.md; decision-blocker:upstream-revision-binding (cleared 2026-10-04); TASKBOARD C-008.
