---
format: aep.planning-md/3
id: review-result:conformance-suite-acceptance-r2
kind: review-result
status: active
title: Conformance suite, acceptance critic, round 2
relations:
- reviews: story:conformance-suite
revision: 1
---
needs-revision

story:conformance-suite — the Outcome promises "`task check` runs it with the flag" and `Taskfile.yml` is in scope, but the acceptance names no observable for it, so the story can pass `conform_run_checks_requirements` while `task check` still never runs `conform run`. `Taskfile.yml`'s `check` task has no `conform` invocation today, so this is a real transition that nothing checks. The acceptance should say what shows the recipe changed, for example that the named test also asserts the `check` recipe contains `conform run --requirements conformance/requirements.yaml`. — .engineering/planning/story/conformance-suite.md:143

Round 1's finding is fixed. The "existing `conform_run` tests pass unchanged" sentence is gone. Its content now lives inside the one named test, as the first assertion: output with no `requirements:` line and exit 0 when run without the flag.

Not flagged:
- The failing state before the work is named (`## ESS first`, "Red on that commit"). `git grep conform_run_checks_requirements` finds only the story, so the test does not exist yet.
- Every assertion is an exit status or a quoted output string.
- The count of 15 is correct. `conformance/scenarios/` covers 15 distinct requirement ids.

**What I read:** 2 artifacts, `story:conformance-suite` and `review-result:conformance-suite-acceptance-r1`. I used `aep plan artifact show` on each, plus the critic procedure and rubric, `Taskfile.yml`, `conformance/scenarios/`, `crates/canon-cli/tests/` and `git grep`.

**What I could not establish:**
- You named revision 11, but the store prints revision 12. I judged revision 12 as printed. The story file in that tree also has uncommitted changes against git HEAD.
- Out of my lane, not counted toward the verdict: whether the `ess/` and `task ess-gate` items under `## ESS first` belong in the story's scope. That is for the scope and design critics.

```findings
- file: .engineering/planning/story/conformance-suite.md
  line: 143
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the Outcome promises 'task check runs it with the flag' and Taskfile.yml is in scope, but the acceptance names no observable for it, so the named test can pass while the check recipe still has no conform run invocation"
```
