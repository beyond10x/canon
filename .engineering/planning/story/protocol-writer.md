---
format: aep.planning-md/3
id: story:protocol-writer
kind: story
status: implemented
title: Write a protocol/1 model out as canonical JSON text with a pure library function
tags:
- from-request
refs:
- provider: sdc
  reference: docs/canon-requests.md
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon/src/model/mod.rs
- confidence: inferred
  path: crates/canon/src/model/write.rs
- confidence: cited
  path: crates/canon/tests/adversary_protocol_write.rs
- confidence: inferred
  path: crates/canon/tests/protocol_write.rs
- confidence: cited
  path: ess/domains/protocol.yaml
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-11-20T23:28:16Z", actor: "human:timo", revision: 7, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
- {from: "proposed", to: "active", at: "2026-11-20T23:28:16Z", actor: "human:timo", revision: 8, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
- {from: "active", to: "implemented", at: "2026-11-21T16:01:29Z", actor: "human:timo", revision: 14, decided_on: {"recorded":{"test_result":1,"review_outcome":6}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w7"}
---
## Outcome

The `b10x-canon` library writes a `protocol/1` model back out as text. `parse` reads that text back to an equal model.

- Function: `b10x_canon::model::write(protocol: &Protocol) -> String`, beside `model::parse` (`model/mod.rs:57`). It is total, pure and infallible. It writes any model value, including one the validator refuses (a repeated identifier, a dangling reference). `canon validate` stays the only judge of validity.
- Output: canonical JSON, which `parse` reads as YAML (`model/parse.rs:25-29`).
  - Declaration sections keep model order, repeated identifiers included (`Declarations`, `model/mod.rs:284-288`).
  - Inside a declaration body and at the top level, keys follow the order the Rust struct declares its fields.
  - An absent `Option` and an empty defaulted section are left out. `Some` of an empty value is written (`case: {}`, `independent_on: []`), because the model tells it apart from `None` (`mod.rs:93`, `predicate.rs:244`).
  - A claim test is always written with `is`. An outcome's `decision` is written as `{"decision": <name>}`.
  - Indent is two spaces and the text ends with one newline.
  - Strings escape `"`, `\` and every character below U+0020. They also escape U+007F–U+009F, U+2028, U+2029, U+FEFF, U+FFFE and U+FFFF as `\uXXXX`, the characters YAML cannot hold as written (`generate/mod.rs:52-56`).
- Guarantees, each held by a test:
  - `parse(&write(&p)) == Ok(p)` for every `p` that `parse` gives.
  - `write(&parse(&write(&p))?) == write(&p)`: the second write is a fixpoint.
  - The same model gives the same bytes.
- What `write(parse(text))` does not keep:
  - comments, YAML style, anchors and aliases (aliases come out expanded);
  - key order inside bodies and section order;
  - how a scalar was spelled: `values: [true, 2]` comes out as `"true"` and `"2"` (`InputValue`, `ids.rs:214-219`), and numbers in `arguments` come out as `serde_json` spells them;
  - claim-test sugar;
  - a default key that was written out (`requires: []` is not written).
- What it keeps: every model value, declaration order and repeated identifiers.
- The model gains no `Serialize` derive. The writer is hand-written, as `canon-ir/1`'s is (`ir/json.rs:1-8`). Inside it, every model struct is destructured with no `..`, as `diff/mod.rs:953-972` does. A field added to the model then stops the writer compiling.
- Unchanged: `canon-ir/1` bytes, every evaluation document, the `canon` command surface and `ess/` typed declarations.

## Why

- Request 9: "A protocol builder or writer: the `protocol/1` model can be read and not written out, so a program cannot emit a protocol. Needed by any SDK and binding."
  - `$HOME/beyond10x/sdc/docs/canon-requests.md:48-58` (SDK section): Rust core, bindings later, "a builder that emits `protocol/1` so a program does not write YAML".
  - `$HOME/beyond10x/sdc/docs/loom-sdk-and-agent-node.md:34-41,67,92-93`: a Rust host can build a `Protocol` in memory but cannot write it out. That file is unverified research from another session.
- Verified in the canon tree at `557e75b`:
  - `Protocol` derives `Debug, Clone, PartialEq, Eq, Deserialize` only (`model/mod.rs:68`). So does every protocol body type (`mod.rs:102-271`, `predicate.rs:111-262`, `requirement.rs:25`).
  - Only the identifiers derive `Serialize` (`ids.rs:13,36`).
  - Every field is `pub` (`mod.rs:70-98`). `Declarations::new` is public (`:299`), and its entries are private (`:286-288`).
  - `Protocol` has no `Default`, so a program must spell every section.
  - `parse` is `serde_yaml_ng::from_str` (`parse.rs:25-29`).
  - `Predicate` and `OutcomeRequirement` are read through `try_from` written forms (`predicate.rs:112`, `requirement.rs:26`). A derived `Serialize` would write Rust variant tags.
  - A derived `Serialize` would also write `None` as `null`, which every optional key refuses on read (`present.rs:13-14,35-42`). This is why the writer is hand-written.
  - `canon-ir/1` is written by a hand-written renderer (`ir/json.rs:77-139`, `Ir::canonical_json`, `ir/mod.rs:545-548`).
  - That renderer cannot be reused as is, for two reasons:
    - Its objects are `BTreeMap`s (`json.rs:24`), which would sort declarations and drop repeated identifiers.
    - It writes U+007F, U+2028 and the other characters listed above as themselves (`json.rs:122-139`). Changing that would change `canon-ir/1` bytes.
  - The YAML-safe escape set already exists in `generate::quote` (`generate/mod.rs:497-515`).
  - Programs already build a `Protocol` in Rust:
    - `crates/canon/tests/adversary2_ir_order.rs:347-380`;
    - `crates/canon/src/eval/read.rs:207-213`, which rebuilds one from `canon-ir/1` and is held by a byte comparison (`:98-100`).
  - A search of `crates/` and `Cargo.toml` for `napi`, `pyo3`, `wasm-bindgen`, `uniffi` and `cdylib` finds nothing.
  - `website/static/schemas/protocol-1.schema.json` is generated from the model (`website/docs/reference/protocol.md:8`).
  - `serde_json` has no `preserve_order` in any workspace manifest. `canon-expr` enables `arbitrary_precision` (`crates/canon-expr/Cargo.toml:14`).
  - There are 71 files under `fixtures/` that start with `format: protocol/1`.
  - Design: Canon "should remain usable as a pure library" (`canon-protocol-calculus-design.md:1235`).
- Boundary: the model is Canon's, so its writer is Canon's.
  - Bindings (napi, pyo3, wasm) are out of scope.
  - Loom's re-export and Loom's workflow layer are Loom's (`loom-sdk-and-agent-node.md:92-98`).
- Not verified:
  - whether `parse` accepts JSON protocol text: no JSON protocol fixture or test exists;
  - serde_yaml_ng's nesting limit;
  - whether serde_yaml_ng reads `serde_json` float spellings such as `1e300` back as numbers;
  - whether repeated keys in a flow mapping reach `Declarations` as they do from block YAML.

## Decision needed

| Option | `ess/` and model | Cost |
|---|---|---|
| A. Canonical JSON writer | No typed change; one comment | A second small renderer with ordered objects and the wider escape set. No YAML layout rules. Every string is quoted, so YAML's implicit typing cannot apply. It is what `protocol-1.schema.json` describes. |
| B. YAML block writer | No typed change; one comment | Indentation, empty collections and block-or-flow choices are more rules to get wrong. Quoting reuses `generate::quote`. The output looks like the fixtures. |
| C. A or B plus a `ProtocolBuilder` | No typed change; the builder is a library type | A second public API that every story adding a model field must extend, designed with no consumer naming its shape. A binding at the document boundary (`loom-sdk-and-agent-node.md:94-98`) does not need it. |

Recommendation: A, the canonical JSON writer only, with a builder left for a story whose consumer (Loom or a binding) names its API.

## ESS first

- First commit, `ess/domains/protocol.yaml:2-3`: the header comment "parsing that syntax … is tested by Canon itself" also names writing it (`model::write`) and the round trip. The comment names this story. No typed declaration changes: a library function is not a model, a document or a command (`crates/canon/tests/ess_model_matches.rs:1-35`; precedent story:argument-validator).
- Same commit: `crates/canon/tests/protocol_write.rs` (new), with the acceptance expectations.
- Red on that commit: `protocol_write.rs` fails to build, because `b10x_canon::model::write` does not exist. A new function can be red only by not compiling. The failing build is recorded.

## Scope

- In:
  - `model::write` and its module docs (escape set, key order, round-trip contract) — cited
  - the exhaustive destructuring — cited
  - the ESS comment — cited
  - a status row — inferred
- Out:
  - a `canon fmt` subcommand. It would need a declaration in `ess/domains/cli.yaml` and would edit `crates/canon-cli/src/lib.rs` and `main.rs`, which story:protocol-imports, story:protocol-floor and story:canon-mutate edit. It would also delete the comments of hand-written protocols. — cited
  - a builder (§ Decision needed) — cited
  - bindings (napi, pyo3, wasm) and Loom's re-export — cited
  - comment-preserving or style-preserving rewrites — cited
  - any change to `ir/json.rs` or `canon-ir/1` bytes — cited
  - a `Serialize` derive on the model — cited
  - writers for `canon-case/1`, `canon-evidence/1` and `canon-authority/1` — inferred
- Surfaces:
  - `crates/canon/src/model/write.rs` (new) — inferred (name)
  - `crates/canon/src/model/mod.rs` (one `mod` line beside `:26`, one `pub use` beside `:57`) — cited
  - `ess/domains/protocol.yaml` (comment, `:2-3`) — cited
  - `crates/canon/tests/protocol_write.rs` (new) — inferred
  - `website/status.yaml` — inferred
- Regenerated at integration only if `canon-docs` reads the changed module docs: `website/docs/reference/protocol.md` — inferred.
- Confidence: high on where the writer goes and why it is hand-written. Medium on JSON being read back through serde_yaml_ng for every edge case.

## Order

- Depends on nothing. Every section and form it writes is in the tree.
- Shares `crates/canon/src/model/mod.rs` with story:protocol-imports (adds `imports` and `Import`), story:supplied-evidence (`supplied_by` on `EvidenceKind`) and story:argument-validator (a doc line). It shares `ess/domains/protocol.yaml` with every active story.
- story:child-outcome-revision adds `revision: current` on `children` (`child-outcome-revision.md:239`). story:revision-ordering may add to `predicate.rs`.
- Whichever lands second adds the new field's line to `model/write.rs`. The build fails until it does: the destructuring stops it compiling, and expectation 7 fails at test time.
- story:authority-revision-binding edits `model/authority.rs` (`canon-authority/1`), not `model/mod.rs` or `protocol/1`, so this writer does not cover it.
- story:canon-mutate hands "the mutant's text" to the runner (`canon-mutate.md:61`) and does not say how it writes it. It may use this writer (inferred). There is no edge between the two stories.

## Acceptance

`crates/canon/tests/protocol_write.rs` passes, with these expectations:

1. For every file under `fixtures/` that starts with `format: protocol/1` and that `parse` accepts (walked as `missing_keys.rs:215-235` walks them): `parse(&write(&p)) == Ok(p)`.
2. For each of those, `write(&parse(&write(&p)).unwrap())` equals `write(&p)` byte for byte. Two calls of `write` give the same bytes.
3. The output is JSON that `serde_json` accepts and ends with one newline. Claims declared `b` then `a` are written `b` first.
4. Claims holding `a` twice are written twice and read back twice. `validate` then reports `duplicate-identifier` exactly as it does for the YAML source.
5. A description holding U+0000, a tab, U+007F, U+0085, U+2028, U+2029 and U+FEFF reads back equal. The written text contains none of those characters raw.
6. `case: Some(CaseSection::default())` and `case: None` read back distinct. So do `independent_on: Some(vec![])` and `None`.
7. `writer_covers_every_specified_field` takes every field of every `canon.protocol` type that `Protocol` reaches in the authored `ess/domains/protocol.yaml`, with the header written in place as in `ess_model_matches.rs:63-74`. Each must appear as a written key in the output for at least one corpus protocol. The test fails naming any field that none does.

## Source

Request 9 (prompt), `$HOME/beyond10x/sdc/docs/canon-requests.md:48-58`, `$HOME/beyond10x/sdc/docs/loom-sdk-and-agent-node.md` (read in full). Canon tree `$HOME/.local/state/worktree/trees/b10x/canon/canon-wave-2026-11-20-w1` at `557e75b`: `AGENTS.md`, `crates/canon/src/model/{mod,parse,predicate,requirement,ids,present,authority,decision}.rs`, `ir/{mod,json}.rs`, `eval/read.rs`, `generate/mod.rs`, `diff/mod.rs`, `tests/{ess_model_matches,missing_keys,adversary2_ir_order}.rs`, `ess/domains/{protocol,cli}.yaml`, and the stories capability-scope, protocol-floor, argument-validator, canon-mutate, protocol-imports, supplied-evidence, revision-ordering, child-outcome-revision and authority-revision-binding. 2026-11-20.

## Decided

Option A of § Decision needed: the canonical JSON writer `model::write` only. A builder waits for a story whose consumer names its API. Decided on 2026-11-21 by the wave coordinator on the drafter's recommendation; the operator accepted the story without choosing (`approval-record:accept-sdc-requests-9-11-2026-11-21`). The unit builds this option and does not reopen the choice.

## Decided in review, after adversary pass 1

The adversary pass (`review-result:adversary-protocol-writer-pass-1`) was red: four findings, all
introduced by the unit, six red cases and four green cases that kill mutants the unit's suite let
through. Decided on 2026-11-21 by the wave coordinator; the session the operator named
orchestrator of this plan may overrule it.

- **D1, the round trip holds for keys of at most 1022 bytes as written.** The YAML reader behind
  `parse` drops a simple key whose text runs more than 1024 bytes before its `:`, the two quotes
  included. `parse` itself gives longer keys: a plain key of 1023 or 1024 bytes, an explicit `?`
  key of any length, and a key whose escapes grow it past the limit when written. JSON has one
  spelling for a key, so no JSON text reads such a key back through `parse`. The three ways out
  were: narrow the guarantee, have `parse` refuse such keys, or write those keys in a YAML form
  that is not JSON. The second changes which `protocol/1` documents `parse` accepts, one day
  after 0.10.0; the third gives up § Decided option A. The guarantee is narrowed. § Outcome reads
  from now on: `parse(&write(&p)) == Ok(p)` for every `p` that `parse` gives in which every
  key, as written with its escapes, is at most 1022 bytes; this holds for identifiers and for
  property names inside `arguments`. For a longer key `write` still returns JSON that
  `serde_json` reads with the key intact, and `parse` refuses that text. The module docs state
  the limit in bytes of written text, not in characters, and no longer say `parse` gives no such
  key. Tests pin both sides of the limit (1022 reads back, 1023 does not) for a plain key, a key
  grown by escapes and a property name inside `arguments`, and that the text for a longer key is
  JSON holding the key.
- **D2, `write` recurses once per nesting level, and the docs say so.** A predicate or an
  `arguments` value built in Rust and nested 5000 deep overflows a 2 MiB stack inside `write`.
  `parse` gives at most 123 levels of predicate and 125 of `arguments`, so no parsed model
  reaches it. The writer stays recursive: an explicit stack would change every function of a
  769-line file for a model no reader of protocol text can produce. § Outcome's "total" reads from
  now on: `write` returns for every model whose nesting the calling thread's stack holds; it
  uses stack in proportion to the nesting depth. The module docs drop the sentence that a model
  nested deeper than `parse` reads "is still written" and state this instead. A test holds a
  model built in Rust at 1000 levels on a thread with a 2 MiB stack.
- **D3, the adversary's cases.** The four green cases stay as they are. The six red cases are
  rewritten by the correction to pin D1 and D2: each keeps its input and asserts the behaviour
  decided above. The case at 5000 levels becomes the case at 1000 levels.

- **D4, the depth test holds 500 levels, not 1000.** Added on 2026-11-21 after correction 1
  (`94150485`) measured the stack: in a debug build on the workstation's compiler a predicate
  1150 deep is written on a 2 MiB stack and one 1200 deep overflows it. 1000 levels leave about
  15 percent, and a stack overflow aborts the whole test binary; the compiler in continuous
  integration is a release newer and its frames may be larger. The test and the module docs name
  500 levels, which leaves a factor of two. D2 is otherwise unchanged.

## Decided in review, after adversary pass 2

The second and last adversary pass (`review-result:adversary-protocol-writer-pass-2`) was green:
the four findings of pass 1 are closed, the key limit is 1022 bytes in 135 combinations of place
and key form, and `parse` refuses every longer key it was given. It returned one warning and one
note, both introduced by the unit, and three green cases (`26b7de25`). Decided on 2026-11-21 by
the wave coordinator; the session the operator named orchestrator of this plan may overrule it.

- **D5, the depth figure is 400 levels, for every predicate shape.** D4 took 500 levels from a
  measurement of `not` alone. Pass 2 measured each shape on a 2 MiB stack in a debug build: `not`
  holds 1152 levels, `all` and `any` hold 808, an `arguments` array 2209 and an `arguments`
  object 1945. `all` and `any` take two calls per level, so 500 levels leave them a factor of
  1.62. The module docs and both depth cases (`not`, and the `all` case pass 2 added) name 400
  levels, which leaves a factor of two for the deepest-running shape. The docs sentence "recurses
  once per level" is corrected to say that `all` and `any` take two calls per level. This is the
  correction after the last pass; the coordinator reads its diff.
- **D6, the note.** The two mutants the unit's cases let through (a backspace written in six
  bytes, more stack in the list of predicates) are killed by the pass's cases, which stay.

## Scope as built

Written by the wave coordinator at the close of wave 2026-11-20-w7 from the implementor's reports, the two adversary reports (`review-result:adversary-protocol-writer-pass-1`, red; `review-result:adversary-protocol-writer-pass-2`, green) and the merge of `impl/protocol-writer` (`aa53d6ae`). It corrects the body where the two differ.

- `b10x_canon::model::write(&Protocol) -> String` is in the new `crates/canon/src/model/write.rs`; `crates/canon/src/model/mod.rs` gains `mod write;`, `pub use write::write;` and one doc sentence. No `Serialize` derive, no change under `ir/`, no change to the command line.
- The round trip `parse(&write(&p)) == Ok(p)` holds for every `p` that `parse` gives in which every key is at most 1022 bytes as written with its escapes (Decided in review, D1). § Outcome said it for every `p` that `parse` gives. For a longer key `write` returns JSON that `serde_json` reads with the key intact, and `parse` refuses that text. Pass 2 measured 1022 and 1023 in 135 combinations of 15 places and 9 key forms; indentation does not count toward the limit.
- `write` is infallible and recursive: one call per level of nesting and two per level of `all` or `any`. It returns for every model whose nesting the calling thread's stack holds (D2); the docs and two cases name 400 levels on a 2 MiB stack in a debug build (D5), where `all` and `any` hold 808 levels and `not` holds 1152. `parse` gives at most 123 levels of `not`, 61 of `all` and 125 of `arguments`.
- An integer beyond 64 bits is written as a double in the build that holds doubles; a number beyond the range of a double (`1e400`) is written as its text and reads back as a string, in any document. `arguments: Some(null)` has no `protocol/1` spelling. The module docs state the three.
- Over the corpus, 128 of 128 files read back equal and are fixpoints of `write` after `parse`, in both builds of `serde_json`.
- Every model struct is destructured with no `..`; `writer_covers_every_specified_field` reads `ess/domains/protocol.yaml` and fails naming a field of the specification no corpus protocol writes.
- The escape set equals that of `generate::quote`; the rule is copied, not shared.
- `ess/domains/protocol.yaml`: the header comment only, which names the writer and the key limit.
- Added by the unit: `crates/canon/src/model/write.rs`, `crates/canon/tests/protocol_write.rs` (34 cases). Added by the adversary: `crates/canon/tests/adversary_protocol_write.rs` (13 cases: five pin D1 on inputs `parse` gives, two pin the depth figure, six hold what a mutant passed the unit's suite on). 19 of 19 mutants of pass 1 and 3 of 3 of pass 2 are killed at the merge.
- No scenario and no requirement: the writer is a library function with no evaluation meaning.
