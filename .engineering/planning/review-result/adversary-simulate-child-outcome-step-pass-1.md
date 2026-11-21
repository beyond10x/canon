---
format: aep.planning-md/3
id: review-result:adversary-simulate-child-outcome-step-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:simulate-child-outcome-step: red, 1 blocker, 1 warning, 4 notes'
relations:
- reviews: story:simulate-child-outcome-step
revision: 1
---
First adversary pass on the unit of story:simulate-child-outcome-step (head b9f31802; the adversary's cases at 65a963a0). Recorded as the adversary returned it.

unit: story:simulate-child-outcome-step, `impl/simulate-child-outcome-step` at b9f31802, plus my test commit 65a963a0
verdict: red
cases: executed 1062→1067, red 3
origin: introduced 4, pre-existing 2, undecided 0
wrote-outside-worktree: `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w6/simulate-child-outcome-step/adversary-p1/` (logs, probes, scripts, binary copies, `cmpall/`, `mod.rs.pristine`)
needs-coordinator: yes. Finding 1 needs option A's rule changed: a child-outcome step must not come from a child that has already ended. The story's "Why both claims still hold" does not hold. Also: `canon-base` and `canon-new` were not in the scratch directory.

**1. Diff stat** (commit 65a963a0, as `b10x-bot[bot]`; afterwards `git status` was clean):
```
 crates/canon-cli/tests/adversary_simulate_child_outcome_cli.rs | 46
 crates/canon/tests/adversary_simulate_child_outcome.rs         | 277
 fixtures/child-outcome-adversary/either-once/protocol.yaml     | 52
 fixtures/child-outcome-adversary/ended-once/protocol.yaml      | 74
```
All four paths are test files or fixtures. Two charter conflicts I am declaring:
- **Mutants were applied in place.** The brief asked for it. The adversary charter forbids it. I held the lease, restored the file from a saved copy, and `git status` was clean afterwards.
- **Base binary.** It is a copy of `canon-release-prep/target/debug/canon` (f9504cfe). That is 4c35a9e plus the clippy loop fix and the version bump, so it stands in for `canon-base`. `canon-new` is my own release build of b9f31802.

**2. Cases added.** Each was red or green as stated when first run alone, before any suite run.
- `adversary_simulate_child_outcome.rs:86` `a_dead_end_after_the_only_child_ended_is_reported` (**red**).
  - Fixture `ended-once`: `open` runs once, the only child ends `accepted`, and capability `licence` (needed by `publish`) is denied.
  - The evaluator gives every action and every outcome `blocked`, and refuses a second record for C-2 with `duplicate-child-case`.
  - Red output: `a real case reaches {opened, child accepted, licence denied} and cannot leave it, and the walk reports no finding … simulated: protocol `adversary.ended_once` revision 1: 31 reachable states, 16 terminable (outcome `declined` 14, outcome `done` 4), 10 waiting on supplied evidence, a child outcome or expiry, 0 findings (0 stuck or cycle states)`
- `adversary_simulate_child_outcome_cli.rs:33`, the same from the command line (**red**): `expected exit 1 and a stuck-state or cycle … got exit Some(0)`.
- `adversary_simulate_child_outcome.rs:138` `no_printed_finding_holds_two_outcomes_of_the_only_child` (**red**).
  - Fixture `either-once`: an `any` over `children [alpha]` and `children [beta]`.
  - The evaluator: `open` is blocked once it has run, and `alpha`+`beta` recorded for C-2 are refused as `duplicate-child-case`.
  - Red output: `{evidence `case_outcome` result `alpha`, evidence `case_outcome` result `beta`, evidence `opened`, action `open` opened a child} after action `open` produces evidence `opened`, then a child of action `open` records … `alpha`, then a child of action `open` records … `beta``
- `:214` `a_set_mark_counts_in_the_size_of_a_state` and `:260` `a_state_without_a_mark_does_not_contain_one_with_it` (**green**). They pin § Outcome 4.
  - With m5 applied (marks left out of `size`), `:216` fails: two findings are printed instead of one.
  - With m4 applied (marks left out of `records_within`), `:262` fails: one finding with `1 other stuck state contains it` instead of two.

**3. Suite run** (after the cases existed): `export PATH=$HOME/.cache/ess/toolchains/0.77.0:$PATH; cargo test --workspace --no-fail-fast`
- Result: 1064 passed, 3 failed (the three cases above), 3 ignored, 150 summaries, `EXIT 101`.
- An earlier run without the ess PATH also failed 94 ESS tests. That was the environment: rerun with the PATH, they pass.
- `cargo clippy --workspace --all-targets -- -D warnings` (stable): `EXIT 0`.

**4. Findings** (they cover 65a963a0)

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon/src/simulate/mod.rs:903 | acceptance | blocker | NEEDS-CHANGE | introduced | any opener whose guard closes after one step: the large fixtures, `closed`, `ended-once` |
| crates/canon-cli/src/lib.rs:154 | contract-drift | warning | NEEDS-CHANGE | introduced | `either-once`; any `children` read whose result depends on two outcomes together |
| crates/canon/src/simulate/mod.rs:675 | mutant | note | CONFIRMED | introduced | m4 survives every test of the unit; now killed by `:260` |
| crates/canon/src/simulate/mod.rs:685 | mutant | note | CONFIRMED | introduced | m5 survives every test of the unit; now killed by `:214` |
| crates/canon/src/simulate/mod.rs:636 | judgement | note | CONFIRMED | pre-existing | clippy 1.100 `needless_range_loop`, the only error on the branch; fixed on `release/prep` 3c9ce06c, not on this branch |
| fixtures/investigation/child-outcome-revision.yaml | judgement | note | CONFIRMED | pre-existing | protocol `epic`, action `open_story`: engineering vocabulary under AGENTS.md:17; not in this diff |

- **mod.rs:903, missed dead end.** The child-outcome step is offered from every state with a mark set, with no end. So a child that has already ended can "end again" with an unlisted outcome. That makes `declined` reachable, so a real dead end counts as waiting and is never reported. The base printed a finding with its limit line (exit 1); the branch exits 0.
  - Fix to name: let a mark stand for a child with no terminal record yet. A plain terminal successor clears the mark, and only a new step of the opening action sets it again. This also removes finding 2's false state.
- **lib.rs:154, the stated limit.** The smallest case is `either-once` (17 states).
  - "Two classes stand for two children, also for an action taken once" appears only in the module docs (mod.rs:171-174).
  - `canon simulate --help`, `website/docs/reference/cli.md:148` and `cli.yaml` state no limit at all now that the `limit:` line is gone.
  - A reader of the reference page cannot know that the printed `alpha`+`beta` state is not a real one.

**5. Attacked and not broken**
- **Claim 4 (same bytes).** 110 tracked `protocol/1` files without `case_open` give the same stdout and exit on base and branch, and identical bytes across two runs.
  - The 12 with `case_open` differ only as expected: the `limit:` line is gone, and the cycle moved to the smaller marked state. That includes the unpinned `case-composition.yaml` and `child-outcome-revision.yaml`. No new output contains `limit:`.
- **child-expiry-held.** All 5 findings hold an unlisted record. I checked findings 2 and 3 with `evaluate --at`: `done`, `lock` and `open_change` are all blocked.
- **Claim 5 (bound).** 12 kinds plus 2 openers: refused with `reachable-state-bound`, stdout empty, exit 1. The base walks 32768 states; 1 opener walks 36864.
- **Order.** Actions declared in reverse order give the same bytes. With two marks set, the path names the least marked action.
- **Claim 6 (vocabulary).** No source-domain identifiers among the added lines. "story" appears only in `story:` ids; "safety plan" in the event fixture is plain English.
- **Mutants.** m1 (no mark on the empty step), m2 (child step without a mark) and m3 (no replacing successor) are each killed by the unit's tests.
- **Large fixtures.** Still read by `simulate_child_outcome.rs` (`LARGE`) and by the check and validate tests that walk all of `fixtures/`.
- **Clippy 1.100.** No new error.
- Not attacked: the order in which an atemporal expiry runs.

**6. Paths written outside the worktree.** All under `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w6/simulate-child-outcome-step/adversary-p1/`:
- binary copies: `canon-base`, `canon-new`
- logs: `build.log`, `clippy100.log`, `clippy-stable.log`, `red-*.log`, `new-lib.log`, `suite*.log`, `mutant*.log`, `mutants*.log`, `cmpall.log`
- comparison output: `cmpall/`
- probes: `probe/`
- scripts: `*.sh`, `*.py`
- `mod.rs.pristine`, `msg-1.txt`

The lease is released, and `worktree discard-cache` freed 7613 MiB.

```findings
[{"file": "crates/canon/src/simulate/mod.rs", "line": 903, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "The child-outcome step is offered from every marked state forever, so an only child that already ended can end again unlisted; a real dead end (ended-once) counts as waiting and the walk exits 0, where the base exited 1."}, {"file": "crates/canon-cli/src/lib.rs", "line": 154, "category": "contract-drift", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "either-once prints a stuck state holding two outcomes of the one child of an opening taken once; the counting limit is in the module docs only, and the reference page and cli.yaml state no limit now that the limit line is gone."}, {"file": "crates/canon/src/simulate/mod.rs", "line": 675, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Leaving the marks out of records_within (R5 containment) kept every test of the unit green; a_state_without_a_mark_does_not_contain_one_with_it now kills it."}, {"file": "crates/canon/src/simulate/mod.rs", "line": 685, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Leaving the marks out of size kept every test of the unit green; a_set_mark_counts_in_the_size_of_a_state now kills it."}, {"file": "crates/canon/src/simulate/mod.rs", "line": 636, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "clippy 1.100 refuses this loop as needless_range_loop; release/prep 3c9ce06c fixes it and this branch does not carry the fix."}, {"file": "fixtures/investigation/child-outcome-revision.yaml", "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "Protocol epic and action open_story are engineering vocabulary that AGENTS.md line 17 excludes from Canon."}]
```
