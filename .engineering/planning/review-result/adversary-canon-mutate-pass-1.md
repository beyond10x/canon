---
format: aep.planning-md/3
id: review-result:adversary-canon-mutate-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on canon mutate: red by one case'
relations:
- reviews: story:canon-mutate
revision: 1
---
Adversary pass 1 on the unit of story:canon-mutate, branch impl/canon-mutate at a1cf482a, run on Opus on 2026-11-21. The report follows verbatim.

unit: story:canon-mutate, branch impl/canon-mutate, findings cover a1cf482a; the cases are committed at d43716d1
verdict: red
cases: executed 1251→1253, red 1
origin: introduced 2, pre-existing 0, undecided 0
wrote-outside-worktree: 1 directory, $HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w9/canon-mutate/adversary-p1/
needs-coordinator: yes. Should `canon mutate` stop handing the mutant in where a scenario reads the protocol only as an import?

**1. What I changed** (`git diff --stat a1cf482a d43716d1`): only test files and test fixtures. No implementation file changed.
```
 crates/canon-cli/tests/adversary_mutate.rs                  | 116 +++
 fixtures/mutate-adversary/import/consumer.yaml              |  17 +
 fixtures/mutate-adversary/import/scenarios/consumer.yaml    |  32 +
 fixtures/mutate-adversary/import/scenarios/shared.yaml      |  27 +
 fixtures/mutate-adversary/import/shared.yaml                |  43 +
```
Commit d43716d1, made as the bot: `test: adversary pass 1 on canon mutate`. The tree is clean and the lease is released.

To test the unit's tests I swapped mutated copies into `crates/canon/src/mutate/mod.rs` and `crates/canon-cli/src/mutate.rs`, then restored each with `git checkout --`. The brief asked for this. The skill's rule forbids editing a file under attack even briefly; I held the lease the whole time.

**2. Cases added** (`crates/canon-cli/tests/adversary_mutate.rs`)
- **`a_mutant_handed_in_for_a_pinned_import_is_not_killed_by_the_digest` — red.** It rests on the story's § Outcome: the mutant goes in for "the fixture whose path is the `--path` file, and every other fixture unchanged".
  - Setup: `shared.yaml` is the fixture of one scenario. `consumer.yaml` imports it, pinned by digest.
  - Expected: 3 weak points, exit 1.
  - Output when the case was written, run alone:
    `left: (Some(0), "killed: drop-conjunct claim `item.ready` true_when all[0]: RELAXATION (weakening); scenario `adversary-consumer-both-checks` fails\n…mutate: 3 mutants, 3 killed, 0 survived (0 weak points), 0 invalid\n", "")`
  - Why the consumer scenario fails: I ran it by hand against the mutant. It stops at the compile step with `import-digest-mismatch`. It never reaches the step that checks behaviour.
- **`mutate_reports_a_mutant_that_does_not_read_back_as_unwritable` — green.** It pins the read-back guard (claim 1).
  - Setup: a protocol with a 1023-byte artifact identifier; its mutants are written with a 1025-byte quoted key, which the reader drops.
  - It passes on the real code. On the copy with the guard removed it fails: `left: (Some(0), "killed: drop-conjunct claim `c` true_when all[0]: …; scenario `A` fails…`.

**3. Full suite**, run after both cases existed: `cargo test --workspace --no-fail-fast` gave 1252 passed, 1 failed, 3 ignored over 166 summaries, `suite exit 101`. The only failure is `a_mutant_handed_in_for_a_pinned_import_is_not_killed_by_the_digest`. My new fixtures break none of the tests that walk every fixture. `rustfmt --check` and `cargo clippy -p canon-cli --tests` both exit 0.

**4. Findings**

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon/src/mutate/mod.rs:344 | contract-drift | warning | INFEASIBLE | introduced | A scenario set where the `--path` protocol is one scenario's fixture and another fixture's import. Nothing in `conformance/scenarios/` has that shape: no imported document is a fixture. I built the case. |
| crates/canon/src/mutate/mod.rs:415 | mutant | note | CONFIRMED | introduced | With the read-back guard removed, the unit's tests stayed green. The new green case now covers it. A protocol needs an identifier of 1023 bytes or more to reach the guard. |

- **First finding, in short:** imports are pinned by the digest of the compiled IR, and the pin is required (`model/mod.rs:168`). So when the mutant is also handed in for an import, the importing scenario fails on the pin for every mutant that changes the IR. Weak points are hidden and the run exits 0.
- **Fix to name, not applied:** read imports unchanged, as the story's sentence says, or skip scenarios that read the protocol only as an import.

**Mutants checked by hand**: 15, all equal to what `canon mutate` printed. For each I wrote the mutant, ran `canon conform run` on copies of its scenarios, and ran `canon diff` from protocol to mutant.

| fixture | mutant | `canon mutate` | mine | equal |
|---|---|---|---|---|
| mutate/protocol.yaml lacking | drop all[1] | weak point, RELAXATION | 4 passed; RELAXATION | yes |
| mutate/protocol.yaml complete | drop all[1] / all[0] | killed by observation-only / falsification-only, RELAXATION | same failing scenarios; RELAXATION | yes |
| mutate/protocol.yaml lacking | all-to-any, remove-capability, both change-result | killed (falsification-only, no-authority, falsification-only ×2); RELAXATION, RELAXATION, BREAKING ×2 | same first failure and categories | yes |
| action-admissibility | drop all[0], drop all[1], all-to-any | weak point ×3, RELAXATION | 1 passed; RELAXATION | yes |
| action-admissibility | remove-precondition, remove-capability | killed by CANON-AUTHORITY-001, RELAXATION | CANON-AUTHORITY-001 fails; RELAXATION | yes |
| decision-outcomes | drop all[0], drop all[1], all-to-any | weak point ×3, RELAXATION | 1 passed; RELAXATION | yes |

**5. Attacked and not broken**
- **Sites (claim 2):** I wrote a protocol with every site shape in the brief and counted 37 mutants by hand; the output has 37, in the documented order, under the bound. A second protocol with an order form, a `children` form and `independent_on` gave the 7 I expected, with nothing inside those forms mutated.
- **`NO SEMANTIC CHANGE` on a dropped conjunct:** this looked wrong but is right. The conjunct is implied by a sibling, and `canon diff` agrees.
- **Probe mutations:** 12 run.
  - 10 turned the unit's tests red: each of the five operators skipped, killed and survived swapped, weak point taken from `TIGHTENING`, baseline check removed, mutant order reversed, exit 0 on a weak point.
  - The read-back guard removal stayed green (finding 2 above).
  - First-to-last import scenario stayed green, but it cannot change any result: compose refuses an unused or missing import and the digest pins the content.
- **Edges:** 17 cases. `--path` relative, with `./`, absolute or through a symbolic link gives exit 0. Missing path, directory, unreadable scenario file, subdirectory named `*.yaml`, or `--scenarios` that is a file gives exit 2. Empty scenario directory, a file that is no scenario, an empty or IR `--path`, or another working directory gives exit 1 with nothing on stdout. Usage errors give 64.
- **Path handling and the library:** the copied `read_fixture` is the same as the one in `conform.rs`, line for line. Scenario files are read in sorted order. Two runs gave the same bytes. The library reads no file, clock or environment.
- **Expected outputs and claim 8:** the three `.txt` files are exactly what the binary prints. In `lib.rs`, `main.rs` and `cli.yaml`, every changed line belongs to mutate, and the floor's wording is kept.
- **§ Acceptance:** holds as written. The weakening is in the claim the outcome reads, because the diff classifies any change to an outcome's own requirement as `BREAKING` (`diff/mod.rs:53`), so such a mutant can never be a weak point.
- **Stale `# read from: lib.rs:<line>` comments in `cli.yaml`:** they were already wrong at aa8d8dd2. The mutate change moves them further. Not a finding.

**6. Written outside the worktree:** `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w9/canon-mutate/adversary-p1/`, holding the scripts, the mutated copies (`mutants/`), the probe protocols (`probe/`, `hand/`), the logs and `msg-1.txt`.

```findings
[{"file": "crates/canon/src/mutate/mod.rs", "line": 344, "category": "contract-drift", "severity": "warning", "verdict": "INFEASIBLE", "origin": "introduced", "message": "Handing the mutant in where a scenario reads the protocol as an import makes that scenario fail on import-digest-mismatch for every mutant that changes the IR, so weakening mutants are reported killed and the run exits 0 where the story's fixture-only substitution gives weak points and exit 1."}, {"file": "crates/canon/src/mutate/mod.rs", "line": 415, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Removing the read-back guard left the unit's tests green; the adversary case mutate_reports_a_mutant_that_does_not_read_back_as_unwritable now fails on that mutant."}]
```
