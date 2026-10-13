---
format: aep.planning-md/3
id: story:evidence-order-predicate
kind: story
status: active
title: 'Predicate over evidence order: first result of a kind, one kind before another'
refs:
- provider: atlas
  reference: adr:0084
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:three-valued-claims
- depends_on: story:evaluator-skeleton
- depends_on: story:evidence-freshness
- depends_on: story:invalidation-rules
scope:
- confidence: cited
  path: conformance/scenarios/evidence-order.yaml
- confidence: cited
  path: crates/canon-docs/
- confidence: cited
  path: crates/canon/src/check/
- confidence: cited
  path: crates/canon/src/diff/
- confidence: cited
  path: crates/canon/src/eval/
- confidence: cited
  path: crates/canon/src/explain/
- confidence: cited
  path: crates/canon/src/ir/
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/validate/
- confidence: cited
  path: ess/
- confidence: cited
  path: fixtures/investigation/evidence-order.yaml
- confidence: cited
  path: website/docs/reference/
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-13T00:31:27Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-13T00:31:27Z", actor: "human:timo", revision: 7}
---
## Outcome

`protocol/1` gains a predicate over evidence order (Atlas ADR 0084 § Decision 2): the first record of
a kind and its result, and one kind's first record before another's. Three-valued: UNKNOWN while
the order is not established. Required for the ADR 0077 parity switch: it carries AEP's
`test-driven` principle, `test.first_result == failed` and `evidence.first_seq.test_result <
evidence.first_seq.diff` (aep `principles/development/test-driven.yaml:26,35` at `c0d5f852`).

A consumer's bug-fix profile needs a failing result recorded at one revision of the change to count
after the fix moves the change to a later revision. Revision binding (story:evidence-revision-binding)
keeps every record bound to a superseded revision away from evidence matches, so no predicate in
`protocol/1` 0.2.0 can read that red result once the fix lands. The order forms read the history.

## First step: the predicate syntax (settled)

ADR 0084 § Open leaves "the syntax of the evidence-order predicate" open. Settled here:

- `{first: {kind: K, result: R}}`, optionally `subject: A` inside the map: whether the first record
  of kind `K` (about artifact `A` when named) has result `R`. `kind` and `result` are required.
- `{before: {first: K1, then: K2}}`, optionally `subject: A` inside the map: whether the first
  record of kind `K1` was observed strictly before the first record of kind `K2` (both about `A`
  when named).

Both are new keys of the one-key predicate map (`crates/canon/src/model/predicate.rs`), new
variants `first` and `before` of the `canon.protocol.Predicate` union, with structs
`canon.protocol.FirstResult {kind, result, subject?}` and `canon.protocol.EvidenceOrder {first,
then, subject?}`.

**Order key: `observed_at`**, the instant story:evidence-freshness added to `canon-evidence/1`. No
new field in `canon-evidence/1`: an evidence set that already carries observation instants needs
nothing new. A per-record sequence number was rejected: it would be a second order a producer must
keep consistent with the instants it already writes. The evaluator reads no clock.

**What the order forms read: the history.** An order form reads every record of its kind (about its
subject when named) that the evidence check accepted, whatever the exclusion stages decide:
records excluded as `revision_mismatch` or `expired`, and records invalidated for the claim, are
read. The exclusion stages say what holds of the current revisions now; order is a fact about what
was observed and when, which no later revision or instant changes. Such a record is not listed in a
claim's `excluded_evidence` because of an order form: that list keeps reading evidence matches only.

### Values

`first: {kind: K, result: R}` over the records it reads:

- no record → `unknown`;
- a record without `observed_at` → `unknown` (its place in the order is not known);
- otherwise the records at the earliest `observed_at`: every one has result `R` → `true`; none has
  it → `false`; some have it and some do not → `unknown` (conflict, as an evidence match).

`before: {first: K1, then: K2}`:

- no record of `K1` and none of `K2` → `unknown`;
- a record of `K2` and none of `K1` → `false` (whatever `K1` record comes later comes after);
- a record of `K1` and none of `K2` → `unknown`;
- both, and a record of either without `observed_at` → `unknown`;
- both, with earliest instants `t1`, `t2`: `t1 < t2` → `true`; `t1 > t2` → `false`; equal →
  `unknown` (one second's resolution cannot order them).

### Validation

`canon validate` refuses an order form whose kind (`kind`, `first` or `then`) the protocol does not
declare, or whose `subject` it does not declare, with the refusals an evidence match gets for the
same fault; and a `before` whose `first` and `then` name the same kind, naming it.

### Other readers of a predicate

- Action reasons: an order form that decides a blocked action is a reason `{"first": K, "result":
  R, "value": <truth>}` or `{"before": K1, "then": K2, "value": <truth>}`, with `"subject": A` added
  when named; the explanation names it the same way.
- `canon diff`: an order form is compared structurally; a changed one is classified `BREAKING`
  (classifying the variants more finely is out of scope).
- `canon check`: a protocol with order forms is checked soundly. Every value each order form can
  take, jointly with the evidence matches over the same records, is reached by some state (the space
  gains what it needs: observation instants on the records it builds, or an ordering dimension);
  a protocol with no order form checks as before, byte for byte. The module docs of
  `crates/canon/src/check/` say how.
- `canon-docs` renders the forms in the generated reference and the protocol graph.

## ESS first

- Specification change, first commit: in `ess/domains/protocol.yaml`, the `canon.protocol.Predicate`
  union gains `first` and `before`, with `canon.protocol.FirstResult` and
  `canon.protocol.EvidenceOrder`; with the fixture `fixtures/investigation/evidence-order.yaml` and
  the scenario file `conformance/scenarios/evidence-order.yaml`.
- Red on that commit: `crates/canon/tests/ess_model_matches.rs` fails naming the new variants, because
  the Rust `Predicate` (`crates/canon/src/model/predicate.rs`) does not have them yet; then
  `CANON-ORDER-001` fails under `canon conform run`.

## Scope

- In: the two forms in the source model, validator, IR, evaluator (`eval/order.rs`, dispatched
  from `eval/claims.rs`), action reasons, explanation, `canon check`, `canon diff` (conservative),
  `canon-docs` and the generated reference.
- Out: revision descent and test identity (story:revision-ordering); independence of the records
  (story:evidence-independence); classifying the forms finely in `canon diff`.
- Known limit, stated in the reference: a red record on any earlier revision counts, descended
  from the current one or not; story:revision-ordering closes it.
- Surfaces: `ess/`, `crates/canon/src/model/`, `crates/canon/src/validate/`, `crates/canon/src/ir/`,
  `crates/canon/src/eval/` (`claims.rs`, `order.rs` new, `actions.rs`, `read.rs`, `depth.rs`,
  `invalidation.rs`, `mod.rs` docs), `crates/canon/src/explain/`, `crates/canon/src/check/`,
  `crates/canon/src/diff/`, `crates/canon-docs/`, `website/docs/reference/` (generated),
  `fixtures/investigation/evidence-order.yaml`, `conformance/scenarios/evidence-order.yaml`.

## Order

depends_on story:three-valued-claims, story:evaluator-skeleton, story:evidence-freshness
(`observed_at`, the order key) and story:invalidation-rules, all implemented. It runs alone in its
wave: it shares `ess/`, `model/`, `validate/` and `ir/` with every story that changes `protocol/1`.

## Acceptance

Conformance scenario `CANON-ORDER-001` passes under `canon conform run` over
`fixtures/investigation/evidence-order.yaml`, for a claim requiring a failed first `test_result`
(`first`) and the first `test_result` before the first `diff` (`before`), with these expectations:

1. red at revision r1, then green and a diff at the current revision r2 (the red record excluded as
   `revision_mismatch` from evidence matches): TRUE;
2. green only: FALSE;
3. a diff before any `test_result`: FALSE;
4. no records: UNKNOWN;
5. green then red on one revision: FALSE;
6. red and green at the same earliest instant: UNKNOWN;
7. a `test_result` without `observed_at`: UNKNOWN.

`crates/canon/tests/ess_model_matches.rs` passes; `task check` passes.

## Source

Atlas ADR 0084 (accepted 2026-10-04) § Decision 2 and § Open; aep
`principles/development/test-driven.yaml:26,35`; draft `canon-stories-step-order.md`; Atlas ADR
0080. Consumer need: a bug-fix profile blocked because Canon 0.2.0 binds every evidence record to
the current revision (Engineering Protocols 0.5.0 release notes, stated limitation).
