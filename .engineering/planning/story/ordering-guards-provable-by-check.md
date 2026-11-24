---
format: aep.planning-md/3
id: story:ordering-guards-provable-by-check
kind: story
status: implemented
title: canon check proves an ordering guard (an action never runs before another)
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/canon-cli/tests/canon_check.rs
- confidence: inferred
  path: crates/canon/src/check/mod.rs
- confidence: inferred
  path: crates/canon/src/check/tests.rs
- confidence: inferred
  path: crates/canon/src/diff/mod.rs
- confidence: inferred
  path: crates/canon/src/eval/read.rs
- confidence: inferred
  path: crates/canon/src/ir/mod.rs
- confidence: inferred
  path: crates/canon/src/model/mod.rs
- confidence: inferred
  path: crates/canon/src/model/write.rs
- confidence: inferred
  path: crates/canon/src/mutate/mod.rs
- confidence: inferred
  path: crates/canon/src/validate/mod.rs
- confidence: inferred
  path: ess/domains/protocol.yaml
- confidence: inferred
  path: fixtures/investigation/check
- confidence: inferred
  path: website/docs/concepts/protocols.md
- confidence: inferred
  path: website/docs/reference
- confidence: inferred
  path: website/static/schemas/protocol-1.schema.json
- confidence: inferred
  path: website/status.yaml
revision: 22
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T03:55:42Z", actor: "human:timo", revision: 2, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w13"}
- {from: "proposed", to: "active", at: "2026-11-23T03:55:43Z", actor: "human:timo", revision: 3, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w13"}
- {from: "active", to: "implemented", at: "2026-11-23T23:09:05Z", actor: "human:timo", revision: 22, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w14"}
---
## Outcome

A guard such as the digest is never posted before the commit is provable by canon check, not only by a simulate probe.

## Why

- Gap 7: check evaluates every combination of records and does not follow preconditions; the consumer proves each guard with a probe outcome that simulate must report as unreached-outcome.
- Reported by the session `bf-runbook` (dev-operator), 2026-11-22, from `canon-gaps.md` in the dev-operator spec directory (22 composed protocols, `runbook` 0.5.0 linking Canon 0.11.0). Canon has not reproduced it; the file quotes the exact error texts.

## Scope

- **In:** the outcome above, as a new optional action field `runs_only_when: <predicate>` of `protocol/1` (decision D8, option A below). The field is an assertion: `canon evaluate` and `canon simulate` never read it, and an action without it lowers to the same canon-ir/1 bytes.
- **Out:** any change to the bytes of existing `protocol/1` inputs. The format change itself was approved by Timo (his answer to `bf-runbook` at 04:52:04 CET on 2026-11-23, read by `host-operator`).
- **Derived** 2026-11-23 by `story-scoper` (main `6dc461644`, Canon 0.14.0). Entries marked inferred are readings.

### Surfaces

- `crates/canon/src/check/mod.rs`: a new check subject and finding `ordering-guard-failed` beside `Subject::Property` (cited, `:358`); the finding comes last, the summary line is unchanged.
- `crates/canon/src/model/mod.rs` `Action` (`:241`) and `model/write.rs` (`:432`, `:478`); `ir/mod.rs` `Action` (`:236`, writer `:843`, key written only when declared); `eval/read.rs` (`:380`); `validate/mod.rs` (`:1077`, same references check as `precondition`); `diff/mod.rs` (`:47`, `:1045`); `mutate/mod.rs` docs.
- `ess/domains/protocol.yaml` `canon.protocol.Action` (`:182`); `website/static/schemas/protocol-1.schema.json`; `website/docs/reference/*` (generated).
- Hand-written: `website/docs/concepts/protocols.md`, `website/status.yaml`, `CHANGELOG.md` (inferred).
- Fixtures `fixtures/investigation/check/ordering-guard-broken/` and `ordering-guard-kept/` (inferred); tests `crates/canon-cli/tests/canon_check.rs`, `crates/canon/src/check/tests.rs`.
- Collides with: any unit touching the `protocol/1` model, the IR writer or reader, validate, diff, `ess/domains/protocol.yaml`, the protocol schema or the generated pages.

### Proposal

```yaml
actions:
  report.open:
    precondition: {all: [{claim: store.committed}, {claim: report.opened, is: unknown}]}
    runs_only_when: {claim: store.committed}
```

- Same grammar and reference checks as `precondition` (`undeclared-claim` names `action `a` runs_only_when`).
- check proves it per state: the guard fails when the action is `admissible` or `approval-required` in some state while the predicate is not `true` (`unknown` counts as broken; blocked by a denial does not count as running). The finding names the least-weight state.
- Limit, stated in the docs: check proves the guard per state, not along an order of actions. A guard that holds only through a chain of preconditions must be named in the action's own precondition.
- Diff (decision D8b, default): adding, removing or changing the field is `NO SEMANTIC CHANGE`, like `records_together`.

### Options (D8, taken on default)

- A (default): the protocol field above. Plain `canon check` sees it; about 15 files; collides with both sibling stories.
- B: a second form in `canon-properties/1` (`subject: {action: a}`, `holds: <predicate>`). About 6 files, no `protocol/1` change, but 22 sidecar files for the consumer and the `--properties` flag, and a different format from the one Timo approved.

### Not established

How check reads the guard's value in a state (`Evaluated` holds action and outcome statuses only); whether Canon 0.14.0's `eval/read.rs` refuses an unknown action key; whether `runbook canon check` forwards `--properties` (decides B's cost).

## Acceptance

A property or form for an ordering guard fails on a protocol that breaks it, with the counterexample, and passes on one that keeps it.

Red first: the field and the two fixtures with their `report.txt` in the spec commit; `ess_model_matches` and `canon_check::ordering_guard_broken` are red on it. Existing check, IR and simulate bytes stay unchanged. About 4 commits.

## Source

- **Request:** `bf-runbook`, gap list of 2026-11-22.
