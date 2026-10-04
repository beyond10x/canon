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
- depends_on: story:evidence-freshness
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
  path: fixtures/investigation/invalidation-rules.yaml
revision: 8
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

## Shared surface

`crates/canon/src/{model,validate,ir,eval}/` and `canon-decision/1`; ordered after
story:evidence-freshness in the evaluator chain.

## Acceptance

Scenario `CANON-INVALIDATION-001` over `fixtures/investigation/invalidation-rules.yaml` passes with
three expectations: the claim is `TRUE` while the upstream artifact is at the recorded revision;
`UNKNOWN`, not `FALSE`, with the evidence listed as excluded for `invalidated` once only the upstream
revision moves; and a rule naming an undeclared artifact is refused by `canon validate` naming it.

## Source

Design § 4.1, § 32; docs/contracts/protocol-core.md; decision-blocker:upstream-revision-binding (cleared 2026-10-04); TASKBOARD C-008.
