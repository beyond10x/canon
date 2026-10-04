---
format: aep.planning-md/3
id: story:subject-bound-evidence-match
kind: story
status: implemented
title: Evidence matches can name the artifact a record must be about
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: conformance/scenarios/subject-bound-evidence-match.yaml
- confidence: cited
  path: crates/canon-docs/
- confidence: cited
  path: crates/canon/src/eval/actions.rs
- confidence: cited
  path: crates/canon/src/eval/claims.rs
- confidence: cited
  path: crates/canon/src/eval/outcomes.rs
- confidence: cited
  path: crates/canon/src/ir/
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/validate/
- confidence: cited
  path: ess/
- confidence: cited
  path: fixtures/investigation/subject-bound-evidence-match.yaml
- confidence: cited
  path: website/
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T07:10:25Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-04T07:10:25Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-04T08:26:14Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":3,"verification":1}}}
---
## Outcome

An evidence match in `protocol/1` can name the artifact its record must be about
(`evidence: {kind, subject}`), so a claim is established only by evidence about that artifact.
Today a match tests kind and result only; any record about a declared artifact at its current
revision satisfies it.

## Why

An independent review of ELS story:incident-response-protocol (wave 2026-10-04-w8) showed an
observation of an unchanged release discharging an obligation about the service. ELS worked around
it by declaring one artifact only; protocols with several artifacts need subject-bound matches.

## ESS first

The predicate union in `ess/domains/protocol.yaml` gains an optional `subject` on the evidence
match; red is `ess_model_matches` until the model follows. Acceptance: a scenario where a record
about another artifact does not satisfy the match.
