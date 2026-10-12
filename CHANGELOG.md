# Changelog

## Unreleased

## [0.2.0] - 2026-10-12

Case inputs and semantic diff. Install the CLI or depend on the libraries from Git tag `0.2.0`.

- Add `canon diff --from --to`: compare two protocol revisions on their compiled `canon-ir/1` and
  classify every change as `TIGHTENING`, `RELAXATION`, `BREAKING`, `EXPANSION` or
  `NO SEMANTIC CHANGE`, with the identifiers it touches. Changed predicates are ordered by sound
  structural implication, and a changed claim by the direction each reader reads it.
- Add case inputs to `protocol/1`: `case: inputs:` declares closed value lists, the `input`
  predicate leaf tests a case's value, and an obligation's `applicable_when` makes it
  `not_applicable` when FALSE, so one compiled protocol serves several profiles. `canon-case/1`
  gains `inputs`; a case missing a declared input, naming an undeclared one or giving an undeclared
  value is refused. Conformance scenario `CANON-INPUT-001`.
- `canon check` enumerates input values as a state dimension and names them in its summary.
- Existing protocols without `case` or `applicable_when` evaluate as before.

## [0.1.0] - 2026-10-05

The first source release. Install the CLI or depend on the libraries from Git tag `0.1.0`.

- Include the `b10x-canon` protocol model, strict validation, canonical compilation, three-valued
  evidence evaluation, authority and outcome decisions, invalidation and structured explanations.
- Include the `canon` CLI for validation, compilation, evaluation, finite-state checks, generated
  conformance scenarios and conformance execution.
- Add the `b10x-canon-expr` library: typed catalog expressions, Logos lexing, Pratt parsing,
  hygienic recipes, exact numeric comparisons, deduplicated acquisition plans and pure
  three-valued evaluation with retained-context replay.
- Specify expression wire contracts in ESS and enforce model parity. Raise the ESS gate and
  verified CI installation to 0.53.0.
- Add an assertion concept guide and generated contract/limit reference. Existing protocol
  evaluation and command-line behavior are unchanged.
