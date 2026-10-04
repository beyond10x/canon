---
format: aep.planning-md/3
id: story:evidence-revision-binding
kind: story
status: implemented
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
- depends_on: story:evaluator-skeleton
scope:
- confidence: cited
  path: conformance/scenarios/evidence-revision-binding.yaml
- confidence: cited
  path: crates/canon-docs/src/pages.rs
- confidence: cited
  path: crates/canon/src/eval/binding.rs
- confidence: cited
  path: crates/canon/src/eval/mod.rs
- confidence: cited
  path: crates/canon/src/model/evidence.rs
- confidence: cited
  path: ess/domains/protocol.yaml
- confidence: cited
  path: fixtures/investigation/evidence-revision-binding.yaml
- confidence: cited
  path: website/docs/concepts/evidence-and-revisions.md
- confidence: cited
  path: website/docs/status/where-this-stands.md
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:50Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-04T05:19:06Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-10-04T06:13:33Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"test_result":1,"review_outcome":5,"verification":1}}}
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

## Shared surface and order (re-plan 2026-10-04)

The evaluator chain of the earlier plan is gone: story:evaluator-skeleton splits
`crates/canon/src/eval/` into one file per concept and lands the `excluded_evidence` slot in each
`claims` entry with its three reasons. This story owns `eval/binding.rs`, the revision-binding
exclusion stage, and runs beside story:obligations, story:action-admissibility, story:outcomes and
story:evidence-freshness.

- Kept: depends_on story:three-valued-claims (it binds the evidence and case shapes that story
  defines and changes which evidence the claim evaluation sees), story:conformance-runner (its
  acceptance is a scenario), story:evaluator-skeleton (its file, stage hook and slot).
- Not an edge any more: nothing orders it against the other evaluator stories.

## Scope

- Out: whether Canon owns a cross-project evidence envelope or consumes a shared one (design
  § 39.3) is not decided here; this story defines only what the pure evaluator reads.
  Freshness (C-008). Invalidation through an upstream artifact (see
  `decision-blocker:upstream-revision-binding`).
- Fixture: `fixtures/investigation/evidence-revision-binding.yaml`, a copy of the base (the base
  already declares the explanation artifact); it exists so this story's scenario names a fixture no
  other story edits.
- Surfaces: `crates/canon/src/eval/binding.rs`,
  `fixtures/investigation/evidence-revision-binding.yaml`,
  `conformance/scenarios/evidence-revision-binding.yaml`.

## ESS first

- Specification change: none in `ess/`. The subject fields already exist (story:three-valued-claims)
  and the `excluded_evidence` slot and its reasons are story:evaluator-skeleton's. The first commit
  is Canon's semantic specification for this story: the scenario file
  `conformance/scenarios/evidence-revision-binding.yaml` and its fixture.
- Red on that commit: scenario `CANON-EVIDENCE-001` fails under `canon conform run`, because the
  skeleton's binding stage excludes nothing: the claim stays `TRUE` after the case snapshot advances
  the artifact, and an evidence record naming an undeclared subject is not refused.

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

### Coordinator and adversary decisions (wave 2026-10-04-w7)

- An evidence record about an undeclared artifact is refused as `undeclared-artifact`, naming it.
- Pass 1 F1: the behaviour is right and the doc was wrong. Exclusion removes the record; a claim
  decided by an evidence match on it becomes UNKNOWN, while a claim that tests `is: unknown` is
  decided by that unknown and can become TRUE or FALSE. The Outcome line above ("never FALSE")
  holds for evidence-decided claims only (fixed in docs).
- Pass 1 F2: no published page says subject or revision has no effect (fixed).
- Pass 2: a stale record is listed as excluded under each claim that reaches its kind, not at
  decision level; the docs say so (fixed). Present tense and the UNKNOWN wording on the concept pages
  (fixed). Pages outside the typed scope overlap with parallel units (note); the hard-coded example
  sentence in canon-docs (no-op).
