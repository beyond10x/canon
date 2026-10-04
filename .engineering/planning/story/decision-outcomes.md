---
format: aep.planning-md/3
id: story:decision-outcomes
kind: story
status: proposed
title: Decision-based outcomes
refs:
- provider: taskboard
  reference: C-007
relations:
- decomposes: epic:canon-kernel
- depends_on: story:invalidation-rules
- depends_on: story:conformance-runner
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: conformance/scenarios/
- confidence: cited
  path: crates/canon/src/eval/
- confidence: cited
  path: crates/canon/src/ir/
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/validate/
- confidence: cited
  path: fixtures/investigation/decision-outcomes.yaml
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:50Z", actor: "human:timo", revision: 8}
---
## Outcome

Canon gains a decision construct (operator, 2026-10-04: decision-blocker:outcome-decision-source).
A protocol may declare that an outcome requires an explicit decision rather than a claim (the
design § 12 `inconclusive` example; `declined`, `abandoned` in § 4.6). Decisions are a new
evaluation input, `canon-decisions/1`, each naming the outcome, the deciding principal and the case
revision it was taken at; the source model, validator and IR gain `requires: decision: <name>`.
Completes the decision half of TASKBOARD C-007.

## Extends

`canon-decision/1` `outcomes` entries report a decision-based outcome as `legitimate` only with a
matching decision for the current case revision, and as `blocked` naming the missing decision otherwise.

## Shared surface

`crates/canon/src/{model,validate,ir,eval}/` and `canon-decision/1`; ordered after
story:invalidation-rules in the evaluator chain.

## Acceptance

Scenario `CANON-OUTCOME-002` over `fixtures/investigation/decision-outcomes.yaml` passes with three
expectations: `inconclusive` is `blocked` naming `explicitly_inconclusive` with no decision input;
`legitimate` with a `canon-decisions/1` decision at the current case revision; `blocked` again when
that decision names a superseded case revision.

## Source

Design § 4.6, § 12; decision-blocker:outcome-decision-source (cleared 2026-10-04); TASKBOARD C-007.
