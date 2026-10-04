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
- depends_on: story:action-admissibility
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: conformance/scenarios/outcomes.yaml
- confidence: cited
  path: crates/canon/src/eval/
- confidence: cited
  path: fixtures/investigation/outcomes.yaml
revision: 4
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

## Shared surface and order (operator decision 2026-10-04)

`crates/canon/src/eval/` and `canon-decision/1` are shared by the evaluator chain three-valued-claims
→ evidence-revision-binding → obligations → action-admissibility → outcomes → evidence-freshness →
explanation. This story depends_on story:action-admissibility for that reason and runs after it.

## Scope

- Out: outcomes whose requirement is an explicit decision rather than a claim, such as the
  `inconclusive` outcome of the design § 12 example (`requires: decision: explicitly_inconclusive`).
  What supplies such a decision is undecided; see `decision-blocker:outcome-decision-source`.
- Fixture: `fixtures/investigation/outcomes.yaml`, a copy of the base (which declares the
  `supported` outcome); it exists so this story's scenario names a fixture no other story edits.
- Surfaces: `crates/canon/src/eval/`, `fixtures/investigation/outcomes.yaml`,
  `conformance/scenarios/outcomes.yaml`.

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
