---
format: aep.planning-md/3
id: approval-record:accept-sdc-request-14-2026-11-21
kind: approval-record
status: approved
title: Story for request 14 of the SDC session accepted under the operator's standing grant
tags:
- operator-instruction
relations:
- decides: story:child-outcome-subject
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-21T00:43:59Z", actor: "human:timo", revision: 2, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
---
## Decision

The story drafted from request 14 of the session working on the SDC prototype is accepted on
arrival under the operator's standing grant of 2026-11-21, recorded in `AGENTS.md` § Work:

> Just to be clear, you should auto-accept everything coming from the Loom or STC sessions. Yeah,
> you have my full grant ahead, so everything they would file, they do it for a reason, and it's
> basically auto-accepted in that sense.

"STC" is read as the `sdc` session. The grant covers acceptance into the plan and the order of
work; it does not cover a push, a pull request, a release or a change to permissions.

The moves run as actor `human:timo`, executor `agent:claude-140bdba0`, correlation
`wave/2026-11-20-w2`. The story moves draft -> proposed -> active.

## The story

| story | request | decision taken inside it |
|---|---|---|
| story:child-outcome-subject | 14 | option A: an optional `subject` on the `children` form; the form reads only the `case_outcome` records about that artifact |

The operator did not choose among the options; the coordinator takes the drafter's
recommendation and records it in a `## Decided` section. It is reversible until the unit merges.

## What the acceptance does not cover

- No plan-critic panel read the draft. One drafter wrote it from the code and ran nothing.
- The drafter found that story:child-outcome-revision already gives most of the request: with
  `revision: current`, a child opened at a superseded revision of the wave is not read, a dropped
  one included. This story adds only the missing part, the artifact the form selects by.
- A child that was opened and has no `case_outcome` record is invisible to Canon: "every" ranges
  over the records given. Writing an `open` record when a child is opened is the runner's duty;
  the story states it in the reference and adds no check for it.
