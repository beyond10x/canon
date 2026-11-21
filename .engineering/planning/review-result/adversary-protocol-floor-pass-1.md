---
format: aep.planning-md/3
id: review-result:adversary-protocol-floor-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on the unit of story:protocol-floor: red by three surviving mutants, no defect in the code'
relations:
- reviews: story:protocol-floor
revision: 1
---
Adversary pass 1 on the unit of story:protocol-floor (wave 2026-11-20-w9), run on Opus against `9738a3c8`, recorded as returned.

unit: story:protocol-floor, impl/protocol-floor at 9738a3c8 (findings); adversary commit 4df0f19a
verdict: red: 3 mutants survive the unit's suite; the code itself matched the classifier everywhere I probed
cases: executed 1237→1240 (163→164 test summaries), red 0 against the implementation (each new case fails on one surviving mutant)
origin: introduced 3, pre-existing 0, undecided 0
wrote-outside-worktree: 1 directory, `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w9/protocol-floor/adversary-p1/` (probe/, mut/, gate and case logs, msg-1.txt)
needs-coordinator: no

**1. Diff stat** (`git diff --stat 9738a3c8 HEAD`): `crates/canon-cli/tests/adversary_floor.rs | 272 +`. It is a test file only; no fixture was added.

**2. Cases added** in `crates/canon-cli/tests/adversary_floor.rs`. All three are green on 9738a3c8. The protocols are built from the unit's floor fixture inside each test. Each verdict is also checked against `canon diff` for the same two documents with the protocol given the floor's id.
- `a_protocol_that_only_rewrites_descriptions_holds_and_counts_them`: exit 0, prints `NO SEMANTIC CHANGE: 2`.
- `each_count_is_the_classifiers_number_of_that_category`: exit 0 with counts 2, 1, 1.
- `several_changes_below_the_floor_print_in_the_classifiers_order`: 4 lines below the floor across 3 sections, in the classifier's order. With `--floor` and `--ir` swapped, only the `BREAKING` line is printed.
- The first run of the cases alone had a mistake of mine (an obligation that tested evidence, refused as `evidence-in-discharge`). I fixed it and the second run gave 3 passed.

**3. Suite run** (`gate.sh`, the unit's 5 steps), after the cases existed:
- ess-gate 0, clippy 0, tests 0 (164 summaries, 1240 passed, 0 failed, 3 ignored), conform 0 (28 passed).
- fmt exited 1 on my own file. I ran `cargo fmt -p canon-cli`; the check then exited 0 and only my file had changed.

**Mutants.** Each was swapped in from a copy in scratch and restored with `git checkout --`. The tree was clean afterwards apart from my file. The aep skill forbids mutating a file under attack even briefly; I followed the brief's swap instruction instead.

| mutant | unit's `floor.rs` | adversary case |
|---|---|---|
| header not shared | red, `floor.rs:78`, `different-protocol` | red |
| header: only the id shared | red, `floor.rs:86`, `NO SEMANTIC CHANGE: 1` | green |
| `BREAKING` admitted | red, `floor.rs:78`, changes-outcome exits 0 | red |
| `RELAXATION` admitted | red, `floor.rs:78`, drops-capability exits 0 | red |
| `EXPANSION` refused | red, `floor.rs:78`, composed exits 1 | red |
| TIGHTENING and EXPANSION counts swapped | red, `floor.rs:86` | red |
| exit 0 on a breached floor | red, `floor.rs:78` | red |
| lines below the floor sorted by category, then id, descending | **green** | red, `adversary_floor.rs:253`, BREAKING printed first |
| `NO SEMANTIC CHANGE` refused | **green** | red, `:142`, exits 1 with `floor: breached` |
| `NO SEMANTIC CHANGE` counted as `EXPANSION` | **green** | red, `:142`, `EXPANSION: 2` |

**4. Findings** (they cover 9738a3c8)

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon-cli/tests/floor.rs:159 | mutant | warning | NEEDS-CHANGE | introduced | Any protocol that rewrites a description of the floor. No unit fixture has a `NO SEMANTIC CHANGE` change, so § Outcome's "exits 0 when every change is ... `NO SEMANTIC CHANGE`" is unchecked. Fix: merge `adversary_floor.rs` or add an equivalent fixture. |
| crates/canon-cli/tests/floor.rs:195 | mutant | warning | NEEDS-CHANGE | introduced | Any protocol with two or more changes below the floor. Each breached fixture prints one line, so "in the classifier's deterministic order" is unchecked. |
| crates/canon-cli/tests/floor.rs:170 | mutant | note | CONFIRMED | introduced | The `NO SEMANTIC CHANGE` count is 0 in every pinned output, so counting it as an `EXPANSION` stays green. |

**Forms compared with the classifier** (`canon floor` against `canon diff` with one shared id, exact bytes): all **35 equal**.

| form (classifier's category) | floor exit |
|---|---|
| `records_together` removed (NSC); `revises` added (REL) / removed (TIG); `decided_by` to judgement (TIG); `author` to judgement (TIG×2) | 0 / 1 / 0 / 0 / 0 |
| `supplied_by` removed or changed (BRK); `max_age` longer (REL), shorter (TIG), same length (NSC), removed (REL) | 1 / 1, 0, 0, 1 |
| `independent_on` dimension added (TIG) / removed (REL); dimension declared (EXP) | 0 / 1; 0 |
| arguments widened (REL) / removed (BRK); `case_open` changed (BRK); `children` changed (BRK); `before` changed (BRK) | 1 / 1; 1; 1; 1 |
| input value added (EXP); input added (BRK); invalidation rule removed (REL), claim added (TIG), upstream changed (BRK) | 0; 1; 1, 0, 1 |
| scope changed (REL+TIG); capability added (TIG); `applicable_when` removed (TIG) / changed (BRK) | 1; 0; 0 / 1 |
| outcome added (EXP) / changed (BRK); evidence kind added (EXP); artifact added (BRK); obligation added (TIG); claim description (NSC); 10 changes at once | 0 / 1; 0; 1; 0; 0; 1 |

`ground_evidence` and decision outcomes are case records, not IR declarations, so neither the floor nor `canon diff` compares them.

**5. Attacked and not broken**
- Claim 2: the IR header today holds only `id`, `revision` and `description` (`ir/mod.rs:166`), and the classifier ignores the id and revision (`diff/mod.rs:743`). Sharing the header hides nothing semantic. A floor compiled from imports also works: 3 runs.
- Direction: the swapped pairs give mirrored verdicts, and both the floor against itself and the 10-change protocol against itself hold with three zeros: 5 runs.
- Inputs: an empty file, valid JSON that is no IR, broken JSON, a `protocol/1` source, `canon-ir/2`, a directory, a missing file (exit 2) and clap misuse (exit 64) all refuse with nothing on stdout. The import fixture compiled without `--import` is `missing-import`: 13 runs.
- Claim 6: two runs give identical bytes.
- Claim 7: every changed line in `lib.rs`, `cli.yaml` and `cli.md` at `738f895a`..HEAD belongs to the floor.
- The unit's changes to `adversary_ess_commands.rs` and `adversary_scenario_generation.rs` are not weakened. The first renames the stand-in command from `Floor` to `Unclaimed`; the second accepts only `MissingImport` for `floor/`.
- Other tests that read the fixtures:
  - `validate::fixture_protocols` and `protocol_write::corpus` parse all 7.
  - `check/tests.rs` `fixture_protocols` silently skips the 3 that need an import.
  - The generation test expects those 3 to be refused with `MissingImport`.
  - None of them composes an import fixture.

**6. Paths written outside the worktree:** `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w9/protocol-floor/adversary-p1/`, holding `probe/`, `mut/`, `gate.sh`, `gate{1..5}.log`, `gate.status`, `build0.*`, `case{1,2}.*` and `msg-1.txt`. Lease `adv1-protocol-floor` is released.

```findings
[{"file": "crates/canon-cli/tests/floor.rs", "line": 159, "category": "mutant", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "No fixture makes a NO SEMANTIC CHANGE change, so a floor that refuses one (exit 1, floor: breached) keeps the unit's suite green."}, {"file": "crates/canon-cli/tests/floor.rs", "line": 195, "category": "mutant", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "Every breached fixture prints a single line, so lines below the floor printed out of the classifier's order keep the unit's suite green."}, {"file": "crates/canon-cli/tests/floor.rs", "line": 170, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Every pinned holds output has a NO SEMANTIC CHANGE count of 0, so counting such changes as EXPANSION keeps the unit's suite green."}]
```
