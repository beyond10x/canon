---
format: aep.planning-md/3
id: review-result:review-wave-9-simulate
kind: review-result
status: active
title: 'Review of wave 9 at f7dcbf11a, area simulate: needs change, 3 red cases, 5 findings'
relations:
- reviews: story:child-outcome-recovery
- reviews: story:artifact-revised-by
revision: 1
---
Review of the integration branch `wave/2026-11-21` at `f7dcbf11a`, area **simulate** (`canon simulate`, child-outcome recovery, the external revise step), one of four `aep:adversary` agents on Opus run at once on 2026-11-22 (about 25 to 30 min each). The reviewer's report is in the coordinator's scratch (`w10/review/simulate/`); this record is written at the close-out from the coordinator's facts of that report, not verbatim. Its cases are on branch `review/simulate` (commit `144493d9f`, not pushed).

unit: wave 2026-11-20-w9, integration branch at `f7dcbf11a`, area simulate
verdict: NEEDS-CHANGE
cases: executed not recorded; red 3 added by the reviewer, plus 1 that was already failing (F3)
origin: introduced 5, pre-existing 0, undecided 0
wrote-outside-worktree: `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w10/review/simulate/` (scratch only)
needs-coordinator: yes (D1 fix rule, F1 decision)

## Findings and what became of each

Severity `should-fix` of the reviewer is written `warning` in the findings block.

| id | severity | origin | file:line | finding | disposition |
|---|---|---|---|---|---|
| D1 | should-fix | introduced | `crates/canon/src/simulate/recovery.rs:63` | `recovery::split` split the `case_outcome` dimension per `case_open` subject with nothing revisable: festival 86 states and event 178 against the committed 78 and 122, `report.txt` bytes differed (against story:child-outcome-recovery acceptance 6), and the state bound refused protocols 0.10.0 walked (probe: N openers about N subjects, no revise: N=3 212→8000 states, N=4 644→`reachable-state-bound`). | Fixed by correction unit 1, `32fc74fd5`: split only when a revision exists (an action has `revises` or an artifact has `revised_by`). This is wider than the rule the coordinator gave the unit (split only when a `case_open` subject is revisable), which broke `revise-child-other-subject` (D126). |
| F1 | should-fix | introduced | `crates/canon/src/simulate/recovery.rs:264` | A child opened about the space's own artifact never ends against the revision it was opened at (`split.redirected.get(action)?`); two dead ends go unreported depending only on artifact names; the story's outcome 3 and outcome 5 contradict each other here. | Decided D125 (taken on default): documented as a limit in the `canon simulate` help (unit 1 `abb281809`); the reviewer's case for it was deleted, not merged; backlog draft `story:simulate-child-about-own-artifact`. |
| F2 | should-fix | introduced | `crates/canon-cli/src/lib.rs` | `canon simulate --help` and `website/docs/reference/cli.md` omitted the external revise step, `revised_by` and the summary phrase. | Fixed, `abb281809`. |
| F3 | should-fix | introduced | `crates/canon-cli/tests/canon_simulate.rs` | `canon_simulate::simulate_sets_aside_child_outcomes_of_the_revised_subject` was killed at 60.1 s in the first whole-wave gate (two walks of a 3.5 s fixture). | Fixed, `473278cd4`: one walk; the second in its own test, about 4 s. |
| N1 | note | introduced | `crates/canon/src/simulate/tests.rs` | No test pinned that the external revise step is tried after the supplied steps. | Fixed, `1a83e5af3`; mutant proof: swapping the loops fails it. |

The reviewer named no line for F2, F3 and N1; the findings block gives line 1 for those files.

```findings
[
{"file":"crates/canon/src/simulate/recovery.rs","line":63,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"D1: the case_outcome dimension was split per case_open subject with nothing revisable, so festival (86 vs 78 states) and event (178 vs 122) changed their report bytes and the state bound refused protocols 0.10.0 walked; fixed in 32fc74fd5 (split only when a revision exists, D126)."},
{"file":"crates/canon/src/simulate/recovery.rs","line":264,"category":"judgement","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"F1: a child opened about the space's own artifact never ends against the revision it was opened at, so two dead ends go unreported depending only on artifact names; decided D125: documented limit in canon simulate help (abb281809), backlog draft story:simulate-child-about-own-artifact."},
{"file":"crates/canon-cli/src/lib.rs","line":1,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"F2: canon simulate --help and cli.md omitted the external revise step, revised_by and the summary phrase; fixed in abb281809."},
{"file":"crates/canon-cli/tests/canon_simulate.rs","line":1,"category":"performance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"F3: simulate_sets_aside_child_outcomes_of_the_revised_subject was killed at 60.1 s in the first whole-wave gate (two walks of a 3.5 s fixture); fixed in 473278cd4 (one walk, the second in its own test, about 4 s)."},
{"file":"crates/canon/src/simulate/tests.rs","line":1,"category":"mutant","severity":"note","verdict":"NEEDS-CHANGE","origin":"introduced","message":"N1: no test pinned that the external revise step is tried after the supplied steps; fixed in 1a83e5af3, and swapping the two loops fails it."}
]
```
