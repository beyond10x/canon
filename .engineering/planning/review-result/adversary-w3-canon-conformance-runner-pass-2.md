---
format: aep.planning-md/3
id: review-result:adversary-w3-canon-conformance-runner-pass-2
kind: review-result
status: active
title: Wave 2026-10-04-w3 adversary, canon story:conformance-runner, pass 2
relations:
- reviews: story:conformance-runner
revision: 1
---
unit: canon/conformance-runner, working tree on ef4bcf7 plus the uncommitted phase 2 and pass-1 fixes (worktree `~/.local/state/worktree/trees/b10x/canon/canon-w3-conformance-runner`)
verdict: CONFIRMED (4 red cases: 2 INFEASIBLE warnings, 2 CONFIRMED notes; nothing holds the unit as a blocker)
cases: executed 144→160, red 4
origin: introduced 6 / pre-existing 0 / undecided 0
wrote-outside-worktree: 21 paths (part 6)
needs-coordinator: whether symlink confinement is in scope (finding 1) — the unit's own test promises "refused unread"; the doc promises a text-only check

**1. Diff stat**

`git --no-pager diff --stat` lists only `crates/canon-cli/src/main.rs` and `crates/canon/src/conform/mod.rs`. Both are the implementor's uncommitted phase 2; I did not edit them. My three files are untracked, so they are not in that output:
- `crates/canon-cli/tests/adversary2_conform_paths.rs`
- `crates/canon-cli/tests/adversary2_conform_exit.rs`
- `crates/canon/tests/adversary2_conform_lines.rs`

I did not touch any implementation file.

**2. Cases added.** Each file was run alone first. Logs are in `scratch/adv2-{paths,exit,lines}.log`.

| case | now | red output from the run alone |
|---|---|---|
| paths.rs:72 `a_fixture_symlink_leading_outside_the_working_directory_is_not_read` | red | `passed: scenario `RUNNER-COMPILE-PASSES`` / `conform: 1 passed, 0 failed, 0 unreadable` (it read `link.yaml`, which points to `../outside/protocol.yaml`) |
| paths.rs:91 `a_fixture_under_a_symlinked_directory_leading_outside_is_not_read` | red | same output (`fixtures`, a symlink to `../outside`) |
| exit.rs:62 `a_missing_registry_is_reported_as_the_documentation_shows` | red | `left: "error[unreadable]: \"no-such-registry\": No such file or directory (os error 2)\n"`; the doc at mod.rs:74 says `<dir>` |
| exit.rs:138 `a_registry_file_that_is_not_a_scenario_is_not_echoed` | red | `unreadable: scenarios/a.yaml: invalid type: string "SECRET-SCALAR-VALUE", expected struct RawScenario` |

12 more cases are green guards. Paths: `.`/`//` segments, a fixture path that is a directory (os error 21), a 6006-byte path. Exit: registry path is a file → 2; non-UTF-8 and directory `*.yaml` entries → `unreadable:` lines, exit 1; ids that differ only in case. Lines: CRLF scenario file and CRLF fixture both pass; `|-` differs at line 81 (the last); `|+` differs at line 82; a CRLF on expectation line 2 differs at line 2; the scenario shape from story:three-valued-claims parses into 4 evaluate steps and fails `unsupported` at the first.

**Mutants.** All three were applied together in a scratch copy with its own build dir, then deleted. Every one of the unit's own tests and all pass-1 tests stayed green under them (`scratch/adv2-mutant.log`).

| mutant | caught by |
|---|---|
| M6 mod.rs:437 compares with `trim_end()` (no longer byte for byte) | lines.rs:67, :80 (`left: None, right: Some("…at line 82")`) |
| M4 mod.rs:508 duplicate-id check ignores case | exit.rs:113 |
| M8 main.rs:~112 an unreadable scenario file exits 2 early | exit.rs:92 (`left: ""`) |

**3. Suite run** (after the cases existed): `cargo test --workspace --locked --no-fail-fast` gives 156 passed, 4 failed, exit 101. The only failures are the 4 cases above. `cargo test -- --list` lists 160 tests. `cargo fmt --all --check` exits 0 and `cargo clippy … -D warnings` exits 0, after rustfmt (edition 2024) ran on my three files only. The 144 comes from the per-binary counts of that same run minus my 16 cases.

**4. Findings** (tree as above)

| # | file:line | finding | verdict / origin | what reaches it |
|---|---|---|---|---|
| 1 | mod.rs:310, main.rs:114 | Fixture confinement only checks the text. A fixture that is a symlink, or sits under a symlinked directory, pointing outside the working directory is read and can make a scenario pass. `conform_registry.rs` promises "refused unread". Fix: canonicalize the path in the main.rs:114 reader and check it starts with the canonical working directory, or narrow the doc at mod.rs:306. | INFEASIBLE / introduced, warning | Only a symlink committed to the repo. I found nobody doing that, and CI already runs repo code. |
| 2 | main.rs:86, :95 vs mod.rs:74 | stderr prints the path with Rust debug quoting (`"dir"`). The doc's `<dir>` and the stdout `unreadable:` lines print it unquoted. | CONFIRMED / introduced, note | Plain `canon conform run`: the default `conformance/scenarios` does not exist yet, so it exits 2 with this line. |
| 3 | mod.rs:237 | When a registry file does not parse, the serde message is shown as-is: it quotes the file's content and the internal type name `RawScenario`. Pass 1 limited fixture errors to kind and position only; registry files get no such limit. | CONFIRMED / introduced, note | Any non-scenario `*.yaml` in a `--scenarios` directory. |
| 4 | mod.rs:437 | M6 survived the unit's suite. The byte-for-byte compare is only tested on `first_difference` directly, not through a scenario. | CONFIRMED / introduced, note | Closed by lines.rs:67/:80 |
| 5 | mod.rs:508 | M4 survived the unit's suite. | CONFIRMED / introduced, note | Closed by exit.rs:113 |
| 6 | main.rs:105 | M8 survived the unit's suite. | CONFIRMED / introduced, note | Closed by exit.rs:92 |

**5. Attacked, could not break**
- `..` look-alikes (U+2025, U+FF0E): these are ordinary file names on Linux, not an escape. Windows-only path quirks (trailing dots, `CON`): could not run them here.
- Empty and `.` components; a directory or over-long fixture path: one line, exit 1, same output on a second run.
- unreadable, failed and exit 0/1/2 are consistent on every path I tried. Ids that differ only in case get stable, separate lines.
- CRLF and trailing-newline differences are reported at the right line.
- Next story: its 4-step evaluate scenario parses, and evaluate steps can be wired inside `run`/`compile_step` without changing any public signature. One risk: `StepKind`, `Verdict` and `Scenario` are exhaustive public types and `EvaluateInputs` exposes `serde_yaml_ng::Value`, but nothing outside this workspace matches on them.

**6. Paths written outside the worktree**
- `~/.cache/ga-wave-2026-10-04-w3/canon-conformance-runner/scratch/adv2-{paths,exit,lines,mutant,suite,fmt,clippy}.log` (7 files)
- `~/.cache/ga-wave-2026-10-04-w3/canon-conformance-runner/scratch/adv2-mutant` and `adv2-mutant-target`: created and deleted again.
- 12 test workspaces under `~/.cache/b10x-target/canon-w3-conformance-runner/tmp/`: `adversary2-paths-{file-link,dir-link,dots,directory,long}` and `adversary2-exit-{missing,file,entries,case,echo}` (10), plus `adversary2-outside`… — the `outside/` dirs are inside each `adversary2-paths-*` root. The other `adversary2-*` entries in that tmp dir come from earlier units.
- Worktree session lease `adversary2-canon-conformance-runner`: taken and released.

```findings
- file: crates/canon/src/conform/mod.rs
  line: 310
  category: boundary
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: "is_confined is text-only, so a fixture that is a symlink or sits under a symlinked directory pointing outside the working directory is read and the scenario passes (adversary2_conform_paths.rs:72, :91), contrary to conform_registry.rs's refused-unread guarantee"
- file: crates/canon-cli/src/main.rs
  line: 86
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "an unreadable registry is reported as error[unreadable]: \"dir\": with debug quoting, while mod.rs:74 documents <dir> and stdout lines print paths unquoted (adversary2_conform_exit.rs:62)"
- file: crates/canon/src/conform/mod.rs
  line: 237
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "a registry file that does not parse as a scenario has its content and the internal type name RawScenario quoted on its unreadable line, unlike fixtures, which pass 1 limited to kind and position (adversary2_conform_exit.rs:138)"
- file: crates/canon/src/conform/mod.rs
  line: 437
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "comparing with trim_end() survives the unit's suite; the byte-for-byte trailing-newline contract is now guarded by adversary2_conform_lines.rs:67 and :80"
- file: crates/canon/src/conform/mod.rs
  line: 508
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "a case-insensitive duplicate-id check survives the unit's suite; now guarded by adversary2_conform_exit.rs:113"
- file: crates/canon-cli/src/main.rs
  line: 105
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "exiting 2 early when a scenario file is unreadable survives the unit's suite; now guarded by adversary2_conform_exit.rs:92"
```
