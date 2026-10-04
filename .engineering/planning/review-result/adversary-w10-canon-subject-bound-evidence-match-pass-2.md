---
format: aep.planning-md/3
id: review-result:adversary-w10-canon-subject-bound-evidence-match-pass-2
kind: review-result
status: active
title: Wave 2026-10-04-w10 adversary, canon story:subject-bound-evidence-match, pass 2
relations:
- reviews: story:subject-bound-evidence-match
revision: 1
---
unit: canon/subject-bound-evidence-match, working tree on 3c38c75 with uncommitted phase 2 and the pass-1 fixes
verdict: NEEDS-CHANGE
cases: executed 415→421, red 2
origin: introduced 2 / pre-existing 0 / undecided 2
wrote-outside-worktree: 3 paths (listed in part 6)
needs-coordinator: yes. The explanation unit reads evidence by kind only, and it edits two files this unit also edits.

**Pass 2 result:** the evaluator does what the story asks; I could not break it. The ELS case holds end to end with the authority granted: the test result about the release admits and legitimates nothing. Reason order and the IR order also hold. Two things are wrong outside the evaluator: the generated evaluation reference still describes the old behaviour, and the explanation unit will clash with this one at merge.

### 1. `git --no-pager diff --stat`
The tracked diff is the implementor's 27 files from phase 2, unchanged by me. I added only two untracked test files:
- `crates/canon/tests/adversary2_subject_e2e.rs`
- `crates/canon/tests/adversary2_subject_docs.rs`

I touched no non-test path.

### 2. Cases added, each run alone before the suite

| Case | Asserts | Now |
|---|---|---|
| `e2e::a_passing_test_result_about_the_release_admits_nothing_with_the_merge_granted` | ELS `software.change/1` shape, with `repository.merge` and `release.promote` granted, and a passing `test_result` about the release at its current revision v1. Expected: `tests.pass` unknown, `verified` open, both actions blocked, `accepted` and `shipped` blocked, and termination through `accepted` refused as `illegitimate-termination`. The control (a passing result about the implementation at R2) gives admissible and legitimate. | green |
| `e2e::a_stale_own_result_and_a_current_foreign_one_still_admit_nothing` | The implementation's result at R1 is listed under all three claims. The release's result is listed nowhere. Merge stays blocked. | green |
| `e2e::reasons_of_one_kind_with_and_without_a_subject_come_in_kind_then_subject_order` | Outcome reasons come in the order `{k}`, `{k,a}`, `{k,b}`, `{l}`, and `present` counts only the records each match reads. The action names only the `false` members. In the rendered output, `subject` comes after `present`. | green |
| `e2e::every_order_of_result_and_subject_combinations_compiles_to_one_ir` | 18 combinations (two kinds × three results × three subjects) plus two duplicates, in 200 seeded orders. Every order gives the same IR bytes and the same decision. The IR reads back, and members are ordered by kind, then result, then subject, each once. | green |
| `docs::the_evaluation_reference_gives_the_evidence_reason_the_evaluator_writes` | Observed first: `[{"evidence":"test_result","present":false,"subject":"change"}]`. Then the reference's `actions` bullet must name `subject` and must not define `present` as "whether a record of the kind applies". | **red** |
| `docs::the_evaluation_reference_says_which_claims_list_an_excluded_record` | Observed first: an excluded record about the docs is not listed under `tests.pass`. Then the summary paragraph must not say "listed as excluded under each claim that reaches its kind". | **red** |

Red output, each case run alone:
```
panicked at crates/canon/tests/adversary2_subject_docs.rs:86:5:
the reference's action reason has no `subject` key, which the evaluator writes:
- `actions` maps each declared action ... or `{"evidence": <kind>, "present": <whether a record of the kind applies>}`, or ...
panicked at crates/canon/tests/adversary2_subject_docs.rs:118:5:
the reference says an excluded record is listed under each claim that reaches its kind; `tests.pass` reaches `test_result` and lists nothing:
Revision binding excludes ... each listed as excluded under each claim that reaches its kind. ...
```

Proof that the green cases can fail, run on a scratch copy with its own target dir:
- **Mutant 1:** `reads` ignores the subject (`claims.rs:165`). Three cases go red. For example, `tests.pass` came out `True` where `Unknown` was expected, merge came out `"admissible"` where `"blocked"` was expected, and `{k,b}` reported `present:true`.
- **Mutant 2:** the subject order is reversed in `canonical_order` (`ir/mod.rs:275-278`). The IR-order case goes red.

### 3. Suite (run after the cases existed)
- Command: `cargo test --workspace --locked --no-fail-fast`
- Result: EXIT=101, 419 passed, 2 failed (the two docs cases above).
- `-- --list` shows all six new tests in this tree's binaries.
- Log: `scratch/adv2-suite.log`

### 4. Findings
1. **The action reason is documented with the old shape** (`crates/canon/src/eval/mod.rs:135`, generated into `website/docs/reference/evaluation.md:144`). It reads `{"evidence": <kind>, "present": <whether a record of the kind applies>}`. The evaluator now adds `subject`, and gives `present:false` even when a record of the kind applies.
   - What reaches it: any `canon evaluate` whose action precondition uses a subject-bound match.
   - Verdict: NEEDS-CHANGE, introduced.
   - Fix: reword the doc the way `actions.rs:13-19` is worded, then regenerate.
2. **The exclusion summary is stale** (`crates/canon/src/eval/mod.rs:118`, generated into `website/docs/reference/evaluation.md:127`). It still says each excluded record is "listed as excluded under each claim that reaches its kind". Steps 4 at `:105-107` were updated; this paragraph was not.
   - Verdict: NEEDS-CHANGE, introduced.
3. **The explanation unit reads evidence by kind only** (`canon-w10-explanation/crates/canon/src/explain/mod.rs:159` `because` and `:320` `reached_kinds`). After merge, a reason `{evidence:k, present:false, subject:a}` would be followed by records about `b` listed as `applied`.
   - `claims::reads` is `pub(super)` (`claims.rs:160`), so `crate::explain` cannot call it.
   - That unit also edits `eval/outcomes.rs:203-206` (the `reasons` doc, which this unit rewrote) and `eval/mod.rs`, so the brief's "disjoint files" does not hold.
   - What reaches it: the merge of the two units. I could not run that merge here.
   - Verdict: INFEASIBLE in this unit, origin undecided.
4. **No conformance scenario fixes the new reason shape.** `ess/domains/protocol.yaml:434-444` says each section's shape is fixed by its story's scenarios. `CANON-EVIDENCE-003` has no actions or outcomes step, so `{evidence, present, subject}` is held only by unit tests and adversary tests.
   - Verdict: CONFIRMED, note, introduced.

### 5. Attacked and could not break
- Other kind-only readers: freshness and evidence declaration are kind-only by design. `invalidation.rs` and `explain` in this tree are stubs. `binding.rs` reads `subject` against the case. Validation also covers subjects in discharge predicates. `read_ir` recompiles, so a subject naming an undeclared artifact is refused.
- Landing graph: two `establishes` edges that differ only in subject merge into one identical edge (a `BTreeSet`). That is correct at the kind level, and the claim node's predicate keeps the subject. The graph is built only from `fixtures/investigation/protocol.yaml`, which has no subject. I could not find the definition of the `b10x-protocol-graph/1` format locally to check whether it allows a `subject` key.
- Docs, status row and `product.json`: eight scenario files, "eight ship today" and the `CANON-EVIDENCE-003` rows match `conform` (8 passed). The concept pages' tables match the scenario steps.
- The phase-2 `ess/domains/protocol.yaml` diff changes only the `read from` line numbers in comments.

### 6. Paths written outside the worktree
- `~/.cache/ga-wave-2026-10-04-w10/canon-subject-bound-evidence-match/scratch/adv2-e2e.log`
- `~/.cache/ga-wave-2026-10-04-w10/canon-subject-bound-evidence-match/scratch/adv2-docs.log`
- `~/.cache/ga-wave-2026-10-04-w10/canon-subject-bound-evidence-match/scratch/adv2-suite.log`

The mutant copy `scratch/adv2-mutant` and its target dir `scratch/adv2-mutant-target` were deleted. The shared build dir was used as briefed.

### 7. Findings block
```findings
- file: crates/canon/src/eval/mod.rs
  line: 135
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "The generated evaluation reference (website/docs/reference/evaluation.md:144) gives the action evidence reason as {evidence, present: whether a record of the kind applies} with no subject key, while the evaluator writes subject and present:false when a record of the kind about another artifact applies; red in adversary2_subject_docs."
- file: crates/canon/src/eval/mod.rs
  line: 118
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "The exclusion summary (evaluation.md:127) still says an excluded record is listed under each claim that reaches its kind, but a claim reaching the kind only through a subject-bound match does not list a record about another artifact; red in adversary2_subject_docs."
- file: crates/canon/src/explain/mod.rs
  line: 320
  category: judgement
  severity: warning
  verdict: INFEASIBLE
  origin: undecided
  message: "The concurrent explanation unit collects reached kinds and lists records by kind only (because at :159), and claims::reads is pub(super) so it cannot use it; after merge a subject-bound reason with present:false would be followed by records about other artifacts listed as applied, and both units edit eval/outcomes.rs and eval/mod.rs."
- file: ess/domains/protocol.yaml
  line: 434
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "Section shapes are said to be fixed by conformance scenarios, but no scenario holds the new {evidence, present, subject} reason shape; CANON-EVIDENCE-003 has no actions or outcomes step."
```
