---
format: aep.planning-md/3
id: story:argument-validator
kind: story
status: active
title: Check a call's arguments against an action's argument schema with a pure library function
tags:
- from-request
refs:
- provider: sdc
  reference: docs/canon-requests.md
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:action-arguments
scope:
- confidence: cited
  path: crates/canon/src/model/mod.rs
- confidence: cited
  path: crates/canon/src/validate/arguments.rs
- confidence: cited
  path: crates/canon/src/validate/mod.rs
- confidence: inferred
  path: crates/canon/tests/adversary_action_arguments.rs
- confidence: inferred
  path: crates/canon/tests/argument_validation.rs
- confidence: inferred
  path: crates/canon/tests/support/schemas.rs
- confidence: cited
  path: ess/domains/protocol.yaml
- confidence: cited
  path: website/docs/concepts/protocols.md
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-11-20T22:35:01Z", actor: "human:timo", revision: 12, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
- {from: "proposed", to: "active", at: "2026-11-20T22:35:01Z", actor: "human:timo", revision: 13, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
---
## Outcome

The `b10x-canon` library exports a pure function that checks one call's arguments against an action's argument schema. That schema is in Canon's argument-schema dialect.

- Function: `b10x_canon::validate::check_arguments(schema: Option<&serde_json::Value>, arguments: &serde_json::Value) -> Result<(), ArgumentsRefused>`. A runner that holds an `ir::Action` passes `action.arguments.as_ref()`. A runner that holds `canon-ir/1` as JSON passes `ir["actions"][id]["arguments"]`, which is what the sdc driver reads (`driver/src/main.rs:236,246`).
- `ArgumentsRefused::Schema(Vec<(String, String)>)`: the schema is outside the dialect. The function returns, unchanged, the (pointer inside the schema, reason) pairs that `validate::arguments::check` gives `canon validate`. It then checks no argument. This case arises only for a caller-built IR, because `ir::compile` and `eval::read_ir` accept only schemas in the dialect.
- `ArgumentsRefused::Arguments(Vec<ArgumentRefusal>)`: the arguments do not satisfy the schema. `ArgumentRefusal { pointer, keyword, reason }` gives:
  - `pointer`: the RFC 6901 pointer into the arguments document, `""` for the whole document.
  - `keyword`: the dialect keyword that refused.
  - `reason`: one line of text.
- Absent schema: the action takes only `{}` (`ir/mod.rs:169-171`). The function treats an absent schema as `{"type": "object", "additionalProperties": false}`.
- Meaning per keyword is JSON Schema 2020-12, as the dialect promises (`validate/arguments.rs:4-6`):
  - `type`: an `integer` is any number with no fractional part, so `1.0` is an integer.
  - `enum`: values are compared by JSON Schema equality, using the existing `validate::arguments::same`.
  - `required`: a missing name is refused at the object's pointer, and the reason names the name.
  - `additionalProperties: false`: an undeclared member is refused at the member's pointer.
  - `properties` and `items` recurse.
  - `minItems`, `maxItems`: counted on the array.
  - `minLength`, `maxLength`: count Unicode code points.
  - `minimum`, `maximum`: compared exactly. A JSON number that is not an integer is never compared through the bound rounded to f64.
  - `title` and `description` never refuse.
- When `type` refuses at a location, the other keywords of that schema are not checked there.
- Every refusal is reported, not only the first. Refusals come in document order, with object members in code-point order and, at one location, keywords in code-point order. This is the order rule of `check` (`validate/arguments.rs:28-32`). The same input gives the same refusals.
- The function is pure: no IO, no clock, no allocation beyond its result. It descends only into members and items that a schema describes, so its depth is bounded by `MAX_SCHEMA_DEPTH` (32, `validate/arguments.rs:40`).
- No document format, predicate form, evaluator path or command changes. `canon-ir/1`, `canon-decision/1` and the `canon` command surface are byte-identical.

## Why

- Request item 7 (`$HOME/beyond10x/sdc/docs/canon-requests.md`): "Pure argument validator exported from the Canon library".
- The reference leaves call checking to every runner (`protocol.md:92` at `8f1f452`). The sdc driver checks only top-level `required` and `additionalProperties: false` (`$HOME/beyond10x/sdc/driver/src/main.rs:240-254`). It lists "Argument validation covers only the top level" as gap 2 (`$HOME/beyond10x/sdc/docs/driver.md:224`).
- Verified in the canon tree:
  - The dialect is defined only in `crates/canon/src/validate/arguments.rs:1-32`. The `TYPES` (`:43`), `REFUSED` (`:48-66`) and `applies_to` (`:69-78`) tables admit exactly these keywords:
    - all seven types: `type`, `title`, `description`, `enum`;
    - object: `properties`, `required`, `additionalProperties` (boolean only);
    - array: `items`, `minItems`, `maxItems`;
    - string: `minLength`, `maxLength`;
    - integer and number: `minimum`, `maximum` (integers).
  - The top level is `type: object` (`:233-235`). No reference form exists.
  - `canon validate` reports offences as `action-arguments` (`validate/mod.rs:611-621`).
  - `canon-ir/1` carries the schema as declared, with sorted keys (`ir/mod.rs:37-38,169-171,661-662`).
  - `read_ir` refuses an IR whose schema is outside the dialect (`tests/adversary_action_arguments.rs:373-390`).
  - `diff/arguments.rs:1-45` classifies schema changes but validates no document.
  - A 2020-12 meaning of the dialect already exists as the test oracle `accepts` (`tests/adversary_action_arguments.rs:619-700`). The new function must agree with it.
- Dependencies: `Cargo.lock` holds 59 packages and none is a JSON Schema validator. `crates/canon/Cargo.toml` has `serde_json` with `float_roundtrip` and without `arbitrary_precision`. A general validator crate would be a new dependency: a cost in build time, supply chain and possibly meaning, because it would admit keywords the dialect refuses. The closed dialect needs none, so this story adds none.
- Boundary (AGENTS.md § Boundary, § Rules): the dialect is Canon's and the function is pure. It names no engineering vocabulary and evaluates no call. Calling the action, choosing arguments and acting on a refusal stay the runner's.
- This reverses one line of story:action-arguments: § Scope "Out: validating an action's arguments (the runner does it)", and the § Outcome sentence "Canon never validates an action's arguments itself". The same claim is in `crates/canon/src/model/mod.rs:207-209`, `ess/domains/protocol.yaml:165-166`, `website/docs/concepts/protocols.md:135-136`, and the generated `website/docs/reference/protocol.md:92` and `website/static/schemas/protocol-1.schema.json:239`.
- The public API surface of `b10x-canon`: no test enumerates it and no page lists it. Library functions appear only:
  - in `CHANGELOG.md` at release (precedent: `b10x_canon::ir::check_openings`, `CHANGELOG.md:20`);
  - in generated pages through module docs. `canon-docs` writes the module docs of `validate/arguments.rs` into `website/docs/reference/validation.md` under "The argument-schema dialect" (`crates/canon-docs/src/pages.rs:1223-1228`).
- `tests/adversary_action_arguments.rs:425-437` requires the published `arguments` description to contain `action-arguments`. The rewritten text keeps it.
- Not verified: whether the sdc driver can depend on `b10x-canon` by Git tag; how many transitive dependencies a JSON Schema crate would add (not checked offline).

## ESS first

- Specification change, first commit: in `ess/domains/protocol.yaml`, the comment on `canon.protocol.Action.arguments` (`:164-181`) is rewritten:
  - Canon evaluates no call.
  - The library function `validate::check_arguments` checks a call's arguments against the schema with 2020-12 meaning.
  - A runner uses it or any 2020-12 validator.
  - The `# read from:` line names this story.
- No typed declaration changes. `ArgumentRefusal` and `ArgumentsRefused` are library types like `validate::Problem`, which `ess/` does not declare either. ESS declares models, documents and commands (`tests/ess_model_matches.rs:1-35`).
- In the same commit, `crates/canon/tests/argument_validation.rs` (new) checks the acceptance expectations over the actions of `fixtures/investigation/action-arguments.yaml`.
- Red on that commit: `argument_validation.rs` fails to build, because `b10x_canon::validate::check_arguments` does not exist. A new function can be red only by not compiling. The failing build is recorded.

## Scope

- In:
  - `check_arguments`, `ArgumentRefusal`, `ArgumentsRefused` and the re-export — cited
  - the module docs of `validate/arguments.rs` stating the call semantics — cited
  - the rewritten claim in model docs, the ESS comment and the concept page — cited
  - a status row — inferred
- Out:
  - a `canon` subcommand. The request asks for a library export. A command would need a declaration in `ess/domains/cli.yaml` and would edit `crates/canon-cli/src/lib.rs`, which story:protocol-floor and story:canon-mutate edit — cited
  - a general JSON Schema validator, or any keyword outside the dialect — cited
  - a new dependency — cited
  - a conformance step kind for calls. `canon-conformance/1` steps are only `compile` and `evaluate` (`conform/mod.rs:41-43`). Dialect rules are held by Rust tests, as story:action-arguments did — cited
  - JavaScript or Python bindings — cited
  - changes to the sdc driver, which are the runner's — cited
- Surfaces:
  - `crates/canon/src/validate/arguments.rs` — cited
  - `crates/canon/src/validate/mod.rs` (one `pub use`, `:67`) — cited
  - `crates/canon/src/model/mod.rs` (doc, `:207-209`) — cited
  - `ess/domains/protocol.yaml` (comment, `:164-181`) — cited
  - `website/docs/concepts/protocols.md` (`:133-136`) — cited
  - `crates/canon/tests/argument_validation.rs` (new) — inferred
  - `crates/canon/tests/support/schemas.rs` (new; the generators and `accepts`, moved from `adversary_action_arguments.rs:442-700`) — inferred
  - `crates/canon/tests/adversary_action_arguments.rs` (uses the moved module) — inferred
  - `website/status.yaml` — inferred
- At integration, regenerated, not hand-edited: `website/docs/reference/validation.md`, `website/docs/reference/protocol.md`, `website/static/schemas/protocol-1.schema.json`, `website/data/status.json`.
- Confidence: high on the function's home and semantics. Medium on the test-support move.

## Order

- Depends on story:action-arguments (implemented): the dialect, `check` and `same`.
- Shares `crates/canon/src/validate/mod.rs` and `ess/domains/protocol.yaml` with story:protocol-imports and story:revision-ordering (both active). It shares `crates/canon/src/model/mod.rs` with story:protocol-imports.
- It touches one line or comment in each shared file. It still runs in a wave without those two stories.
- It does not touch `crates/canon/src/lib.rs`, `crates/canon/src/ir/mod.rs` or `crates/canon-cli/`, so it does not overlap story:canon-mutate or story:protocol-floor.

## Acceptance

`crates/canon/tests/argument_validation.rs` passes, with these expectations:

1. With an absent schema:
   - `{}` is accepted;
   - `{"x": 1}` gives exactly one refusal, at `/x`, keyword `additionalProperties`;
   - `[]` gives exactly one, at `""`, keyword `type`.
2. Against `inspect` of `fixtures/investigation/action-arguments.yaml`:
   - `{"source": "logs"}` is accepted;
   - each of these documents gives exactly the refusals named for it:

| Arguments | Pointer | Keyword |
|---|---|---|
| `{}` | `""` | `required` |
| `{"source": ""}` | `/source` | `minLength` |
| `{"source": "a", "limit": 0}` | `/limit` | `minimum` |
| `{"source": "a", "limit": 51}` | `/limit` | `maximum` |
| `{"source": "a", "filters": ["x"]}` | `/filters/0` | `enum` |
| `{"source": "a", "y": 1}` | `/y` | `additionalProperties` |

3. `{"source": 5}` gives one refusal at `/source` with keyword `type`, and none for `minLength` or `maxLength`.
4. `limit: 3.0` is accepted and `limit: 3.5` is refused with keyword `type`. A 200-code-point `source` of 4-byte characters is accepted under `maxLength: 200`.
5. A schema with `maximum: 9007199254740993` accepts 9007199254740993 and refuses 9007199254740994.
6. A document failing at `/source` and at `/limit` lists `/limit` first. Two calls give equal results.
7. `{"type": "object", "$ref": "#"}` returns `ArgumentsRefused::Schema` with the pair `validate::validate` reports for it (`/$ref`, "`$ref` is outside the dialect"), and no argument refusal.
8. Over the fixed-seed random schemas and documents of the moved generators:
   - `check_arguments` returns `Ok` exactly when `accepts` returns true;
   - more than 1000 schemas are compared.

## Source

- Request item 7 of `$HOME/beyond10x/sdc/docs/canon-requests.md`. Evidence:
  - `$HOME/beyond10x/sdc/driver/src/main.rs:236-254`
  - `$HOME/beyond10x/sdc/docs/driver.md:217-224`
  - `website/docs/reference/protocol.md:92` at `8f1f452`
- Read in canon at `15cfece`:
  - `AGENTS.md`
  - `crates/canon/src/validate/arguments.rs`, `crates/canon/src/validate/mod.rs`
  - `crates/canon/src/diff/arguments.rs`
  - `crates/canon/src/ir/mod.rs`
  - `crates/canon/src/eval/read.rs`
  - `crates/canon/src/model/mod.rs:200-212`
  - `crates/canon/tests/adversary_action_arguments.rs`, `crates/canon/tests/ess_model_matches.rs`
  - `crates/canon/src/conform/mod.rs:1-50`
  - `crates/canon-docs/src/pages.rs:1-40,1205-1235`
  - `ess/domains/protocol.yaml:155-195`, `ess/domains/conform.yaml`
  - `Cargo.toml`, `crates/canon/Cargo.toml`, `Cargo.lock`
  - `website/status.yaml`, `website/docs/concepts/protocols.md`
  - `docs/plan/wave-plan-2026-11-20.md:120-160`
  - stories action-arguments, protocol-floor, protocol-imports, revision-ordering, canon-mutate, capability-scope
- Date: 2026-11-20.

## Decided in review, 2026-11-21

Adversary pass 1 (`review-result:adversary-argument-validator-pass-1`) was red: 9 red cases, 1 blocker, 3 warnings, 3 notes. The wave coordinator decides on 2026-11-21; the session working on the SDC prototype, which orchestrates this plan, may overrule.

**D1. One exact comparison of numbers, everywhere in the argument-schema code.** The unit made the helper that decides whether two JSON numbers are the same value exact, where it compared through a double before. Two outputs changed for numbers above 2^53: `canon validate` and `canon compile` now accept an `enum` such as `[9007199254740993, 9007199254740992.0]`, which they refused as listing one value twice, and `canon diff` now reports a change of `enum` from `[9007199254740992.0]` to `[9007199254740993]` as `BREAKING`, where it printed `NO SEMANTIC CHANGE`. The new outputs are the ones JSON Schema 2020-12 gives; the old ones were a defect of the comparison through a double. The exact comparison stays. No fixture in the repository has such an enum, so no existing protocol's output changes. The adversary's two cases that pin the old outputs are re-pinned to the new ones in its second pass.

**D2. The comparison is exact however `serde_json` is built.** `crates/canon-expr` turns on `serde_json`'s `arbitrary_precision`, and Cargo unifies features across a workspace build, so the `canon` binary and `cargo test --workspace` hold a number as its decimal text, while `cargo test -p b10x-canon` holds it as an integer or a double. The function must give the right answer in both builds: a number is compared by its exact decimal value when the text is held, a number with no finite double (`1e400`) is above every finite maximum and below none, and no comparison goes through a double that cannot hold the value. The module documentation states which value is compared in each build. The unit's tests run green in both builds, and the report quotes both runs.

**D3. The sentence "no allocation beyond its result" of § Outcome is withdrawn.** The function allocates a pointer buffer, a sorted list of member names and the escaped tokens. That is not a defect; the sentence was the draft's.
