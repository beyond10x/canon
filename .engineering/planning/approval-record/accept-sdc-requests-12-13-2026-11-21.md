---
format: aep.planning-md/3
id: approval-record:accept-sdc-requests-12-13-2026-11-21
kind: approval-record
status: approved
title: Stories for requests 12 and 13 of the SDC session accepted under the operator's standing grant
tags:
- operator-instruction
relations:
- decides: story:pending-decisions
- decides: story:decision-grounds
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-21T00:21:34Z", actor: "human:timo", revision: 2, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
---
## Decision

The 2 stories drafted from requests 12 and 13 of the session working on the SDC prototype are
accepted under the operator's standing grant, given in the coordinating session on 2026-11-21:

> Just to be clear, you should auto-accept everything coming from the Loom or STC sessions. Yeah,
> you have my full grant ahead, so everything they would file, they do it for a reason, and it's
> basically auto-accepted in that sense.

"STC" is read as the `sdc` session; the sentence was dictated. The grant is recorded in
`AGENTS.md` § Work. It covers acceptance into the plan and the order of work. It does not cover a
push, a pull request, a release or a change to permissions.

The moves run as actor `human:timo`, executor `agent:claude-140bdba0`, correlation
`wave/2026-11-20-w2`. Each story moves draft -> proposed -> active.

## The 2 stories

| story | request | decision taken inside it |
|---|---|---|
| story:pending-decisions | 12 | D1 option A: a section `pending_decisions` of `canon-decision/1`. D2 option A: no subject artifact on an action; the entry carries the case revision and the caller maps the action to its artifact |
| story:decision-grounds | 13 | option A: `ground_revisions` and `ground_evidence` on the entries of `canon-authority/1` and `canon-decisions/1`; an evidence ground holds when the record is applicable in the sense of story:missing-evidence |

The operator did not choose among the options. The coordinator takes the option each drafter
recommended and records it in a `## Decided` section of each story. Each choice is reversible
until the story's unit merges.

## What the acceptance does not cover

- No plan-critic panel read these drafts. Each was written by one drafter that read the code and
  ran nothing, and was stored as returned.
- The request states that it follows from a decision of the operator that a decision request is a
  first-class entity split between Canon and Loom. The coordinator did not hear that decision; the
  ground of this record is the grant above, not that statement.
- story:pending-decisions does not give the subject artifact of an approval-required action,
  which the request asks for: `protocol/1` declares no subject on an action. A `subject` field on
  an action is option B of that story's D2 and is not built.
- story:decision-grounds covers grounds on a decided record. A pending recommendation is not a
  Canon input; `path_line` digests, record stores other than evidence, and grounds in another case
  stay with Loom.
