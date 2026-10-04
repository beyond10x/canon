---
format: aep.planning-md/3
id: story:evidence-freshness
kind: story
status: implemented
title: Expire evidence against the evaluation instant
refs:
- provider: taskboard
  reference: C-008
relations:
- decomposes: epic:canon-kernel
- depends_on: story:conformance-runner
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:ess-hard-gate
- depends_on: story:three-valued-claims
- depends_on: story:evaluator-skeleton
scope:
- confidence: cited
  path: conformance/scenarios/evidence-freshness.yaml
- confidence: cited
  path: crates/canon-cli/src/lib.rs
- confidence: cited
  path: crates/canon-docs/
- confidence: cited
  path: crates/canon/src/eval/claims.rs
- confidence: cited
  path: crates/canon/src/eval/evidence.rs
- confidence: cited
  path: crates/canon/src/eval/freshness.rs
- confidence: cited
  path: crates/canon/src/eval/read.rs
- confidence: cited
  path: crates/canon/src/ir/
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/validate/
- confidence: cited
  path: ess/
- confidence: cited
  path: fixtures/investigation/evidence-freshness.yaml
- confidence: cited
  path: website/
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:50Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-04T05:19:06Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-10-04T06:13:34Z", actor: "human:timo", revision: 14, decided_on: {"recorded":{"test_result":1,"review_outcome":6,"verification":1}}}
---
## Outcome

Evidence can expire. A protocol can declare a maximum age for evidence; at evaluation, evidence
older than that age relative to the evaluation instant passed in is inapplicable, and a claim left
without applicable evidence is `UNKNOWN`, never `FALSE` (design § 8, CANON-EVIDENCE-002). The
instant is an explicit input; the evaluator reads no clock. Where in `protocol/1` the `max_age`
field sits (on the evidence-kind declaration or on an evidence match) is this story's decision,
recorded in the source model, validator, IR and its conformance scenario.

## Extends (operator decision 2026-10-04)

- `protocol/1` and `canon-ir/1` gain `max_age`.
- `canon evaluate` gains the evaluation instant input, `--at`.
- `canon-evidence/1`, defined by story:three-valued-claims, gains `observed_at`, which age is
  measured from.
- `canon-decision/1`: expired evidence is added, with reason `expired`, to the same excluded-evidence
  list in each `claims` entry that story:evidence-revision-binding introduced. story:explanation
  traces that list and adds no reason of its own.

## Shared surface and order (re-plan 2026-10-04)

The evaluator chain of the earlier plan is gone. story:evaluator-skeleton lands the `--at` flag
(read and passed through unparsed), the freshness exclusion stage in `eval/freshness.rs` and the
`excluded_evidence` slot with its `expired` reason, so this story runs beside
story:evidence-revision-binding, story:obligations, story:action-admissibility and story:outcomes.
It still changes the source model (`max_age`, whose position is this story's decision and is not
settled anywhere the skeleton could read it), so it heads the three stories that change
`model/`, `validate/`, `ir/` and `ess/`: this story, then story:decision-outcomes, then
story:invalidation-rules.

- Kept: depends_on story:conformance-runner (its acceptance is a scenario), story:ess-hard-gate
  (`ess/`).
- Added: depends_on story:three-valued-claims (it changes which evidence that story's claim
  evaluation sees; was transitive before), story:evaluator-skeleton (flag, stage, slot, and the
  skeleton's own `model/`/`ess/` change lands first).
- Removed: depends_on story:outcomes (ordering only, on `eval/` and `crates/canon-cli/`, now split);
  depends_on story:evidence-revision-binding (it only needed the excluded-evidence list to exist,
  which the skeleton now lands; freshness and revision binding are independent exclusions).
- `observed_at` is optional on `canon-evidence/1`: evidence without it is never expired, so no
  earlier scenario changes (re-plan rule, story:evaluator-skeleton).

## Scope

- Fixture (operator decision 2026-10-04): the variant `fixtures/investigation/evidence-freshness.yaml`
  is the base plus a maximum age on the evidence `explanation.supported` draws on. The base is not
  edited.
- Out: invalidation of claims when a bound upstream artifact changes (CANON-INVALIDATION-001),
  which story:invalidation-rules builds. Invalidation of evidence bound to the subject artifact’s
  own superseded revision is C-004.
- Surfaces: `crates/canon/src/model/`, `crates/canon/src/validate/`, `crates/canon/src/ir/`,
  `ess/`, `crates/canon/src/eval/freshness.rs`, `crates/canon/src/eval/evidence.rs`,
  `fixtures/investigation/evidence-freshness.yaml`, `conformance/scenarios/evidence-freshness.yaml`.

## ESS

This story changes the `protocol/1` source model (`max_age`) and the `canon-evidence/1` input (`observed_at`). It updates `ess/` (domain `canon.protocol`, set up by story:ess-hard-gate) in this same
story. Every new declaration cites the file and line it was read from, and `task ess-gate` stays
green with no `UNMAPPED:` (Atlas ADR 0076). `ess_gate` does not compare `ess/` with the Rust
model, so the review of this story is what checks that the two agree.

## ESS first

- Specification change, first commit: `max_age` on the `protocol/1` declaration this story chooses
  (evidence kind or evidence match) in `ess/domains/protocol.yaml`, and the optional `observed_at`
  on the `canon-evidence/1` declaration in `ess/`; with them, the scenario file
  `conformance/scenarios/evidence-freshness.yaml` and its fixture.
- Red on that commit: `ess_model_matches` fails naming `max_age`, because the Rust model does not
  have it; and scenario `CANON-EVIDENCE-002` fails, because the fixture's `max_age` does not parse
  and the skeleton refuses `--at`.

## Acceptance

Conformance scenario `CANON-EVIDENCE-002` passes under `canon conform run` over
`fixtures/investigation/evidence-freshness.yaml`, with two expectations and the evaluation instant
the only input that changes: `explanation.supported` is `TRUE` at an instant within its evidence's
maximum age, and `UNKNOWN`, not `FALSE`, at an instant past it.

## Source

TASKBOARD C-008 (build pack, now Atlas `docs/design/governed-autonomy/TASKBOARD.md`);
`docs/design/canon-protocol-calculus-design.md` § 8, § 9, § 13, § 41 items 6 and 12;
CANON-EVIDENCE-002, CANON-INVALIDATION-001 (§ 32).

### Coordinator and adversary decisions (wave 2026-10-04-w7)

- `max_age` is declared on the evidence kind. A record expires when instant minus `observed_at` is
  more than `max_age`; at exactly `max_age` it still applies; with no `--at` or no `observed_at`,
  nothing expires.
- Pass 1: the published schemas give `Age` and `Instant` their own patterns, with calendar validity
  checked by Canon (fixed); the refusal message states "without leading zeros" (fixed); the ESS
  comment describes the strict UTC form (fixed); edits to shared files were authorised (no-op).
- Pass 2: an over-long age has its own message (fixed); an unreadable `max_age` in a caller-built IR
  is refused, never silently disabling expiry (fixed); the merge with story:evidence-revision-binding
  is resolved by the coordinator (fixed at merge).
