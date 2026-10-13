---
format: aep.planning-md/3
id: release-plan:canon-0-3-0
kind: release-plan
status: approved
title: Release Canon 0.3.0
relations:
- delivers: story:evidence-order-predicate
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-13T02:29:55Z", actor: "human:timo", revision: 2}
---
## Intent

Release Canon 0.3.0 with predicates over evidence order in `protocol/1`
(story:evidence-order-predicate): `first` and `before`, ordered on `observed_at` over the evidence
history. A consumer's bug-fix profile needs a failing result recorded before a fix to count after
it; Canon 0.2.0 binds every evidence record to the current revision. New `protocol/1` forms are a
feature, so the minor version moves.

## Scope and procedure

Set the workspace version and lockfile to 0.3.0; add the 0.3.0 entry to CHANGELOG.md; update
installation and status prose; update the decision provenance version in
`conformance/scenarios/explanation.yaml` and `crates/canon-cli/tests/adversary_skel_cli.rs`,
keeping every other expected output; move the story to implemented and regenerate the
documentation, so its capability reads shipped. No protocol semantics change in the preparation,
so it needs no new specification or red test.

Run `task check` on the preparation, land it through a green pull request, then publish an
annotated bot tag 0.3.0 on that exact main commit and a bot-authored GitHub Release. Verify the
peeled tag, the required checks on that commit, the Release and its source archives. This is a
source release: no binary assets and no crates.io publication.

## Evidence

The wave's pull request https://github.com/beyond10x/canon/pull/14 merged as 4fa8964 with
`task check`, the documentation build, and security and privacy checks green on its head fd3ff92.
