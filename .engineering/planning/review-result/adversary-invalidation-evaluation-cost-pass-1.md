---
format: aep.planning-md/3
id: review-result:adversary-invalidation-evaluation-cost-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:invalidation-evaluation-cost (wave 2026-11-20-w2): green, 1 note, introduced'
relations:
- reviews: story:invalidation-evaluation-cost
revision: 1
---
Adversary pass 1 on story:invalidation-evaluation-cost, wave 2026-11-20-w2, `aep:adversary` on Opus.
The report below is as the agent returned it.

unit: story:invalidation-evaluation-cost, branch impl/invalidation-evaluation-cost at 2742538 (base 15cfece)
verdict: green
cases: executed 919→919, red 0
origin: introduced 1, pre-existing 0, undecided 0
wrote-outside-worktree: 1 path, the assigned scratch directory (`$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w2/invalidation-evaluation-cost/adversary-p1/`)
needs-coordinator: no

I found no output difference against the base, so I added no case and made no commit. The one finding is a gap in the tests, not a defect: the new per-walk memo has no test that would fail if it were removed.

**1. Diff stat.** `git --no-pager diff --stat` is empty and `git status --short` is empty. The tree is as I was handed it, and the lease is released.

**2. Cases added.** None. Every differential comparison was identical, so there was no shrunk case to pin.

**3. Suite run.** `zsh $SCRATCH/suite.sh` (`cargo test --workspace --locked`, ess 0.77.0 on PATH), run once in the background with no case of mine present:
- `suite exit 0`. 132 `test result:` lines: 919 passed, 0 failed, 3 ignored.
- `invalidation_evaluation_cost.rs`: `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.30s`
- The implementor reported 684 tests over 82 result lines, so their count measured a narrower selection than this one. `before` is therefore this same run.

**4. Findings** (covering 2742538)

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| `crates/canon/src/eval/claims.rs:290` | mutant | note | CONFIRMED | introduced | No test fails if this line is dropped. Shape that needs it: a diamond where each claim has its own rule. |

- **What the line does:** it stores a value in the per-walk memo (`alone`). That path is used when a context is the starting context of only one claim, which is the case for every claim of a chain or diamond that carries its own rule.
- **Why no test catches its removal:**
  - Nothing checks the memo's effect on values: a pure function gives the same value with or without it. Only cost would change.
  - `adversary2_review_hardening.rs:88` (a diamond of 2000 claims) has no rules, so it only uses the shared memo.
  - The cost tests are a chain, which has no diamond.
- **What removing it would do:** a 2000-claim diamond with one rule per claim would cost on the order of fib(2000) instead of n²/2.
- **How established:** by reading. I could not build the mutant, because a second copy of the tree is not allowed.
- **Suggested test (not added):** that diamond, with rule `r_i` invalidating `c_i` and one moved record per rule, evaluated within a time budget.

**5. Attacked and not broken**

Differential runs, base oracle (`canon 0.9.0` at 6f44325, it runs) against the head release binary. Each comparison checks stdout, stderr and exit status. Generators use fixed seeds.
- **Protocols generated:** 7900, compared under `validate`. 4926 of them compiled, and their IR and `compile` output were compared too. 314 were also compared under `check`.
- **What the generators covered:**
  - shapes: diamonds; chains whose declared order differs from their test order; `not` and `any` branches over other kinds; `is: true|false|unknown`;
  - rules: 0 to 14, several on one upstream, undeclared names, inert rules;
  - claims: cycles, claims declared twice;
  - evidence: tricky ids (`e1/e01/E1/e10`), file order different from id order, subject-revision mismatch, mixed old/new upstream revisions per record, producers and `independent_on`;
  - outcomes and obligations, so explanations get listed.
- **Evaluations compared:** 14778, all decided. They hold 244637 claim and explanation `value` entries and 120408 `invalidated` exclusions. Differences: 0.
- **Fixtures:** 213 runs of validate, compile and check over 71 protocol fixtures, all `check` `report.txt` cases included, gave 0 differences. `canon conform run` output was byte-identical: `conform: 19 passed, 0 failed, 0 unreadable`.
- **Probes by hand:** a chain where every claim also reads `k`, at 50, 100, 200 and 400 claims, gave identical bytes. Head took 0.009 to 0.107 s, the oracle 0.18 to 72 s. A duplicate claim plus cycle plus undeclared-name protocol gave identical `validate` output.

Checked by reading:
- **The invariant** (a tested claim's invalidated set is contained in the tester's): it holds by construction. Records move from a named claim to every claim that tests it, and the index is built from `claim_references`, the same edges evaluation follows.
- **`reached_by_claim`:** skipping a claim already entered only drops matches on a cycle, and compile refuses cycles.
- **Evaluation refusals cannot be reached from the CLI:** `read_ir` recompiles and validates the IR, so `claim-cycle` and `undeclared-artifact` only come from an IR a library caller builds. That is why the differential saw no evaluation refusals.
- **Skipping the not-independent pass:** with no demanding match, `accepts` is always true, so the skip changes nothing.
- **`validate` rewrite:** it reads the first declaration and the same edges as before, so problems come out in the same order.

Timing tests:
- The quadratic-ratio test ran 5 times in debug: about 1.85 ms against 54 to 58 ms, a ratio of 29 to 31 against a bound of 64. That is about 2× headroom, and the lock serialises the two timed tests. I saw no flakiness.
- The ratio test alone would not catch a small-constant cubic term at 50/400 claims. The 4000-claim test (12.3 s against 120 s in debug) would.

Two theories I could not turn into evidence:
- **Hash spread:** the fixed hash puts the test's own `c%05d` ids into 32 of 8192 bucket starts. A CLI timing against random ids was dominated by noise and the 700 MB output, so I am not claiming a cost.
- **Cost per record:** each evidence-match evaluation scans every record, so the cost also grows with the record count. The depth bound caps that shape at about 1950 claims, and 1600 claims took 1.98 s in release.

**6. Written outside the worktree.** Only `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w2/invalidation-evaluation-cost/adversary-p1/`: the generators (`diff.py`, `diff_big.py`, `diff_dup.py`, `why.py`, `everyread.py`, `hashprobe.py`), the shell scripts, the run logs and `run-*` directories, `probes/`, `ir/`, `every/` and `build-release.log`. About 4.3 MB; I deleted the large outputs. The head release binary is in the worktree's own `target/release/canon`.

```findings
[{"file": "crates/canon/src/eval/claims.rs", "line": 290, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Dropping the per-walk memo store leaves every test green: the only diamond cost test has no rules and so uses only the shared memo, and a diamond whose claims each carry a rule would then cost exponential time."}]
```
