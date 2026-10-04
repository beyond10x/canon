---
format: aep.planning-md/3
id: story:evidence-revision-binding
kind: story
status: proposed
title: Apply evidence only to the revision it is bound to
refs:
- provider: taskboard
  reference: C-004
relations:
- decomposes: epic:canon-kernel
- depends_on: story:three-valued-claims
- depends_on: story:conformance-runner
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: conformance/scenarios/evidence-revision-binding.yaml
- confidence: cited
  path: crates/canon/src/eval/
- confidence: cited
  path: fixtures/investigation/evidence-revision-binding.yaml
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:50Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome

Evidence is applicable only to the revision it is bound to. Each evidence record names one subject
artifact and the revision it concerns; the case snapshot names the current revision of each
artifact; evidence whose subject revision is not current is excluded before claims are evaluated,
so it never establishes or contradicts a claim about the current revision (design § 9). Exclusion
leaves a claim `UNKNOWN`, never `FALSE`.

## Extends (operator decision 2026-10-04)

This story extends the shapes story:three-valued-claims defines and introduces none:

- `canon-evidence/1`: the subject and subject revision fields become binding. A record whose subject
  is not a declared artifact is refused naming it.
- `canon-case/1`: its artifact revisions become the reference the binding is checked against.
- `canon-decision/1`: each entry in `claims` gains an excluded-evidence list, each item an evidence
  id and its reason, here `revision_mismatch`. story:evidence-freshness adds the reason `expired`
  to the same list, and story:explanation traces it.

## Shared surface and order (operator decision 2026-10-04)

`crates/canon/src/eval/` and `canon-decision/1` are shared by the evaluator chain three-valued-claims
→ evidence-revision-binding → obligations → action-admissibility → outcomes → evidence-freshness →
explanation. This story depends_on story:three-valued-claims for that reason and runs after it.

## Scope

- Out: whether Canon owns a cross-project evidence envelope or consumes a shared one (design
  § 39.3) is not decided here; this story defines only what the pure evaluator reads.
  Freshness (C-008). Invalidation through an upstream artifact (see
  `decision-blocker:upstream-revision-binding`).
- Fixture: `fixtures/investigation/evidence-revision-binding.yaml`, a copy of the base (the base
  already declares the explanation artifact); it exists so this story's scenario names a fixture no
  other story edits.
- Surfaces: `crates/canon/src/eval/`, `fixtures/investigation/evidence-revision-binding.yaml`,
  `conformance/scenarios/evidence-revision-binding.yaml`.

## Domain relations

- Case → Artifact: a case snapshot names exactly one current revision per declared artifact.
  Stated in design § 9. Not an ess/1 document: Canon opts out of ESS for language semantics
  (AGENTS.md § ESS, Atlas ADR 0067).
- Evidence → Artifact revision: each evidence record is bound to exactly one subject artifact
  revision; many records may bind to the same revision. Stated in design § 9 and CANON-EVIDENCE-001
  (§ 32). Not an ess/1 document, for the same reason.

## Acceptance

Conformance scenario `CANON-EVIDENCE-001` passes under `canon conform run`, with three expectations:
`explanation.supported` is `TRUE` while its evidence is bound to the current revision of the
explanation artifact; it is `UNKNOWN`, not `FALSE`, with the same evidence when only the case
snapshot advances that artifact to a new revision; and an evidence record whose subject is not a
declared artifact is refused, the refusal naming that subject.

## Source

TASKBOARD C-004 (build pack, now Atlas `docs/design/governed-autonomy/TASKBOARD.md`);
`docs/design/canon-protocol-calculus-design.md` § 9, § 39.3, § 41 item 5; CANON-EVIDENCE-001 (§ 32).
