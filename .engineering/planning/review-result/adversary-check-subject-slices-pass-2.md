---
format: aep.planning-md/3
id: review-result:adversary-check-subject-slices-pass-2
kind: review-result
status: active
title: 'Adversary pass 2 on story:check-subject-slices (wave 2026-11-20-w2): green, 1 note, introduced'
relations:
- reviews: story:check-subject-slices
revision: 1
---
Adversary pass 2 on story:check-subject-slices, wave 2026-11-20-w2, `aep:adversary` on Opus, on the
correction `17bd8ef..903d41d`. The report below is as the agent returned it.

unit: story:check-subject-slices
verdict: green
cases: executed 10→15, red 1
origin: introduced 1, pre-existing 0, undecided 0
wrote-outside-worktree: $HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w2/check-subject-slices/adversary-p2/ (sdc.sh, logs, msg-2.txt)
needs-coordinator: no

I could not break the correction. Both pass 1 findings are answered. The one red case is the help test, which stays red until the coordinator applies `check-help.patch`, as you said. The only new finding is a note.

## Findings

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon/src/check/space.rs:1238 | invariant | note | CONFIRMED | introduced | In a protocol that declares no artifact, each evidence dimension has 1 value but 2 producer sets. `probes()` still adds a state with that dimension at value 1, which is outside the whole product. It holds no record and evaluates exactly like the least state, so no report changes and nothing panics. It breaks the doc's own "its values below the number of sets": a later change to `evidence()` could hit `records()` on it. |

## Pass 1 findings
- **Unreadable `max_age` with no subject:** fixed. `an_unreadable_max_age_is_refused_when_no_subject_reads_a_state` now passes.
- **Help text:** the patch text is true of the code: slices, findings and witnesses as the whole space gives, a summary counting the whole space, and a refusal naming the subject and the slice's dimensions. One omission: when the whole product is too large to count, the refusal names the whole product, not a subject. The `generate` help sentence is also true.

## Attacked, not broken
- **"The first refused state is a probe": 23 evaluator refusal sites read, none refuted.** Every check the evaluator makes on a state the space builds depends on one record, one capability, one decision or one input value:
  - Record checks read the kind, subject, revisions, upstreams, producer, child case and instant. They never read the result.
  - Record ids, child cases and producers are unique by construction.
  - Instants are always readable.
  - A second class, a later upstream vector, history values from 2 up and later last-record values all come later in index order than a probe that refuses the same way.
  - CURRENT_FIRST and the order dimension only move valid instants.
  - So `undeclared-evidence-kind` and the duplicate refusals can't arise from check's own inputs, and the invalidation refusal does come first at the least state.
- **Refusal order: 5 new probes against the whole product (forced by the `zz` action from pass 1), all byte-identical.** `PROBED` reads a kind, an ordered kind with history and `case_outcome` only through an obligation.
  - three offenders together;
  - a capability before an input value, though the evaluator checks the case first;
  - a binding refusal at a record beside an earlier-stage input refusal;
  - an unreadable `max_age` on an ordered kind;
  - the unchanged protocol.
  - Order follows dimension index, never declaration order, in both paths.
- **Probe count and cost.** The count is linear in the sum of dimension sizes; the 196608-state `sliced` fixture gets 9 probes by my count. Release timings:
  - `sliced`: 151 ms;
  - SDC `sdc.plan`: 11.2 s (12 s in pass 1); `sdc.spec`: 7.7 s (8 s).
  - Probes change neither `Report::states` nor the summary line.
  - The SDC files changed revision since pass 1 (another session), so the report diffs are not head's.
- **Probes on a whole product above the bound.** Ranks fall back to all tied at order 0, which is a valid whole state. No panic.
- **Byte identity, all ok.** The 9 fixtures in `canon_check_reports_investigation_defects`, the 4 generated-protocol oracle seeds, `every_fixture_protocol_checks_alike_sliced_and_whole`, and the 8 pass-1 probes.

## Suite on 903d41d plus dfa6c15
`cargo test --workspace --locked --no-fail-fast`, ess 0.77.0 on PATH: 948 passed, 1 failed, 3 ignored. That is your 943 plus my 5. The plain run without `--no-fail-fast` stops at the failing binary and shows 780.
- `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 48.30s` (adversary_check_slices.rs)
- `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.34s` (the help test, red by design)

Commit `dfa6c15` (bot) appends the 5 passing probes to `crates/canon/tests/adversary_check_slices.rs`.

```findings
[{"file": "crates/canon/src/check/space.rs", "line": 1238, "category": "invariant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "in a protocol that declares no artifact each evidence dimension has radix 1 but two producer sets, so probes() adds a state with the dimension at value 1, outside the whole product; it holds no record and evaluates as the least state, so no report changes, but it contradicts the doc 'its values below the number of sets' and would reach records() if evidence() ever read a record-less kind"}]
```
