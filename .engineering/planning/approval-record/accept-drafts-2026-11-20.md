---
format: aep.planning-md/3
id: approval-record:accept-drafts-2026-11-20
kind: approval-record
status: approved
title: 'Operator accepted every draft on 2026-11-20: 7 stories, 1 epic, 2 visions, 1 verification report'
tags:
- operator-instruction
relations:
- decides: epic:protocol-analysis
- decides: story:canon-mutate
- decides: story:capability-scope
- decides: story:ess-command-surface
- decides: story:invalidation-evaluation-cost
- decides: story:protocol-floor
- decides: story:protocol-imports
- decides: story:revision-ordering
- decides: vision:O2
- decides: vision:governed-autonomy
- decides: verification-report:assertion-expressions
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-20T20:58:27Z", actor: "human:timo", revision: 2, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w1"}
---
## Decision

The operator accepted every draft in this store on 2026-11-20, in the session that coordinates the
wave plan of that date. The operator's message, verbatim:

> Okay, so at first basically take all epics and all stories which are there or designs in draft,
> accept everything, no more drafts, maybe rescope the planning and then prioritize and start
> implementing this with the slash implementing skill and use Opus subagents, not Fable.

The moves are the operator's; the coordinating session only runs them (actor `human:timo`,
executor `agent:claude-140bdba0`, correlation `wave/2026-11-20-w1`).

## What was in draft

Read from `aep plan artifact list --status draft` on `main` at `8f1f452`, 11 artifacts:

| artifact | move |
|---|---|
| epic:protocol-analysis | draft -> proposed -> active |
| story:canon-mutate | draft -> proposed -> active |
| story:capability-scope | draft -> proposed -> active |
| story:ess-command-surface | draft -> proposed -> active |
| story:invalidation-evaluation-cost | draft -> proposed -> active |
| story:protocol-floor | draft -> proposed -> active |
| story:protocol-imports | draft -> proposed -> active |
| story:revision-ordering | draft -> proposed -> active |
| vision:O2 | draft -> in_review -> approved |
| vision:governed-autonomy | draft -> in_review -> approved |
| verification-report:assertion-expressions | draft -> approved |

The story and epic lifecycles here have no `accepted` rung; `active` is the accepted status
(`aep plan artifact lifecycle story`). The vision lifecycle's accepted status is `approved`.
`verification-report` declares no lifecycle; `approved` is the status the store's release plans and
approval records use for a record nobody disputes.

`docs/design/canon-protocol-calculus-design.md` carried `Status: Design proposal` on its line 3. It
is the one design document in draft outside the store, and the same message accepts it; its status
line is changed in the same commit.

## What the acceptance does not cover

- No plan-critic panel ran on these drafts before the moves. The operator accepted them as written
  and asked for a rescope afterwards; the rescope is recorded in each story's `## Scope` section and
  typed scope entries, by `story-scoper` agents, after this record.
- story:ess-command-surface named no red test and story:invalidation-evaluation-cost named no
  surface when they were accepted. Both gaps are closed by the rescope, not by this record.
- story:revision-ordering still has an open design step (the revision relation, Atlas ADR 0084
  § Open). Accepting the story does not settle it; the story settles it in its first step.

## Wave plan

The accepted stories are sequenced in `docs/plan/wave-plan-2026-11-20.md`, derived with
`aep plan artifact waves --kind story --status active` after the rescope. Sub-agents run on the
Opus model, as the operator's message says.
