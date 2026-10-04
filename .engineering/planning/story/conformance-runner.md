---
format: aep.planning-md/3
id: story:conformance-runner
kind: story
status: proposed
title: Run conformance scenarios with canon conform run
refs:
- provider: taskboard
  reference: C-010
relations:
- decomposes: epic:canon-kernel
- depends_on: story:canon-ir
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon-cli/
- confidence: cited
  path: crates/canon/src/conform/
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:50Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome

Canon has a conformance scenario format and a runner, so every story whose acceptance is a named
conformance scenario has something to write it in and run it with. A scenario file holds one named
scenario: its id, the `CANON-*` requirements it covers, the protocol fixture it compiles, and an
ordered list of steps, each giving its inputs and its expected result. A step is either a compile
step (expected `canon-ir/1`) or an evaluate step (case snapshot, evidence set, optional authority
decisions and evaluation instant; expected a `canon-decision/1` document or a refusal naming an
identifier). `canon conform run` (clap derive, in `crates/canon-cli`) runs every scenario in the
registry, compares each step's output byte for byte with its expectation, exits 0 when every
scenario passes, and exits non-zero naming each failing scenario and step otherwise.

The registry is the directory `conformance/scenarios/`: one file per story, named
`<story-name>.yaml`, read in sorted order. There is no index file, so adding a scenario edits no
shared file.

Evaluate steps call the library entry point story:three-valued-claims provides. Until it lands, the
runner refuses an evaluate step naming it as unsupported; story:three-valued-claims wires it.

## Operator decision (2026-10-04)

This story was split out of story:conformance-suite so the scenario format and runner exist before
the stories whose acceptance names a scenario. It owns the scenario file format, `canon conform
run` and the registry under `conformance/`. story:conformance-suite keeps only the coverage and
determinism check over the finished registry, applied only when `--requirements` is given, so
nothing this story's acceptance checks changes when it lands.

## Scope

- In: the scenario format, `canon conform run`, the registry convention.
- Out: the requirement catalogue, coverage check and rerun/permutation determinism check
  (story:conformance-suite); `canon conform synthesize` (design § 29).
- Surfaces: `crates/canon/src/conform/`, `crates/canon-cli/`.

## Acceptance

The named test `conform_run_reports_each_scenario` (in `crates/canon-cli/tests/`) passes with five
expectations over two scenario directories: over an all-passing directory of one compile scenario
whose expected `canon-ir/1` for the base investigation fixture matches, `canon conform run` reports
that scenario passed and exits 0; over a mixed directory of three files (that passing scenario, a
compile scenario whose expected IR differs, and a file that does not parse as a scenario) it
reports the differing scenario failed by its id and step, reports the unparseable file unreadable
by its path, exits non-zero, and prints byte-identical output across two runs.

## Source

TASKBOARD C-010 (build pack, now Atlas `docs/design/governed-autonomy/TASKBOARD.md`);
`docs/design/canon-protocol-calculus-design.md` § 28, § 29, § 32;
`review-result:canon-kernel-design-r1` (conformance-suite finding).
