---
format: aep.planning-md/3
id: story:fast-test-step
kind: story
status: draft
title: The gate's test step ends within two minutes
relations:
- decomposes: epic:canon-kernel
scope:
- confidence: cited
  path: .config/nextest.toml
- confidence: cited
  path: .github/workflows/check.yml
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/canon-cli/tests/adversary_scenario_generation.rs
revision: 7
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

Written by the wave coordinator on 2026-11-21 from the tree at `98169001`; every line is
**cited** (read from the tree) or **inferred**.

- **In:** the runner and its configuration; the test profile; the tests named by the
  measurement; the `slow` lane; the gate's and CI's test step; the sentence in `AGENTS.md` that
  names the lanes. — cited from the rule
- **Out:** a change to what any test checks; the product's code under `crates/*/src` other than
  test modules and `#[cfg(test)]` helpers (inferred: a slow enumeration in the product is a
  finding for the backlog, not this story).
- **Files:** `Taskfile.yml` (`check`, line 13; a `test` and a `test-slow` task) — cited
- **Files:** `.config/nextest.toml` (new) — inferred
- **Files:** `Cargo.toml` (profile section; none exists today) — cited
- **Files:** `.github/workflows/check.yml` (runs `task check`, line 66; installs the runner; runs
  the `slow` lane) — cited
- **Files:** `AGENTS.md` (the gate's lanes) — cited
- **Files:** the test files the measurement names, among them
  `crates/canon-cli/tests/adversary_scenario_generation.rs`,
  `crates/canon/tests/adversary_evidence_order_since.rs`,
  `crates/canon/tests/adversary2_review_hardening.rs`, and the unit test modules of `b10x-canon`
  — inferred from the binary times; the crate each file is in is not checked
- **Would collide with:** every story that adds tests. It runs alone, before the next feature
  wave.

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
