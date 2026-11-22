---
format: aep.planning-md/3
id: review-result:review-wave-9-eval
kind: review-result
status: active
title: 'Review of wave 9 at f7dcbf11a, area eval: confirmed, 1 red case, 4 findings'
relations:
- reviews: story:revision-ordering
- reviews: story:decision-condition
revision: 1
---
Review of the integration branch `wave/2026-11-21` at `f7dcbf11a`, area **eval** (evaluation and order forms), one of four `aep:adversary` agents on Opus run at once on 2026-11-22 (about 25 to 30 min each). The reviewer's report is in the coordinator's scratch (`w10/review/eval/`); this record is written at the close-out from the coordinator's facts of that report, not verbatim. Its cases are on branch `review/eval` (commit `7d6c45bfb`, not pushed).

unit: wave 2026-11-20-w9, integration branch at `f7dcbf11a`, area eval
verdict: CONFIRMED
cases: executed 227→228, red 1
origin: introduced 4, pre-existing 0, undecided 0
wrote-outside-worktree: `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w10/review/eval/` (scratch only)
needs-coordinator: no

## Case added

- `check::tests::adversary_descended_unknown_beside_a_placed_first_of_its_kind_is_reached` (`7d6c45bfb`), red on `f7dcbf11a`. One record (kind k, result passed, current revision, an instant, no item; the case lists `descends_from: [r0]`) gives `red_then_green` unknown, `passes` true and `first_pass` true under `canon evaluate`, so the outcome requiring the three is legitimate; `canon check` reports it `unreachable-outcome` and `canon generate` refuses the protocol.

## Findings and what became of each

Severity `should-fix` of the reviewer is written `warning` in the findings block.

| id | severity | origin | file:line | finding | disposition |
|---|---|---|---|---|---|
| E1 | should-fix | introduced | `crates/canon/src/check/space.rs:1742` | The check space drops an item only together with every instant, so `descended` unknown beside a placed `first` over the same kind is never reached; `canon check` reported a false `unreachable-outcome` (`canon evaluate` reaches it two ways). | Fixed by correction unit 2: `88f838ab2` red test, `f903df4b0` fix (a history value gets a no-item mode and a last-earlier mode; reviewer's protocol 72→216 states, no finding; protocols without the mix unchanged; `revision-ordering.yaml` identical), `ddc337201` sibling soundness test (29.0 s, slow lane), merged in `2f3d14b14`. Left open: one unreached combination, see draft `story:check-descended-two-result-history`. |
| E2 | note | introduced | `crates/canon/src/check/mod.rs:263,279` | The module docs said an outcome's slice is its one decision dimension; the code adds what `when` reads. | Fixed (docs): `b3a97c66f`. |
| E3 | note | introduced | `crates/canon/src/check/mod.rs:857` | `unproduced-evidence` does not name a decision outcome whose `when` reads unproduced evidence (story:decision-condition D4 B kept the readers unchanged). | Not changed: backlog, draft `story:review-notes-wave-9`. |
| E4 | note | introduced | `crates/canon/src/check/mod.rs:683` | `canon check` keeps every slice's states and evaluations until it returns; peak memory is the sum of the slices. Not measured. | Not changed: backlog, draft `story:review-notes-wave-9`. |

The reviewer named no file:line for E3 and E4; the lines above are read by the close-out from `f7dcbf11a` (the `UnproducedEvidence` push and `check_keeping`).

```findings
[
{"file":"crates/canon/src/check/space.rs","line":1742,"category":"property","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"E1: the check space drops an item only together with every instant, so descended unknown beside a placed first over the same kind is never reached and canon check reports a false unreachable-outcome; fixed by correction unit 2 (f903df4b0, merged 2f3d14b14)."},
{"file":"crates/canon/src/check/mod.rs","line":263,"category":"contract-drift","severity":"note","verdict":"NEEDS-CHANGE","origin":"introduced","message":"E2: the module docs (lines 263 and 279) said an outcome's slice is its one decision dimension; the code adds what when reads; fixed in b3a97c66f."},
{"file":"crates/canon/src/check/mod.rs","line":857,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"E3: unproduced-evidence does not name a decision outcome whose when reads unproduced evidence (decision-condition D4 B kept readers unchanged); not changed, backlog draft story:review-notes-wave-9."},
{"file":"crates/canon/src/check/mod.rs","line":683,"category":"performance","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"E4: canon check keeps every slice's states and evaluations until it returns, so peak memory is the sum of the slices; not measured; not changed, backlog draft story:review-notes-wave-9."}
]
```
