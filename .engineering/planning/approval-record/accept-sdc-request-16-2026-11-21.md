---
format: aep.planning-md/3
id: approval-record:accept-sdc-request-16-2026-11-21
kind: approval-record
status: approved
title: Story for request 16 of the SDC session accepted under the operator's standing grant
tags:
- operator-instruction
relations:
- decides: story:simulate-child-outcome-step
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-21T05:02:20Z", actor: "human:timo", revision: 2, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w5"}
---
## Decision

The story drafted from request 16 of the session working on the SDC prototype is accepted on
arrival under the operator's standing grant of 2026-11-21, recorded in `AGENTS.md` § Work:

> Just to be clear, you should auto-accept everything coming from the Loom or STC sessions. Yeah,
> you have my full grant ahead, so everything they would file, they do it for a reason, and it's
> basically auto-accepted in that sense.

"STC" is read as the `sdc` session. The grant covers acceptance into the plan and the order of
work; it does not cover a push, a pull request, a release or a change to permissions.

That session's words of 2026-11-21: "A separate story for the `case_outcome` step that may arrive
after the opening action was taken (request 16, accepted on arrival)".

The moves run as actor `human:timo`, executor `agent:claude-140bdba0`, correlation
`wave/2026-11-20-w5`. The story moves draft -> proposed -> active.

## The story

| story | request | decision taken inside it |
|---|---|---|
| story:simulate-child-outcome-step | 16 | option A: one mark per opening action; a child-outcome step while a mark is set; a terminal class also replaces `open` |

The `sdc` session did not choose among the options; the coordinator takes the drafter's
recommendation and records it in a `## Decided` section. It is reversible until the unit merges.

## What the acceptance does not cover

- No plan-critic panel read the draft. One drafter wrote it from the code, ran the release binary
  of `impl/canon-simulate` and built nothing.
- Every figure for the options comes from a Python walk the drafter wrote over the compiled IR,
  which reproduces the binary's numbers for today's rule. The story's acceptance repeats the
  figures from the Rust binary.
- The limit line the story removes does not exist yet: story:canon-simulate adds it (its rule R7)
  before it merges.
- The drafter names four sentences of story:simulate-record-groups and story:simulate-revise-step
  that read differently once a walk state carries marks; they are in the story's § Order and are
  for those units.
