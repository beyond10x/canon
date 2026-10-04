---
format: aep.planning-md/3
id: epic:protocol-analysis
kind: epic
status: draft
title: 'Protocol analysis: static checks, generated scenarios and mutation testing'
relations:
- serves: vision:governed-autonomy
- serves: vision:O2
revision: 2
---
## Outcome

Canon analyses a `protocol/1` document without IO: it finds defects by exhaustive static checking,
generates the conformance scenarios that witness each outcome and each blocked action, and measures
how well those scenarios pin the protocol by mutating it.

1. **`canon check`.** Exhaustive over the protocol's finite state space: which evidence kinds are
   present and with which result values, each obligation's status, each authority decision. It
   reports:
   - outcomes no state reaches;
   - actions whose precondition holds in no state;
   - claims that need an evidence kind no action's `may_produce` lists;
   - outcomes reachable without any action that requires authority (an authority bypass);
   - declared properties checked over all states, for example "`emergency.leave` admissibility is
     independent of `cause.identified`" (Atlas `docs/design/governed-autonomy/incident-response-walkthrough.md`
     § 1). A way to declare properties beside a protocol is needed; its format is open.
2. **Scenario generation.** One minimal witness evidence set per outcome and per blocked action,
   written as `canon-conformance/1` scenarios that `canon conform run` (story:conformance-runner,
   implemented) executes.
3. **`canon mutate`.** Mutation operators: drop a conjunct, `all` → `any`, remove a precondition,
   remove a capability requirement, change an evidence result value. Each mutant runs against the
   conformance scenarios; survivors are reported. Each mutant is classified with the semantic diff
   (story:semantic-diff, proposed) as weakening or strengthening; a weakening survivor is a weak point.
   The classification depends on semantic-diff.

## Decision

Operator decision of 2026-10-04, option A: the analysis lives in Canon. Rejected: B, extending
ESS hardening tooling to read Canon protocols. The analysis needs Canon's evaluator and IR; ESS works
on system specifications.

## Prior art

- The `ess:hardening` skill's mutation technique (mutate, run the suite, report survivors).
- ESS reachability checks for lifecycle states.

## Open

- The format for declaring properties beside a protocol.
- The bound on state-space size before `canon check` refuses (the space grows with evidence kinds,
  result values, obligations and authority decisions).

Both are settled in the first step of story:canon-check.

## Depends on

story:three-valued-claims (evaluation), story:conformance-runner (implemented), story:semantic-diff
(classification in part 3), and the obligation and authority stories for those dimensions of the
state space.

## Decomposition

| part | story | depends on |
|---|---|---|
| 1 | story:canon-check | story:three-valued-claims, story:obligations, story:action-admissibility, story:outcomes, story:decision-outcomes |
| 2 | story:scenario-generation | story:canon-check, story:conformance-runner |
| 3 | story:canon-mutate | story:semantic-diff, story:conformance-runner, story:action-admissibility |

## Acceptance

Over the investigation fixture family, `canon check` names each seeded defect and passes the base,
`canon generate` writes witness scenarios that `canon conform run` passes, and `canon mutate`
reports a weakening survivor for a scenario set that lacks a witness and not once the witness is added.
Each part's named test is its story's acceptance.

## Source

Draft `canon-epic-protocol-analysis.md` (wave 2026-10-04-w4 hand-off); operator decision
2026-10-04 (option A).
