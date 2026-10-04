---
format: aep.planning-md/3
id: review-result:adversary-w4-canon-three-valued-claims-pass-2
kind: review-result
status: active
title: Wave 2026-10-04-w4 adversary, canon story:three-valued-claims, pass 2
relations:
- reviews: story:three-valued-claims
revision: 1
---
unit: canon/three-valued-claims, working tree at 4c33a4d plus uncommitted phase 2 and pass-1 fixes, in ~/.local/state/worktree/trees/b10x/canon/canon-w4-three-valued-claims
verdict: NEEDS-CHANGE is not returned; 3 red cases (2 findings, both INFEASIBLE) plus 2 surviving mutants (CONFIRMED)
cases: executed 204→210, red 3
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: 6 paths in the assigned scratch dir, 1 build dir now deleted, plus system core dumps (part 6)
needs-coordinator: F5 is about the plan for story:evaluator-skeleton, not about this unit

**1. Diff.** `git --no-pager diff --stat` shows only the implementer's tracked changes: 11 files, +1149/−150, identical to what I was handed. My two files are new and untracked, so `diff --stat` cannot list them. `git status --short` shows them:
```
?? crates/canon-cli/tests/adversary2_tvc_cli.rs
?? crates/canon/tests/adversary2_tvc_eval.rs
```
I changed no implementation file. I ran `rustfmt` on my two files only.

**2. Cases added.** Each was red or green when first run alone, as shown below.

| Case | Asserts | Now |
|---|---|---|
| `adversary2_tvc_eval.rs` `the_ir_of_a_deeply_nested_predicate_the_compiler_accepts_reads_back` | A 123-deep `not` compiles, so `read_ir(canonical_json)` must return that IR | red |
| `adversary2_tvc_eval.rs` `an_ir_with_a_claim_cycle_is_refused_not_a_stack_overflow` | `evaluate` on a hand-built IR whose claim tests itself does not crash. Runs in a child process | red |
| `adversary2_tvc_eval.rs` `ir_text_describing_a_claim_cycle_is_refused_as_a_cycle` | `read_ir` refuses cyclic IR text with `claim-cycle` | green |
| `adversary2_tvc_eval.rs` `a_claim_tested_twice_at_every_level_is_evaluated_once` | 40 levels, each claim testing the next twice, finish within 20 s with c00 = true | green; red with the memo removed |
| `adversary2_tvc_cli.rs` `what_canon_compile_prints_for_a_deep_predicate_canon_evaluate_accepts` | The same 123-deep IR goes from `canon compile` to `canon evaluate` with exit 0 | red |
| `adversary2_tvc_cli.rs` `the_first_refusable_record_by_file_name_is_the_one_named` | With 26 bad records, the refusal names `e-a` | green; red with the sort reversed |

Red output, verbatim (logs: `scratch/adversary2-tvc-eval-red.log`, `adversary2-tvc-cli-red.log`):
```
serde_json alone: recursion limit exceeded at line 135 column 267
assertion `left == right` failed: the IR canon compile printed for a 123-deep `not` is refused
  left: Err(Refusal { code: "malformed-input", message: "not canon-ir/1: not well-formed JSON" })
evaluating a cyclic IR crashed the process (signal: 6 (SIGABRT) (core dumped)): fatal runtime error: stack overflow, aborting
assertion `left == right` failed: the IR canon compile printed is refused: error[malformed-input]: not canon-ir/1: not well-formed JSON
  left: Some(1)  right: Some(0)
```
Mutants, run in copies under scratch with build dir `canon-w4-mutants` (log: `scratch/adversary2-mutants.log`):

| Mutant | My case | Unit suite without my files |
|---|---|---|
| `claims.rs:26` memo disabled (`.filter(|_| false)`) | timeout at 20 s, exit 101 | exit 0, 204 passed |
| `main.rs:116` `names.sort(); names.reverse();` | names `e-z`, exit 101 | exit 0, 204 passed |

**3. Suite run, after the cases existed.** `cargo test --workspace --locked --no-fail-fast` gave exit 101: 207 passed, 3 failed, 210 executed. The 3 failures are the red cases above. `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --locked -- -D warnings` both exit 0.

**4. Findings**

| # | file:line | Verdict / origin | What was measured | What reaches it |
|---|---|---|---|---|
| F1 | `crates/canon/src/eval/read.rs:68` | INFEASIBLE / introduced | `read_ir` says it "accepts exactly the bytes `canon compile` prints". serde_json's default depth limit of 128 refuses a 123-deep `not` that `canon compile` accepts, and the message wrongly says "not well-formed JSON". `canon conform run` never reads IR text, so it passes where `canon evaluate` refuses. | The documented compile → evaluate pipeline, but only with a predicate at least 123 deep. No real protocol comes near that. Fix: either the validator refuses nesting deeper than the IR reader can read back, or `read_ir` lifts the limit (`unbounded_depth` + `disable_recursion_limit`) and gives an accurate message. |
| F2 | `crates/canon/src/eval/claims.rs:20` | INFEASIBLE / introduced | `evaluate` is public and every field of `Ir` is public. A self-testing claim overflows the stack and aborts the process instead of returning a Refusal. | Nothing found. `read_ir` and `conform` both recompile, and the validator refuses the cycle (green case above). Fix: an in-progress mark in `value_of` that returns a Refusal, a non-constructible `Ir`, or the precondition stated on `evaluate`. |
| F3 | `crates/canon/src/eval/claims.rs:26` | CONFIRMED / introduced | Removing the memo keeps the unit suite green, and evaluation becomes exponential. | Any protocol that tests the same claim twice per level. story:evaluator-skeleton is about to split this file, which is when the memo could get lost. |
| F4 | `crates/canon-cli/src/main.rs:116` | CONFIRMED / introduced | Reversing the documented "sorted file-name order" keeps the unit suite green. | Any evidence directory holding two refusable records. |
| F5 | `crates/canon/src/conform/mod.rs:488` | INFEASIBLE / introduced | Evaluate steps compare the whole decision byte for byte. story:evaluator-skeleton puts `excluded_evidence` inside each `claims` entry and says CANON-CLAIM-001 "unedited, passes", but its tolerance is per section. All 4 expectations list `claims` entries, so a new field inside an entry breaks them unless the slot is left out when empty. | That story's plan. The format itself, one object per claim, does allow the extension. |

**5. Attacked and could not break**
- **Cycles through the CLI or conform:** refused as `claim-cycle`.
- **Long claim chains:** `canon compile` hits a stack overflow at about 17k claims before `evaluate` is reached (debug build). That is pre-existing compiler behaviour.
- **Deep `all` nesting:** the YAML reader's limit stops it at 62 levels, before the JSON limit.
- **Evidence directory entries:**
  - A symlink to a file outside the directory is read, as documented.
  - A symlink to a directory, a dangling or looping symlink, a FIFO, or `/dev/null` is refused with exit 2, and reading does not block.
  - Two symlinks to the same file are refused as `duplicate-identifier`.
- **Evidence for another case:** a `case:` field is refused as unknown, so it cannot be written.
- **Subject or revision mismatch:** ignored, as the story intends.
- **`canon-decision/1` against the ess/ Decision type:** field names, order, `Truth` wire values and the absence of nulls all match.
- **Refusal expectations in evaluate steps:** compared by code, no gap found.

**6. Paths written outside the worktree** (scratch = `~/.cache/ga-wave-2026-10-04-w4/canon-three-valued-claims/scratch`)
- Kept: `scratch/adversary2-tvc-eval-red.log`, `scratch/adversary2-tvc-cli-red.log`, `scratch/adversary2-suite.log`, `scratch/adversary2-mutants.log`, `scratch/p2-mutant-memo.suite.log`, `scratch/p2-mutant-sort.suite.log`.
- Deleted: `~/.cache/b10x-target/canon-w4-mutants` and the scratch probe copies and probe script.
- System: core dumps from systemd-coredump, about 3 MB under `/var/lib/systemd/coredump`. Each run of the F2 red case adds one.
- I also built into the brief's `CARGO_TARGET_DIR`.
- `scratch/adversary2-conform-lines.patch` was already there and is not mine.

```findings
- file: crates/canon/src/eval/read.rs
  line: 68
  category: boundary
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: "read_ir refuses as 'not well-formed JSON' the IR canon compile prints for a 123-deep predicate (serde_json depth limit 128), so canon evaluate rejects valid compiler output that canon conform run accepts"
- file: crates/canon/src/eval/claims.rs
  line: 20
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "public eval::evaluate on a library-built Ir whose claim tests itself overflows the stack and aborts the process instead of returning a Refusal"
- file: crates/canon/src/eval/claims.rs
  line: 26
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "disabling the claim-value memo leaves the unit suite green (204 passed) while evaluation turns exponential; adversary2 case catches it"
- file: crates/canon-cli/src/main.rs
  line: 116
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "reversing the documented sorted file-name order of evidence records leaves the unit suite green (204 passed); adversary2 case catches it"
- file: crates/canon/src/conform/mod.rs
  line: 488
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "byte-for-byte decision comparison means an excluded_evidence field inside each claims entry breaks all 4 CANON-CLAIM-001 expectations, contradicting story:evaluator-skeleton's 'unedited, passes' unless the slot is omitted when empty"
```
