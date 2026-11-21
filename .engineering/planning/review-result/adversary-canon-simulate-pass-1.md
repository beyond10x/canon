---
format: aep.planning-md/3
id: review-result:adversary-canon-simulate-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:canon-simulate (wave 2026-11-20-w4): red, 3 red cases, 4 warnings and 2 notes, introduced'
relations:
- reviews: story:canon-simulate
revision: 1
---
Adversary pass 1 on story:canon-simulate, wave 2026-11-20-w4, `aep:adversary` on Opus, on `impl/canon-simulate` at `e82f6de`. The agent was stopped once by a model limit and resumed. The report below is as the agent returned it; its clippy run on the new test file was still queued when it reported.

unit: story:canon-simulate, branch impl/canon-simulate at 9c92fd1 (findings cover e82f6de plus one test commit)
verdict: red. 3 cases red, 3 judgement findings
cases: executed 1001→1004, red 3
origin: introduced 6, pre-existing 0, undecided 0
wrote-outside-worktree: 1 path, `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w4/canon-simulate/adversary-p1/` (probes/, eval1/, eval2/, group.py, bound.py, msg-1.txt, red.log, suite.log, clippy.log)
needs-coordinator: yes. Two story rules need your call: whether a decision step counts as a way out of a stuck state or a cycle, and how findings are grouped (the story is silent on grouping).

**What I changed:** one new file, `crates/canon/tests/adversary_simulate.rs` (192 lines added), committed as 9c92fd1 by the bot. No implementation file, unit test or fixture is touched.

**Red run of that file alone, before the suite:** `test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`

**Suite:** `cargo test --workspace --locked --no-fail-fast` exits 101. Summed over its 141 summary lines (my sum): 1001 passed, 3 failed, 3 ignored. The only failing target is `error: test failed, to rerun pass -p b10x-canon --test adversary_simulate`, and its three failures are my three cases.

**Lint:** `cargo clippy --locked -p b10x-canon --test adversary_simulate -- -D warnings` exits 0 with no warning. `rustfmt --check` on the file exits 0.

## Findings

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon/src/check/space.rs:2360 | acceptance | warning | NEEDS-CHANGE | introduced | Five upstream artifacts invalidate a claim that reads a result. That gives one evidence dimension 2^64 values, more than a `u32` holds (`unwalkable`). The walk refuses with `reachable-state-bound`, yet it would reach only 3 states; four upstream artifacts walk fine. The story says a protocol "the check refuses whose reachable part fits" can be simulated. sdc.plan already declares 5 artifacts. Case: `adversary_simulate.rs:63` |
| crates/canon/src/simulate/mod.rs:584 | acceptance | warning | NEEDS-CHANGE | introduced | The walk never lets a `max_age` record expire. An action that becomes admissible only after an observation expires is reported `dead-action`, and its outcome `unreached-outcome`. `eval::evaluate_with` at an instant two hours later (`--at`) gives `renew` status `admissible`. The story's Scope does not exclude expiry. sdc.release and limitations/monolith declare `max_age` (`health_observation`). Case: `adversary_simulate.rs:115` |
| crates/canon/src/simulate/mod.rs:472 | acceptance | note | NEEDS-CHANGE | introduced | A state counts as waiting when any supplied step changes it, even when that step does not get it unstuck. Take `mutual` plus one supplied kind nothing reads: `{}` is counted as 1 waiting. The stuck state is then reported at `{note}` with path `protocol q supplies evidence note`, not at `{}` with path `empty`. Case: `adversary_simulate.rs:183` |
| crates/canon/src/simulate/mod.rs:469 | judgement | warning | NEEDS-CHANGE | introduced | `stuck-state` and `cycle` ignore decision steps. 624 SDC cycle findings are in protocols with an outcome that needs only a decision: spec 521, epic 78, sdc 11, story 7, release 7. No such cycle state has its decision taken, so `decision explicitly_abandoned/dropped/withheld taken` leads straight to a legitimate outcome. Probe `probe.decision` reports `stuck-state {draft}`, yet `decision accept taken` makes it terminable. This is the story's Outcome as written, and it contradicts claim 2 ("no exit"). The story is what needs to change. |
| crates/canon/src/simulate/mod.rs:568 | judgement | warning | CONFIRMED | introduced | Findings are not grouped. One root cause gives one finding per reachable combination of unrelated records: 514 findings from one cause on sdc.plan. sdc.release repeats each cycle once per capability value (undecided, denied, granted). The story only says "each one names its state" and is silent on grouping, so this is your decision. |
| crates/canon/src/simulate/mod.rs:22 | judgement | note | CONFIRMED | introduced | A reported `cycle` is never a loop of steps. Every step only adds, so a story cycle is a dead end where every step of every admissible action adds nothing new. This is the story's own definition and the code does what it says, but the word misleads an SDC reader. |

## The SDC cycle findings

- **One cause on three protocols:** all 1113 cycles on plan, spec and epic contain a `roadmap_approval` with both `approved` and another result. `roadmap.read` has no precondition, so it can always add a second record with another result. Mixed results make `roadmap.approved` `unknown`, which blocks every other action, and from then on every step of `roadmap.read` returns the same state.
- **Why there are so many:** each finding is that one cause times the set of downstream records produced before it. On plan the path lengths count 1, 2, 9, 36, 84, 126, 126, 84, 36, 9, 1: 514 findings, about 2^9, over 163 distinct sets of claims that are not true.
- **Followed by hand, plan finding 1:** state {roadmap_approval approved, roadmap_approval another result}, path `roadmap.read produces … result approved, then roadmap.read produces … roadmap`. I wrote that state as a case (e1 `approved`, e2 `rejected`, roadmap r1) and ran `canon evaluate`. `roadmap.read` is admissible. spec.gate, plan.validate, plan.cover, plan.scope, plan.status, plan.decompose and amend.upstream are blocked, and `passed` is blocked. Every `roadmap.read` step re-adds a class already present. Under the story's model the finding is sound.
- **A real case gets out:** move the roadmap to r2 and add one approval at r2. Then `roadmap.approved` is true and `spec.gate` admissible, because revision binding sets the old records aside. The walk never revises an artifact, a limit the story states. So none of these findings is false within the story's limits, but they are one artefact of that limit, not 514 defects.
- **The other protocols:** direction, intake, story, release and monolith follow the same pattern, a check recorded both a result and another result. In five protocols every cycle state also has a way out through a decision step (the decision-step row above).
- **What is implemented as written:** the cycle rule ("every step of every admissible action returns the state") and the order by path length, then rendering. Grouping, decision steps as exits and expiry are for you to decide.

## Attacked and not broken

- **Bound edge, 2 probes:** exactly 65536 reachable states exits 0; 65537 is refused with `reachable-state-bound` and no partial report.
- **Case inputs, 1 probe:** 3×2 input values give 6 initial states, with correct stuck and terminable counts.
- **Determinism, 3 checks:** repeated runs give identical bytes, and sdc.plan with its actions declared in reverse order gives the same bytes (md5 628198c3…).
- **Cost, 3 runs (release build):** spec 0.53 s, plan 0.21 s, monolith 1.29 s.
- **Hand evaluation, 3 cases through `canon evaluate`:** one sdc.plan cycle state, its way out by revision, and the expiry case.
- **Fixture edit f29f4e7, 5 protocols:** each change corrects a wrong expectation, and all seven expectations are still asserted. But every happy-path fixture now avoids `result:` matches, so no fixture shows a clean protocol that reads a result. Under this walk such a protocol cannot be clean.
- **Claim 4:** in the check code only visibility changed (`produces`, `supplied`, `Walked`), and `space.rs` only gained an appended impl block. No check output path changed.
- **The `state-space-bound/protocol.yaml` suffix rule:** it also requires the refusal reason `state-space-bound`, and `canon generate` really refuses both fixtures with it. Nothing is hidden.
- **`cli.yaml` against clap:** input `path` and outcomes `unreadable`/`rejected`/`nothing-found` match exit statuses 2/1/0. The `# read from:` citations point to story lines, and those lines exist.
- **Steps the evaluator would refuse:** the unbound dimension always uses a declared artifact. I did not probe `case_outcome` / `duplicate-child-case`.

```findings
[{"file": "crates/canon/src/check/space.rs", "line": 2360, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a dimension over u32 values is refused as reachable-state-bound although the walk reaches three states, contrary to the story's rule that a protocol whose reachable part fits is simulated"},
 {"file": "crates/canon/src/simulate/mod.rs", "line": 584, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "records never expire in the walk, so an action admissible after a max_age record lapses is reported dead and its outcome unreached although the evaluator finds it admissible"},
 {"file": "crates/canon/src/simulate/mod.rs", "line": 472, "category": "acceptance", "severity": "note", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "a state counts as waiting when any supplied step changes it, so a supplied kind nothing reads hides the empty stuck state and reports it at a later state"},
 {"file": "crates/canon/src/simulate/mod.rs", "line": 469, "category": "judgement", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "stuck-state and cycle ignore decision steps, so 624 SDC cycles are reported in states a single decision step makes terminable"},
 {"file": "crates/canon/src/simulate/mod.rs", "line": 568, "category": "judgement", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "findings are not grouped, so one root cause on sdc.plan yields 514 cycle findings that differ only in unrelated records; the story is silent on grouping"},
 {"file": "crates/canon/src/simulate/mod.rs", "line": 22, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "a reported cycle is a dead end whose admissible actions add nothing, never a loop of steps, as the story defines it, and the name misleads"}]
```
