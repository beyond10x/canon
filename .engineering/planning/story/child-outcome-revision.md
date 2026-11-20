---
format: aep.planning-md/3
id: story:child-outcome-revision
kind: story
status: active
title: Bind a child case's outcome to the parent's current revision on the children form
tags:
- from-request
refs:
- provider: sdc
  reference: docs/canon-requests.md
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:case-composition
scope:
- confidence: cited
  path: conformance/requirements.yaml
- confidence: cited
  path: conformance/scenarios/child-outcome-revision.yaml
- confidence: inferred
  path: crates/canon/src/check/space.rs
- confidence: cited
  path: crates/canon/src/eval/actions.rs
- confidence: cited
  path: crates/canon/src/eval/claims.rs
- confidence: cited
  path: crates/canon/src/eval/composition.rs
- confidence: inferred
  path: crates/canon/src/eval/mod.rs
- confidence: inferred
  path: crates/canon/src/explain/mod.rs
- confidence: cited
  path: crates/canon/src/ir/mod.rs
- confidence: cited
  path: crates/canon/src/model/predicate.rs
- confidence: inferred
  path: crates/canon/src/validate/mod.rs
- confidence: inferred
  path: crates/canon/tests/case_composition.rs
- confidence: cited
  path: ess/domains/protocol.yaml
- confidence: cited
  path: fixtures/investigation/child-outcome-revision.yaml
revision: 21
transitions:
- {from: "draft", to: "proposed", at: "2026-11-20T22:35:01Z", actor: "human:timo", revision: 20, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
- {from: "proposed", to: "active", at: "2026-11-20T22:35:01Z", actor: "human:timo", revision: 21, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
---
## Outcome

A `children` form can bind each child's outcome to the parent's current revision of the child's
subject. In `protocol/1` the form `{children: {in: [O, ...], revision: current}}` reads only the
`case_outcome` records whose `subject_revision` is the case snapshot's current revision of their
`subject`. A record about an earlier revision does not count, and the form never reports it as
contradiction. With no such record the form is `unknown`. Over the records it reads, `false`,
`unknown` and `true` follow the existing rules (CANON-COMPOSE-001). A form without `revision`, or
with `revision: any`, reads every accepted record, as it does today. `canon-ir/1` writes
`"revision": "current"` only when the protocol declares it, so the IR of every existing protocol
stays the same byte for byte. A decision's reason names the form with its key in the same way.
A parent whose subject artifact is revised after its children ended therefore has its
`revision: current` claim fall back to `unknown`, and an outcome that requires the claim is blocked.
No new refusal or validation code is added. `inert-invalidation` is unchanged.

## Why

- Request item 5 (`$HOME/beyond10x/sdc/docs/canon-requests.md:33-36`) says a spec revised after
  its stories passed still leaves `children` TRUE. The prototype's invalidation rule on the claim
  (`$HOME/beyond10x/sdc/docs/limitations/child-outcomes-invalidated.protocol.yaml:13-34`) is
  refused: `error[inert-invalidation]` (`$HOME/beyond10x/sdc/docs/limitations/output.txt:29-31`).
  `$HOME/beyond10x/sdc/docs/boundary.md:110` classes the problem as Canon language and names two
  directions.
- Verified in the tree at `15cfece`:
  - `composition::value` (`crates/canon/src/eval/composition.rs:31-47`) reads `history`. The
    module doc (`:8-10`) says a later revision, an expiry or an invalidation does not undo a
    child's outcome.
  - `claims.rs:187-188` and `:236` pass `evidence.history`, not `left`, to the form.
  - `eval/mod.rs:427` runs revision binding. That stage marks a `case_outcome` record at an old
    revision `revision_mismatch` (`eval/binding.rs:38-47`). `eval/mod.rs:439` still gives the
    form every record.
  - Conformance step `an-outcome-about-an-earlier-revision-counts`
    (`conformance/scenarios/case-composition.yaml:397-436`) pins the current behaviour: a record at
    `f1` with the case at `f2` gives TRUE.
  - `ess/domains/protocol.yaml:671-695` gives the reason: "a child left out would let the parent
    conclude without it".
  - `inert-invalidation` is pushed at `crates/canon/src/validate/mod.rs:720`, from
    `reaches_evidence` (`:739-760`). That function counts only `Predicate::Evidence`. Rule 9 of the
    module doc (`:57-61`) says a `children` form "reads the history, which an invalidation rule does
    not change". The refusal is correct: the rule could keep no record from the claim.
  - `duplicate-child-case` (`composition.rs:75-106`) allows one record per child "whatever their
    revisions". A child's record therefore cannot move to a new revision. Only a new child, with a
    new id, can have a record at the new revision.
  - `canon check` already gives each dimension a `children` form reads a history dimension of
    superseded records (`crates/canon/src/check/mod.rs:222-238`, `check/space.rs:56-58`).
- Atlas ADR 0081 (`$HOME/beyond10x/atlas/architecture/adr/0081-cases-compose-at-run-time.md:36-41`)
  says nothing about revisions. story:case-composition does not either.
- Boundary: opening children again after a revision and writing their `case_outcome` records
  belongs to the runner (Loom), not Canon (`canon-requests.md:62`, `boundary.md:87`). Canon owns
  the form's value only.
- Could not verify: no `canon` binary is on `PATH` (`command not found`), so `output.txt:30` was
  not reproduced. No document says which revision a runner writes as a `case_outcome` record's
  `subject_revision`: the one when the child opened, or the one when its outcome was written.

## Decision needed

| | Option | Change to `ess/` and the model | Existing protocols, IR bytes | What the runner must do |
|---|---|---|---|---|
| A | `revision: current` on the `children` form | `ChildOutcomes` gains `revision: Optional<canon.protocol.ChildRevision>`, a new enum with the variants `any` and `current`. The form reads records at the current subject revision. `EvidenceSets` needs the case's revisions. | No change. The key is written only when it is `current`, as `ir/mod.rs:18-28` already does for ten optional keys, so existing IR and decisions stay identical. | Write each `case_outcome` record's `subject_revision` as the revision the child was opened about, and never move it. Open new children, with new ids, after a revision. |
| B | Explicit reopen, with no language change | None. At most, the reference documents a pattern. | No change. | Open new children on a revision. Also write a separate record at the current revision that the claim requires beside `children`, so binding or an invalidation rule can reach it, or else write the old child's record back to `open`. Canon cannot check either, and a protocol that leaves this out stays silently TRUE, which is the SDC gap. |
| C | Let invalidation rules reach `case_outcome` records | `reaches_evidence` (`validate/mod.rs:739`) counts `children`. The form reads history minus the records invalidated for the claim, using `upstream_revisions`. The rule 9 text and the `ChildOutcomes` doc change. There is no new field. | IR bytes do not change, and the SDC probe validates as written. Decisions do change: an existing claim that combines `children` with an evidence match under a rule changes value when a `case_outcome` record names an old upstream revision. `canon check` gives `case_outcome` upstream variants (`check/mod.rs:39-52`), and the SDC is already at the bound. | Write `upstream_revisions` on every `case_outcome` record, and open new children on a revision. |

Under A, a record about an earlier revision is not counted rather than made `unknown`. Making it
`unknown` would block the form for good, because `duplicate-child-case` lets an old child's record
never become current. The decided cost: under A, a stale `abandoned` child no longer makes the
form FALSE.

Recommendation: option A. It is the only option that changes no existing IR or decision, uses the
revision the case snapshot already holds, needs no new `canon check` dimension, and makes the
binding part of the protocol rather than an unchecked convention. Option B leaves it to every
protocol and runner. Option C changes existing evaluation output.

## ESS first

- First commit:
  - In `ess/domains/protocol.yaml`, add the field `revision: Optional<canon.protocol.ChildRevision>`
    on `canon.protocol.ChildOutcomes` and the enum `canon.protocol.ChildRevision`
    (`["any", "current"]`), each with its `# read from:` comment.
  - Add `CANON-COMPOSE-002` to `conformance/requirements.yaml`.
  - Add the fixture `fixtures/investigation/child-outcome-revision.yaml` and the scenario
    `conformance/scenarios/child-outcome-revision.yaml`.
- Red on that commit:
  - `crates/canon/tests/ess_model_matches.rs` fails naming the new field and enum, because the Rust
    model has neither.
  - `canon conform run --requirements conformance/requirements.yaml` fails `CANON-COMPOSE-002` at
    `compile`, because `#[serde(deny_unknown_fields)]` on `ChildOutcomes`
    (`crates/canon/src/model/predicate.rs:262-263`) refuses `revision`.

## Scope

- In: the `revision` key on the `children` form, its parsing, IR form, evaluation, reasons and
  explanation, and scenario `CANON-COMPOSE-002`. — cited
- Out:
  - Opening children again and writing `case_outcome` records (Loom). — cited
  - Option C's change to `inert-invalidation`. — cited
  - Classifying `revision` in `canon diff` beyond the existing rule: any change to a `children`
    form is BREAKING (`diff/mod.rs:420-429`, `:1165`). — cited
  - Request item 6 (parent-to-child evidence). — cited
- Surfaces:
  - `ess/domains/protocol.yaml` — cited
  - `crates/canon/src/model/predicate.rs` — cited
  - `crates/canon/src/eval/composition.rs` — cited
  - `crates/canon/src/eval/claims.rs` — cited
  - `crates/canon/src/eval/actions.rs`: `named.children` is a `BTreeSet<&[String]>` at `:187` and
    `:323`, keyed by outcome list only. — cited
  - `crates/canon/src/eval/mod.rs` — inferred
  - `crates/canon/src/ir/mod.rs` — cited
  - `crates/canon/src/validate/mod.rs` — inferred
  - `crates/canon/src/explain/mod.rs` — inferred
  - `crates/canon/src/check/space.rs` — inferred
  - `crates/canon/tests/case_composition.rs` — inferred
  - `conformance/requirements.yaml` — cited
  - `conformance/scenarios/child-outcome-revision.yaml` — cited
  - `fixtures/investigation/child-outcome-revision.yaml` — cited
  - `website/status.yaml` — inferred
- Confidence: medium. The evaluation and IR sites are cited. Whether `check` needs a change, or
  whether its existing history dimension already reaches every value, is inferred.

## Order

- Depends on story:case-composition (implemented).
- Shares `ess/domains/protocol.yaml`, `model/predicate.rs`, `ir/mod.rs`, `validate/mod.rs`,
  `eval/claims.rs` and `conformance/requirements.yaml` with story:revision-ordering (active).
- Shares `eval/claims.rs` and `eval/mod.rs` with story:invalidation-evaluation-cost (active).
- Shares `ess/domains/protocol.yaml` and `conformance/requirements.yaml` with
  story:protocol-imports (active).
- Each pair runs in a different wave. This story contradicts none of them: it adds no order form
  and does not touch invalidation.

## Acceptance

Stated for option A. Conformance scenario `CANON-COMPOSE-002` passes under `canon conform run` over
`fixtures/investigation/child-outcome-revision.yaml`. In the fixture, claim `delivered.current` is
`{children: {in: [delivered], revision: current}}` and claim `delivered.any` is the same form
without `revision`. Child cases are opened about artifact `spec`, and outcome `built` requires
`delivered.current`. The scenario has these expectations:

1. `compile` writes `{"children": {"in": ["delivered"], "revision": "current"}}` for
   `delivered.current` and `{"children": {"in": ["delivered"]}}`, with no `revision` key, for
   `delivered.any`.
2. Spec is at `s2` and the only record, about `s1`, says `delivered`. `delivered.current` is
   `unknown`, `delivered.any` is `true`, and `built` is blocked.
3. Spec is at `s2`. One child is `delivered` about `s1` and one is `delivered` about `s2`.
   `delivered.current` is `true`.
4. Spec is at `s2`. One child is `abandoned` about `s1` and one is `delivered` about `s2`.
   `delivered.current` is `true` and `delivered.any` is `false`.
5. Spec is at `s2`. One child is `abandoned` about `s2`, and `delivered.current` is `false`. With
   that child `open` instead, `delivered.current` is `unknown`.
6. In expectation 2, neither claim lists the `s1` record in `excluded_evidence`. The blocked
   action's reason names the form with `"revision": "current"` and the value `unknown`.
7. `CANON-COMPOSE-001` still passes unchanged, including `an-outcome-about-an-earlier-revision-counts`.

`crates/canon/tests/ess_model_matches.rs` passes.

## Source

Request item 5 of `$HOME/beyond10x/sdc/docs/canon-requests.md`, 2026-11-20. Files read:

- In the sdc directory: `docs/limitations/output.txt`,
  `docs/limitations/child-outcomes-invalidated.protocol.yaml` and `docs/boundary.md`.
- In the canon tree at `15cfece`: `AGENTS.md`, `crates/canon/src/model/{mod,predicate,evidence}.rs`,
  `crates/canon/src/eval/{composition,claims,binding,invalidation,mod,actions}.rs`,
  `crates/canon/src/validate/mod.rs`, `crates/canon/src/ir/mod.rs`,
  `crates/canon/src/check/{mod,space}.rs`, `crates/canon/src/diff/mod.rs`, `ess/domains/protocol.yaml`,
  `conformance/requirements.yaml` and `conformance/scenarios/case-composition.yaml`.
- Plans and records: story:case-composition, story:revision-ordering,
  story:invalidation-evaluation-cost, story:protocol-floor, story:protocol-imports and
  story:canon-mutate, and Atlas ADR 0081.

## Runner duty under option A

Option A reads `subject_revision` on a `case_outcome` record as the revision the child was opened
about. Canon does not write that record and cannot check who did. Whoever runs the cases owes two
things, and the reference page for the `children` form states them when this story lands:

1. A `case_outcome` record's `subject_revision` is the parent's revision of the child's subject at
   the moment the child case was opened. It is never moved to a later revision.
2. After the parent revises that subject, the runner opens new child cases with new ids.
   `duplicate-child-case` allows one record per child whatever its revision, so an old child's
   record can never become current.

The session working on the SDC prototype told the coordinator on 2026-11-20 that its runner (the
`sdc` driver's child-case runtime, being written that day) does both. That is that runner's
convention and no Canon rule; the drafter found no Canon document and nothing in Atlas ADR 0081
that says which revision a runner writes.

## Decided

Option A of § Decision needed: the `revision: current` key on the `children` form, with the runner duty of § Runner duty under option A. Decided on 2026-11-20 by the wave coordinator on the drafter's recommendation, after the operator accepted the story without choosing (`approval-record:accept-sdc-requests-2026-11-20`). The unit builds this option and does not reopen the choice.
