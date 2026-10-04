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
- depends_on: story:evidence-revision-binding
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:ess-hard-gate
scope:
- confidence: cited
  path: conformance/scenarios/obligations.yaml
- confidence: cited
  path: crates/canon/src/eval/
- confidence: cited
  path: crates/canon/src/ir/
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/validate/
- confidence: cited
  path: ess/
- confidence: cited
  path: fixtures/investigation/obligations.yaml
revision: 6
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

This story adds the discharge predicate field (`discharged_when`) to the obligation construct in
the source model, the validator (its references resolve) and `canon-ir/1`; story:protocol-source-model
declares obligations without it.

## Extends (operator decision 2026-10-04)

Adds the `obligations` section (each obligation's id and status) to `canon-decision/1`, which
story:three-valued-claims defines.

## Shared surface and order (operator decision 2026-10-04)

`crates/canon/src/eval/` and `canon-decision/1` are shared by the evaluator chain three-valued-claims
→ evidence-revision-binding → obligations → action-admissibility → outcomes → evidence-freshness →
explanation. This story depends_on story:evidence-revision-binding for that reason and runs after
it. It also edits `model/`, `validate/` and `ir/`, which story:evidence-freshness edits later in the
same chain.

## Scope

- Fixture (operator decision 2026-10-04): the variant `fixtures/investigation/obligations.yaml` is
  the base plus one obligation, `establish.explanation`, discharged when `explanation.supported` is
  `TRUE`. The base is not edited.
- Surfaces: `crates/canon/src/model/`, `crates/canon/src/validate/`, `crates/canon/src/ir/`,
  `crates/canon/src/eval/`, `fixtures/investigation/obligations.yaml`,
  `conformance/scenarios/obligations.yaml`.

## Domain relations

- Obligation → Claim: many-to-many through the discharge predicate. The obligation and claim
  constructs are listed in design § 4.1 and the open-obligation output in § 14 and
  `docs/contracts/protocol-core.md`; the discharge rule follows design § 8. Not an ess/1 document:
  Canon opts out of ESS for language semantics (AGENTS.md § ESS, Atlas ADR 0067).

## ESS

This story changes the `protocol/1` source model: it adds `discharged_when` to the obligation. It updates `ess/` (domain `canon.protocol`, set up by story:ess-hard-gate) in this same
story. Every new declaration cites the file and line it was read from, and `task ess-gate` stays
green with no `UNMAPPED:` (Atlas ADR 0076). `ess_gate` does not compare `ess/` with the Rust
model, so the review of this story is what checks that the two agree.

## Acceptance

Conformance scenario `CANON-OBLIGATION-001` (a requirement § 32 does not list: an obligation whose
discharge predicate is not `TRUE` is open) passes under `canon conform run` over
`fixtures/investigation/obligations.yaml`, with three expectations and the evidence set the only
input that changes: `establish.explanation` is `open` while `explanation.supported` is `UNKNOWN`,
`open` while it is `FALSE`, and `discharged` once it is `TRUE`.

## Source

TASKBOARD C-005 (build pack, now Atlas `docs/design/governed-autonomy/TASKBOARD.md`);
`docs/design/canon-protocol-calculus-design.md` § 4.1, § 7.2, § 8, § 14, § 41 item 7.
