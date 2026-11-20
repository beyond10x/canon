---
format: aep.planning-md/3
id: approval-record:accept-sdc-requests-9-11-2026-11-21
kind: approval-record
status: approved
title: Stories for requests 9, 10 and 11 of the SDC prototype accepted under the operator's instruction for request stories
tags:
- operator-instruction
relations:
- decides: story:protocol-writer
- decides: story:judgement-marking
- decides: story:evidence-redelivery
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-20T23:28:15Z", actor: "human:timo", revision: 2, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
---
## Decision

The 3 stories drafted from requests 9, 10 and 11 of the SDC prototype are accepted under the
operator's instruction of 2026-11-20 for stories drafted from that prototype's requests. The
instruction, verbatim:

> So those newly submitted things which you drafted, accept all of them. So they are not supposed
> to remain drafts. They should be implemented as soon as possible, like even with a higher
> priority than all the other pre-existing stories and epics.

The instruction was given when 7 such drafts existed. Requests 9, 10 and 11 reached the
coordinator after it, from the session working on the SDC prototype. That this instruction covers
the 3 later drafts is the coordinator's reading, not the operator's sentence. The coordinator
reported the reading to the operator on 2026-11-20 as an open decision with this default (draft
the 3 stories, accept them under the instruction, schedule them after the 7), and the operator's
next message, on 2026-11-21, told the session to continue and did not object. The operator's first
instruction for this plan was "accept everything, no more drafts".

The session working on the SDC prototype asked for these stories. Its word is not the operator's
and is not the ground of this record.

The moves run as actor `human:timo`, executor `agent:claude-140bdba0`, correlation
`wave/2026-11-20-w2`. Each story moves draft -> proposed -> active. Each move is reversible until
the story's unit merges.

## The 3 stories

| story | request | decision taken inside it |
|---|---|---|
| story:protocol-writer | 9 | option A: a canonical JSON writer, `model::write`; no builder |
| story:judgement-marking | 10 | option A: the three fields `author`, `revises`, `decided_by` and the enum `canon.protocol.Author`; the three checks are follow-up requests, not stories |
| story:evidence-redelivery | 11 | option A: an equal copy of an evidence record counts once; a differing copy is refused as `duplicate-identifier` |

The operator did not choose among the options. The coordinator takes the option each drafter
recommended and records it in a `## Decided` section of each story.

## What the acceptance does not cover

- No plan-critic panel read these drafts. Each was written by one drafter that read the code and
  ran nothing, and was stored as returned.
- story:evidence-redelivery covers one part of request 11: the refusal of an equal re-delivered
  evidence record. The rest of the request (a run-journal format and a fold, an `in-doubt` action
  status, a marking for actions safe to run again, an input digest in `computed_from`, the same
  rule for decisions and authority documents) is listed in that story's § Order as follow-ups and
  is not drafted. A journal format needs a decision against design § 33 first.
- story:judgement-marking adds the fields with no evaluation meaning. The checks
  `self-judged-evidence`, `unchecked-judgement` and `judgement-bypass`, and a revise step in
  `canon simulate`, are listed in that story's § Order as follow-ups and are not drafted.
- Request 8 (an explicit "absent" artifact revision) is not a story: the operator's relayed word
  was none for 8.
