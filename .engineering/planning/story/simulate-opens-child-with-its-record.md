---
format: aep.planning-md/3
id: story:simulate-opens-child-with-its-record
kind: story
status: implemented
title: canon simulate opens a child together with its running record
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/canon-cli/src/lib.rs
- confidence: inferred
  path: crates/canon-cli/tests/canon_simulate.rs
- confidence: inferred
  path: crates/canon-cli/tests/simulate_child_outcome.rs
- confidence: inferred
  path: crates/canon/src/simulate/mod.rs
- confidence: inferred
  path: crates/canon/src/simulate/recovery.rs
- confidence: inferred
  path: crates/canon/src/simulate/tests.rs
- confidence: inferred
  path: crates/canon/tests/adversary3_simulate_recovery.rs
- confidence: inferred
  path: crates/canon/tests/adversary_child_outcome_subject.rs
- confidence: inferred
  path: crates/canon/tests/adversary_simulate.rs
- confidence: inferred
  path: crates/canon/tests/adversary_simulate_child_outcome.rs
- confidence: inferred
  path: crates/canon/tests/child_outcome_subject.rs
- confidence: inferred
  path: ess/domains/cli.yaml
- confidence: inferred
  path: fixtures/child-outcome-large
- confidence: inferred
  path: fixtures/investigation/child-outcome
- confidence: inferred
  path: fixtures/investigation/simulate
- confidence: inferred
  path: website/docs/reference/cli.md
revision: 22
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T03:55:39Z", actor: "human:timo", revision: 20, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w13"}
- {from: "proposed", to: "active", at: "2026-11-23T03:55:39Z", actor: "human:timo", revision: 21, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w13"}
- {from: "active", to: "implemented", at: "2026-11-23T07:14:16Z", actor: "human:timo", revision: 22, decided_on: {"recorded":{"test_result":1}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w13"}
---
## Outcome

In the `canon simulate` walk a child is opened together with its running `case_outcome` record `open`, as the runner writes it, so a probe cannot reach a state a real run cannot.

## Why

- Gap 8 of `bf-runbook` (dev-operator, 2026-11-22): the runner writes the child's record `open` in the step that opens it; the walk adds it in a later child-outcome step. In between, a `children` form over four parallel readers on one subject reads "three ended complete, the fourth opened with no record" as true; the probe `nobody-after-incomplete-read` on `needs-you` is reachable in the walk and not in a real run. With one reader it holds.
- The evaluator is not the cause: `eval/composition.rs:68-88` gives `unknown` while any child has an `open` record. The opening step only raises the action's count of open children (`simulate/mod.rs:1333-1368`) and the `open` record arrives in `:1430-1483`. A replacing child-outcome step that takes the count from two to one also drops the shared record.

## Scope

Derived 2026-11-23 by `story-scoper`; entries marked inferred are readings, not facts.

- **In:** the opening step (its record, group and revise forms) adds the `open` record wherever the action's children record; a count above none always holds an `open` record and the reverse (`open_held` at `mod.rs:1264` covers one direction; `recovery::late` goes through it).
- **Decisions, defaults:** D-a the opening step keeps its text; D-b a `case_outcome` dimension with no `open` class gets no record at opening; D-c the child step that records `open` stays.
- **Out:** any change to `protocol/1`, `canon-ir/1`, the evaluator or `canon check`; fixture inputs (`protocol.yaml`).
- **Surfaces:** `crates/canon/src/simulate/mod.rs`, `crates/canon/src/simulate/recovery.rs`, `crates/canon/src/simulate/tests.rs`, `crates/canon-cli/src/lib.rs`, `ess/domains/cli.yaml`, `website/docs/reference/cli.md`, tests `canon_simulate.rs`, `simulate_child_outcome.rs`, `child_outcome_subject.rs`, `adversary_child_outcome_subject.rs`, `adversary_simulate_child_outcome.rs`, `adversary_simulate.rs`, `adversary3_simulate_recovery.rs`, fixtures `fixtures/investigation/child-outcome`, `fixtures/child-outcome-large`, `fixtures/investigation/simulate`.
- **Expected report changes (15 fixtures, to be confirmed by a run):** `investigation/child-outcome/{closed,denied,ends-open,repeat,twice,two-records}`, `child-outcome-large/{event,festival}`, `investigation/simulate/{child-expiry,child-expiry-held,revise-child,revise-child-current,revise-child-late,revise-child-other-subject,revise-child-two-subjects}`. The other 158 of 173 fixtures keep identical simulate bytes.
- **State count:** by hand-trace the new reachable set is a subset of the old one, so no protocol newly exceeds the 65,536 bound; findings can still change (a state only escapable through an unreal state becomes stuck).
- **Order:** first of the two simulate stories (`story:refused-approval-is-not-stuck` follows).

## Acceptance

1. New fixture `fixtures/investigation/simulate/children-open-record/`: four opening actions on one subject, a claim `children: {in: [complete], subject: ...}`, an outcome `probe` that requires a `none` record while the claim is not true. On the base commit `canon simulate` prints no `unreached-outcome: outcome probe` (red); after the change it prints it (green).
2. Its one-reader twin prints `unreached-outcome: outcome probe` before and after.
3. On the 15 fixtures every state of the new walk is a state of the old one (a test compares the sets of rendered states, or the reachable count is not larger); each changed `report.txt` is named in the commit.
4. In no printed state does a count above none appear without `result open` for the same dimension, nor the reverse.
5. The other 158 fixtures give byte-identical `canon simulate` output against a build of `210f929d9`; `check`, `generate` and `conform` stay byte-identical on all 173.
6. New tests under 5 s; the 60 s and 120 s bounds hold. needs-you's walked state count before and after is measured on the consumer's files and reported.

## Source

- **Request:** `bf-runbook` gap list of 2026-11-22; accepted by host-operator on Timo's delegation, 2026-11-23.
