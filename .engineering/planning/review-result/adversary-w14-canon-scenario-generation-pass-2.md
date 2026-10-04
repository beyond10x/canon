---
format: aep.planning-md/3
id: review-result:adversary-w14-canon-scenario-generation-pass-2
kind: review-result
status: active
title: Wave 2026-10-04-w14 adversary, canon story:scenario-generation, pass 2
relations:
- reviews: story:scenario-generation
revision: 1
---
unit: canon/scenario-generation, working tree = f5c070f + phase 2 + pass-1 fixes (uncommitted)
verdict: CONFIRMED (8 red cases: 2 warnings, 6 notes; no blocker)
cases: executed 496→508, red 8
origin: introduced 9 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 path kept (part 6); the scratch copy, its target dir and the probe dirs are deleted
needs-coordinator: yes. The staging design breaks a pre-created `--out` (findings 1 and 2), and probably a mounted one too (finding 9, not measured). Is that acceptable, or should an existing empty `--out` be written into directly?

**1. Diff and scope**

`git --no-pager diff --stat` shows the same 10 tracked implementor files as before (88 insertions, 22 deletions). I changed none of them. I added one untracked file: `crates/canon-cli/tests/adversary2_scenario_generation.rs`. I touched no non-test path, ran no `git add` and wrote nothing to `.engineering/`.

**2. Cases** (each run alone before the suite; logs are `scratch/adversary-p2/alone-*.log`)

| line | case | now | red output, verbatim (trimmed) |
|---|---|---|---|
| 137 | `an_existing_empty_out_under_a_parent_that_is_not_writable_is_written` | RED | `error[unreadable]: shared/.out.canon-generate-1547324: Permission denied (os error 13)` / `left: Some(2) right: Some(0)` |
| 167 | `an_existing_empty_out_keeps_its_permissions` | RED | `the existing empty --out, mode 0700, was replaced by a new directory of mode 755` |
| 188 | `an_out_name_within_the_file_name_limit_is_written` (240-byte `--out`) | RED | `error[unreadable]: ./.ooo…ooo.canon-generate-1547666: File name too long (os error 36)` |
| 207 | `an_out_written_with_a_trailing_dot_component_is_written` | RED | `error[unreadable]: out/.: Invalid argument (os error 22)` |
| 222 | `a_failed_write_leaves_no_directory_it_created` | RED | `left: ["new", "p.yaml"] right: ["p.yaml"]` |
| 245 | `an_out_that_is_a_symbolic_link_to_an_empty_directory_is_written_through` | RED | `error[unreadable]: out: Not a directory (os error 20)` |
| 349 | `identifiers_equal_under_unicode_case_folding_are_refused_as_a_file_name_collision` | RED | `"s" and "ſ": exit Some(0)`, `"σ" and "ς": exit Some(0)`, `"μ" and "µ": exit Some(0)` |
| 385 | `a_capital_letter_and_its_final_lower_case_form_are_refused_as_a_file_name_collision` | RED | ``outcomes `Σ` and `ς` were not refused as file-name-collision: exit Some(0)`` |
| 262 | `no_failure_removes_a_symbolic_link_out_or_what_it_points_to` | green | kills mutant M1 (below) |
| 296 | `a_symbolic_link_out_of_the_working_directory_in_a_parent_component_is_refused` | green | guard |
| 313 | `a_working_directory_that_is_a_symbolic_link_generates_scenarios_conform_run_passes` | green | guard |
| 373 | `the_kelvin_sign_and_the_capital_sharp_s_are_refused_as_a_file_name_collision` (plus `Go`/`gO`) | green | guard |

**Mutants and a fix probe.** Both ran in the scratch copy `scratch/adversary-p2/m1` with target dir `m1-target`, and both are deleted.
- **M1:** removed `let _ = std::fs::remove_dir_all(&staging);` at `generate.rs:155`. Every existing target in `-p canon-cli -p b10x-canon` stayed green; only `no_failure_removes…` went red (`left: [".out.canon-generate-1551366", "out", "p.yaml", "real"]`). Before this case, nothing guarded the cleanup of the staging directory.
- **Fix probe for the fold:**
  - `fold` as `….to_uppercase().to_lowercase()` turns the Kelvin/ẞ guard red, because `ß→SS→ss` while `ẞ→ẞ→ß`.
  - `….to_lowercase().to_uppercase().to_lowercase()` turns both fold cases green, keeps every existing test green, and gives `passed=483 failed=6`. The 6 are the staging cases.

**3. Gate** (unit build dir, run after the cases existed; logs in `scratch/adversary-p2/gate/`)

| command | exit | summary line |
|---|---|---|
| `cargo fmt --all --check` | 1, then 0 | the first run flagged only my file (5 diffs, 0 elsewhere); I ran `rustfmt` on that file alone |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized] target(s) in 0.07s` |
| `cargo test --workspace --locked` | 101 | `test result: FAILED. 4 passed; 8 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s` / `error: test failed, to rerun pass \`-p canon-cli --test adversary2_scenario_generation\`` |
| the same with `--no-fail-fast` | 101 | 85 result lines, passed=500 failed=8; `error: 1 target failed: \`-p canon-cli --test adversary2_scenario_generation\`` |
| `ess specify validate … && ess verify conform synthesize …` | 0 | `canon v1 — 3 file(s), valid` / `0 scenario(s) (0 authored), 0 refusal(s), written to …/suite.json` |
| `cargo run -q --locked -p canon-docs -- generate --check` | 0 | `canon-docs: 15 generated files are current` |
| `cargo run -q --locked -p canon-cli -- conform run` | 0 | `conform: 9 passed, 0 failed, 0 unreadable` |

`-- --list` shows all 12 new cases in this tree. The `canon` binary (11:25:16) is newer than every source it was built from.

**4. Findings, with what reaches each**

1. **Read-only parent** (`generate.rs:136`, warning). A pre-created, empty, writable `--out` is refused when its parent cannot be written, because the staging directory goes beside it. The error also names a hidden path the user never gave. Reach: the `--out` help, "absent or empty". Nothing in the repository shows anyone doing this; a container volume such as `/out` run as a non-root user would hit it.
2. **Permissions lost** (`generate.rs:144–146`, note). An existing `--out` is replaced, not written into. Its mode (0700 became 0755), owner, group and inode are lost. On Unix, `rename` onto an empty directory is atomic, so the `remove_dir` is only needed on Windows, and that is where the window opens.
3. **Long `--out` name** (`generate.rs:132–136`, note). An `--out` name of 237 bytes or more is legal but fails. The staging name adds `.` before it and `.canon-generate-<pid>` after it.
4. **Trailing dot** (`generate.rs:117`, `144`, note). `--out out/.` passes `file_name()`, then `rmdir("out/.")` fails with EINVAL.
5. **Leftover parents** (`generate.rs:131`, note). `create_dir_all(parent)` is not undone when generation fails. The docs say "appears whole or not at all".
6. **Symlink `--out`** (`generate.rs:144`, note). A symbolic link to an empty directory fails with ENOTDIR (`rmdir` on the link). Nothing is deleted (case 262).
7. **Case folding** (`generate/mod.rs:158–159`, note). `fold` uses `to_lowercase`, which does not equate `Σ`/`ς`, `ſ`/`s` or `µ`/`μ`. Reach: nothing found. Identifiers like these are constructed, so the verdict is INFEASIBLE, but the refusal's doc overclaims. The fix: `to_lowercase().to_uppercase().to_lowercase()`, probed above.
8. **Mutant M1** (`generate.rs:155`, note). The existing suite missed it; case 262 now kills it.
9. **Mountpoint `--out`** (`generate.rs:144`, judgement, not measured, because mounts were ruled out). If `--out` is a mountpoint, `rmdir` gives EBUSY, and a rename across devices would give EXDEV. That makes the common container pattern `-v $PWD/out:/out` fail.

**Verified on Linux vs not:**
- Run on Linux, not refused (exit 0): `ß`/`ss`, `ı`/`i`, `İ`/`i`, `Σ`/`ς`.
- Run on Linux, refused: `ǅ`/`ǆ`, Kelvin/`k`, `ẞ`/`ß`.
- Not verified: whether APFS folds `ß` to `ss`. NTFS colliding `ı`/`i` and `Σ`/`ς`, and keeping `ẞ`/`ß` apart (so refusing that pair over-refuses, harmlessly), is inferred from UnicodeData and not run.

**5. Attacked, not broken**
- **Existing staging directory:** `create_dir` fails with AlreadyExists and returns before the cleanup closure, so the existing directory is untouched. I read this but could not run it, because a child's pid cannot be chosen in advance.
- **`--out` of `.`, `..`, `/`, `x/..`:** always refused as `out-not-empty`, since the confined protocol lies under the working directory.
- **Deletion:** no failure deletes anything but our own staging directory or an empty `--out` (`remove_dir` only removes empty directories). A symbolic-link `--out` and its target survive (case 262).
- **The remove-then-rename window:** a file added in the window makes `rename` fail with ENOTEMPTY and nothing is deleted. The empty directory is recreated, but with its permissions lost (finding 2).
- **Confinement:** a link in a parent component pointing outside is refused. A working directory that is itself a symbolic link works end to end, because generate and `conform run` use the same canonicalize logic.
- **Dependency:**
  - `unicode-normalization` 0.1.25 is `MIT OR Apache-2.0`. `default-features = false` only drops the empty `std` feature.
  - `tinyvec` 1.13.3 is `Zlib OR Apache-2.0 OR MIT`.
  - `generate/` contains no `std::fs`, `std::io`, `std::env`, `std::net` or clock use, so the library stays IO-free.
- **Docs:** library refusals and `out-not-empty` happen before any IO, so "Refused, writing nothing" holds. Exit-code row 2 matches.

**6. Written outside the worktree**
- `~/.cache/ga-wave-2026-10-04-w14/canon-scenario-generation/scratch/adversary-p2/`, kept: 12 `alone-*.log` files and `gate/`.
- Deleted: the scratch copy `scratch/adversary-p2/m1`, its target dir `m1-target`, and the 20 test dirs `…/b10x-target/canon-w14-scenario-generation/tmp/adversary2-generate-*` (rerunning the tests recreates them).
- The gate wrote `…/canon-w14-scenario-generation/suite.json` (the gate's own output).

```findings
- file: crates/canon-cli/src/generate.rs
  line: 136
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "an existing empty writable --out whose parent is not writable exits 2, because the staging directory must be created beside it"
- file: crates/canon-cli/src/generate.rs
  line: 144
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "an existing empty --out is removed and replaced by the staging directory, losing its mode (0700 became 0755), owner and inode"
- file: crates/canon-cli/src/generate.rs
  line: 132
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "an --out name of 237 bytes or more is legal but fails ENAMETOOLONG once the staging prefix and suffix are added"
- file: crates/canon-cli/src/generate.rs
  line: 117
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "--out out/. naming an existing empty directory exits 2 with EINVAL from rmdir"
- file: crates/canon-cli/src/generate.rs
  line: 131
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "a failed generation leaves the parent directories create_dir_all made for --out, against 'appears whole or not at all'"
- file: crates/canon-cli/src/generate.rs
  line: 144
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "an --out that is a symbolic link to an empty directory exits 2 with ENOTDIR from rmdir on the link"
- file: crates/canon/src/generate/mod.rs
  line: 159
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "fold lowercases instead of case-folding, so Σ/ς, s/ſ and μ/µ are not refused as file-name-collision although the refusal claims to catch every case-insensitive collision; no protocol using such identifiers was found"
- file: crates/canon-cli/src/generate.rs
  line: 155
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "removing the staging-directory cleanup left every pre-existing test green; no_failure_removes_a_symbolic_link_out_or_what_it_points_to now kills it"
- file: crates/canon-cli/src/generate.rs
  line: 144
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "an --out that is a mountpoint (container volume) would fail with EBUSY on rmdir; not measured because mounts were out of bounds"
```
