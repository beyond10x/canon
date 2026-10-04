---
format: aep.planning-md/3
id: review-result:adversary-w7-canon-obligations-pass-2
kind: review-result
status: active
title: Wave 2026-10-04-w7 adversary, canon story:obligations, pass 2
relations:
- reviews: story:obligations
revision: 1
---
unit: canon/obligations, working tree at b38a76e plus uncommitted phase 2 and the pass-1 and pass-2 test files
verdict: NEEDS-CHANGE (1 finding: the gate's `cargo fmt --all --check` step fails on the pass-1 test file; I found no defect in the obligations code)
cases: executed 260→268, red 0
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 path (the assigned build dir)
needs-coordinator: run `cargo fmt` on crates/canon/tests/adversary_obligations_order.rs before committing. As an adversary I may not rewrite an existing case.

**1. Diff stat**
```
 crates/canon-cli/tests/adversary_skel_cli.rs |  41 +++++++---   (coordinator's)
 crates/canon/src/eval/obligations.rs         | 113 +++++++++++++++++++++++++--   (implementor's)
```
Untracked: `crates/canon/tests/adversary2_obligations_ir.rs` (mine) and `adversary_obligations_order.rs` (pass 1). I changed no implementation path.

**2. Cases added** in `~/.local/state/worktree/trees/b10x/canon/canon-w7-obligations/crates/canon/tests/adversary2_obligations_ir.rs`. All 8 are green now. The first single-file run had 3 failures, all from my own YAML indentation typo in `EVERY_FORM` ("unknown field `o2.false`"). That was not a finding, and I fixed it.

| Case | What it asserts |
|---|---|
| `every_discharge_predicate_survives_read_ir_and_decides_the_same` (:74) | 8 predicate forms: `is: true/false/unknown`, `not`, `all`, empty `any`, empty `all`, `any` mixed. Each survives the `read_ir` round trip and gives the literal statuses over 3 evidence sets. |
| `read_ir_refuses_a_hand_written_discharge_predicate_the_validator_refuses` (:137) | `read_ir` refuses a hand-written canon-ir/1 whose predicate names an undeclared claim (`undeclared-claim`) or tests evidence (`evidence-in-discharge`). |
| `an_undeclared_claim_is_unknown_to_obligations_as_to_claims` (:175) | In an IR built by hand, an undeclared claim is `unknown` in obligations and in claims alike. |
| `obligation_ids_that_need_escaping_render_as_json_and_read_back` (:206) | Ids containing a quote, a backslash, `}],`, `\n`, NUL, U+2028, DEL and tab parse back exactly, in code-point order. |
| `the_obligations_section_is_the_same_bytes_…` (:241) | The output is byte-identical across runs, threads and evidence orders. |
| `a_recorded_termination_leaves_the_obligations_section_unchanged` (:267) | `termination: supported` does not change any status on the unit's own fixture. |
| `every_status_follows_the_claim_values_the_decision_reports` (:311) | All 32 subsets of 5 records, including records bound to revision r0: each status equals its predicate over `decision.claims`. This will catch an obligations section that reads evidence from before the exclusion stages, once those stages land. |
| `a_discharge_predicate_nested_to_the_ir_bound_reads_and_evaluates` (:403) | A predicate nested exactly to `MAX_IR_DEPTH` reads and evaluates on a 2 MiB test thread, and the parity of the nested `not`s decides the status. |

**3. Suite run after the cases existed**
- `cargo test --workspace --locked` exited 0. The 44 summary lines add up to 268 passed: 260 before plus my 8. `--list` shows all 8 tests in this tree.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: CLIPPY=0.
- `cargo fmt --all --check`: FMT=1, with diffs in `crates/canon/tests/adversary_obligations_order.rs` at :47, :57, :64 and :70. My own file passes.

**4. Finding**

| File:line | Verdict / origin | What was measured | What reaches it |
|---|---|---|---|
| crates/canon/tests/adversary_obligations_order.rs:47 | NEEDS-CHANGE / introduced | `cargo fmt --all --check` exits 1. Four hunks: rustfmt splits the `ids` array, the `rendered` binding, the `assert!` at :65 and the `if/else` at :73 over several lines. | Gate step 1 in the brief, and CI. The "unit is green" count did not cover fmt. Fix: `cargo fmt --all`. |

**5. Attacked, could not break**
- **Mutant that returns `discharged` for every obligation:** killed by obligations.rs:83 (the Unknown row expects open), the pass-1 test at :79 onwards (odd entries are open), the `claim-unknown` and `claim-false` steps of CANON-OBLIGATION-001, the CLI test's `SKELETON_NO_EVIDENCE_DECISION`, and my :74.
- **Mutant that reverses the order:** killed by the row order at obligations.rs:83, pass-1 :59 and :81, and my :74.
- **Mutant that passes `&applicable` instead of `&[]`:** killed by the pass-1 evidence-match case.
- **Determinism:** there is no `HashMap` or `HashSet` in `crates/`. `Ir.obligations` is a `BTreeMap`, and the renderer re-keys every object.
- **A claim excluded by another stage:** all three exclusion stages are still stubs, so this cannot be simulated through the public API. My :311 property is the guard for after the merge.
- **Termination:** obligations.rs never reads `case.termination`. An open obligation in a terminated case stays open, and no refusal comes from this section.

**6. Paths written outside the worktree**
- `~/.cache/b10x-target/canon-w7-obligations`, the assigned build dir.

```findings
- file: crates/canon/tests/adversary_obligations_order.rs
  line: 47
  category: judgement
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "The pass-1 adversary test file is not rustfmt-clean (hunks at :47, :57, :64, :70), so gate step `cargo fmt --all --check` exits 1 on this tree."
```
