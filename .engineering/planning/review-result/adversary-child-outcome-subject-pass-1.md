---
format: aep.planning-md/3
id: review-result:adversary-child-outcome-subject-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:child-outcome-subject: red, 1 blocker, 3 warnings, 1 note'
relations:
- reviews: story:child-outcome-subject
revision: 1
---
First adversary pass on the unit of story:child-outcome-subject (head 6c53e232; the adversary's cases at d0468c46). Recorded as the adversary returned it.

unit: story:child-outcome-subject, impl/child-outcome-subject at 6c53e232 (my cases are commit d0468c46)
verdict: red
cases: executed 1068→1075, red 4
origin: introduced 4, pre-existing 0, undecided 0
wrote-outside-worktree: 1 directory, `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w6/child-outcome-subject/adversary-p1/`
needs-coordinator: yes. The `simulate` finding is in `simulate/mod.rs`. That file is outside this story's scope. story:simulate-child-outcome-step lists "binding a child outcome to the opening's subject" as out of its scope, so no story covers it.

1. `git diff --stat 6c53e232..HEAD`: `crates/canon/tests/adversary_child_outcome_subject.rs | 151 +` (test file only). **Charter deviation:** the brief said to mutate and then restore from git, so I applied the 4 mutations in the worktree itself, one after another, while holding the lease. The adversary procedure forbids that. Each file was restored from `HEAD`, and `git status` was clean apart from my test file before the commit.

2. **Cases** in `crates/canon/tests/adversary_child_outcome_subject.rs`. The first run of the file alone gave `test result: FAILED. 0 passed; 4 failed`.
   - `check_finds_no_bypass_through_a_child_opened_about_another_artifact`: **red**. `left: "authority-bypass: outcome `beta.done` is legitimate without authority in state {evidence `case_outcome` about `beta` result `passed` earlier}\n…1 finding\n"`
   - `check_slices_a_children_form_to_its_subject`: **red**. `state-space-bound: … outcome `alpha.done` reads 373248 states … `case_outcome` about `alpha` 8, … about `beta` 8, … about `gamma` 8, history … 9 ×3`
   - `simulate_does_not_reach_an_outcome_through_a_child_about_another_artifact`: **red**. `path: action `alpha.open` produces evidence `case_outcome` about `beta` …`, `1 terminable (outcome `beta.done` 1)`
   - `simulate_over_the_fixture_opens_ideas_only_about_the_intake`: **red**, on the unit's own fixture. `path: action `ideas.open` produces evidence `case_outcome` about `wave` result `open` …`
   - Three cases are **green** now and go red under mutants that the unit's tests and `conform` let through: `check_reads_each_forms_own_subject_in_one_predicate` (m2), `check_reads_the_subject_of_a_form_without_revision` (m3), `compile_keeps_the_subject_of_a_form_under_not` (m4).

3. **Suite** (`cargo test --workspace --locked --no-fail-fast`, exit 101)
   - The first run lacked ess 0.77.0 on PATH, so 9 targets failed: my file and 8 ESS targets.
   - I reran those 8 with PATH set (`export PATH=$HOME/.cache/ess/toolchains/0.77.0:$PATH; cargo test … --test ess_gate …`): exit 0, 2+4+7+11+30+15+19+13 = 101 passed.
   - Only my target stays red: `3 passed; 4 failed`.
   - About the 1075: the ESS tests run nested cargo, which muddles a naive sum of the log, so I did not take it from the log. It is the implementor's 1068 plus my 7, and my 7 are seen running.
   - `conform run --requirements`: `conform: 26 passed, 0 failed`, `requirements: 27 implemented, 27 covered`. Clippy on the file is clean, and `fmt --check` is clean.
   - **Mutants**, each run against the lib tests, `child_outcome_subject`, my file and `conform`:
     - m1, `revision: current` compared with the first artifact's revision when `subject` is set: red. The lib stays green; it fails `reasons_and_the_explanation…` and `a_subject_selects…`, and `failed: scenario CANON-COMPOSE-003 step an-open-idea-beside-the-wave-stories`.
     - m2, `reads_in` giving every form the first form's subject: **green** on lib (332), the unit file (8) and conform (26).
     - m3, `reads_in` keeping the subject only with `revision: current`: **green** on all three.
     - m4, `normalize` dropping the subject under `not`: **green** on all three.

4. **Findings**

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon/src/simulate/mod.rs:679 | acceptance | blocker | NEEDS-CHANGE | introduced | `canon simulate` on the unit's own fixture. `check::produces` ignores the `case_open` subject, so an action opening children about one artifact walks records about another. Result: a false path, and `beta.done` reported terminable when it is legitimate in no reachable state. |
| crates/canon/src/check/mod.rs:1029 | acceptance | warning | NEEDS-CHANGE | introduced | `canon check` on two `case_open` actions where only the capability-gated one is about the form's subject. It prints a false `authority-bypass`. |
| crates/canon/src/check/space.rs:2242 | property | warning | NEEDS-CHANGE | introduced | `evidence_reached` and `positive_reads` (:2296) read a `children` form as `(case_outcome, None)`. A slice then takes every subject's dimension, and `check` refuses a protocol whose slices hold 72 states each. Its message names dimensions the outcome does not read, which contradicts story:check-subject-slices rule 1. A `first` form naming 3 subjects checks fine (104000 states). |
| crates/canon/tests/child_outcome_subject.rs:325 | mutant | warning | CONFIRMED | introduced | m2, m3 and m4 survive the unit's suite. Every `check` case uses `revision: current` and puts one subject per predicate, and no case has `not` over a subject. The random cross-check generator (`check/tests.rs:1989`) never writes `subject`. |
| crates/canon/src/check/mod.rs:305 | judgement | note | INFEASIBLE | introduced | A form naming an artifact that no `case_open` is about: `check` prints 0 findings. `unproduced-evidence` is decided per kind, and the story declines a refusal, so this stays as it is unless that changes. |

Where the base comparison stands:
- The base binary `…/simulate-child-outcome-step/canon-base` answered `--version` with `canon 0.9.0`, then was deleted while my pass ran. My fixture comparison got exit 127 and is void.
- The origin is "introduced" because the base cannot express `subject`. Before this unit, `case_outcome` had one unbound dimension, so ignoring the `case_open` subject was correct.
- Fix to name, not applied:
  - Bind `produces` and simulate's `of_kinds` to dimensions that `read_as(case_outcome, Some(case_open.subject))`.
  - Make `evidence_reached` and `positive_reads` return `form.subject.as_ref()`.

5. **Attacked and not broken**
   - `check` against my own reasoning on 1 mixed protocol (5184 states): 2 findings, both correct; history dimension under `not` and subject is right.
   - Records, 6 cases. No `subject` and no `subject_revision` are both refused `malformed-input`. An undeclared subject is `undeclared-artifact`. A snapshot without `wave` is `missing-artifact`. One child about two subjects is `duplicate-child-case`, as the spec says. An `open` child about `intake` leaves `wave.passed` `true`.
   - `validate` holders: 4 of 4 named right (claim, obligation, action, outcome), including under `not` and `any`.
   - IR order with all 8 combinations of revision and subject: in-list, then revision, then subject. Reasons and explanation on an action and an outcome follow the same order. Key order in the source does not matter.
   - Existing scenarios and fixtures with `children` are unchanged in the diff and pass.
   - `diff`: changing `subject` is BREAKING, the same as `revision` (`diff/mod.rs:1277`).
   - Scenario step kinds are only `compile` and `evaluate`, so the implementor is right that a scenario cannot hold a validation refusal.
   - `missing_evidence` lists `may_produce` kinds only, not `children`.
   - canon-docs renders only `fixtures/investigation/protocol.yaml`, which has no `children`, so no generated page is wrong. `pages.rs:1348` would drop `subject` and `revision` if one appeared.

6. **Outside the worktree:** `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w6/child-outcome-subject/adversary-p1/`. It holds `build1.log`, `red-alone.log`, `compare.sh`, `compare.log`, `compare/`, `records.sh`, `records/`, `order-eval.sh`, `order/`, `probe/` (8 yaml), `mutate.py`, `mutate.sh`, `mutate2.sh`, `mutations.log`, `mutations2.log`, `suite.log`, `suite-ess.log`, `conform.log`, `clippy.log`, `count.sh`, `count2.py`, `msg-1.txt`. Lease released; `discard-cache` freed 7117 MiB.

```findings
[{"file": "crates/canon/src/simulate/mod.rs", "line": 679, "category": "acceptance", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "canon simulate lets an action that opens children about one artifact produce case_outcome records about another, so on the unit's fixture it walks impossible paths and reports an outcome terminable that no reachable state makes legitimate."},
 {"file": "crates/canon/src/check/mod.rs", "line": 1029, "category": "acceptance", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "check::produces ignores the case_open subject, so canon check reports a false authority-bypass when only a capability-gated action opens children about the form's subject."},
 {"file": "crates/canon/src/check/space.rs", "line": 2242, "category": "property", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "evidence_reached and positive_reads take a children form with subject as reading every case_outcome dimension, so canon check refuses as state-space-bound a protocol whose slices hold 72 states and names dimensions the outcome does not read."},
 {"file": "crates/canon/tests/child_outcome_subject.rs", "line": 325, "category": "mutant", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "three mutants survive the unit's tests and conform: reads_in using the first form's subject, reads_in keeping subject only with revision current, and normalize dropping subject under not."},
 {"file": "crates/canon/src/check/mod.rs", "line": 305, "category": "judgement", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "a form naming an artifact no case_open action is about gets no check finding because unproduced-evidence is decided per kind, and the story declines a refusal."}]
```
