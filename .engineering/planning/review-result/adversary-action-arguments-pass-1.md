---
format: aep.planning-md/3
id: review-result:adversary-action-arguments-pass-1
kind: review-result
status: active
title: Wave action-arguments adversary, canon story:action-arguments, pass 1
relations:
- reviews: story:action-arguments
revision: 1
---
The adversary pass on story:action-arguments found one blocker and four lesser findings at b6657c5; the fixes and the probes, kept as regression tests, are 04b9405. Every reference form, every non-object top level and the classifier's soundness held.

```
unit: story:action-arguments, tree canon-adv-args, branch review/action-arguments-adversary, at b6657c5
verdict: NEEDS-CHANGE
cases: executed 6→13 (crates/canon/tests/adversary_action_arguments.rs), red 6
origin: introduced 5 / pre-existing 0 / undecided 0
```

## Findings

1. `crates/canon/Cargo.toml`, blocker, introduced: without serde_json's `float_roundtrip`, `read_ir` refused the IR `canon compile` printed for a number `enum` value such as `1.602176634e-19` (one ulp off on read-back, so not the canonical text), so `canon diff` and `canon evaluate` failed on a protocol `canon validate` accepted. `cargo test --workspace` hid it: it unifies canon-expr's `arbitrary_precision` into b10x-canon, which `-p canon-cli` does not.
2. `crates/canon/src/validate/arguments.rs` (`MAX_SCHEMA_DEPTH`), warning, introduced: through `properties` a schema level costs two YAML levels, and the YAML parser's recursion limit refused 64 levels (deepest that parsed: 63) before the dialect's bound of 64 could.
3. `crates/canon/src/diff/arguments.rs` (`enumeration`), note, introduced: enum values were compared by `Value` equality, so writing `1.0` for `1` was `BREAKING: arguments changed` although the validator treats them as one value.
4. `website/static/schemas/protocol-1.schema.json`, note, introduced: the published `protocol/1` schema leaves `arguments` untyped, so an editor accepts `arguments: 5`, which `canon validate` refuses.
5. `crates/canon/src/model/present.rs`, note, infeasible: an integer `enum` value wider than 128 bits is read by YAML as a double and carried as `1e+39`.

## Decisions on the findings

- 1: b10x-canon turns on `float_roundtrip`; `task check` runs the action-arguments probes with `-p b10x-canon`, the feature set the CLI ships.
- 2: the bound is 32 levels, within the YAML limit by either path.
- 3: the classifier compares enum values with the validator's equality.
- 4: kept: the specification's field is `Json` by design (the dialect is Canon's validator's to hold); the published description names `canon validate` and `action-arguments`, and a test pins it.
- 5: kept as a limit of YAML numbers; no test.

## What held

`$ref` to http, to a relative file and to `file:`, `$dynamicRef`, `$dynamicAnchor`, `$anchor`, `$vocabulary` and `$id`, top level and nested, are each refused at their pointer, and one hidden in `$defs`, a schema-valued `additionalProperties` or an `enum` value is refused once at that keyword; properties named `$ref` or `$id` are accepted. Every non-object top level is one `action-arguments` problem. `read_ir` refuses hand-edited IR with `$ref` or a top-level `type: string`, and a 2000-deep schema in IR without a crash. On 3925 fixed-seed random schema pairs, each checked on 300 documents, no narrowing was called widening, no widening narrowing, and no `rewritten` pair accepted different documents.
