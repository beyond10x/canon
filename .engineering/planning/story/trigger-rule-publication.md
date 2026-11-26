---
format: aep.planning-md/3
id: story:trigger-rule-publication
kind: story
status: draft
title: canon trigger check-set refuses a republished key and an auto rule over a guarded protocol
relations:
- decomposes: epic:trigger-rules
- serves: vision:governed-autonomy
- depends_on: story:trigger-rule-document
- depends_on: story:protocol-access-class
revision: 1
---
## Outcome

`canon trigger check-set <rule files> --published <index> --protocols <map>` checks a rule set
before it is published and fails naming the rule and the reason.

## Why

Request acceptance 2 and the publication half of 7: a published rule is immutable, and a rule in
`auto` over a guarded protocol is invalid when it is published, not only when it is evaluated.

## Scope

- **In:** the published index (`canon-trigger-index/1`: rule key to document digest, the caller's
  file); failures `key-republished` (same key, other digest), `duplicate-key` (two documents in the
  set), `guarded-auto`, `unknown-protocol`, `unknown-input` (a `start.input` name P does not
  declare), `missing-input` (a required case input of P not mapped).
- **Out:** storing the index; signing.
- **Surfaces:** `crates/canon/src/trigger/`, `crates/canon-cli/src/lib.rs`,
  `ess/domains/trigger.yaml`, `ess/domains/cli.yaml`, `conformance/`, `website/docs/reference/`.

## Acceptance

`stale@1` and `stale@2` in one set pass; an edited `stale@1` against an index holding `stale@1`
with another digest fails with `key-republished`; an `auto` rule over a guarded protocol fails with
`guarded-auto`; each other failure has a scenario.

## ESS first

First commit: `canon-trigger-index/1` in `ess/domains/trigger.yaml`, `trigger-check-set` in
`cli.yaml`, a conformance scenario per acceptance line; the command parity test fails (red
recorded).

## Source

- **Request:** trigger-rule kernel request of 2026-11-26, acceptance 2 and 7.
- **Design:** `docs/design/trigger-rules.md` § 7.
