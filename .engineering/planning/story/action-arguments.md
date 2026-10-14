---
format: aep.planning-md/3
id: story:action-arguments
kind: story
status: active
title: Declare one argument schema per action
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- depends_on: story:evidence-independence
- depends_on: story:semantic-diff
scope:
- confidence: cited
  path: crates/canon-docs/
- confidence: cited
  path: crates/canon/src/diff/
- confidence: cited
  path: crates/canon/src/ir/
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/validate/
- confidence: cited
  path: crates/canon/tests/action_arguments.rs
- confidence: cited
  path: ess/
- confidence: cited
  path: fixtures/investigation/action-arguments.yaml
- confidence: cited
  path: website/
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-10-14T21:46:07Z", actor: "human:timo", revision: 11}
- {from: "proposed", to: "active", at: "2026-10-14T21:46:07Z", actor: "human:timo", revision: 12}
---
## Outcome

A protocol's definition declares one argument schema per action, and a runner reads it from the
protocol it loads. Each `actions:` entry of `protocol/1` takes an optional `arguments`, beside
`effect` and `requires`: a JSON Schema in Canon's argument-schema dialect. `canon validate` refuses
any schema outside the dialect, naming the keyword and its path; `canon compile` carries the schema
unchanged into `canon-ir/1`; `canon diff` classifies a changed schema as narrowing, widening or
changed. Absent, the action takes no arguments: a runner accepts only the empty object `{}` and refuses any other.
Canon never validates an action's arguments itself: it evaluates no call; the runner that calls the
action validates them against the schema.

## The dialect (settled 2026-10-14)

A subset of JSON Schema 2020-12. Every schema in it is a valid 2020-12 schema with the same meaning,
so a runner can hand it to any 2020-12 validator. Protocols load offline, so the dialect has no
reference at all.

- Every schema, the top level and each nested one, names exactly one `type`: `object`, `array`,
  `string`, `integer`, `number`, `boolean` or `null`. The top level is `type: object`.
- Annotations, anywhere: `title`, `description` (strings).
- `enum`, on any type: a non-empty list of distinct JSON scalars, each of the schema's type.
- `object`: `properties` (a map of name to schema), `required` (a list of distinct names, each
  declared in `properties`), `additionalProperties` (a boolean only).
- `array`: `items` (one schema), `minItems`, `maxItems`.
- `string`: `minLength`, `maxLength`.
- `integer` and `number`: `minimum`, `maximum`, integers.
- Every bound is a non-negative integer where JSON Schema requires it, and a minimum is at most its
  maximum.
- Anything else is refused as outside the dialect: `$ref`, `$defs`, `$schema`, `$id`, `allOf`,
  `anyOf`, `oneOf`, `not`, `if`/`then`/`else`, `patternProperties`, `pattern`, `format`, `const`,
  `exclusiveMinimum`, `exclusiveMaximum`, a list-valued `type`, a schema-valued
  `additionalProperties`, a keyword on a type it does not apply to, and any unknown keyword.
- Nesting is bounded by the same depth bound as predicates.

Validation problem code: `action-arguments`, one problem per offending keyword, its message naming
the action, the JSON Pointer of the keyword inside the schema and why it is refused.

## Versioning

Every change to an action's `arguments` is made under a new `protocol.revision`, like every other
change to a protocol. `canon diff` tells a runner what the change does to calls:

| Change | Category | Line |
|---|---|---|
| `arguments` introduced on an action (it took only `{}`) | `BREAKING` | `arguments introduced` |
| `arguments` removed from an action (it now takes only `{}`) | `BREAKING` | `arguments removed` |
| only `title` or `description` changed, anywhere in the schema | `NO SEMANTIC CHANGE` | `arguments rewritten` |
| the new schema accepts only argument documents the old one accepted | `TIGHTENING` | `arguments narrowed` |
| the old schema accepts only argument documents the new one accepts | `RELAXATION` | `arguments widened` |
| anything else, or both | `BREAKING` | `arguments changed` |

Narrowing and widening are decided structurally and soundly, never guessed. Per keyword: `integer`
to `number` widens and the reverse narrows, any other type change is changed; an `enum` introduced
or a value removed narrows, removed or a value added widens; a lower bound introduced or raised, or
an upper bound introduced or lowered, narrows, and the reverse widens; a name added to `required`
narrows, removed widens. For an object, each property name is compared by its effective schema: the
declared one, else "any value" when `additionalProperties` is absent or true and "no value" when it
is false; `items` and each property recurse. The directions of all keywords are combined: none is
`rewritten`, only narrowing is `narrowed`, only widening is `widened`, both is `changed`.

## ESS first

- Specification change, first commit: in `ess/domains/protocol.yaml`, the field `arguments` of type
  `Optional<Json>` on `canon.protocol.Action`; the fixture
  `fixtures/investigation/action-arguments.yaml` (a protocol whose actions carry schemas in the
  dialect) and the refusal fixtures under `crates/canon/tests/`.
- Red on that commit: `crates/canon/tests/ess_model_matches.rs` fails naming `arguments`; then
  `crates/canon/tests/action_arguments.rs` fails on the dialect refusals, the IR carry and the diff
  lines.

## Scope

- In: `arguments` in model, validator, IR and `canon-ir/1` JSON; the diff classification and its
  module table; the generated reference pages and JSON Schemas; the concept page on protocols
  (the dialect and the versioning table); a status row.
- Out: validating an action's arguments (the runner does it); references between schemas or to
  documents outside the protocol; the dialect keywords listed as refused, each a later story if a
  consumer needs it.
- Surfaces: `ess/`, `crates/canon/src/model/mod.rs`, `crates/canon/src/validate/`,
  `crates/canon/src/ir/`, `crates/canon/src/diff/` (new `diff/arguments.rs`),
  `crates/canon-docs/`, `website/docs/`, `website/static/schemas/`, `website/data/`,
  `website/status.yaml`, `fixtures/investigation/action-arguments.yaml`,
  `crates/canon/tests/action_arguments.rs`.

## Order

Changes `protocol/1`: shares `ess/`, `model/`, `validate/` and `ir/` with every such story and runs
one per wave. Inside the wave two units: the field (spec, model, validator, IR, docs) and the diff
classifier (`diff/arguments.rs`, a pure function over two optional JSON values), which have disjoint
files until the classifier is wired into `diff/mod.rs` at integration.

## Acceptance

`canon validate` accepts `fixtures/investigation/action-arguments.yaml` and refuses, each with an
`action-arguments` problem naming the pointer, a top-level schema that is not `type: object`, a
`$ref` (local or remote), an `anyOf`, a schema without `type`, a `required` name not in
`properties` and a `minimum` above its `maximum`. `canon compile` of the fixture writes each action's
`arguments` byte-for-byte as declared (as JSON) and none for an action without one.
`crates/canon/tests/ess_model_matches.rs` passes. `canon diff` of the fixture against a revision
with one required property added reports `TIGHTENING` `arguments narrowed`, with `additionalProperties`
false turned true reports `RELAXATION` `arguments widened`, with both reports `BREAKING`
`arguments changed`, and with only a description changed reports `NO SEMANTIC CHANGE`.

## Source

A runner's request (2026-10-14): a protocol's definition declares one argument schema per action,
and the runner reads it from the protocol it loads; `crates/canon/src/model/mod.rs:184-197`,
`crates/canon/src/ir/mod.rs:129-139`; design § 10 (actions), § 31 (semantic diff); JSON Schema
2020-12.
