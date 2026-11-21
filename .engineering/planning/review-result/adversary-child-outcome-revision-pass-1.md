---
format: aep.planning-md/3
id: review-result:adversary-child-outcome-revision-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:child-outcome-revision (wave 2026-11-20-w4): green, 4 notes, introduced'
relations:
- reviews: story:child-outcome-revision
revision: 1
---
Adversary pass 1 on story:child-outcome-revision, wave 2026-11-20-w4, `aep:adversary` on Opus, on `impl/child-outcome-revision` at `2f8bfe5`. The report below is as the agent returned it.

unit: story:child-outcome-revision, findings cover branch impl/child-outcome-revision at 2f8bfe5 (my test commit on top is 174183d)
verdict: green
cases: executed 999→1001, red 0
origin: introduced 4, pre-existing 0, undecided 0
wrote-outside-worktree: $HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w4/child-outcome-revision/adversary-p1/{build.log,case.log,suite.sh,suite.log,msg-1.txt,probe/}
needs-coordinator: no

I couldn't make anything fail. The four findings are all notes: three gaps in tests or documentation and one remark on the unit's patch.

**1. Diff stat** (`git diff --stat 2f8bfe5..HEAD`): only a test file. No path outside the tests was touched.
```
 .../tests/adversary_child_outcome_revision.rs      | 134 +++++++++++++++++++++
 1 file changed, 134 insertions(+)
```

**2. Cases added**, in `crates/canon/tests/adversary_child_outcome_revision.rs`. Both pass now and passed on their first run.
- `the_bound_form_compares_each_record_with_its_own_subject`: two artifacts, `spec` at `s2` and `plan` at `p1`, five record sets. A record about `spec` at `p1`, or about `plan` at `s2`, is not read. Each record is compared with its own subject's revision. Both claims have an empty `excluded_evidence` list. Before this file, every unit test, the CANON-COMPOSE-002 scenario and every `canon check` state either declared one artifact or gave all artifacts the same revision. So a version comparing a record with some other artifact's revision (or with any artifact's) would have passed every test.
- `revision_any_outside_a_claim_is_no_key`: `revision: any` in an action precondition and in an outcome requirement under `not` compiles to the same bytes as no key, and `canon diff` reports no change. `revision: current` gives different bytes.
- First run of the file alone, `cargo test -p b10x-canon --test adversary_child_outcome_revision`:
```
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```
- I did not run the cases against a mutated copy: that needs a second target directory, which the brief forbids. That they would catch the mutation is my reading of the code, not a measurement.

**3. Suite**, run after the cases existed: `cargo test --workspace` with ess 0.77.0 first on PATH. 140 summaries, 1001 passed, 0 failed, 3 ignored, exit 0, `slot: job 13013 waited 8s for a build slot, ran 556s`. The new file inside that run:
```
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**4. Findings** (tree 2f8bfe5)

| file:line | category | severity | verdict | origin | what reaches it |
|---|---|---|---|---|---|
| crates/canon/src/eval/composition.rs:181 | mutant | note | CONFIRMED | introduced | Any protocol with two artifacts at different revisions. Until 174183d no test pinned which artifact's revision a record is compared with. |
| crates/canon/src/check/mod.rs:222 | contract-drift | note | CONFIRMED | introduced | Readers of the `canon check` module doc (not rendered to the website). It still says a `children` form reads every record "at any revision", which is no longer true for `revision: current`. |
| crates/canon/src/check/tests.rs:1918 | property | note | CONFIRMED | introduced | Claim 4 (sliced and whole-product reports agree). The protocol generators at `:1918` and `:2082` only ever write forms without the key, so the committed sliced-vs-whole tests never see a `current` form. The claim rests on the implementor's uncommitted probe. I can't add the case: those helpers are crate-private and the file is not mine. |
| website/docs/reference/evaluation.md:252 | judgement | note | CONFIRMED | introduced | Applying item 2 of `needs-coordinator.patch`. It edits `eval/mod.rs`, which `canon-docs` renders into this page (`pages.rs:60`), but the patch carries no regenerated page, so `slot task docs-generate` has to run after it. |

On the three patch items:
- **Item 1** (re-export `ChildRevision`): right. It is needed only so outside users can name the type of the public field; nothing breaks without it.
- **Item 2:** right, with the regeneration caveat above.
- **Item 3:** right; `outcomes.rs` is not rendered to any page.

**5. Attacked and not broken**
- IR bytes: 68 fixtures under `fixtures/` compile to identical bytes with the old oracle binary `canon-6f44325` and the new binary. The other 28 don't compile on the oracle and were skipped; all 28 compile on the new binary. I used the oracle only for this comparison.
- `read_ir` and the exclusion stages: `read_ir` refuses `"any"` and other values. Normalisation runs on claims, obligations, preconditions and requirements before members are sorted and de-duplicated. In the conformance runner, the claims section is compared byte for byte, so the scenario does pin acceptance 6.
- Edge cases, 6 probes:
  - a missing artifact, or an empty `artifacts` section, is refused as `missing-artifact`, so the "no revision for this subject" branch can't be reached;
  - the same child at two revisions is refused as `duplicate-child-case`, as the story intends;
  - wrong artifact, right artifact, and one artifact moved while the other did not all give correct values, reasons and explanation.
- CANON-COMPOSE-002: removing the filter fails the step `only-an-outcome-about-an-earlier-revision`, and so does listing stale records as excluded. Comparing with the wrong artifact passes the scenario (the first finding).
- `canon check`, `generate` and `conform` on a probe protocol (two artifacts, `current` and keyless forms, a `current` form inside a deeper claim, a child-opening action under authority): 9792 states, 0 findings. The 7 generated scenarios pass, and the fixture's 3 pass. The `only_stale` outcome (a stale record with no current one) is reached.
- `canon diff`: `any` against no key and changes to `revision` are classified from normalised IR. `ordered-child-outcomes` and `inert-invalidation` match any `children` form, and their code is unchanged.
- `ess/domains/protocol.yaml`: I spot-checked 17 `predicate.rs` references and 4 story references. All point at the lines they name.

**6. Written outside the worktree:** the scratch paths in the header. `probe/` holds `two.yaml`, `fix.yaml`, `two.ir.json`, `run.sh`, `ircmp.sh`, `eval.sh`, `gen-two/`, `gen-fix/`, `e/` and `ir/`.

```findings
[{"file": "crates/canon/src/eval/composition.rs", "line": 181, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "No test before 174183d gave two artifacts different revisions, so a revision: current filter comparing a record with another or any artifact's revision passed the whole suite; the adversary case now pins it."},
 {"file": "crates/canon/src/check/mod.rs", "line": 222, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The check module doc still says a children form reads every case_outcome record at any revision, which is false for revision: current."},
 {"file": "crates/canon/src/check/tests.rs", "line": 1918, "category": "property", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The generated sliced-versus-whole check tests never write revision: current, so claim 4 rests on an uncommitted probe."},
 {"file": "website/docs/reference/evaluation.md", "line": 252, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "Item 2 of needs-coordinator.patch edits eval/mod.rs, which renders into this page, but the patch carries no regenerated page, so docs-generate must run after applying it."}]
```
