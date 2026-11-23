---
format: aep.planning-md/3
id: story:supplied-evidence-required-at-open
kind: story
status: active
title: A protocol can require that its opener supplies a record when it opens the case
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: inferred
  path: crates/canon-cli/src/check_set.rs
- confidence: inferred
  path: crates/canon-cli/src/lib.rs
- confidence: inferred
  path: crates/canon-cli/tests/canon_simulate.rs
- confidence: inferred
  path: crates/canon-cli/tests/check_across_protocol_files_cli.rs
- confidence: inferred
  path: crates/canon/src/check/space.rs
- confidence: inferred
  path: crates/canon/src/diff/mod.rs
- confidence: inferred
  path: crates/canon/src/eval/mod.rs
- confidence: inferred
  path: crates/canon/src/eval/read.rs
- confidence: inferred
  path: crates/canon/src/ir/composition.rs
- confidence: inferred
  path: crates/canon/src/ir/mod.rs
- confidence: inferred
  path: crates/canon/src/model/mod.rs
- confidence: inferred
  path: crates/canon/src/model/write.rs
- confidence: inferred
  path: crates/canon/src/mutate/mod.rs
- confidence: inferred
  path: crates/canon/src/simulate/mod.rs
- confidence: inferred
  path: crates/canon/src/validate/mod.rs
- confidence: inferred
  path: ess/domains/cli.yaml
- confidence: inferred
  path: ess/domains/protocol.yaml
- confidence: inferred
  path: fixtures/check-across-protocol-files
- confidence: inferred
  path: fixtures/investigation/invalid
- confidence: inferred
  path: fixtures/investigation/simulate
- confidence: inferred
  path: website/docs/reference
- confidence: inferred
  path: website/static/schemas/protocol-1.schema.json
revision: 27
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T03:55:42Z", actor: "human:timo", revision: 2, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w13"}
- {from: "proposed", to: "active", at: "2026-11-23T03:55:42Z", actor: "human:timo", revision: 3, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w13"}
---
## Outcome

A protocol declares that a supplied evidence kind must be present at case open, and the check refuses an opener that cannot supply it.

## Why

- Gap 6: a case whose supplied record never arrives has no admissible action and waits for good; simulate counts it as waiting on supplied evidence or expiry, not as a finding.
- Reported by the session `bf-runbook` (dev-operator), 2026-11-22, from `canon-gaps.md` in the dev-operator spec directory (22 composed protocols, `runbook` 0.5.0 linking Canon 0.11.0). Canon has not reproduced it; the file quotes the exact error texts.

## Scope

- **In:** the outcome above, as a new optional evidence-kind field `required_at_open: true` of `protocol/1` (decision D9, option A below) and a new `canon check-set` finding `unsupplied-at-open`.
- **Out:** any change to the bytes of existing `protocol/1` inputs. The format change itself was approved by Timo (his answer to `bf-runbook` at 04:52:04 CET on 2026-11-23, read by `host-operator`).
- **Derived** 2026-11-23 by `story-scoper` (main `6dc461644`, Canon 0.14.0). Entries marked inferred are readings.

### Surfaces

- `crates/canon/src/ir/composition.rs` (`check_set`, `SetFinding`): the opener proof.
- `crates/canon/src/model/mod.rs:191-212` `EvidenceKind`; `ir/mod.rs:209-217,335-339,740-750` (key written only when true, like `supplied_by`); `eval/read.rs:228-232`; `model/write.rs:356-380`; `diff/mod.rs:20-26,879-893`; `validate/mod.rs:925-995` (new problem `at-open-without-supplier`).
- `crates/canon/src/check/space.rs:2727-2753` (`initial_count`, `initial_states`, shared by simulate); `simulate/mod.rs:123-125,191-206` docs; `crates/canon-cli/src/lib.rs:345-362` and `check_set.rs` docs.
- `ess/domains/protocol.yaml:122-141`, `ess/domains/cli.yaml:555-600`; schema and `website/docs/reference/*` (generated).
- Inferred: a runner helper `eval::missing_at_open(ir, case, records)` in `eval/mod.rs` or a new `eval/opening.rs`; `mutate/mod.rs` docs; `CHANGELOG.md`; fixtures `fixtures/check-across-protocol-files/{unsupplied-at-open,supplied-at-open}/`, `fixtures/investigation/simulate/at-open/`, `fixtures/investigation/invalid/at-open-without-supplier.yaml`.
- Tests: `crates/canon-cli/tests/check_across_protocol_files_cli.rs:36` (table), `canon_simulate.rs:79-85` (table), unit tests in `ir/composition.rs` and `ir/mod.rs`.
- Collides with: `ir/composition.rs`, `check/space.rs`, `simulate/mod.rs`, `model/mod.rs`, `ir/mod.rs`, `eval/read.rs`, `ess/domains/{protocol,cli}.yaml`, the schema and the generated pages.

### Proposal

```yaml
evidence_kinds:
  jira_tickets_read: {supplied_by: read.jira.tickets, max_age: 12h, required_at_open: true}
```

- `required_at_open` needs `supplied_by` on the same kind, else `at-open-without-supplier`. `false` compiles to no key.
- Runner at case open: copies a current record of the kind into the child, or refuses to open it; Canon offers `eval::missing_at_open`. `canon evaluate` bytes do not change.
- `check-set` finding `unsupplied-at-open`: for every action of a protocol P in the set whose `case_open.protocol` is C and every kind k that C requires at open, P must declare k and the action's `precondition`, when true, must imply a current record of k (an `evidence` match implies it; `all` implies it if any member does; `any` if every member does; `claim ... is: true` follows `true_when`; an order form over k implies it; `not`, `input` and `children` do not). A protocol nothing in the set opens gets no finding. The summary line keeps its counts.
- simulate: each initial state holds one current record of every required kind (`initial_count` multiplies by that); the supplied step stays. `canon check` reads nothing from the field in this story.
- diff: adding the field is `TIGHTENING`, removing it `RELAXATION`.

### Options (D9, taken on default)

- A (default): the field above, opener proof through the opening action's precondition (about 80 lines in `composition.rs`).
- B: the same field plus `case_open.supplies: [k]` on the opener; check-set only checks the list covers C's required kinds. A second format field and a weaker proof.

### Not established

Whether `evidence is: false` needs a record (decides whether `claim is: false` counts in the walk); whether `eval/read.rs` refuses unknown canon-ir/1 keys; how a copied record's subject matches the child's artifact ids (gap 9); whether any other path reaches the `{}` state of gap 6.

## Acceptance

A fixture whose opener never supplies the record is a finding; one that does supply it passes.

Red first: the two check-set fixture rows with their `report.txt`, and the invalid fixture refused as `at-open-without-supplier`. Existing IR, check, check-set and simulate bytes stay unchanged. About 4 commits.

## Source

- **Request:** `bf-runbook`, gap list of 2026-11-22.
