---
title: Three-valued truth
description: Every claim is TRUE, FALSE or UNKNOWN, and UNKNOWN is never FALSE.
---

# Three-valued truth

Every claim has one of three values. The difference between "contradicted" and "not yet
established" decides what should happen next, so the two are never collapsed. Missing or stale
evidence is never a contradiction.

| Value | Meaning |
|---|---|
| `TRUE` | Applicable evidence establishes the claim's predicate. Only `TRUE` satisfies a positive requirement. |
| `FALSE` | Applicable evidence contradicts the predicate. The claim is refuted, not merely open. |
| `UNKNOWN` | The applicable evidence is not enough to decide. Something should be observed before anything is concluded. |

## Examples

These come from the design document.

| Situation | Value |
|---|---|
| The check was never run | `UNKNOWN` |
| The check ran and failed | `FALSE` |
| The check ran and passed | `TRUE` |
| It passed for revision R1; the current revision is R2 | `UNKNOWN` |
| An observation is older than the protocol allows | `UNKNOWN` |

## What exists today

:::note[Shipped: claim evaluation]

`canon evaluate` gives every claim of a case one of the three values from its evidence records. An
evidence match with a result is `unknown` when no record of the kind exists, `false` when records
exist and none has the result, `true` when every record of the kind has that result, and `unknown`
when records of the kind disagree. The [evaluation reference](../reference/evaluation.md) has every
rule, and the [worked example](../reference/investigation-example.md) shows the values for five
evidence situations.

:::

:::note[Shipped: the row about revisions]

A record bound to a revision of its artifact that is not the case's current one is left out of
claim evaluation, so the R1/R2 row is `UNKNOWN` as shown.

:::

:::caution[Planned: the row about age]

The last row needs freshness, which is not built yet: records carry no time.

:::
