---
format: aep.planning-md/3
id: review-result:adversary-child-outcome-subject-pass-2
kind: review-result
status: active
title: 'Adversary pass 2 on story:child-outcome-subject: nothing the unit must fix, 2 notes on reference wording'
relations:
- reviews: story:child-outcome-subject
revision: 1
---
Second and last adversary pass on the unit of story:child-outcome-subject (head 5db11b33; the adversary's cases at 0e20d43b). Recorded as the adversary returned it.

unit: story:child-outcome-subject pass 2, at 5db11b33 plus my test commit 0e20d43b
verdict: nothing found that the unit must fix; 2 notes on doc wording only
cases: executed 1161→1164, red 0
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 directory, `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w6/child-outcome-subject/adversary-p2/`
needs-coordinator: none

Your step 1 and step 2 hold. On 15 protocols, the outcomes `canon simulate` reaches and the bypasses `canon check` reports matched my own enumeration of real cases, judged by `canon evaluate`. My 3 mutants are each red in the unit's tests and in mine. Both byte claims hold.

1. `git diff --stat 5db11b33..HEAD`: only `crates/canon/tests/adversary_child_outcome_subject.rs | 100 +`, a test file.
   - **Temporary edits in the worktree**, all under my lease, each restored from `HEAD` afterwards. `git status` was clean apart from my test file before the commit.
     - The 3 mutants.
     - One build of the wave branch without this unit: I set the 14 source files the unit changed to their `a1a937ec` text, built, saved the binary as `canon-wave`, restored the files, and rebuilt. The rebuilt binary is byte-identical to the one I built before.

2. **Cases added**, all green on 5db11b33:
   - `simulate_records_a_child_about_an_unnamed_artifact_without_subject`: three artifacts. `gamma` is no form's subject and is not the stand-in artifact of the dimension without a subject; its children must still count for a form without `subject`.
   - `simulate_keeps_a_named_child_out_of_the_dimension_without_subject`: only an opener about `alpha`, and a form names `alpha`. No real case gives "form without subject `true`, form about `alpha` `unknown`".
   - `check_decides_a_bypass_per_dimension`: three protocols.
     - A free opener about `alpha` and a gated one about `beta`, outcome needing both forms: 0 findings.
     - A free opener read only by a form without subject: 0 findings.
     - Two openers about `alpha`, one free: `authority-bypass`.
   - All 7 pass 1 cases now pass.

3. **Suite** with ess 0.77.0 first on PATH:
   - `cargo test --workspace --locked --no-fail-fast`: exit 0, 156 summaries, 1164 passed, 0 failed, 3 ignored.
   - `conform`: `28 passed, 0 failed`, `requirements: 29 implemented, 29 covered`. Clippy on my file is clean.
   - Mutants (each run against both test files, the lib's `check::` and `simulate::` tests (73), and conform):
     - **ma**, `produced_by` gives the dimension without a subject only to its own stand-in artifact: red. Mine: `left: "simulated: protocol `gamma` revision 1: 60 reachable states, 0 terminable (outcome `no.alpha` 0)…"`. The unit's `check_binds_a_childs_record_to_the_artifact_it_was_opened_about` also fails.
     - **mb**, `produced_by` gives an opener both the subject dimension and the one without: red. Mine: `left: "…`alpha-only` revision 1: 144 reachable states, 23 terminable (outcome `none.alpha` 3…"`. The unit's `simulate_over_the_fixture_records_each_child_about_its_opening` also fails (464 states instead of 400).
     - **mc**, `bypass` decided per kind again: red. Mine: `left: "authority-bypass: outcome `done` … about `alpha` result `passed` earlier, … about `beta` result `passed` earlier}…"`. Two of the unit's cases also fail.
     - For all three, the lib's `check::` and `simulate::` tests (73) and conform (28) stay green. The lib's random cross-check compares sliced with whole checking, not with real cases.

4. **Findings**

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| ess/domains/cli.yaml:288 | contract-drift | note | CONFIRMED | introduced | The doc says a child records about its `case_open` subject "when a children form names that artifact", and `simulate/mod.rs:61` says "when a form names". The code (`space.rs` `produced_by`, `named`) also counts an evidence match that names the artifact. Protocol `probe/matched.yaml` (a form without subject, plus `evidence: {kind: case_outcome, subject: alpha}`): the walk prints "records evidence `case_outcome` about `alpha`" where the doc predicts "without a subject". The behaviour is true against real cases (`done` legitimate in 2 real states, walk 3 terminable). |
| crates/canon-cli/src/lib.rs:148 | contract-drift | note | CONFIRMED | introduced | The `canon simulate` help text (also `website/docs/reference/cli.md:148`) still says only that an `open` record "keeps some count above none". It says nothing on which dimension a child's outcome arrives in, or on the narrowed D5. Only `cli.yaml` and the module doc carry those. Nothing in it is false; it is incomplete. |

Covers 5db11b33. Fix to name, not applied: write "when something reads that artifact by name (a form, an evidence match)" at both places, and add the binding sentence to the simulate help.

5. **Attacked and not broken**
   - **Real-case enumeration** (`real.py`): one child per opening, its `open` record written at once, up to 2 children per opener, outcomes taken from the forms plus `other`, capabilities granted or denied. 15 protocols:
     - your fixture, my 2 pass 1 protocols, your `pair` and `relay`;
     - 10 of mine: `same`, `mixed`, `alpha-only`, `gamma`, `cross`, `anynot`, `both`, `nosubject`, `unbound-free`, `matched`.
     - Outcome reachable or not: 15 of 15 agree with `simulate` (counts above 0 versus 0; `contra`, `none.alpha` and `beta.done` unreached in both).
     - Bypass: 8 of 8 agree with `check`. Reported in `same`, `anynot`, `nosubject`; not reported in `governed`, `both`, `unbound-free`, `fixture`, `cross`.
     - Every protocol with real dead ends gets at least one stuck-state or cycle finding. I checked 7 printed dead-end states by hand (`gamma` 4, `anynot` 1, `mixed` 1, `expiry` 1): each is a real dead end.
   - **Form with `subject` beside one without** (`mixed`, `gamma`, `alpha-only`): no walk state shows a combination of values no real case gives, in either direction.
   - **Opener whose children add no record** (the D3 case): the pinned cycle, `unreached-outcome` and `no-outcome-reachable` are all true; 0 waiting states.
   - **Expired `open` record in a subject dimension** (`expiry.yaml`): 4 cycles checked, each a real dead end.
   - **Slices:** `check_slices_a_children_form_to_its_subject` is green at 373248 states with 0 findings, so no slice is refused any more.
   - **Claim 2 (bytes):** 128 protocol documents with no form naming `subject` (17 of them open children), plus my `nosubject`: `check` and `simulate` stdout, stderr and exit code identical, 256 of 256, between `canon-wave` and head.
     - `canon-wave` refuses the unit's fixture: ``unknown field `subject` ``.
   - **Claim 3 (`evaluate` unchanged):** steps 1 and 2 touch only a doc comment under `eval/`. 27 scenarios pass under both `canon-wave` and head.
   - **The merge (d323e7b1):** 29 requirements; both count tests say 29. All four other units' adversary files pass: `adversary_decision_grounds` 6, `adversary_argument_validation` 20, `adversary_simulate_child_outcome` 5, `adversary_simulate_child_outcome_cli` 1.

**Pass 1 findings:**
- `simulate/mod.rs:679` blocker: answered.
- `check/mod.rs:1029` bypass: answered.
- `space.rs:2242` slices: answered.
- `child_outcome_subject.rs:325` mutants: answered (now pinned by 5 cases).
- `check/mod.rs:305` note: stays, as you decided (D3). The pinned output for it is true.

6. **Outside the worktree:** `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w6/child-outcome-subject/adversary-p2/`. It holds:
   - binaries: `canon-head`, `canon-wave`, `canon-sim-p2`;
   - scripts: `real.py`, `mk.py`, `runall.sh`, `bytes.sh`, `buildwave.sh`, `mutate.py`, `mutate.sh`;
   - inputs and work: `probe/`, `work/`, `bytes/`, `msg-1.txt`;
   - logs: `runall.log`, `bytes.log`, `buildwave.log`, `build1.log`, `mutations.log`, `suite.log`, `conform.log`, `clippy.log`.

   Lease released. `discard-cache` freed 7190 MiB; the first attempt was refused by a `tail` process, and the retry succeeded once it had exited.

```findings
[{"file": "ess/domains/cli.yaml", "line": 288, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "cli.yaml and the simulate module doc bind a child's record to its case_open subject only when a children form names the artifact, but produced_by also binds it when only an evidence match names it, so the walk prints about alpha where the doc predicts without a subject."},
 {"file": "crates/canon-cli/src/lib.rs", "line": 148, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "the canon simulate help text and generated cli.md say nothing of the dimension a child's outcome arrives in or of D5 narrowed to the actions producing that dimension, which only cli.yaml comments and the module doc state."}]
```
