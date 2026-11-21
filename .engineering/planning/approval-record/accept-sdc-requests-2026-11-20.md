---
format: aep.planning-md/3
id: approval-record:accept-sdc-requests-2026-11-20
kind: approval-record
status: approved
title: Operator accepted the 7 stories drafted from the SDC prototype's requests and put them first
tags:
- operator-instruction
relations:
- decides: story:authority-revision-binding
- decides: story:missing-evidence
- decides: story:check-subject-slices
- decides: story:canon-simulate
- decides: story:child-outcome-revision
- decides: story:supplied-evidence
- decides: story:argument-validator
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-20T22:35:00Z", actor: "human:timo", revision: 2, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
---
## Decision

The operator accepted the 7 stories drafted from the SDC prototype's requests, in the coordinating
session on 2026-11-20, and put them ahead of the stories accepted earlier that day. The operator's
message, verbatim:

> So those newly submitted things which you drafted, accept all of them. So they are not supposed
> to remain drafts. They should be implemented as soon as possible, like even with a higher
> priority than all the other pre-existing stories and epics.

The moves are the operator's; the coordinating session only runs them (actor `human:timo`,
executor `agent:claude-140bdba0`, correlation `wave/2026-11-20-w2`). Each story moves
draft -> proposed -> active.

## The 7 stories

| story | request | decision taken inside it |
|---|---|---|
| story:authority-revision-binding | 1 | option A: every bound entry applies only at its case revision; a lasting deny is written unbound |
| story:missing-evidence | 2 | D1 option 2 (a record is applicable when binding and freshness leave it and no claim excludes it); D2 option B (a new top-level section) |
| story:check-subject-slices | 3 | option A: one slice per subject, the whole check refused when any slice exceeds the bound |
| story:canon-simulate | 4 | option A: authority and decision steps are steps of the walk |
| story:child-outcome-revision | 5 | option A: `revision: current` on the `children` form |
| story:supplied-evidence | 6 | option A: `supplied_by` on an evidence kind |
| story:argument-validator | 7 | none needed |

Six drafts held a "Decision needed" section. The operator did not choose among the options. The
coordinator takes the option each drafter recommended, which is also what the session working on
the SDC prototype recommended for each, and records it in a `## Decided` section of each story.
The operator's standing instruction for this plan is to decide and report, and each choice is
reversible until its unit merges.

## What the acceptance does not cover

- No plan-critic panel read these drafts. Each was written by one drafter and stored as returned.
- Requests 8, 9 and 10 of the plan page are not stories and are not accepted.
- The reorder of the plan is in `docs/plan/wave-plan-2026-11-20.md`, § Replan.
