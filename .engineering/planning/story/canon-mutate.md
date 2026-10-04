---
format: aep.planning-md/3
id: story:canon-mutate
kind: story
status: draft
title: Measure scenario strength with canon mutate
relations:
- decomposes: epic:protocol-analysis
- depends_on: story:semantic-diff
- depends_on: story:conformance-runner
- depends_on: story:action-admissibility
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon-cli/src/main.rs
- confidence: cited
  path: crates/canon-cli/src/mutate.rs
- confidence: cited
  path: crates/canon-cli/tests/canon_mutate.rs
- confidence: cited
  path: crates/canon/src/mutate/
- confidence: cited
  path: fixtures/investigation/mutate/
revision: 2
---
## Outcome

`canon mutate --scenarios <dir> <protocol>` (clap derive) applies five mutation operators to the
protocol: drop a conjunct, `all` → `any`, remove a precondition, remove a capability requirement,
change an evidence result value. Each mutant runs against the given `canon-conformance/1` scenarios;
a mutant no scenario fails is a survivor. Each mutant is classified with the semantic diff
(story:semantic-diff) as weakening or strengthening, and a weakening survivor is reported as a weak
point of the scenario set. Output is deterministic. Part 3 of epic:protocol-analysis.

The scenario directory is an input: the hand-written `conformance/scenarios/` or what
`canon generate` (story:scenario-generation) writes. This story needs neither to exist in a
particular form, so it does not depend on story:scenario-generation.

## ESS first

- Specification change: none in `ess/`. Mutants are `protocol/1` documents and the classification
  is story:semantic-diff's. The first commit is the named test below with its fixture: the
  investigation protocol and two scenario sets under `fixtures/investigation/mutate/`, one lacking a
  witness for one outcome.
- Red on that commit: `mutate_reports_weakening_survivors` fails, because `canon mutate` does not
  exist.

## Scope

- In: the five operators, running mutants through the conformance runner, classification through
  `crates/canon/src/diff/`, the survivor report.
- Out: operators over constructs the kernel does not have yet (independence, evidence order, case
  composition, capability scope); extending story:semantic-diff's categories.
- Surfaces: `crates/canon/src/mutate/` (new), `crates/canon-cli/src/mutate.rs` (new),
  `crates/canon-cli/src/main.rs` (the subcommand variant), `fixtures/investigation/mutate/` (new),
  `crates/canon-cli/tests/canon_mutate.rs` (new).

## Order

depends_on story:semantic-diff (classification, as the epic states), story:conformance-runner
(mutants run through it), and story:action-admissibility (a mutant that removes a precondition or a
capability requirement is killed only by scenarios that compare the `actions` section).

## Acceptance

The named test `mutate_reports_weakening_survivors` (in `crates/canon-cli/tests/canon_mutate.rs`)
passes with four expectations over `fixtures/investigation/mutate/`: every operator yields at least
one mutant and each mutant is labelled weakening or strengthening; with the scenario set that lacks
a witness, the mutant that weakens that outcome's requirement survives and is reported as a weak
point; with the complete set, that mutant is killed; and two runs give identical bytes.

## Source

epic:protocol-analysis part 3; the `ess:hardening` skill's mutation technique; Atlas ADR 0080.
