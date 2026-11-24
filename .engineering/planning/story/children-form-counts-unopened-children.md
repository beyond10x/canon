---
format: aep.planning-md/3
id: story:children-form-counts-unopened-children
kind: story
status: implemented
title: A children form can wait for all children of a set, including those not yet opened
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: conformance/requirements.yaml
- confidence: inferred
  path: conformance/scenarios/children-count.yaml
- confidence: inferred
  path: crates/canon/src/check/mod.rs
- confidence: inferred
  path: crates/canon/src/check/space.rs
- confidence: inferred
  path: crates/canon/src/diff/mod.rs
- confidence: inferred
  path: crates/canon/src/eval/actions.rs
- confidence: inferred
  path: crates/canon/src/eval/composition.rs
- confidence: inferred
  path: crates/canon/src/eval/read.rs
- confidence: inferred
  path: crates/canon/src/explain/mod.rs
- confidence: inferred
  path: crates/canon/src/ir/composition.rs
- confidence: inferred
  path: crates/canon/src/ir/mod.rs
- confidence: inferred
  path: crates/canon/src/model/predicate.rs
- confidence: inferred
  path: crates/canon/src/simulate/mod.rs
- confidence: inferred
  path: crates/canon/src/simulate/recovery.rs
- confidence: inferred
  path: crates/canon/src/validate/mod.rs
- confidence: inferred
  path: ess/domains/protocol.yaml
- confidence: inferred
  path: fixtures/investigation/simulate/children-count
- confidence: inferred
  path: website/docs/reference
- confidence: inferred
  path: website/static/schemas/protocol-1.schema.json
- confidence: inferred
  path: website/status.yaml
revision: 26
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T03:55:41Z", actor: "human:timo", revision: 2, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w13"}
- {from: "proposed", to: "active", at: "2026-11-23T03:55:41Z", actor: "human:timo", revision: 3, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w13"}
- {from: "active", to: "implemented", at: "2026-11-23T23:09:06Z", actor: "human:timo", revision: 26, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w14"}
---
## Outcome

A claim over the children of a case can say that every child of a named set ended, or that N children ended, without listing every outcome combination and without becoming true while a child is still unopened.

## Why

- Gap 2: {children: {in: [...]}} is true once every opened child ended; the consumer chained its openers and listed four reader outcomes (64 values, 65 history values per subject).
- Reported by the session `bf-runbook` (dev-operator), 2026-11-22, from `canon-gaps.md` in the dev-operator spec directory (22 composed protocols, `runbook` 0.5.0 linking Canon 0.11.0). Canon has not reproduced it; the file quotes the exact error texts.

## Scope

- **In:** the outcome above, as a new optional key `count: N` on the `children` form of `protocol/1` (decision D10, option A below).
- **Out:** any change to the bytes of existing `protocol/1` inputs. The format change itself was approved by Timo (his answer to `bf-runbook` at 04:52:04 CET on 2026-11-23, read by `host-operator`).
- **Derived** 2026-11-23 by `story-scoper` (main `6dc461644`, Canon 0.14.0). Medium confidence: the counter in check and simulate is a design, not traced code.

### Surfaces

- `crates/canon/src/model/predicate.rs:340-367` (`ChildOutcomes`), `:414`; `eval/composition.rs:40-138`; `ir/mod.rs:454,676,1118-1132`; `eval/read.rs:551`; `validate/mod.rs:308-314,441,630,1495` (new `zero-child-count`).
- `crates/canon/src/check/space.rs` (`walk_evidence` `:2800`, children dimensions `:237/:2423/:2466/:2634/:2676`) and `check/mod.rs:236-262` docs; `simulate/mod.rs:1210-1370` (`walk`, `Layout::marking`/`opening`, child-outcome step), `:2232`, `simulate/recovery.rs:149`.
- `ess/domains/protocol.yaml:973-1030` (`canon.protocol.ChildOutcomes`); schema; `website/docs/reference/*` (generated); `CHANGELOG.md`; `website/status.yaml`.
- Inferred: `conformance/requirements.yaml` (new CANON-COMPOSE-004), `conformance/scenarios/children-count.yaml`, `fixtures/investigation/simulate/children-count/` (five readers on one subject), `explain/mod.rs:542`, `eval/actions.rs:296`, `diff/mod.rs:1340`, `ir/composition.rs:375`.
- Collides with: any unit on `check/space.rs`, `simulate/mod.rs`, `validate/mod.rs`, `model/predicate.rs`, `ess/domains/protocol.yaml` or the schema.

### Proposal

```yaml
readers.ended:
  true_when: {children: {in: [fresh, quiet, partial, unreadable], subject: sources, count: 5}}
```

- `count` is an integer of 1 or more; 0 is refused as `zero-child-count`. It combines with `subject` and `revision`.
- Evaluate over the records the form reads (one per child): `false` when a child ended outside `in` (as today); otherwise `unknown` when fewer than `count` records are read or a child is `open`; `true` when at least `count` children ended with a listed outcome. A child not yet opened has no record, so the form stays `unknown` and never becomes true early.
- canon-ir/1: `"count": N` written only when present, after `in`; an existing protocol's IR is byte-identical.
- check: every `case_outcome` dimension a counting form reads gets a counter of records read, capped at N; state size grows by a factor of N+1 per counted dimension. simulate: each opening step raises the counter; a revision zeroes the current-only counter. This also closes the gap-8 probe `nobody-after-incomplete-read` for five readers on one subject.
- Doc duty: an opener whose guard stays open can open two children and fake the count; the author guards each opener with its `*_opened` record.

### Options (D10, taken on default)

- A (default): `count: N` on the form. One format change, about 3 commits in `space.rs` and `simulate/mod.rs`.
- B: `opened_by: [action, ...]` on the form plus an `opened_by` field on `case_outcome` records. Two format changes (`protocol/1` and `canon-evidence/1`), a new runner duty, about 4^5 = 1024 values in check against about 4x6 for A.
- Sub-decision (default: not in this story): making `in` optional.
- Acceptance wording: under Canon's rule that `unknown` is not `false`, the form is `unknown` while a child is unopened; the acceptance below says "not true".

### Not established

How the counter interacts with profiles, history and `revision: current` in `check/space.rs` (2953 lines, traced only to `walk_evidence`); where `walk`'s child-outcome step raises the counter; whether `mutate` and `generate` need changes; the state-space cost on the consumer's `refresh-read` (gap 4 may return).

## Acceptance

A protocol with five parallel readers expresses the wait in one claim, within the state bound, and the claim is not true while any reader is unopened.

Red first: the field in `ess/`, CANON-COMPOSE-004 with its scenario and the five-reader fixture; `ess_model_matches` and `canon conform run` fail on it. Existing check, IR and decision bytes stay unchanged. About 5 commits; time `canon check` and `canon simulate` on `refresh-read` before and after.

## Source

- **Request:** `bf-runbook`, gap list of 2026-11-22.
