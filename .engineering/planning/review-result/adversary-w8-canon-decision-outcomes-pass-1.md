---
format: aep.planning-md/3
id: review-result:adversary-w8-canon-decision-outcomes-pass-1
kind: review-result
status: active
title: Wave 2026-10-04-w8 adversary, canon story:decision-outcomes, pass 1
relations:
- reviews: story:decision-outcomes
revision: 1
---
unit: canon/decision-outcomes, committed d39d426 plus the uncommitted phase-2 working tree in ~/.local/state/worktree/trees/b10x/canon/canon-w8-decision-outcomes
verdict: CONFIRMED
cases: executed 380→386, red 2
origin: introduced 6 / pre-existing 1 / undecided 0
wrote-outside-worktree: 6 paths (part 6)
needs-coordinator: I briefly put a mutant library into the shared build dir and have repaired it. Any `cargo test -p b10x-canon` another agent ran between 08:43:33 and 08:46 linked mutant code (part 6).

The decision logic held up against every attack; I could not make it misbehave. Both red cases are about the generated docs page. One more case is green now but is the only test that catches a mutant the unit's own tests miss.

**1. `git --no-pager diff --stat`**
The tracked diff is the implementor's uncommitted phase 2 (37 files, 825+/178-), unchanged by me. My changes are two untracked test files:
```
?? crates/canon/tests/adversary_decisions_docs.rs
?? crates/canon/tests/adversary_decisions_eval.rs
```
No non-test path was edited. I did run `touch` on `crates/canon/src/eval/case.rs` to repair the build dir. Its content is unchanged: sha256 `3af1688b…6e58` before and after. Only the mtime moved.

**2. Cases added (each run alone first)**

| file | test | asserts | now |
|---|---|---|---|
| adversary_decisions_docs.rs | `the_documents_page_lists_the_canon_decisions_1_entry_and_its_keys` | documents.md has a `canon-decisions/1` section, its 4 keys and a `Principal` identifier row | **red** |
| adversary_decisions_docs.rs | `the_revision_identifier_is_not_described_as_an_artifacts_only` | the `Revision` row does not describe only artifacts | **red** |
| adversary_decisions_eval.rs | `a_case_revision_that_is_not_an_identifier_is_refused_in_its_place` | a case `revision: 'c 2'` is refused as `invalid-identifier`, after the protocol id and before the artifacts | green; red on the mutant |
| adversary_decisions_eval.rs | 3 more: several/duplicate/shared decisions; no revision or superseded revision with a termination; `read_ir` round trip and `decision` standing alone | | green |

Red output, captured before the suite ran (scratch `adv1-docs-red.log`):
```
panicked at crates/canon/tests/adversary_decisions_docs.rs:31:5:
documents.md has no `canon-decisions/1` section, though `canon evaluate --decisions` reads one and its model type is in crates/canon/src/model/explicit.rs
panicked at crates/canon/tests/adversary_decisions_docs.rs:58:5:
  left: "| `Revision` | Identifies one revision of an artifact. |"
test result: FAILED. 0 passed; 2 failed
```
Mutant check: I removed `identifier("case revision", …)` in a copy of the tree (case.rs:33) and ran `cargo test -p b10x-canon --no-fail-fast` there. Only my case failed on it (`adversary_decisions_eval.rs:68:41`); the other failures were the 2 docs cases plus `ess_gate.rs:433`, which failed only because the copy has no `.github`. canon-cli was not run against the mutant, and grep finds no case-revision case in canon-cli tests or conformance.

**3. Suite, run after the cases existed**
`cargo test --workspace --locked --no-fail-fast` gave exit 101 (summed `test result` lines: passed 384, failed 2). The only failures are the 2 docs cases. `-- --list` shows the new tests in this tree, and the log shows `Compiling b10x-canon (…/canon-w8-decision-outcomes/crates/canon)`.

**4. Findings (tree as above)**

| # | file:line | finding | verdict / origin | what reaches it |
|---|---|---|---|---|
| F1 | crates/canon-docs/src/pages.rs:248 | `EVALUATION_DOCUMENTS` leaves out `ExplicitDecision`. documents.md says it lists every key of what `canon evaluate` reads, but has no `canon-decisions/1` section and no `Principal` row. The keys appear only in prose in evaluation.md. Fix: add a 4th Document with root `ExplicitDecision` and format `DECISIONS_FORMAT`, then regenerate. | CONFIRMED / introduced | the published reference page |
| F2 | crates/canon/src/model/ids.rs:68 | `Revision` is documented as "one revision of an artifact", but `Case.revision` and `case_revision` now use it. | CONFIRMED / introduced | documents.md identifier table |
| F3 | crates/canon/src/eval/case.rs:33 | No test of the unit's covered the case-revision identifier check: removing it left all 380 of the unit's tests green. My case now catches it. | CONFIRMED / introduced | `canon evaluate --case` with any `revision:` |
| F4 | crates/canon/src/model/ids.rs:119 | The doc comment says the principal is "recorded as given", but it is only identifier-checked (decisions.rs:81) and then dropped. A decided outcome prints just `{"status":"legitimate"}`, with no decision, principal or case revision (design § 37 wants those recorded). Nothing can say who may decide. That matches the blocker's choice of option 3 over 1, and "Mandate resolves authority". | CONFIRMED / introduced | every `--decisions` run |
| F5 | crates/canon/src/eval/outcomes.rs:73 | Input checking is uneven. A decision for an undeclared outcome refuses the whole evaluation, even one at a superseded revision, which "applies to nothing". A wrong decision name for a declared outcome, or an exact duplicate, is accepted silently. Authority refuses duplicates. | CONFIRMED / introduced | a typo in `decision:` makes the outcome blocked with no refusal |
| F6 | crates/canon/src/model/requirement.rs:116 | An outcome cannot require a decision and a claim together: `decision` must stand alone, and inside `all`/`any`/`not` it is refused. This matches the story and design § 12. Only a protocol that needs both would hit it. | INFEASIBLE / introduced | no protocol shown to need it |
| F7 | crates/canon/src/eval/decisions.rs:59 | If recording a termination moves the case revision, then a decision taken before termination never matches the terminated snapshot, and option A refuses every decided termination. Nothing specifies who assigns case revisions. I don't know if any caller does this. | INFEASIBLE / introduced | not shown |
| F8 | website/product.json:58 | The `canon conform run` row says "one scenario covers CANON-CLAIM-001 and -002"; there are 7 scenarios. The row is identical at base 8fc260a. The unit rewrote the rows around it. | CONFIRMED / pre-existing | landing page |
| F9 | eval/case.rs, eval/read.rs, crates/canon-docs/src/, crates/canon/tests/adversary2_{ir_order,outcomes_polarity}.rs | Edited outside the story's scope. The adversary2 edits follow the new types; one assertion moves `unsupported-input`→`malformed-input` because behaviour changed as designed. | CONFIRMED / introduced | scope bookkeeping |

**5. Attacked and could not break**
- Several, duplicate, conflicting and shared-name decisions: each applies only to the outcome it names.
- A case with no revision: no decision applies.
- Termination at a superseded revision, or with no revision: `illegitimate-termination`.
- A decision for an outcome that requires a predicate: no effect.
- `read_ir` round trip of `{"decision":…}`. Refused with `malformed-input`: decision beside `any`, inside `all`/`not`, non-string, or in a claim.
- Parse refuses `decision` in an obligation, an action, inside `any`, beside `is`, or as a list.
- The JSON Schema `oneOf` matches the parser.
- Docs, product.json and status rows match behaviour, apart from F1, F2 and F8.
- Mutants already caught by the unit's tests: outcome/revision/None checks in `applies`, the undeclared-outcome loop, the principal identifier check, the `is_sequence` guard, decision identifier validation.

**6. Paths written outside the worktree**
- ~/.cache/ga-wave-2026-10-04-w8/canon-decision-outcomes/scratch/adv1-docs-red.log
- …/scratch/adv1-eval.log
- …/scratch/adv1-suite.log
- …/scratch/adv1-canon-p.log
- …/scratch/mutant-case-revision.log
- …/scratch/mutant-case-revision/ (the mutant copy; deleted)
- Build output in ~/.cache/b10x-target/canon-w8-decision-outcomes.

The incident: cargo names `-p` artifacts by paths relative to the workspace, so the mutant copy overwrote this tree's `-p b10x-canon` library (`libb10x_canon-224685fb…rlib`, 08:43:33). Cargo then treated it as fresh for the real tree, and a later `-p` run of mine linked mutant code. `--workspace` artifacts use different hashes and were not affected, so the gate's suite evidence stands. I repaired it by touching case.rs and rebuilding: `Compiling b10x-canon (…/canon-w8-decision-outcomes/…)`, 297 passed, 2 failed (only F1/F2). Next time a mutant copy needs its own target dir. My session lease is released.

```findings
- file: crates/canon-docs/src/pages.rs
  line: 248
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "documents.md claims to list every key of every document canon evaluate reads, but EVALUATION_DOCUMENTS omits ExplicitDecision, so canon-decisions/1, its four keys and the Principal identifier appear on no key-listing page"
- file: crates/canon/src/model/ids.rs
  line: 68
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "Revision is documented as the revision of an artifact while Case.revision and ExplicitDecision.case_revision now use it for the case's own revision"
- file: crates/canon/src/eval/case.rs
  line: 33
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "removing the case revision identifier check left all 380 of the unit's tests green; adversary_decisions_eval now pins the refusal and its order"
- file: crates/canon/src/model/ids.rs
  line: 119
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the principal is said to be recorded as given but is only identifier-checked and discarded; a decided outcome's entry names no decision, principal or case revision"
- file: crates/canon/src/eval/outcomes.rs
  line: 73
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "a decision for an undeclared outcome refuses the evaluation even at a superseded revision, while a misspelled decision name or a duplicate entry for a declared outcome is accepted silently"
- file: crates/canon/src/model/requirement.rs
  line: 116
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "an outcome cannot require both a decision and a claim; as specified, and no protocol was shown to need it"
- file: crates/canon/src/eval/decisions.rs
  line: 59
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: "if recording a termination advances the case revision, a decision taken before termination never matches the terminated snapshot and every decided termination is refused; nothing specifies who assigns case revisions"
- file: website/product.json
  line: 58
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: "the canon conform run row still says one scenario covers CANON-CLAIM-001 and -002 while seven scenarios exist"
- file: crates/canon/src/eval/read.rs
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "eval/case.rs, eval/read.rs, crates/canon-docs/src and two adversary2 test files were edited outside the story's typed scope"
```
