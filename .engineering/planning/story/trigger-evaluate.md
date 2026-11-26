---
format: aep.planning-md/3
id: story:trigger-evaluate
kind: story
status: draft
title: evaluate lists every match with its firing id, deterministically and without IO
relations:
- decomposes: epic:trigger-rules
- serves: vision:governed-autonomy
- depends_on: story:trigger-rule-document
revision: 1
---
## Outcome

`b10x_canon::trigger::evaluate(rule_set, snapshot, protocols, history, now) -> plan` and
`canon trigger evaluate` list every match of every rule with its firing id and the verdicts
`would-start`, `offer`, `baseline` and `undetermined`, deterministically and with no IO.

## Why

Request acceptance 3, 4, 5, 6 and 9: a rule set is simulated against a snapshot before it is live,
and the evaluator in `beyond10x/runbook` links this operation.

## Scope

- **In:** the documents `canon-trigger-snapshot/1`, `canon-trigger-history/1` (firings,
  activations) and `canon-trigger-plan/1`; matching on kind, state field and `where`; `UNKNOWN`
  `where` listed as `undetermined`; the firing id for both `refire` values; the input built from
  `start.input`; `baseline` from the activation instant or its row list; `draft` rules evaluated
  and marked `rule_state: draft`; plan order by rule key, kind, row id; `canon trigger evaluate`.
- **Out:** `held` and `refused` (story:trigger-holds-and-refusals).
- **Surfaces:** `crates/canon/src/trigger/`, `crates/canon-cli/src/lib.rs`,
  `ess/domains/trigger.yaml`, `ess/domains/cli.yaml`, `conformance/`, `website/docs/reference/`.

## Acceptance

Over 3 matching and 4 non-matching rows an active `auto` rule lists 3 `would-start` and an active
`offer` rule 3 `offer`; the plan's bytes are equal over 2 runs and over a reversed row order; a row
with a later `entered_at` gives a second firing id, and one firing id under `once_per_subject`;
rows that matched at activation list `baseline`; a `where` over an absent field lists
`undetermined`; `canon trigger evaluate` exits 0 in a read-only directory with no network
namespace (`unshare -n`).

## ESS first

First commit: the three documents in `ess/domains/trigger.yaml`, `trigger-evaluate` in `cli.yaml`,
a conformance scenario per acceptance line; the model and command parity tests fail (red recorded).

## Source

- **Request:** trigger-rule kernel request of 2026-11-26, § Semantics needed for simulation.
- **Design:** `docs/design/trigger-rules.md` § 6.
