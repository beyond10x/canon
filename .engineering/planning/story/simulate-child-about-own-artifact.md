---
format: aep.planning-md/3
id: story:simulate-child-about-own-artifact
kind: story
status: active
title: canon simulate ends a child about the space's own artifact against the revision it was opened at
tags:
- from-finding
refs:
- provider: decision
  reference: D125
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon-cli/src/lib.rs
- confidence: cited
  path: crates/canon/src/simulate/recovery.rs
- confidence: cited
  path: crates/canon/tests/adversary3_simulate_recovery.rs
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T17:21:51Z", actor: "human:timo", revision: 3, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w11"}
- {from: "proposed", to: "active", at: "2026-11-22T17:21:51Z", actor: "human:timo", revision: 4, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w11"}
---
## Outcome

In `canon simulate`, a child opened about the artifact the space already uses can end against the revision it was opened at, after that artifact is revised; or the story says which of story:child-outcome-recovery's outcome 3 and outcome 5 holds here and the walk follows it.

## Finding

- Review of wave 9, area simulate, finding F1 (`review-result:review-wave-9-simulate`): `crates/canon/src/simulate/recovery.rs:264` (`split.redirected.get(action)?`) gives no redirected dimension for such a child, so it never ends against the earlier revision. Two dead ends go unreported, depending only on artifact names.
- story:child-outcome-recovery outcome 3 and outcome 5 contradict each other for this case.
- Decided D125 (taken on default, 2026-11-22): documented as a limit in the `canon simulate` help (`abb281809`): "a child opened about the artifact the space already uses does not end against an earlier revision; the walk of such a protocol is unchanged". The reviewer's case `a_child_about_the_spaces_own_subject_may_end_against_the_earlier_revision` (branch `review/simulate`, `144493d9f`) was not merged.

## Acceptance

The reviewer's case, restored, passes or is replaced by a case that pins the decided reading; the two dead ends are reported in the walk or the help states why not; every existing simulate fixture gives unchanged bytes.

## Source

Close-out of wave 2026-11-20-w9. Not measured: the state count of a protocol of this shape after a change.
