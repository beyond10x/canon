---
format: aep.planning-md/3
id: story:local-gate-equals-ci
kind: story
status: active
title: The local gate and CI use one pinned compiler and one environment
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: .cargo/config.toml
- confidence: cited
  path: .github/workflows/check.yml
- confidence: cited
  path: .github/workflows/pages.yml
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: rust-toolchain.toml
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T21:46:08Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-11-21T21:46:08Z", actor: "human:timo", revision: 9}
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

Written by the wave coordinator on 2026-11-21 from the tree at `a93962cd`.

- **Files:** `rust-toolchain.toml` (new) — cited
- **Files:** `.github/workflows/check.yml` (the toolchain step, `env`) — cited
- **Files:** `.github/workflows/pages.yml` (it runs `cargo test --locked -p canon-docs`, line 39;
  its toolchain step) — cited
- **Files:** `.cargo/config.toml` (new) — inferred
- **Files:** `Taskfile.yml`, `scripts/` (the watch script) — inferred
- **Files:** `AGENTS.md` — cited
- **Out:** the test step's time (story:fast-test-step); the path filter
  (story:planning-only-checks).
- **Would collide with:** story:fast-test-step and story:planning-only-checks
  (`.github/workflows/check.yml`, `Taskfile.yml`, `AGENTS.md`).

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
