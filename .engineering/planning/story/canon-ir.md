---
format: aep.planning-md/3
id: story:canon-ir
kind: story
status: implemented
title: Define the canon-ir/1 normalized model
refs:
- provider: taskboard
  reference: C-002
relations:
- decomposes: epic:canon-kernel
- depends_on: story:protocol-source-model
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon-cli/
- confidence: cited
  path: crates/canon/src/ir/
- confidence: cited
  path: fixtures/investigation/canon-ir.yaml
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:50Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-04T01:17:40Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-10-04T01:59:16Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":5,"verification":1}}}
---
## Outcome

`canon compile` turns a valid `protocol/1` document into `canon-ir/1`, the normalized form every
evaluator consumes: canonical ordering, explicit defaults, no authoring sugar, stable identifiers,
fully resolved references, deterministic serialization suitable for hashing, no filesystem paths
and no environment-dependent interpretation (design § 30). The IR carries the protocol id and
revision so an evaluation can record which protocol applied (design § 37). Compilation is a pure
function of the validated source. Because the acceptance compiles two equivalent documents from
different input paths and different working directories, identical bytes also show that the IR
embeds neither the input path nor the working directory.

## Scope

- In: module `crates/canon/src/ir/`; the `compile` subcommand in `crates/canon-cli`; the variant
  `fixtures/investigation/canon-ir.yaml`, an equivalent copy of the base fixture with declarations
  reordered and optional fields written out at their defaults (operator decision 2026-10-04: every
  story after protocol-source-model adds its own variant and never edits the base).
- Surfaces: `crates/canon/src/ir/`, `crates/canon-cli/`, `fixtures/investigation/canon-ir.yaml`.

## Acceptance

The named test `canon_ir_canonical_form` (in `crates/canon-cli/tests/`) passes: `canon compile`
run from the repository root on `fixtures/investigation/protocol.yaml`, and run from
`fixtures/investigation/` on `canon-ir.yaml` (the reordered, defaulted copy), prints byte-identical
`canon-ir/1`.

## Source

TASKBOARD C-002 (build pack, now Atlas `docs/design/governed-autonomy/TASKBOARD.md`);
`docs/design/canon-protocol-calculus-design.md` § 12, § 30, § 37, § 41 item 3.
