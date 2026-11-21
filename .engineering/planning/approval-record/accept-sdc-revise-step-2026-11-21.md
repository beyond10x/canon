---
format: aep.planning-md/3
id: approval-record:accept-sdc-revise-step-2026-11-21
kind: approval-record
status: approved
title: Story for the SDC session's revise-step request accepted under the operator's standing grant
tags:
- operator-instruction
relations:
- decides: story:simulate-revise-step
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-21T04:50:38Z", actor: "human:timo", revision: 2, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w5"}
---
## Decision

The story drafted from the `sdc` session's request for a revise step in the `canon simulate` walk
is accepted under the operator's standing grant of 2026-11-21, recorded in `AGENTS.md` § Work:

> Just to be clear, you should auto-accept everything coming from the Loom or STC sessions. Yeah,
> you have my full grant ahead, so everything they would file, they do it for a reason, and it's
> basically auto-accepted in that sense.

"STC" is read as the `sdc` session. The grant covers acceptance into the plan and the order of
work; it does not cover a push, a pull request, a release or a change to permissions.

That session first asked for a draft only ("not accepted yet; I decide once the SDC fix shows
whether simulate reaches exit 0 without it"). After the classification of the 87 findings on its
fixed protocols it decided, on 2026-11-21: "Choice 1: A, and with that the story is accepted under
the grant".

The moves run as actor `human:timo`, executor `agent:claude-140bdba0`, correlation
`wave/2026-11-20-w5`. The story moves draft -> proposed -> active.

## The story

| story | request | decision taken inside it |
|---|---|---|
| story:simulate-revise-step | revise step (2026-11-21) | option A: a revise step only for `revises` of an admissible action, the case revision fixed, no language change |

## What the acceptance does not cover

- No plan-critic panel read the draft. One drafter wrote it from the code, ran the release binary
  of `impl/canon-simulate` and built nothing.
- The counts after a revise step (48 of 87 findings leave, 39 stay) are expected from the
  classification and were not run.
- Option C (`revised_by` on an artifact) is not accepted; it is to be a separate draft the `sdc`
  session decides after this story is implemented.
- The four false findings of class e are not this story's: they go to a separate story for a
  child-outcome step.
