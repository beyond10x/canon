---
format: aep.planning-md/3
id: story:conformance-suite
kind: story
status: proposed
title: Run the normative conformance suite
refs:
- provider: taskboard
  reference: C-010
relations:
- decomposes: epic:canon-kernel
- depends_on: story:explanation
- depends_on: story:conformance-runner
- depends_on: story:three-valued-claims
- depends_on: story:evidence-revision-binding
- depends_on: story:obligations
- depends_on: story:action-admissibility
- depends_on: story:outcomes
- depends_on: story:evidence-freshness
- depends_on: story:decision-outcomes
- depends_on: story:invalidation-rules
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: conformance/requirements.yaml
- confidence: cited
  path: crates/canon-cli/
- confidence: cited
  path: crates/canon/src/conform/
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T00:00:50Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":5}}}
---
## Outcome

Canon has a normative conformance suite that is authoritative for what a protocol means (Atlas
ADR 0067): a catalogue, `conformance/requirements.yaml`, of the normative requirements the kernel
implements (the `CANON-*` identifiers of design § 32 plus those the scenarios add, such as
`CANON-OBLIGATION-001` and `CANON-EXPLAIN-001`), each covered by one or more scenarios in the
registry `conformance/scenarios/`. `canon conform run --requirements conformance/requirements.yaml`
checks coverage and determinism over that registry: each scenario is evaluated twice and once more
from an equivalent input in permuted order, and an implemented requirement without a passing
scenario, or a scenario whose output differs on rerun or under permuted input, fails the run
naming it. `task check` runs it with the flag.

## Additive to the runner (operator decision 2026-10-04)

The coverage and permutation checks are additive: they apply only when `--requirements <file>` is
given. Without the flag, `canon conform run` behaves exactly as story:conformance-runner defines it,
so that story's acceptance is unaffected by this one. The flag lives in `crates/canon-cli`, which is
why this story's surfaces include it and why story:semantic-diff, which also changes
`crates/canon-cli`, depends_on this story.

## Operator decision (2026-10-04)

The scenario format, `canon conform run` and the registry moved to story:conformance-runner. This
story keeps only "every implemented `CANON-*` requirement has a passing scenario, byte-identical on
rerun and with input reordered", and depends_on the stories whose requirements it covers:
story:three-valued-claims (CANON-CLAIM-001, -002), story:evidence-revision-binding
(CANON-EVIDENCE-001), story:obligations (CANON-OBLIGATION-001), story:action-admissibility
(CANON-AUTHORITY-001), story:outcomes (CANON-OUTCOME-001), story:evidence-freshness
(CANON-EVIDENCE-002), story:explanation (CANON-EXPLAIN-001, CANON-DETERMINISM-001), and
story:conformance-runner.

## Scope

- Out: CANON-INDEPENDENCE-001 and recovery rules, which no TASKBOARD Canon item implements, and
  CANON-INVALIDATION-001, which waits on `decision-blocker:upstream-revision-binding`. The
  catalogue lists them as not yet implemented rather than omitting them.
- Out: the scenario format and runner (story:conformance-runner); `canon conform synthesize`
  (design § 29).
- Surfaces: `crates/canon/src/conform/`, `crates/canon-cli/`, `conformance/requirements.yaml`,
  `Taskfile.yml`.

## Acceptance

The named test `conform_run_checks_requirements` (in `crates/canon-cli/tests/`) passes with three
expectations for `canon conform run --requirements`: given `conformance/requirements.yaml` over the
real registry `conformance/scenarios/`, it exits 0; given a fixture catalogue listing an
implemented requirement that no scenario covers, it exits non-zero naming that requirement; and
over a fixture scenario whose output differs under permuted input, it exits non-zero naming that
scenario.

## Source

TASKBOARD C-010 (build pack, now Atlas `docs/design/governed-autonomy/TASKBOARD.md`);
`docs/design/canon-protocol-calculus-design.md` § 28, § 29, § 32, § 41 item 14; AGENTS.md § Rules;
build pack `ROADMAP.md` Phase 1 exit criterion.
