---
title: Evidence bound to revisions
description: Evidence is about one revision of one artifact; when the artifact moves on, a claim that evidence decided goes back to UNKNOWN.
---

# Evidence bound to revisions

A protocol declares *artifacts*: the things evidence is about. Evidence is bound to the artifact
revision it was observed on. When the artifact moves on, that evidence no longer applies to the
current revision: claims are evaluated as if it had not been given. A claim decided by an evidence
match on it goes back to `UNKNOWN`, not to `FALSE`. A claim that tests whether another claim is
`UNKNOWN` is decided by that `UNKNOWN`, so it can become `TRUE` or `FALSE`.

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

:::note[Shipped: revision binding]

Artifacts are declared in `protocol/1`, a `canon-case/1` snapshot records each artifact's current
revision, and every `canon-evidence/1` record names its subject artifact and revision
([evaluation documents](../reference/documents.md)). `canon evaluate` refuses a record whose
subject the protocol does not declare (`undeclared-artifact`), and leaves a record bound to any
other revision of its subject out of claim evaluation: the decision lists it as excluded under
each claim that reaches its kind, in `excluded_evidence`, with the reason `revision_mismatch`. Conformance scenario
`CANON-EVIDENCE-001` holds the table above.

:::

:::note[Shipped: freshness]

An evidence kind may declare a `max_age`, and a `canon-evidence/1` record may give its
`observed_at`. Given an evaluation instant (`canon evaluate --at`), a record older than its kind's
`max_age` at that instant is left out of claim evaluation and listed as excluded with the reason
`expired`, so a claim it alone decided is `UNKNOWN`, never `FALSE`. Without an instant, or without
`observed_at`, nothing expires: Canon reads no clock. Conformance scenario `CANON-EVIDENCE-002`
holds this.

:::

:::caution[Planned: invalidation]

An upstream change invalidates nothing yet; see
[where this stands](../status/where-this-stands.md).

:::
