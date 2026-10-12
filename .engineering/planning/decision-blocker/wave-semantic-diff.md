---
format: aep.planning-md/3
id: decision-blocker:wave-semantic-diff
kind: decision-blocker
status: cleared
title: Nobody has approved the semantic-diff wave toward the case-inputs release
revision: 4
transitions:
- {from: "open", to: "cleared", at: "2026-10-12T15:56:42Z", actor: "human:timo", revision: 4}
---
## Question

Approve the next wave, `semantic-diff`, as the first step toward the release that carries case
inputs in `protocol/1` and obligations derived from them (story:case-inputs).

## Options

- A: wave `semantic-diff` with one unit, story:semantic-diff (store wave 12; every
  story it depends on is implemented). story:case-inputs depends on it and follows as the next
  wave; release 0.2.0 is cut after that wave merges.
- B (recommended): one wave carrying story:semantic-diff then story:case-inputs serially on one integration
  branch and one pull request: one CI full gate instead of two, a larger review.
- C: wave 12 as the store lists it, story:semantic-diff with story:conformance-suite; the second
  unit is not on the path to the release.

## Cost

One tree per unit; each tree builds into its own `target/`. The build size of this workspace has
not been measured; the estimate is at most 8G per tree.

## Recommendation

B: story:case-inputs cannot start before story:semantic-diff merges, so B reaches the release in one pull request.

## Decided (2026-10-12)

B: story:semantic-diff then story:case-inputs, serially on the integration branch
`wave/semantic-diff`, one pull request, release 0.2.0 after it merges. Each build target is held
to 8G, one build at a time.
