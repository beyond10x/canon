---
format: aep.planning-md/3
id: story:evidence-order-subsecond
kind: story
status: implemented
title: Order forms compare observed_at at the resolution it is written in
summary: an instant may carry a fraction of a second; first, before and after order records within one second, and overlapping instants stay unordered
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:evidence-order-after
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T07:12:56Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-29T07:12:56Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-29T08:14:14Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":4}}}
---
## Outcome

An `observed_at` (and the evaluation instant) may carry a fraction of a second, and the order forms
`first`, `before` and `after` compare instants at the resolution they are written in. A consumer
recorded a change and a failed run of its tests 0.4 s apart, both in one second; Canon 0.7.0 reads
both as the same whole second, so `after` is `unknown` and the consumer's edit stays blocked. In a
consumer's self-run cases the two records were within one second in 8 of 16.

Every instant Canon 0.7.0 accepts keeps its meaning, and every order value over whole-second
instants is unchanged: a protocol and evidence without fractions evaluate, check and generate byte
for byte as before.

## The instant (settled)

`YYYY-MM-DDTHH:MM:SSZ` as before, or with a fraction of one to nine digits after the seconds:
`YYYY-MM-DDTHH:MM:SS.fffZ`. Anything else is refused as before (`invalid-instant`), including an
empty fraction (`12:00:00.Z`), more than nine digits and a comma.

An instant names the interval its last written digit spans: `12:00:00Z` is the second
`[12:00:00, 12:00:01)`, `12:00:00.4Z` is `[12:00:00.4, 12:00:00.5)`, `12:00:00.400Z` is
`[12:00:00.400, 12:00:00.401)`. Canon does not invent digits that were not written.

## Order (settled)

Within one evaluation, the order forms compare every instant at one resolution: the coarsest one
written among the records of the evidence set that have an `observed_at`. An instant is read at
that resolution by dropping the digits finer than it, which is the interval at that resolution
that contains the interval it names. Two records are ordered when their instants differ at that
resolution, and not ordered when they are equal there. So:

- evidence written all in milliseconds orders two records 0.4 s apart within one second;
- equal instants are not ordered;
- one record written in whole seconds makes the evaluation compare in whole seconds, so
  `12:00:00Z` beside `12:00:00.400Z` is not ordered, and neither are `12:00:00.200Z` and
  `12:00:00.500Z` in that evidence set;
- an order the forms report is always an order of the intervals the instants name: Canon never
  orders two records whose written instants overlap.

One resolution per evaluation keeps the order a weak order (ties are transitive), which the
`canon check` state space rests on: every evidence set still maps to a state that evaluates the
same way, with no change to the space. Comparing each pair at its own precision would not: with
`12:00:00Z`, `12:00:00.2Z` and `12:00:00.5Z`, the first ties both others, which do not tie each
other, and no check state produces that (adversary pass 1,
`check_reaches_what_mixed_precision_evidence_makes_legitimate`).

The 0.7.0 rules of `first`, `before` and `after` apply unchanged, with "instant" read at that
resolution. With whole seconds everywhere they are the 0.7.0 values. The rules for records without
`observed_at` and for missing kinds are unchanged. A consumer that wants sub-second order writes
every `observed_at` at the same precision.

Expiry (`max_age`, story:evidence-freshness) keeps counting whole seconds: it reads an instant's
whole seconds and ignores its fraction, as it reads `max_age` in whole seconds.

## ESS first

- Specification change, first commit: the comment of `canon.protocol.Instant` in
  `ess/domains/protocol.yaml` states the fraction and the interval; scenario steps in
  `conformance/scenarios/evidence-order-subsecond.yaml` (`CANON-ORDER-003`) over the existing order
  fixtures, and the requirement in `conformance/requirements.yaml`.
- Red on that commit: `canon conform run` fails `CANON-ORDER-003` (a fractional `observed_at` is
  refused as `invalid-instant`).

## Scope

- In: `crates/canon/src/model/time.rs` (reading the fraction), `crates/canon/src/eval/order.rs`,
  the evidence check and the evaluation instant (accepting the fraction), freshness (whole
  seconds, stated), the refusal text, `canon check` and `canon generate` where they write or
  compare instants, the generated reference, `CHANGELOG.md`.
- Out: a total order of equal instants (none is invented); time zones other than `Z`; leap seconds.

## Acceptance

`CANON-ORDER-003` passes under `canon conform run`, with these expectations:

1. a `tests_change` at `12:00:00.100Z` and a failed run at `12:00:00.500Z`: `after` TRUE;
2. the same records the other way round: FALSE;
3. both at `12:00:00.500Z`: UNKNOWN (equal instants are not ordered);
4. `12:00:00Z` and `12:00:00.500Z` (mixed precision): UNKNOWN; `12:00:00Z` and `12:00:01.200Z`:
   ordered;
5. `before` and `first` over records in one second at different fractions: ordered as written;
6. `12:00:00.Z`, ten fraction digits and `12:00:00,5Z` refused as `invalid-instant`.

Every existing conformance expectation and golden check report is unchanged; `task check` passes.

## Delivered

- An instant may carry a fraction of one to nine digits, in evidence and as the evaluation instant;
  the `invalid-instant` refusal names both forms.
- The order forms compare every instant at one resolution per evaluation, the coarsest written
  among the records with an `observed_at` (`ticks` in `eval/order.rs`); the 0.7.0 rules apply over
  those ticks, so whole-second evidence evaluates, checks and generates as before and the
  `canon check` space is unchanged.
- Expiry keeps whole seconds.
- Adversary pass 1 found that comparing each pair at its own precision is not a weak order, which
  made `canon check` report an outcome unreachable that mixed-precision evidence makes legitimate;
  the settled rule replaced it, and the contracts of `first`, `before` and `after` now name the
  resolution.
