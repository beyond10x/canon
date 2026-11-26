---
format: aep.planning-md/3
id: epic:trigger-rules
kind: epic
status: draft
title: 'Trigger rules: declare when the world starts a protocol, and simulate it'
summary: canon-trigger/1 rule documents and a pure evaluate(rule_set, snapshot, protocols, history, now) -> plan; the evaluator lives in beyond10x/runbook.
relations:
- serves: vision:O2
- serves: vision:governed-autonomy
revision: 1
---
## Outcome

A rule "entities of kind K in state S where X holds start protocol P with input M" is declared once
in a shared, versioned Canon document (`canon-trigger/1`), and a rule set is evaluated by one pure
operation against a state snapshot, the protocols' declared classes, a firing and approval history
and an explicit instant, giving a deterministic plan of matches with a verdict each. Canon starts
nothing; an evaluator outside Canon acts on the plan.

## Why

Consumers that run protocols want some of them started by the state of the world. Today each
writes its own rule file and matching and cannot test a rule set without a live engine. The owner
decided on 2026-11-26 that the declaration and its semantics live in Canon, for simulation, and the
evaluator in `beyond10x/runbook`.

## Scope

- **In:** the rule document and its validation; the `where` language as a restricted
  `canon-expr/1`; a protocol's access class and guard; `evaluate` with the verdicts `would-start`,
  `offer`, `baseline`, `held`, `refused`, `undetermined`; the publication check over a rule set;
  the `canon trigger` subcommands, ESS domain `canon.trigger`, conformance scenarios and reference
  pages.
- **Out:** snapshot building, storage of firings and approvals, who may approve, run submission,
  any daemon. Those are the evaluator's.

## Acceptance

The nine acceptance statements of the request hold, each by a named test or conformance scenario:
validation names the failing field; two versions are two keys and an edited published key fails;
`evaluate` is byte-identical over repeated runs and any input row order; 3 of 7 rows match as 3
`would-start` or 3 `offer`; re-entry gives a new firing id except under `once_per_subject`;
matches at activation are `baseline`; the three `refused` cases; the two `held` cases; and
`evaluate` runs with no writable directory and no network.

## Design

`docs/design/trigger-rules.md`.

## Source

- **Request:** trigger-rule kernel request of 2026-11-26 from the consumer coordinator, with its
  companion request to `beyond10x/runbook` for the evaluator.
- **Decision:** the owner, 2026-11-26: "kernel in canon (for simulation purposes) and evaluator in
  runbook".
