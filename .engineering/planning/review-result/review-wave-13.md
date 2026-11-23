---
format: aep.planning-md/3
id: review-result:review-wave-13
kind: review-result
status: active
title: 'Review of wave 13 at 05d970050: needs change, 2 red cases, 5 findings'
relations:
- reviews: story:check-across-protocol-files
- reviews: story:simulate-opens-child-with-its-record
- reviews: story:refused-approval-is-not-stuck
revision: 1
---
Review of the integration branch `wave/2026-11-23-w13` at `05d970050` (base `210f929d9`, Canon 0.13.0), one `aep:adversary` on Opus over the three stories of the wave, 2026-11-23. The reviewer's cases are on local branch `review/w13-wave` at `b7a49e546`.

unit: wave 2026-11-23-w13, integration branch at `05d970050`
verdict: NEEDS-CHANGE
cases: executed 150→152, red 2
origin: introduced 5, pre-existing 0, undecided 0
needs-coordinator: whether decision D3 option B stands (F3). The coordinator reported the false premise to host-operator; D3 stands.

## Findings and what became of each

| id | severity | unit | finding | disposition |
|---|---|---|---|---|
| F1 | warning | 2 | the opening step gives its `open` record producer profile 0; when every reader of `case_outcome` demands independence that is one of the demands, so the walk never holds a running child whose record meets no demand and prints a false `unreached-outcome` | Fixed: `1424e44bb`, merged `8d4855c3c`. A dimension gets the record at opening only when some reader demands no independence; only those dimensions are held to the pairing rule. `needs-you` stays 1431 states |
| F2 | note | 1 | `check-set` removes duplicate `--path` values by spelling, so `review.yaml` and `./review.yaml` are read twice and the summary double-counts | Fixed: `2f8dbdb4d`, merged `593285519`. Each path is resolved with `fs::canonicalize` first; the consumer's 43 openings are 32 |
| F3 | warning | 3 | options A and B change the simulate output of seven existing fixtures (three lose findings: 2 to 1, 4 to 2, 2 to 1) and no commit or test names them; the wave plan said no fixture holds a denial | Fixed: `334371b2e`, merged `8d4855c3c`. 13 fixture reports are pinned by bytes (the seven and six more that unit 2 changed); the commit body has the before and after counts. D3 stands (reported to host-operator) |
| F4 | warning | 1 | two files with one protocol id are pooled, so an old revision of a child hides an `undeclared-child-outcome` and turns exit 1 into exit 0 | Fixed: `2f8dbdb4d`. New finding `duplicate-protocol-id` (exit 1); no pooling |
| F5 | note | 1 | the `check-set` help names exit 0 and 1 only | Fixed: `2f8dbdb4d`; `check-set` and `mutate` help state exit 2 |

Tried and not broken (from the report): 799 runs of the wave binary against `210f929d9` (check, compile, conform run, generate, evaluate) gave 0 differences; unit 2's 23 simulate changes are all named in its commits; `needs-you` goes 24410 to 1431 states; a cycle across three files is found and the `--path` order changes nothing; order forms and evidence matches are read; a decision whose `when` is false keeps the finding; no denial text appears when the blocked action's precondition is false; `check` and `simulate` take the same time as the base.

The four `adversary2_ess_model` tests were not run by the reviewer (story:probe-built-once-for-ess-model-tests).

```findings
[
{"file":"crates/canon/src/simulate/mod.rs","line":1340,"category":"acceptance","severity":"warning","verdict":"INFEASIBLE","origin":"introduced","message":"F1: the opening step's open record takes the first independence demand as its producer, so a protocol whose case_outcome readers all demand independence gets a false unreached-outcome"},
{"file":"crates/canon-cli/src/check_set.rs","line":22,"category":"contract-drift","severity":"note","verdict":"INFEASIBLE","origin":"introduced","message":"F2: duplicate paths are removed by spelling, so review.yaml and ./review.yaml are read twice and the summary double-counts, against the help's read once"},
{"file":"crates/canon/src/simulate/mod.rs","line":1793,"category":"contract-drift","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"F3: options A and B change the simulate output of seven existing fixtures, three of which lose findings, while the unit commit and the D3 premise say no existing fixture holds a denial"},
{"file":"crates/canon/src/ir/composition.rs","line":424,"category":"judgement","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"F4: two files with one protocol id are pooled, so adding an old revision of a child hides an undeclared-child-outcome and turns exit 1 into exit 0"},
{"file":"crates/canon-cli/src/lib.rs","line":354,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"F5: the check-set help names exit 0 and 1 only, though an unreadable file exits 2 and the first failing file in sorted order decides between 1 and 2"}
]
```
