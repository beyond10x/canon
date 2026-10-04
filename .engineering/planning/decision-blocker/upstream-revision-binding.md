---
format: aep.planning-md/3
id: decision-blocker:upstream-revision-binding
kind: decision-blocker
status: cleared
title: Nobody has decided how evidence or a claim binds to an upstream artifact revision
relations:
- blocks: epic:canon-kernel
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-04T00:00:35Z", actor: "human:timo", revision: 3}
---
## Question

How is evidence, or a claim, bound to the revision of an artifact other than its own subject, so
that changing that upstream artifact invalidates dependent claims whose only support was bound to
its previous revision (CANON-INVALIDATION-001)?

Nothing settles it. Design § 9 shows evidence bound to one subject revision; § 32 requires
invalidation through a bound upstream artifact without saying how the binding is expressed. Canon
opts out of ESS for language semantics, so no ess/1 document answers it, and no code does.

## Options seen in the sources

1. An evidence record carries further revision bindings besides its subject (Evidence → artifact
   revisions becomes one-to-many).
2. The protocol declares a dependency between artifacts, and the case snapshot records which
   upstream revision each artifact revision was derived from.
3. The protocol declares invalidation rules (design § 4.1 lists them; the contract sketch names
   `InvalidationRule`) that name the claims an upstream change invalidates.

## What it stops

The invalidation half of TASKBOARD C-008. `story:evidence-freshness` covers freshness only;
invalidation through the subject artifact’s own superseded revision is `story:evidence-revision-binding`.

## Source

`docs/design/canon-protocol-calculus-design.md` § 4.1, § 9, § 32; `docs/contracts/protocol-core.md`.

## Decision (operator, 2026-10-04)

Invalidation rules declared by the protocol name the claims an upstream artifact change invalidates. Built by story:invalidation-rules.
