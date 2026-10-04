---
format: aep.planning-md/3
id: review-result:adversary-w7-canon-evidence-revision-binding-pass-1
kind: review-result
status: active
title: Wave 2026-10-04-w7 adversary, canon story:evidence-revision-binding, pass 1
relations:
- reviews: story:evidence-revision-binding
revision: 1
---
unit: canon/evidence-revision-binding. Covers the working tree at c755b24 plus the uncommitted phase 2, in ~/.local/state/worktree/trees/b10x/canon/canon-w7-evidence-revision-binding
verdict: CONFIRMED (2 red cases, both are documentation that no longer matches behaviour; no implementation defect found)
cases: executed 260→264, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 paths
needs-coordinator: yes. Finding 2's text sits in files outside this unit's scope: `crates/canon/src/model/evidence.rs` (evidence-freshness owns it), `crates/canon-docs/src/pages.rs`, and hand-written website pages.

**1. `git --no-pager diff --stat`**
```
 crates/canon/src/eval/binding.rs     | 123 +++++++++++++++++++++++++++++++++--
 crates/canon/src/eval/mod.rs         |  56 ++++++++++++----
 website/docs/reference/evaluation.md |  17 +++--
?? crates/canon/tests/adversary_binding_docs.rs
?? crates/canon/tests/adversary_binding_eval.rs
```
The three modified files are the implementor's uncommitted phase 2. I did not edit them. My only changes are the two new test files, which are untracked.

**2. Cases added (each run alone first, before the suite)**

| Test | Asserts | Now |
|---|---|---|
| `adversary_binding_eval.rs::a_claim_the_stale_record_alone_decided_is_never_false` | Claim `settled: {not: {claim: base, is: unknown}}` is TRUE from e1 alone. Once the case moves `a` to r2, e1 is excluded, and binding.rs:9-10 says the claim should be "`unknown`, never `false`" | red |
| `adversary_binding_docs.rs::no_published_page_says_subject_revision_does_not_affect_evaluation` | First shows that the subject revision alone moves a claim from TRUE to UNKNOWN, then checks that no published page still says it has no effect | red |
| `adversary_binding_eval.rs::several_artifacts_at_different_revisions_bind_each_record_to_its_own_subject` | Probe: two artifacts at different revisions, records in rotated orders. Same render each time; exclusions listed per claim, including through claim references | green |
| `adversary_binding_eval.rs::refusals_win_over_exclusion_and_missing_binding_fields_are_malformed` | Probe: a record that is both undeclared and stale is refused in either order; duplicate ids are refused before binding runs; a record missing either binding field is `malformed-input` | green |

Red output when run alone:
```
assertion `left != right` failed: binding.rs docs: a claim the excluded record alone decided is `unknown`, never `false`
  left: False
 right: False
```
```
pages still say the binding fields do not affect evaluation:
website/docs/reference/documents.md: "they do not yet affect evaluation"
website/docs/reference/investigation-example.md: "subject revision are read but do not yet affect the result"
website/docs/concepts/evidence-and-revisions.md: "do not affect evaluation yet"
website/docs/status/where-this-stands.md: "Records name a revision today, but it does not affect the result"
```

**3. Suite (run after the cases existed)**
- Command: `cargo test --workspace --locked --no-fail-fast` with the brief's env. Exit 101.
- Result: 262 passed, 2 failed, 264 executed (260 + my 4). The only failures are the two cases above.
- Log: `~/.cache/ga-wave-2026-10-04-w7/canon-evidence-revision-binding/scratch/adversary1-suite.log`

**4. Findings**

| # | file:line | Verdict / origin | Measured | What reaches it |
|---|---|---|---|---|
| 1 | crates/canon/src/eval/binding.rs:9 | CONFIRMED / introduced | The doc (and the story's outcome) says exclusion leaves a claim `unknown`, never `false`. In fact a claim built with `is: unknown` flips: `not {claim: base, is: unknown}` goes TRUE→FALSE, and `{claim: base, is: unknown}` goes FALSE→TRUE, so the excluded record also "establishes" a claim | `is: unknown` is part of the documented language (eval/mod.rs:28-30). Any protocol that uses it reaches this when the case snapshot advances. The investigation fixture does not use it. The behaviour is the correct three-valued result, so the fix is to the doc: limit the promise to evidence matches and say that claims testing `is: unknown` are decided |
| 2 | website/docs/reference/documents.md:41 | CONFIRMED / introduced | Four public pages still say subject and revision do not affect evaluation. Their sources: `crates/canon/src/model/evidence.rs:12`, `crates/canon-docs/src/pages.rs:1240`, `website/docs/concepts/evidence-and-revisions.md:35` (hand-written), `website/docs/status/where-this-stands.md:20` | Readers of the published site. The comment at `ess/domains/protocol.yaml:235` is stale in the same way; I did not test it. `generate --check` passes because source and page are stale together |

**5. Attacked and could not break**
- Undeclared and stale in one record: refused, never excluded, in either order.
- A record missing its subject or its revision: refused as `malformed-input` before binding runs.
- Several artifacts at different revisions: each record is checked against its own subject.
- Duplicate ids: refused by `evidence::check` before binding, so `set_aside` cannot double-exclude.
- `excluded_evidence`: sorted, no duplicates, and the same whatever order the evidence comes in.
- The transitive `excluded_for` walk lists stale records under claims that reach a kind through a claim reference.
- The scenario's step 3 still fails if the binding refusal is removed: the record would be excluded and the evaluator would return a decision.
- CANON-CLAIM-001 passes whether binding runs or not, because all its evidence is bound to r1, the current revision. That is not a defect in this unit.
- Deviation: I did not take a worktree session lease, because the `worktree` CLI has no lease verb I could call by hand.

**6. Paths written outside the worktree**
- `~/.cache/ga-wave-2026-10-04-w7/canon-evidence-revision-binding/scratch/adversary1-suite.log`
- `~/.cache/b10x-target/canon-w7-evidence-revision-binding`: the assigned build dir, which now holds my test binaries.

**7.**
```findings
- file: crates/canon/src/eval/binding.rs
  line: 9
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "The doc promises exclusion leaves a claim unknown and never false, but a claim built with is: unknown goes TRUE to FALSE (and its complement FALSE to TRUE) when its only record goes stale, as red case adversary_binding_eval::a_claim_the_stale_record_alone_decided_is_never_false shows."
- file: website/docs/reference/documents.md
  line: 41
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "Four published pages (documents.md:41 from model/evidence.rs:12, investigation-example.md:250 from canon-docs pages.rs:1240, concepts/evidence-and-revisions.md:35, status/where-this-stands.md:20) still say subject and subject revision do not affect evaluation, which this unit made false."
```
