---
format: aep.planning-md/3
id: story:floor-guards-added-actions
kind: story
status: implemented
title: canon floor refuses an added action that produces the floor's finding without its capability
tags:
- from-finding
refs:
- provider: decision
  reference: D124
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon-cli/src/lib.rs
- confidence: cited
  path: crates/canon-cli/tests/floor.rs
- confidence: cited
  path: crates/canon/src/floor/mod.rs
- confidence: cited
  path: fixtures/investigation/floor
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T17:21:51Z", actor: "human:timo", revision: 3, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w11"}
- {from: "proposed", to: "active", at: "2026-11-22T17:21:51Z", actor: "human:timo", revision: 4, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w11"}
- {from: "active", to: "implemented", at: "2026-11-22T21:20:36Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
---
## Outcome

`canon floor` refuses a protocol that adds an action producing the floor's finding without the floor's capability, or the documented limit is lifted by another rule the operator picks. Today such a protocol passes.

## Finding

- Review of wave 9, area model, finding F1 (`review-result:review-wave-9-model`): `crates/canon/src/floor/mod.rs:7` admits every `EXPANSION`, and an added action is one.
- Reproduction (branch `review/model`, commit `c5c912273`, not merged): `crates/canon-cli/tests/adversary_w10_model.rs`, `an_added_action_that_publishes_without_the_floors_capability_breaches_the_floor`. It keeps `publish_finding` and adds `publish_finding_quietly` with the same precondition and no capability. `canon floor` prints `floor: holds`, `EXPANSION: 1` and exits 0. story:protocol-floor acceptance 3 refuses the same guarantee when the requirement is dropped instead of bypassed.
- Decided D124 (taken on default, 2026-11-22): the limit is documented in the `canon floor` help and `floor/mod.rs` (`abb281809`); a floor rule for added actions is a design change and gets this story.

## Acceptance

The reviewer's case passes: `canon floor` exits 1, prints `floor: breached` and names the added action. Every expectation of `floor_admits_only_tightening_and_expansion` still holds. The limit sentence leaves the `canon floor` help.

## Source

Close-out of wave 2026-11-20-w9. Not run here: nothing; numbers are the reviewer's.
