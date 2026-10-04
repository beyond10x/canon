---
format: aep.planning-md/3
id: review-result:adversary-w10-canon-explanation-pass-2
kind: review-result
status: active
title: Wave 2026-10-04-w10 adversary, canon story:explanation, pass 2
relations:
- reviews: story:explanation
revision: 1
---
```
unit: canon/explanation, worktree canon-w10-explanation at be5d15b plus one untracked test file
verdict: NEEDS-CHANGE
cases: executed 438→443, red 2
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 paths (scratch/adversary-p2/gate/, scratch/adversary-p2/mutants.log)
needs-coordinator: yes. The doc fix sits in crates/canon/src/model/decision.rs, which is outside this unit's scope. Also: should computed_from identify the protocol and the evidence by content, not only by id?
```

The unit needs changes: 2 new cases fail, and a third covers a gap that the existing tests miss.

**1. Diff.** `git --no-pager diff --stat` shows nothing, because the only change is an untracked file: `?? crates/canon/tests/adversary2_explain.rs`. It is a test file, and nothing else in the worktree changed.

**2. Cases** in `~/.local/state/worktree/trees/b10x/canon/canon-w10-explanation/crates/canon/tests/adversary2_explain.rs` (`--list` shows all 5):

| Case | Asserts | Now |
|---|---|---|
| `a_tagged_authority_decision_is_recorded_as_the_decision_that_applied` | `decision: !granted` and `decision: !denied` are recorded in `computed_from` as the decision that applied | **red** |
| `the_documents_page_does_not_call_the_explanation_unwritten` | the `explanation` row in `documents.md` does not say "not written yet" | **red** |
| `subject_reach_is_traced_once_in_a_fixed_order_whatever_the_evidence_order` | records matched both with and without a subject are listed once; absent entries ordered `l`, `l/a`, `l/b`; every claim's excluded entries equal its `excluded_evidence`; all 6 evidence orders give the same bytes | green; mutants M2, M3, M5, M7 and M8 turn it red |
| `every_claim_a_reason_names_is_listed_with_its_decided_value` | every `{"claim"}` pointer has an entry in `claims` with the value the decision gives it | green; mutant M6 turns it red |
| `a_true_claim_only_another_claim_tests_is_listed` | a `true` claim that only another claim tests still gets an entry in `claims` | green; mutant M1 turns it red |

Red output when each case was first run alone:
```
panicked at crates/canon/tests/adversary2_explain.rs:80:9:
assertion `left == right` failed: the authority decision that applied, `admissible`, is recorded as it applied
  left: Array [Object {"capability": String("cap"), "decision": String("")}]
 right: Array [Object {"capability": String("cap"), "decision": String("granted")}]
```
```
.../website/docs/reference/documents.md: the `explanation` row says it is not written: | `explanation` | any JSON value | when present | The structured explanation; not written yet (story:explanation). |
```
Run against M1 (line 370 changed to `let _ = ();`), the case fails: `left: Object {"top": ...}`, so `base` is missing from `claims`.

**3. Gate**, run in the unit's build dir after all cases existed:

| Command | Exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy ... -D warnings` | 0 |
| `cargo test --workspace --locked` | 101; summed totals: passed=441 failed=2 |
| `ess specify validate ... && ess verify conform synthesize` | 0: `canon v1 — 2 file(s), valid` |
| `canon-docs generate --check` | 0: `canon-docs: 14 generated files are current` |
| `canon-cli conform run` | 0: `conform: 9 passed, 0 failed, 0 unreadable` |

My test binary's own summary line: `test result: FAILED. 3 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out`. The before-count of 438 comes from this same run with my 5 cases subtracted.

**4. Findings** (all at be5d15b):
- **Authority recorded wrongly** (`explain/mod.rs:263`). The evaluator reads the authority text with `authority::read`. `explain::authority` reads the same text again into a `String`, so the two can disagree. In tagged form, a grant and a denial are both recorded as `""`, and `granted` written plainly gives different bytes from the tagged form. Any `--authority` document reaches it, because the CLI accepts the tagged YAML form. The documented form does not use it, and I found nothing that writes it. Suggested fix: record the `Authority` the evaluator already parsed.
- **Stale doc row.** `model/decision.rs:40` still says "not written yet", and the generated `documents.md:73` repeats it. The `canon-docs` check stays green because the page matches its source.
- **Test gap, no behaviour bug.** Before my M1 case, no test covered `claims` following claim references to a `true` claim that only another claim tests (`explain/mod.rs:370`).
- **Judgement, no case written:**
  - `computed_from` identifies the protocol by id and revision only (line 207), and the evidence by id only (line 214). Two evidence sets with the same ids but different subject revisions get identical `computed_from` and different decisions. The coordinator chose this shape, so it is your call whether that meets "which evidence set applied" in design § 37.
  - The comment in `conformance/scenarios/explanation.yaml` still says "every record of its kind" and shows absent entries without a subject.

**5. Attacked and could not break:**
- Byte order of evidence ids matches code-point order, consistent with `excluded_evidence`.
- Permuting the case's artifacts changes nothing.
- `--decisions` in tagged or plain form is recorded the same.
- Merge keys are refused by both readers.
- Mutants M2–M8 were each caught by the existing suite.
- The hand-written status rows match the output.

**6. Written outside the worktree:**
- `~/.cache/ga-wave-2026-10-04-w10/canon-explanation/scratch/adversary-p2/gate/` (gate logs 1–6)
- `~/.cache/ga-wave-2026-10-04-w10/canon-explanation/scratch/adversary-p2/mutants.log`
- I deleted the scratch copy and its target dir (665M).

```findings
- file: crates/canon/src/explain/mod.rs
  line: 263
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "explain re-parses --authority into String fields, so `decision: !granted` and `!denied`, which the evaluator applies as granted and denied, are both recorded in computed_from as an empty decision"
- file: crates/canon/src/model/decision.rs
  line: 40
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "the generated documents page row for `explanation` still says it is not written yet, while every decision now carries one"
- file: crates/canon/src/explain/mod.rs
  line: 370
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "dropping the claim-reference walk in claims() passed the whole existing suite; the new case a_true_claim_only_another_claim_tests_is_listed catches it"
- file: crates/canon/src/explain/mod.rs
  line: 214
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "computed_from identifies the protocol and the evidence set by id only, so two evidence sets with equal ids and different content that decide differently get identical computed_from"
- file: conformance/scenarios/explanation.yaml
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the scenario comment still describes listing every record of a kind and absent entries without a subject, which the subject-bound matching after the merge no longer does"
```
