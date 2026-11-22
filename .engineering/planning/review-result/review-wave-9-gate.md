---
format: aep.planning-md/3
id: review-result:review-wave-9-gate
kind: review-result
status: active
title: 'Review of wave 9 at f7dcbf11a, area gate: needs change, 2 red cases, 8 findings'
relations:
- reviews: story:fast-test-step
- reviews: story:planning-only-checks
- reviews: story:local-gate-equals-ci
revision: 1
---
Review of the integration branch `wave/2026-11-21` at `f7dcbf11a`, area **gate** (the gate, CI files, test layout, `scripts/ci-watch`, the classifier), one of four `aep:adversary` agents on Opus run at once on 2026-11-22 (about 25 to 30 min each). The reviewer's report is in the coordinator's scratch (`w10/review/gate/`); this record is written at the close-out from the coordinator's facts of that report, not verbatim. Its cases are on branch `review/gate` (commit `c5c3cb494`, not pushed; cherry-picked by correction unit 1).

unit: wave 2026-11-20-w9, integration branch at `f7dcbf11a`, area gate
verdict: NEEDS-CHANGE
cases: executed 18→20, red 2
origin: introduced 8, pre-existing 0, undecided 0
wrote-outside-worktree: `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w10/review/gate/` (scratch only)
needs-coordinator: yes (G1 decision)

## Cases added

- `adversary_gate_ci_watch::a_pull_request_whose_task_check_job_never_reported_is_not_reported_green` (canon-cli): `scripts/ci-watch` exits 0 with `all 1 check(s) passed` when `gh pr checks` reports only the job of `pages.yml`. Red on `f7dcbf11a`, green after `b166206f1`.
- `adversary_gate_lanes::the_docs_site_lane_runs_every_test_under_the_runner_that_kills_at_60_s` (b10x-canon): `task docs-site-check` ran `cargo test -p canon-docs`, outside cargo-nextest. Red on `f7dcbf11a`, green after `a27aeab95`.

## Findings and what became of each

Severity `should-fix` of the reviewer is written `warning` in the findings block.

| id | severity | origin | file:line | finding | disposition |
|---|---|---|---|---|---|
| G1 | blocker | introduced | `.github/workflows/check.yml:97` | The slow lane runs inside the required job and holds tests killed at 60 s, so the job is red. | Decided D123 (Timo via `host-operator`, 2026-11-22 10:35 CET: "A: slow lane may fail, story"): `continue-on-error: true` (`4e2bd72d9`); draft `story:slow-lane-under-bound` lists the killed tests and its acceptance removes the flag. |
| G2 | should-fix | introduced | `scripts/ci-watch:42` | `scripts/ci-watch` exited 0 when the job `canon / task check` never reported. | Fixed, `b166206f1`: `--require`, `CI_WATCH_REQUIRE`, default `canon / task check`. |
| G3 | should-fix | introduced | `Taskfile.yml:110` | `task docs-site-check` ran `cargo test -p canon-docs` outside the 60 s kill. | Fixed, `a27aeab95` (also `pages.yml`, which got a nextest install step). |
| G4 | should-fix | introduced | `Taskfile.yml:75` | The local test step ran over 120 s under normal host load (151.1 s in gate 1; 158.5 and 150.8 s in the unit's runs). | Not changed. The bound is wall time with `num-cpus` threads on a shared host. Gate 2 measured 55.4 s on the step's own line at load 67 (inferred: the earlier figures held queue and build time inside the step). A CI run on a runner is the proof; not run yet. Backlog, draft `story:review-notes-wave-9`. |
| G5 | note | introduced | `Taskfile.yml:71` | The ten-slowest list left out the `ess-gate` tests. | Fixed, `428ce4dc5`: the sum counts them; none is among the ten slowest. |
| G6 | note | introduced | `crates/canon/tests/main.rs:117` | The layout test does not flag an undeclared `.rs` file in a tests subdirectory; `canon-docs` has no layout test. INFEASIBLE today. | Not changed: backlog, draft `story:review-notes-wave-9`. |
| G7 | note | introduced | `.github/workflows/check.yml:40` | A classifier answer outside the three words runs no step. INFEASIBLE today. | Not changed: backlog, draft `story:review-notes-wave-9`. |
| G8 | note | introduced | `scripts/ci-watch:12` | `ci-watch` for a branch without a pull request waited 3600 s. | Fixed, `b166206f1`: exits 2 at once. |

The reviewer named no line for G3 to G8 except as given in the coordinator's brief to unit 1 (`scripts/ci-watch:42-49`); the other lines are read by the close-out from `f7dcbf11a`.

```findings
[
{"file":".github/workflows/check.yml","line":97,"category":"boundary","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"G1: the slow lane inside the required job holds tests killed at 60 s, so the job is red; decided D123 (Timo: A, slow lane may fail, story): continue-on-error in 4e2bd72d9, story:slow-lane-under-bound removes it."},
{"file":"scripts/ci-watch","line":42,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"G2: scripts/ci-watch exited 0 when the job canon / task check never reported; fixed in b166206f1 (--require, CI_WATCH_REQUIRE, default canon / task check)."},
{"file":"Taskfile.yml","line":110,"category":"boundary","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"G3: task docs-site-check ran cargo test -p canon-docs outside the runner that kills at 60 s; fixed in a27aeab95, pages.yml too."},
{"file":"Taskfile.yml","line":75,"category":"performance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"G4: the local test step ran over 120 s under normal host load (151.1 s; 158.5 and 150.8 s in the unit's runs); not changed: gate 2 measured 55.4 s on the step's own line at load 67, a CI run is the proof; backlog draft story:review-notes-wave-9."},
{"file":"Taskfile.yml","line":71,"category":"acceptance","severity":"note","verdict":"NEEDS-CHANGE","origin":"introduced","message":"G5: the ten-slowest list omitted the ess-gate tests; fixed in 428ce4dc5, the sum counts them."},
{"file":"crates/canon/tests/main.rs","line":117,"category":"boundary","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"G6: the layout test does not flag an undeclared .rs in a tests subdirectory and canon-docs has no layout test; infeasible today; backlog draft story:review-notes-wave-9."},
{"file":".github/workflows/check.yml","line":40,"category":"boundary","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"G7: a classifier answer outside the three words runs no step; infeasible today; backlog draft story:review-notes-wave-9."},
{"file":"scripts/ci-watch","line":12,"category":"contract-drift","severity":"note","verdict":"NEEDS-CHANGE","origin":"introduced","message":"G8: ci-watch for a branch without a pull request waited 3600 s; fixed in b166206f1, it exits 2 at once."}
]
```
