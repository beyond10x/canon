---
format: aep.planning-md/3
id: review-result:adversary-w14-canon-review-hardening-w7-pass-2
kind: review-result
status: active
title: Wave 2026-10-04-w14 adversary, canon story:review-hardening-w7, pass 2
relations:
- reviews: story:review-hardening-w7
revision: 1
---
unit: canon/review-hardening-w7, working tree 49fe773 plus the uncommitted phase 2 and pass-1 fixes, plus my 2 untracked test files
verdict: NEEDS-CHANGE
cases: executed 540→545, red 2
origin: introduced 3 / pre-existing 2 / undecided 0
wrote-outside-worktree: 2 locations (part 6)
needs-coordinator: findings 1 and 2 are pre-existing (both red on `a0cc691`) and both break the story's "never aborts" promise. Should they hold this unit or go to a new story?

**1. `git --no-pager diff --stat`**: `42 files changed, 860 insertions(+), 355 deletions(-)`. That is the implementor's diff, unchanged. My only files are the 2 untracked test files `?? crates/canon/tests/adversary2_review_hardening.rs` and `?? crates/canon-cli/tests/adversary2_review_hardening.rs`. I touched no non-test path. Both files were later reformatted with rustfmt only.

**2. Cases added** (each run alone first; logs in `scratch/adversary-p2/case-*.log`)

| Case | Asserts | Now |
|---|---|---|
| lib `a_claim_reached_again_through_a_deeper_test_is_measured_through_it` | `r: any[d, not…d]`: `d` is measured where it is first reached, then reused at the deeper level. Exactly 4096 levels is evaluated; 4097 is refused, naming `r -> d -> c00000 -> …` | green. Red only under mutant M-D1 (finding 3) |
| lib `a_diamond_within_the_bound_is_evaluated` | 2000 claims, each testing the next two (about 2^2000 paths), evaluate with every claim decided | green, 6.2 s in debug |
| lib `claims_kept_apart_by_invalidation_rules_are_evaluated_in_time_linear_in_the_chain` | a chain 8 times longer, one invalidation rule per claim, takes at most 64 times as long | **red**: `a chain eight times as long took 84.157740088s, against 372.684007ms: more than quadratic growth` |
| cli `check_refuses_a_long_chain_of_claims_without_aborting` | `canon check` on 40 000 chained claims (1.6 MB) exits 1 with `predicate-too-deep` | **red**: `ExitStatus(unix_wait_status(134))` / `thread 'main' has overflowed its stack` |
| cli `an_id_in_three_files_names_the_three_in_file_name_order` | one id in `m.yaml`, `z.json` and `a.yaml` gives `evidence/a.yaml, evidence/m.yaml, evidence/z.json: …`, the same on 3 runs | green (red on base, so the feature is new) |

**3. Gate** (run after the cases existed, in the unit build dir; logs in `scratch/adversary-p2/g1–g6.log`)

| Command | Exit | Summary |
|---|---|---|
| fmt | 0 | after formatting my 2 files only |
| clippy | 0 | `Finished \`dev\` profile [unoptimized] target(s) in 0.12s` |
| cargo test --workspace --no-fail-fast | 101 | 97 result lines: 543 passed, 2 failed, 3 ignored. The 2 failures are exactly my red cases, so this tree's tests ran. The FAILED lines: `test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 88.50s` and `test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 22.01s`. All 97 lines verbatim: `g3-summaries.txt` |
| ess | 0 | `canon v1 — 3 file(s), valid`, `0 scenario(s) (0 authored), 0 refusal(s)` |
| canon-docs --check | 0 | `canon-docs: 16 generated files are current` |
| conform run | 0 | `conform: 11 passed, 0 failed, 0 unreadable` |

**4. Findings**

| # | file:line | Verdict / origin / severity | Measured | What reaches it |
|---|---|---|---|---|
| 1 | `crates/canon/src/validate/mod.rs:513` | NEEDS-CHANGE / pre-existing / warning | The validator's `claim_cycles` recurses along claim references on the caller's thread, before `check::check`'s depth bound. Core dump frame: `validate::claim_cycles` (`marks.insert`). Debug: 10k claims exit 1, 20k and 40k abort. Release: `check` and `compile` abort at 40k (6–7 s) and at 100k (48–73 s, so compile time grows faster than linear) | Any protocol author, through `canon check`, `compile`, `validate`, `conform run`, and `generate` (int tree 2ab14a5 compiles first). Fix: an iterative DFS, as in `depth.rs` |
| 2 | `crates/canon/src/eval/claims.rs:66`, `eval/invalidation.rs:122` | NEEDS-CHANGE / pre-existing / warning | Evaluation time grows about cubically: 50 claims 0.37 s, 400 claims 84 s. Base is the same: 0.48 s and 87.6 s. The depth bound admits about 4000 such claims. Likely causes, from a profile I did not attribute: per-context memo keys clone the `EvidenceId` set; the `invalidated_claims` fixpoint rescans every claim per round; `Invalidated::lists` → `reached_matches` runs per record | A protocol with one invalidation rule per claim of a long chain. I found no such protocol in the tree |
| 3 | `crates/canon/src/eval/depth.rs:141` | CONFIRMED / introduced / note | Mutant M-D1 (`if false && level + below > …`): every other `b10x-canon` test stays green. Only my reached-again case goes red: `4097 levels through the second test are refused: Decision {…}` | A test gap only; my case closes it |
| 4 | `conformance/scenarios/review-hardening-w7.yaml:40` | CONFIRMED / introduced / note | `present: true` with a subject appears only under `outcomes` (line 121). `actions` pins only `false`. M-R1 (actions passes `&[]` to `unmet`): `conform: 11 passed`, and only the 2 `eval::actions` unit tests catch it. M-R2 (outcomes passes `&[]`) is caught by CANON-REASONS-001 alone: `failed: scenario \`CANON-REASONS-001\` step \`about-the-subject\`` | The p1fix report's "present both false and true" holds across the two sections, not within each |
| 5 | `website/docs/status/where-this-stands.md:31` | CONFIRMED / introduced / note | The row lists CANON-REASONS-001 among the ids the scenarios cover. That scenario's `covers:` is `[CANON-EVIDENCE-003]`; every other id in the row is a covered id. Line 47 has it right | Readers of the status page |

**5. Attacked and held**
- **Effective depth is the real recursion depth:** one `Walk::predicate` frame per level; a claim test adds `value_in`; `is:` adds nothing. The obligations, actions and outcomes sections look claim values up rather than recursing into claims.
- **Diamond:** `depth.rs` measures each claim once and runs in linear time. Evaluation memoizes per context, so a diamond without invalidation is linear.
- **4096 consistency:** `read_ir`'s JSON bound is stricter than the predicate bound. `evaluate_with` and `check::check` call the same `depth::check` first. The explanation walks are iterative, and `visit` recursion is bounded by a predicate's own depth.
- **Citations:** every refusal of an evidence record cites it: the `check_record` refusals (format, identifiers, `observed_at`, repeated id, undeclared kind, upstream undeclared or repeated) and `binding.rs`. The `invalidation.rs:86` refusal names a rule, not a record. File naming with three files is deterministic.
- **Exit 64:** ELS calls the library, not the binary (`crates/els/tests/support/mod.rs`). Nothing in canon expects exit 2 for a usage error; generate's exit-2 test is a write failure.
- **Docs:** the Depth section matches `depth.rs`, and the authority section matches decision 7.

**6. Written outside the worktree**
- `~/.cache/ga-wave-2026-10-04-w14/canon-review-hardening-w7/scratch/adversary-p2/`: 18 logs, 1.8M, kept. I deleted my scratch copies (`tree/`, `base/`), their target dirs (753M, 163M, plus an 84M release build) and the probe protocols.
- Build dir `~/.cache/b10x-target/canon-w14-review-hardening-w7/`: my test binaries, and the gate rewrote `suite.json`. I deleted the probe dirs under `tmp/`: mine (`adversary2-rh-w7-*`) and the pass-1 dirs my gate run recreated (`adversary-rh-w7-*`, 35M).
- `/var/lib/systemd/coredump`: 10 SIGABRT cores from my runs, 1.3–3.5M each. Not deleted. Each suite run adds one more while finding 1 stays red.
- Free space dropped to 9.3G during my release probe, while other agents were also building. I stopped and deleted my builds, and it recovered to 20G before I built again. It was 15G at the end.
- I found no lease verb in the `worktree` CLI, so I took no explicit session lease.

```findings
- file: crates/canon/src/validate/mod.rs
  line: 513
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: "claim_cycles recurses along claim references on the caller's thread, so canon check, compile, validate, conform run and generate abort with a stack overflow on 40000 chained claims (debug and release) instead of reaching the predicate-too-deep refusal."
- file: crates/canon/src/eval/claims.rs
  line: 66
  category: property
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: "Evaluating a chain of claims with one invalidation rule each grows about cubically (50 claims 0.37 s, 400 claims 84 s, the same on a0cc691), while the depth bound admits chains of about 4000."
- file: crates/canon/src/eval/depth.rs
  line: 141
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "Ignoring a memoized claim's depth when it is reached again at a deeper level survives every existing test; only the new reached-again case kills it."
- file: conformance/scenarios/review-hardening-w7.yaml
  line: 40
  category: acceptance
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "CANON-REASONS-001 pins present true with a subject only in outcomes; an actions mutant that drops the evidence passes all 11 scenarios and is caught only by eval::actions unit tests."
- file: website/docs/status/where-this-stands.md
  line: 31
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "The conformance row lists CANON-REASONS-001 as a covered id, but that scenario covers CANON-EVIDENCE-003; it is a scenario id, not a covered requirement."
```
