---
format: aep.planning-md/3
id: review-result:adversary-simulate-child-outcome-step-pass-2
kind: review-result
status: active
title: 'Adversary pass 2 on story:simulate-child-outcome-step: green, 2 notes on wording'
relations:
- reviews: story:simulate-child-outcome-step
revision: 1
---
Second and last adversary pass on the unit of story:simulate-child-outcome-step, after its correction (head 70eda28e; the adversary's case at b1b984d4). Recorded as the adversary returned it.

unit: story:simulate-child-outcome-step pass 2, `impl/simulate-child-outcome-step` at 70eda28e plus my test commit b1b984d4
verdict: green
cases: executed 1070→1071, red 0
origin: introduced 2, pre-existing 0, undecided 0
wrote-outside-worktree: `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w6/simulate-child-outcome-step/adversary-p2/` (enumerator, probes, logs, binary copy, `cmp3/`, `rw/`, `expiry/`, `bound/`, `mod.rs.pristine`)
needs-coordinator: no

I found nothing the unit has to fix before merging. The count rule (D1 with D5) held against my own enumerator of real cases, a hand check of every pinned expiry finding, the byte comparison and seven mutants. The two findings below are wording only.

**1. Diff stat** (b1b984d4, made as `b10x-bot[bot]`): `crates/canon/tests/adversary_simulate_child_outcome.rs | 46 +` and nothing else. The tree was clean afterwards.

Mutants were applied in place under my lease, as the brief allowed. A trap and each loop iteration put back the saved original. Afterwards `git diff --stat` on `mod.rs` was empty.

**2. Case added**
- `adversary_simulate_child_outcome.rs:309` `a_mark_at_two_or_more_contains_and_outweighs_a_mark_at_one`. It is green, and was green when run alone.
- It covers what my pass 1 cases `:214`/`:260` cannot: their protocols never raise a mark past one. Here `{started, sealed}` is stuck with the mark of `open` at one and at two-or-more. Only the state at one is printed, with `1 other stuck state contains it`.
- It turns red on m9 (the mark capped at one) and on m10 (two-or-more encoded as `10`).

**3. Suite** (ess 0.77.0 on PATH): `cargo test --workspace --no-fail-fast` gave 1071 passed, 0 failed, 3 ignored over 150 summaries, `EXIT 0`. Clippy with `-D warnings`: `EXIT 0`. `cargo fmt --check` on both crates: `EXIT 0`.

**4. Findings** (they cover b1b984d4)

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon-cli/src/lib.rs:154 | contract-drift | note | CONFIRMED | introduced | `--help`, `cli.yaml`, `cli.md`: the D3 sentence |
| ess/domains/cli.yaml | contract-drift | note | CONFIRMED | introduced | `--help`, `cli.yaml`: "an outcome other than open lowers it" |

**Finding 1: the D3 sentence warns about a case that does not arise.**
- A walk reaches the second record of an opening action only by a second step. That step is taken only where the action is admissible again, and admissibility is judged the same way a runner's taking is. So the walked path is a real two-taking path, and every state D3 points at is a real state.
- On `two-records`, my enumerator allowed one taking to record both kinds and capped takings at one. The printed `{booked, briefed, unlisted}` is still true: one taking, and its only child ends with an unlisted outcome.
- So I found no protocol where D3 prints a false finding, and none can exist under this argument. The implementor's sentence about `two-records` holds.
- What one-record steps really cost is the opposite direction, and it predates this unit (story:simulate-record-groups): where an action's guard closes on its first record, the walk never reaches the two records together, so an outcome that needs both is reported unreached.
- The sentence in the help, `cli.yaml` and the reference would make a protocol author discount findings that are true.

**Finding 2: "lowers it" is incomplete.** The help and `cli.yaml` say an outcome other than `open` lowers the count. From two-or-more the code also keeps it at two-or-more, as a second successor. The module docs state this correctly; the help and `cli.yaml` do not.

**5. Attacked and not broken**
- **My own enumerator** (`realwalk.py`: real takings, child openings and outcomes, every state judged by `canon evaluate`, at most 3 takings per opener). I ran it on 15 protocols: the 7 fixtures (`ended-once`, `either-once`, `twice`, `repeat`, `closed`, `ends-open`, `denied`) and 8 of my own (two openers sharing `case_outcome`, two openers that record nothing, an opener admissible for ever, `twice` with a licence, a form with `revision: current`, a form under `not`, children recording `open` twice, and `two-records` with multi-record takings).
  - 24 printed findings, all true.
  - No protocol with a real dead end exits 0.
  - The real-state counts equal the walk's state counts on the small fixtures.
- **Hand checks with the evaluator:** all 5 findings of `child-expiry-held` and both of `child-expiry` match a hand-built case at `--at`. The cycles at `opened 2 or more children` in `case-composition.yaml` and `child-outcome-revision.yaml` are real: the opener has no guard, so it can be taken a third time.
- **Two-or-more never returning to an exact count:** checked that the lowering never invents a dead end or hides one. Reaching mark two-or-more needs the opener admissible at mark one, and then a real case can always take it again.
- **The encoding:** `10` can never arise; only `marked()` writes the two bits.
- **Mutants:** 7 of 7 are killed by the unit's own tests:
  - m6: two-or-more only lowers
  - m7: two-or-more never lowers
  - m8: D5 removed
  - m9: mark capped at one
  - m10: two-or-more encoded as `10`
  - m11: `open` lowers the mark too
  - m12: D5 checks current records only
- **Bytes:**
  - The 110 protocols without `case_open` give the same output as the saved outputs of the wave-base-equivalent and pre-correction binaries. Pass 1's binaries had been removed, so I compared against the outputs pass 1 saved.
  - All 127 protocols give identical bytes over two runs.
  - Only the fixtures with `case_open` differ.
- **The bound:** 11 kinds with 1 opener walks 40960 states. With 2 openers it is refused with `reachable-state-bound`, stdout empty, exit 1.

**Answers to my pass 1 findings**
- `mod.rs:903` (a dead end missed): answered. `ended-once` now reports its true stuck state.
- `lib.rs:154` (the counting limit missing from the reference): answered. `either-once` prints only true findings, and the limit that remains (D3) is stated in the reference, though worded as in finding 1.
- `mod.rs:675`/`:685` (mutants surviving the unit's tests): answered. The unit's own tests now kill both.
- `mod.rs:636` (clippy 1.100) and the vocabulary in `child-outcome-revision.yaml`: both predate this unit; I did not recheck them.

**6. Paths written outside the worktree.** All under `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w6/simulate-child-outcome-step/adversary-p2/`:
- binary copy: `canon-new`
- scripts: `realwalk.py`, `mkprobes.py`, `patchscripts.py`, `expiry_check.py`, `mutate.py`, `mutants.sh`, `cmp3.sh`, `runbound.sh`
- output directories: `probe/`, `rw/`, `expiry/`, `bound/`, `cmp3/`
- `rw-*.txt`, `mod.rs.pristine`, `msg-1.txt`
- logs: `build.log`, `alone-lib.log`, `mutant-*.log`, `mutants.log`, `suite.log`, `clippy.log`, `fmt.log`

The lease is released, and `worktree discard-cache` freed 6993 MiB.

```findings
[{"file": "crates/canon-cli/src/lib.rs", "line": 154, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The D3 sentence says a state reached through a second child of a two-record taking may not be stuck in a real case, but the walk takes the second record only where the action is admissible again, so that state is a real two-taking state, and two-records' finding is true even with one taking."}, {"file": "ess/domains/cli.yaml", "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The help and cli.yaml say an outcome other than open lowers the count, while from two-or-more the code also keeps it at two-or-more as a second successor, which only the module docs state."}]
```
