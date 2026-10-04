---
format: aep.planning-md/3
id: review-result:adversary-w3-canon-conformance-runner-pass-1
kind: review-result
status: active
title: Wave 2026-10-04-w3 adversary, canon story:conformance-runner, pass 1
relations:
- reviews: story:conformance-runner
revision: 1
---
unit: canon/conformance-runner — working tree on ef4bcf7 (base 46dc424) plus uncommitted phase 2
verdict: NEEDS-CHANGE
cases: executed 127→137, red 1
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 5 paths (part 6)
needs-coordinator: none

The unit is green, but the suite can't tell good code from broken code in six places. I made six small breaking edits to a copy of the code ("mutants"). The unit's own suite stayed green on every one; my new cases catch all six. One of my cases is red on the current code.

**1. `git --no-pager diff --stat` (the worktree)**
```
 crates/canon-cli/src/main.rs    |  74 ++++-
 crates/canon/src/conform/mod.rs | 683 +++++++-
```
Both of those are the implementor's uncommitted phase 2, not mine. I only added two new, untracked test files:
- `crates/canon/tests/adversary_conform_steps.rs`
- `crates/canon-cli/tests/adversary_conform_registry.rs`

I touched no implementation file.

**2. Cases added (each run on its own first)**

| Case | Asserts | Now | Breaking edit it catches (unit suite stays green) |
|---|---|---|---|
| `adversary_conform_steps.rs:29` `a_failing_second_step_fails_the_scenario` | a passing step, then a step whose expected IR differs → `Failed{second, ...line 1}` | green | M1: `run` checks only the first step, so the scenario passes |
| `:46` `the_first_failing_step_is_reported_not_the_last` | a differing compile step, then an evaluate step → the first one is reported | green | — |
| `:63` `a_step_id_that_is_not_an_identifier_is_refused` | `id: 'a b'` is refused | green | M2: the step-id check is removed |
| `:76` `covers_naming_only_the_prefix_is_refused` | `covers: [CANON-]` is refused | **red** | — |
| `:89` `covers_with_whitespace_is_refused` | `'CANON-A B'` is refused | green | M3: the identifier check on `covers` is removed |
| `:100` `an_alias_bomb_in_evaluate_inputs_is_refused` | a 10^10 alias bomb in `case` → Err in under 10 s | green | — |
| `:128` `the_documented_example_scenario_parses` | the module doc's yaml example, read at run time, parses into steps `compile` and `no-evidence` | green | — |
| `adversary_conform_registry.rs:50` `only_yaml_files_of_the_directory_itself_are_read` | `.txt`, `.yml`, `.YAML`, `README`, `.yaml.bak` and `nested/inner.yaml` are all ignored → exact report, exit 0 | green | M4: `names.retain` is dropped |
| `:72` `a_missing_registry_directory_exits_unreadable` | stdout empty, stderr `error[unreadable]: `, exit 2 | green | M5: a missing directory exits 0 |
| `:86` `a_scenario_file_with_a_byte_order_mark_passes` | a scenario file with a BOM still passes | green | M6: `read_text` no longer strips the BOM |

The red output, captured before anything else ran (line 76 then; line 79 after rustfmt):
```
thread 'covers_naming_only_the_prefix_is_refused' panicked at crates/canon/tests/adversary_conform_steps.rs:76:5:
`covers: [CANON-]` parsed: Ok(["CANON-"])
test result: FAILED. 6 passed; 1 failed
```

**3. Full suite, run after the cases existed**

`cargo test --workspace --locked --no-fail-fast` → EXIT=101.
- 137 tests ran, 136 passed. The only failure is `covers_naming_only_the_prefix_is_refused`.
- The implementor's `phase2-gate.log` sums to 127 executed, so my 10 cases account for the difference.
- `cargo fmt --all --check` → 0, after I ran rustfmt on my two files.
- `cargo clippy -p b10x-canon -p canon-cli --all-targets -- -D warnings` → clean.

Mutant runs: I used a copy under `scratch/adv1/mutant` with its own build directory, `canon-w3-conform-adv1-mutant`. Both are deleted.

**4. Findings** (all on the working tree above)

| # | file:line | Verdict / origin | What was measured | What reaches it |
|---|---|---|---|---|
| F1 | `crates/canon/src/conform/mod.rs:241` | NEEDS-CHANGE / introduced | `covers: [CANON-]` is accepted, although it names no requirement | any scenario file; story:conformance-suite will count coverage from `covers`. Fix: require text after the prefix |
| F2 | `crates/canon/src/conform/mod.rs:354` | CONFIRMED / introduced | Every scenario the unit runs has one step, so M1 survives. `mod.rs:657` is named "...after the steps before it" but has no step before it | each scenario that has more than one step. My test `:29` closes the gap |
| F3 | `crates/canon-cli/src/main.rs:100,87,74` and `mod.rs:263,241` | CONFIRMED / introduced | M2–M6 survive the unit's suite | the file filter, the missing-directory path, the BOM strip, the step-id check and the `covers` identifier check. My tests close all five |
| F4 | `crates/canon/src/conform/mod.rs:62` | CONFIRMED / introduced | The doc gives exits 0/1 only and says everything goes to stdout. A missing registry directory exits 2 with its message on stderr — including plain `canon conform run` in the repo root today, because `conformance/scenarios/` does not exist | default `--scenarios` (`main.rs:55`). Fix: one line in the doc, and possibly create the empty registry directory |
| F5 | `crates/canon/src/conform/mod.rs:62` | CONFIRMED / introduced | An empty registry prints `conform: 0 passed, 0 failed, 0 unreadable` and exits 0 — it passes without running anything | judgement call; story:conformance-suite may cover this through coverage |
| F6 | `crates/canon/src/conform/mod.rs:32` (plus `main.rs:114`, `mod.rs:386`) | INFEASIBLE / introduced | `fixture:` accepts absolute and `..` paths. A probe with `fixture: <abs>/secret.txt` printed `fixture does not parse: invalid type: string "https://user:s3cr3t-token@example.org"` to stdout | a checked-in scenario file run in CI. Nothing runs `canon conform run` in CI yet, so I found nothing that reaches this |

**5. What I attacked and could not break**
- Sort order: entries come off disk unsorted (`ls -U` gives passes, not-a-scenario, differs), and the acceptance test catches a missing sort.
- Alias bomb: refused quickly.
- Byte-for-byte compare: `actual == expected` with a trailing-newline check; I found no way for a normalisation to hide a real difference.
- No IO in the library: `run_registry` takes a closure.
- A directory named `x.yaml`: reported unreadable, exit 1.
- Empty `steps`, or a step with no expectation: refused.
- Evaluate steps: always fail as "not supported yet".
- A reused scenario id: reported unreadable.
- The doc's own example scenario: parses.

**6. Paths written outside the worktree**
- `~/.cache/ga-wave-2026-10-04-w3/canon-conformance-runner/scratch/adv1/` — contains `reg/`, `empty/`, `secret.txt` (a fake token) and `suite.log`
- `~/.cache/b10x-target/canon-w3-conformance-runner/tmp/adversary-conform-bom`
- `~/.cache/b10x-target/canon-w3-conformance-runner/tmp/adversary-conform-filter`
- `~/.cache/b10x-target/canon-w3-conformance-runner/tmp/adversary-conform-missing`
- The test artefacts in the shared brief build directory `~/.cache/b10x-target/canon-w3-conformance-runner`

I also took and released a worktree lease (`adversary-w3-conform-p1`).

**7. Findings block**
```findings
- file: crates/canon/src/conform/mod.rs
  line: 241
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "covers accepts the bare prefix `CANON-`, which names no requirement; adversary_conform_steps.rs:76 is red"
- file: crates/canon/src/conform/mod.rs
  line: 354
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "no unit test runs a scenario with more than one step, so evaluating only the first step stays green; the test at mod.rs:657 has no step before its evaluate step despite its name"
- file: crates/canon-cli/src/main.rs
  line: 100
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "removing the yaml filter, exiting 0 on a missing directory (:87), dropping the BOM strip (:74), the step-id check (mod.rs:263) or the covers identifier check (mod.rs:241) all leave the unit suite green"
- file: crates/canon/src/conform/mod.rs
  line: 62
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the report doc gives exits 0/1 only and all output on stdout, but a missing registry directory, including the default conformance/scenarios which does not exist, exits 2 with stderr only"
- file: crates/canon/src/conform/mod.rs
  line: 62
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "an empty registry reports 0 passed and exits 0, a vacuous pass"
- file: crates/canon/src/conform/mod.rs
  line: 32
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "fixture accepts absolute and .. paths and a non-protocol fixture's content is echoed into the report; no CI caller of canon conform run exists yet"
```
