---
format: aep.planning-md/3
id: story:decision-outcomes
kind: story
status: implemented
title: Decision-based outcomes
refs:
- provider: taskboard
  reference: C-007
relations:
- decomposes: epic:canon-kernel
- depends_on: story:conformance-runner
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:ess-hard-gate
- depends_on: story:outcomes
- depends_on: story:evaluator-skeleton
- depends_on: story:evidence-freshness
scope:
- confidence: cited
  path: conformance/scenarios/decision-outcomes.yaml
- confidence: cited
  path: crates/canon-cli/src/evaluate.rs
- confidence: cited
  path: crates/canon-cli/src/lib.rs
- confidence: cited
  path: crates/canon-cli/src/main.rs
- confidence: cited
  path: crates/canon/src/conform/mod.rs
- confidence: cited
  path: crates/canon/src/eval/decisions.rs
- confidence: cited
  path: crates/canon/src/eval/mod.rs
- confidence: cited
  path: crates/canon/src/eval/outcomes.rs
- confidence: cited
  path: crates/canon/src/ir/
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/validate/
- confidence: cited
  path: crates/canon/tests/ess_model_matches.rs
- confidence: cited
  path: ess/
- confidence: cited
  path: fixtures/investigation/decision-outcomes.yaml
- confidence: cited
  path: website/
revision: 17
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:50Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-10-04T06:15:14Z", actor: "human:timo", revision: 13}
- {from: "active", to: "implemented", at: "2026-10-04T07:09:00Z", actor: "human:timo", revision: 17, decided_on: {"recorded":{"test_result":1,"review_outcome":6,"verification":1}}}
---
## Outcome

Canon gains a decision construct (operator, 2026-10-04: decision-blocker:outcome-decision-source).
A protocol may declare that an outcome requires an explicit decision rather than a claim (the
design § 12 `inconclusive` example; `declined`, `abandoned` in § 4.6). Decisions are a new
evaluation input, `canon-decisions/1`, each naming the outcome, the deciding principal and the case
revision it was taken at; the source model, validator and IR gain `requires: decision: <name>`.
Completes the decision half of TASKBOARD C-007.

## Extends

`canon-decision/1` `outcomes` entries report a decision-based outcome as `legitimate` only with a
matching decision for the current case revision, and as `blocked` naming the missing decision otherwise.

## Shared surface and order (re-plan 2026-10-04)

story:evaluator-skeleton lands `eval/decisions.rs` and passes a decisions input through the
pipeline unparsed, so this story does not touch `eval/mod.rs` or `canon-decision/1`. It parses
`canon-decisions/1` in `decisions.rs`, extends outcome evaluation in `eval/outcomes.rs`, and adds
the `canon evaluate` flag that reads the decisions file in `crates/canon-cli/src/evaluate.rs` (its
name is this story's; the skeleton could not land it). The shape of `requires: decision:` and of
`canon-decisions/1` is not settled anywhere the skeleton could read it, so this story still changes
`model/`, `validate/`, `ir/` and `ess/`, which story:evidence-freshness and
story:invalidation-rules change too. It runs between them, before story:invalidation-rules,
because story:explanation needs it and does not need story:invalidation-rules.

- Kept: depends_on story:conformance-runner (its acceptance is a scenario), story:ess-hard-gate
  (`ess/`).
- Added: depends_on story:outcomes (it extends that story's outcome evaluation and its `outcomes`
  entries; was transitive through the chain before), story:evaluator-skeleton (file, input hook),
  story:evidence-freshness (shared `model/`, `validate/`, `ir/`, `ess/`: ordering only, because
  those files are not split).
- Removed: depends_on story:invalidation-rules (ordering only; the order of the two is reversed,
  see above).

## Scope

- Surfaces: `crates/canon/src/model/`, `crates/canon/src/validate/`, `crates/canon/src/ir/`,
  `ess/`, `crates/canon/src/eval/decisions.rs`, `crates/canon/src/eval/outcomes.rs`,
  `crates/canon-cli/src/evaluate.rs`, `fixtures/investigation/decision-outcomes.yaml`,
  `conformance/scenarios/decision-outcomes.yaml` (the earlier scope named the whole
  `conformance/scenarios/` directory; this story writes one file in it, and the earlier scope
  omitted the CLI flag the new input needs).

## ESS

This story changes the `protocol/1` source model (`requires: decision:`) and adds the `canon-decisions/1` input. It updates `ess/` (domain `canon.protocol`, set up by story:ess-hard-gate) in this same
story. Every new declaration cites the file and line it was read from, and `task ess-gate` stays
green with no `UNMAPPED:` (Atlas ADR 0076). `ess_gate` does not compare `ess/` with the Rust
model, so the review of this story is what checks that the two agree.

## ESS first

- Specification change, first commit: the decision requirement on `canon.protocol.Outcome`
  (`requires: decision: <name>`) in `ess/domains/protocol.yaml` and the `canon-decisions/1`
  declaration in `ess/`; with them, the scenario file `conformance/scenarios/decision-outcomes.yaml`
  and its fixture.
- Red on that commit: `ess_model_matches` fails naming the outcome requirement, because the Rust
  model does not have the decision variant; and scenario `CANON-OUTCOME-002` fails, because the
  fixture's `requires: decision:` does not parse.

## Acceptance

Scenario `CANON-OUTCOME-002` over `fixtures/investigation/decision-outcomes.yaml` passes with three
expectations: `inconclusive` is `blocked` naming `explicitly_inconclusive` with no decision input;
`legitimate` with a `canon-decisions/1` decision at the current case revision; `blocked` again when
that decision names a superseded case revision.

## Source

Design § 4.6, § 12; decision-blocker:outcome-decision-source (cleared 2026-10-04); TASKBOARD C-007.

### Coordinator and adversary decisions (wave 2026-10-04-w8)

- Spec (phase 1): a decision requirement is its own variant of an outcome requirement, standing alone;
  `canon-case/1` gains an optional `revision`; a decision applies only when its name, outcome and
  case revision all match; an outcome without one is blocked with `{decision, present: false}`.
- decision-blocker:terminated-case-reevaluation cleared with option A: the illegitimate-termination
  refusal stays.
- Pass 1: the documents reference lists `canon-decisions/1` (fixed); `Revision` covers the case's
  own revision (fixed); a decided outcome records `decided_by` {decision, principal} (fixed, spec
  change found by review); a duplicate entry and a decision its outcome does not require are refused
  (fixed); an outcome cannot require both a decision and a claim (no-op, as designed).
- Open: if recording a termination advances the case revision, a decision taken before termination
  no longer matches the terminated snapshot. Who assigns case revisions is not specified.
- Pass 2: the documents, status and concept pages name `decided_by`, `canon-decisions/1` and
  decision requirements (fixed); the decisions schema states `uniqueItems` (fixed); `decided_by` records
  every principal whose decision applied, sorted (fixed, design § 37); a missing key reading as an
  explicit null is pre-existing and moves to story:review-hardening-w7 (escalated).
