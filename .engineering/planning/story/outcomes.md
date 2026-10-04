---
format: aep.planning-md/3
id: story:outcomes
kind: story
status: proposed
title: Evaluate outcomes and refuse undeclared completion
refs:
- provider: taskboard
  reference: C-007
relations:
- decomposes: epic:canon-kernel
- depends_on: story:three-valued-claims
- depends_on: story:conformance-runner
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:evaluator-skeleton
scope:
- confidence: cited
  path: conformance/scenarios/outcomes.yaml
- confidence: cited
  path: crates/canon/src/eval/outcomes.rs
- confidence: cited
  path: fixtures/investigation/outcomes.yaml
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:51Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

Canon evaluates every outcome a protocol declares as `legitimate` or `blocked`: an outcome is
legitimate only when its required predicate over claims evaluates `TRUE`, and a blocked outcome
names each claim that is not `TRUE` with its value. A case reaches completion only through a
declared outcome: a case snapshot that records termination through an outcome the protocol does
not declare is refused, naming that outcome (CANON-OUTCOME-001).

## Extends (operator decision 2026-10-04)

- `canon-case/1`, defined by story:three-valued-claims, gains the termination field: the outcome id
  the case terminated through, absent while the case is open.
- `canon-decision/1` gains the `outcomes` section: each outcome's id, status and reasons.

## Shared surface and order (re-plan 2026-10-04)

The evaluator chain of the earlier plan is gone. story:evaluator-skeleton lands the `termination`
field of `canon-case/1` (and its ESS declaration), the `outcomes` slot and the file
`eval/outcomes.rs`. This story evaluates outcomes and refuses an undeclared termination in that
file, and runs beside story:evidence-revision-binding, story:obligations,
story:action-admissibility and story:evidence-freshness.

- Kept: depends_on story:three-valued-claims (outcome requirements are evaluated over its claim
  values), story:conformance-runner (its acceptance is a scenario), story:evaluator-skeleton (field,
  file, slot).
- Removed: depends_on story:action-admissibility (ordering only, on the shared `eval/` directory,
  now split).

## Scope

- Out: outcomes whose requirement is an explicit decision rather than a claim, such as the
  `inconclusive` outcome of the design § 12 example (`requires: decision: explicitly_inconclusive`);
  story:decision-outcomes builds them on top of this story (decision-blocker:outcome-decision-source,
  cleared 2026-10-04).
- Fixture: `fixtures/investigation/outcomes.yaml`, a copy of the base (which declares the
  `supported` outcome); it exists so this story's scenario names a fixture no other story edits.
- Surfaces: `crates/canon/src/eval/outcomes.rs`, `fixtures/investigation/outcomes.yaml`,
  `conformance/scenarios/outcomes.yaml`.

## ESS first

- Specification change: none in `ess/` here; this story relies on story:evaluator-skeleton's
  `termination` on the case snapshot. The `outcomes` section is not declared in `ess/` (no story
  owned it in the earlier plan, and the shape of its reasons is not settled enough to declare up
  front). The first commit is Canon's semantic specification for this story: the scenario file
  `conformance/scenarios/outcomes.yaml` and its fixture.
- Red on that commit: scenario `CANON-OUTCOME-001` fails under `canon conform run`, because the
  skeleton's outcomes stub emits no `outcomes` section and accepts a termination through
  `abandoned`.

## Domain relations

- Outcome → Claim: many-to-many through the outcome requirement predicate. Stated in design § 4.6
  and § 12. Not an ess/1 document: Canon opts out of ESS for language semantics (AGENTS.md § ESS,
  Atlas ADR 0067).

## Acceptance

Conformance scenario `CANON-OUTCOME-001` passes under `canon conform run`, with three expectations:
the `supported` outcome is `blocked`, with a reason naming `explanation.supported` and its value,
while that claim is `UNKNOWN` or `FALSE`; it is `legitimate` once the claim is `TRUE`; and a case
snapshot whose termination field names `abandoned`, an outcome the protocol does not declare, is
refused with an error naming `abandoned`.

## Source

TASKBOARD C-007 (build pack, now Atlas `docs/design/governed-autonomy/TASKBOARD.md`);
`docs/design/canon-protocol-calculus-design.md` § 4.6, § 12, § 41 item 11; CANON-OUTCOME-001 (§ 32).
