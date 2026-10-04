---
format: aep.planning-md/3
id: story:evidence-freshness
kind: story
status: proposed
title: Expire evidence against the evaluation instant
refs:
- provider: taskboard
  reference: C-008
relations:
- decomposes: epic:canon-kernel
- depends_on: story:evidence-revision-binding
- depends_on: story:conformance-runner
- depends_on: story:outcomes
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:ess-hard-gate
scope:
- confidence: cited
  path: conformance/scenarios/evidence-freshness.yaml
- confidence: cited
  path: crates/canon-cli/
- confidence: cited
  path: crates/canon/src/eval/
- confidence: cited
  path: crates/canon/src/ir/
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/validate/
- confidence: cited
  path: ess/
- confidence: cited
  path: fixtures/investigation/evidence-freshness.yaml
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:50Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome

Evidence can expire. A protocol can declare a maximum age for evidence; at evaluation, evidence
older than that age relative to the evaluation instant passed in is inapplicable, and a claim left
without applicable evidence is `UNKNOWN`, never `FALSE` (design § 8, CANON-EVIDENCE-002). The
instant is an explicit input; the evaluator reads no clock. Where in `protocol/1` the `max_age`
field sits (on the evidence-kind declaration or on an evidence match) is this story's decision,
recorded in the source model, validator, IR and its conformance scenario.

## Extends (operator decision 2026-10-04)

- `protocol/1` and `canon-ir/1` gain `max_age`.
- `canon evaluate` gains the evaluation instant input, `--at`.
- `canon-evidence/1`, defined by story:three-valued-claims, gains `observed_at`, which age is
  measured from.
- `canon-decision/1`: expired evidence is added, with reason `expired`, to the same excluded-evidence
  list in each `claims` entry that story:evidence-revision-binding introduced. story:explanation
  traces that list and adds no reason of its own.

## Shared surface and order (operator decision 2026-10-04)

`crates/canon/src/eval/` and `canon-decision/1` are shared by the evaluator chain three-valued-claims
→ evidence-revision-binding → obligations → action-admissibility → outcomes → evidence-freshness →
explanation. This story depends_on story:outcomes for that reason and runs after it. It edits
`model/`, `validate/` and `ir/`, which story:protocol-source-model and story:canon-ir create and
story:obligations edits earlier in the same chain, so no two of them run at once.

## Scope

- Fixture (operator decision 2026-10-04): the variant `fixtures/investigation/evidence-freshness.yaml`
  is the base plus a maximum age on the evidence `explanation.supported` draws on. The base is not
  edited.
- Out: invalidation of claims when a bound upstream artifact changes (CANON-INVALIDATION-001).
  Invalidation of evidence bound to the subject artifact’s own superseded revision is C-004. How
  evidence or a claim is bound to an upstream artifact revision is undecided; see
  `decision-blocker:upstream-revision-binding`.
- Surfaces: `crates/canon/src/model/`, `crates/canon/src/validate/`, `crates/canon/src/ir/`,
  `crates/canon/src/eval/`, `crates/canon-cli/`, `fixtures/investigation/evidence-freshness.yaml`,
  `conformance/scenarios/evidence-freshness.yaml`.

## ESS

This story changes the `protocol/1` source model (`max_age`) and the `canon-evidence/1` input (`observed_at`). It updates `ess/` (domain `canon.protocol`, set up by story:ess-hard-gate) in this same
story. Every new declaration cites the file and line it was read from, and `task ess-gate` stays
green with no `UNMAPPED:` (Atlas ADR 0076). `ess_gate` does not compare `ess/` with the Rust
model, so the review of this story is what checks that the two agree.

## Acceptance

Conformance scenario `CANON-EVIDENCE-002` passes under `canon conform run` over
`fixtures/investigation/evidence-freshness.yaml`, with two expectations and the evaluation instant
the only input that changes: `explanation.supported` is `TRUE` at an instant within its evidence's
maximum age, and `UNKNOWN`, not `FALSE`, at an instant past it.

## Source

TASKBOARD C-008 (build pack, now Atlas `docs/design/governed-autonomy/TASKBOARD.md`);
`docs/design/canon-protocol-calculus-design.md` § 8, § 9, § 13, § 41 items 6 and 12;
CANON-EVIDENCE-002, CANON-INVALIDATION-001 (§ 32).
