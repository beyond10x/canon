---
format: aep.planning-md/3
id: review-result:issue5-scope-r1
kind: review-result
status: active
title: Issue 5 stories, scope critic, round 1
relations:
- reviews: story:protocol-imports
- reviews: story:protocol-floor
- reviews: story:semantic-diff
- reviews: story:case-composition
revision: 1
---
needs-revision

protocol-floor — states that story:ess-command-surface declares `canon floor` "as `canon diff` is", but that story's Outcome lists only `validate`, `compile`, `evaluate`, `conform run` and `diff`, so nothing claims the `canon floor` declaration; the body should say that story's list must gain `floor`, or the story should declare it itself — .engineering/planning/story/protocol-floor.md:82 (against .engineering/planning/story/ess-command-surface.md:20)

Every other promise in the set traces to the issue, and I found nothing else to change.

**What I read.** Seven artifacts: epic:canon-kernel, the four drafted stories, story:ess-command-surface and issue 5. I ran `aep plan artifact show` on the epic and on the four stories, `show story:ess-command-surface`, `aep plan artifact graph` and `gh issue view 5 -R beyond10x/canon`. I also read ADR 0086, design § 39.5, `crates/canon/src/model/mod.rs`, `ess/domains/protocol.yaml` and `git status`. I did not run `aep plan artifact kinds` or `relations`.

**Promises.** The issue makes five promises and all five trace to the set; the epic itself is not the source of any of them.
- A story for imports: story:protocol-imports.
- A story for the floor construct: story:protocol-floor.
- An order for story:semantic-diff: "Order after issue 5 (2026-10-07)".
- An order for story:case-composition: "Order" and "Scope", plus `depends_on story:protocol-imports`.
- `protocol.adopt/1` stays out: it is named in the Out sections of both new stories.

**What I could not establish.**
- Whether `refines` and `--import` on `diff` and `check` are wanted beyond the issue's words. I judged them traceable to ADR 0086 ("may tighten claims and preconditions"; "`canon check` passes") and did not call them reach.
- Out of my lane, not counted in the verdict:
  - protocol-imports's refusal of `canon generate` is stated in Out but not checked by the acceptance (acceptance).
  - protocol-floor's claim that it shares no source file with story:case-composition is a parallel-safety question.
  - `refines` does not cover `evidence_kinds`, so a profile cannot tighten `max_age`. ADR 0086 does not require it, so this is a design question.

```findings
- file: .engineering/planning/story/protocol-floor.md
  line: 82
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the body says story:ess-command-surface declares `canon floor` as it does `canon diff`, but that story's Outcome lists only validate, compile, evaluate, conform run and diff, so nothing claims the `canon floor` declaration; the body should say that story's list must gain `floor`, or declare it itself"
```
