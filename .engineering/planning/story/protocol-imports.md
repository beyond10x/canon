---
format: aep.planning-md/3
id: story:protocol-imports
kind: story
status: draft
title: Compose protocols from pinned imports into one closed canon-ir/1
refs:
- provider: atlas
  reference: adr:0086
- provider: github
  reference: https://github.com/beyond10x/canon/issues/5
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:case-inputs
- depends_on: story:ess-command-surface
scope:
- confidence: inferred
  path: conformance/scenarios/imports.yaml
- confidence: inferred
  path: crates/canon-cli/src/check.rs
- confidence: inferred
  path: crates/canon-cli/src/generate.rs
- confidence: inferred
  path: crates/canon-cli/src/lib.rs
- confidence: inferred
  path: crates/canon-cli/src/main.rs
- confidence: inferred
  path: crates/canon-cli/tests/imports.rs
- confidence: inferred
  path: crates/canon/src/conform/
- confidence: inferred
  path: crates/canon/src/ir/
- confidence: inferred
  path: crates/canon/src/model/
- confidence: inferred
  path: crates/canon/src/validate/
- confidence: inferred
  path: ess/
- confidence: inferred
  path: fixtures/investigation/imports/
- confidence: inferred
  path: website/data/
- confidence: inferred
  path: website/docs/reference/
- confidence: inferred
  path: website/static/schemas/
- confidence: inferred
  path: website/status.yaml
revision: 7
---
## Outcome

A `protocol/1` document can import other protocols and compiles, with them, into one closed
`canon-ir/1` (design § 39.5: "Imports must compile to a closed deterministic IR"). A downstream
profile of a library protocol is then written as an import plus what it adds or tightens, without
copying the library protocol (https://github.com/beyond10x/canon/issues/5, gap 1; Atlas ADR 0086
§ Decision 1, "composition only").

1. **The `imports` section.** Optional, keyed by protocol id like every other declaration section.
   Each entry pins the imported protocol's `revision` and the `sha256:` digest of its compiled
   `canon-ir/1` bytes. There is no range and no "latest": a compile reads exactly the revision the
   source names, or refuses.
2. **Closed compile.** `compile` stays a pure function: of the root document and the imported
   documents passed to it, and of nothing else. The CLI reads only the files it is given with a
   repeatable `--import <path>` on `validate`, `compile` and `check`. Imported declarations keep
   their identifiers and are inlined; the IR names the root protocol's id and revision only, and
   holds no import reference, so `canon-ir/1`'s types do not change. Imports are transitive. A part
   reached through two paths is inlined once. The order of the `--import` flags does not change one
   byte of the IR. `canon diff` and `canon floor` read compiled `canon-ir/1`, which is already
   closed, so they take no `--import`. Every declaration section composes by these rules, the
   `case: inputs` of story:case-inputs included: an imported input is inlined like an imported
   claim, so a profile's own `input` predicates and `applicable_when` guards read the library
   protocol's inputs, and a case under the composed protocol supplies a value for each.
3. **Refinement.** The importing document may redeclare an imported declaration of any section
   (a case input, artifact, evidence kind, claim, obligation, action, outcome or invalidation rule)
   under the same identifier only when the import lists that identifier under `refines`. The redeclaration replaces the imported one. Whether it is a
   tightening is not this story's check: story:protocol-floor's `canon floor`, with the compiled
   import as the floor, answers that.
4. **Refusals**, each exit 1 naming the import or the identifier: an import with no supplied
   document; a supplied document no import names; a revision or digest that differs from the pin
   (printing the expected and the actual digest); an import cycle (printing the cycle); an
   identifier the root and an import both declare without `refines`; an identifier two imports
   declare differently.
5. **`canon generate`** refuses a root with imports, exit 1, naming its first import: the
   scenarios it writes name one fixture, and a composed fixture needs its imports beside it.

## Order

- depends_on story:case-inputs: both change `protocol/1` in `ess/`, `crates/canon/src/model/`,
  `crates/canon/src/validate/` and `crates/canon/src/ir/`, so one waits for the other;
  story:case-inputs goes first because a downstream protocol is blocked on it today.
- depends_on story:ess-command-surface: `--import` changes three commands, and with that story
  landed the change is declared in `ess/` first, in this story's first commit.
- story:protocol-floor and story:case-composition depend on this story.
- Order recorded for https://github.com/beyond10x/canon/issues/5 (2026-10-07):
  story:semantic-diff → story:case-inputs → story:protocol-imports → story:protocol-floor;
  story:ess-command-surface after story:semantic-diff and before story:protocol-imports;
  story:case-composition after story:protocol-imports.
- Other draft stories that change `protocol/1` (story:capability-scope,
  story:evidence-independence, story:evidence-order-predicate, story:revision-ordering) have no edge
  to this one; `aep plan artifact waves` keeps each in a wave of its own by the shared directories.
- Shared files named for the waves: `crates/canon-cli/src/lib.rs` and `crates/canon-cli/src/main.rs`
  with story:canon-mutate (the `Command` enum and its dispatch), and `crates/canon/src/conform/` with
  story:conformance-suite. Neither has an edge to this story; `waves` reports both collisions and
  never puts the pair in one wave.

## Scope

- In: § Outcome 1 to 5; an optional `imports` list of fixture paths in `canon-conformance/1`
  scenarios, under the same confinement as `fixture`.
- Out: checking that a refinement tightens (story:protocol-floor); finding imported documents
  anywhere the caller did not name (a registry, a search path, a network); `protocol.adopt/1`,
  which belongs to the engineering-protocols repository.
- Surfaces: `ess/`, `crates/canon/src/model/`, `crates/canon/src/validate/`, `crates/canon/src/ir/`,
  `crates/canon/src/conform/`, `crates/canon-cli/src/lib.rs`, `crates/canon-cli/src/main.rs`,
  `crates/canon-cli/src/check.rs`, `crates/canon-cli/src/generate.rs`,
  `crates/canon-cli/tests/imports.rs`, `fixtures/investigation/imports/`,
  `conformance/scenarios/imports.yaml`, `website/docs/reference/`, `website/data/` and
  `website/static/schemas/` (regenerated by `task docs-generate`), `website/status.yaml`.

## ESS first

- Specification change, first commit: in `ess/domains/protocol.yaml`, the optional `imports` field
  on `canon.protocol.Protocol`, keyed by `canon.protocol.ProtocolId`, of a new type
  `canon.protocol.Import` (revision, digest, `refines` as a list of declaration identifiers); the
  `--import` argument on `validate`, `compile` and `check` in the command surface
  story:ess-command-surface declares; the fixture directory `fixtures/investigation/imports/`; and
  the scenario file `conformance/scenarios/imports.yaml`. An import is a value inside its protocol,
  so it is a type, not an entity, as the other declaration sections are.
- Red on that commit: `crates/canon/tests/ess_model_matches.rs` fails naming `imports`, because the
  Rust model does not have it; `CANON-IMPORT-001` fails under `canon conform run`, because the
  model's `deny_unknown_fields` refuses `imports`.

## Acceptance

`task check` exits 0 on the story's branch, and its run includes the named test
`imports_compose_into_closed_ir` (in `crates/canon-cli/tests/imports.rs`) passing over
`fixtures/investigation/imports/` (a root protocol, two parts it imports, and a third part both of
them import; one part declares a case input that a root claim reads) with fifteen expectations:

1. `canon compile` of the root with its parts prints the same bytes as `canon compile` of
   `fixtures/investigation/imports/flattened.yaml`, one document holding the same declarations
   inline;
2. the shared part's declarations appear once in that IR;
3. a claim the root lists under `refines` has the root's predicate in that IR;
4. two `canon compile` runs that pass the same imports in opposite order print identical bytes;
5. `canon validate` of the root with its parts exits 0;
6. `canon check` of the root with its parts prints the same report as `canon check` of
   `flattened.yaml`;
7. `canon generate` of the root exits 1 naming its first import;
8. to 13. one per refusal of § Outcome 4: each exits 1 naming its import or identifier, and the
   digest refusal prints both digests;
14. the root claim that reads the imported case input evaluates TRUE and FALSE for the two values
    the input declares;
15. `canon conform run` reports `CANON-IMPORT-001` (`conformance/scenarios/imports.yaml`, a
    compile step with the root's IR and one evaluate step) passed.

`crates/canon/tests/ess_model_matches.rs` passes.

## Source

https://github.com/beyond10x/canon/issues/5 (gap 1); `docs/design/canon-protocol-calculus-design.md`
§ 39.5; Atlas ADR 0086 § Decision 1 and § Open; Atlas ADR 0080 (spec first, then red).
