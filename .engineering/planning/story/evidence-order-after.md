---
format: aep.planning-md/3
id: story:evidence-order-after
kind: story
status: active
title: 'Predicate over evidence order: a record of a kind with a result after the first of another'
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:evidence-order-predicate
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: conformance/requirements.yaml
- confidence: cited
  path: conformance/scenarios/evidence-after.yaml
- confidence: cited
  path: crates/canon-cli/src/lib.rs
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
  path: crates/canon/src/generate/
- confidence: cited
  path: crates/canon/src/ir/
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/validate/
- confidence: cited
  path: ess/domains/protocol.yaml
- confidence: cited
  path: fixtures/investigation/evidence-after.yaml
- confidence: cited
  path: website/docs/reference/
- confidence: cited
  path: website/static/schemas/
- confidence: cited
  path: website/status.yaml
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T22:43:54Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-28T22:43:54Z", actor: "human:timo", revision: 3}
---
## Outcome

`protocol/1` gains a third order form over the evidence history, `after`, which reads a record
later than the first of a kind: "a `test_result` with result `failed` was observed strictly after
the first `tests_change`". The two order forms of 0.2.0 read only the first record of a kind:
`first` the result of the first record, `before` the order of two first records. Neither can say
that some later record of a kind, with a given result, followed the first record of another, which
a consumer's test-first profile needs: the tests change, then a run of them fails, whatever ran
before the change.

`first` and `before` keep their meaning, IR, canonical order and checker results; so does every
other predicate form. A protocol without an `after` form compiles, evaluates, checks, diffs and
generates byte for byte as before.

## The form (settled)

`{after: {first: K1, then: {kind: K2, result: R}}}`, optionally `subject: A` inside the outer map:
whether some record of kind `K2` with result `R` was observed strictly after the first record of
kind `K1`, both about `A` when it is named. `first`, `then`, `then.kind` and `then.result` are
required. A new key of the one-key predicate map (`crates/canon/src/model/predicate.rs`), a new
variant `after` of the `canon.protocol.Predicate` union, with structs `canon.protocol.EvidenceAfter
{first, then, subject?}` and `canon.protocol.LaterRecord {kind, result}`.

The name: `after` collides with no predicate key, no field of the predicate map, no IR key and no
reason key. It reads with `before`: in both, `first` names the kind observed first and `then` what
comes after it; `before` compares the two first records, `after` reads every record of `then`. The
`then` map carries the result so it is plain which side the result constrains.

It reads the history as `first` and `before` do: every record of `K1` and of `K2` (about `A` when
named) that the evidence check accepted, including those the exclusion stages keep from evidence
matches (`revision_mismatch`, `expired`, invalidated). The order key is `observed_at`, compared in
whole seconds. No new evidence field.

### Values

Let the first records be the records of `K1`, and the later candidates the records of `K2` with
result `R`, both about `A` when named.

- no record of `K1` → `unknown` (the first record, the reference point, is not established);
- no later candidate → `false` (whatever its instants, nothing of `K2` with `R` was observed);
- a record of `K1`, or a later candidate, without `observed_at` → `unknown`;
- otherwise, with `t1` the earliest instant of `K1`: a later candidate observed after `t1` →
  `true`;
- otherwise a later candidate observed at `t1` that is not itself the only record of `K1` at `t1`
  → `unknown` (one second's resolution cannot order two records at one instant, as `before` treats
  equal first instants);
- otherwise → `false`.

A record of `K2` with another result takes no part: its instant, or the lack of one, decides
nothing, as a record of a third kind decides nothing for `before`. The conventions are those of
`first` and `before`: a record the comparison reads without an instant leaves the form `unknown`;
equal instants are not ordered; what is decided without comparing instants (`before`'s record of
`then` and none of `first`, here no later candidate at all) is decided without them.

`K1` and `K2` may be one kind: `{after: {first: test_result, then: {kind: test_result, result:
failed}}}` asks whether a failed run followed the first run. A record is not later than itself, so
a first run that failed, alone, is `false`. Unlike `before`, whose one-kind form can never be true,
this is not refused.

### Validation

`canon validate` refuses an `after` whose `first` or `then.kind` the protocol does not declare
(`undeclared-evidence-kind`, `first` before `then`, a kind named twice once) or whose `subject` it
does not declare (`undeclared-artifact`), with the refusals an evidence match gets; and an `after`
in an obligation's discharge or applicability predicate, as the other order forms are refused
there. A malformed form (a missing or unknown key, `then` not a map) is a parse error.

### Other readers of a predicate

- IR: `{"after": {"first": K1, "subject": A, "then": {"kind": K2, "result": R}}}`, as authored, the
  subject only when named. Among the members of an `all` or `any`, `after` comes first: the forms
  compare in code-point order of their keys, `after`, `all`, `any`, `before`, `claim`, `evidence`,
  `first`, `input`, `not`, which keeps the order of every other form. Two `after` forms compare by
  `first`, then `then.kind`, then `then.result`, then subject (none first).
- Action reasons and the explanation: `{"after": K1, "then": K2, "result": R, "value": <truth>}`,
  with `"subject": A` when named. Order reasons come `after` first, then `before`, then `first`
  (code-point order of the key, which keeps `before` ahead of `first`).
- `canon diff`: compared structurally like the other order forms; a predicate whose order forms
  changed is `BREAKING`.
- `canon check`: a protocol with an `after` form is checked soundly; one without is checked as
  before, byte for byte. The space keeps the dimensions the order forms already have and, only
  when an `after` reads evidence, adds for each evidence dimension an `after` reads as its `then`
  one later record per result asked for: absent, at no known instant, or placed in the order of
  first observations, which then orders those later records too. In such a protocol a dimension
  whose order is unknown gets an earlier record without a result or an instant (its current
  records keep their instants), and a dimension with earlier records has its current records just
  after its own first observations rather than after every one, so a current record of `K2` does
  not count as later than every first record. The kinds an `after` reads count for
  `unproduced-evidence`, and for `authority-bypass` in every position, as the other order forms'
  do. The module docs of `crates/canon/src/check/` say how.
- `canon generate` writes the later records as the space builds them.
- `canon-docs` renders the form in the generated reference and the protocol graph.

## ESS first

- Specification change, first commit: in `ess/domains/protocol.yaml`, the `canon.protocol.Predicate`
  union gains `after`, with `canon.protocol.EvidenceAfter` and `canon.protocol.LaterRecord`; the
  fixture `fixtures/investigation/evidence-after.yaml`, the scenario file
  `conformance/scenarios/evidence-after.yaml` (`CANON-ORDER-002`) and the requirement
  `CANON-ORDER-002` in `conformance/requirements.yaml`.
- Red on that commit: `crates/canon/tests/ess_model_matches.rs` fails naming the new variant and
  structs, because the Rust `Predicate` does not have them yet; `canon conform run` fails
  `CANON-ORDER-002`, whose fixture does not parse.

## Scope

- In: the form in the source model, validator, IR and canonical order, evaluator
  (`eval/order.rs`), action reasons, explanation, `canon check` and `canon generate`, `canon diff`,
  `canon-docs` and the generated reference, `CHANGELOG.md` and the status page.
- Out: revision descent (story:revision-ordering); a later record of `K2` with any result (the
  result is required); counting records; an `after` over the first record of `K2` only, which
  `before` already says.
- Known limit, as for the other order forms: a record bound to any earlier revision counts,
  descended from the current one or not.
- Surfaces: `ess/domains/protocol.yaml`, `crates/canon/src/model/` (`predicate.rs`,
  `requirement.rs`, `mod.rs`), `crates/canon/src/validate/`, `crates/canon/src/ir/`,
  `crates/canon/src/eval/` (`order.rs`, `claims.rs`, `actions.rs`, `read.rs`, `depth.rs`),
  `crates/canon/src/explain/`, `crates/canon/src/check/`, `crates/canon/src/diff/`,
  `crates/canon/src/generate/`, `crates/canon-cli/src/lib.rs` (help text), `crates/canon-docs/`,
  `website/docs/reference/` and `website/static/schemas/` (generated), `website/status.yaml`,
  `fixtures/investigation/evidence-after.yaml`, `conformance/scenarios/evidence-after.yaml`,
  `conformance/requirements.yaml`, `CHANGELOG.md`.

## Order

depends_on story:evidence-order-predicate (the order forms, the history and the check space's
history and order dimensions), implemented. It runs alone in its wave: it shares `ess/`,
`model/`, `validate/`, `ir/` and `check/` with every story that changes `protocol/1`.

## Acceptance

Conformance scenario `CANON-ORDER-002` passes under `canon conform run` over
`fixtures/investigation/evidence-after.yaml`, for a claim requiring a failed `test_result` after
the first `tests_change` (`after`), an action requiring the same about the change as subject, and
a claim requiring a failed `test_result` after the first `test_result` (one kind), with these
expectations:

1. tests changed, then a failed run at a superseded revision, then a passing run at the current
   one (the failed record excluded from the evidence match as `revision_mismatch`): TRUE; the
   one-kind claim FALSE, its only failed record being the first record;
2. a failed run, then the tests changed, then a passing run: FALSE;
3. a failed run and no `tests_change`: UNKNOWN;
4. the tests changed and only a passing run follows: FALSE;
5. a failed run at the instant of the first `tests_change`: UNKNOWN; with a second failed run a
   second later: TRUE;
6. a failed run without `observed_at`: UNKNOWN; a passing run without `observed_at` beside a
   failed run after the change: TRUE;
7. a `tests_change` without `observed_at`: UNKNOWN;
8. two failed runs a second apart and no `tests_change`: the one-kind claim TRUE, the other UNKNOWN;
9. a failed run about another artifact: read by the claim, not by the action bound to the change.

The `compile` step pins the IR and the canonical order of `after` among members. `canon check`
tests show every value of `after` reached jointly with `first`, `before` and evidence matches over
the same records, and the golden check reports of protocols without `after` unchanged.
`crates/canon/tests/ess_model_matches.rs` passes; `task check` passes.

## Source

A consumer's test-first profile: the tests are changed, then a run of them is seen failing, then
the change makes them pass. Canon 0.6.0 has `first` and `before`, which read only first records
(story:evidence-order-predicate).
