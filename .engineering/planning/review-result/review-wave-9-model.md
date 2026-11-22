---
format: aep.planning-md/3
id: review-result:review-wave-9-model
kind: review-result
status: active
title: 'Review of wave 9 at f7dcbf11a, area model: confirmed, 1 red case, 5 findings'
relations:
- reviews: story:protocol-floor
- reviews: story:canon-mutate
- reviews: story:revision-ordering
- reviews: story:decision-condition
revision: 1
---
Review of the integration branch `wave/2026-11-21` at `f7dcbf11a`, area **model** (the model, IR, `canon diff`, the writer, `canon floor`, `canon mutate`), one of four `aep:adversary` agents on Opus run at once on 2026-11-22 (about 25 to 30 min each). The reviewer's report is in the coordinator's scratch (`w10/review/model/`); this record is written at the close-out from the coordinator's facts of that report, not verbatim. Its case is on branch `review/model` (commit `c5c912273`, not pushed and not merged).

unit: wave 2026-11-20-w9, integration branch at `f7dcbf11a`, area model
verdict: CONFIRMED
cases: executed 37→38, red 1
origin: introduced 5, pre-existing 0, undecided 0
wrote-outside-worktree: `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w10/review/model/` (scratch only)
needs-coordinator: yes (F1 decision)

## Case added

- `crates/canon-cli/tests/adversary_w10_model.rs`, `an_added_action_that_publishes_without_the_floors_capability_breaches_the_floor`, red on `f7dcbf11a`: it keeps `publish_finding` and adds `publish_finding_quietly` with the same precondition and no capability; `canon floor` prints `floor: holds`, `EXPANSION: 1` and exits 0. Not merged (D124).

## Also checked, not broken

- The IR of all 129 `protocol/1` files of the base tree is identical on both binaries.
- `canon-docs generate --check`: 18 files current.

## Findings and what became of each

Severity `should-fix` of the reviewer is written `warning` in the findings block.

| id | severity | origin | file:line | finding | disposition |
|---|---|---|---|---|---|
| F1 | should-fix | introduced | `crates/canon/src/floor/mod.rs:7` | `canon floor` passes every `EXPANSION`, and an added action is one, so a new action that publishes the floor's finding without the floor's capability is not caught. | Decided D124 (taken on default): documented limit in the `canon floor` help and `floor/mod.rs` (unit 1 `abb281809`); the red test is not merged; backlog draft `story:floor-guards-added-actions` (a floor rule for added actions is a design change). |
| F2 | note | introduced | `crates/canon/src/eval/decision_condition.rs:374` | `canon mutate` makes no mutant at a conditioned decision and its report does not say it skipped the site (documented, pinned by `mutate_makes_no_mutant_at_a_conditioned_decision`). | Not changed: backlog, draft `story:review-notes-wave-9`. |
| F3 | note | introduced | `crates/canon/src/diff/mod.rs:1339` | The implication arm `(Descended, Descended) => left == right` was run by no test. | Fixed, `1b60756cb`; mutant `=> false` is caught (`BREAKING` against `TIGHTENING`). |
| F4 | note | introduced | `crates/canon/src/diff/mod.rs:840` | Adding `when` to a floor's decision is `BREAKING` although it only narrows (decided by story:decision-condition acceptance 8). | Not changed: design note, backlog, draft `story:review-notes-wave-9`. |
| F5 | note | introduced | `crates/canon/src/ir/mod.rs` | Writer, IR and diff documentation drifted from the code. | Fixed, `8583b4f3b` (the IR doc now counts eighteen exceptions; also `model/write.rs`, `canon-ir.md`). |

The reviewer named no line for F2, F4 and F5; F2 and F4 are read by the close-out from `f7dcbf11a` (the pinning test, the `requirement changed` push); F5 is given line 1.

```findings
[
{"file":"crates/canon/src/floor/mod.rs","line":7,"category":"design","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"F1: canon floor passes every EXPANSION and an added action is one, so a new action that publishes the floor's finding without the floor's capability is not caught; decided D124: documented limit (abb281809), red test not merged, backlog draft story:floor-guards-added-actions."},
{"file":"crates/canon/src/eval/decision_condition.rs","line":374,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"F2: canon mutate makes no mutant at a conditioned decision and the report does not say it skipped the site (documented and pinned by mutate_makes_no_mutant_at_a_conditioned_decision); not changed, backlog draft story:review-notes-wave-9."},
{"file":"crates/canon/src/diff/mod.rs","line":1339,"category":"mutant","severity":"note","verdict":"NEEDS-CHANGE","origin":"introduced","message":"F3: the implication arm (Descended, Descended) => left == right was run by no test; fixed in 1b60756cb, the mutant => false is caught (BREAKING vs TIGHTENING)."},
{"file":"crates/canon/src/diff/mod.rs","line":840,"category":"design","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"F4: adding when to a floor's decision is BREAKING although it only narrows, as decision-condition acceptance 8 decided; not changed, design note in backlog draft story:review-notes-wave-9."},
{"file":"crates/canon/src/ir/mod.rs","line":1,"category":"contract-drift","severity":"note","verdict":"NEEDS-CHANGE","origin":"introduced","message":"F5: writer, IR and diff documentation drifted from the code; fixed in 8583b4f3b (the IR doc now counts eighteen exceptions)."}
]
```
