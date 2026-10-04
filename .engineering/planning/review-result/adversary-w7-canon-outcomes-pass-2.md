---
format: aep.planning-md/3
id: review-result:adversary-w7-canon-outcomes-pass-2
kind: review-result
status: active
title: Wave 2026-10-04-w7 adversary, canon story:outcomes, pass 2
relations:
- reviews: story:outcomes
revision: 1
---
unit: canon/outcomes, working tree cf91239 + uncommitted phase 2 + pass-1 fixes (worktree canon-w7-outcomes)
verdict: CONFIRMED (no defect in the implementation; 5 mutants the existing suite misses, all killed by the new file)
cases: executed 263→272, red 0
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 paths
needs-coordinator: none

**1. Diff**

The tracked `git diff --stat` matches what I was handed: 5 files, +398/−33. None of it is mine. My only change is one new untracked test file:

```
?? crates/canon/tests/adversary2_outcomes_polarity.rs
```

I made no change to implementation files, `ess/`, or the planning store.

**2. Cases added** (`crates/canon/tests/adversary2_outcomes_polarity.rs`)

I ran the file alone first, with `--list` showing all 9 tests in this tree. All 9 pass. So none of them failed on the code; their value is that they catch mutants.

I made 5 mutants of `eval/outcomes.rs` in a scratch copy under a renamed package (`b10x-canon-adv2mut`), so no artifact clashes with the unit's build. Against each mutant I ran the crate's lib tests and 19 of its 21 integration tests (the two that start `cargo` were left out). Every mutant failed only tests in the new file. No existing test failed:

| Mutant (outcomes.rs line) | Killed only by |
|---|---|
| M1 `:116` a second `not` does not flip back | `nested_negations_…` :61, `a_deeply_negated_…` :257 |
| M2 `:122` `all`/`any` ignore polarity (`!= True`) | `a_negated_connective_…` :105, `an_any_inside_a_not_inside_an_all_…` :89 |
| M4 `:134` evidence leaf ignores polarity | `a_negated_evidence_match_…` :134, :89 |
| M7 reasons in walk order instead of sorted | `reasons_are_claims_in_claim_id_order_…` :171 |
| M8 claim reasons not deduplicated | `a_claim_tested_with_different_is_values_…` :147, :171 |

M1's red output, verbatim:
```
thread 'nested_negations_name_the_reason_the_equivalent_plain_test_names' panicked at crates/canon/tests/adversary2_outcomes_polarity.rs:74:5:
assertion `left == right` failed
```

**Why the existing suite misses M7 and M8:** `canon compile` sorts and deduplicates the members of every `all`/`any` (ir/mod.rs:181). So the flat requirements in the outcomes.rs unit tests (:249, :303) look the same whether the code sorts reasons or not.

**Why no CLI test or scenario catches M1, M2 or M4:** every fixture's outcome requirement is the plain `claim: explanation.supported`, with no `not`. I found that with grep and did not run the CLI suite against the mutants.

**3. Suite** (run after the cases existed)

`cargo test --workspace --locked` exited 0. The summary lines add up to 272 passed, 0 failed, across 42 binaries.

**4. Judgement findings**

| file:line | verdict / origin | finding | what reaches it |
|---|---|---|---|
| crates/canon/src/eval/outcomes.rs:67 | CONFIRMED / introduced | An illegitimate-termination refusal replaces the whole decision, and its message drops the reasons. A case that ended legitimately becomes impossible to evaluate as soon as later evidence makes its claim unknown, for example a second falsification attempt with a different result. | Any caller that evaluates a closed case again with more evidence. Inferred from reading the code and the `terminated-blocked` step; I did not run that exact case. |
| crates/canon/src/eval/outcomes.rs:135 | CONFIRMED / introduced | The evidence reason `{evidence: l}` comes out the same in two opposite situations. My test at :134 gives it when an `l` record exists under a `not`. The unit test `mixed` gives it when no `l` record exists. A claim reason carries its value; an evidence reason does not. | The reason output of `canon evaluate`. Measured with both tests. |
| crates/canon-cli/tests/adversary_skel_cli.rs:103 | CONFIRMED / introduced | The test is named `a_recorded_termination_changes_no_decision`, but it now asserts that both terminations are refused. | Whoever reads the test next. |

**5. Attacked and could not break**
- The polarity walk through `not not`, three `not`s, an `any` inside a `not` inside an `all`, and 4000 nested `not`s run from the test thread.
- A claim tested with different `is` values: it is named once, with its own value.
- Order of refusals: claim-cycle and unsupported-input both come before undeclared-outcome and illegitimate-termination.
- A legitimate outcome never carries reasons.
- Reasons are always claims, then evidence, then unsatisfiable.
- The doc comments in outcomes.rs at :1–27, :39–41, :100–104 and :142–143 all match the behaviour.
- A termination that is legitimate only through excluded evidence can't be tested yet: the exclusion stages in this tree exclude nothing.

**6. Paths written outside the worktree**
- `~/.cache/ga-wave-2026-10-04-w7/canon-outcomes/scratch/adv2-mutant/` — the repo copy, the mutant sources, and the logs for the baseline and each mutant.
- `~/.cache/ga-wave-2026-10-04-w7/canon-outcomes/scratch/adversary2-cases.log` and `adversary2-suite.log`.
- `b10x_canon_adv2mut*` artifacts in `~/.cache/b10x-target/canon-w7-outcomes` (the renamed package, so they don't collide with the unit's build).

```findings
- file: crates/canon/src/eval/outcomes.rs
  line: 116
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "Mutants M1 (:116), M2 (:122), M4 (:134), M7 and M8 (reason order and deduplication) each leave the existing canon suite green and are killed only by crates/canon/tests/adversary2_outcomes_polarity.rs."
- file: crates/canon/src/eval/outcomes.rs
  line: 67
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "An illegitimate-termination refusal replaces the whole decision and drops the reasons, so a case that ended legitimately cannot be evaluated once later evidence makes its claim unknown."
- file: crates/canon/src/eval/outcomes.rs
  line: 135
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "The reason {evidence: kind} is the same for a missing required record and for a present record under not, because evidence reasons carry no value while claim reasons do."
- file: crates/canon-cli/tests/adversary_skel_cli.rs
  line: 103
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "The test named a_recorded_termination_changes_no_decision now asserts that both terminations are refused."
```
