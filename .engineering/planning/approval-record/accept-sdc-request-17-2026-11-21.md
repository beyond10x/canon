---
format: aep.planning-md/3
id: approval-record:accept-sdc-request-17-2026-11-21
kind: approval-record
status: approved
title: Story for request 17 of the SDC session accepted under the operator's standing grant
tags:
- operator-instruction
relations:
- decides: story:child-outcome-recovery
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-21T07:04:05Z", actor: "human:timo", revision: 2, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w6"}
---
## Decision

The story drafted from request 17 of the session working on the SDC prototype is accepted under
the operator's standing grant of 2026-11-21, recorded in `AGENTS.md` § Work:

> Just to be clear, you should auto-accept everything coming from the Loom or STC sessions. Yeah,
> you have my full grant ahead, so everything they would file, they do it for a reason, and it's
> basically auto-accepted in that sense.

"STC" is read as the `sdc` session. The grant covers acceptance into the plan and the order of
work; it does not cover a push, a pull request, a release or a change to permissions.

That session asked for a draft first ("I accept after your drafter states the options") and, with
the options before it, wrote on 2026-11-21: "`story:child-outcome-recovery` is accepted as drafted
(L0, no language change; W1 in `canon simulate`, wave 9 or later, after the child-outcome step,
the revise step and story:child-outcome-subject)."

The moves run as actor `human:timo`, executor `agent:claude-140bdba0`, correlation
`wave/2026-11-20-w6`. The story moves draft -> proposed -> active.

## The story

| story | request | decision taken inside it |
|---|---|---|
| story:child-outcome-recovery | 17 | L0: no language change, `revision: current` already gives the recovery; W1: in `canon simulate`, one `case_outcome` dimension per `case_open` subject, a revise step that sets aside only the revised subject's records, a third value of the opening mark |

## What the acceptance does not cover

- No plan-critic panel read the draft. One drafter wrote it from the code and ran an older binary
  on the request's real case; the behaviour of `revision: current` is read from the evaluator and
  the scenario, not run.
- The two `canon simulate` stories it builds on are not built: its red run and every walk count
  are inferred.
- The drafter found, out of scope, that an expired `case_outcome` record held by rule R2a of
  story:canon-simulate is kept at a superseded revision, so a form with `revision: current` does
  not read it in the walk while the evaluator does. It matters only when an evidence match also
  reads `case_outcome`. It is in the plan page's backlog.
