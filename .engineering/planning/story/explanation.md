---
format: aep.planning-md/3
id: story:explanation
kind: story
status: implemented
title: Emit a deterministic structured explanation
refs:
- provider: taskboard
  reference: C-009
relations:
- decomposes: epic:canon-kernel
- depends_on: story:evidence-revision-binding
- depends_on: story:obligations
- depends_on: story:action-admissibility
- depends_on: story:outcomes
- depends_on: story:evidence-freshness
- depends_on: story:conformance-runner
- depends_on: story:decision-outcomes
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:evaluator-skeleton
scope:
- confidence: cited
  path: conformance/scenarios/explanation.yaml
- confidence: cited
  path: crates/canon/src/eval/authority.rs
- confidence: cited
  path: crates/canon/src/eval/decision.rs
- confidence: cited
  path: crates/canon/src/eval/decisions.rs
- confidence: cited
  path: crates/canon/src/eval/mod.rs
- confidence: cited
  path: crates/canon/src/explain/
- confidence: cited
  path: crates/canon/src/model/decision.rs
- confidence: cited
  path: fixtures/investigation/explanation.yaml
- confidence: cited
  path: website/
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:50Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-04T07:10:25Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-10-04T08:26:15Z", actor: "human:timo", revision: 13, decided_on: {"recorded":{"test_result":1,"review_outcome":5,"verification":1}}}
---
## Outcome

Every non-`TRUE` claim, open obligation, non-admissible action and blocked outcome in a
`canon-decision/1` document carries a structured explanation: a chain of reasons down to the
evidence records that applied or were excluded, each exclusion with the reason recorded in its
claim's excluded-evidence list (`revision_mismatch`, `expired`). The explanation records what the
decision was computed from, so it can be reconstructed later: protocol id and revision, Canon
semantics version, case snapshot, evidence set, authority decisions and evaluation instant
(design § 37). Serialization is canonical: equivalent normalized inputs give byte-identical output
(CANON-DETERMINISM-001).

## Extends (operator decision 2026-10-04)

This story adds the `explanation` section to `canon-decision/1`, which story:three-valued-claims
defines and the earlier links of the chain extend. It does not introduce the document. Exclusions
are not decided here: it traces the excluded-evidence list in each `claims` entry that
story:evidence-revision-binding introduced and story:evidence-freshness extended, and adds no
exclusion reason of its own.

## Shared surface and order (re-plan 2026-10-04)

story:evaluator-skeleton lands the `explanation` slot in `canon-decision/1`, the placeholder type
in `explain/mod.rs` and the call from the pipeline, so this story fills `crates/canon/src/explain/`
and does not touch `eval/mod.rs`. It keeps `eval/decision.rs` in scope because it owns the
canonical serialization CANON-DETERMINISM-001 requires; no story beside it edits that file.

- Kept: depends_on story:evidence-revision-binding, story:obligations, story:action-admissibility,
  story:outcomes, story:evidence-freshness and story:decision-outcomes — each is behaviour this
  story explains (exclusion reasons, open obligations, non-admissible actions, blocked outcomes
  including decision-based ones); story:conformance-runner (its acceptance is a scenario).
- Added: depends_on story:evaluator-skeleton (slot, placeholder, pipeline call).
- It runs beside story:invalidation-rules, which it does not depend on: it traces the
  `invalidated` reason like any other and adds none.

## Scope

- Fixture: `fixtures/investigation/explanation.yaml`, a copy of the base; it exists so this story's
  scenario names a fixture no other story edits.
- Surfaces: `crates/canon/src/explain/`, `crates/canon/src/eval/decision.rs`,
  `fixtures/investigation/explanation.yaml`, `conformance/scenarios/explanation.yaml`.

## ESS first

- Specification change: none in `ess/`. The explanation section's structure is this story's, and
  the earlier plan gave the decision document's extension sections no ESS owner. The first commit
  is Canon's semantic specification for this story: the scenario file
  `conformance/scenarios/explanation.yaml` and its fixture.
- Red on that commit: scenario `CANON-EXPLAIN-001` fails under `canon conform run`, because the
  skeleton's explanation stub emits no `explanation` section.

## Acceptance

Conformance scenario `CANON-EXPLAIN-001` (also covers CANON-DETERMINISM-001) passes under `canon
conform run`, with three expectations for the investigation fixture with its only evidence bound to
a superseded revision: the `explanation` section traces the blocked `supported` outcome to
`explanation.supported` being `UNKNOWN` and from there to each excluded evidence record with reason
`revision_mismatch`; a second run gives byte-identical output; and the same evidence set in
reverse order gives byte-identical output.

## Source

TASKBOARD C-009 (build pack, now Atlas `docs/design/governed-autonomy/TASKBOARD.md`);
`docs/design/canon-protocol-calculus-design.md` § 13, § 14, § 37, § 41 item 13;
`docs/contracts/protocol-core.md`; CANON-DETERMINISM-001 (§ 32).

## Coordinator decisions (wave 2026-10-04-w10)

- Shape: `claims` lists each non-true claim with `because` (records applied or excluded, kinds absent);
  outcomes, actions and obligations point into `claims`; `computed_from` records protocol and revision,
  the case snapshot, the sorted evidence ids and any supplied authority, decisions and `at`.
- `semantics` is `{"canon": <crate version>, "format": "canon-decision/1"}`, because the design names
  no semantics identifier. A canon version bump must update CANON-EXPLAIN-001 and the generated example.
- Five existing tests that compared or searched whole decisions set the recorded `computed_from` field
  aside after asserting it, keeping every other byte compared.
