---
format: aep.planning-md/3
id: story:scenario-generation-assertions
kind: story
status: active
title: The scenario-generation test catches a witness in a state that is not legitimate and a refused fixture that generates
tags:
- from-finding
refs:
- provider: decision
  reference: D122
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon/src/generate/mod.rs
- confidence: cited
  path: crates/canon/tests/adversary_scenario_generation.rs
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T17:21:52Z", actor: "human:timo", revision: 3, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w11"}
- {from: "proposed", to: "active", at: "2026-11-22T17:21:52Z", actor: "human:timo", revision: 4, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w11"}
---
## Outcome

The scenario-generation test catches the two gaps the generator study of wave 9 found, and runs within the test budget.

## Finding

From the generator study of wave 9 (D122; branch `study/generation-tests`, merged in `f7dcbf11a`; commits `c499c378b`, `4d77f65d3`, `3e8559225`):

- **(a)** A mutant that lets an outcome be witnessed in a state where it is not legitimate (mutant M3 of the study) survives `every_generated_witness_passes_and_is_minimal_over_every_investigation_protocol` (`crates/canon/tests/adversary_scenario_generation.rs`): `1 passed`, 320.70 s. Mutants 1 and 2 were caught (158.05 s, 173.34 s).
- **(b)** A fixture on the test's list of expected refusals that starts to generate is not flagged. `3e8559225` added three refused fixtures of story:decision-condition to that list.
- **Time.** `c499c378b` cut the test's state evaluations from 144,961 to 45,499 and its time from 249 to 544 s to 89 to 201 s (loaded host). It is still killed at 60 s in the slow lane (gate 2 on `2f3d14b14`, 10 of 63 killed). The study's option: commit each fixture's scenarios with a digest of their sources; the gate lane runs the committed scenarios and the removals; the slow lane regenerates and compares bytes. Also: compile each fixture once and profile the test.
- **Vocabulary.** Fixtures under `fixtures/investigation/` use engineering words (`test_result`, `merge`), which `AGENTS.md` § Boundary excludes without a fixture exception (`review-result:adversary-revision-ordering-pass-1`, its sixth finding).

## Acceptance

Mutant M3 fails the test; a fixture moved off the refusal list without generating, or onto it while generating, fails the test; the gate-lane part runs under 60 s; the vocabulary question has an answer in `AGENTS.md` or the fixtures.

## Source

Close-out of wave 2026-11-20-w9. The study's own report is not in the repository; the numbers are from its logs in the coordinator's scratch (`w10/genstudy/`).
