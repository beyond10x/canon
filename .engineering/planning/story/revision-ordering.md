---
format: aep.planning-md/3
id: story:revision-ordering
kind: story
status: draft
title: Order red and green across descended revisions with the same test id
refs:
- provider: atlas
  reference: adr:0084
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:evidence-revision-binding
- depends_on: story:evidence-order-predicate
scope:
- confidence: cited
  path: conformance/scenarios/revision-ordering.yaml
- confidence: cited
  path: crates/canon/src/eval/order.rs
- confidence: cited
  path: crates/canon/src/ir/
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/validate/
- confidence: cited
  path: ess/
- confidence: cited
  path: fixtures/investigation/revision-ordering.yaml
revision: 2
---
## Outcome

Revision-based ordering (Atlas ADR 0084 § Decision 3, "later"): red on revision R1 (test added, fix
absent) and green on revision R2 descended from R1, with the same test id. It closes the gap AEP's
`test-driven` principle states about itself: it does not check "that the failing test is a test *of
the change*" (aep `principles/development/test-driven.yaml:50`). Draft only until
story:evidence-revision-binding and the revision-relation construct exist.

## First step: the revision relation

ADR 0084 § Open leaves "the syntax ... of the revision relation" open. This story settles it in its
first step, before the ESS-first commit, and records it in this body: how a case or an evidence
record states that R2 descends from R1, and how a test id is carried so red and green can be
matched.

## ESS first

- Specification change, first commit: in `ess/domains/protocol.yaml`, the revision relation and the
  predicate form that uses it, as the first step settles them; with the fixture
  `fixtures/investigation/revision-ordering.yaml` and the scenario file
  `conformance/scenarios/revision-ordering.yaml`.
- Red on that commit: `crates/canon/tests/ess_model_matches.rs` fails naming the new declarations;
  then `CANON-ORDER-002` fails under `canon conform run`.

## Scope

- In: the revision relation and the red-on-R1, green-on-descendant predicate.
- Out: computing descent from a VCS (Canon reads it as input); classifying the construct in
  `canon diff`.
- Surfaces: `ess/`, `crates/canon/src/model/`, `crates/canon/src/validate/`, `crates/canon/src/ir/`,
  `crates/canon/src/eval/order.rs` (extending story:evidence-order-predicate's file),
  `fixtures/investigation/revision-ordering.yaml`, `conformance/scenarios/revision-ordering.yaml`.

## Order

depends_on story:evidence-revision-binding (evidence bound to revisions, the draft's edge) and
story:evidence-order-predicate (it extends that story's order predicate and file, and ADR 0084 § Open
pairs the two syntaxes).

## Acceptance

Conformance scenario `CANON-ORDER-002` passes under `canon conform run` over
`fixtures/investigation/revision-ordering.yaml`, with four expectations: test T failed on R1 and
passed on R2 descended from R1 is TRUE; T failed on R1 and passed on R3 not descended from R1 is
FALSE; a failure and a pass under different test ids is FALSE; no record is UNKNOWN.
`crates/canon/tests/ess_model_matches.rs` passes.

## Source

Atlas ADR 0084 (accepted 2026-10-04; on Atlas branch `plan/ga-adrs-0084-0085` at `38266148`, not yet
on `main`) § Decision 3 and § Open; aep `principles/development/test-driven.yaml:50`; draft
`canon-stories-step-order.md`; Atlas ADR 0080.
