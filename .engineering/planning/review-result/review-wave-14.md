---
format: aep.planning-md/3
id: review-result:review-wave-14
kind: review-result
status: active
title: 'Review of wave 14 at 51c12f86e: needs change, 4 red cases, 7 findings'
relations:
- reviews: story:ordering-guards-provable-by-check
- reviews: story:supplied-evidence-required-at-open
- reviews: story:children-form-counts-unopened-children
revision: 1
---
Review of the integration branch `wave/2026-11-23-w14` at `51c12f86e` (base `6dc461644`, Canon 0.14.0), one `aep:adversary` on Opus over the three stories of the wave, 2026-11-23. The reviewer's cases are commit `7aa9031a3` (branch `review/w14-wave`), merged into the wave branch as `813e031a0`.

unit: wave 2026-11-23-w14, integration branch at `51c12f86e`
verdict: NEEDS-CHANGE
cases: executed 293→297, red 4
origin: introduced 7, pre-existing 0, undecided 0
needs-coordinator: none

## Findings and what became of each

| id | severity | unit | finding | disposition |
|---|---|---|---|---|
| W14-1 | blocker | 3 | `Layout::ended` raised the current-revision count when a step replaced both the current and the earlier `open` record, so `with_children` evaluated three records for two children and a `count: 3` form was true while a reader was unopened | Fixed: correction A `9b24c9c37`, merged `b95c93813`. `ended` raises only on `{current: false, expired: true}`; `with_children` caps the current copies. Unit test asserts all 8 rows of replaced-record combinations |
| W14-2 | warning | 1 | `canon generate` ran the guard pass through `check_keeping` and refused (`ordered-child-outcomes`, `state-space-bound`) where the protocol without the guard generates | Fixed: correction B `521babdcb`, merged `1b8de091d`. Guards are lowered only when check does not keep its spaces |
| W14-3 | warning | 3 | the published schema and reference gave `count` a minimum of 0, which validate refuses | Fixed: `29375ddc4` (a list of at-least-one fields in `canon-docs`; ESS 0.97.4 cannot express the minimum on a type no view publishes), pages regenerated `352569c62` |
| W14-4 | note | 3 | `with_children` built the count's copies in every state: count 1000 took 29.7 s, 5000 over 60 s | Fixed: correction A refuses a count above 64 in `canon simulate` as `child-count-bound` |
| W14-5 | note | 1 | a removed `runs_only_when` is NO SEMANTIC CHANGE, so `canon floor` holds when a guard is dropped | Documented (coordinator's choice): floor help, `cli.yaml`, `protocol.yaml`, `protocols.md`; pinned by `a_floor_does_not_pin_a_guard` |
| W14-6 | note | 1 | only the literal `all: []` is refused as `vacuous-runs-only-when` | Documented (coordinator's choice); pinned by `only_the_literal_empty_all_is_refused_as_a_vacuous_guard` |
| W14-7 | note | 3 | `check` and `simulate` help said nothing about `count` | Fixed: correction B, help texts and `cli.md` |

One review case, `generate_gives_the_same_output_with_a_guard_whose_slice_is_above_the_bound`, could not pass: its protocol has 18 evidence kinds and is above the bound without the guard too (it failed on the base build). It was removed in `29375ddc4`; correction B's `generate_reads_no_guard` covers the promise with a protocol of 10 kinds.

Tried and not broken (from the report): 852 comparisons of validate, compile, check, simulate and check-set against `6dc461644` differ only on the wave's 7 new fixtures; the consumer's 36 protocols and a check-set over them, 109 comparisons, 0 differ; a 0.14.0 reader refuses each new key; `unsupplied-at-open` with two openers, a guard, `not`, chains and shuffled `--path` order; IR key order for `count`.

```findings
[
{"file":"crates/canon/src/simulate/mod.rs","line":1010,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"W14-1: a child ending that replaces both the current and the earlier open record raises the current-revision count again, so a count-3 form is true while a reader is unopened"},
{"file":"crates/canon/src/generate/mod.rs","line":209,"category":"contract-drift","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"W14-2: generate runs the guard pass and refuses where the protocol without the guard generates"},
{"file":"website/static/schemas/protocol-1.schema.json","line":853,"category":"contract-drift","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"W14-3: the schema and reference give count a minimum of 0, which validate refuses"},
{"file":"crates/canon/src/simulate/mod.rs","line":1026,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"W14-4: simulate builds the count's copies for every state; count 1000 takes 29.7 s"},
{"file":"crates/canon/src/diff/mod.rs","line":1113,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"W14-5: a removed runs_only_when is NO SEMANTIC CHANGE, so canon floor holds when a guard is dropped"},
{"file":"crates/canon/src/validate/mod.rs","line":1129,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"W14-6: only the literal all: [] is refused as vacuous"},
{"file":"crates/canon-cli/src/lib.rs","line":1,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"W14-7: check and simulate help do not describe count"}
]
```
