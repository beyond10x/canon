---
format: aep.planning-md/3
id: story:invalidation-rules
kind: story
status: implemented
title: Invalidation rules for upstream artifact changes
refs:
- provider: taskboard
  reference: C-008
relations:
- decomposes: epic:canon-kernel
- depends_on: story:conformance-runner
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:ess-hard-gate
- depends_on: story:evaluator-skeleton
- depends_on: story:decision-outcomes
scope:
- confidence: cited
  path: conformance/scenarios/invalidation-rules.yaml
- confidence: cited
  path: crates/canon-docs/src/pages.rs
- confidence: cited
  path: crates/canon/src/check/
- confidence: cited
  path: crates/canon/src/eval/actions.rs
- confidence: cited
  path: crates/canon/src/eval/binding.rs
- confidence: cited
  path: crates/canon/src/eval/claims.rs
- confidence: cited
  path: crates/canon/src/eval/decision.rs
- confidence: cited
  path: crates/canon/src/eval/evidence.rs
- confidence: cited
  path: crates/canon/src/eval/invalidation.rs
- confidence: cited
  path: crates/canon/src/eval/mod.rs
- confidence: cited
  path: crates/canon/src/eval/read.rs
- confidence: cited
  path: crates/canon/src/explain/mod.rs
- confidence: cited
  path: crates/canon/src/ir/
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/model/decision.rs
- confidence: cited
  path: crates/canon/src/validate/
- confidence: cited
  path: crates/canon/tests/adversary2_invalidation.rs
- confidence: cited
  path: crates/canon/tests/adversary2_ir_order.rs
- confidence: cited
  path: crates/canon/tests/adversary_invalidation.rs
- confidence: cited
  path: ess/
- confidence: cited
  path: fixtures/investigation/invalidation-rules.yaml
- confidence: cited
  path: website/
revision: 22
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:51Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-10-04T08:27:40Z", actor: "human:timo", revision: 13}
- {from: "active", to: "implemented", at: "2026-10-04T09:50:52Z", actor: "human:timo", revision: 22, decided_on: {"recorded":{"test_result":1,"review_outcome":3,"verification":1}}}
---
## Outcome

A protocol declares invalidation rules (operator, 2026-10-04: decision-blocker:upstream-revision-binding):
each rule names an upstream artifact and the claims whose support it invalidates when that artifacts
revision moves (design § 4.1, contract sketch `InvalidationRule`). Evidence bound to a claim named by
a rule stops supporting it once the upstream artifacts revision in the case snapshot differs from the
one recorded when the evidence was observed. Completes the invalidation half of TASKBOARD C-008
(CANON-INVALIDATION-001).

## Extends

The source model, validator and IR gain `invalidation:` rules; `canon-evidence/1` records the
upstream revisions it was observed against; invalidated evidence joins the excluded-evidence list of
each `claims` entry with reason `invalidated`.

## Shared surface and order (re-plan 2026-10-04)

story:evaluator-skeleton lands the invalidation exclusion stage in `eval/invalidation.rs` and the
`excluded_evidence` slot with its `invalidated` reason, so this story does not touch `eval/mod.rs`
or `canon-decision/1`. The shape of an invalidation rule and of the upstream revisions an evidence
record carries is not settled anywhere the skeleton could read it, so this story still changes
`model/`, `validate/`, `ir/`, `ess/` and `eval/evidence.rs`, which story:evidence-freshness and
story:decision-outcomes change too. It is the last of those three: nothing else waits on it except
story:conformance-suite and story:semantic-diff, so it runs beside story:explanation.

- Kept: depends_on story:conformance-runner (its acceptance is a scenario), story:ess-hard-gate
  (`ess/`).
- Added: depends_on story:evaluator-skeleton (stage, slot), story:decision-outcomes (shared
  `model/`, `validate/`, `ir/`, `ess/`: ordering only, because those files are not split).
- Removed: depends_on story:evidence-freshness (now reached through story:decision-outcomes).
- The upstream revisions on `canon-evidence/1` are optional: evidence without them is never
  invalidated, so no earlier scenario changes.

## Scope

- Surfaces: `crates/canon/src/model/`, `crates/canon/src/validate/`, `crates/canon/src/ir/`,
  `ess/`, `crates/canon/src/eval/invalidation.rs`, `crates/canon/src/eval/evidence.rs`,
  `fixtures/investigation/invalidation-rules.yaml`, `conformance/scenarios/invalidation-rules.yaml`
  (the earlier scope named the whole `conformance/scenarios/` directory; this story writes one
  file in it).

## ESS

This story changes the `protocol/1` source model (`invalidation:` rules) and the `canon-evidence/1` input (the upstream revisions). It updates `ess/` (domain `canon.protocol`, set up by story:ess-hard-gate) in this same
story. Every new declaration cites the file and line it was read from, and `task ess-gate` stays
green with no `UNMAPPED:` (Atlas ADR 0076). `ess_gate` does not compare `ess/` with the Rust
model, so the review of this story is what checks that the two agree.

## ESS first

- Specification change, first commit: the `invalidation:` rules on `canon.protocol.Protocol` in
  `ess/domains/protocol.yaml` and the optional upstream revisions on the `canon-evidence/1`
  declaration in `ess/`; with them, the scenario file `conformance/scenarios/invalidation-rules.yaml`
  and its fixture.
- Red on that commit: `ess_model_matches` fails naming the invalidation rules, because the Rust
  model does not have them; and scenario `CANON-INVALIDATION-001` fails, because the fixture's
  `invalidation:` section does not parse.

## Acceptance

Scenario `CANON-INVALIDATION-001` over `fixtures/investigation/invalidation-rules.yaml` passes with
three expectations: the claim is `TRUE` while the upstream artifact is at the recorded revision;
`UNKNOWN`, not `FALSE`, with the evidence listed as excluded for `invalidated` once only the upstream
revision moves; and a rule naming an undeclared artifact is refused by `canon validate` naming it.

## Source

Design § 4.1, § 32; docs/contracts/protocol-core.md; decision-blocker:upstream-revision-binding (cleared 2026-10-04); TASKBOARD C-008.

## Coordinator decisions (wave 2026-10-04-w11)

- Per-claim exclusion: a record is excluded as `invalidated` only from the claims a rule names and
  every claim built on them; a claim the rule does not name that reads the same kind keeps it. This
  supersedes "does not touch `eval/mod.rs`": the scope adds `eval/mod.rs`, `eval/claims.rs` and the
  places that enumerate exclusion reasons (binding, decision, explain, model/decision, canon-docs
  pages, website).
- `canon check` models invalidation: per-claim exclusion lets the evaluator reach claim
  combinations no check state held (probe: `unreachable-outcome` reported for an outcome `evaluate`
  finds legitimate). The state space gains one two-valued dimension per (rule, evidence dimension
  a claim the rule invalidates reads): upstream moved or not; when moved, that dimension's record is
  excluded for the named claims and every claim built on them. Scope adds `check/`, `eval/read.rs`
  (the IR read-back) and two struct-literal touches.
- As built and accepted: the moved flags are keyed by (upstream artifact, evidence dimension), not
  by rule, because a record stores one revision per upstream artifact and rules on one artifact move
  together; flagged dimensions are those the invalidated claim set (named plus built-on, from
  `eval::invalidated_claims`) reads through its own matches. An absent record adds no states
  (`1 + (2^c - 1) * 2^m` values per dimension); a protocol with no rules keeps its state count.
- Adversary pass 1: a record is listed as `invalidated` only under a claim whose own evidence
  matches would read it; a rule naming a claim with no evidence match of its own is refused as
  `inert-invalidation`. `canon check` lets a moved and a current record of one dimension coexist:
  each present result class carries the set of moved-vectors among its records.
- Adversary pass 2: a claim is evaluated with the records invalidated for it excluded throughout,
  including inside the claims it tests (their reported values stay their own); a record is listed
  under a claim when invalidated for it and read by some match reached from it; `inert-invalidation`
  refuses a rule naming a claim that reaches no evidence match at all. Witnesses prefer current
  records over records observed before an upstream move.
