---
format: aep.planning-md/3
id: story:protocol-source-model
kind: story
status: proposed
title: Define the protocol/1 minimal source model
refs:
- provider: taskboard
  reference: C-001
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/canon-cli/
- confidence: cited
  path: crates/canon/Cargo.toml
- confidence: cited
  path: crates/canon/src/lib.rs
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/validate/
- confidence: cited
  path: fixtures/investigation/invalid/
- confidence: cited
  path: fixtures/investigation/protocol.yaml
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:51Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

Canon parses and validates a `protocol/1` source document. The minimal source model declares a
protocol id and revision, artifacts, evidence kinds, claims with their predicates, obligations (id
and description; the discharge predicate is added by story:obligations), actions (precondition,
capability requirement, effect class, evidence kinds they may produce) and outcomes with their
requirements, each with an optional human description. Predicates are a small, total,
deterministic expression language: `all`, `any`, `not`, an evidence match by kind and result, and a
test on a claim value (design § 39.2). Validation resolves every reference and reports errors in a
stable order. `canon validate --path` (clap derive) is a thin shell over the library. The model
carries no engineering vocabulary.

This story owns the typed identifiers every later story uses: newtypes for protocol, case,
artifact, revision, claim, evidence, obligation, action and outcome ids, in `model`. They replace
the bootstrap `ProtocolId`/`CaseId`/`ClaimId`/`ActionId` and close the epic's "Typed ids" promise.

## Operator decisions (2026-10-04)

- Crate layout: one library crate `crates/canon` (package `b10x-canon`) with modules `model`,
  `validate`, `ir`, `eval`, `explain`, `diff`, `conform`, and one CLI crate `crates/canon-cli`
  (binary `canon`, clap derive). The design § 28 split into more crates is deferred until the
  kernel stabilizes. This story writes `crates/canon/src/lib.rs` declaring all seven modules, the
  five it does not build as empty modules, so no later story edits `lib.rs`.
- Fixtures: this story creates the base investigation fixture `fixtures/investigation/protocol.yaml`.
  Every later story adds its own variant file `fixtures/investigation/<story-name>.yaml` and never
  edits the base.

## Scope

- In: design § 41 capabilities 1–2; typed identifiers; the base fixture, which is the design § 12
  investigation example extended with an artifact for the explanation under investigation and
  without the `inconclusive` outcome, whose `decision:` requirement the model does not have (see
  `decision-blocker:outcome-decision-source`); invalid variants under `fixtures/investigation/invalid/`.
- Out: named states and transitions (design § 39.1 says test before baking scalar state, and § 41
  does not list them); protocol imports (§ 39.5); recovery rules; independence requirements;
  evidence freshness (C-008); obligation discharge predicates (C-005); semantics of every construct
  (C-003 to C-007).
- The bootstrap types in `crates/canon/src/lib.rs` are replaced. Their test uses engineering
  identifiers (`merge`, `tests.pass`, `CHG-1`) that the kernel must not keep.
- Surfaces: `crates/canon/Cargo.toml`, `crates/canon/src/lib.rs`, `crates/canon/src/model/`,
  `crates/canon/src/validate/`, `crates/canon-cli/`, `Cargo.lock`,
  `fixtures/investigation/protocol.yaml`, `fixtures/investigation/invalid/`.

## Domain relations

- Protocol → Case: one protocol revision governs many cases; a case is governed by exactly one
  immutable protocol revision; the protocol is static input and exists before any case. Stated in
  design § 4.1 and § 36. Not an ess/1 document: Canon opts out of ESS for language semantics
  (AGENTS.md § ESS, Atlas ADR 0067), so the conformance suite is what decides it.

## Acceptance

`canon validate --path` accepts `fixtures/investigation/protocol.yaml` and rejects each invalid
variant (a reference to an undeclared claim, a duplicate identifier, an action producing an
undeclared evidence kind) with an error naming the offending identifier, byte-identical across
repeated runs.

## Source

TASKBOARD C-001 (build pack, now Atlas `docs/design/governed-autonomy/TASKBOARD.md`);
`docs/design/canon-protocol-calculus-design.md` § 4, § 12, § 28, § 39, § 41;
`docs/contracts/protocol-core.md`; ROADMAP Phase 1 "typed identifiers".
