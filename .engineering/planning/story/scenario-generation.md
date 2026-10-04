---
format: aep.planning-md/3
id: story:scenario-generation
kind: story
status: draft
title: Generate minimal witness scenarios with canon generate
relations:
- decomposes: epic:protocol-analysis
- depends_on: story:canon-check
- depends_on: story:conformance-runner
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon-cli/src/generate.rs
- confidence: cited
  path: crates/canon-cli/src/main.rs
- confidence: cited
  path: crates/canon-cli/tests/scenario_generation.rs
- confidence: cited
  path: crates/canon/src/generate/
- confidence: cited
  path: fixtures/investigation/generate/
revision: 2
---
## Outcome

`canon generate --out <dir> <protocol>` (clap derive) writes `canon-conformance/1` scenarios: one
minimal witness evidence set per outcome and per blocked action, found over the state space
story:canon-check enumerates. Minimal means removing any one evidence record from the witness
changes the expected result. `canon conform run` (story:conformance-runner, implemented) executes
the generated scenarios unchanged. Output is deterministic. Part 2 of epic:protocol-analysis.

## ESS first

- Specification change: none in `ess/`. `canon-conformance/1` is the runner's existing format, and
  the state space is story:canon-check's. The first commit is the named test below with its
  expected scenario files for the base investigation protocol under
  `fixtures/investigation/generate/`.
- Red on that commit: `generated_scenarios_witness_every_outcome` fails, because `canon generate`
  does not exist.

## Scope

- In: witness search, minimisation, scenario writing, the subcommand.
- Out: mutation (story:canon-mutate); scenarios for obligations' discharge, which no part of the
  epic asks for.
- Surfaces: `crates/canon/src/generate/` (new), `crates/canon-cli/src/generate.rs` (new),
  `crates/canon-cli/src/main.rs` (the subcommand variant), `fixtures/investigation/generate/` (new),
  `crates/canon-cli/tests/scenario_generation.rs` (new).

## Order

depends_on story:canon-check (it reuses the state-space enumerator) and story:conformance-runner
(the format it writes and the runner its acceptance uses). No file under `crates/canon/src/eval/`.

## Acceptance

The named test `generated_scenarios_witness_every_outcome` (in
`crates/canon-cli/tests/scenario_generation.rs`) passes with four expectations over the base
investigation protocol: `canon generate` writes exactly one scenario per declared outcome and per
action blocked in some state; each matches the committed expectation under
`fixtures/investigation/generate/` byte for byte; `canon conform run` over the output directory
passes every scenario; and for each scenario, removing any one evidence record makes
`canon conform run` fail that scenario.

## Source

epic:protocol-analysis part 2; Atlas ADR 0080.
