---
format: aep.planning-md/3
id: story:three-valued-claims
kind: story
status: proposed
title: Evaluate claims in three-valued truth
refs:
- provider: taskboard
  reference: C-003
relations:
- decomposes: epic:canon-kernel
- depends_on: story:canon-ir
- depends_on: story:conformance-runner
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:ess-hard-gate
scope:
- confidence: cited
  path: conformance/scenarios/three-valued-claims.yaml
- confidence: cited
  path: crates/canon-cli/
- confidence: cited
  path: crates/canon/src/conform/
- confidence: cited
  path: crates/canon/src/eval/
- confidence: cited
  path: crates/canon/tests/ess_model_matches.rs
- confidence: cited
  path: ess/
- confidence: cited
  path: fixtures/investigation/three-valued-claims.yaml
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:51Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

Canon evaluates every claim of a compiled protocol to `TRUE`, `FALSE` or `UNKNOWN` from an evidence
set, through `canon evaluate --ir --case --evidence` (clap derive) over a pure library function.
`TRUE` means the evidence establishes the predicate, `FALSE` that it contradicts it, and `UNKNOWN`
that it is insufficient to decide; `UNKNOWN` and `FALSE` are never collapsed, and only `TRUE`
satisfies a positive requirement (design § 8). The evaluator reads nothing but its arguments: no
clock, network, filesystem lookup, credentials or implicit `latest` (design § 13).

An evidence match by kind and result evaluates `UNKNOWN` when no record of the kind exists, `FALSE`
when records of the kind exist and none has the result, `TRUE` when records of the kind have only
that result, and — decided here — `UNKNOWN` when records of the kind disagree (one with the result,
one without), because conflicting evidence does not establish the predicate and does not by itself
contradict it.

## Input and output shapes (operator decision 2026-10-04)

This story defines the documents `canon evaluate` reads and writes; later stories extend them and
never replace them.

- `canon-case/1`, the minimal case snapshot: case id, protocol id, and the current revision of each
  declared artifact.
- `canon-evidence/1`, one evidence record: evidence id, kind, result, subject (an artifact id) and
  subject revision. Here subject and subject revision are read and type-checked but do not affect
  evaluation; story:evidence-revision-binding makes them binding.
- `canon-decision/1`, the output envelope: protocol id and revision, case id, and a `claims`
  section with each claim's value.

Extended later by: story:evidence-revision-binding (revision binding), story:obligations
(`obligations`), story:action-admissibility (`actions`, `--authority` reading `canon-authority/1`),
story:outcomes (`outcomes`, case termination field), story:evidence-freshness (`max_age`, the
evaluation instant input), story:explanation (`explanation`).

## Shared surface and order (re-plan 2026-10-04)

This story is the base every evaluator story evaluates on. It owns `crates/canon/src/eval/` while
it runs and may lay it out as it likes; story:evaluator-skeleton, which runs next, splits it into
one file per concept and lands the shared slots, so the later evaluator stories run side by side
rather than in a chain. It also wires evaluate steps into `canon conform run`
(story:conformance-runner), on which it depends.

So that later stories do not change this story's results, its scenario's evidence names declared
artifacts at their current revision.

## Scope

- In: the evaluator, the three input/output shapes above, the `evaluate` subcommand, the variant
  `fixtures/investigation/three-valued-claims.yaml` (a copy of the base: this story needs no
  protocol change; the file exists so its scenario names a fixture no other story edits), the
  scenario file `conformance/scenarios/three-valued-claims.yaml`.
- `crates/canon/tests/ess_model_matches.rs` (added in the re-plan): it asserts the specification
  declares exactly one entity, `Protocol`, so declaring the Case entity changes it.
- Surfaces: `crates/canon/src/eval/`, `crates/canon/src/conform/`, `crates/canon-cli/`, `ess/`,
  `crates/canon/tests/ess_model_matches.rs`, `fixtures/investigation/three-valued-claims.yaml`,
  `conformance/scenarios/three-valued-claims.yaml`.

## Domain relations

- Evidence ↔ Claim: many-to-many; one evidence record may establish, contradict or fail to decide
  several claims, and a claim may draw on several evidence records through its predicate. Stated
  in design § 4.5 and § 12. Not an ess/1 document: Canon opts out of ESS for language semantics
  (AGENTS.md § ESS, Atlas ADR 0067).

## ESS

This story defines the input shapes `canon-case/1` and `canon-evidence/1`, and the output `canon-decision/1`. The case shape brings in the Case entity and its relation to Protocol. It updates `ess/` (domain `canon.protocol`, set up by story:ess-hard-gate) in this same
story. Every new declaration cites the file and line it was read from, and `task ess-gate` stays
green with no `UNMAPPED:` (Atlas ADR 0076). `ess_gate` does not compare `ess/` with the Rust
model, so the review of this story is what checks that the two agree.

## ESS first

- Specification change, first commit: the `canon-case/1` (Case entity and its relation to
  Protocol), `canon-evidence/1` and `canon-decision/1` declarations in `ess/`, together with
  Canon's own semantic specification for this story, the scenario file
  `conformance/scenarios/three-valued-claims.yaml` and its fixture.
- Red on that commit: `ess_model_matches` fails, because it asserts exactly one entity and the
  specification now declares two; and scenario `CANON-CLAIM-001` fails under `canon conform run`,
  because the runner refuses evaluate steps until this story wires `canon evaluate`.

## Acceptance

Conformance scenario `CANON-CLAIM-001` (covers CANON-CLAIM-001 and CANON-CLAIM-002) passes under
`canon conform run`, with four expectations for `explanation.supported`: `UNKNOWN` with no evidence;
`FALSE` with a supporting observation and a falsification attempt whose result is `refuted`; `TRUE`
with a supporting observation and a falsification attempt that `survived`; `UNKNOWN` with a
supporting observation and two falsification attempts, one `survived` and one `refuted`.

## Source

TASKBOARD C-003 (build pack, now Atlas `docs/design/governed-autonomy/TASKBOARD.md`);
`docs/design/canon-protocol-calculus-design.md` § 8, § 9, § 12, § 13, § 14, § 29, § 41 item 4;
CANON-CLAIM-001, CANON-CLAIM-002 (§ 32).
