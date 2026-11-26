---
format: aep.planning-md/3
id: story:trigger-holds-and-refusals
kind: story
status: active
title: evaluate holds repeated and capped matches and refuses what the rule set does not allow
relations:
- decomposes: epic:trigger-rules
- serves: vision:governed-autonomy
- depends_on: story:trigger-evaluate
- depends_on: story:protocol-access-class
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-26T11:25:20Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-26T11:25:21Z", actor: "human:operator", revision: 3}
---
## Outcome

`evaluate` lists `held` (`already-fired`, `cap-reached`) and `refused` (`disabled`,
`unknown-protocol`, `guarded-auto`, `access-mismatch`, `write-unapproved`, `entered-at-missing`)
with their reasons, in the precedence of design § 6.

## Why

Request acceptance 7 and 8: an evaluator must never start a match the rule set does not allow, and
evaluating twice must not start a match twice.

## Scope

- **In:** approvals in `canon-trigger-history/1`, read for the exact rule key; the `protocols` map
  from story:protocol-access-class; the cap counted per UTC day of `now`, from the history and then
  from the plan's own earlier `would-start` entries in plan order; one conformance scenario per
  reason.
- **Out:** who may write an approval; the publication check.
- **Surfaces:** `crates/canon/src/trigger/`, `ess/domains/trigger.yaml`, `conformance/`,
  `website/docs/reference/`.

## Acceptance

A `write` rule in `auto` with no approval for its key lists `refused` (`write-unapproved`), and
with an approval for the previous version still `refused`; a guarded protocol in `auto` lists
`refused` (`guarded-auto`); a rule flag differing from the protocol class lists `refused`
(`access-mismatch`); a `disabled` rule lists `refused` (`disabled`); with `cap: 2`, 1 firing today
in the history and 3 matches, the plan lists 1 `would-start` and 2 `held` (`cap-reached`);
evaluating again with the plan's firings added to the history lists every match `held`
(`already-fired`).

## ESS first

First commit: approvals and the reason values in `ess/domains/trigger.yaml` and a conformance
scenario per acceptance line; the scenarios fail (red recorded).

## Source

- **Request:** trigger-rule kernel request of 2026-11-26, § Verdicts, acceptance 7 and 8.
- **Design:** `docs/design/trigger-rules.md` § 6.
