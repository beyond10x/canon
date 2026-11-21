---
format: aep.planning-md/3
id: approval-record:accept-sdc-request-15-2026-11-21
kind: approval-record
status: approved
title: Story for request 15 of the SDC session accepted under the operator's standing grant
tags:
- operator-instruction
relations:
- decides: story:simulate-record-groups
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-21T04:43:46Z", actor: "human:timo", revision: 2, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w5"}
---
## Decision

The story drafted from request 15 of the session working on the SDC prototype is accepted on
arrival under the operator's standing grant of 2026-11-21, recorded in `AGENTS.md` § Work:

> Just to be clear, you should auto-accept everything coming from the Loom or STC sessions. Yeah,
> you have my full grant ahead, so everything they would file, they do it for a reason, and it's
> basically auto-accepted in that sense.

"STC" is read as the `sdc` session. The grant covers acceptance into the plan and the order of
work; it does not cover a push, a pull request, a release or a change to permissions.

The moves run as actor `human:timo`, executor `agent:claude-140bdba0`, correlation
`wave/2026-11-20-w5`. The story moves draft -> proposed -> active.

## The story

| story | request | decision taken inside it |
|---|---|---|
| story:simulate-record-groups | 15 | option A: `records_together` on an action; the walk adds a group in one step and never offers a grouped kind alone |

The `sdc` session, which the operator named the orchestrator of this plan on 2026-11-21, chose
option A on 2026-11-21 after the drafter's figures; the story's `## Decided` section quotes it.
The operator did not choose among the options. The choice is reversible until the unit merges.

## What the acceptance does not cover

- No plan-critic panel read the draft. One drafter wrote it from the code, ran the release binary
  of `impl/canon-simulate` and built nothing.
- The figures for options A and B come from a Python walk the drafter wrote over the compiled IR.
  It reproduces the binary's numbers for today's walk on three protocols. The `sdc` session made
  it a condition that the story's acceptance repeats the figures from the Rust binary.
- The request arrived while story:canon-simulate, on which the story depends, was in its second
  adversary pass and not merged.
