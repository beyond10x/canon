---
format: aep.planning-md/3
id: release-plan:canon-0-5-0
kind: release-plan
status: approved
title: Release Canon 0.5.0
relations:
- delivers: story:action-arguments
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-14T21:46:22Z", actor: "human:timo", revision: 2}
---
## Intent

Release Canon 0.5.0 with one argument schema per action (story:action-arguments): an optional
`arguments` on each `actions:` entry of `protocol/1`, in Canon's argument-schema dialect (a subset of
JSON Schema 2020-12 with no references), refused by `canon validate` outside the dialect, carried
unchanged into `canon-ir/1` and classified by `canon diff`. A new `protocol/1` field is a feature, so
the minor version moves.

## Wave

One wave, branch `wave/action-arguments`, one pull request:

1. Unit field: spec first (`ess/`, fixture, red tests), then model, validator, IR, `canon-ir/1`
   JSON, generated reference pages and schemas, concept page and status row.
2. Unit diff: `crates/canon/src/diff/arguments.rs`, a pure classifier over two optional schemas,
   with its own tests; wired into `diff/mod.rs` and its module table at integration.

An adversary pass runs on each unit as it turns green, alongside the other.

Width: 2. Every other ready or proposed story that changes `protocol/1` shares `ess/`, `model/`,
`validate/` and `ir/` and runs one per wave; the requirements catalogue (story:conformance-suite) is
the next wave.

## Scope and procedure

Set the workspace version and lockfile to 0.5.0; add the 0.5.0 entry to CHANGELOG.md; update
installation and status prose; update the decision provenance version in
`conformance/scenarios/explanation.yaml` and `crates/canon-cli/tests/adversary_skel_cli.rs`,
keeping every other expected output; move the story to implemented and regenerate the
documentation.

Land the preparation through a green pull request, then publish an annotated bot tag 0.5.0 on that
exact main commit and a bot-authored GitHub Release. Verify the peeled tag, the required checks on
that commit, the Release and its source archives. Source release only.
