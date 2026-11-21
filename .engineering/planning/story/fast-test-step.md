---
format: aep.planning-md/3
id: story:fast-test-step
kind: story
status: active
title: The gate's test step ends within two minutes
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: inferred
  path: .config/nextest.toml
- confidence: cited
  path: .github/workflows/check.yml
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: Taskfile.yml
- confidence: inferred
  path: crates/canon-cli/Cargo.toml
- confidence: inferred
  path: crates/canon-cli/tests
- confidence: inferred
  path: crates/canon-cli/tests/adversary2_review_hardening.rs
- confidence: inferred
  path: crates/canon-expr/Cargo.toml
- confidence: inferred
  path: crates/canon-expr/tests
- confidence: inferred
  path: crates/canon/Cargo.toml
- confidence: inferred
  path: crates/canon/src/check/tests.rs
- confidence: inferred
  path: crates/canon/tests
- confidence: inferred
  path: crates/canon/tests/adversary2_invalidation.rs
- confidence: inferred
  path: crates/canon/tests/adversary2_review_hardening.rs
- confidence: inferred
  path: crates/canon/tests/adversary_check_slices.rs
- confidence: inferred
  path: crates/canon/tests/adversary_check_space.rs
- confidence: inferred
  path: crates/canon/tests/adversary_evidence_order_since.rs
- confidence: inferred
  path: crates/canon/tests/adversary_scenario_generation.rs
- confidence: inferred
  path: crates/canon/tests/argument_validation.rs
- confidence: inferred
  path: crates/canon/tests/invalidation_evaluation_cost.rs
- confidence: inferred
  path: crates/canon/tests/missing_keys.rs
revision: 16
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T21:04:59Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-11-21T21:04:59Z", actor: "human:timo", revision: 9}
---
## Outcome

The test step of the gate (`cargo test --workspace --locked` in `task check`, `Taskfile.yml:13`)
ends within 2 minutes of wall time on the workstation, no single test runs longer than 60 s, the
runner kills a test at 60 s and counts it as failed, and the step prints its ten slowest tests.
A check that cannot be brought under the budget runs in a second lane named `slow`.

The budget is the workspace rule `~/beyond10x/AGENTS.md` § Tests stay fast (2026-11-21):
"The test step of a repository's gate ends within 2 minutes of wall time", "No single test runs
longer than 60 s", "A step over the budget is a defect. It gets a story and is fixed before the
next feature."

## State on 2026-11-21, measured

The gate on `1c402241` (wave 8), on the workstation, with a warm build:

- The test step took 755 s. The build before the first test took 24.63 s
  (`Finished test profile ... in 24.63s`); the 162 test binaries report 728.5 s in their
  `finished in` lines, one after the other.
- Four binaries ran longer than 60 s, 18 longer than 5 s:

  | binary | seconds | tests |
  |---|---|---|
  | `b10x-canon` unit tests (`src/lib.rs`) | 195.28 | 354 |
  | `tests/adversary_scenario_generation.rs` | 92.50 | 2 |
  | `tests/adversary_evidence_order_since.rs` | 63.40 | 7 |
  | `tests/adversary2_review_hardening.rs` | 63.09 | 2 |
  | `tests/adversary_check_slices.rs` | 50.22 | 14 |
  | `tests/missing_keys.rs` | 46.13 | 2 |
  | `tests/adversary2_invalidation.rs` | 30.30 | 9 |
  | `tests/adversary_check_space.rs` | 24.84 | 3 |
  | `tests/invalidation_evaluation_cost.rs` | 23.53 | 4 |
  | `tests/argument_validation.rs` | 18.90 | 22 |

- The same step in a unit of wave 9 (`a1cf482a`): 166 binaries, 754.8 s in `finished in` lines.
- Not measured: the time of a single test. `cargo test` prints none; `cargo-nextest` is installed
  on the workstation and prints them.
- Not measured: how much of the time is the unoptimized build of the tests. The test profile has
  no setting in `Cargo.toml`, so `canon check`'s enumeration and the conformance runs execute
  without optimization.

## What the unit does

Measure first, then take the cheapest step that reaches the budget, and measure again after each:

1. Per-test times with `cargo nextest run --workspace --locked` (one run; it is the measurement
   this story needs, and the only whole-workspace run before the fix). Record the 20 slowest tests
   and the wall time.
2. The runner: `cargo-nextest` as the gate's test step, with a committed `.config/nextest.toml`:
   `slow-timeout = { period = "60s", terminate-after = 1 }` and the ten slowest printed. It runs
   test binaries at once; under `slot` it takes the slots the pool gives. Doc-tests, which nextest
   does not run, keep a `cargo test --doc` line if the workspace has any.
3. Optimization of the code under test in the test profile (`[profile.test.package.b10x-canon]`
   or `[profile.dev.package."*"]`): measure build time and run time before and after; keep the
   setting only if the sum goes down.
4. An expensive product made once per test binary, never per test: the compiled IR of a fixture,
   an enumerated state space, a built binary. `OnceLock` in the binary, read by every test in it.
   Name each test changed and its time before and after.
5. Only for what 1 to 4 leave over the budget: a `slow` lane (`task test-slow`, a nextest profile
   or an `#[ignore]` with a reason that names the lane), run in CI and before a release, never in
   the gate's test step.

Not allowed: deleting a test, weakening an expectation, lowering the number of cases a test
enumerates, raising the timeout, or sharding. The number of tests in the gate lane plus the
`slow` lane equals the number before the story (state both).

## Scope

Derived 2026-11-21 by `story-scoper` from the tree at `efe0859e`. Every line is **cited** (read at the file:line given) or **inferred**. The coordinator's scope was written at `98169001` and is replaced.

- **In:** the gate's test step and its runner config; the test profile; the tests the measurement names; the second lane; the sentence in `AGENTS.md` that names the lanes. — cited (story Outcome)
- **Out:** what any test checks; `crates/*/src` outside `#[cfg(test)]` code. — inferred
- **Primary surface:** the gate's configuration: `Taskfile.yml`, `.config/nextest.toml`, `.github/workflows/check.yml`. — cited
- **Files:** `Taskfile.yml` — cited. `check` task line 13 is `cargo test --workspace --locked`. Five more `cargo test` lines also run tests: line 15 (`adversary_action_arguments`) and `ess-gate` lines 21-24. Those 4 `ess-gate` lines run 6 binaries a second time.
- **Files:** `.config/nextest.toml` — inferred. Neither `.config/` nor `.cargo/` exists (`ls` failed on both). The typed scope marks this path cited, which is wrong: it is a new file.
- **Files:** `Cargo.toml` — cited. 7 lines (`[workspace]`, `[workspace.package]`) and no `[profile.*]`. `crates/*/Cargo.toml` have none either.
- **Files:** `.github/workflows/check.yml` — cited. Its only test step is `run: task check` at line 66 (job `check`, `timeout-minutes: 30`, line 23). It installs go-task (36-47) and ess (49-63) and has no nextest install.
- **Files:** `AGENTS.md` — cited. No sentence names the test step or lanes today. Gate sentences: line 60 (`task ess-gate` "runs first in `task check`"), line 62 ("while the gate is red"), line 135 (the release step: "Land ... with `task check` and CI green"), line 161 (`task docs-check`, "part of `task check`"). Lines 50, 59 and 87 name `task ess-gate` only.
- **Files:** `crates/canon/tests/adversary_scenario_generation.rs` — inferred. This replaces the typed `crates/canon-cli/tests/...` entry, which is the wrong crate. The measured 2-test binary is the `crates/canon` one (2 `#[test]`, 254 lines). The `canon-cli` file has 4 `#[test]` and spawns the `canon` binary (`CARGO_BIN_EXE_canon`, line 26). Cost: `every_generated_witness_passes_and_is_minimal_...` (line 134) generates every `fixtures/investigation` protocol and re-runs `conform::run` once per removal. The second test is cheap.
- **Files:** `crates/canon/tests/adversary_evidence_order_since.rs` — inferred. 7 tests, each calling `check()` on a protocol and a seeded random-history helper (`combinations`, lines 78-110); the whole file is 601 lines. The product enumerated differs per test; there is no shared fixture.
- **Files:** `crates/canon/tests/adversary2_review_hardening.rs` — inferred. 2 tests, in `crates/canon` and also in `crates/canon-cli`; the story's table does not say which was measured. The `canon` one builds a 3000-link chain twice and a 2000-claim diamond and evaluates them. The `canon-cli` one writes a 40 000-claim, 1.6 MB protocol and spawns `canon check` (lines 55-75). Both files are in play until a nextest run names one.
- **Files:** `crates/canon/tests/adversary_check_slices.rs` — inferred. 14 tests; the eight `probe_*` ones each run `check` over slices and over the whole product (header lines 9-15). Each probe builds its own protocol.
- **Files:** `crates/canon/tests/missing_keys.rs` — inferred. 2 tests. The second reads every fixture under `fixtures/` and every `conformance/scenarios` file (lines 212-258), then re-parses each with every key removed, at every depth. Parse-only; no compile product is shared.
- **Files:** `crates/canon/tests/adversary2_invalidation.rs` — inferred. 9 tests; 2 of them loop 4000 and 2000 seeded evaluations over `FAMILY` (3 members, line 91; loops at 226, 272).
- **Files:** `crates/canon/tests/adversary_check_space.rs` — inferred. 3 tests; one enumerates exactly 65 536 states and refuses a 17-kind space (lines 63-96). That one test is the cost.
- **Files:** `crates/canon/tests/invalidation_evaluation_cost.rs` — inferred. 4 tests: 3 time-bounded chain and diamond evaluations (400-claim and 4000-claim chains, 500-claim diamond) and 1 pinned decision. Timing detail is in Safety fact.
- **Files:** `crates/canon/tests/argument_validation.rs` — inferred. 22 tests over `fixtures/investigation/action-arguments.yaml`; the heavy ones run seeded schema rounds (300 documents per schema, `for round in 0..4000`, lines 745-770).
- **Files:** `crates/canon/src/check/tests.rs` — inferred. This is the one `src` file that holds the `b10x-canon` unit time. It has 59 tests in 2600 lines. Heavy tests: three `for _ in 0..20_000` soundness loops (lines 625, 828, 1029; 953 has 5 000); `generated_protocols_check_alike_sliced_and_whole_0..3` (150 protocols each, line 2205); `every_probe_is_a_state_of_the_whole_product` (600 generated protocols plus all fixtures, 2296); `every_fixture_protocol_checks_alike_sliced_and_whole` (2236).
- **Files:** other `b10x-canon` unit modules — inferred. `crates/canon/src/simulate/tests.rs` (20 tests), `crates/canon/src/eval/order.rs` (three `20_000` loops, lines 824, 1308, 1333) and `crates/canon/src/validate/mod.rs` (fixtures walk, line 2116) look expensive. `generate/tests.rs` and `mutate/tests.rs` did not. 366 `#[test]` sit in `crates/canon/src` against 354 measured at `1c402241`; I could not reconcile them.
- **Not touched:** `.github/workflows/pages.yml` — inferred. Line 39 runs `cargo test --locked -p canon-docs`, a test run outside `task check` that is not the gate's step. It is also in `story:local-gate-equals-ci`. `README.md:39` and `website/docs/getting-started.md:80` name `task check` and need no change. The `docs/plan/wave-*.md` files record past gates. No script, hook or Makefile exists in the repository (`git ls-files`). The release text is only `AGENTS.md:135`.
- **Safety fact:** 1. Doc-tests: none run. Every doc fence in `crates` opens as `yaml` (9) or `text` (3), checked by `awk` over lines matching `//[/!] ```` in `git grep` hits; the indented doc lines are list continuations (`simulate/mod.rs:86-101`, `validate/mod.rs:81-83`). So no `cargo test --doc` line is needed. Lib crates `b10x-canon`, `b10x-canon-expr`, `canon-cli` would only print 0 doc-tests. `canon-docs` is bin-only. Unproven, since `cargo test --doc` was not run. 2. Wall-clock tests, which behave differently under an optimized profile or one process per test: `invalidation_evaluation_cost.rs:206` and `:272` branch on `cfg!(debug_assertions)` (budgets 120 s and 30 s in debug, 10 s and 5 s release). Line 26 `TIMED` serializes only inside one process. Line 177 asserts a 400-claim time `<= short*64`. The 120 s budget exceeds the 60 s kill, but the whole binary measured 23.53 s. Other timed tests: `src/validate/mod.rs:2307` (10 s), `adversary2_admissibility_authority.rs:88` (<20 s), `adversary2_admissibility_reasons.rs:206-212` (<20 s), `adversary2_tvc_eval.rs:144` (20 s), `adversary_conform_steps.rs:115-119` (<10 s), `review_properties.rs:453` (20 s). `adversary2_review_hardening.rs:111-122` only prints a time. 3. Already ignored, 3 tests, all children of a parent test that re-runs the same binary via `current_exe` with `--ignored --exact`: `child_evaluates_a_chain_of_claims_each_within_the_bound` (`adversary_review_hardening.rs:102`), `child_checks_a_caller_built_ir_beyond_the_bound` (`:132`), `child_evaluates_a_caller_built_ir_nested_beyond_the_bound` (`review_depth_beyond_bound.rs:30`); each reason names its parent test. `adversary2_tvc_eval.rs:103`, `adversary_ess_commands.rs:72` and `adversary2_ess_commands.rs:78` also use `current_exe`. Under nextest these paths are unproven.
- **Confidence:** medium. The gate files are cited from the tree; the per-test costs are a reading and have no measured times.
- **Would collide with:** `Taskfile.yml`, `.github/workflows/check.yml` and `AGENTS.md` (also in `story:local-gate-equals-ci`, with `.cargo/config.toml` and `rust-toolchain.toml`); and `crates/canon/src/check/tests.rs` and `crates/canon/tests/*.rs` for any story that adds tests.
- **Not established by the scoper:** which `adversary2_review_hardening.rs` was measured; the cost split between build and run; the 354/366 count gap; which unit modules take the 195 s; whether tests that share fixed scratch names or `CARGO_TARGET_TMPDIR` dirs collide when nextest runs binaries at once.

## Acceptance

- `task check`'s test step on the workstation with a warm build ends within 120 s of wall time,
  measured three times; the three times are in the report.
- The step's output lists its ten slowest tests, and none is over 60 s.
- A test made to sleep 61 s is killed by the runner and fails the step; shown once in the
  report, not committed.
- The number of tests in the gate lane plus the `slow` lane equals the number before the story;
  every test that moved to `slow` is named with its time and the reason steps 2 to 4 did not
  bring it under the budget.
- CI (`check.yml`) runs the same test step and the `slow` lane and is green.
- `AGENTS.md` names the two lanes, the budget and the file that holds the runner's limits.

## Source

`~/beyond10x/AGENTS.md` § Tests stay fast, a rule for every repository under that
folder, quoting the operator on 2026-11-21. The `runbook` session sent its path to this session
on 2026-11-21 and asked for the measured time of the gate's test step.

## Added 2026-11-21: the step's own bound

The workspace rule gained two lines the same evening (`~/beyond10x/AGENTS.md` § Tests stay fast):
"The step itself fails when it runs over 120 s, locally and in CI. A repository whose runner does
not kill a test at 60 s yet gets that change as its first budget story", and "Every sub-agent
brief that runs tests names the 60 s bound and the 120 s step budget." The `host-operator`
session sent them to this session.

They bind this story, in addition to § Acceptance:

- **First commit after the measurement: the runner's 60 s kill.** `.config/nextest.toml` with
  `slow-timeout = { period = "60s", terminate-after = 1 }`, and the gate's test step run by it,
  land before any test is made faster. A test that runs over 60 s today then fails; the unit
  brings it under the bound or moves it to the `slow` lane in the commits that follow, and the
  branch is not merged while one fails.
- **The step fails over 120 s**, locally and in CI: the gate's test task measures its own wall
  time (build excluded: the tests are built by a step before it) and exits non-zero over 120 s,
  printing the time and the ten slowest tests. In CI the same task runs, so the same bound holds;
  the unit states the time of the CI run and, if the runner is slower than the workstation, says
  so with both numbers and does not raise the bound.
- **A test that proves the bound**: the task run on a tree with a test made to sleep 61 s exits
  non-zero by the kill, and with the step's limit set to 1 s exits non-zero by the step bound;
  both shown in the report, neither committed as a sleeping test.
- `AGENTS.md` states both bounds, so every brief written from it names them.

## Added at stage 1 of wave 10, 2026-11-21: what the scope changes

The scope (§ Scope, read from the tree at `efe0859e`; nothing was run for it) corrects this story
and names what the measurement has to settle. Each binds the unit in addition to § Acceptance.

- **The 92.50 s binary is `crates/canon/tests/adversary_scenario_generation.rs`** (2 tests), not
  the file of that name under `crates/canon-cli/tests/`. The typed scope is corrected.
- **Two files are named `adversary2_review_hardening.rs`**, one per crate; the measurement says
  which one took 63.09 s.
- **The 195 s of the `b10x-canon` unit tests** are not attributed. Candidates read from the code:
  `crates/canon/src/check/tests.rs` (three loops of 20 000, 150 generated protocols in each of
  four tests, 600 in another), `crates/canon/src/eval/order.rs` (three loops of 20 000),
  `crates/canon/src/simulate/tests.rs`, `crates/canon/src/validate/mod.rs:2116`. The measurement
  names the tests; the unit changes only those.
- **`Taskfile.yml` runs tests on six lines**, not one: line 13, line 15 and the four lines of
  `ess-gate` (21-24), which run 6 binaries a second time. The 120 s bound is on all test runs of
  `task check` together, and the unit says how the second runs are counted or removed.
- **Tests with a wall-clock budget**, which an optimized test profile or one process per test can
  change: `crates/canon/tests/invalidation_evaluation_cost.rs:206,272` branch on
  `cfg!(debug_assertions)` (120 s and 30 s in debug, 10 s and 5 s otherwise) and serialize by a
  lock inside one process (line 26); also `src/validate/mod.rs:2307`,
  `adversary2_admissibility_authority.rs:88`, `adversary2_admissibility_reasons.rs:206-212`,
  `adversary2_tvc_eval.rs:144`, `adversary_conform_steps.rs:115-119`, `review_properties.rs:453`.
  A budget over 60 s is brought under the kill bound. No budget is raised.
- **Tests that re-run their own binary** (`current_exe` with `--ignored --exact`):
  `adversary_review_hardening.rs:102,132`, `review_depth_beyond_bound.rs:30`,
  `adversary2_tvc_eval.rs:103`, `adversary_ess_commands.rs:72`, `adversary2_ess_commands.rs:78`.
  The unit shows each one passes under the new runner. The 3 ignored tests are children of those
  parents and stay ignored.
- **Scratch paths.** Tests that write to a fixed name or to `CARGO_TARGET_TMPDIR` can collide
  when binaries run at once; the unit checks it and fixes a collision by a per-test directory.
- **No doc-test runs today** (every doc fence is `yaml` or `text`); the unit confirms it once and
  the new step does not need a `--doc` run if so.
- **The count of tests is taken from the runner**, before and after. 366 `#[test]` were counted
  in `crates/canon/src` against 354 run at `1c402241`; the unit states the reason for the 12.
- `.github/workflows/pages.yml:39` runs `cargo test -p canon-docs` outside the gate. It is not in
  the 120 s bound; the unit reports its time.

## Added 2026-11-21 at 23:22 CET: few test binaries, no full debug info

`~/beyond10x/AGENTS.md` § Tests stay fast gained a line (lines 44-47): "Test binaries are few:
one integration-test binary per crate (`tests/main.rs` with modules), and `[profile.dev] debug =
false` or a test profile without full debug info." Its cause, in the same line: another
repository's first whole-workspace nextest run "linked 516 test executables of up to 269 MB
each, 33 GB in `target/debug/deps` within 15 minutes; canon's step runs 162 binaries one after
another." The `host-operator` session sent it for this story and asks for the binary count
before and after.

Counted at `67e5dee8`: 100 files in `crates/canon/tests`, 55 in `crates/canon-cli/tests`, 3 in
`crates/canon-expr/tests`; each is one test executable today.

It binds this story, in addition to § Acceptance:

- One integration-test binary per crate, and a test profile without full debug info. The unit
  measures link time and the size of `target/` before and after, and reports the number of test
  executables before and after, per crate.
- A test file that is in no module tree never runs and nothing fails. A test reads each crate's
  `tests/` directory and fails when a `.rs` file there is not a declared module of the one
  binary. The count of tests from the runner is equal before and after.
- The form that keeps every test file at its path is preferred (`autotests = false` and one
  `[[test]]`): other stories and the store's typed scopes name files under `crates/*/tests/`.
- Everything that names a test binary changes with it: the `--test` lines of `Taskfile.yml`,
  `AGENTS.md`, the tests that re-run their own binary with `--exact`. `pages.yml` is not this
  unit's; what must change there is handed over as a patch.
- Tests that relied on one process per file are named, with the runner each remaining test line
  uses.
