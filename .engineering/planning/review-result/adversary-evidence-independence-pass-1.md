---
format: aep.planning-md/3
id: review-result:adversary-evidence-independence-pass-1
kind: review-result
status: active
title: Wave independence adversary, canon story:evidence-independence, pass 1
relations:
- reviews: story:evidence-independence
revision: 1
---
The adversary pass on story:evidence-independence found two defects and committed three failing tests for them (9d63f5f). The core rule held under every probe: a record failing a demand never made a claim, action or outcome TRUE or FALSE.

```
unit: story:evidence-independence, tree canon-unit-independence, branch unit/evidence-independence; findings cover 481010a..2a1ff3a (red cases on top as 9d63f5f)
verdict: NEEDS-CHANGE
cases: executed 485→492, red 3
origin: introduced 3 / pre-existing 0 / undecided 0
```

## Findings

1. `crates/canon/src/diff/mod.rs` (`independence_changed`), warning, introduced: a dimension added to `independent_on` under `not` was reported as RELAXATION, "demands less independence", though it adds a demand and blocks a case (author-only approval) that was admissible before. Red: `adversary_independence_diff.rs`, two cases.
2. `crates/canon/src/check/space.rs` (`profiles`), warning, introduced: one bit per (class, producer profile) made a four-kind software-change protocol exceed the 65536-state bound (524288 states, against 512 without demands); the documented minimality was false, since only the down-closed set of demands reading each class matters. Red: `adversary_independence_check.rs`.
3. `crates/canon/tests/evidence_independence.rs`, note: the unit's own diff test pinned the wrong wording of finding 1.
4. `crates/canon/src/diff/mod.rs`, note, infeasible within the story: a demand added to a positive match is TIGHTENING by design § 31, yet it can turn UNKNOWN into TRUE when an independent approval meets a non-independent rejection.

## Decisions on the findings

- 1 and 3: a changed demand under `not` is BREAKING; the wording follows the dimensions (added: more independence; removed: less). Positive polarity keeps design § 31.
- 2: the state space enumerates one value per down-closed set of demands per record class, and drops the producer-less profile for kinds no plain match or order form reads.
- 4: one line in the diff documentation states the limit.

## What held

Failing records decide nothing through `any`, `not`, claim tests, actions, outcomes, invalidation and freshness; `independent_on` on an order form is refused at parse; producer edge cases (`[]`, absent, no producer, superset, subset) behave as specified; duplicate and undeclared dimensions are refused, also in hand-edited IR; decision and explanation output are byte-identical under permuted map, dimension and evidence order, and agree on `not_independent`; dimension declarations removed are BREAKING and added are EXPANSION; check reported no false finding; every witness `canon generate` wrote passed `canon conform run`.
