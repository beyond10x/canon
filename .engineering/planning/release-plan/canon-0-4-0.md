---
format: aep.planning-md/3
id: release-plan:canon-0-4-0
kind: release-plan
status: draft
title: Release Canon 0.4.0
relations:
- delivers: story:evidence-independence
- delivers: story:ess-pin-0-59
revision: 1
---
## Intent

Release Canon 0.4.0 with evidence requirements that demand an independent producer
(story:evidence-independence, CANON-INDEPENDENCE-001), on a specification held to ess 0.59.0
(story:ess-pin-0-59). New `protocol/1`, `canon-evidence/1` and `canon-case/1` fields are a feature,
so the minor version moves.

## Wave

One wave, branch `wave/independence`, one pull request:

1. story:ess-pin-0-59: the pin moves from 0.53.0 to 0.59.0; no behaviour change.
2. story:evidence-independence: spec first, red, then implementation; one adversary pass.

Checked and not in the wave: evidence bound to its subject revision (story:evidence-revision-binding,
story:subject-bound-evidence-match), freshness (story:evidence-freshness) and upstream invalidation
(story:invalidation-rules) shipped in 0.1.0 to 0.3.0. The requirements catalogue
(story:conformance-suite) stays for a later wave.

## Scope and procedure

Set the workspace version and lockfile to 0.4.0; add the 0.4.0 entry to CHANGELOG.md; update
installation and status prose; update the decision provenance version in
`conformance/scenarios/explanation.yaml` and `crates/canon-cli/tests/adversary_skel_cli.rs`,
keeping every other expected output; move the stories to implemented and regenerate the
documentation.

Land the preparation through a green pull request, then publish an annotated bot tag 0.4.0 on that
exact main commit and a bot-authored GitHub Release. Verify the peeled tag, the required checks on
that commit, the Release and its source archives. Source release only.
