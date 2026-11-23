---
format: aep.planning-md/3
id: story:check-across-protocol-files
kind: story
status: implemented
title: canon check resolves opened protocol ids, named child outcomes and the producers of supplied evidence
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
  path: crates/canon-cli/src/main.rs
- confidence: inferred
  path: crates/canon-cli/tests/check_across_protocol_files_cli.rs
- confidence: inferred
  path: crates/canon/src/ir/composition.rs
- confidence: inferred
  path: crates/canon/src/ir/mod.rs
- confidence: inferred
  path: crates/canon/src/model/mod.rs
- confidence: inferred
  path: crates/canon/src/model/predicate.rs
- confidence: inferred
  path: crates/canon/tests/check_across_protocol_files.rs
- confidence: inferred
  path: ess/domains/cli.yaml
- confidence: inferred
  path: ess/domains/protocol.yaml
- confidence: inferred
  path: fixtures/check-across-protocol-files
- confidence: inferred
  path: website/docs/reference/cli.md
- confidence: inferred
  path: website/docs/reference/protocol.md
- confidence: inferred
  path: website/status.yaml
revision: 21
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T03:55:37Z", actor: "human:timo", revision: 19, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w13"}
- {from: "proposed", to: "active", at: "2026-11-23T03:55:37Z", actor: "human:timo", revision: 20, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w13"}
- {from: "active", to: "implemented", at: "2026-11-23T07:14:14Z", actor: "human:timo", revision: 21, decided_on: {"recorded":{"test_result":1}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-23-w13"}
---
## Outcome

A check over a set of protocol files fails when a parent opens a protocol that does not exist, names a child outcome the child never declares, or reads a supplied kind that no action of its supplier produces.

## Why

- Gap 1 of `bf-runbook` (dev-operator, 2026-11-22, `canon-gaps.md`): `case_open.protocol` and `supplied_by` are named and never resolved; `canon check` passes such a parent in silence. The consumer wrote `check.sh` over the IR to do it, for 32 files.
- No `protocol/1` change is needed: every field the check reads exists (`case_open.protocol`, `.subject`, `supplied_by`, `children.in`, `may_produce`, outcome keys). The only check over a set today is `ir::check_openings`, which has no caller outside tests.

## Scope

Derived 2026-11-23 by `story-scoper`; entries marked inferred are readings, not facts.

- **In:** given a set of `protocol/1` files, a finding with a stable code when (1) an action's `case_open.protocol` is not in the set, (2) a protocol names a `case_outcome` result about subject `s` (in `children.in`, an evidence match or an order form) that no protocol it opens on `s` declares (`open` excepted), (3) a kind's `supplied_by` is not in the set, (4) the supplier has no action whose `may_produce` lists that kind id. The existing cycle refusal `case-open-cycle` applies to the set.
- **In (decision D1, default B):** a new command `canon check-set --path <file>`, `--path` repeatable; exit 0 clean, 1 on a finding, 2 on an unreadable file.
- **In (decision D2, default A):** a supplied kind matches a supplier's `may_produce` entry of the same id.
- **In:** the "named and never resolved" doc comments of `EvidenceKind.supplied_by` and `CaseOpening` name the new check; generated reference and schema description regenerated.
- **Out:** any change to bytes of existing `protocol/1` inputs; a new field or form (D2 option B is a format change and waits for Timo); a change to `canon check` or `canon simulate` on a single file; set members with `imports:` (refused with a named error); the consumer's list of child outcomes left to a catch-all.
- **Surfaces:** `crates/canon/src/ir/composition.rs`, `crates/canon/src/ir/mod.rs`, `crates/canon/src/model/mod.rs`, `crates/canon/src/model/predicate.rs`, `crates/canon-cli/src/lib.rs`, `crates/canon-cli/src/main.rs`, `crates/canon-cli/src/check_set.rs`, `ess/domains/cli.yaml`, `ess/domains/protocol.yaml`, `website/status.yaml`, `fixtures/check-across-protocol-files/`, new test files in both `tests/main.rs` lists.
- **Shared with the other wave 13 stories (merge, not order):** `crates/canon-cli/src/lib.rs`, `ess/domains/cli.yaml`, `website/docs/reference/cli.md`, the two `tests/main.rs` lists, `CHANGELOG.md`.

## Acceptance

1. In `fixtures/check-across-protocol-files/` each defect set (opens a missing protocol; names a child outcome the child lacks in a `children.in` and in an evidence match; missing supplier; supplier with no action producing the kind) exits 1 with one line of a stable code naming the protocol, the action or kind and the missing id.
2. A clean set exits 0; naming `open` is never a finding; an outcome named about subject `s` but declared only by the child opened on `t` is a finding.
3. A set whose openings form a cycle prints `case-open-cycle`.
4. `canon check` and `canon simulate` bytes are identical on every fixture before and after; `task ess-gate` and `task docs-check` pass.
5. Run by hand over the 32 consumer files: exit 0, no finding, matching the consumer's `check.txt`.
6. Each new test under 5 s; no test over 60 s; the test step within 120 s.

## Source

- **Request:** `bf-runbook` gap list of 2026-11-22; accepted by host-operator on Timo's delegation, 2026-11-23.
