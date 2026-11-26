---
format: aep.planning-md/3
id: approval-record:accept-trigger-rules-2026-11-26
kind: approval-record
status: approved
title: The trigger-rule request is accepted and D1 is decided as A
tags:
- operator-instruction
relations:
- decides: epic:trigger-rules
- decides: story:protocol-access-class
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-26T11:25:20Z", actor: "human:operator", revision: 2}
---
## Decision

The owner, 2026-11-26, relayed by the operator: the trigger-rule kernel lives in Canon "for
simulation purposes" and the evaluator in `beyond10x/runbook`. The request from the consumer
coordinator is accepted into the plan under the standing grant for consumer requests (`AGENTS.md`
§ Work): epic:trigger-rules and its five stories move to `active`.

D1, decided by the operator on the owner's behalf on 2026-11-26: option A. Each action of
`protocol/1` gets an optional `access: read_only | write`, absent meaning `write`; the protocol's
class is derived from its actions; `canon validate` refuses `read_only` on an action that revises an
artifact or opens a case. Reason given: additive and fails safe.

## Options put to it

- A: optional `access` per action, class derived (the pick).
- B: class derived from existing fields only; an action that writes outside Canon without
  `revises` would read as `read_only`.
- C: the consumer declares the class; the rule's flag is then unchecked.
