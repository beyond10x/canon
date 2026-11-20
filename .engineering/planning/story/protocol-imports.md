---
format: aep.planning-md/3
id: story:protocol-imports
kind: story
status: active
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
- depends_on: story:conformance-suite
- depends_on: story:case-composition
scope:
- confidence: cited
  path: conformance/requirements.yaml
- confidence: cited
  path: conformance/scenarios/imports.yaml
- confidence: cited
  path: crates/canon-cli/src/check.rs
- confidence: cited
  path: crates/canon-cli/src/conform.rs
- confidence: cited
  path: crates/canon-cli/src/generate.rs
- confidence: cited
  path: crates/canon-cli/src/lib.rs
- confidence: cited
  path: crates/canon-cli/src/main.rs
- confidence: cited
  path: crates/canon-cli/tests/imports.rs
- confidence: cited
  path: crates/canon/src/conform/mod.rs
- confidence: inferred
  path: crates/canon/src/ir/imports.rs
- confidence: cited
  path: crates/canon/src/ir/mod.rs
- confidence: cited
  path: crates/canon/src/model/mod.rs
- confidence: cited
  path: crates/canon/src/validate/mod.rs
- confidence: inferred
  path: ess/domains/cli.yaml
- confidence: cited
  path: ess/domains/protocol.yaml
- confidence: cited
  path: fixtures/investigation/imports/
revision: 47
transitions:
- {from: "draft", to: "proposed", at: "2026-11-20T20:58:27Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":4}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w1"}
- {from: "proposed", to: "active", at: "2026-11-20T20:58:27Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"review_outcome":4}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w1"}
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
   byte of the IR. `canon diff`, `canon evaluate` and `canon floor` read compiled `canon-ir/1`, which is
   already closed, so they take no `--import`. `canon conform run` compiles fixture sources in-process
   (`crates/canon/src/conform/mod.rs:510-525`); it takes no flag either, because each scenario
   names its own imports (§ Scope). Every declaration section composes by these rules, the
   `case: inputs` of story:case-inputs included: an imported input is inlined like an imported
   claim, so a profile's own `input` predicates and `applicable_when` guards read the library
   protocol's inputs, and a case under the composed protocol supplies a value for each.
3. **Refinement.** The importing document may redeclare an imported declaration of any section
   (a case input, artifact, evidence kind, claim, obligation, action, outcome, invalidation rule or
   independence dimension: the 9 declaration sections of `Protocol`,
   `crates/canon/src/model/mod.rs:70-97`) under the same identifier only when the import lists that identifier under `refines`. The redeclaration replaces the imported one. Whether it is a
   tightening is not this story's check: story:protocol-floor's `canon floor`, with the compiled
   import as the floor, answers that.
4. **Refusals**, each exit 1 naming the import or the identifier: an import with no supplied
   document; a supplied document no import names; a revision or digest that differs from the pin
   (printing the expected and the actual digest); an import cycle (printing the cycle); an
   identifier the root and an import both declare without `refines`; an identifier two imports
   declare differently (two identical declarations are one declaration); a `refines` entry naming
   an identifier its import does not declare. The cycle check runs before the pin check: pinned
   digests make a real cycle impossible to build, since the IR of A needs the IR of B and the
   reverse, so a cycle is only ever met with a wrong pin.
5. **`canon generate`** refuses a root with imports, exit 1, naming its first import: the
   scenarios it writes name one fixture, and a composed fixture needs its imports beside it.

## Order

- depends_on story:case-inputs (implemented): satisfied. The `case` section of story:case-inputs
  (`Protocol.case`, `crates/canon/src/model/mod.rs:90-93`) is one of the sections imports compose.
- depends_on story:case-composition (implemented 2026-10-29): this story composes its `case_open`
  like any other declaration; § Case composition says how.
- depends_on story:conformance-suite (implemented): `conformance/requirements.yaml` exists, and
  `CANON-IMPORT-001` is added to it.
- depends_on story:ess-command-surface: `--import` changes three commands, and with that story
  landed the change is declared in `ess/domains/cli.yaml` first, in this story's first commit.
- story:protocol-floor depends on this story.
- Order recorded for https://github.com/beyond10x/canon/issues/5 (2026-10-07):
  story:semantic-diff → story:case-inputs → story:protocol-imports → story:protocol-floor;
  story:ess-command-surface after story:semantic-diff and before story:protocol-imports.
- story:capability-scope and story:revision-ordering share `ess/domains/protocol.yaml`,
  `crates/canon/src/model/mod.rs`, `validate/mod.rs`, `ir/mod.rs` and
  `conformance/requirements.yaml` with this story and have no edge to it; the three run in three
  different waves.
- story:canon-mutate and story:protocol-floor share `crates/canon-cli/src/lib.rs`, `main.rs` and
  `ess/domains/cli.yaml` with this story; the three run in three different waves.

## Scope

Rescoped 2026-11-20 by `story-scoper` against `8f1f452`, written by the wave coordinator. Every
line is **cited** (read from the story or the tree) or **inferred** (a reading that could be
wrong).

- **In:** § Outcome 1 to 5; an optional `imports` list of fixture paths in `canon-conformance/1`
  scenarios, under the same confinement as `fixture` (both checks: the text, then the resolved
  path); validation of the composed document, so an imported action that opens a case under the
  root's id is refused as `self-opened-case`. — cited
- **Out:** checking that a refinement tightens (story:protocol-floor); finding imported documents
  anywhere the caller did not name (a registry, a search path, a network); `protocol.adopt/1`,
  which belongs to the engineering-protocols repository; `--import` on `diff` and `evaluate`, which
  read compiled IR (`crates/canon-cli/src/lib.rs:151-158`, `:73-75`); `--import` on `conform run`,
  whose scenarios name their own imports; any change to `ir::check_openings`
  (`crates/canon/src/ir/composition.rs:58`). — cited
- **Primary surface:** `crates/canon/src/model/mod.rs` and `crates/canon/src/ir/` — cited
- **Files:** `ess/domains/protocol.yaml` (`imports` on `canon.protocol.Protocol`, the new type
  `canon.protocol.Import`) — cited
- **Files:** `ess/domains/cli.yaml` (the `import` input on `validate`, `compile` and `check`; the
  file is created by story:ess-command-surface) — inferred
- **Files:** `crates/canon/src/model/mod.rs:70` (`Protocol`, 9 declaration sections today; add
  `Import`) — cited
- **Files:** `crates/canon/src/ir/mod.rs:185` (`compile`; its module documentation feeds
  `canon-ir.md`) — cited
- **Files:** `crates/canon/src/ir/imports.rs` (new: compose the root and its imports into one
  `Protocol`, and the refusals of § Outcome 4) — inferred
- **Files:** `crates/canon/src/validate/mod.rs:473` (per-section validation; the `imports`
  section's ids, digest form and `refines`) — cited
- **Files:** `crates/canon/src/conform/mod.rs` (`RawScenario` :268, `compiled_fixture` :510, the
  fixture reader in `run_scenarios` :932) — cited
- **Files:** `crates/canon-cli/src/lib.rs` (`Validate` :59, `Compile` :65 and `Check` :120 gain
  `--import`; the `Generate` help) — cited
- **Files:** `crates/canon-cli/src/main.rs` (dispatch, `read_protocol` :87, `compile_command`
  :109, `validate_command` :123) — cited
- **Files:** `crates/canon-cli/src/check.rs:15-25`, `crates/canon-cli/src/generate.rs:58-65`,
  `crates/canon-cli/src/conform.rs:76` (`read_fixture`: a scenario's imports are read with the same
  resolved-path check) — cited
- **Files:** `crates/canon-cli/tests/imports.rs` (new), `fixtures/investigation/imports/` (new,
  owned whole), `conformance/scenarios/imports.yaml` (new), `conformance/requirements.yaml`
  (`CANON-IMPORT-001`, beside `CANON-COMPOSE-001` at :115) — cited
- **Coordinator's at integration, not in the typed scope:** the story's entry in
  `website/status.yaml`, and the regenerated `website/docs/reference/protocol.md`, `cli.md`,
  `website/static/schemas/protocol-1.schema.json`, `website/data/status.json`,
  `website/data/canon.terminal.json` (it records `canon conform run`, which gains a line), and
  `canon-ir.md`, `validation.md` and `conformance.md` if their module documentation changes.
- **Confidence:** medium. Every file was read. Where composition lives (a new `ir/imports.rs`, or
  `model/`) is a design choice.
- **Would collide with:** any unit on the `protocol/1` model, `ir/mod.rs`, `validate/mod.rs` or
  `conformance/requirements.yaml` (story:capability-scope, story:revision-ordering), on the clap
  `Command` enum, its dispatch or `ess/domains/cli.yaml` (story:protocol-floor,
  story:canon-mutate), or on the conformance scenario parser.
- **Safety fact:** `ir::compile(&Protocol)` is called from 92 files. A separate compose step that
  leaves `compile`'s signature alone keeps all of them untouched. `imports` is optional and absent
  from the IR, so the IR of every existing protocol stays byte-identical. Unproven.

## Case composition

story:case-composition landed on 2026-10-29, after this story was drafted. Read by the scoper on
2026-11-20:

- `case_open` is a field of `Action` (`crates/canon/src/model/mod.rs:218`), not a section.
  Refining an action replaces it whole, `case_open` included.
- `children` is a predicate form that reads the evidence kind `case_outcome`
  (`crates/canon/src/model/predicate.rs:251`). Each part that uses it declares `case_outcome` to
  compile on its own, so two such parts meet the rule of § Outcome 4 for an identifier two imports
  declare: identical declarations are one declaration; different ones are refused.
- Inlining turns "a part opens a case under the root's id" into "the root opens itself", which
  validation of the composed document refuses as `self-opened-case`. Expectation 18 holds it.
- `check_openings` is decision D206 (`ir/composition.rs:58`). It reads compiled IR, which holds no
  import reference. Import edges and opening edges are two graphs; neither cycle check uses the
  other's edges.
- `ordered-child-outcomes` (`crates/canon/src/check/mod.rs:314`) can first appear after
  composition, when one part uses `children` and another an order form. `canon check` reports it;
  `canon compile` does not.

## ESS first

- Specification change, first commit: in `ess/domains/protocol.yaml`, the optional `imports` field
  on `canon.protocol.Protocol`, keyed by `canon.protocol.ProtocolId`, of a new type
  `canon.protocol.Import` (revision, digest, `refines` as a list of declaration identifiers); the
  `--import` argument on `validate`, `compile` and `check` in the command surface
  story:ess-command-surface declares; the fixture directory `fixtures/investigation/imports/`; and
  the scenario file `conformance/scenarios/imports.yaml`. An import is a value inside its protocol,
  so it is a type, not an entity, as the other declaration sections are.
- Red on that commit: `crates/canon/tests/ess_model_matches.rs::specification_and_model_are_equal`
  fails naming `imports` and `Import`, because the Rust model does not have them; and
  `canon conform run --requirements conformance/requirements.yaml` (`Taskfile.yml:14`) exits 1 on
  `CANON-IMPORT-001`, because the scenario parser's own `deny_unknown_fields`
  (`crates/canon/src/conform/mod.rs:268`) refuses the scenario's `imports` key.
  `every_canon_subcommand_is_declared_in_ess` (story:ess-command-surface) fails naming the
  `import` input, declared in `ess/` and absent from clap. The named test
  `imports_compose_into_closed_ir` is code and arrives with the implementation.

## Acceptance

`task check` exits 0 on the wave's integration branch with this story merged (the wave coordinator
regenerates the documentation there), and its run includes the named test
`imports_compose_into_closed_ir` (in `crates/canon-cli/tests/imports.rs`) passing over
`fixtures/investigation/imports/` (a root protocol, two parts it imports, and a third part both of
them import; one part declares a case input that a root claim reads) with eighteen expectations:

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
    compile step with the root's IR and one evaluate step) passed;
16. a supplied document whose revision differs from the pin exits 1 naming the import (expectations
    8 to 13 hold the digest trigger of that refusal; this holds the revision trigger);
17. a `refines` entry naming an identifier its import does not declare exits 1 naming the
    identifier;
18. a part whose action opens a case under the root's id exits 1 with `self-opened-case`.

Expectations 16 to 18 were added by the rescope of 2026-11-20, from gaps the scoper counted
between § Outcome and the fifteen expectations. Three gaps stay open and are named here: the
refinement of a section other than claims has no expectation; "printing the cycle" is not
asserted; the confinement of a scenario's `imports` paths has no expectation.

`crates/canon/tests/ess_model_matches.rs` passes.

## Source

https://github.com/beyond10x/canon/issues/5 (gap 1); `docs/design/canon-protocol-calculus-design.md`
§ 39.5; Atlas ADR 0086 § Decision 1 and § Open; Atlas ADR 0080 (spec first, then red).
