---
title: Evaluation
description: An evaluation is a pure function of the protocol, the case, the evidence, authority decisions, explicit decisions and an instant.
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

Canon adds one input to the design's list: the explicit decisions (`canon-decisions/1`) an outcome
may require instead of a predicate. They are passed in like the authority decisions, never looked
up.

There is no hidden clock, network access, model call, persistence lookup or implicit "latest".
Equivalent normalized inputs give equivalent normalized outputs.

## What exists today

:::shipped[Claims, obligations, actions, outcomes and the explanation]

`canon evaluate` takes the compiled protocol, a `canon-case/1` snapshot, a set of
`canon-evidence/1` records and, optionally, `canon-authority/1` authority decisions (`--authority`),
`canon-decisions/1` explicit decisions (`--decisions`) and the evaluation instant (`--at`). It
writes a `canon-decision/1` document with:

- the three-valued value of every claim, from the evidence that applies: evidence bound to another
  revision of its artifact, or older than its kind allows at the evaluation instant, is listed as
  excluded and does not count;
- each declared obligation, `open` or `discharged`;
- each declared action, `admissible`, `approval-required` or `blocked`, with the reasons;
- each declared outcome, `legitimate` or `blocked`, with the reasons; an outcome that requires an
  explicit decision is `legitimate` only with one taken at the case snapshot's revision;
- an `explanation`: what the decision was computed from, and why each claim that is not `true`,
  open obligation, action that is not admissible and blocked outcome has its status, down to the
  evidence records that applied or were excluded.

A case snapshot that records termination through an undeclared or blocked outcome is refused. The
order of the evidence does not matter, and the same inputs give the same bytes. Inputs that do not
fit the protocol are refused with a stable code. See the
[evaluation reference](../reference/evaluation.md) and the
[evaluation documents](../reference/documents.md).

:::

:::shipped[Invalidation]

An invalidation rule keeps evidence observed against an earlier revision of an upstream artifact
from the claims it names and every claim built on them; see
[invalidation](./evidence-and-revisions.md#invalidation).

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
