---
format: aep.planning-md/3
id: story:case-composition
kind: story
status: draft
title: 'Compose cases at run time: case.open and case_outcome evidence'
refs:
- provider: atlas
  reference: adr:0081
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:evaluator-skeleton
- depends_on: story:three-valued-claims
- depends_on: story:invalidation-rules
scope:
- confidence: cited
  path: conformance/scenarios/case-composition.yaml
- confidence: cited
  path: crates/canon/src/eval/claims.rs
- confidence: cited
  path: crates/canon/src/eval/composition.rs
- confidence: cited
  path: crates/canon/src/ir/
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/validate/
- confidence: cited
  path: ess/
- confidence: cited
  path: fixtures/investigation/case-composition.yaml
revision: 3
---
## Outcome

Cases compose at run time (Atlas ADR 0081, option A). Two kernel primitives:

1. **Action `case.open(protocol, subject)`.** A protocol action may declare that it opens a child
   case under a named protocol for a named subject. Its authority requirements are declared by the
   parent protocol like those of any other action. The validator refuses an opening whose protocol
   id is malformed. Canon opens nothing itself; the action is admitted or blocked like any other.
2. **Evidence kind `case_outcome`.** A child case's outcome is evidence in the parent: one record
   per child, carrying the child case id and its outcome (`open` while the child runs, otherwise its
   terminal outcome, for example `accepted`, `declined`, `inconclusive`, `abandoned`). AEP supplies
   the records from the parent–child links it stores (ADR 0081 § 3). Two records for one child are
   refused naming the child.

A parent claim can range over its children: TRUE once every child's outcome is in a listed set,
FALSE once any child ended outside it, UNKNOWN while any child is `open` or when there is no child
record (Canon's rule: no applicable evidence is `UNKNOWN`). ADR 0081's example: an audit's
`findings.resolved` is TRUE once every child `software.change` is `accepted` or `declined`.

## ESS first

- Specification change, first commit: in `ess/domains/protocol.yaml`, the optional opening on
  `canon.protocol.Action` (protocol id and subject), the child-case field on the evidence record,
  and the predicate variant that ranges over children; with the fixture
  `fixtures/investigation/case-composition.yaml` and the scenario file
  `conformance/scenarios/case-composition.yaml`.
- Red on that commit: `crates/canon/tests/ess_model_matches.rs` fails naming the new fields, because
  the Rust model does not have them; `CANON-COMPOSE-001` fails under `canon conform run`.

## Blocked on

- decision-blocker:child-case-cycles — whether a case opening its own ancestor is refused by a
  static check (§ 39.5's closed-IR rule) or elsewhere.
- decision-blocker:commissions-per-case — whether one case may hold several commissions.

## Scope

- In: the opening on actions, its validation and IR form; the child-case field and the uniqueness
  check on `case_outcome` records; the predicate over children and its evaluation.
- Out: opening or tracking child cases (AEP and Commission); classifying these constructs in
  `canon diff` (story:semantic-diff § Scope excludes constructs the kernel did not have); static
  imports (§ 39.5), which stay as they are.
- Surfaces: `ess/`, `crates/canon/src/model/`, `crates/canon/src/validate/`, `crates/canon/src/ir/`,
  `crates/canon/src/eval/claims.rs` (the predicate dispatch), `crates/canon/src/eval/composition.rs`
  (new), `fixtures/investigation/case-composition.yaml`,
  `conformance/scenarios/case-composition.yaml`.

## Order

depends_on story:evaluator-skeleton (it lands the per-concept `eval/` files this story adds one
beside) and story:three-valued-claims (the claim evaluation it extends).

depends_on story:invalidation-rules is an ordering edge, stated as one. Every story that changes
`protocol/1` shares `ess/`, `model/`, `validate/` and `ir/`, so `aep plan artifact waves` runs them
one per wave whatever the edges say. story:invalidation-rules is the last of the proposed chain
(story:evidence-freshness → story:decision-outcomes → story:invalidation-rules) that
story:semantic-diff is written against; this edge keeps that chain in its waves and puts the ADR
0081/0083/0084 model stories after it, beside story:semantic-diff rather than in front of it.

## Acceptance

Conformance scenario `CANON-COMPOSE-001` passes under `canon conform run` over
`fixtures/investigation/case-composition.yaml`, with five expectations: the action declaring
`case.open` compiles with its protocol and subject in `canon-ir/1`; `findings.resolved` is UNKNOWN
with one child `open`; TRUE with every child `accepted` or `declined`; FALSE with one child
`abandoned`; and two `case_outcome` records for one child are refused naming it.
`crates/canon/tests/ess_model_matches.rs` passes.

## Source

Atlas ADR 0081 (accepted 2026-10-04) § Decision and § Open; canon
`docs/design/canon-protocol-calculus-design.md` § 39.5; Atlas
`docs/design/governed-autonomy/incident-response-walkthrough.md` § 2.6; Atlas ADR 0080.
