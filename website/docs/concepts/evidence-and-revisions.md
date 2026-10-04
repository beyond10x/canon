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

## Evidence about one artifact

A protocol with several artifacts often needs a claim that only evidence about one of them can
decide. An evidence match that names a `subject` reads only records about that artifact. A record
about another declared artifact does not match it, even when that record is bound to the other
artifact's current revision: it neither establishes the claim nor contradicts it. A match that
names no subject reads records about any declared artifact.

| Records | Match with `subject: explanation` | Match without a subject |
|---|---|---|
| An attempt about the explanation survived | `TRUE` | `TRUE` |
| An attempt about the dataset survived | `UNKNOWN` | `TRUE` |
| An attempt about the explanation was refuted, one about the dataset survived | `FALSE` | `UNKNOWN` |

A record about another artifact is not listed as excluded under a claim whose match is bound to a
subject, because it does not match. That holds even when revision binding excludes it: the
record is listed only under the claims with a match that would read it.

## Freshness

Evidence can also expire. An observation older than the protocol allows becomes inapplicable, and
expiry alone never implies `FALSE`.

## Invalidation

When an upstream artifact changes, claims whose only support was bound to its previous revision are
invalidated. A protocol says which claims depend on which upstream artifact with invalidation
rules:

```yaml
invalidation:
  dataset.revised:
    upstream: dataset
    invalidates:
      - explanation.supported
```

An evidence record says which upstream revisions it was observed against:

```yaml
upstream_revisions:
  dataset: d1
```

Once the case snapshot's revision of `dataset` is no longer `d1`, the record no longer supports
`explanation.supported`, nor any claim built on it. Exclusion is per claim: a claim the rule does
not name, and that is not built on one it names, still reads the record, even when it reads the
same kind of evidence. A record that records no revision of the upstream artifact is never
invalidated.

:::note[Shipped: revision binding]

Artifacts are declared in `protocol/1`, a `canon-case/1` snapshot records each artifact's current
revision, and every `canon-evidence/1` record names its subject artifact and revision
([evaluation documents](../reference/documents.md)). `canon evaluate` refuses a record whose
subject the protocol does not declare (`undeclared-artifact`), and leaves a record bound to any
other revision of its subject out of claim evaluation: the decision lists it as excluded under
each claim with a match that would read it, in `excluded_evidence`, with the reason
`revision_mismatch`. Conformance scenario `CANON-EVIDENCE-001` holds the table above.

:::

:::note[Shipped: subject-bound evidence matches]

An evidence match in `protocol/1` may name a `subject`, which must be a declared artifact
(`canon validate` refuses another as `undeclared-artifact`). Conformance scenario
`CANON-EVIDENCE-003` holds the table under
[evidence about one artifact](#evidence-about-one-artifact).

:::

:::note[Shipped: freshness]

An evidence kind may declare a `max_age`, and a `canon-evidence/1` record may give its
`observed_at`. Given an evaluation instant (`canon evaluate --at`), a record older than its kind's
`max_age` at that instant is left out of claim evaluation and listed as excluded with the reason
`expired`: an evidence match it alone decided is `UNKNOWN` rather than `FALSE`, and claims follow
their predicates. Without an instant, or without `observed_at`, nothing expires: Canon reads no clock. Conformance scenario `CANON-EVIDENCE-002`
holds this.

:::

:::note[Shipped: invalidation]

A `protocol/1` document may declare `invalidation:` rules, and a `canon-evidence/1` record may give
its `upstream_revisions` ([evaluation documents](../reference/documents.md)). `canon validate`
refuses a rule whose upstream artifact or invalidated claim the protocol does not declare
(`undeclared-artifact`, `undeclared-claim`), and one that names a claim reaching no evidence match
(`inert-invalidation`). `canon evaluate` evaluates each claim the rule names, and every claim built
on them, without an invalidated record anywhere in its evaluation, including in the claims it
tests, and lists the record in its `excluded_evidence`, with the reason `invalidated`, when a match
it reaches reads it. A named claim loses the support the record gave, and claims built on it follow
their predicates; every other claim keeps the record. `canon check` checks records
observed before an upstream move and after it, alone and together. Conformance scenario
`CANON-INVALIDATION-001` holds this.

:::
