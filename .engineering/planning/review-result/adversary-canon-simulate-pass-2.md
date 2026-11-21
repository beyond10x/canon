---
format: aep.planning-md/3
id: review-result:adversary-canon-simulate-pass-2
kind: review-result
status: active
title: 'Adversary pass 2 on story:canon-simulate (wave 2026-11-20-w5, carried from wave 4): red, 1 red case, 1 warning and 1 note, introduced'
relations:
- reviews: story:canon-simulate
revision: 1
---
Adversary pass 2, the last, on story:canon-simulate, carried from wave 2026-11-20-w4 into wave 2026-11-20-w5, `aep:adversary` on Opus, on the correction `9c92fd1..4b36d52` of `impl/canon-simulate`. The report below is as the agent returned it.

unit: story:canon-simulate, branch impl/canon-simulate at 5017c20 (the correction 9c92fd1..4b36d52 plus my one test commit)
verdict: red. One case is red: the expiry step drops a child-case outcome that the evaluator still counts.
cases: executed 1006→1007, red 1
origin: introduced 2, pre-existing 0, undecided 0
wrote-outside-worktree: `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w4/canon-simulate/adversary-p2/` (probes/, eval-child/, sdc/, gen_bound.py, gen_inputs.py, sdc.sh, time16.sh, msg-1.txt, adv-before.log, red.log, suite.log, clippy.log, time16.log, sdc.log)
needs-coordinator: yes. This was the last pass, and the R2 warning below needs a fix by the implementor.

**What I changed:** one file, `crates/canon/tests/adversary_simulate.rs` (79 lines added, 2 removed), committed as 5017c20 by the bot. I did not touch any implementation file, unit test or fixture.
- **`:183` case:** the implementor is right. The output differed only by the ending `; 1 other stuck state contains it`, and R5 requires it because the stuck state `{note}` contains `{}`. I added that ending to the expected string; the case is green now.
- **New case `:222`** `an_expired_child_outcome_still_counts_for_the_children_form`: red.

**Results:**
- **Before my edit:** `test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
- **My file after the edit:** `test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`
- **Workspace suite** (`cargo test --workspace --locked --no-fail-fast`, ess 0.77.0 on PATH): exits 101. Summed over its 141 summary lines (my sum): 1006 passed, 1 failed, 3 ignored. The only failing target is `adversary_simulate`: `test result: FAILED. 3 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.54s`.
- **Lint:** `cargo clippy --locked -p b10x-canon --test adversary_simulate -- -D warnings` exits 0, and `cargo fmt -p b10x-canon -- --check` exits 0.

## Findings

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon/src/simulate/mod.rs:669 | acceptance | warning | NEEDS-CHANGE | introduced | A protocol whose `case_outcome` kind declares `max_age` and is read by a `children` form. The evaluator's `children` form reads every accepted record, expired ones too (`eval/composition.rs:8-10`: an expiry "does not undo" a child's outcome). The expiry step removes the record, so the state after it does not evaluate as the real case (contrary to R2). Probe `probes/child-expiry.yaml`: open a child, lock once it ended accepted, the outcome needs both. The walk reports `stuck-state: state {evidence `lock`}` with a path ending `then evidence `case_outcome` result `accepted` expires`. `canon evaluate --at` two hours later on that case gives `done` `legitimate` and `resolved` `true`. The reported stuck state is false. Case: `adversary_simulate.rs:222` |
| crates/canon/src/simulate/mod.rs:839 | performance | note | CONFIRMED | introduced | The R5 search for the smallest state compares states pairwise. If many stuck states are not contained in each other (16 two-valued case inputs, every state stuck), 65,536 states take 157.7 s. With no stuck states, 65,536 states take 12.4 s (`rec-exact`). 16,384 states take 19.3 s. No SDC protocol comes near this (slowest 1.87 s), and the 5 s claim covers SDC protocols only. |

## Pass 1's findings against the correction

1. space.rs:2360, a dimension wider than a `u32`: answered (R1). Case `:63` is green, and the five-upstream probe walks 3 states.
2. mod.rs:584, records never expire: answered (R2). Case `:115` is green. The exception is `case_outcome`, which is the new warning above.
3. mod.rs:472, waiting when any supplied step changes the state: answered (R3). Waiting is 0 and the state is `{}` with path `empty`. The only difference was the R5 ending, which I re-pinned.
4. mod.rs:469, decision steps ignored: answered (R4). For example, sdc.release now ends with `decision `explicitly_withheld` taken makes outcome `withheld` legitimate`.
5. mod.rs:568, no grouping: answered (R5). sdc.plan gives 1 finding with `513 other cycle states contain it` (pass 1 had 514 findings).
6. mod.rs:22, the word `cycle`: answered as decided (R6). The module docs and help text say what a `cycle` is.

## Attacked and not broken

- **R1, 4 probes:**
  - Five upstream artifacts: 3 states.
  - 65,536 states built from records (16 kinds): exit 0. Add one state (a 17th kind) and it is refused with `reachable-state-bound`.
  - 65,536 states built from case inputs: exit 0.
- **`canon check` unchanged:** the four hunks in `check/mod.rs` and the code appended to `space.rs` do not change what check prints. No check test or fixture changed in `d3a4fe2...HEAD`.
- **R2, 3 probes:** the expiry fixture; `fresh.yaml`, a supplied kind with `max_age` (all 8 states, the 2 terminable and the 1 waiting counted by hand); `child-expiry.yaml` (the finding).
  - Hand evaluations with `--at`: 1 (child-expiry), plus the unit's own expiry test.
  - The `children` form is the only reader of expired records left: order forms are refused before the walk. Invalidation runs on the records left after expiry. I checked both by reading the code, without a probe.
- **R3:** action and authority steps only add, so every state that is not open reaches a state where no such step changes anything. There every capability is decided, so no action is `approval-required`, and that state is reported.
  - Probes: renew-loop, fresh, cap and the inputs family. No unreported dead end, and waiting was right in each.
- **R4, 1 probe:** the decision names come in name order, and one decision serving two outcomes lists both outcomes in identifier order.
  - An outcome can require only a single decision: the parser refuses `decision` inside `all`. So "a decision plus a claim" and "two decisions for one outcome" cannot be written.
- **R5, 6 probes:**
  - `cap`: an undecided capability is contained in both granted and denied. `{}` is printed with `5 other stuck states contain it`; denied is not contained in undecided.
  - Inputs with 10, 12, 13, 14 and 16 two-valued case inputs: no state contains one with other input values (every stuck state is printed).
  - The summary's findings and states are right in each.
- **SDC, 8 protocols run twice:** identical bytes in every run. Slowest: intake 1.87 s; limitations/monolith 2.56 s.
  - sdc.plan checked by hand: the state is the approved result plus another result, the path is two `roadmap.read` steps, 513 others (pass 1's 514), and no decision suffix since sdc.plan has no decision outcome.
  - sdc.release: `withheld` requires only `explicitly_withheld`.
- **Spec commit 7a9d05e:** the nine old `report.txt` files changed only in their summary line, and no finding line was removed.
  - The deleted unit test `a_dimension_a_state_cannot_hold_is_refused` expected exactly what R1 reverses.
  - `a_decision_is_a_step` was replaced by a stricter exact-text test.
- **`ess/domains/cli.yaml` and help text:** their wording matches the printed output and the waiting rule.

```findings
[{"file": "crates/canon/src/simulate/mod.rs", "line": 669, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "the expiry step drops a case_outcome record that the evaluator's children form still reads after it expires, so a protocol with max_age on case_outcome gets a false stuck-state (adversary_simulate.rs:222), contrary to R2"},
 {"file": "crates/canon/src/simulate/mod.rs", "line": 839, "category": "performance", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the R5 search for the smallest state is quadratic in stuck states; 65536 stuck states not contained in each other take 157.7 s against 12.4 s for 65536 states with no finding; no SDC protocol is affected"}]
```
