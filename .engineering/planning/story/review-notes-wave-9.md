---
format: aep.planning-md/3
id: story:review-notes-wave-9
kind: story
status: implemented
title: 'Six notes of the wave 9 review: fix, file or close each'
tags:
- from-finding
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: .github/workflows/check.yml
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/canon/src/check/mod.rs
- confidence: cited
  path: crates/canon/src/diff/mod.rs
- confidence: cited
  path: crates/canon/src/mutate/mod.rs
- confidence: cited
  path: crates/canon/tests/main.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T17:21:52Z", actor: "human:timo", revision: 3, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w11"}
- {from: "proposed", to: "active", at: "2026-11-22T17:21:52Z", actor: "human:timo", revision: 4, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w11"}
- {from: "active", to: "implemented", at: "2026-11-22T21:20:37Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
---
## Outcome

Each note below is fixed, made a story of its own, or closed with a reason. One draft for the six notes, not one story each.

## The notes (review of wave 9 at `f7dcbf11a`)

1. **E3** (`review-result:review-wave-9-eval`), `crates/canon/src/check/mod.rs:857`: `unproduced-evidence` does not name a decision outcome whose `when` reads unproduced evidence. story:decision-condition D4 B kept the readers unchanged.
2. **E4** (same review), `crates/canon/src/check/mod.rs:683`: `canon check` keeps every slice's states and evaluations until it returns; peak memory is the sum of the slices. Not measured.
3. **Model F2** (`review-result:review-wave-9-model`): `canon mutate` makes no mutant at a conditioned decision and its report does not say it skipped the site. Documented and pinned by `mutate_makes_no_mutant_at_a_conditioned_decision` (`crates/canon/src/eval/decision_condition.rs:374`).
4. **Model F4** (same review), `crates/canon/src/diff/mod.rs:840`: adding `when` to a floor's decision is `BREAKING` although it only narrows (story:decision-condition acceptance 8). Design note.
5. **Gate G4** (`review-result:review-wave-9-gate`), `Taskfile.yml:75`: the local test step ran over 120 s under host load (151.1 s at `f7dcbf11a`; 158.5 and 150.8 s in the unit's runs). Gate 2 on `2f3d14b14` measured 55.4 s on the step's own line at load 67 on 20 CPUs. The bound is wall time with `num-cpus` threads on a shared host. Not proven on a CI runner.
6. **Gate G6 and G7** (same review), both INFEASIBLE today: the layout test (`crates/canon/tests/main.rs:117`) does not flag an undeclared `.rs` file in a tests subdirectory, and `canon-docs` has no layout test; a classifier answer outside the three words (`.github/workflows/check.yml:40`) runs no step.

## Acceptance

Each of the six has a recorded outcome: a commit that fixes it, a story id, or a sentence why it stays. For note 5, a CI run of the test step on a runner, with its time, is recorded.

## Source

Close-out of wave 2026-11-20-w9, from the four review results.
