---
format: aep.planning-md/3
id: epic:canon-kernel
kind: epic
status: active
title: 'Canon kernel: deterministic protocol evaluation'
summary: Typed ids, three-valued truth, IR, evidence applicability, obligations, admissibility, outcomes, explanation, conformance, semantic diff.
refs:
- provider: atlas
  reference: epic:ga-canon-kernel
relations:
- serves: vision:governed-autonomy
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:41Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-04T00:00:41Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

Canon expresses and deterministically evaluates a generic evidence-governed protocol, with no
engineering vocabulary in the kernel. Covers TASKBOARD C-001 … C-011.

## Acceptance

Canon's conformance suite runs a non-engineering investigation protocol fixture and shows that the
same normalized input yields byte-identical normalized evaluation, that evidence bound to a
superseded revision evaluates the claim as `UNKNOWN`, and that an action whose required claim is not
`TRUE` is reported blocked with its reason.

## Source

Atlas `epic:ga-canon-kernel`; Atlas ADR 0067; `docs/design/canon-protocol-calculus-design.md`;
`docs/contracts/protocol-core.md`.
