---
format: aep.planning-md/3
id: review-result:adversary-w10-canon-canon-check-pass-1
kind: review-result
status: active
title: Wave 2026-10-04-w10 adversary, canon story:canon-check, pass 1
relations:
- reviews: story:canon-check
revision: 1
---
unit: canon/canon-check, working tree of ~/.local/state/worktree/trees/b10x/canon/canon-w10-canon-check (HEAD 4e8da47 plus uncommitted phase 2)
verdict: NEEDS-CHANGE (3 suite gaps that let a defect through, each covered by a new case; 1 false finding once the subject-bound unit merges)
cases: executed 410→413, red 0 (in this tree; 2 red in a scratch merge with subject-bound-evidence-match)
origin: introduced 7 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 paths (listed in part 6)
needs-coordinator: yes. The subject-bound gap can only be fixed after both units merge, because this tree's model has no `subject` field on an evidence match.

**1. Diff stat**

`git --no-pager diff --stat` lists only the implementor's 11 uncommitted tracked files. My only addition is one untracked test file, `crates/canon/tests/adversary_check_space.rs`. No implementation file, nothing under `ess/`, and no non-test path was touched by me.

**2. Cases added**

All three cases are in `~/.local/state/worktree/trees/b10x/canon/canon-w10-canon-check/crates/canon/tests/adversary_check_space.rs`. They are green on the real code. Each one is red on a mutant that survives the unit's whole suite: the 9 `check::` library tests and 3 `canon_check` tests stayed green under every mutant. The mutants ran in a scratch copy with its own target dir.

| Case | Mutant | Unit suite | This case, red output |
|---|---|---|---|
| `a_result_only_an_action_precondition_matches_is_a_class` | `space.rs:324`: action preconditions dropped from `predicates()` | green | `left: "unsatisfiable-precondition: action \`gate\`: its precondition holds in no state\nchecked: … 2 states, 0 properties, 1 finding\n"` |
| `a_result_only_an_outcome_requirement_matches_is_a_class` | `space.rs:324`: outcome requirements dropped | green | `left: "unreachable-outcome: outcome \`done\` is legitimate in no state\nchecked: … 4 states, 0 properties, 1 finding\n"` |
| `a_space_of_exactly_the_bound_is_checked_and_one_past_it_refused` | `mod.rs:261`: `<=` changed to `<` | green | `left: "refused state-space-bound: protocol \`p\` revision 1 has 65536 states, more than the bound of 65536: …"` |

There is also a case kept out of the tree, at `~/.cache/ga-wave-2026-10-04-w10/canon-canon-check/scratch/adversary-p1/adversary_check_subject.rs`. I ran it in a scratch merge: the subject-bound tree with this unit's `check/`, `model/properties.rs`, `model/mod.rs` and `lib.rs` laid over it. Neither unit edits those three non-check files, and the merge compiled without conflicts. Both tests are red:
```
unreachable-outcome: outcome `done` is legitimate in no state
checked: protocol `p` revision 1: 2 states, 0 properties, 1 finding
unreachable-outcome: outcome `split` is legitimate in no state
checked: protocol `p` revision 1: 8 states, 0 properties, 1 finding
```

**3. Suite run, after the cases existed**

`cargo test --workspace --locked` (in the assigned build dir) exited 0: 413 passed, 0 failed. `--list` shows all 3 new names. Clippy with `-D warnings` and `cargo fmt --check` are clean on the new file.

**4. Findings**

| file:line | verdict / origin | What I measured | What reaches it |
|---|---|---|---|
| crates/canon/src/check/space.rs:49 | INFEASIBLE / introduced | Every evidence record is written about the first artifact, and result classes are collected per kind with the subject ignored. In the merge, a match that names artifact `b` never sees a record, so reachable outcomes are reported unreachable (red case above). | The subject-bound unit's `{evidence: {kind, subject}}`, which merges first in this wave. It cannot be fixed in this tree. The fix is to make the dimension per (kind, subject named by any match), with the unbound case separate. |
| crates/canon/src/check/space.rs:324 | CONFIRMED / introduced | Nothing in the suite checks that results matched only in action preconditions or outcome requirements become classes. Dropping either survives the suite. | Any protocol whose preconditions or outcomes match a result directly. |
| crates/canon/src/check/mod.rs:261 | CONFIRMED / introduced | Nothing tests a space of exactly 65,536 states; changing `<=` to `<` survives. | The story's "above it refuses". The 16-kind case takes about 7 s in debug. |
| crates/canon/src/check/mod.rs:305 | CONFIRMED / introduced | `unproduced-evidence` only looks at claims. When an outcome and a precondition read an unproduced kind directly, the report says `0 findings`, so the outcome counts as reachable. | The story's literal wording says "claims", so the code follows its spec. This is a gap in the spec. |
| crates/canon/src/check/mod.rs:299 | CONFIRMED / introduced | A kind produced only by an action whose precondition never holds counts as produced. The only finding is `unsatisfiable-precondition`, and the outcome that depends on that kind is reported reachable. | This is the brief's "unproduced evidence" scenario. It is consistent with the static definition that the documentation gives. |
| crates/canon/src/check/mod.rs:413 | CONFIRMED / introduced | `withdrawn: {requires: {claim: approved, is: unknown}}` produces `authority-bypass … in state {}`. The outcome is legitimate *because* the governed evidence is absent. | Any `is: unknown` test over governed evidence. The documented definition holds vacuously here. |
| crates/canon/src/check/mod.rs:36 | CONFIRMED / introduced | The rustdoc example says "98304 states" but lists dimensions 4·3·2 = 24. | Only readers of the rustdoc. |

**5. Attacked and could not break**

- **Classes:** the claim evaluator treats a record with no result the same as one with any unmatched result, so the class abstraction is exact before the merge. That covers `not` over a result and matches without a result.
- **Overflow:** with 81 capabilities the refusal says "more than u128::MAX states", which is accurate. A kind with 2^128 values is already tested.
- **Properties:** properties over a shared decision name, over a claim that reads nothing, and over decision outcomes all behaved correctly.
- **Kind-level counterexample:** the one I probed was a real dependency, not a false failure.
- **Exit statuses:** clean 0, findings 1, refusal 1, bad or mismatched properties 1, invalid protocol 1. A missing protocol, missing properties file or a directory gives 2. All match EXIT_STATUSES.
- **Determinism:** ordering comes from BTreeMap and sorted properties, and witness ties are broken by index order. The acceptance test already compares two runs.

**6. Paths written outside the worktree**

- `~/.cache/ga-wave-2026-10-04-w10/canon-canon-check/scratch/adversary-p1/adversary_check_subject.rs`
- `~/.cache/ga-wave-2026-10-04-w10/canon-canon-check/scratch/adversary-p1/suite.log`

I created and then deleted the scratch copies `mutant/` and `merged/` and their build dirs `target-mutant/` and `target-merged/`, plus three probe YAML files. A throwaway probe test inside the worktree was also deleted.

**7. Findings block**

```findings
- file: crates/canon/src/check/space.rs
  line: 49
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "Once subject-bound evidence matches merge, every checked record is about the first artifact and classes ignore the match subject, so an outcome needing evidence about another artifact is falsely reported unreachable (red in a scratch merge)."
- file: crates/canon/src/check/space.rs
  line: 324
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "Dropping action preconditions or outcome requirements from the class collector survives the unit's suite; adversary_check_space.rs now kills both mutants."
- file: crates/canon/src/check/mod.rs
  line: 261
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "No case checks a space of exactly 65536 states, so a `<` bound mutant survives; adversary_check_space.rs now kills it."
- file: crates/canon/src/check/mod.rs
  line: 305
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "An outcome or precondition that reads an unproduced evidence kind directly, not through a claim, gets no finding at all and the outcome counts as reachable."
- file: crates/canon/src/check/mod.rs
  line: 299
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "A kind produced only by an action whose precondition never holds counts as produced, and outcomes needing it are reported reachable."
- file: crates/canon/src/check/mod.rs
  line: 413
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "An outcome legitimate because governed evidence is absent (`is: unknown`) is reported as an authority bypass in state {}."
- file: crates/canon/src/check/mod.rs
  line: 36
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "The rustdoc bound-refusal example says 98304 states but lists dimensions whose product is 24."
```
