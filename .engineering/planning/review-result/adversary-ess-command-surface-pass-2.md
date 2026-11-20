---
format: aep.planning-md/3
id: review-result:adversary-ess-command-surface-pass-2
kind: review-result
status: active
title: 'Adversary pass 2 on story:ess-command-surface (wave 2026-11-20-w1): red, 3 findings, all introduced'
relations:
- reviews: story:ess-command-surface
revision: 2
---
Adversary pass 2 on story:ess-command-surface, wave 2026-11-20-w1, `aep:adversary` on Opus (the
same agent as pass 1, resumed). The report below is as the agent returned it; the only change is
that `&lt;`, `&gt;` and `&amp;`, which the transport escaped, are written as the characters they
stand for.

unit: story:ess-command-surface at 889a93c
verdict: red
cases: executed 68→80, red 1
origin: introduced 3, pre-existing 0, undecided 0
wrote-outside-worktree: $HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w1/ess-command-surface/adversary-p2/{probe.sh,probes/,inc/,mirror.sh,mirror.log,suite.sh,suite.log,msg.txt}; adversary-p1/mirror/ (rewritten, 369M with target/)
needs-coordinator: no

**1. Diff since 8ceb315:** `crates/canon-cli/tests/adversary2_ess_commands.rs | 132 +` and nothing else; that is a test file. `git status` is clean. All mutants ran in the scratch mirror. The executed count is ess-gate 52 plus `adversary_ess_commands` 16 before, plus `adversary2_ess_commands` 12 after. Of those 12, 10 are the parity cases compiled in again.

**2. Cases** in `crates/canon-cli/tests/adversary2_ess_commands.rs`. They use the same child-process harness as pass 1, running the real parity test.
- `a_single_valued_flag_declared_as_a_map_is_named`: **RED**. It declares `canon evaluate --authority`, which clap takes once, as `Optional<Map<String, String>>`. ess 0.77.0 compiles and synthesizes that. Output from running it alone, before the suite:
  ```
  panicked at crates/canon-cli/tests/adversary2_ess_commands.rs:112:5:
  ESS declares `canon evaluate --authority`, which clap takes once, as Optional<Map<String, String>>, and the parity test passed without naming it:
  test parity::every_canon_subcommand_is_declared_in_ess ... [7 commands, 3 scenarios each] ok
  ```
  The same probe with `Map<String, String>` on the required `--ir` also passes the gate.
- `a_single_valued_flag_declared_as_a_list_fails_the_child`: green. It is the control showing the harness sees a failure.

**3. Suite**, run after the case existed:
- `slot task ess-gate`: 11 + 28 + 3 + 10 = 52 passed, exit 0.
- `adversary_ess_commands`: 16 passed, exit 0.
- `adversary2_ess_commands`: `FAILED. 11 passed; 1 failed`, exit 101.
- clippy `-D warnings` and `fmt --all --check`: exit 0.

**Pass-1 findings, with each original probe run again:**
- F1, shared wire (`:157`): **closed**. My `an_ess_command_sharing_a_wire_is_named` is green, and the shared wire is now reported.
- F2, global and parent flags (`:107`): **closed**. Mirror mutant with `--probe-global` on `Cli` and `--probe-parent` on `Conform`: `every_canon_subcommand_is_declared_in_ess` fails and names `canon check --probe-global`.
- F3, the three uncovered branches (`:190`): **closed in `task check`, still open in `task ess-gate`**.
  - With the three branches deleted from `differences()`, my pass-1 cases go red (3 FAILED).
  - Those cases run only through `cargo test --workspace`; no case in `ess_commands_match.rs` reaches those branches.
  - This is the same finding, not a new one.
- F4, hyphenated flags (`AGENTS.md:42`): **closed**. The input name is now the flag with every `-` written `_`, and a long flag holding `_` is reported.
- F5, collection kind and types (`:154`): **closed for `List`**. The element type is not compared, by design, and is not re-raised. `Map` is the new finding below.

**4. Findings** (they cover 889a93c)

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon-cli/tests/ess_commands_match.rs:235 | contract-drift | warning | CONFIRMED | introduced | an author declaring a key/value or collection input; nothing in the tree today |
| crates/canon-cli/tests/ess_commands_match.rs:164 | mutant | note | CONFIRMED | introduced | nothing found; any later `visible_alias` |
| AGENTS.md:46 | contract-drift | note | CONFIRMED | introduced | nothing found; any later `ArgAction::Count` flag |

- `:235` treats only `kind == "list"` as a collection, so a `Map` passes for a flag clap takes once. The fix I am naming but did not apply: treat every collection kind as a collection, or require the declared kind to be `list` or a scalar.
- `:164` reads only `get_long()`. In mirror run 3, a flag alias `--file` and a subcommand alias `v` both pass the gate undeclared, and `canon v --file X` parses.
- `AGENTS.md:46` says a flag taken more than once is a `List`. In mirror run 3, a `Count` flag (`--verbose --verbose`) declared `Optional<String>` passes, because `repeatable` (`:127`) does not count `Count` as repeated.

**5. Attacked and not broken:**
- These failed loudly: `List<Optional<String>>`, `Optional<Set<…>>` (ess refuses it), a wire differing only in case, and inputs `_at` and `a__t`.
- Inherited flags cannot collide: a leaf's own flag overwrites the inherited one with the same name.
- The duplicate-wire report holds whichever of the two commands sorts first.
- `Optional<Optional<String>>` and `Integer` pass, as the stated rule "the element type is not compared" allows.

**6. Written outside the worktree:** the paths in the header, under `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w1/ess-command-surface/`. Inside the worktree, `target/tmp/adversary2-ess-commands/` was also written.

```findings
- file: crates/canon-cli/tests/ess_commands_match.rs
  line: 235
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: ess_surface counts an input as a collection only when its kind is list, so a single-valued flag declared as Optional<Map<String, String>> or Map<String, String> passes the gate, contrary to AGENTS.md § ESS.
- file: crates/canon-cli/tests/ess_commands_match.rs
  line: 164
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: collect_leaves reads only get_long and subcommand names, so a visible flag alias (--file) or subcommand alias (canon v) that clap accepts passes the gate undeclared.
- file: AGENTS.md
  line: 46
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: AGENTS.md says a flag clap takes more than once is a List input, but repeatable() treats an ArgAction::Count flag as single, so a repeated --verbose declared Optional<String> passes.
```
