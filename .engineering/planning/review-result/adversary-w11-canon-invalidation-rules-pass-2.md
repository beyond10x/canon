---
format: aep.planning-md/3
id: review-result:adversary-w11-canon-invalidation-rules-pass-2
kind: review-result
status: active
title: Wave 2026-10-04-w11 adversary, canon story:invalidation-rules, pass 2
relations:
- reviews: story:invalidation-rules
revision: 1
---
unit: canon/invalidation-rules, working tree at bdf8284 plus uncommitted phase 2 and the pass-1 fixes, plus my one new test file
verdict: NEEDS-CHANGE
cases: executed 498→507, red 3
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 11 log files in scratch/adversary-p2/ (the copy, its target dir and the backups are deleted)
needs-coordinator: yes. Finding 1 needs a ruling: pass-1 decision 2 still breaks when a named claim reads a record both through its own match and through a helper claim.

**1. Diff stat**
`git --no-pager diff --stat`: `33 files changed, 1189 insertions(+), 181 deletions(-)`. All 33 are the implementor's uncommitted changes. My file `crates/canon/tests/adversary2_invalidation.rs` is untracked, so the stat doesn't list it. I changed no other file.

**2. Cases** (all in `crates/canon/tests/adversary2_invalidation.rs`. The first run was this file alone, saved as `adversary-p2/red.log`, exit 101.)

| Case | Asserts | Now | Shown it can fail |
|---|---|---|---|
| `a_named_claim_that_also_reads_the_record_through_a_helper_lists_it_yet_rests_on_it` | `named: any[{claim: helper}, {evidence: k}]`: a record listed as `invalidated` under `named` does not decide `named` | **red** | — |
| `a_claim_built_on_a_named_one_never_becomes_false_when_only_the_upstream_moves` | the status row: such claims are `UNKNOWN`, never `FALSE`. Here `z: {not: {claim: x, is: unknown}}` | **red** | — |
| `an_unrelated_rule_does_not_change_a_bypass_witness` | adding a rule on `x` leaves the bypass witness of `done` (which reads `k` directly) at `{evidence `k`}` | **red** | — |
| `check_and_evaluate_reach_the_same_claim_combinations_across_a_family` | 3 protocols, a 27-outcome grid each. No claim combination that `evaluate` reaches on 4000 seeded random cases is unreachable for check, and the sets are equal | green | M1 (no vectors) and M1b (first upstream only): red |
| `the_family_has_the_state_counts_the_formula_gives` | 256 / 4096 / 64 states | green | M1: red |
| `excluded_evidence_and_the_explanation_agree_under_every_claim` | `because` exclusions equal `excluded_evidence`, and nothing listed is `applied` (2000 cases × 3 protocols) | green | M2 (explain skips `lists`): red |
| `a_named_claim_whose_only_match_is_under_not_is_not_inert` | validates; goes from FALSE to UNKNOWN and lists `e1` | green | M6 (no claim exclusion): red |
| `inert_invalidation_comes_between_references_and_cycles` | order is undeclared-claim, then inert (only for `on` and `c1`, not `base`), then claim-cycle | green | M3 (no inert check) and M4 (claim test counts as a match): red |
| `the_bound_refusal_names_many_upstream_artifacts` | m=4 gives exactly 65536 states. m=5 refusal text lists `u00`…`u04` and 4294967296. m=32 ends ` 2^(1 * 2^32)` | green | M5 (upstreams not named): red |

Red output, verbatim:
```
assertion `left == right` failed: `named` lists `e1` as invalidated yet its value changes when `e1` is removed
  left: True
 right: Unknown
assertion `left != right` failed: the status row says never FALSE
  left: False
 right: False
  left: ["authority-bypass: outcome `done` is legitimate without authority in state {evidence `k` observed before upstream `up` moved}"]
 right: ["authority-bypass: outcome `done` is legitimate without authority in state {evidence `k`}"]
```

**3. Gate** (unit build dir, run after the cases existed, `adversary-p2/gate2.log`; the first run, `gate.log`, failed fmt on my file, so I rustfmt'd it and ran the gate again)
```
cargo fmt --all --check                                         EXIT=0
cargo clippy --workspace --all-targets --locked -- -D warnings  EXIT=0
cargo test --workspace --locked --no-fail-fast                  EXIT=101  84 summary lines, passed 504, failed 3
  test result: FAILED. 6 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 16.61s
ess specify validate … && ess verify conform synthesize …        EXIT=0  canon v1 — 3 file(s), valid / 0 scenario(s) (0 authored), 0 refusal(s)
cargo run -q --locked -p canon-docs -- generate --check         EXIT=0  canon-docs: 15 generated files are current
cargo run -q --locked -p canon-cli -- conform run               EXIT=0  conform: 10 passed, 0 failed, 0 unreadable
```
`cargo test -- --list` lists 507 tests, and all 9 new names are among them.

**4. Findings** (all on the working tree at bdf8284)

| # | file:line | Verdict / origin | What was measured | What reaches it |
|---|---|---|---|---|
| 1 | `crates/canon/src/eval/invalidation.rs:43` (`lists`) | NEEDS-CHANGE / introduced, warning | `named` passes validation, lists `e1` as `invalidated`, and is TRUE only because of `e1` (UNKNOWN without it). Pass-1 decision 2 still breaks when the named claim also has a match of its own. | Any named claim that reads a kind itself and also tests a claim reading the same kind, e.g. "passed = any[ci_passed, test_run pass]". |
| 2 | `website/docs/status/where-this-stands.md:28`; also `concepts/evidence-and-revisions.md:113` and `ess/domains/protocol.yaml:219` | NEEDS-CHANGE / introduced, note | "become UNKNOWN, never FALSE": `z` (built on `x` through `is: unknown`) goes from TRUE to FALSE with one record when only the upstream revision moves. | Any `is: unknown` or `is: false` test on an invalidated claim. The freshness text has the same wording (`ess:85`, product.json:63); that part is pre-existing. |
| 3 | `crates/canon/src/check/space.rs:329` (`weight`), `check/mod.rs:450` | NEEDS-CHANGE / introduced, note | A moved variant weighs the same as a current record, and in the rendering tie-break `" observed…"` (a space) sorts before `}`. So adding a rule unrelated to the outcome changes the bypass witness to a stale record. This matches the documented tie-break, so this is a judgement call. | Any protocol that adds a rule where a witness dimension gains an upstream. |
| 4 | `ess/domains/protocol.yaml:219` | CONFIRMED / introduced, note | The comment cites "adversary pass 1, coordinator decision 2" (process history in a spec) and the line runs past the file's wrap width. `ess_model_matches` holds; the listing text matches behaviour. | Readers of `ess/`. |

**5. Attacked, not broken**
- The check encoding is exact against `evaluate` across the family: two upstreams on one dimension, a shared upstream, a helper, bound and unbound dimensions, `not` and `is: unknown`.
- Bound arithmetic covers m=4, 5 and 32. The listed numbers multiply to the total wherever the total fits in a `u128`.
- `excluded_evidence` and `because` agree under every explained claim. `because` shows `applied` under a claim only when some reached claim keeps the record (documented).
- `inert-invalidation`: a match under `not` is not inert, a claim built on a named one is refused, and the order matches the docs.
- `ess_model_matches` passes. CLI help and `cli.md` match the code. Minor: "or both, as two records" holds only for one upstream; with m upstreams a class can be up to 2^m records.

**6. Paths written outside the worktree**
- `~/.cache/ga-wave-2026-10-04-w11/canon-invalidation-rules/scratch/adversary-p2/`: `red.log`, `gate.log`, `gate2.log`, `list.log`, `mutant-M1-no-vectors.log`, `mutant-M1b-first-upstream-only.log`, `mutant-M2-explain-no-invalidated.log`, `mutant-M3-no-inert.log`, `mutant-M4-inert-counts-claims.log`, `mutant-M5-bound-unnamed.log`, `mutant-M6-no-claim-exclusion.log`.
- Deleted: `adversary-p2/tree/` (the mutant copy), `adversary-p2/target/` (its build dir), and four `*.orig` backups.
- The gate rewrote `$CARGO_TARGET_DIR/suite.json` in the unit's build dir.

```findings
- file: crates/canon/src/eval/invalidation.rs
  line: 43
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "a named claim with its own match that also tests a helper reading the same kind lists the moved record as invalidated while its TRUE value rests only on that record, so pass-1 decision 2's property still fails on a protocol validation accepts"
- file: website/docs/status/where-this-stands.md
  line: 28
  category: contract-drift
  severity: note
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the status row, concept note and ESS comment say invalidated claims and claims built on them become UNKNOWN never FALSE, yet a claim built on a named one through is: unknown goes from TRUE to FALSE when only the upstream revision moves"
- file: crates/canon/src/check/space.rs
  line: 329
  category: judgement
  severity: note
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "a moved variant weighs the same as a current record and sorts first in the rendering tie-break, so adding a rule unrelated to an outcome changes its authority-bypass witness to a record observed before an upstream move"
- file: ess/domains/protocol.yaml
  line: 219
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the InvalidationRule comment cites adversary pass 1 and coordinator decision 2, process history that will rot in a spec, on a line past the file's wrap width"
```
