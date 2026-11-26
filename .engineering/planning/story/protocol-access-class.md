---
format: aep.planning-md/3
id: story:protocol-access-class
kind: story
status: active
title: A protocol declares its access class and whether it is guarded
relations:
- decomposes: epic:trigger-rules
- serves: vision:governed-autonomy
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-26T11:25:20Z", actor: "human:operator", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-26T11:25:20Z", actor: "human:operator", revision: 3}
---
## Outcome

A protocol declares what its actions may change, and `canon-ir/1` carries the protocol's derived
access class (`read_only` or `write`) and whether it is guarded; `canon trigger protocols <files>`
prints the `{class, guarded, inputs}` map that `evaluate` reads.

## Why

Request open question 2 and acceptance 7: a rule's `access` flag must be checked against the
protocol, and a guarded protocol must be refused in `auto`. Today `protocol/1` has no access class;
`effect` is a free name.

## Scope

- **In:** an optional `access: read_only | write` on an action, absent meaning `write`;
  `canon validate` refuses `read_only` on an action with `revises` or `case_open`; the protocol class
  is `read_only` when every action is; `guarded` when some action has a non-empty `requires`; both
  in `canon-ir/1`; `canon trigger protocols`; regenerated reference pages.
- **Out:** any other change to `protocol/1`; any change to evaluation of a case.
- **Surfaces:** `crates/canon/src/model/mod.rs` (Action), `crates/canon/src/validate/`,
  `crates/canon/src/ir/`, `crates/canon-cli/src/lib.rs`, `ess/domains/protocol.yaml`,
  `ess/domains/cli.yaml`, `conformance/`, `website/docs/reference/`.

## Acceptance

A protocol whose actions are all `read_only` compiles to class `read_only`; one action without
`access` makes it `write`; a `read_only` action with `revises` fails `canon validate` with
`read-only-action-writes`; a protocol with one action that `requires` a capability is `guarded`.
Existing protocols compile to the same `canon-ir/1` bytes apart from the two new fields.

## ESS first

First commit: `ess/domains/protocol.yaml` adds the action field and the IR fields; `cli.yaml`
declares `trigger-protocols`; a conformance scenario per acceptance line; `ess_model_matches` fails
(red recorded).

## Source

- **Request:** trigger-rule kernel request of 2026-11-26, open question 2, acceptance 7.
- **Design:** `docs/design/trigger-rules.md` § 5.
