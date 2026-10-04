---
format: aep.planning-md/3
id: story:obligations
kind: story
status: proposed
title: Evaluate obligations as open or discharged
refs:
- provider: taskboard
  reference: C-005
relations:
- decomposes: epic:canon-kernel
- depends_on: story:three-valued-claims
- depends_on: story:conformance-runner
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:evaluator-skeleton
scope:
- confidence: cited
  path: conformance/scenarios/obligations.yaml
- confidence: cited
  path: crates/canon/src/eval/obligations.rs
- confidence: cited
  path: fixtures/investigation/obligations.yaml
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:51Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":3}}}
---
## Outcome

Canon evaluates every obligation a protocol declares as `open` or `discharged`. An obligation
carries a discharge predicate over claim values; it is discharged only when that predicate
evaluates `TRUE`, and `UNKNOWN` or `FALSE` leaves it open (design § 8: only `TRUE` satisfies a
positive requirement). Whether an obligation is discharged by a claim being decided or by it being
true is the protocol author’s choice, expressed through the claim-value tests of the predicate
language; Canon does not fix it.

The discharge predicate field (`discharged_when`) is added to the obligation construct in the
source model, the validator (its references resolve), `canon-ir/1` and `ess/` by
story:evaluator-skeleton (re-plan 2026-10-04: moved there so this story no longer shares `model/`,
`validate/`, `ir/` and `ess/` with the stories that change the source model). This story evaluates
it.

## Extends (operator decision 2026-10-04)

Adds the `obligations` section (each obligation's id and status) to `canon-decision/1`, which
story:three-valued-claims defines. The slot is story:evaluator-skeleton's; this story fills it.

## Shared surface and order (re-plan 2026-10-04)

The evaluator chain of the earlier plan is gone: this story owns `crates/canon/src/eval/obligations.rs`
and runs beside story:evidence-revision-binding, story:action-admissibility, story:outcomes and
story:evidence-freshness.

- Kept: depends_on story:three-valued-claims (it evaluates the discharge predicate over that
  story's claim values), story:conformance-runner (its acceptance is a scenario),
  story:evaluator-skeleton (`discharged_when`, its file and slot).
- Removed: depends_on story:evidence-revision-binding (ordering only, on the shared `eval/`
  directory, now split); depends_on story:ess-hard-gate (this story no longer edits `ess/`).

## Scope

- Fixture (operator decision 2026-10-04): the variant `fixtures/investigation/obligations.yaml` is
  the base plus one obligation, `establish.explanation`, discharged when `explanation.supported` is
  `TRUE`. The base is not edited.
- Surfaces: `crates/canon/src/eval/obligations.rs`, `fixtures/investigation/obligations.yaml`,
  `conformance/scenarios/obligations.yaml`.

## Domain relations

- Obligation → Claim: many-to-many through the discharge predicate. The obligation and claim
  constructs are listed in design § 4.1 and the open-obligation output in § 14 and
  `docs/contracts/protocol-core.md`; the discharge rule follows design § 8. Not an ess/1 document:
  Canon opts out of ESS for language semantics (AGENTS.md § ESS, Atlas ADR 0067).

## ESS first

- Specification change: none in `ess/` here; this story relies on story:evaluator-skeleton's
  `canon.protocol.Obligation.discharged_when`. The first commit is Canon's semantic specification
  for this story: the scenario file `conformance/scenarios/obligations.yaml` and its fixture.
- Red on that commit: scenario `CANON-OBLIGATION-001` fails under `canon conform run`, because the
  skeleton's obligations stub emits no `obligations` section.

## Acceptance

Conformance scenario `CANON-OBLIGATION-001` (a requirement § 32 does not list: an obligation whose
discharge predicate is not `TRUE` is open) passes under `canon conform run` over
`fixtures/investigation/obligations.yaml`, with three expectations and the evidence set the only
input that changes: `establish.explanation` is `open` while `explanation.supported` is `UNKNOWN`,
`open` while it is `FALSE`, and `discharged` once it is `TRUE`.

## Source

TASKBOARD C-005 (build pack, now Atlas `docs/design/governed-autonomy/TASKBOARD.md`);
`docs/design/canon-protocol-calculus-design.md` § 4.1, § 7.2, § 8, § 14, § 41 item 7.
