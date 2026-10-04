---
format: aep.planning-md/3
id: story:ess-hard-gate
kind: story
status: draft
title: Specify the protocol/1 data model in ESS under the ADR 0076 hard gate
refs:
- provider: atlas
  reference: adr:0076
relations:
- decomposes: epic:canon-kernel
- depends_on: story:protocol-source-model
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: .github/workflows/check.yml
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/canon/tests/ess_gate.rs
- confidence: cited
  path: ess/
revision: 3
---
## Outcome

Canon carries an ESS specification under `ess/` and `task check` fails unless it passes all four
steps of the Atlas ADR 0076 gate: `ess specify validate --path ess --strict-requires` exits 0;
`ess specify compile --path ess` exits 0; `ess verify conform synthesize --path ess` reports 0
refusals; and no file under `ess/` contains the string `UNMAPPED:`. ADR 0076 amends ADR 0067:
the ESS specification covers Canon's data model and command surface. The meaning of a protocol
stays in Canon's own conformance suite.

The specification is `ess/system.yaml` (`format: ess/20`, system `canon`, domains
`[canon.protocol]`), `ess/ess-inputs.yaml` (`format: ess-inputs/2`, `requires: ess 0.52.0`,
listing every authored file, `scenarios: []`) and one domain, `canon.protocol`, retrofitted from
the `protocol/1` source model that story:protocol-source-model built in `crates/canon/src/model/`.
A probe on 2026-10-04 showed that a `canon.protocol` domain with a recursive `Predicate` union
(tag `kind`) validates and synthesizes with 0 refusals at `ess/20` with ess 0.52.0. The probe
union had four variants. The real model has five (it adds `evidence`), and the domain declares
all five.

## Model to specify

The line numbers below were read from the wave-1 tree on 2026-10-04, before it was committed. The
implementor reads them again on merged `main`. Every declaration in `ess/` carries a
`# read from: <file>:<line>` comment beside it, following the `ess:retrofitting` convention.

| ESS declaration | kind | read from |
|---|---|---|
| `Protocol` | the domain's only entity. Identity `protocol_id: ProtocolId` (the header `id`). Fields: `format: String`, `revision: Integer`, `description: Optional<String>`, and the six declaration sections below. One lifecycle state `Declared`, which is terminal. | `crates/canon/src/model/mod.rs:29-44`, `:50-54`; `parse.rs:8` |
| `ProtocolId`, `ArtifactId`, `EvidenceKindId`, `ClaimId`, `ObligationId`, `ActionId`, `OutcomeId`, `CapabilityId`, `EffectClass` | `newtype` of `String` | `crates/canon/src/model/ids.rs:47-94` |
| `artifacts: Map<ArtifactId, Artifact>`; `Artifact {description: Optional<String>}` | struct | `mod.rs:34`, `:60-62` |
| `evidence_kinds: Map<EvidenceKindId, EvidenceKind>`; `EvidenceKind {description}` | struct | `mod.rs:36`, `:68-70` |
| `claims: Map<ClaimId, Claim>`; `Claim {description, true_when: Predicate}` | struct | `mod.rs:38`, `:76-79` |
| `obligations: Map<ObligationId, Obligation>`; `Obligation {description}` | struct | `mod.rs:40`, `:85-87` |
| `actions: Map<ActionId, Action>`; `Action {description, precondition: Optional<Predicate>, requires: List<CapabilityRequirement>, effect: Optional<EffectClass>, may_produce: List<EvidenceProduction>}` | struct | `mod.rs:42`, `:94-104` |
| `CapabilityRequirement {capability: CapabilityId}`, `EvidenceProduction {evidence: EvidenceKindId}` | struct | `mod.rs:110-111`, `:117-118` |
| `outcomes: Map<OutcomeId, Outcome>`; `Outcome {description, requires: Predicate}` | struct | `mod.rs:44`, `:124-127` |
| `Predicate` | `union`, tag `kind`, variants `all`, `any` (each holding a `List<Predicate>`), `not` (one `Predicate`), `evidence` (`EvidenceMatch`), `claim` (`ClaimTest`) | `predicate.rs:57-64` |
| `EvidenceMatch {kind: EvidenceKindId, result: Optional<String>}` | struct | `predicate.rs:70-73` |
| `ClaimTest {claim: ClaimId, is: Truth}` | struct | `predicate.rs:78-80` |
| `Truth` | `enum` `true`, `false`, `unknown` | `predicate.rs:15-18` |

Decisions taken here:

- **A declaration section is a `Map`.** The source writes each section as a map keyed by
  identifier (`mod.rs:132`). The parser keeps duplicate keys only so that the validator can
  report them. That is validation behaviour and belongs to Canon's tests, not to ESS.
- **ESS specifies the typed model, not the `protocol/1` surface syntax.** The source writes a
  predicate as a map with exactly one of the keys `all`, `any`, `not`, `evidence` or `claim`, and
  `is` defaults to `true` (`predicate.rs:112-173`, `:169`). ESS models the same predicate as a union with
  the tag `kind`. Parsing the source syntax is tested by Canon.
- **One state, no commands.** The model has no lifecycle. A retrofit declares only the states the
  code enters, so `Protocol` has one terminal state and the domain has no commands. Synthesis
  therefore produces 0 scenarios and 0 refusals.

## Relations (all settled; none is left open)

- **Protocol to each declaration, including the declarations inside a predicate.** These are
  value types held in the entity's fields. They have no identity outside their protocol, so they
  are not entities and need no `relations:` entry (`mod.rs:29-44`).
- **References inside one protocol.** A predicate refers to a claim or an evidence kind
  (`predicate.rs:71`, `:79`), and an action's `may_produce` refers to an evidence kind
  (`mod.rs:118`). Each is a field typed by the identifier newtype. The rule that every reference
  resolves is `protocol/1` validation semantics, which lives in `crates/canon/src/validate/` and
  Canon's tests. ESS does not state it.
- **Protocol to Case** (design § 4.1, § 36). Not declared here. The source model holds no case:
  `CaseId` is declared at `ids.rs:51-54` and re-exported at `mod.rs:20`, but no struct uses it. The case entity and this relation
  arrive with the story that introduces the case snapshot document, in the same story (see
  `## Later stories` below).

## Disjoint from story:canon-ir

This story runs beside story:canon-ir, and the two share no surface. This story owns `ess/`,
`crates/canon/tests/ess_gate.rs`, the `ess-gate` task in `Taskfile.yml` and its line in `check`,
the ess install step in `.github/workflows/check.yml`, and `AGENTS.md` § ESS. story:canon-ir owns
`crates/canon/src/ir/`, `crates/canon-cli/` and `fixtures/investigation/canon-ir.yaml`. This story
adds no dependency, so `crates/canon/Cargo.toml` and `Cargo.lock` stay untouched: the test runs
the `ess` binary and writes scratch output under `env!("CARGO_TARGET_TMPDIR")`.

## Scope

- In:
  - `ess/system.yaml`, `ess/ess-inputs.yaml` and `ess/domains/protocol.yaml`.
  - `crates/canon/tests/ess_gate.rs`, which runs the four gate steps. If `ess` is not on `PATH`,
    the test fails with a message that names ess 0.52.0. It does not skip.
  - In `Taskfile.yml`, the task `ess-gate` (`cargo test -p b10x-canon --test ess_gate --locked`)
    and a `- task: ess-gate` line as the first step of `check`.
  - In `.github/workflows/check.yml`, an `Install ess` step before `task check`. It is copied from
    beyond10x/commission's `.github/workflows/check.yml`: `ESS_VERSION: "0.52.0"`,
    `ESS_SHA256: 54ae56c63135afc9d8da12e36ef3dcc3d6c4ec8fdc6c2c30fb34bd2d8dc8a5ce`, the release
    asset checked against `SHA256SUMS`, and the binary directory added to `$GITHUB_PATH`.
  - `AGENTS.md` § ESS, replaced. The new text says that ESS specifies Canon's data model and
    command surface under a hard gate (Atlas ADR 0076: validate `--strict-requires`, compile,
    synthesize with 0 refusals, no `UNMAPPED:`, enforced by `task ess-gate` inside `task check`),
    and that the meaning of a protocol stays in Canon's conformance suite (ADR 0067 as amended by
    ADR 0076).
- Out:
  - The case snapshot, evidence and decision documents named in ADR 0076. They do not exist in
    the source model yet; each one enters `ess/` with the story that introduces it.
  - The command surface (`canon validate`, and later `canon compile` and `canon conform run`).
    No story owns it yet; it needs its own story.
  - Generated Rust from `ess generate synthesize --target rust`.
  - Any change to `crates/canon/src/`.
- Surfaces: `ess/`, `crates/canon/tests/ess_gate.rs`, `Taskfile.yml`, `.github/workflows/check.yml`,
  `AGENTS.md`.

## Acceptance

The named test `ess_gate` in `crates/canon/tests/ess_gate.rs` passes under `task check`, which
runs it through `task ess-gate`. It holds these expectations:

1. `ess specify validate --path ess --strict-requires` exits 0.
2. `ess specify compile --path ess` exits 0.
3. `ess verify conform synthesize --path ess --out <CARGO_TARGET_TMPDIR>/canon-ess-suite.json`
   exits 0 and its output reports 0 refusals.
4. No file under `ess/` contains `UNMAPPED:`. The failure message names each offending file and
   line.
5. Negative control: the test copies `ess/` into `CARGO_TARGET_TMPDIR`, appends the line
   `# UNMAPPED: probe` to the copied `domains/protocol.yaml`, and runs the step-4 scan on the
   copy. The scan must report exactly that file and line. Re-adding a `# UNMAPPED:` line to the
   real `ess/` therefore makes `ess_gate`, and with it `task check`, fail.
6. `ess specify toolchain which`, run from the repository root, names ess 0.52.0, which is the
   release that `requires:` pins.

## Later stories

Every later story that changes the source model must update `ess/` in the same story:
story:obligations, story:evidence-freshness, story:invalidation-rules, story:decision-outcomes and
the input shapes of story:three-valued-claims. `ess_gate` does not compare `ess/` with the Rust
model, so a story that changes the model and leaves `ess/` behind still passes the gate. Only the
review of that story catches the drift. Each of those stories carries an `## ESS` section that
says this.

## Source

Atlas ADR 0076 (`architecture/adr/0076-ess-is-a-hard-gate-for-commission-loom-and-canon.md`);
Atlas ADR 0067; the wave-1 model in `crates/canon/src/model/` (story:protocol-source-model);
beyond10x/commission `.github/workflows/check.yml` (the `Install ess` step); the ESS probe of
2026-10-04.

## Spec and model stay equal

Canon's Rust model predates its ESS specification and stays hand-written. Under the
`ess:specifying` rule for a hand-written model, this story adds `crates/canon/tests/ess_model_matches.rs`:
it runs `ess specify compile --path ess --format json` and compares, for every entity and value type
the spec declares, its fields and their types (and every union variant) with the Rust model under
`crates/canon/src/model/`, failing and naming the first difference. It runs in `task ess-gate`, so a
model change that leaves `ess/` behind fails the hard gate. Acceptance expectation 7: adding a field
to a model struct without adding it to `ess/` makes this test fail naming the field.
