---
format: aep.planning-md/3
id: story:canon-mutate
kind: story
status: implemented
title: Measure scenario strength with canon mutate
relations:
- decomposes: epic:protocol-analysis
- depends_on: story:semantic-diff
- depends_on: story:conformance-runner
- depends_on: story:action-admissibility
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:conformance-suite
- depends_on: story:ess-command-surface
scope:
- confidence: cited
  path: crates/canon-cli/src/lib.rs
- confidence: cited
  path: crates/canon-cli/src/main.rs
- confidence: cited
  path: crates/canon-cli/src/mutate.rs
- confidence: cited
  path: crates/canon-cli/tests/canon_mutate.rs
- confidence: cited
  path: crates/canon/src/lib.rs
- confidence: cited
  path: crates/canon/src/mutate/mod.rs
- confidence: inferred
  path: ess/domains/cli.yaml
- confidence: cited
  path: fixtures/investigation/mutate/
revision: 21
transitions:
- {from: "draft", to: "proposed", at: "2026-11-20T20:58:27Z", actor: "human:timo", revision: 3, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w1"}
- {from: "proposed", to: "active", at: "2026-11-20T20:58:27Z", actor: "human:timo", revision: 4, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w1"}
- {from: "active", to: "implemented", at: "2026-11-22T11:12:32Z", actor: "human:timo", revision: 21, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}, correlation: "wave-2026-11-20-w9"}
---
## Outcome

`canon mutate --path <protocol> --scenarios <dir>` (clap derive; every subcommand that reads a
protocol source takes `--path`, `crates/canon-cli/src/lib.rs:59-147`) applies five mutation
operators to the protocol: drop a conjunct, `all` → `any`, remove a precondition, remove a
capability requirement, change an evidence result value. Each mutant runs against the given
`canon-conformance/1` scenarios; a mutant no scenario fails is a survivor. Output is deterministic.
Part 3 of epic:protocol-analysis.

Each mutant is classified with the semantic diff (story:semantic-diff, `diff::diff`,
`crates/canon/src/diff/mod.rs:257`) and carries the categories of its changes. The classifier
landed with five categories (`diff/mod.rs:145-151`), so the two labels this story was drafted with
are defined over them: a mutant is **weakening** when its diff holds at least one `RELAXATION`,
and **strengthening** when it holds only `TIGHTENING`; a mutant that is neither is reported with
its categories as they are (`BREAKING`, `EXPANSION`, `NO SEMANTIC CHANGE`). A weakening survivor is
reported as a weak point of the scenario set.

Decided by the rescope of 2026-11-20, where the draft was silent:

- **Exit status.** 0 when no weak point is reported, 1 when at least one is ("a check found
  something", the existing meaning of 1 in `EXIT_STATUSES`, `lib.rs:21-43`).
- **Which scenarios run against a mutant.** `conform::run_registry`
  (`crates/canon/src/conform/mod.rs:875`) reads no file itself; the caller's `read_fixture` closure
  supplies each fixture's text. `canon mutate` hands the mutant's text in for the fixture whose
  path is the `--path` file, and every other fixture unchanged. A scenario set in which no scenario
  names the `--path` file as its fixture is refused, exit 1.
- **A mutant the validator or the compiler refuses** is reported as `invalid` with the refusal's
  code. It is neither killed nor a survivor.

The scenario directory is an input: the hand-written `conformance/scenarios/` or what
`canon generate` (story:scenario-generation) writes. This story needs neither to exist in a
particular form, so it does not depend on story:scenario-generation.

## ESS first

- Specification change, first commit: the `mutate` command with its inputs `path` and `scenarios`
  in `ess/domains/cli.yaml`, the command surface story:ess-command-surface declares; the fixture
  set `fixtures/investigation/mutate/` (the investigation protocol and two scenario sets, one
  lacking a witness for one outcome); and the named test below. Mutants are `protocol/1` documents
  and the classification is story:semantic-diff's, so the model does not change.
- Red on that commit: `mutate_reports_weakening_survivors` fails, because `canon mutate` is not a
  subcommand and clap refuses it with exit 64 (`crates/canon-cli/src/main.rs:206-214`); and
  `every_canon_subcommand_is_declared_in_ess` (story:ess-command-surface) fails naming `mutate`,
  declared in `ess/` and absent from clap.

## Scope

Rescoped 2026-11-20 by `story-scoper` against `8f1f452`, written by the wave coordinator. Every
line is **cited** (read from the story or the tree) or **inferred** (a reading that could be
wrong).

- **In:** the five operators over `protocol/1`; running each mutant through
  `conform::run_registry`; classifying each with `diff::diff`; the survivor report; the `mutate`
  subcommand and its exit-status wording. — cited
- **Out:** operators over independence demands (`independent_on`,
  `crates/canon/src/model/predicate.rs:244`), order forms (`first`, `before`, `after`, `since`,
  `predicate.rs:121-132`), `children` forms (`predicate.rs:135`) and `case_open`
  (`crates/canon/src/model/mod.rs:218`). All four are in the kernel since 0.9.0 and are left for a
  follow-up story. Operators over capability scope (story:capability-scope). Extending
  story:semantic-diff's categories. — cited
- **Primary surface:** `crates/canon/src/mutate/mod.rs` (new) — cited
- **Files:** `crates/canon-cli/src/lib.rs` (the `Mutate` variant of `Command`, `lib.rs:57-164`, and
  the `EXIT_STATUSES` wording, `lib.rs:21-43`) — cited
- **Files:** `crates/canon-cli/src/main.rs` (the `mod` line and the dispatch arm) — cited
- **Files:** `crates/canon-cli/src/mutate.rs` (new, laid out like `check.rs` and `generate.rs`:
  `pub(crate) fn run`) — cited
- **Files:** `crates/canon/src/lib.rs` (`pub mod mutate;`, beside `lib.rs:8-16`) — cited
- **Files:** `crates/canon-cli/tests/canon_mutate.rs` (new) — cited
- **Files:** `fixtures/investigation/mutate/` (new directory, owned whole) — cited
- **Files:** `ess/domains/cli.yaml` (the `mutate` declaration; the file is created by
  story:ess-command-surface) — inferred
- **Coordinator's at integration, not in the typed scope:** the story's entry in
  `website/status.yaml`, and the regenerated `website/docs/reference/cli.md` and
  `website/data/status.json`. `canon-docs` walks clap (`crates/canon-docs/src/pages.rs:123`), so no
  code in `crates/canon-docs/` changes.
- **Not touched:** `conformance/requirements.yaml`, `ess/system.yaml`,
  `ess/domains/protocol.yaml`, `website/static/schemas/`.
- **Confidence:** high. The body names the new files, and the tree shows where the variant, the
  dispatch and the library entry points live.
- **Would collide with:** any unit that adds or changes a `canon` subcommand
  (`crates/canon-cli/src/lib.rs`, `main.rs`, `ess/domains/cli.yaml`) or adds a library module
  (`crates/canon/src/lib.rs`): story:protocol-imports and story:protocol-floor.
- **Not established by the scoper:** whether every mutant compiles (an emptied `all`, for
  example); the file names under `fixtures/investigation/mutate/`.

## Order

Every earlier edge is satisfied: story:semantic-diff, story:conformance-runner,
story:action-admissibility and story:conformance-suite are implemented.

depends_on story:ess-command-surface (edge added 2026-11-20): `canon mutate` is a new command, and
with that story landed a new command is declared in `ess/domains/cli.yaml` in the first commit.

Shares `crates/canon-cli/src/lib.rs`, `main.rs` and `ess/domains/cli.yaml` with
story:protocol-imports and story:protocol-floor, and `crates/canon/src/lib.rs` with
story:protocol-floor, so the three run in three different waves.

## Acceptance

The named test `mutate_reports_weakening_survivors` (in `crates/canon-cli/tests/canon_mutate.rs`)
passes with five expectations over `fixtures/investigation/mutate/`: every operator yields at least
one mutant and each mutant carries its semantic-diff categories; with the scenario set that lacks
a witness, the mutant that weakens that outcome's requirement survives, is reported as a weak
point, and the command exits 1; with the complete set, that mutant is killed and the command exits
0; a scenario set that names no scenario for the `--path` file is refused with exit 1; and two runs
give identical bytes. `every_canon_subcommand_is_declared_in_ess` passes.

## Source

epic:protocol-analysis part 3; the `ess:hardening` skill's mutation technique; Atlas ADR 0080;
rescope of 2026-11-20 (`approval-record:accept-drafts-2026-11-20`).

## Decided in review, after adversary pass 1

`review-result:adversary-canon-mutate-pass-1` was red by one case. The coordinator decided the
points below on 2026-11-21; the correction is `4a545374` and `d3e3e09f`.

- **D1. The mutant is handed in only where the protocol is a scenario's fixture.** The unit had
  also handed it in where a scenario names the protocol under `imports`. An import is pinned by
  the digest of its compiled IR, so every importing scenario failed on `import-digest-mismatch`
  for every mutant that changes the IR; the mutant counted as killed and the run exited 0. § Outcome
  already says it: the mutant goes in for "the fixture whose path is the `--path` file, and every
  other fixture unchanged". Now only the scenarios whose fixture is the protocol run, in the
  baseline and against the mutants; a scenario that names the protocol only as an import is not
  run; a set in which the protocol is only ever an import is refused `no-scenario`.
- **D2. No digest pin is rewritten.** Running a mutant of an imported protocol through its
  importers' scenarios needs the importer's pin rewritten per mutant. That is not this story; it
  is on the plan's backlog page, cost not assessed.
- **D3. The read-back guard needed no code.** Removing it left the unit's tests green. The
  adversary's case `mutate_reports_a_mutant_that_does_not_read_back_as_unwritable` fails on that
  mutant. A protocol reaches the guard only with an identifier of 1023 bytes or more.
- **D4. No second adversary pass.** The correction changes 59 lines of
  `crates/canon/src/mutate/mod.rs`, in the one function that chooses the scenarios, plus comments
  and help text. The coordinator read it. The adversary's red case passes unchanged, and two unit
  tests pin the rules of D1 (`a_protocol_named_only_as_an_import_is_refused`,
  `a_scenario_importing_the_protocol_does_not_run`).
- **D5. § Acceptance holds as written.** The classifier gives `BREAKING` for any change to an
  outcome's own requirement, so such a mutant is never a weak point. The weak point of the
  `lacking` set is a mutant of the claim the outcome reads. The adversary checked 15 mutants by
  hand against `canon conform run` and `canon diff`; all 15 were equal to what `canon mutate`
  printed.
- **D6. The measurement the brief asked for could not be made.** The coordinator's brief named
  `fixtures/investigation/protocol.yaml` with `conformance/scenarios/`; no scenario of the suite
  names that file, and `canon mutate` refuses it (`no-scenario`) in 0.10 s. The unit measured the
  27 fixtures the suite does name: 0.11 to 0.50 s each on a debug binary, 0 invalid mutants. The
  same 27 gave the same counts after the correction.

## Scope as built

Written at the close of wave 2026-11-20-w9 from the wave page, the adversary report (`review-result:adversary-canon-mutate-pass-1`), the review of the integration branch (`review-result:review-wave-9-model`) and the merge of `impl/canon-mutate` (`f1e72413`: 27 files, +2347 −12). It corrects the body where the two differ.

- Each mutant is the parsed model with one change, written with `model::write`. The correction after pass 1 (`4a545374`, `d3e3e09f`) hands the mutant in only for a scenario whose fixture is the mutated protocol, and runs only those scenarios (D1 to D6, "Decided in review").
- Every path of the typed scope changed. Outside it: `crates/canon/src/mutate/tests.rs`, `crates/canon-cli/tests/adversary_mutate.rs` and `fixtures/mutate-adversary/import/` (4 files, the adversary's), and the generated `website/docs/reference/cli.md`.
- After the merge: story:revision-ordering and story:decision-condition each changed `crates/canon/src/mutate/mod.rs` for their new forms. The review found that `canon mutate` makes no mutant at a conditioned decision and does not report the skipped site (model F2, a note; documented and pinned by `mutate_makes_no_mutant_at_a_conditioned_decision`); it is in story:review-notes-wave-9 (draft).
- Acceptance: `mutate_reports_weakening_survivors` passed in the test step and `every_canon_subcommand_is_declared_in_ess` in the `ess-gate` step of the gate below. Not shown at the close-out: the five expectations one by one (assertions of that test).
- `website/data/status.json` is not regenerated by the close-out; it reads this story's status.

**Gate of the close-out**, on `wave/2026-11-21` at `2f3d14b14`, 2026-11-22 10:43 to 11:01 UTC, host load 67 on 20 CPUs, rustc 1.100.0, each step run once by the coordinator's gate script (not `task check` as one command): build exit 0 (125 s); `task ess-gate` 0; `cargo fmt --check` 0; `cargo clippy --workspace --all-targets --locked -- -D warnings` 0; `task test` 0 (1275 tests; the step's own line: `ess-gate 2.2 s + workspace 50.4 s + -p b10x-canon 2.7 s = 55.4 s, bound 120 s`; slowest 16.9 s); `canon conform run` 0 (`30 passed, 0 failed`; `31 implemented, 31 covered`); `task docs-check` 0 (18 generated files current); `task test-slow` exit 201 (63 tests: 53 passed, 10 killed at 60 s; its CI step may fail, D123, story:slow-lane-under-bound). Recorded as `test_result` evidence. The four review results of the wave are `review-result:review-wave-9-eval`, `-simulate`, `-model` and `-gate`.
