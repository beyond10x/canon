---
format: aep.planning-md/3
id: story:conformance-suite
kind: story
status: active
title: Run the normative conformance suite
refs:
- provider: taskboard
  reference: C-010
relations:
- decomposes: epic:canon-kernel
- depends_on: story:explanation
- depends_on: story:conformance-runner
- depends_on: story:three-valued-claims
- depends_on: story:evidence-revision-binding
- depends_on: story:obligations
- depends_on: story:action-admissibility
- depends_on: story:outcomes
- depends_on: story:evidence-freshness
- depends_on: story:decision-outcomes
- depends_on: story:invalidation-rules
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: conformance/requirements.yaml
- confidence: cited
  path: crates/canon-cli/src/conform.rs
- confidence: inferred
  path: crates/canon-cli/src/lib.rs
- confidence: inferred
  path: crates/canon-cli/src/main.rs
- confidence: inferred
  path: crates/canon-cli/tests/conform_requirements.rs
- confidence: cited
  path: crates/canon/src/conform/
- confidence: inferred
  path: crates/canon/src/model/catalogue.rs
- confidence: inferred
  path: crates/canon/src/model/mod.rs
- confidence: inferred
  path: crates/canon/tests/ess_model_matches.rs
- confidence: inferred
  path: ess/domains/conform.yaml
- confidence: inferred
  path: ess/ess-inputs.yaml
- confidence: inferred
  path: ess/system.yaml
- confidence: inferred
  path: fixtures/conformance/requirements/
- confidence: inferred
  path: website/docs/reference/
- confidence: inferred
  path: website/status.yaml
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:50Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":5}}}
- {from: "proposed", to: "active", at: "2026-10-15T04:11:44Z", actor: "human:timo", revision: 14, decided_on: {"recorded":{"review_outcome":9}}}
---
## Outcome

Canon has a normative conformance suite that is authoritative for what a protocol means (Atlas
ADR 0067): a catalogue, `conformance/requirements.yaml`, of the normative requirements the kernel
implements, each covered by one or more scenarios in the registry `conformance/scenarios/`.
`canon conform run --requirements conformance/requirements.yaml` checks coverage and determinism
over that registry, and `task check` runs it with the flag.

## The catalogue (`canon-requirements/1`)

A YAML document:

```yaml
format: canon-requirements/1
requirements:
  CANON-CLAIM-001:
    status: implemented        # implemented | planned
    statement: A required positive predicate evaluating to UNKNOWN MUST NOT be treated as satisfied.
    source: design § 32        # where the requirement is stated
```

Keys are `CANON-` identifiers, each once; unknown keys are refused. The catalogue at this story's
merge lists, as `implemented`, every requirement a registry scenario covers on `origin/main`
(0.5.0): CANON-CLAIM-001, CANON-CLAIM-002, CANON-EVIDENCE-001, CANON-EVIDENCE-002,
CANON-EVIDENCE-003, CANON-AUTHORITY-001, CANON-INDEPENDENCE-001, CANON-OUTCOME-001,
CANON-OUTCOME-002, CANON-INVALIDATION-001, CANON-DETERMINISM-001, CANON-EXPLAIN-001,
CANON-OBLIGATION-001, CANON-INPUT-001 and CANON-ORDER-001 (15). Statements come from design § 32
where it states the requirement, else from the story that introduced the scenario, cited in
`source`. `planned` exists for a requirement stated but not yet implemented; none is listed today
(recovery rules have no stated `CANON-*` requirement, and none is invented here).

## The checks (additive to the runner)

Without `--requirements`, `canon conform run` behaves exactly as story:conformance-runner defines
it, byte for byte (operator decision 2026-10-04). With it, after the registry has run:

1. **Coverage.** Every `implemented` requirement has at least one scenario that covers it and
   passed; otherwise `uncovered: requirement `<id>``. A scenario covering an id the catalogue does
   not list fails the run as `unlisted: requirement `<id>` covered by scenario `<scenario>``.
   A `planned` requirement needs no scenario; one that a passing scenario covers is reported
   `planned requirement `<id>` is covered` and fails the run, so the catalogue is moved forward
   rather than drifting.
2. **Determinism.** Every evaluate step runs three times: as written, again as written, and once
   with the `evidence`, `authority` and `decisions` lists in reverse order (an equivalent input:
   none of the three is ordered by position, CANON-DETERMINISM-001). The three canonical
   `canon-decision/1` outputs (or refusal codes) must be byte-identical; otherwise
   `nondeterministic: scenario `<id>` step `<step>`: <rerun|permuted> differs at line <n>`.
3. The summary line gains `requirements: <n> implemented, <n> covered, <n> planned`. Exit status
   is 1 when any check fails, 2 when the catalogue cannot be read (`error[unreadable]`), with the
   runner's existing codes otherwise.

The checks live in the library (`crates/canon/src/conform/`), pure like the runner; the CLI reads
the catalogue file and passes its text in.

## ESS first

- Specification change: `ess/` declares the `canon-requirements/1` document, as `ess/domains/check.yaml`
  declares `canon-properties/1`: a new domain `canon.conform` in `ess/domains/conform.yaml`, listed
  in `ess/system.yaml` and in `ess/ess-inputs.yaml`'s `specification`, with
  `RequirementCatalogue { format, requirements: Map<RequirementId, NormativeRequirement> }`,
  `NormativeRequirement { status, statement, source }` and `RequirementStatus { implemented,
  planned }`, each carrying `# read from:`. (`OutcomeRequirement` in `model/requirement.rs` is an
  unrelated type; these names avoid it.)
- The Rust model is hand-written where the parity test reads it: a new file
  `crates/canon/src/model/catalogue.rs`, declared from `crates/canon/src/model/mod.rs`.
  `crates/canon/tests/ess_model_matches.rs` adds `canon.conform` to `DOMAINS` and
  `RequirementCatalogue` to `DOCUMENTS` (no entity reaches the catalogue), so the model is held
  equal to `ess/`. The checks in `crates/canon/src/conform/` read that model type. `task ess-gate`
  green on this commit.
- The first commit adds that specification and model, `conformance/requirements.yaml`, the
  fixtures the named test reads (a catalogue with an implemented requirement no scenario covers; a
  registry with a scenario covering an unlisted id; a scenario whose decision differs when its
  evidence list is reversed) and the test.
- Red on that commit: `conform_run_checks_requirements` fails, because `canon conform run` has no
  `--requirements` flag.
- The flag changes the CLI: `task docs-generate` regenerates `website/docs/reference/`, and
  `website/status.yaml` gains the capability.

## Acceptance

The named test `conform_run_checks_requirements` (in `crates/canon-cli/tests/`) passes, and it
asserts each of: `canon conform run` over the real registry `conformance/scenarios/` without
`--requirements` prints exactly the runner's lines and summary with no `requirements:` line and
exits 0; with `--requirements conformance/requirements.yaml` it exits 0 and prints
`requirements: 15 implemented, 15 covered, 0 planned`; given a fixture catalogue listing an
implemented requirement no scenario covers it exits 1 printing `uncovered: requirement `<that
id>``; given a fixture registry whose scenario covers an id the catalogue omits it exits 1
printing `unlisted: …` naming both; over a fixture scenario whose decision differs when its
evidence list is reversed it exits 1 printing `nondeterministic: scenario `<id>` step `<step>`:
permuted …`; and the `check` task of `Taskfile.yml`, read as YAML, runs a command containing
`conform run --requirements conformance/requirements.yaml`.

## Scope

- In: `ess/domains/conform.yaml`, `ess/system.yaml`, `ess/ess-inputs.yaml`,
  `crates/canon/src/model/catalogue.rs`, `crates/canon/src/model/mod.rs`,
  `crates/canon/tests/ess_model_matches.rs`, `crates/canon/src/conform/` (the checks),
  `crates/canon-cli/src/conform.rs`, `crates/canon-cli/src/lib.rs` (the flag),
  `crates/canon-cli/src/main.rs` (the `conform run` arm passes it), `crates/canon-cli/tests/`
  (the named test), `conformance/requirements.yaml`, `fixtures/conformance/requirements/`,
  `Taskfile.yml` (`task check` passes the flag), `website/docs/reference/` (generated),
  `website/status.yaml`, `CHANGELOG.md`.
- Out: the scenario format and runner (story:conformance-runner); `canon conform synthesize`
  (design § 29); argument-schema validation of story:action-arguments, which is a `canon validate`
  rule held by its own tests and states no `CANON-*` requirement.

## Order

Every depends_on of this story is implemented, and it is the only ready unit. It shares surfaces
with stories still in draft, each of which now depends_on this story, so this lands first and
they rebase onto it:

- story:protocol-imports — `crates/canon/src/conform/`, `crates/canon-cli/src/lib.rs`,
  `crates/canon-cli/src/main.rs`, `website/docs/reference/`, `website/status.yaml`, and its new
  scenario CANON-IMPORT-001.
- story:ess-command-surface — `ess/system.yaml` (this story adds the `canon.conform` domain, so
  that story's "no other story adds a domain" no longer holds) and the declaration of
  `conform run`, which must include `--requirements`.
- story:canon-mutate, story:protocol-floor — `crates/canon-cli/src/main.rs`; canon-mutate also
  calls the runner in `crates/canon/src/conform/`, whose existing entry points this story keeps.
- story:capability-scope, story:revision-ordering, story:case-composition (and
  story:protocol-imports) — each adds a scenario with a new id (CANON-SCOPE-001, CANON-ORDER-002,
  CANON-COMPOSE-001, CANON-IMPORT-001). Once this story lands, a scenario covering an id the
  catalogue omits fails `task check` as `unlisted`, so each of those stories adds its requirement
  to `conformance/requirements.yaml` in its own first commit.

## Source

TASKBOARD C-010 (Atlas `docs/design/governed-autonomy/TASKBOARD.md`);
`docs/design/canon-protocol-calculus-design.md` § 28, § 29, § 32, § 41 item 14; AGENTS.md § Rules;
build pack `ROADMAP.md` Phase 1 exit criterion.
