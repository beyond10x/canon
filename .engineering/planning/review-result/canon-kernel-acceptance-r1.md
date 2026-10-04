---
format: aep.planning-md/3
id: review-result:canon-kernel-acceptance-r1
kind: review-result
status: active
title: Canon kernel decomposition — acceptance critic, round 1
relations:
- reviews: epic:canon-kernel
- reviews: story:action-admissibility
- reviews: story:canon-ir
- reviews: story:conformance-suite
- reviews: story:evidence-freshness
- reviews: story:evidence-revision-binding
- reviews: story:explanation
- reviews: story:obligations
- reviews: story:outcomes
- reviews: story:protocol-source-model
- reviews: story:semantic-diff
- reviews: story:three-valued-claims
revision: 1
---
needs-revision
story:outcomes — the acceptance checks only the `legitimate`/`blocked` split and never shows a case terminating through an undeclared outcome being refused, which is half the Outcome and the story's title — .engineering/planning/story/outcomes.md:37
story:action-admissibility — the Outcome promises that a scenario records which status a denied capability reports, but the acceptance names no denied-capability case and no `--authority` input, so that transition can pass unchecked — .engineering/planning/story/action-admissibility.md:36
story:action-admissibility — the acceptance reads "over the investigation fixture" an action with a precondition and a capability requirement, but design § 12 has no preconditions or capabilities and the story does not say it adds them (the source-model story extends the fixture only "with an artifact") — .engineering/planning/story/action-admissibility.md:36
story:obligations — the acceptance reports an obligation "over the investigation fixture", but the design § 12 fixture declares no obligation and no story says who adds one, so the check cannot run as written — .engineering/planning/story/obligations.md:33
story:three-valued-claims — the Outcome says the story decides and records the evidence-disagrees-with-itself case, but the acceptance covers only no evidence, a non-`survived` result and the supporting-plus-survived case — .engineering/planning/story/three-valued-claims.md:37
story:canon-ir — the acceptance joins two independent outcomes, byte-identical IR for reordered/defaulted input "and the emitted IR contains no filesystem path", so one can pass while the other fails — .engineering/planning/story/canon-ir.md:26

What I read: 13 of 13 ids given (11 stories, 2 decision-blockers). I ran `aep plan artifact kinds`, `list`, `lifecycle` (decision-blocker, blocker, story) and `show` on every id, plus `show epic:canon-kernel`. I read design §§ 8–10, 12 and 31–32, and `crates/canon/src/lib.rs`.

What I could not establish:
- The decision-blockers have no acceptance section. I did not flag this, because `.claude/plugins/cache/b10x/aep/0.19.2/skills/planning/SKILL.md:328` says a blocker is cleared by the answer to its question.
- `story:explanation` has an Outcome clause (the document records protocol id and revision, semantics version, case revision, evidence set, authority decisions and instant) that its acceptance never checks. I treated that as thin ambition, not a missing check, so it is not a finding.
- Out of my lane: the `depends_on` edges, whether C-008 or C-009 sit correctly in the ordering, and whether the fixture extension in `story:protocol-source-model` belongs to its scope. The last is also what the fixture findings above rest on.

```findings
- file: .engineering/planning/story/outcomes.md
  line: 37
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: the acceptance checks only the legitimate/blocked split and never shows a case terminating through an undeclared outcome being refused, which is half the Outcome and the story's title
- file: .engineering/planning/story/action-admissibility.md
  line: 36
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the Outcome promises that a scenario records which status a denied capability reports, but the acceptance names no denied-capability case and no --authority input, so that transition can pass unchecked
- file: .engineering/planning/story/action-admissibility.md
  line: 36
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the acceptance reads "over the investigation fixture" an action with a precondition and a capability requirement, but design § 12 has no preconditions or capabilities and the story does not say it adds them
- file: .engineering/planning/story/obligations.md
  line: 33
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the acceptance reports an obligation over the investigation fixture, but the design § 12 fixture declares no obligation and no story says who adds one, so the check cannot run as written
- file: .engineering/planning/story/three-valued-claims.md
  line: 37
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the Outcome says the story decides and records the evidence-disagrees-with-itself case, but the acceptance covers only no evidence, a non-survived result and the supporting-plus-survived case
- file: .engineering/planning/story/canon-ir.md
  line: 26
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: the acceptance joins two independent outcomes, byte-identical IR for reordered/defaulted input and the IR containing no filesystem path, so one can pass while the other fails
```
