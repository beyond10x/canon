---
format: aep.planning-md/3
id: story:refused-approval-is-not-stuck
kind: story
status: implemented
title: A refused approval ends in a legitimate state, not a stuck one
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
  path: crates/canon/src/simulate/tests.rs
- confidence: inferred
  path: ess/domains/cli.yaml
- confidence: inferred
  path: fixtures/investigation/child-outcome/denied
- confidence: inferred
  path: fixtures/investigation/simulate
- confidence: inferred
  path: website/docs/concepts/protocols.md
- confidence: inferred
  path: website/docs/reference/cli.md
- confidence: inferred
  path: website/status.yaml
revision: 17
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T03:55:38Z", actor: "human:timo", revision: 15, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w13"}
- {from: "proposed", to: "active", at: "2026-11-23T03:55:38Z", actor: "human:timo", revision: 16, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w13"}
- {from: "active", to: "implemented", at: "2026-11-23T07:14:17Z", actor: "human:timo", revision: 17, decided_on: {"recorded":{"test_result":1}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w13"}
---
## Outcome

When a required capability is denied, `canon simulate` gives the protocol's author a way to end the case that needs no workaround action, or names the existing form that does it.

## Why

- Gap 5 of `bf-runbook` (dev-operator, 2026-11-22): with `requires: [{capability: owner.approve}]` denied, `canon simulate` prints `stuck-state: ... capability owner.approve denied: no outcome is legitimate and no action is admissible or approval-required`. Predicates cannot read authority decisions, so the consumer added an `approval.lapse` action (no capability) and an outcome `unapproved` in two protocols.
- The stuck-state rule is applied in one place, `crates/canon/src/simulate/mod.rs:1583-1597`, its text at `:442-448`. Decision steps do not count when simulate decides whether a state is open (rule R4, `:190-193`): an outcome that requires a decision does not remove the finding, it only adds "decision d taken makes outcome o legitimate". `canon check` has no stuck-state finding.

## Scope

Derived 2026-11-23 by `story-scoper`; entries marked inferred are readings, not facts.

- **In (decision D3):** option A: when a printed stuck-state or cycle holds a denied capability that blocks an action whose precondition is true, the text adds one clause naming that block and the two existing forms that declare the end (an outcome with `requires: {decision: ..., when: ...}`, or an action without the capability whose record an outcome reads); exit stays 1. Option B, on top of A: a state that is not open, where a denial blocks an action and a decision step leads to a legitimate outcome, is no longer a finding; the summary counts it (`, D ending by a decision after a denial`, printed only when D > 0). B reverses part of the R4 decision taken after adversary pass 1 of wave 9.
- **Out:** any change to `protocol/1` (option C, a predicate that reads authority, is a format and architecture change and waits for Timo); `canon check`; `eval/authority.rs`.
- **Surfaces:** `crates/canon/src/simulate/mod.rs`, `crates/canon/src/simulate/tests.rs`, `crates/canon-cli/tests/canon_simulate.rs`, `fixtures/investigation/simulate/`, `fixtures/investigation/child-outcome/denied/report.txt`, `crates/canon-cli/tests/simulate_child_outcome.rs`, `crates/canon-cli/src/lib.rs`, `ess/domains/cli.yaml`, `website/docs/reference/cli.md`; inferred: `crates/canon/src/check/mod.rs` (only if the denial reasons must be carried out of `Evaluated`), `website/docs/concepts/protocols.md`, `website/status.yaml`.
- **Order:** runs after `story:simulate-opens-child-with-its-record` (same walk file).

## Acceptance

1. First commit: only the `canon.cli.Simulate` comments in `ess/domains/cli.yaml` and a fixture `fixtures/investigation/simulate/denied-decision/` (`owner.approve` required on the approval, plus an outcome `unapproved` that requires a decision); a named test in `canon_simulate.rs` is red on it.
2. Option A: in `simulate/denied` the one stuck-state line names the denied capability, the action it blocks and both existing forms; exit stays 1; the five `report.txt` files that hold a denial (`simulate/{denied,contained,cycle,revise-denied}`, `child-outcome/denied`) are the only ones that change.
3. Option B: `denied-decision` exits 0 and the summary carries `1 ending by a decision after a denial`; `decision-exit/report.txt` is byte-identical.
4. No change under `crates/canon/src/model/` or `ess/domains/protocol.yaml`; `task ess-gate` and `task docs-check` pass.
5. New tests under 5 s; the 60 s and 120 s bounds hold.

## Source

- **Request:** `bf-runbook` gap list of 2026-11-22; accepted by host-operator on Timo's delegation, 2026-11-23.
