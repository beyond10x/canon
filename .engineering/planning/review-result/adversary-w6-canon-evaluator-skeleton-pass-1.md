---
format: aep.planning-md/3
id: review-result:adversary-w6-canon-evaluator-skeleton-pass-1
kind: review-result
status: active
title: Wave 2026-10-04-w6 adversary, canon story:evaluator-skeleton, pass 1
relations:
- reviews: story:evaluator-skeleton
revision: 1
---
```
unit: canon/evaluator-skeleton, worktree canon-w6-evaluator-skeleton at 962ac65 plus uncommitted phase 2
verdict: NEEDS-CHANGE
cases: executed 224→229, red 2
origin: introduced 6 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 paths (logs in scratch); no extra build dir
needs-coordinator: yes. Choose the fix for F1 and F3: the validator refuses evidence in section predicates, or the sections get the applicable evidence. That choice belongs to mod.rs/validate/, which the wave-7 stories cannot edit.
```

**1. `git --no-pager diff --stat`.** It is the same 14 files the implementor changed, and none of them is mine. The tree has two new files, both added by me and both test files: `crates/canon-cli/tests/adversary_skel_cli.rs` and `crates/canon/tests/adversary_skel_lib.rs`. I changed no implementation file.

**2. Cases I added.** Each file was run alone first.

| Case | Asserts | Now |
|---|---|---|
| `adversary_skel_cli::a_discharge_predicate_that_tests_evidence_is_refused` | `canon validate` refuses a `discharged_when: {evidence: …}`, because the model and ess say the predicate is "over claim values" | red |
| `adversary_skel_cli::a_declared_obligation_changes_no_decision` | evaluating the skeleton fixture, which has an obligation, gives exactly the base decision bytes | green |
| `adversary_skel_cli::a_recorded_termination_changes_no_decision` | a case with `termination: supported` or `abandoned` gives the same bytes | green |
| `adversary_skel_lib::listed_sections_each_byte_identical_pass_in_any_order` | two listed sections, each byte-identical, pass in either order | red |
| `adversary_skel_lib::unresolved_discharge_references_come_after_claims_and_before_actions` | the order the validate docs give: claims, then obligations, then actions, then outcomes | green |

Red output, verbatim:
```
assertion `left == right` failed: a discharge predicate over evidence is refused; stdout: valid: protocol `investigation` revision 1
  left: Some(0)
 right: Some(1)
```
```
assertion `left == right` failed: each listed section equals the decision's byte for byte
  left: Failed { step: "e", reason: "canon-decision/1 differs from the expectation at line 2" }
 right: Passed
```

**3. Suite.** I ran `cargo test --workspace --locked --no-fail-fast` in the brief's target dir after the cases existed, and it exited 101. Summary lines added up: 227 passed, 2 failed (the two cases above), 229 run. The "224" before count is 229 minus my 5. `cargo fmt --all --check` exits 0, and clippy with `-D warnings` on both crates' tests is clean. The test names in the run are the ones in my files.

**4. Findings**

| # | file:line | What I measured | What reaches it | Verdict |
|---|---|---|---|---|
| F1 | crates/canon/src/validate/mod.rs:202 | A discharge predicate that tests evidence passes validation (red case). `obligations::section` only gets claim values (eval/mod.rs:506), so story:obligations cannot evaluate it inside its own file. | Any protocol author. The predicate language allows `evidence:` anywhere. | NEEDS-CHANGE, blocker |
| F2 | crates/canon/src/eval/claims.rs:13 | The only `pub(super)` item is `values`. `Walk::predicate`, `all`, `any` and `not` are private, so there is no shared way to evaluate a predicate over claim values. obligations, actions and outcomes each need one (for discharge, precondition and requires), so each must either copy the three-valued logic or edit claims.rs. If they edit claims.rs, three parallel stories conflict on one file. | The wave-7 plan itself. | NEEDS-CHANGE, warning |
| F3 | crates/canon/src/eval/mod.rs:507-508 | `actions::section` and `outcomes::section` get no evidence either, but the validator (pre-existing) accepts evidence nodes in preconditions and requires. That is the same hole as F1 for two more stories. | Any protocol with `precondition: {evidence: …}`. | NEEDS-CHANGE, warning |
| F4 | crates/canon/src/eval/mod.rs:543-610 | `set_aside` and `excluded_for` cannot be reached: every stage is a stub, so the early return at :567 always fires. No test covers them, and the later stories may not edit this file. | story:evidence-revision-binding is the first code to reach it. | CONFIRMED, warning |
| F5 | crates/canon/src/conform/mod.rs:99 | Sections that each match byte for byte fail when listed out of canonical order, and the message names no section (red case). The story says "each byte for byte"; the module docs say the whole filtered document is compared. | A scenario author writing keys out of order. | CONFIRMED, note |
| F6 | crates/canon/src/eval/mod.rs:574 | `excluded_for` only looks at the claim's own `true_when`. A claim that depends on an excluded kind through another claim lists nothing, and no story settles which rule is right. | story:evidence-revision-binding and story:evidence-freshness expectations. | CONFIRMED, note |

Mutants were reasoned from the code, not built, because the coordinator told me to build only in the brief's dir:
- **Survived the unit's suite, now killed by my cases:**
  - `obligations::section` returning `Some` when the protocol declares obligations. Nothing evaluated an obligation, and CANON-CLAIM-001 now ignores sections it does not list.
  - `outcomes` refusing any termination. No test anywhere reads a case with a `termination` field.
  - The obligations validation loop moved after the actions loop.
- **Still surviving (F4):** dropping `listed.sort()`, inverting `kinds.contains`, and `set_aside` adding a record to the excluded list without removing it.

**5. What I attacked and could not break**
- **CANON-CLAIM-001:** still byte-identical, and the base decision is unchanged with an obligation or a termination present.
- **Section comparison:** no expectation passes that should not. An omitted section passes; an added section the decision lacks fails; duplicate keys, a lone `{}` and non-object expectations all fail.
- **Limits and isolation:** `DEEP_STACK` is still 64 MiB and the whole pipeline runs on it. claims.rs is unchanged from the base, so the memo and cycle guard are kept. The library does no file or environment access.
- **Per-story files:** binding, freshness and action-admissibility can each work in their own file. The `--at`, `--authority` and `diff` stubs behave as the story says.
- **discharged_when:** an undeclared claim is refused. The value is normalised in the IR and survives `read_ir` through the CLI.
- **Supplied.decisions:** no CLI flag or scenario input reaches it, so its refusal can never fire. It is harmless and out of the story's scope.

**6. Paths written outside the worktree**
- `~/.cache/ga-wave-2026-10-04-w6/canon-evaluator-skeleton/scratch/adv1-lib.log`
- `~/.cache/ga-wave-2026-10-04-w6/canon-evaluator-skeleton/scratch/adv1-cli.log`
- `~/.cache/ga-wave-2026-10-04-w6/canon-evaluator-skeleton/scratch/adv1-suite.log`
- The test binaries were built in the brief's `~/.cache/b10x-target/canon-w6-evaluator-skeleton`. `canon-w6-mutants` was never created.

**7. Findings block**

```findings
- file: crates/canon/src/validate/mod.rs
  line: 202
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "A discharge predicate that tests evidence validates, though model and ess define it over claim values and obligations::section receives only claim values, so story:obligations cannot evaluate it without editing mod.rs or validate/."
- file: crates/canon/src/eval/claims.rs
  line: 13
  category: judgement
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "No shared predicate-over-claim-values evaluator is exposed, so obligations, actions and outcomes must each duplicate the three-valued logic or edit claims.rs concurrently."
- file: crates/canon/src/eval/mod.rs
  line: 507
  category: judgement
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "actions::section and outcomes::section get no evidence while the validator accepts evidence nodes in preconditions and requires, so those stories cannot evaluate a valid protocol within their own files."
- file: crates/canon/src/eval/mod.rs
  line: 567
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "set_aside and excluded_for are unreachable while every stage is a stub, so mutants in them survive the suite and the first defect lands on a story forbidden to edit mod.rs."
- file: crates/canon/src/conform/mod.rs
  line: 99
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "Listed sections each byte-identical to the decision fail when written out of canonical order, contrary to the story's each-byte-for-byte wording, and the failure names no section."
- file: crates/canon/src/eval/mod.rs
  line: 574
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "Exclusions are attributed only to claims whose own true_when names the kind, not to claims depending on it through a claim reference, a policy no story settles and later stories cannot change."
```
