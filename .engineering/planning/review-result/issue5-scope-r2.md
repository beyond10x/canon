---
format: aep.planning-md/3
id: review-result:issue5-scope-r2
kind: review-result
status: active
title: Issue 5 stories, scope critic, round 2
relations:
- reviews: story:protocol-imports
- reviews: story:protocol-floor
- reviews: story:case-inputs
- reviews: story:semantic-diff
- reviews: story:case-composition
- reviews: story:ess-command-surface
revision: 1
---
needs-revision

story:case-inputs — the Outcome quotes TASKBOARD rows as "C-001 'case inputs'" and "C-005 'obligations derived by rule'", but TASKBOARD C-001 is "Define `protocol/1` minimal source model" and C-005 is "Implement obligations", both claimed by implemented stories; the body should drop the quoted labels and say the story closes a gap those stories left, that the downstream story:software-change-profiles named, so its trace to the parent is stated rather than borrowed — .engineering/planning/story/case-inputs.md:50 (against /home/timo/beyond10x/atlas/docs/design/governed-autonomy/TASKBOARD.md:15,19 and .engineering/planning/story/protocol-source-model.md:9)

story:case-inputs — Outcome 5 classifies only gaining and losing `applicable_when`, and Scope Out is silent on a changed `applicable_when`, a changed `inputs` declaration (values added or removed) and a changed `input` predicate leaf, so the "classifies every change" promise of story:semantic-diff is narrowed with nothing recording what `canon diff` and `canon floor` do with those changes; the body should classify them or list them under Out — .engineering/planning/story/case-inputs.md:72 and :97 (against .engineering/planning/story/semantic-diff.md:32)

**What I read.** 11 artifacts and sources: epic:canon-kernel, the three drafts, the three edited stories, issue 5, ADR 0086, design § 39.5 and TASKBOARD.
- Commands: `aep plan artifact show` for the epic and all six stories, `aep plan artifact graph`, `aep plan artifact findings review-result:issue5-scope-r1`, `gh issue view 5 -R beyond10x/canon`.
- Also read: the scope-r1 record and the other three round-1 records' messages (grep only), and story:software-change-profiles in full.

**Promises.** I extracted 6 and traced all 6.
- Imports story: story:protocol-imports.
- Floor story: story:protocol-floor.
- Order for story:semantic-diff: "Order after issue 5 (2026-10-07)".
- Order for story:case-composition: "Order" and "Scope", plus `depends_on story:protocol-imports`.
- `protocol.adopt/1` out: the Out sections of both new stories.
- Downstream case-inputs need: story:case-inputs.

The round-1 scope finding is fixed. story:ess-command-surface's Outcome now says later stories declare `--import` and `canon floor` in `ess/` and depend on it, and both depend_on edges exist.

**What I could not establish.**
- Out of my lane, not counted in the verdict:
  - story:semantic-diff's "ESS first" still says "the six revision pairs" while its Acceptance now has eight. Case-inputs adds two more pairs to the same named test, so "one per revision pair in the directory" will stop holding (design or acceptance).
  - Whether `aep plan artifact waves` actually puts story:semantic-diff in the next wave (parallel-safety). I did not run `waves`.
- The issue says `protocol.adopt/1` is filed in beyond10x/els, while both Out sections say it "belongs to the engineering-protocols repository". `gh repo view beyond10x/els` resolves to engineering-protocols, so I did not call it a finding.

```findings
- file: .engineering/planning/story/case-inputs.md
  line: 50
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the Outcome quotes TASKBOARD rows as C-001 'case inputs' and C-005 'obligations derived by rule', but TASKBOARD C-001 is 'Define protocol/1 minimal source model' and C-005 is 'Implement obligations', both claimed by implemented stories; the body should drop the quoted labels and say the story closes a gap those stories left, named by the downstream story:software-change-profiles, so its trace to the parent is stated rather than borrowed"
- file: .engineering/planning/story/case-inputs.md
  line: 72
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "Outcome 5 classifies only gaining and losing applicable_when and Scope Out is silent on a changed applicable_when, a changed inputs declaration and a changed input predicate leaf, so story:semantic-diff's promise to classify every change is narrowed with nothing recording what canon diff and canon floor do with those changes; classify them or list them under Out"
```
