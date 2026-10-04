---
title: Evaluation
description: An evaluation is a pure function of the protocol, the case, the evidence, authority decisions and an instant.
---

# Evaluation

An evaluation interprets one case under one protocol at one instant. In the design its inputs are
explicit:

```text
Evaluation = f(
  protocol_ir,
  case_snapshot,
  evidence_set,
  authority_decisions,
  evaluation_instant
)
```

There is no hidden clock, network access, model call, persistence lookup or implicit "latest".
Equivalent normalized inputs give equivalent normalized outputs.

## What exists today

:::note[Shipped: claims]

`canon evaluate` takes the compiled protocol, a `canon-case/1` snapshot and a set of
`canon-evidence/1` records, and writes a `canon-decision/1` document with the three-valued value of
every claim. The order of the evidence does not matter, and the same inputs give the same bytes.
Inputs that do not fit the protocol are refused with a stable code. See the
[evaluation reference](../reference/evaluation.md) and the
[evaluation documents](../reference/documents.md).

:::

:::caution[Planned: everything beyond claims]

Authority decisions and the evaluation instant are not inputs yet. The decision does not yet say
which obligations are open, which actions are admissible or blocked, or which outcomes are earned,
and it carries no explanation.

:::

The design's sketch of the full output, which is not a final schema:

```yaml
case_revision: 17
claims:
  tests.pass:
    value: unknown
    because:
      - current implementation is R2
      - applicable test evidence only exists for R1
obligations:
  - id: verify.tests
    status: open
actions:
  - id: tests.run
    status: admissible
  - id: repository.merge
    status: blocked
    because:
      - tests.pass is UNKNOWN
outcomes:
  accepted:
    status: blocked
```
