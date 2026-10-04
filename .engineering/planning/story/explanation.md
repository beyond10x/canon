---
format: aep.planning-md/3
id: story:explanation
kind: story
status: proposed
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
scope:
- confidence: cited
  path: conformance/scenarios/explanation.yaml
- confidence: cited
  path: crates/canon/src/eval/
- confidence: cited
  path: crates/canon/src/explain/
- confidence: cited
  path: fixtures/investigation/explanation.yaml
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:50Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
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

## Shared surface and order (operator decision 2026-10-04)

`crates/canon/src/eval/` and `canon-decision/1` are shared by the evaluator chain three-valued-claims
→ evidence-revision-binding → obligations → action-admissibility → outcomes → evidence-freshness →
explanation. This story is the last link and depends_on story:evidence-freshness for that reason.

## Scope

- Fixture: `fixtures/investigation/explanation.yaml`, a copy of the base; it exists so this story's
  scenario names a fixture no other story edits.
- Surfaces: `crates/canon/src/explain/`, `crates/canon/src/eval/`,
  `fixtures/investigation/explanation.yaml`, `conformance/scenarios/explanation.yaml`.

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
