# Changelog

## [Unreleased]

Order forms compare `observed_at` at the resolution it is written in.

- An instant (`observed_at`, `--at`, a conformance step's `at`) may carry a fraction of one to nine
  digits after the seconds: `YYYY-MM-DDTHH:MM:SS.fffZ`. An empty fraction (`12:00:00.Z`), more than
  nine digits and a comma stay refused as `invalid-instant`, whose message now names both forms:
  `… is not an instant in UTC written YYYY-MM-DDTHH:MM:SSZ or YYYY-MM-DDTHH:MM:SS.fffZ (one to nine
  fraction digits)`.
- `first`, `before` and `after` compare every instant of one evaluation at one resolution: the
  coarsest precision written among the records with an `observed_at`. Each instant is read with
  its finer digits dropped; different instants are ordered and equal ones are not, and the 0.7.0
  rules apply to those instants as they did to seconds. Evidence all in milliseconds is ordered
  within a second (`12:00:00.200Z` before `12:00:00.500Z`); one record at `12:00:05Z` makes the
  evaluation compare seconds, so the same two records tie. One resolution keeps the order a weak
  order, so `canon check`'s state space, which writes whole seconds, still reaches every value the
  evaluator gives. Over whole seconds every value is the 0.7.0 value. Requirement CANON-ORDER-003,
  scenario `conformance/scenarios/evidence-order-subsecond.yaml`.
- Expiry (`max_age`) still counts whole seconds: it reads an instant's whole seconds and ignores
  its fraction.
- `Instant::digits` gives the number of fraction digits written and `Instant::ticks(digits)` the
  instant at that resolution; `Instant::interval` gives the interval the last written digit spans,
  in nanoseconds; `Instant::seconds` reads a fractional instant as its whole seconds.
- The published `Instant` JSON Schema pattern admits the fraction:
  `^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}(\.[0-9]{1,9})?Z$`.

An order form that reads the latest record of a kind.

- Add the order form `since` to `protocol/1` predicates:
  `{since: {last: K1, then: {kind: K2, result: R}, subject?: A}}`. It tests whether a record of
  `K2` with result `R` was observed after the latest record of `K1`, about that record's subject
  and subject revision. `K1` and `K2` may be one kind. It reads the evidence history as the other
  order forms do, at the same resolution:
  - `unknown` with no record of `K1`;
  - `false` with no record of `K2` with `R`;
  - `unknown` when a compared record has no `observed_at`.

  Otherwise each latest record of `K1` gives a value from the records of `K2` with `R` at its
  subject and subject revision, other than itself:
  - `true` when one is later;
  - `false` when every one is earlier, or there is none;
  - `unknown` otherwise.

  The form is `true` or `false` when every latest record agrees, and `unknown` otherwise. A
  `since` over one kind is never `true`. Requirement CANON-ORDER-004, scenario
  `conformance/scenarios/evidence-order-since.yaml`.
- `canon-ir/1` writes it as authored,
  `{"since": {"last": …, "then": {"kind": …, "result": …}}}`, with `subject` when named. Among the
  members of an `all` or `any`, `since` sorts after `not`, and two compare by last kind, later
  kind, result and subject. The IR of a protocol without `since` is unchanged.
- An evaluation's reasons list each `since` after every `after`, `before` and `first` reason:
  `{"since": K1, "then": K2, "result": R, "value": …}`, with `"subject"` when named.
- `canon validate` resolves both kinds and the subject, and refuses `since` in a discharge or
  applicability predicate. `canon diff` compares it structurally, so a changed `since` is
  `BREAKING`.
- In `canon check`, a protocol with `since` gains:
  - a later-record dimension for each result it compares, as for `after`;
  - a last-record dimension of 3 values for each dimension it reads as `last`: none, a record at
    a superseded revision observed with the current records, or one observed after them.

  It counts `since`'s kinds for `authority-bypass` in every position. A protocol without `since`
  builds the same space, states and reports as before. A protocol whose space exceeds 65536
  states is refused as before; the `since` conformance fixture is such a protocol (126360 states).
- `canon generate` writes the last record at `r0`. The reference pages and the `protocol/1` JSON
  Schema name the form.
- Breaking for Rust consumers: `Predicate` gains the variant `Since(EvidenceSince)`, so an
  exhaustive match over it no longer compiles.

## [0.7.0] - 2026-10-29

An order form that reads a later record. Install the CLI or depend on the libraries from Git tag
`0.7.0`.

- Add the order form `after` to `protocol/1` predicates:
  `{after: {first: K1, then: {kind: K2, result: R}, subject?: A}}`, whether a record of `K2` with
  result `R` was observed strictly after the first record of `K1` (`K1` and `K2` may be one kind).
  It reads the evidence history as `first` and `before` do, records the exclusion stages keep from
  evidence matches included. `unknown` with no record of `K1`; `false` with no record of `K2` with
  `R`; otherwise `unknown` when a compared record has no `observed_at`, `true` when a record of `K2`
  with `R` is later than the first instant of `K1`, `unknown` when one is at that instant and none
  later (a record is not later than itself), `false` otherwise. Records of `K2` with another result
  take no part. Requirement CANON-ORDER-002, scenario
  `conformance/scenarios/evidence-order-after.yaml`.
- `canon-ir/1` writes it as authored, `{"after": {"first": …, "then": {"kind": …, "result": …}}}`
  with `subject` when named; among the members of an `all` or `any`, `after` sorts first, and two
  compare by first kind, later kind, result and subject. The IR of a protocol without `after` is
  unchanged.
- An evaluation's reasons list each `after` before the other order forms:
  `{"after": K1, "then": K2, "result": R, "value": …}`, with `"subject"` when named.
- `canon validate` resolves both kinds and the subject, and refuses `after` in a discharge or
  applicability predicate; `canon diff` compares it structurally like `first` and `before`.
- `canon check` gives a protocol with `after` a later-record dimension per compared result and
  dimension, places later records in the order with the first observations, and counts `after`'s
  kinds for `authority-bypass` in every position. A protocol without `after` checks as before.
- `ess/ess-inputs.yaml` and CI pin ess 0.75.0 (was 0.60.0); the ESS gate tests name the new pin.

## [0.6.0] - 2026-10-15

A normative requirements catalogue the conformance suite is checked against. Install the CLI or
depend on the libraries from Git tag `0.6.0`.

- Add the `canon-requirements/1` catalogue, `conformance/requirements.yaml`: the 15 `CANON-*`
  requirements Canon implements, each with its status (`implemented` or `planned`), statement and
  source. ESS declares it in the new domain `canon.conform`.
- `canon conform run --requirements <catalogue>` checks the registry against it after the run:
  every implemented requirement covered by a passing scenario (`uncovered: …`), no scenario
  covering a requirement the catalogue omits (`unlisted: …`), no planned requirement covered, and
  every evaluate step giving byte-identical output when run again and with its `evidence`,
  `authority` and `decisions` lists reversed (`nondeterministic: …`). The report ends with
  `requirements: <n> implemented, <n> covered, <n> planned`. Without the flag the report is
  unchanged. `task check` runs the registry with the catalogue.
- A catalogue that is refused is named by kind and position only: a wrong `format` or a key that
  is not a `CANON-` id is not quoted back.
- The specification is held to ess 0.60.0, which CI installs.

## [0.5.0] - 2026-10-15

One argument schema per action. Install the CLI or depend on the libraries from Git tag `0.5.0`.

- Each `actions:` entry of `protocol/1` takes an optional `arguments`: a JSON Schema in Canon's
  argument-schema dialect, a subset of JSON Schema 2020-12 with no references. Every schema names
  one `type`, the top level `type: object`; the dialect has `title`, `description`, `enum`,
  `properties`, `required`, a boolean `additionalProperties`, `items`, `minItems`, `maxItems`,
  `minLength`, `maxLength` and integer `minimum` and `maximum`, nested at most 32 deep. Absent, the
  action takes no arguments: a runner accepts only `{}`. Canon validates no call itself.
- `canon validate` refuses a schema outside the dialect as `action-arguments`, one problem per
  offending keyword, naming the action and the keyword's JSON Pointer.
- `canon compile` carries each schema unchanged into `canon-ir/1`, and writes no `arguments` for an
  action without one.
- `canon diff`: a schema introduced or removed is `BREAKING`; one changed only in `title` or
  `description` is `NO SEMANTIC CHANGE`; one that accepts only documents the old one accepted is
  `TIGHTENING` (`arguments narrowed`), the reverse `RELAXATION` (`arguments widened`), anything
  else `BREAKING` (`arguments changed`). Every change is made under a new `protocol.revision`.

## [0.4.0] - 2026-10-13

Evidence requirements that demand an independent producer. Install the CLI or depend on the
libraries from Git tag `0.4.0`.

- Add `independence_dimensions` to `protocol/1`: the dimension ids a protocol demands independence
  on, each with an optional description. Canon gives no dimension a meaning.
- An evidence match takes `independent_on`, a non-empty list of declared dimensions. A record
  satisfies it only when it names a `producer` (new on `canon-evidence/1`) and the case's
  `independence` map (new on `canon-case/1`, written by the trusted context) lists every demanded
  dimension for that producer. A record that fails the demand is inapplicable to that match,
  never contrary: a claim resting only on such records is `unknown`, not `false`
  (`CANON-INDEPENDENCE-001`). Explanations list it as `not_independent`.
- Validation refuses an empty or duplicated `independent_on`, and an undeclared dimension in a
  match or in the case's `independence`.
- `canon diff`: a dimension added to a match read positively is `TIGHTENING`, removed is
  `RELAXATION`; a changed demand under `not` or in a claim read in reverse is `BREAKING`; a
  dimension declaration added is `EXPANSION`, removed is `BREAKING`.
- `canon check` and `canon generate` give records producer profiles for demanded kinds, one value
  per set of demands a record class meets; protocols without demands check as before.
- The specification is held to ess 0.59.0.

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
