---
format: aep.planning-md/3
id: review-result:adversary-ess-command-surface-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:ess-command-surface (wave 2026-11-20-w1): red, 5 findings, all introduced'
relations:
- reviews: story:ess-command-surface
revision: 2
---
Adversary pass 1 on story:ess-command-surface, wave 2026-11-20-w1, `aep:adversary` on Opus. The
report below is as the agent returned it; the only change is that `&lt;`, `&gt;` and `&amp;`, which
the transport escaped, are written as the characters they stand for.

unit: story:ess-command-surface at 036ee3d
verdict: red
cases: executed 46→56, red 1
origin: introduced 5, pre-existing 0, undecided 0
wrote-outside-worktree: $HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w1/ess-command-surface/adversary-p1/{probe.sh,probe2.sh,probes/,probes2/,mirror.sh,mirror2.sh,mirror/ (369M, includes target/),*.log,suite.sh,suite2.sh,msg.txt}
needs-coordinator: no

**1. Diff since d2c8e53:** `crates/canon-cli/tests/adversary_ess_commands.rs | 303 +` and nothing else; that is a test file. `git status` is clean. Nothing in the tree was mutated. All mutants ran in a scratch copy of the repo (`mirror/`).

**2. Cases** in `crates/canon-cli/tests/adversary_ess_commands.rs`. The file compiles `ess_commands_match.rs` in unchanged, as module `parity`. Each case runs the real `parity::every_canon_subcommand_is_declared_in_ess` as a child process, with `CARGO_MANIFEST_DIR` pointing at a copy of `ess/` that the case has edited. So a case turns green when the unit fixes its own test, not when this file changes.
- `an_ess_command_sharing_a_wire_is_named`: **RED**. It adds `canon.cli.Floor` with `wire: validate` and input `budget`, the way an author copying Validate's block in a spec-first commit would. Output from running it alone, before the suite:
  ```
  panicked at crates/canon-cli/tests/adversary_ess_commands.rs:129:5:
  ESS declares canon.cli.Floor (wire `validate`, input `budget`), which is no canon subcommand, and the parity test passed without naming it:
  test parity::every_canon_subcommand_is_declared_in_ess ... synthesized scenarios per command: [7 commands, Floor absent] ok
  ```
- `an_input_only_in_ess_is_named`, `a_command_only_in_clap_is_named`, `an_optional_flag_required_in_ess_is_named`: green now. Each goes red in the mirror when its branch is deleted from `differences()`, and the unit's 4 cases stay green under those mutants.
- `long_flags_on_every_command_line_level_are_declared`: green now. It walks the built clap definition, so global flags reach every leaf and flags on parent commands count. It goes red in the mirror under a global `--probe-global` on `Cli` plus a `--probe-parent` on `Conform`, while the unit's `every_canon_subcommand_is_declared_in_ess` passes (exit 0).
- `an_ess_command_with_its_own_wire_is_named`: green. It is the control showing the child process can see a failure.

**3. Suite**, run after the cases existed (ess 0.77.0 first on `PATH`):
- `slot task ess-gate`: 11 + 28 + 3 + 4 = 46 passed, exit 0. My file is not part of `ess-gate`.
- `cargo test -p canon-cli --test adversary_ess_commands --locked`: `FAILED. 9 passed; 1 failed`, exit 101.
  - The 10 cases are my 6 plus the unit's 4 compiled in again as `parity::*`.
  - With `--skip an_ess_command_sharing_a_wire_is_named`: 9 passed.
- clippy `-D warnings` and `fmt --check`: exit 0.
- Side effect: `cargo test --workspace` now runs the 4 parity cases twice.

**4. Findings** (they cover 036ee3d)

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon-cli/tests/ess_commands_match.rs:157 | acceptance | blocker | CONFIRMED | introduced | the spec-first commit AGENTS.md § ESS requires, made by copying a command block; ess 0.77.0 accepts duplicate wires (measured) |
| crates/canon-cli/tests/ess_commands_match.rs:107 | mutant | warning | CONFIRMED | introduced | nothing in the current tree; any later global flag on `Cli` or flag on `conform` (mirror run 1) |
| crates/canon-cli/tests/ess_commands_match.rs:190 | mutant | note | CONFIRMED | introduced | a regression in `differences()`; spec-first flag stories such as story:protocol-imports rely on the line-190 branch being red (mirror run 2) |
| AGENTS.md:42 | judgement | note | CONFIRMED | introduced | any later flag with a hyphen: ess 0.77.0 refuses `at-time` ("contains '-'"), so "an input of the same name" can't be met |
| crates/canon-cli/tests/ess_commands_match.rs:154 | judgement | note | CONFIRMED | introduced | input types are not compared. `Optional<List<String>>` for single-valued `--authority` passes. Story:protocol-imports adds a repeatable `--import`. This is outside the stated acceptance. |

Fixes I am naming but did not apply:
- Line 157: fail when `insert` returns `Some`, i.e. a duplicate wire.
- Line 107: call `build()` before walking, skip the `help` subcommand, and pass the parent commands' flags down to the leaves.

**5. Attacked and not broken:**
- Dropping `cli.yaml` from `ess-inputs.yaml`, or `canon.cli` from `system.yaml`: ess refuses, so the test fails.
- `ess` missing from `PATH`, or another release: the test panics, it does not skip.
- Positionals and short-only flags: reported as unmapped.
- Default value counts as optional.
- The three controls each check the first difference against specific text.
- A command with 0 scenarios fails, but only for commands that survive the wire de-duplication.
- Every `# read from:` in `lib.rs` and `main.rs` points at the right line.
- `Rejected` and `Unreadable` match the exit statuses at `lib.rs:21-43` and the exit paths in `check.rs`, `diff.rs` and `conform.rs`.

**6. Written outside the worktree:** the paths in the header, all under `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w1/ess-command-surface/adversary-p1/`. `mirror/target` (369M) can be deleted. Inside the worktree, `target/tmp/adversary-ess-commands/` was also written.

```findings
- file: crates/canon-cli/tests/ess_commands_match.rs
  line: 157
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: ess_surface keys commands by naming.wire and overwrites on collision, so an ESS command sharing a wire with a later-sorted one (canon.cli.Floor, wire validate) is never compared and the gate passes with a command that is no canon subcommand.
- file: crates/canon-cli/tests/ess_commands_match.rs
  line: 107
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: collect_leaves walks the unbuilt clap Command and reads only leaf arguments, so a global flag on Cli or a flag on the parent conform command passes the gate undeclared.
- file: crates/canon-cli/tests/ess_commands_match.rs
  line: 190
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: no control reaches the input-only-in-ESS, command-only-in-clap or required-only-in-ESS branches of differences(); deleting each leaves all four unit cases green.
- file: AGENTS.md
  line: 42
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the mapping "every long flag an input of the same name" cannot hold for a hyphenated flag because ess 0.77.0 refuses '-' in field names, and neither the rule nor the checker defines a translation.
- file: crates/canon-cli/tests/ess_commands_match.rs
  line: 154
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: input types, repeatability and defaults are not compared, so Optional<List<String>> declared for a single-valued flag passes the gate.
```
