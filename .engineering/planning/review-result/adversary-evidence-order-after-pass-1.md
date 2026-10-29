---
format: aep.planning-md/3
id: review-result:adversary-evidence-order-after-pass-1
kind: review-result
status: active
title: Adversary pass 1, canon story:evidence-order-after
relations:
- reviews: story:evidence-order-after
revision: 1
---
The adversary pass over the after order form found no break of its semantics, its IR, its
canonical order, its check space or its diff. It found two things to change, both fixed on the
unit branch, and recorded three conventions as kept.

```
unit: story:evidence-order-after, branch unit/evidence-order-after; findings cover 76729fc
verdict: CONFIRMED
cases: 11 added in crates/canon/tests/adversary_evidence_order_after.rs (c4abeb1), all green
origin: introduced 4 / pre-existing 2 / undecided 0
wrote-outside-worktree: scratch only, deleted
```

Attacked, held: equal instants over two kinds and over one kind (a record is not later than
itself; two records at the first instant leave it unknown); records without observed_at of the
first kind, of the later kind with the result, and of the later kind with another result;
records excluded as revision_mismatch, expired and invalidated (read by after, not listed as
excluded because of it); a subject filtering both kinds; a result no record has; `not` and
`any`; the order records are given in; afters that differ in one field kept apart by the
compiler's deduplication and sorted as documented; IR read-back and refusal of malformed or
undeclared hand-written forms; `canon check` reachability over one kind and authority-bypass
under `not`; `canon diff` of every changed field.

```findings
[
{"file":"crates/canon/tests/adversary_scenario_generation.rs","line":152,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"The first full task check failed here: fixtures/investigation/evidence-order-after.yaml reads its kinds bound to a subject and unbound at once, its check space (606916800 states) is beyond the bound, and the test requires every investigation fixture to generate. Fixed in 65094ed: the fixture is named among those refused as state-space-bound, and an inline protocol with after over two kinds, one kind and under not (4212 states) is generated from instead."},
{"file":"crates/canon/src/eval/mod.rs","line":48,"category":"contract-drift","severity":"note","verdict":"NEEDS-CHANGE","origin":"introduced","message":"The evaluator's module docs, an action's reasons (eval/actions.rs) and an explanation's claim reasons (explain/mod.rs) named only first and before, though after is evaluated and its reasons come first. Fixed in 76729fc, with website/docs/reference/evaluation.md regenerated."},
{"file":"crates/canon/src/eval/order.rs","line":135,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"A K1 record without observed_at leaves after unknown even when a later candidate follows every placed K1 record, where true could be derived. Kept by design: it is the convention of first and before, and CANON-ORDER-002 expectation 7 states it. Pinned by adversary_evidence_order_after.rs."},
{"file":"crates/canon/src/eval/order.rs","line":145,"category":"mutant","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"A strict-later mutant (>= for >) fails two cases of adversary_evidence_order_after.rs (equal instants, not over an equal instant), besides the unit tests and CANON-ORDER-002."},
{"file":"crates/canon/src/diff/mod.rs","line":1,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"Adding any conjunct beside an outcome's requirement is BREAKING, whether an evidence match, first, before or after; after is classed as the others are. No change."},
{"file":"crates/canon/src/validate/mod.rs","line":223,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"pre-existing","message":"An invalidation rule whose claim reads only order forms is refused as inert-invalidation, as for first and before: order forms read invalidated records. No change."}
]
```
