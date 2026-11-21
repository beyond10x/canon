---
format: aep.planning-md/3
id: review-result:adversary-check-subject-slices-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:check-subject-slices (wave 2026-11-20-w2): red, 2 findings, both introduced'
relations:
- reviews: story:check-subject-slices
revision: 1
---
Adversary pass 1 on story:check-subject-slices, wave 2026-11-20-w2, `aep:adversary` on Opus. The
report below is as the agent returned it; its findings block is the corrected one it sent after the
store refused the severities `medium` and `low`. The coordinator then changed the word `confirmed` to
`CONFIRMED` in the two `verdict` fields of that block, the letter case the store requires, and
nothing else.

unit: story:check-subject-slices
verdict: red
cases: executed 0→10, red 2
origin: introduced 2, pre-existing 0, undecided 0
wrote-outside-worktree: $HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w2/check-subject-slices/adversary-p1/ (sdc.sh, logs, msg-1.txt)
needs-coordinator: no

The 429 stopped me before I had written anything, so this pass ran from the start. I found no case where the sliced report differs from the whole-product report. The two red cases are elsewhere: the help text still states the old bound, and one evaluator refusal no longer fires.

## Findings

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon-cli/src/lib.rs:120 | docs-contract | medium | confirmed | introduced | `canon check --help` still ends "A state space of more than 65536 states is refused." The same binary checks the 196608-state `sliced` fixture. The story lists this doc comment and `website/docs/reference/cli.md` in scope, and neither changed. The `crates/canon-cli/src/check.rs:1-2` module doc also still says the library "enumerates and evaluates every state". |
| crates/canon/src/check/mod.rs:713 | proves-less | low | confirmed | introduced | A protocol with no outcome, action or property has no subject, so the plan is empty and no state is evaluated. A caller-built IR whose kind has an unreadable `max_age` is then reported as checked instead of refused as `invalid-max-age`. The base evaluated every state, so it refused. With one outcome added, head still refuses. Only the library API reaches this: `canon check` compiles and validates first, so a bad `max_age` never gets that far from the command line. |

Red tests, with each runner's summary line:
- `crates/canon/tests/adversary_check_slices.rs:317` `an_unreadable_max_age_is_refused_when_no_subject_reads_a_state`
  `test result: FAILED. 8 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 53.04s`
- `crates/canon-cli/tests/adversary_check_slices_cli.rs:27` `check_help_does_not_refuse_every_space_above_the_bound`
  `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.40s`

Commit `17bd8ef` (bot) has both files and nothing else. I added no fixture, so `adversary_scenario_generation.rs` is unaffected.

## Attacked and not broken

- **Sliced report against whole report, 8 hand-written protocols, all byte-identical.** `pub(crate)` strategies can't be called from an integration test. So I reach the whole product by adding one action `zz`: its slice is every dimension and it finds nothing. I checked by hand that without `zz` the slices hold fewer states than the whole product. The 8 protocols cover things the generator never writes, together with 22 properties:
  - claims three levels deep, with `not`, `is: false`, `is: unknown`, and an `any` whose branch reads another kind;
  - an invalidation rule built on through claim chains;
  - three inputs whose first-rendered value is not the first declared (`x-y`, `B`, `P`);
  - scoped and repeated capabilities;
  - `case_open` under authority with `children`;
  - shared and reordered `independent_on` profiles;
  - `since` + `first`, `before` + `first` and `after` protocols at 46800, 56160 and 37440 states, above the generator's 2048 cap.
- **Witness choice, by reading.** Every value 0 is the empty one (no record, undecided, not taken, CURRENT, NO_LATER, NO_LAST). Only the order dimension and inputs weigh nothing: the order renders only things that hold records, and an unread input is fixed to its first-rendered value. `erase` grouping, standing, last-record moves and record ids give the same groups and partners on a slice as on the whole product.
- **Global settings a slice could lose.** Slices keep the whole space's `after` flag, producers, `unlisted`, kinds and upstreams. The `late` instant and the order ranks only change positions relative to each other, which the evaluation does not see.
- **Things outside the readers.** Only `Predicate::Input` reads an input. `applicable_when` reaches only obligations, and nothing reads obligation status.
- **SDC protocols.** `canon check` at head on 13 protocols under `$HOME/beyond10x/sdc/protocol/` (8 protocols, 5 probes) printed the same lines as `docs/check-output.txt`. The one exception is `sdc.direction`, which is now revision 3 in the SDC tree. I did not build a base binary, because that needs a checkout of 557e75b in the worktree, which the git rules forbid. All 13 reports have 0 or 1 findings, so this comparison is weak.
- **`canon generate`.** The guard uses the old message. Within the bound it is unchanged, because the slice refusal can't fire when every slice is at most the whole product.
- **The 4 re-pinned test files.** Each still asserts what it asserted before: dimension naming order, the `<=` bound, upstream naming and the refusal path.
- **Not a defect.** A protocol whose whole product overflows `u128` while every slice fits is still refused with the old "has more than … states" message. The module docs say so, and the base refused it too.

The store refused the block above this note's place as unreadable YAML (a value holding `: `). The refusal went back to the author, who returned the block below; it replaces the first one and holds the same two findings.

```findings
[{"file": "crates/canon-cli/src/lib.rs", "line": 120, "category": "docs-contract", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "canon check --help still says a state space of more than 65536 states is refused, yet it checks the 196608-state sliced fixture; the story scoped this doc comment and website/docs/reference/cli.md and neither changed; check.rs:1-2 also still says every state is evaluated. Test crates/canon-cli/tests/adversary_check_slices_cli.rs:27"}, {"file": "crates/canon/src/check/mod.rs", "line": 713, "category": "proves-less", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "with no outcome, action or property there is no slice and no state is evaluated, so a caller-built IR with an unreadable max_age is reported checked instead of refused as invalid-max-age, as the base whole-product enumeration did; library API only. Test crates/canon/tests/adversary_check_slices.rs:317"}]
```
