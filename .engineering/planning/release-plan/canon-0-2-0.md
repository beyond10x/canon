---
format: aep.planning-md/3
id: release-plan:canon-0-2-0
kind: release-plan
status: approved
title: Release Canon 0.2.0
relations:
- delivers: story:semantic-diff
- delivers: story:case-inputs
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-12T20:59:27Z", actor: "human:timo", revision: 2}
---
## Intent

Release Canon 0.2.0 with `canon diff` (story:semantic-diff) and case inputs in `protocol/1` with
obligations derived from them (story:case-inputs). The engineering-protocols repository's
story:software-change-profiles waits on a Canon release that carries case inputs in `protocol/1`
and obligations derived from them.

## Scope and procedure

Set the workspace version and lockfile to 0.2.0; add the 0.2.0 entry to CHANGELOG.md; update
installation and status prose; update the decision provenance version in
`conformance/scenarios/explanation.yaml` and `crates/canon-cli/tests/adversary_skel_cli.rs`,
keeping every other expected output; move both stories to implemented and regenerate the
documentation, so their capabilities read shipped. No protocol semantics change in the
preparation, so it needs no new specification or red test.

Run `task check` on the preparation, land it through a green pull request, then publish an
annotated bot tag 0.2.0 on that exact main commit and a bot-authored GitHub Release. Verify the
peeled tag, the required checks on that commit, the Release and its source archives. This is a
source release: no binary assets and no crates.io publication.

## Evidence

The wave's pull request https://github.com/beyond10x/canon/pull/12 merged as e3e19f5 with
`task check`, the documentation build, and security and privacy checks green on its head 429ee9c.
