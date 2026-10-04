---
title: Evidence bound to revisions
description: Evidence is about one revision of one artifact; when the artifact moves on, the claim goes back to UNKNOWN.
---

# Evidence bound to revisions

A protocol declares *artifacts*: the things evidence is about. Evidence is to be bound to the
artifact revision it was observed on. When the artifact moves on, that evidence stops establishing
claims about the current revision. The claim goes back to `UNKNOWN`, not to `FALSE`.

| | Revision R1 | Revision R2 |
|---|---|---|
| Evidence | A check passes; the record names the artifact and R1. | The R1 record still exists but no longer applies. |
| Claim | `TRUE` | `UNKNOWN` until evidence about R2 arrives. |

The same applies to anything that concerns a particular version: approvals, reviews, datasets,
deployments, inspection records.

## Freshness

Evidence can also expire. An observation older than the protocol allows becomes inapplicable, and
expiry alone never implies `FALSE`.

## Invalidation

When an upstream artifact changes, claims whose only support was bound to its previous revision are
to be invalidated.

:::caution[Status: the record format is shipped, the binding is not]

Artifacts are declared in `protocol/1`, a `canon-case/1` snapshot records each artifact's current
revision, and every `canon-evidence/1` record names its subject artifact and revision
([evaluation documents](../reference/documents.md)). Canon reads and checks those fields, but they
do not affect evaluation yet: revision binding, freshness and invalidation are planned; see
[where this stands](../status/where-this-stands.md).

:::
