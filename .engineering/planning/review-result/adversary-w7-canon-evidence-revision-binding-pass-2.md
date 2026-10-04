---
format: aep.planning-md/3
id: review-result:adversary-w7-canon-evidence-revision-binding-pass-2
kind: review-result
status: active
title: Wave 2026-10-04-w7 adversary, canon story:evidence-revision-binding, pass 2
relations:
- reviews: story:evidence-revision-binding
revision: 1
---
unit: canon/evidence-revision-binding, working tree on c755b24 plus the uncommitted phase 2 and pass-1 fixes (worktree canon-w7-evidence-revision-binding)
verdict: NEEDS-CHANGE
cases: executed 264→269, red 1
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 scratch log, plus builds in the assigned build dir
needs-coordinator: whether a stale record that no claim reaches should appear in the decision (fix the docs, or change the spec)

**1. `git --no-pager diff --stat`**

Same 13 tracked files as handed over, 218+ / 46−. The only file I added is the untracked `crates/canon/tests/adversary2_binding_eval.rs`. I touched no implementation file.

**2. Cases added** (`~/.local/state/worktree/trees/b10x/canon/canon-w7-evidence-revision-binding/crates/canon/tests/adversary2_binding_eval.rs`)

| Case | Asserts | Now |
|---|---|---|
| `exclusion_listing_names_every_record_bound_to_another_revision` | the docs say a record bound to another revision "is listed as excluded"; this checks a stale record of kind `m`, which no claim reaches, appears in the decision | red |
| `probe_a_revision_written_as_an_integer_is_refused_on_either_side` | `revision: 2` and `subject_revision: 2` are each refused as `malformed-input`; `'2'` and `"2"` match; `R1`, `r01` and `sha256:r1` against `r1` are excluded | green |
| `probe_evidence_about_the_case_itself_is_refused_naming_it` | `subject: C-1` (the case id) is refused as `undeclared-artifact` with the exact message | green |
| `probe_a_declared_artifact_missing_from_the_case_is_refused_before_binding` | `missing-artifact` naming `b` comes before any binding outcome | green |
| `probe_a_supplied_input_is_refused_before_an_undeclared_subject` | with `--at` given, `unsupported-input` wins, as documented | green |

Red output from the first run of this file alone:
```
test exclusion_listing_names_every_record_bound_to_another_revision ... FAILED
panicked at crates/canon/tests/adversary2_binding_eval.rs:87:5:
evaluation.md / where-this-stands.md: "a record bound to another revision is listed as excluded"; `stale-m` is bound to r1 of `a` (current r2) and the decision names it nowhere:
{ "case": "C-1", "claims": { "base": { "excluded_evidence": [ { "evidence": "stale-k", "reason": "revision_mismatch" } ], "value": "unknown" } }, ... }
test result: FAILED. 4 passed; 1 failed
```

**3. Suite run** (after the cases existed)

`cargo test --workspace --locked --no-fail-fast` with the brief's build env: EXIT=101. Summed `test result:` lines: 268 passed, 1 failed (the case above). `--list` shows the 5 test names in this tree. `cargo run -q --locked -p canon-docs -- generate --check` printed "11 generated files are current", exit 0.

**4. Findings**

| # | file:line | Finding | Verdict | Origin | What reaches it |
|---|---|---|---|---|---|
| F1 | `crates/canon/src/eval/mod.rs:94` (rendered as `website/docs/reference/evaluation.md:103`) and `website/docs/status/where-this-stands.md:20` | Both say every record bound to another revision is listed as excluded. A record is only listed under claims that reach its kind, so a stale record of any other kind appears nowhere. Step 4 of the mod.rs docs already gives the precise rule. | NEEDS-CHANGE | introduced | Any protocol with an evidence kind that no claim's `true_when` uses, for example one that only an action's `may_produce` names. The validator accepts that. |
| F2 | `website/docs/index.md:20`, `website/docs/concepts/three-valued-truth.md:43` | Both pages were edited but are outside the story's typed scope. The four parallel units will likely edit the same "not built yet" lines, so expect merge conflicts. | CONFIRMED | introduced | the merge into wave/2026-10-04-w7 |
| F3 | `website/docs/concepts/evidence-and-revisions.md:10` | The page still says unconditionally "The claim goes back to `UNKNOWN`, not to `FALSE`", now under a "Shipped" note. Pass 1's `settled` case goes TRUE→FALSE, a behaviour the coordinator accepted. | CONFIRMED | introduced | claims that test `is: unknown` |
| F4 | `website/docs/concepts/evidence-and-revisions.md:8`, `website/docs/concepts/protocols.md:34` | Future tense ("is to be bound", "is meant to be bound") for something now shipped. | CONFIRMED | introduced | readers of the concept pages |
| F5 | `crates/canon-docs/src/pages.rs:1241` | The sentence "Both records are bound to `explanation` at its current revision" is a hard-coded literal, not derived from `EXAMPLE_CASE` or `evidence_record`. It is true today (both r1). | CONFIRMED | introduced | a future edit to the example's case or records |

For F1, the fix is to qualify both sentences ("listed under each claim that reaches its kind"). The alternative is to list every excluded record at decision level, which is a spec change and the coordinator's call.

**5. Attacked and could not break**

- **Schema and model:** `evidence-1.schema.json` and `documents.md` match the model (fields, required list, identifier types). Only the descriptions changed, and generate `--check` is clean.
- **pages.rs edit:** it changes only `investigation-example.md`. The other generated diffs come from the model and mod.rs doc comments.
- **Status rows:** the CANON-EVIDENCE-001 row and the scenario count (2 files) are accurate.
- **Revision types:** integer revisions are refused on both sides, so there is no mismatch between text and number.
- **Evidence about the case:** refused, naming the subject.
- **Declared artifact missing from the case:** refused as `missing-artifact` before binding runs.
- **CLI exit and stderr:** already pinned by the phase-1 test `canon_evaluate_refuses_an_undeclared_subject_naming_it` (exit 1, one stderr line, names the subject), which is green.
- **Obligations:** the section gets the claim values after exclusion (mod.rs:197), but `obligations::section` is a stub returning `None` in this tree. Not testable here (INFEASIBLE) until story:obligations merges.

**6. Paths written outside the worktree**

- `~/.cache/ga-wave-2026-10-04-w7/canon-evidence-revision-binding/scratch/adv2-suite.log`
- `~/.cache/b10x-target/canon-w7-evidence-revision-binding` (the assigned build dir, built into)

I did not acquire a worktree session lease.

```findings
- file: crates/canon/src/eval/mod.rs
  line: 94
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "evaluation.md:103 and where-this-stands.md:20 say every record bound to another revision is listed as excluded, but a stale record of a kind no claim reaches appears nowhere in the decision (adversary2_binding_eval.rs:87 red)"
- file: website/docs/index.md
  line: 20
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "index.md and three-valued-truth.md are edited outside the story's typed scope, on lines the four parallel evaluator units are likely to edit too"
- file: website/docs/concepts/evidence-and-revisions.md
  line: 10
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the page now presents binding as shipped and still says unconditionally that the claim goes back to UNKNOWN, not FALSE, while a claim testing is: unknown goes TRUE to FALSE"
- file: website/docs/concepts/protocols.md
  line: 34
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "protocols.md:34 and evidence-and-revisions.md:8 still describe revision binding in the future tense after it shipped"
- file: crates/canon-docs/src/pages.rs
  line: 1241
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the generated example's claim that both records are bound to explanation at its current revision is a hard-coded literal, not derived from EXAMPLE_CASE or the records"
```
