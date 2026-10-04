---
format: aep.planning-md/3
id: review-result:adversary-w11-canon-invalidation-rules-pass-1
kind: review-result
status: active
title: Wave 2026-10-04-w11 adversary, canon story:invalidation-rules, pass 1
relations:
- reviews: story:invalidation-rules
revision: 1
---
unit: canon/invalidation-rules, working tree at bdf8284 plus the uncommitted phase 2, plus my one test file
verdict: NEEDS-CHANGE
cases: executed 491→496, red 2
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 paths (scratch/adversary-p1/ logs; copy and its target dir already deleted)
needs-coordinator: yes. Finding 2 needs a ruling: should a named claim that only reads evidence through a claim the rule doesn't name lose that support, or stop listing the record as excluded?

**1. Diff stat (mine only)**

`?? crates/canon/tests/adversary_invalidation.rs` is untracked, so `git diff --stat` doesn't show it. Every `M` path in the tree is the implementor's uncommitted phase 2. I changed no file other than this test file.

**2. Cases** (all in `crates/canon/tests/adversary_invalidation.rs`; first run of this file alone, saved in `adversary-p1/red.log`)

| Case | Asserts | Now | Proven able to fail |
|---|---|---|---|
| `check_does_not_call_an_outcome_unreachable_that_evaluate_reaches` | evaluate finds `split` legitimate with one old `pass` record and one current `fail` record of the same kind, so check must not report `split` unreachable | **red** | — |
| `a_record_listed_as_excluded_from_a_claim_does_not_decide_that_claim` | removing a record listed under a claim does not change that claim's value | **red** | — |
| `check_flags_a_dimension_only_a_claim_built_on_a_named_one_reads` | check gives a flag to `m`, which only a claim built on the named one reads; 9 states, 0 findings | green | mutant "flags from the named claims only": red |
| `two_rules_naming_one_claim_list_the_record_once_through_every_level` | 2 upstream artifacts, chain through `not` and `is: unknown`: record listed once, values U/U/T/F/T | green | mutant "claims built on a named one not invalidated": red |
| `binding_wins_over_invalidation_and_lists_the_record_once` | a record with the wrong revision whose upstream also moved is listed once, as `revision_mismatch` | green | mutant "invalidation sees all records": red, `[("e1", RevisionMismatch), ("e1", Invalidated)]` |

Red output, verbatim:
```
assertion `left == right` failed: unreachable-outcome: outcome `split` is legitimate in no state
checked: protocol `p` revision 1: 7 states, 0 properties, 1 finding
  left: ["unreachable-outcome: outcome `split` is legitimate in no state"]
  right: []
assertion `left == right` failed: `named` lists `e1` as Invalidated yet its value changes when `e1` is removed
  left: True
  right: Unknown
```

**3. Gate** (in the unit's build dir, run after the cases existed)
```
cargo fmt --all --check                                         EXIT=0
cargo clippy --workspace --all-targets --locked -- -D warnings  EXIT=0
cargo test --workspace --locked --no-fail-fast                  EXIT=101  83 binaries, passed 494, failed 2
  test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
ess specify validate … && ess verify conform synthesize …        EXIT=0  canon v1 — 3 file(s), valid / 0 scenario(s) (0 authored), 0 refusal(s)
cargo run -q --locked -p canon-docs -- generate --check         EXIT=0  canon-docs: 15 generated files are current
cargo run -q --locked -p canon-cli -- conform run               EXIT=0  conform: 10 passed, 0 failed, 0 unreadable
```
`cargo test -- --list` shows 496 tests, all 5 new names among them.

The unit's own suite (82 binaries, run on the mutant copy) stayed fully green under two mutants: flags taken from the named claims only, and invalidation seeing records binding had already excluded. Only my cases turn red on those.

**4. Findings** (covering the working tree on bdf8284)

| # | file:line | Verdict / origin | What was measured | What reaches it |
|---|---|---|---|---|
| 1 | `crates/canon/src/check/space.rs:430` | NEEDS-CHANGE / introduced, **blocker** | Check gives every record of an evidence dimension the same moved flag. It never builds one record moved and another not, so it reports `split` unreachable while evaluate finds it legitimate. The status row's claim that check "checks every state with an upstream artifact moved" is false. | The case invalidation exists for: an old result recorded against the earlier dataset revision beside a new result against the current one. Any outcome that tells "named claim false" apart from "unnamed claim unknown" gets a false `unreachable-outcome`. |
| 2 | `crates/canon/src/eval/mod.rs:385` (`excluded_for`; also `explain/mod.rs` `read_by`) | NEEDS-CHANGE / introduced, warning | `named: {claim: helper}` with the rule naming `named`: `named` lists `e1` as `invalidated` yet stays TRUE only because of `e1`. That contradicts the story's outcome ("stops supporting it"). | Any protocol that names a claim defined through a helper claim. The rule then changes nothing while the decision says it applied. |
| 3 | `crates/canon/src/check/space.rs:159` | CONFIRMED / introduced, note | Mutant "flags from `rule.invalidates` only": the unit's suite stays green. Now caught by my case. | Test gap only; the code is correct. |
| 4 | `crates/canon/src/eval/mod.rs:297` | CONFIRMED / introduced, note | Mutant "invalidation gets all evidence": the unit's suite stays green, and the record is listed twice. Now caught by my case. | Test gap only; the code is correct. |

Possible fixes, not applied: for 1, give each record of a dimension its own moved flag (this costs state-space size), or narrow what the docs claim check covers. For 2, the coordinator decides between excluding along the claims the named one tests, and listing the record only under claims whose own matches read it.

**5. Attacked, not broken**
- Exclusion through several levels, `not` and `is: unknown`, and a claim that is both named and built on a named claim: values and listings are correct.
- Two rules on different upstream artifacts: the record is listed once.
- A record at the current revision, or one that records no upstream revision, is not invalidated.
- A rule on an artifact the case doesn't list: case validation refuses it first.
- `revision_mismatch` vs `expired` vs `invalidated`: the first stage wins and the record is listed once. `excluded_evidence` stays in evidence-id order.
- Subject-bound matches go through `read_by` and `reads`.
- IR: `read_ir` compares bytes, so an unsorted `invalidates` list is refused.
- Validation order matches the docs. `property` erasure clears a dimension's flags with it.
- Bound count is `1+(2^c-1)·2^m`. The refusal lists 2^c and 2 per flag, so the listed numbers don't multiply to the total (documented).
- Docs, CLI help and product.json are current, apart from the status row in finding 1.

**6. Paths written outside the worktree**
- `~/.cache/ga-wave-2026-10-04-w11/canon-invalidation-rules/scratch/adversary-p1/`: `red.log`, `mutant1.log`, `mutant2.log`, `mutant2-full.log`, `mutant3.log`, `g1.log`–`g6.log`, `list.log`.
- Deleted: `scratch/adversary-p1/tree/` (mutant copy) and `scratch/adversary-p1/target/` (its build dir).
- `$CARGO_TARGET_DIR/suite.json` in the unit's build dir, which the gate rewrote.

```findings
- file: crates/canon/src/check/space.rs
  line: 430
  category: property
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "canon check gives every record of an evidence dimension the same moved flag, so it never builds one record moved and another not, and reports outcome split unreachable while canon evaluate finds it legitimate on an old-and-new-record case"
- file: crates/canon/src/eval/mod.rs
  line: 385
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "a named claim built on a claim the rule does not name lists the moved record as invalidated while its TRUE value still rests on that record, so the rule changes nothing yet the decision says it applied"
- file: crates/canon/src/check/space.rs
  line: 159
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the unit suite stays green when rule-dimension flags come from the named claims only instead of invalidated_claims; now caught by check_flags_a_dimension_only_a_claim_built_on_a_named_one_reads"
- file: crates/canon/src/eval/mod.rs
  line: 297
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the unit suite stays green when invalidation is given every record instead of what binding left, which lists one record twice; now caught by binding_wins_over_invalidation_and_lists_the_record_once"
```
