---
format: aep.planning-md/3
id: approval-record:accept-evidence-digest-2026-11-22
kind: approval-record
status: approved
title: Timo picked A, optional evidence digest, for the approvals finding on 2026-11-22
tags:
- operator-instruction
relations:
- decides: story:evidence-digest-binds-approvals
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-22T22:17:48Z", actor: "human:timo", revision: 2, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
---
## Decision

Timo, 2026-11-22 23:17:01 CET, typed in the `host-operator` session and relayed by it: "A: optional digest (Recommended)" for canon wave 12's approvals finding. Accept `story:evidence-digest-binds-approvals` as the single story of the wave after wave 12: a new optional sha256 digest per evidence ground in canon-authority/1 and canon-decisions/1; mismatch fails as `content_changed`; inputs without the field keep today's bytes. Wave 12 releases without it.

## Options put to him

- A. Optional digest per evidence ground, no new format version. The pick.
- B. New format versions requiring a digest on every evidence ground.
- C. An ingestion check outside Canon.

## Source

- The coordinator's question to `host-operator` of 2026-11-22 after unit 6 of wave 12, and `host-operator`'s relay of his pick (cross-session message).
- Accepting the draft is his act through this pick (`~/.claude/CLAUDE.md` § Host operator: its relays count as Timo's).
