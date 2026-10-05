---
format: aep.planning-md/3
id: story:assertion-expressions
kind: story
status: implemented
title: Compile and evaluate typed catalog assertions without IO
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: .github/workflows
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: README.md
- confidence: cited
  path: crates/canon-expr
- confidence: cited
  path: crates/canon/tests/ess_gate.rs
- confidence: cited
  path: crates/canon/tests/ess_model_matches.rs
- confidence: cited
  path: docs/design
- confidence: cited
  path: ess
- confidence: cited
  path: website/docs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T20:14:42Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-05T20:14:42Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-05T20:37:50Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Intent

Implement the operator-approved `canon-expr/1` language as a generic Canon crate. Engineering
vocabulary and acquisition live in engineering-protocols. The parser and evaluator must remain
maintainable as the catalog grows.

## Acceptance

- Logos lexes bounded source with spans; a handwritten Pratt parser handles literals, variables,
  namespaced calls/facts, fields/indexing, lists/records, boolean logic and nonassociative comparisons.
- Checking resolves signatures, named/default arguments, return types and hygienic recipes;
  cycles, unknown fields, observation-dependent acquisition and non-Bool roots are refused.
- Exact i128 integer/decimal semantics include negative limits and scale boundaries; no floating
  point or implicit truthiness enters evaluation.
- Planning deduplicates requests; evaluation is pure against explicit observations and time.
  Missing/stale evidence is UNKNOWN; malformed evidence is a hard error even in masked branches.
- TRUE/FALSE/UNKNOWN truth tables, known optional absence, source/context/catalog/provider binding,
  deterministic replay and tamper-resistant plan decoding are covered by conformance tests.
- Collection, recipes, source bytes/tokens/nodes/nesting and values have bounded resource usage.
  Generated parse/format/parse and malformed-input tests exercise parser maintenance invariants.
- A new catalog recipe or typed provider function does not require parser/evaluator changes.

## ESS first

The validated typed home is `ess/domains/expr.yaml`, declared before this story. ESS0.53.0
validation, compilation and synthesis succeeded with zero refusals. Semantic acceptance is in
`crates/canon-expr/tests/`. The implementor commits specification and failing semantic tests
first, records the named red run, then implements. `docs/design/assertion-api.md` is the shared
consumer contract. Existing Canon protocol semantics remain unchanged.

## Scope

Cited: new `crates/canon-expr`, `ess/domains/expr.yaml`, ESS manifests and existing parity/gate tests,
CI ESS pin, Cargo lock, design/public documentation. Generic core only. One coherent story;
no multi-child decomposition requiring critic panel.

## Approval

Operator approved the design and instructed “Implement the plan” on 2026-10-05.
`docs/plan/assertions-wave.md` records execution, review and source-publication scope.
