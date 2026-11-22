---
format: aep.planning-md/3
id: review-result:review-wave-11
kind: review-result
status: active
title: 'Review of wave 11 at d21b7cba0: needs change, 4 red cases, 8 findings'
relations:
- reviews: story:wall-clock-ratio-tests-under-load
- reviews: story:scenario-generation-assertions
- reviews: story:check-descended-two-result-history
- reviews: story:review-notes-wave-9
- reviews: story:floor-guards-added-actions
- reviews: story:slow-lane-under-bound
- reviews: story:simulate-child-about-own-artifact
revision: 1
---
Review of the integration branch `wave/2026-11-22-w11` at `d21b7cba0` (base `35d2f642f`, Canon 0.11.1), one `aep:adversary` on Opus over the seven stories of the wave and the ESS pin, 2026-11-22. The reviewer's cases are on local branch `review/w11-wave` at `52d87246a`.

unit: wave 2026-11-22-w11, integration branch at `d21b7cba0`
verdict: NEEDS-CHANGE
cases: executed 240→244, red 4
origin: introduced 8, pre-existing 0, undecided 0
needs-coordinator: the ESS 0.97.4 checksum; whether F3 is in the floor story's scope (decided: yes)

## Findings and what became of each

| id | severity | unit | finding | disposition |
|---|---|---|---|---|
| F1 | blocker | pin | `ESS_SHA256` in `check.yml` was the 0.97.3 checksum, so CI could not install 0.97.4 | Fixed by the coordinator: `43dfbcb37`; `ess_gate.rs` now holds the checksum and fails on a mismatch (red with the old value, green with the new) |
| F2 | blocker | 5 | a one-member `all` around the floor's precondition passes the floor | Fixed: `ddac3733e`, merged `d6e1cf25e` |
| F3 | warning | 5 | an added action with no precondition and no capability passes | Fixed in scope (decided by the coordinator: an added action at least as permissive as the guarded one counts as a repeat): `ddac3733e` |
| F4 | warning | 5 | an action that only records an observation is refused | Fixed: `ddac3733e` |
| F5 | warning | 5 | `canon floor` help does not name the added-action rule | Fixed with a help sentence and the regenerated page: `ddac3733e` |
| F6 | warning | 3 | the new history modes count repeated values: 41472 states at 0.11.1, 69120 at the wave head | Fixed: `c42edd62e`, merged `5c7d8e85a` (53760 states, 0 findings); the reviewer's case runs 22.5 s alone and is in the slow lane |
| F7 | warning | 1 | the count tests pass a mutant that restores the per-evaluation `kept_out` copy and `in_progress` scan | Not fixed. Unit 1 caught the copy half with a counting allocator (`c26369461`); the allocator is an `unsafe impl GlobalAlloc` and failed `unsafe_confined`, which keeps `unsafe` in `eval/worker.rs` (H12). The correction is reverted (`bc1c94f04`). Draft story filed |
| F8 | note | 4 | the E4 probe records `kept.len()+1`, not enumerations alive | Fixed: `a9de4b3fc`, merged `afe6b6bc4`, with a negative control (2 against 1) |

Tried and not broken (from the report): 173 `protocol/1` fixtures give identical `check` and `simulate` bytes at base and head; the validate index gives the same errors for a duplicate claim, a cycle through a claim declared twice and an undeclared claim; the classifier guard refuses an empty answer and `planning-only\ncode`; no assertion of the old timing file was lost.

```findings
[
{"file":".github/workflows/check.yml","line":87,"category":"contract-drift","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"F1: ESS_SHA256 was still the 0.97.3 archive's checksum while ESS_VERSION was 0.97.4, so the Install ess step's grep -Fx fails on every code pull request"},
{"file":"crates/canon/src/floor/mod.rs","line":126,"category":"acceptance","severity":"blocker","verdict":"CONFIRMED","origin":"introduced","message":"F2: an added action whose precondition is the floor's wrapped in a one-member all publishes without finding.publish and canon floor prints floor holds"},
{"file":"crates/canon/src/floor/mod.rs","line":126,"category":"acceptance","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"F3: an added action with no precondition and no capability, strictly more permissive than the guarded action, passes canon floor"},
{"file":"crates/canon/src/floor/mod.rs","line":130,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"F4: a guarded action that produces nothing makes every added action at its precondition a RELAXATION, so an action that only records an observation is refused"},
{"file":"crates/canon-cli/src/lib.rs","line":271,"category":"contract-drift","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"F5: the floor help still says the floor holds when each change as canon diff classifies it is a tightening, expansion or no change, which the added-action rule no longer obeys"},
{"file":"crates/canon/src/check/space.rs","line":356,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"F6: LOW_FIRST and HIGH_FIRST count history values that give the NAMED value's records, so a protocol 0.11.1 checks at 41472 states is refused at 69120"},
{"file":"crates/canon/src/eval/claims.rs","line":290,"category":"mutant","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"F7: a mutant restoring the per-evaluation linear scan and context copy of the old cubic evaluator passes both count tests with identical counts"},
{"file":"crates/canon/src/check/mod.rs","line":835,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"F8: ENUMERATIONS_HELD records kept.len()+1 rather than enumerations alive, so the E4 test only checks that the kept list stays empty"}
]
```
