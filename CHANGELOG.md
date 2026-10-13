# Changelog

## Unreleased

## [0.3.0] - 2026-10-13

Predicates over evidence order. Install the CLI or depend on the libraries from Git tag `0.3.0`.

- Add two predicate forms to `protocol/1`: `first: {kind, result}` holds when the first record of a
  kind has that result, and `before: {first, then}` holds when one kind's first record was observed
  strictly before another's; each may name a `subject`. Records are ordered on `canon-evidence/1`'s
  existing `observed_at`; no evidence field is added.
- Order forms read the case's evidence history, including records that revision binding,
  freshness or invalidation keep from evidence matches, so a failing result recorded at an earlier
  revision counts after a fix moves the subject to a later one. No records, a record without
  `observed_at`, a tie at the earliest instant or conflicting earliest results leave a form
  `unknown`; `before` is `false` once the later kind has a record and the earlier has none.
- Validation refuses an order form naming an undeclared kind or subject, and a `before` naming one
  kind twice (`self-ordered-evidence`). Action reasons and explanations name the forms; `canon diff`
  classifies a changed order form as `BREAKING`.
- `canon check` gains history and ordering dimensions for protocols that use order forms;
  protocols without them check as before. Conformance scenario `CANON-ORDER-001`.
- Known limit: a record on any earlier revision counts, whether or not the current revision
  descends from it.

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
