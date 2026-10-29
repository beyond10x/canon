---
format: aep.planning-md/3
id: story:evidence-order-since
kind: story
status: active
title: 'Predicate over evidence order: a record with a result since the latest of a kind, at its revision'
summary: 'since: {last: K1, then: {kind: K2, result: R}, subject?}, anchored on the latest record of K1 and reading candidates at its subject revision'
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:evidence-order-subsecond
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T07:12:56Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-29T07:12:56Z", actor: "human:timo", revision: 3}
---
## Outcome

`protocol/1` gains a fourth order form, `since`, which reads from the latest record of a kind and
at its revision: "since the tests were last changed, a run of them at that revision failed". A
consumer's test-first rule written with `after` (Canon 0.7.0) still accepts two histories it
should refuse: a failing run about a revision from before the change, observed late, and tests
changed again that pass at once after earlier tests were red. `after` anchors on the first record
of `K1` and reads candidates at any revision; `since` anchors on the latest and reads only
candidates bound to the revision that record is about.

`first`, `before` and `after` keep their meaning, IR, canonical order and checker results; a
protocol without `since` compiles, evaluates, checks, diffs and generates byte for byte as before.

## The form (settled)

`{since: {last: K1, then: {kind: K2, result: R}}}`, optionally `subject: A` inside the outer map:
whether a record of kind `K2` with result `R`, about the same subject and subject revision as the
latest record of kind `K1`, was observed after that record; records about `A` only when it is
named. `last`, `then`, `then.kind` and `then.result` are required. A new key of the one-key
predicate map, a new variant `since` of `canon.protocol.Predicate`, with the struct
`canon.protocol.EvidenceSince {last, then, subject?}`; `then` reuses `canon.protocol.LaterRecord`.

The name: `since` collides with no predicate key, field, IR key or reason key. `last` (not
`first`) says which record of `K1` anchors it.

It reads the history as the other order forms do, excluded records included, and compares
instants as story:evidence-order-subsecond settles (intervals; overlapping instants are not
ordered).

### Values

Let the anchors be the records of `K1` and the candidates the records of `K2` with result `R`,
both about `A` when named. The latest anchors are those no anchor is observed after.

- no anchor → `unknown`;
- no candidate → `false`;
- an anchor or a candidate without `observed_at` → `unknown`;
- otherwise, for each latest anchor `a`, with the candidates at `a`'s subject and subject revision
  other than `a` itself: `true` when one is observed after `a`; `false` when every one is observed
  before `a` or there is none; otherwise `unknown`;
- the form is `true` when every latest anchor gives `true`, `false` when every one gives `false`,
  otherwise `unknown`.

`K1` and `K2` may be one kind: a record is not later than itself.

### Validation and other readers

As for `after` (story:evidence-order-after): undeclared kinds and subject refused with the same
codes, `last` before `then`; refused in an obligation's discharge or applicability predicate; a
malformed form is a parse error. IR `{"since": {"last": K1, "subject": A, "then": {"kind": K2,
"result": R}}}`, subject only when named; among members in code-point order of keys (`since` after
`not`); two `since` forms compare by `last`, `then.kind`, `then.result`, subject. Reasons
`{"since": K1, "then": K2, "result": R, "value": <truth>}` with `"subject"` when named, after
every `after`, `before` and `first` reason. `canon diff`: structural, a changed order form is
`BREAKING`. `canon check` checks a protocol with `since` soundly, and one without byte for byte as
before; `canon generate` writes what the space builds; `canon-docs` renders the form.

## ESS first

- Specification change, first commit: `ess/domains/protocol.yaml` gains the `since` variant and
  `canon.protocol.EvidenceSince`; fixture `fixtures/investigation/evidence-order-since.yaml`,
  scenario `conformance/scenarios/evidence-order-since.yaml` (`CANON-ORDER-004`) and the
  requirement.
- Red on that commit: `crates/canon/tests/ess_model_matches.rs` names the missing variant and
  struct; `canon conform run` fails `CANON-ORDER-004`.

## Scope

- In: model, validator, IR and canonical order, `eval/order.rs`, reasons and explanation,
  `canon check` and `canon generate`, `canon diff`, `canon-docs` and the generated reference,
  `CHANGELOG.md`, the status page.
- Out: revision descent (story:revision-ordering): a candidate at a revision descended from the
  anchor's does not count.

## Acceptance

`CANON-ORDER-004` passes, for a claim requiring a failed `test_result` since the last
`tests_change` about the change, with these expectations:

1. tests changed at h2, then a failed run at h2: TRUE;
2. tests changed at h2, then a failed run about h1 observed later: FALSE;
3. tests changed at h2, failed at h2, passed at h3, changed again at h4, passed at h4: FALSE;
4. as 3, then a failed run at h4: TRUE;
5. a failed run at h2 at the instant of the change at h2: UNKNOWN;
6. no `tests_change`: UNKNOWN; a `tests_change` without `observed_at`: UNKNOWN;
7. two `tests_change` records at one instant at different revisions, a failed run after both at
   one of them: UNKNOWN;
8. one kind: `{since: {last: test_result, then: {kind: test_result, result: failed}}}` with the
   last run failed alone: FALSE.

`canon check` tests show every value of `since` reached jointly with the other order forms; the
golden check reports of protocols without `since` are unchanged; `ess_model_matches` and
`task check` pass.
