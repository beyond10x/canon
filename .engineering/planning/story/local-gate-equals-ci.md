---
format: aep.planning-md/3
id: story:local-gate-equals-ci
kind: story
status: implemented
title: The local gate and CI use one pinned compiler and one environment
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: inferred
  path: .cargo/config.toml
- confidence: cited
  path: .github/workflows/check.yml
- confidence: cited
  path: .github/workflows/pages.yml
- confidence: cited
  path: AGENTS.md
- confidence: inferred
  path: Cargo.toml
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: rust-toolchain.toml
- confidence: inferred
  path: scripts/ci-watch
- confidence: inferred
  path: website/docs/getting-started.md
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T21:46:08Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-11-21T21:46:08Z", actor: "human:timo", revision: 9}
- {from: "active", to: "implemented", at: "2026-11-22T11:12:33Z", actor: "human:timo", revision: 14, decided_on: {"recorded":{"test_result":1}}, correlation: "wave-2026-11-20-w9"}
---
## Outcome

The local gate and CI compile with one pinned rustc, read from one file; what the CI runner does
differently is written in the repository and used by the local gate; and the script that pushes
and waits for CI exits non-zero when a run ends red.

The rule is the workspace file `~/beyond10x/AGENTS.md` § The local gate equals the forge
(2026-11-21): "`rust-toolchain.toml` pins one rustc; the local gate and CI both read it."
"A forge or CI run that ends red makes the push or watch script exit non-zero."

## State on 2026-11-21, checked

- No `rust-toolchain.toml` in the repository. `check.yml` installs `toolchain: stable`, which was
  rustc 1.100.0 in this plan's CI runs; the workstation's stable is rustc 1.99.1
  (`rustc --version`). Clippy of 1.100.0 has refused code that clippy of 1.99.1 accepted in this
  plan (waves 4 to 7; every unit brief since carries the sentence "CI runs clippy from rustc
  1.100.0").
- `check.yml` sets `CARGO_INCREMENTAL=0` and `CARGO_TERM_COLOR=always`; the local gate sets
  neither. No `.cargo/config.toml` exists.
- Two builds of `serde_json` exist: the workspace build keeps number text, `cargo test -p
  b10x-canon` alone reads doubles. CI runs the workspace build only.
- The coordinator's wait script for pull request checks is outside the repository (a scratch
  file) and exits 0 as soon as no check is pending, whatever the checks' results.
- Not checked: stack size of the runner's test threads against the workstation's. Three tests of
  deep nesting overflowed the stack locally in wave 9 when `Predicate` grew to 120 bytes; whether
  CI would have failed the same way is not known.

## What the unit does

1. `rust-toolchain.toml` at the root: one exact version (the current stable at the time of the
   unit, stated), with `clippy` and `rustfmt`. `check.yml` reads it (no `toolchain: stable`).
   Local `cargo` then uses the same compiler. The unit runs fmt and clippy with it and fixes what
   the newer clippy reports, each fix named.
2. The runner's environment in the repository: what `check.yml` sets in `env`, moved to
   `.cargo/config.toml` `[env]` or to the `Taskfile.yml` gate tasks, so that `task check` is the
   same command with the same settings in both places.
3. `task ci-watch` (a script under `scripts/`): given a pull request number or a branch, waits
   until no check is pending, prints each check with its result and link, and exits 0 only when
   every check passed; a failed or cancelled check, or a timeout, exits non-zero.
4. `AGENTS.md`: the pinned version's file, how it is raised (one commit that changes the file and
   fixes what the new compiler reports), and `task ci-watch`.

## Scope

Derived 2026-11-21 by `story-scoper` from `efe0859e4`. Every line is **cited** or **inferred**.

- **In:** one pinned rustc read by `check.yml`, `pages.yml` and local cargo. The runner's environment (`env`) moved into the repository so `task check` runs the same command with the same settings. `task ci-watch`. `AGENTS.md` text. Also the `ess` binary that `task check` finds on `PATH` (item 3 below). — inferred, since the story's four gaps do not name it.
- **Out:** the test step's time (story:fast-test-step) and the path filter (story:planning-only-checks). `shared-gates.yml` and `b10x-docs-site.yml` choose no toolchain. — cited (`shared-gates.yml:16`, `b10x-docs-site.yml:33`; both only `uses:` a shared workflow).
- **Primary surface:** `.github/workflows/check.yml` plus a new `rust-toolchain.toml`. The work is configuration and one script, with no crate source. — cited
- **Files:** `rust-toolchain.toml` (new; none exists, `git ls-files` finds none) — cited. Version 1.100.0 is installed here, so pinning it downloads nothing. Installed: stable (= 1.99.1, the active one), nightly, 1.85, 1.91, 1.97, 1.97.0, 1.98.0, 1.98.1, 1.99.0, 1.99.1, 1.100.0 (`rustup toolchain list`).
- **Files:** `.github/workflows/check.yml` — cited. Toolchain action `dtolnay/rust-toolchain@4360b525…` with `toolchain: stable` and `components: clippy, rustfmt` (lines 31-34). Runner `ubuntu-latest` (22), `timeout-minutes: 30` (23). `env` sets `CARGO_TERM_COLOR: always` and `CARGO_INCREMENTAL: "0"` (15-17). The only build step is `task check` (66).
- **Files:** `.github/workflows/pages.yml` — cited. It compiles Rust at lines 39-41 and 45: `cargo test --locked -p canon-docs`, `cargo build --locked -p canon-cli`, `cargo run … generate --check`, `site-manifest`. Its toolchain is `toolchain: stable` with no components (29-31). `env` repeats the same two variables (16-18). `timeout-minutes: 20` (24), Node 22 (34). It never runs `task`, so `task` env settings do not reach it.
- **Files:** `.cargo/config.toml` (new; the story says none exists and the tree confirms it) — inferred. Only worth creating if `[env]` or `[build] incremental` is chosen over `Taskfile.yml`.
- **Files:** `Taskfile.yml` — cited. `check` (7-16) and `docs-check` (32-36) set no env, and the file has no wait or push task.
- **Files:** `scripts/ci-watch` (new; no `scripts/` directory exists) — inferred.
- **Files:** `AGENTS.md` — cited. The pin file and `task ci-watch` go in § Releases, line 135 ("`task check` and CI green"). Line 104 forbids one setting: "Build into the tree's own `target/`; never set `CARGO_TARGET_DIR`." Nothing else is forbidden locally.
- **Files:** `website/docs/getting-started.md:8` — inferred. It says "You need a current stable Rust toolchain", and `README.md` gives no Rust version. A pin would make "current stable" imprecise, so this line is optional.
- **Files:** `Cargo.toml` (`rust-version`) — inferred. None is set anywhere (`git grep rust-version`); it is optional. Edition 2024 is at `Cargo.toml:7`.
- **Files:** `crates/**` — inferred. Clippy 1.100.0 may force fixes, but nothing here was run, so the files are unknown.
- **Environment differences found** (all cited unless marked)
  - rustc: `rustc --version` prints 1.99.1 (58cf8f9c3, 2026-10-13). CI's `stable` was 1.100.0 per the story, which I could not check from here — inferred.
  - `~/.cargo/config.toml` exists and CI has none of it: `rustc-wrapper = "sccache"`, `[profile.dev] debug = "line-tables-only"`, and `rustflags = ["-C","link-arg=-fuse-ld=lld"]` for x86_64-linux. A repo `[build] rustflags` is ignored whenever a target-specific `rustflags` exists, so use `[env]` or `[build] incremental`, not `rustflags`.
  - No `CARGO_*` or `RUST*` variable is set in this shell. `CARGO_INCREMENTAL` and `CARGO_TERM_COLOR` are set only in the two workflows, nowhere in `Taskfile.yml`.
  - `ess` on the workstation `PATH` is 0.97.1 (`ess --version`). CI installs 0.77.0 (`check.yml:51`), `ess/ess-inputs.yaml:2` requires 0.77.0, and the gate tests fail on the wrong one (`crates/canon/tests/ess_gate.rs:103-106`). The 0.77.0 binary is in `~/.cache/ess/toolchains/0.77.0`, and `docs/plan/wave-plan-2026-11-20.md:91` says it must come first on `PATH`. The story does not cover this.
  - go-task 3.52.0 is pinned in CI (`check.yml:38`); I did not check the local version.
  - Two `serde_json` builds are real: `crates/canon-expr/Cargo.toml:14` enables `arbitrary_precision` and `crates/canon/Cargo.toml:10` does not. `task check` runs per-package `cargo test -p` lines (`Taskfile.yml:21-24`) and a workspace test, locally and in CI alike, so this is not a gate difference.
- **Environment-dependent tests**
  - Stack: no `stacker` and no `RUST_MIN_STACK` anywhere in `crates`. `adversary_protocol_write.rs:131-132` and `:412-413` pin 2 MiB threads explicitly at 400 levels deep. `adversary2_tvc_eval.rs:140` uses a default `thread::spawn`, a 20 s `recv_timeout`, and 40 levels. The library uses 64 MiB (`eval/mod.rs:769`) and 256 MiB (`eval/read.rs:908`) stacks. The explicit 2 MiB threads fix the stack size on both machines, but frame size follows the rustc version, so the single pin is what makes them comparable. Whether CI overflows the same way is inferred.
  - Timing: 20 s, 10 s, 30 s and 120 s budgets (`adversary2_tvc_eval.rs:144`, `adversary_conform_steps.rs:118`, `invalidation_evaluation_cost.rs:207,273`) can behave differently on a slower runner — inferred. The local run took 297 s (`docs/plan/wave-plan-2026-11-20.md:93`).
  - Git-ignored files: `.gitignore` lists only `/target`, `/.engineering/drafts/`, `/website/build/`. No test reads those. Tests read `ess/`, `fixtures/`, `conformance/`, `website/docs/reference/*`, `website/data/status.json` and `website/static/schemas`, all tracked. `canon-docs` reads `.engineering/planning/story` (`status.rs:17`), which is tracked, so changing a story's status can trip docs drift.
  - Locale, time zone, home and network: no `HOME`, `TZ`, `LANG`, `LC_`, socket or HTTP use in `crates`. Tests use `CARGO_TARGET_TMPDIR` and `CARGO_MANIFEST_DIR`. — cited (`git grep`)
- **Release check today:** nothing in `Taskfile.yml`, `scripts/` or `.github/` waits for CI or pushes. `AGENTS.md:135-143` says "with `task check` and CI green", checked by hand. Required checks are verified by hand through `b10x-gates` (`:142-143`). — cited
- **Not touched:** `b10x-docs-site.yml`, `shared-gates.yml`, `crates/**` source, `website/` generated pages, `ess/`. — cited
- **Confidence:** high — every file and setting is read from the tree; only the scale of clippy fixes is open.
- **Would collide with:** `.github/workflows/check.yml`, `Taskfile.yml` and `AGENTS.md` (story:planning-only-checks edits all three, story:fast-test-step edits the first and third). `.cargo/config.toml` and `Cargo.toml` too (story:fast-test-step scopes `Cargo.toml`). — cited from the two stories' `scope:`
- **Not established by the scoper:** the rustc 1.100.0 clippy findings and the files they touch; CI's actual rustc today; local go-task version; whether CI overflows the stack; `.config/nextest.toml` (story:fast-test-step) interacting with the pinned toolchain.

## Acceptance

- `rustc --version` in the repository on the workstation and the version CI prints are equal;
  both lines are in the report.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` exits 0 with the pinned
  compiler locally and in CI.
- `task ci-watch` on a run that ended red exits non-zero, and on a green run exits 0; both shown
  with the run's link.
- `AGENTS.md` names the pin file and `task ci-watch`.

## Source

`~/beyond10x/AGENTS.md` § The local gate equals the forge. The `host-operator` session, on
2026-11-21: "check 4". The check found the four gaps above.

## Added at stage 1 of wave 10, 2026-11-21: what the scope changes

The scope (§ Scope, read from the tree at `efe0859e`) found one more gap and three facts about
the workstation. Each binds the unit in addition to § Acceptance.

- **A fifth gap: `ess`.** CI installs `ess` 0.77.0 (`check.yml`, `ESS_VERSION:`); the `ess` first
  on `PATH` on the workstation is 0.97.1, and `crates/canon/tests/ess_gate.rs:103-106` fails on
  the wrong one. The local gate finds the pinned `ess` itself: `task check` puts
  `~/.cache/ess/toolchains/<version>` first on `PATH` when it exists and otherwise stops with the
  install command. The version is read from one place; `ESS_VERSION:` stays exactly once in
  `check.yml` (`ess_gate.rs:432-444`).
- **`pages.yml` reads the pin too.** `.github/workflows/pages.yml:29-31` says `toolchain: stable`.
  Both workflows take the compiler from `rust-toolchain.toml`.
- **The pin is 1.100.0**, the version CI ran on 2026-11-21 and one installed on the workstation.
  What `cargo clippy -- -D warnings` reports under 1.100.0 is fixed in this unit.
- **The workstation's `~/.cargo/config.toml` is not edited and not copied.** It sets
  `rustc-wrapper = "sccache"`, `[profile.dev] debug = "line-tables-only"` and `rustflags` for the
  target. A `[build] rustflags` in the repository loses to a target `rustflags` of the user's
  file, so the unit carries the runner's settings as environment of the gate (`[env]` in
  `.cargo/config.toml`, or the task's own `env`), and states for each of the three settings
  whether it changes what the gate checks.
- **`.cargo/config.toml` and `scripts/ci-watch` are new files**; the first typed scope marked
  them as read.
- **Tests with a wall-clock budget** can fail on a slower runner and pass here:
  `crates/canon/tests/adversary2_tvc_eval.rs:144`, `adversary_conform_steps.rs:118`,
  `invalidation_evaluation_cost.rs:207,273`. This unit does not change them; it reports each
  one's time on CI and on the workstation. story:fast-test-step owns their time.
- `AGENTS.md` § Releases (line 135) is where the pin file and `task ci-watch` are named.

## Scope as built

Written at the close of wave 2026-11-20-w9 from the wave page, the review of the integration branch (`review-result:review-wave-9-gate`) and the merge of `impl/local-gate-equals-ci` (`33cd84d6`: 8 files, +224 −15). It corrects the body where the two differ.

- Changed: `rust-toolchain.toml` (1.100.0, read by both workflows), `.github/workflows/check.yml`, `.github/workflows/pages.yml`, `Taskfile.yml` (`task ci-watch`, the pinned `ess` first on `PATH`), `AGENTS.md` § Releases, `scripts/ci-watch`, `website/docs/getting-started.md`. Outside the typed scope: `crates/canon-cli/tests/ci_watch.rs`. In the typed scope and not changed: `.cargo/config.toml` (story:fast-test-step created it, `eb7c3a4d7`) and `Cargo.toml`.
- The three workstation settings are carried none: no test reads a `CARGO_*`, `RUST*` or `CI` variable (the unit's reading).
- After the merge: correction unit 1 (`b166206f1`) makes `scripts/ci-watch` require the job `canon / task check` (`--require`, `CI_WATCH_REQUIRE`) and exit 2 at once for a branch without a pull request (gate G2, G8).
- Acceptance: `cargo clippy --workspace --all-targets --locked -- -D warnings` exited 0 with rustc 1.100.0 in the gate below, and the `ci_watch` tests passed (8). `AGENTS.md` names `rust-toolchain.toml` and `task ci-watch`. **Not shown:** the rustc version CI prints beside the local one (the pin on the CI runner is unproven); clippy on CI; `task ci-watch` on a red and on a green run with their links (the unit's run against pull request 28 ended pending, exit 2); the times of the four wall-clock-budget tests (`adversary2_tvc_eval.rs:144`, `adversary_conform_steps.rs:118`, `invalidation_evaluation_cost.rs:207,273`) on CI and on the workstation.

**Gate of the close-out**, on `wave/2026-11-21` at `2f3d14b14`, 2026-11-22 10:43 to 11:01 UTC, host load 67 on 20 CPUs, rustc 1.100.0, each step run once by the coordinator's gate script (not `task check` as one command): build exit 0 (125 s); `task ess-gate` 0; `cargo fmt --check` 0; `cargo clippy --workspace --all-targets --locked -- -D warnings` 0; `task test` 0 (1275 tests; the step's own line: `ess-gate 2.2 s + workspace 50.4 s + -p b10x-canon 2.7 s = 55.4 s, bound 120 s`; slowest 16.9 s); `canon conform run` 0 (`30 passed, 0 failed`; `31 implemented, 31 covered`); `task docs-check` 0 (18 generated files current); `task test-slow` exit 201 (63 tests: 53 passed, 10 killed at 60 s; its CI step may fail, D123, story:slow-lane-under-bound). Recorded as `test_result` evidence. The four review results of the wave are `review-result:review-wave-9-eval`, `-simulate`, `-model` and `-gate`.
