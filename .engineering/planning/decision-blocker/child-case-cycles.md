---
format: aep.planning-md/3
id: decision-blocker:child-case-cycles
kind: decision-blocker
status: cleared
title: Nobody has decided how Canon keeps a case from opening its own ancestor
refs:
- provider: atlas
  reference: adr:0081
relations:
- blocks: story:case-composition
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-29T12:58:53Z", actor: "human:timo", revision: 3}
---
## Question

How does Canon keep a case from opening one of its own ancestors?

Atlas ADR 0081 § Open: "Cycles. A case must not open an ancestor. The closed-IR rule of § 39.5
suggests a static check; undecided."

## Options seen in the sources

1. A static check at compile time over the protocols `case.open` names, refusing a protocol that
   can reach itself (the closed-IR rule of canon `docs/design/canon-protocol-calculus-design.md`
   § 39.5).
2. A run-time check where the case graph is held (AEP stores the parent–child links, ADR 0081 § 3),
   with Canon refusing a `case_outcome` set that names an ancestor.
3. Both.

## What it stops

story:case-composition: whether its validator rejects recursive openings, and which refusal its
scenario names.

## Source

Atlas ADR 0081 § Open.

## Decision

Option 1: a static compile-time check over the protocols `case.open` names, refusing a protocol that can reach itself (the closed-IR rule of `docs/design/canon-protocol-calculus-design.md` § 39.5).

Reason: it is deterministic and needs no case graph at run time. A run-time check is added only if `case.open` can ever name an existing case id.

Decided by the orchestrator under the grant, 2026-10-29, D206 (inside the open point of Atlas ADR 0081 § Open).

`story:case-composition` is unblocked on this point.
