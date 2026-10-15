---
format: aep.planning-md/3
id: release-plan:canon-0-6-0
kind: release-plan
status: approved
title: Release Canon 0.6.0
relations:
- delivers: story:conformance-suite
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-15T04:11:47Z", actor: "human:timo", revision: 2}
---
## Intent

Release Canon 0.6.0 with the normative requirements catalogue (story:conformance-suite):
`conformance/requirements.yaml` in the new `canon-requirements/1` format, declared in `ess/`, and
`canon conform run --requirements <file>`, which checks that every implemented `CANON-*`
requirement has a passing scenario, that no scenario covers an unlisted requirement, and that each
evaluation is byte-identical on rerun and with its evidence, authority and decision lists reversed.
`task check` runs it with the flag. A new CLI flag and a new document format are a feature, so the
minor version moves.

## Wave

One wave, branch `wave/conformance-suite`, one pull request:

1. Unit conformance-suite: spec first (`ess/domains/conform.yaml`, the Rust model in
   `crates/canon/src/model/catalogue.rs`, `conformance/requirements.yaml`, fixtures, the red test
   `conform_run_checks_requirements`), then the checks in `crates/canon/src/conform/`, the flag,
   `task check`, generated reference pages and the status row.

An adversary pass runs on the unit as it turns green.

Width: 1. story:conformance-suite is the only story whose dependencies are all implemented; every
other open story is a draft, and seven of them (protocol-imports, ess-command-surface, canon-mutate,
protocol-floor, capability-scope, revision-ordering, case-composition) depend on it because they
share its surfaces or add scenarios the catalogue must list.

## Scope and procedure

Set the workspace version and lockfile to 0.6.0; add the 0.6.0 entry to CHANGELOG.md; update
installation and status prose; update the decision provenance version in
`conformance/scenarios/explanation.yaml` and `crates/canon-cli/tests/adversary_skel_cli.rs`,
keeping every other expected output; move the story to implemented and regenerate the
documentation.

Land the preparation through a green pull request, then publish an annotated bot tag 0.6.0 on that
exact main commit and a bot-authored GitHub Release. Verify the peeled tag, the required checks on
that commit, the Release and its source archives. Source release only.
