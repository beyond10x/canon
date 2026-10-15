---
format: aep.planning-md/3
id: review-result:conformance-suite-acceptance-r1
kind: review-result
status: active
title: Conformance suite, acceptance critic, round 1
relations:
- reviews: story:conformance-suite
revision: 1
---
needs-revision

story:conformance-suite — the acceptance holds two independent statements, so one can pass while the other fails: the named test `conform_run_checks_requirements`, and a separate sentence, "Without the flag, the existing `conform_run` tests pass unchanged." That second sentence is checked by different tests (`crates/canon-cli/tests/conform_run.rs` and others) and needs its own place or a single-sentence restatement — .engineering/planning/story/conformance-suite.md:136

I did not flag the following:
- The acceptance names a failing state before the work: the `--requirements` flag does not exist, so the named test is red (`## ESS first`).
- Every output is a quoted string or an exit code.
- The count of 15 is right. `conformance/scenarios/` covers 15 distinct ids. `CANON-REASONS-001` is a scenario id whose `covers:` is `[CANON-EVIDENCE-003]`, so it is not a requirement.

**What I read:** 1 artifact (`story:conformance-suite`), using `aep plan artifact show story:conformance-suite`, `aep plan artifact kinds`, `aep plan artifact lifecycle story`, the critic procedure and rubric, `conformance/scenarios/*.yaml`, `crates/canon-cli/tests/`, and `git diff` for the story file.

**What I could not establish:**
- You asked for revision 9. The store prints revision 10, and the story file in that tree has uncommitted changes against git HEAD. I judged revision 10 as printed.
- The Outcome says "`task check` runs it with the flag", and `Taskfile.yml` is in scope. The acceptance has no observable for this, such as `task check` output or exit status. I did not file it as a finding because adding one would add a third statement to an acceptance already flagged for having two. It is better settled together with the fix above.
- Out of my lane, not counted toward the verdict: whether the `ess/` and `task ess-gate` items under `## ESS first` belong in the acceptance. That is coverage and design.

```findings
- file: .engineering/planning/story/conformance-suite.md
  line: 136
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance holds two independent statements, the named test conform_run_checks_requirements passing and 'Without the flag, the existing conform_run tests pass unchanged', so one can pass while the other fails"
```
