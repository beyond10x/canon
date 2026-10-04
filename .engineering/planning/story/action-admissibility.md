---
format: aep.planning-md/3
id: story:action-admissibility
kind: story
status: implemented
title: Evaluate action admissibility from preconditions and authority
refs:
- provider: taskboard
  reference: C-006
relations:
- decomposes: epic:canon-kernel
- depends_on: story:three-valued-claims
- depends_on: story:conformance-runner
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:evaluator-skeleton
scope:
- confidence: cited
  path: conformance/scenarios/action-admissibility.yaml
- confidence: cited
  path: crates/canon-cli/tests/adversary2_skel_conform_cli.rs
- confidence: cited
  path: crates/canon-cli/tests/evaluator_skeleton.rs
- confidence: cited
  path: crates/canon/src/eval/actions.rs
- confidence: cited
  path: crates/canon/src/eval/authority.rs
- confidence: cited
  path: crates/canon/tests/adversary2_skel_authority.rs
- confidence: cited
  path: fixtures/investigation/action-admissibility.yaml
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:50Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-04T05:19:06Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-10-04T06:13:33Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1,"review_outcome":4,"verification":1}}}
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

## Shared surface and order (re-plan 2026-10-04)

The evaluator chain of the earlier plan is gone. story:evaluator-skeleton lands the `--authority`
flag (read and passed through unparsed), the `actions` slot and the files `eval/actions.rs` and
`eval/authority.rs`. This story parses `canon-authority/1` in `authority.rs`, evaluates actions in
`actions.rs`, and runs beside story:evidence-revision-binding, story:obligations, story:outcomes
and story:evidence-freshness.

- Kept: depends_on story:three-valued-claims (preconditions are evaluated over its claim values),
  story:conformance-runner (its acceptance is a scenario), story:evaluator-skeleton (flag, files,
  slot).
- Removed: depends_on story:obligations (ordering only, on the shared `eval/` directory and
  `crates/canon-cli/`, now split).

## Scope

- Fixture (operator decision 2026-10-04): the variant `fixtures/investigation/action-admissibility.yaml`
  is the base plus one action, `publish_finding`, whose precondition is `explanation.supported`
  `TRUE` and which requires the capability `finding.publish`. The base fixture has neither
  preconditions nor capability requirements; it is not edited.
- Out: frontier calculation (`canon frontier`, design § 41 item 10), which the epic does not promise.
- Surfaces: `crates/canon/src/eval/actions.rs`, `crates/canon/src/eval/authority.rs`,
  `fixtures/investigation/action-admissibility.yaml`, `conformance/scenarios/action-admissibility.yaml`.

## ESS first

- Specification change: none in `ess/`. `canon-authority/1` and the `actions` section are not
  declared in `ess/` by this story or any other (the earlier plan did not give them an ESS owner
  either, and the shape of an action's reasons is not settled enough to declare up front). The
  first commit is Canon's semantic specification for this story: the scenario file
  `conformance/scenarios/action-admissibility.yaml` and its fixture.
- Red on that commit: scenario `CANON-AUTHORITY-001` fails under `canon conform run`, because the
  skeleton refuses `--authority` as unsupported and emits no `actions` section.

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

## From wave 2026-10-04-w6 (evaluator-skeleton, adversary pass 2, J3)

`conform/mod.rs` reads a scenario's `authority` as a YAML list. If `canon-authority/1` gains a
`format:` header like the other Canon inputs, this story owns that change to `conform/`.

### Coordinator and adversary decisions (wave 2026-10-04-w7)

- Section shape: keyed by action id, every declared action listed; `canon-authority/1` is a list of
  `{capability, decision: granted|denied}` without a format header; a denied capability outranks an
  undecided one. Malformed authority input (mapping keys, tags, `.nan`) is refused as
  `malformed-input`.
- Pass 1 F1/F2: precondition reasons come from the deciding claim tests, with polarity (fixed). F3:
  the stale `--authority` help and eval docs go to the wave docs pass (escalated). F4: capability
  reasons are sorted and deduplicated (fixed).
- Pass 2 F1: an unsatisfiable precondition gives `{requirement: unsatisfiable}` (fixed). F2: evidence
  reasons are `{evidence, present}` as in story:outcomes (fixed); a false `all` names only its false
  members here, every unmet member in story:outcomes (open, to harmonise later).
