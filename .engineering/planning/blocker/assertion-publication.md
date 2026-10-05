---
format: aep.planning-md/3
id: blocker:assertion-publication
kind: blocker
status: cleared
title: Canon publication requires its App-only branch authority
relations:
- blocks: story:assertion-expressions
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-05T20:59:48Z", actor: "human:timo", revision: 3}
---
Implementation and repository gates are complete. Source publication is blocked by the repository's
remote authority configuration, outside the assertion implementation scope.

`b10x-gates check` passed and signed the candidate receipt. `b10x-gates publish` refused with
`branch authority missing or ambiguous` before pushing. A read-only GitHub ruleset query returned
an empty list for beyond10x/canon. The required name is `b10x-bot-branch-authority`.

The repository instruction forbids reaching the same publication result through another tool after
a Gates refusal. No alternate push was attempted. Restore the required App-only branch authority
through the authorized repository administration path, then rerun the signed publication step.
Engineering-protocols depends on exact source commit `cf2ae40c2284c86d05b33c3003c45c7351f1150b`;
it was tested from the local Git object and has not been published with an unreachable dependency.

The source is committed and retained in its managed tree and archive. No release tag or deployment
has been created. Source verification is recorded separately in verification-report:assertion-expressions.

## Resolution

The operator requested submitted green pull requests on 2026-10-05. Canon's missing
`b10x-bot-branch-authority` ruleset was restored through the bot API, using the exact App-only
configuration required by Gates and already installed on Engineering Protocols. Rule 24531196
is active; its sole bypass actor is the b10x bot App (4579525). No checks or protections were
removed. Source submission can proceed through the normal signed Gates publication path.
