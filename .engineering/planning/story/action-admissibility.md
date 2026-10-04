---
format: aep.planning-md/3
id: story:action-admissibility
kind: story
status: proposed
title: Evaluate action admissibility from preconditions and authority
refs:
- provider: taskboard
  reference: C-006
relations:
- decomposes: epic:canon-kernel
- depends_on: story:three-valued-claims
- depends_on: story:conformance-runner
- depends_on: story:obligations
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: conformance/scenarios/action-admissibility.yaml
- confidence: cited
  path: crates/canon-cli/
- confidence: cited
  path: crates/canon/src/eval/
- confidence: cited
  path: fixtures/investigation/action-admissibility.yaml
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:50Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome

Canon evaluates every declared action against the case. An action is `admissible` only when its
precondition predicate evaluates `TRUE` and every capability requirement it declares is granted by
an authority decision passed in as evaluation input. An action whose precondition is not `TRUE` is
`blocked` with a reason naming the claim and its value. An action whose capability has no decision
is `approval-required` naming the capability. An action whose capability is denied is — decided
here — `blocked` with a reason naming the denied capability, because asking again would not change
a denial (design § 10, § 14, CANON-AUTHORITY-001). Canon resolves no identities and grants nothing:
authority decisions are input (design § 33).

## Extends (operator decision 2026-10-04)

- `canon evaluate` gains `--authority`, reading `canon-authority/1`: a list of decisions, each a
  capability and `granted` or `denied`.
- `canon-decision/1`, defined by story:three-valued-claims, gains the `actions` section: each
  action's id, status and reasons.

## Shared surface and order (operator decision 2026-10-04)

`crates/canon/src/eval/` and `canon-decision/1` are shared by the evaluator chain three-valued-claims
→ evidence-revision-binding → obligations → action-admissibility → outcomes → evidence-freshness →
explanation. This story depends_on story:obligations for that reason and runs after it.

## Scope

- Fixture (operator decision 2026-10-04): the variant `fixtures/investigation/action-admissibility.yaml`
  is the base plus one action, `publish_finding`, whose precondition is `explanation.supported`
  `TRUE` and which requires the capability `finding.publish`. The base fixture has neither
  preconditions nor capability requirements; it is not edited.
- Out: frontier calculation (`canon frontier`, design § 41 item 10), which the epic does not promise.
- Surfaces: `crates/canon/src/eval/`, `crates/canon-cli/`,
  `fixtures/investigation/action-admissibility.yaml`, `conformance/scenarios/action-admissibility.yaml`.

## Domain relations

- Action → capability requirement → authority decision keyed by capability: an action may declare
  capability requirements, and an authority decision grants or denies one capability. Stated in
  design § 6 and § 10; the approval-required status carrying a capability is inferred from the
  bootstrap `ActionStatus` in `crates/canon/src/lib.rs`, which story:protocol-source-model replaces.
  Not an ess/1 document: Canon opts out of ESS for language semantics (AGENTS.md § ESS, Atlas ADR 0067).

## Acceptance

Conformance scenario `CANON-AUTHORITY-001` passes under `canon conform run` over
`fixtures/investigation/action-admissibility.yaml`, with four expectations for `publish_finding`:
`blocked` naming `explanation.supported` and its value while that claim is not `TRUE`;
`approval-required` naming `finding.publish` when the claim is `TRUE` and `--authority` carries no
decision for it; `blocked` naming `finding.publish` as denied when `--authority` denies it; and
`admissible` when the claim is `TRUE` and `--authority` grants it.

## Source

TASKBOARD C-006 (build pack, now Atlas `docs/design/governed-autonomy/TASKBOARD.md`);
`docs/design/canon-protocol-calculus-design.md` § 6, § 10, § 14, § 29, § 33, § 41 items 8–9;
`docs/contracts/protocol-core.md`; CANON-AUTHORITY-001 (§ 32).
