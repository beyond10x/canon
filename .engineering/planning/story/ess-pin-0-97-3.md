---
format: aep.planning-md/3
id: story:ess-pin-0-97-3
kind: story
status: implemented
title: Canon's ESS pin moves from 0.77.0 to 0.97.3 and the gate stays green
tags:
- from-request
refs:
- provider: host-operator
  reference: ess-0.97.3-upgrade
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: .github/workflows/check.yml
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/canon/tests/ess_gate.rs
- confidence: cited
  path: ess/ess-inputs.yaml
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T13:19:06Z", actor: "human:timo", revision: 7}
- {from: "proposed", to: "active", at: "2026-11-22T13:19:06Z", actor: "human:timo", revision: 8}
- {from: "active", to: "implemented", at: "2026-11-22T17:21:43Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w11"}
---
## Outcome

Canon's ESS pin moves from 0.77.0 to 0.97.3 in every place that holds it, and `task ess-gate` and the whole gate stay green on the new release.

1. **Every pin names 0.97.3.** `ess/ess-inputs.yaml` (`requires: ess 0.97.3`), `.github/workflows/check.yml` (`ESS_VERSION`), `AGENTS.md` (the sentence that names the release), the pin constant of `crates/canon/tests/ess_gate.rs` and the tests and comments that name 0.77.0 (`adversary2_ess_gate.rs`, `ess_commands_match.rs`, `ess_model_matches.rs`, `adversary_ess_commands.rs`, `adversary2_ess_commands.rs`).
2. **No change of Canon's behaviour.** The specification compiles, validates and projects to the same Canon model; `canon` output is unchanged; where a generated or committed ESS artifact changes under 0.97.3, it is regenerated and the change is named.
3. **The two breaking changes of 0.97.3 are checked against Canon's use.** (a) `--component <name>` reports only that component's operations (new `scope:` and `unrealised:` lines); (b) read-grant scenarios no longer copy other views' reads. Canon commits no ESS conformance suite (`scenarios: []` in `ess/ess-inputs.yaml`); the story says which of the two reaches Canon's commands and which does not.

## Why

- Timo, 2026-11-22 14:17 CET, relayed by `host-operator`: "inform all recently active sessions about the ess upgrade" and "all should upgrade". ESS 0.97.3 was released at 13:56 CET (https://github.com/beyond10x/ess/releases/tag/0.97.3; 0.97.2 was tagged and never released: skip it).
- 0.97.3 holds the synthesis performance fixes. Canon's gate runs `ess` through `task ess-gate`; at 0.77.0 the `ess-gate` figure on the test step's own line was 2.2 s in the gate run on `2f3d14b14` and 3.5 s in correction unit 1's run, so no speed gain is expected here; the reason to move is one pin across the host.

## ESS first

No `ess/` declaration changes unless 0.97.3 refuses one. The first commit changes only the pin files and the pin constants and states the red: with 0.97.3 on `PATH` and the pins at 0.77.0, `task ess-gate` and `ess_gate.rs` refuse on the pin (name the output); with the pins moved they pass or name what 0.97.3 refuses.

## Scope

- **In:** Outcome 1 to 3.
- **Out:** any Canon feature; installing the shared binary (`host-operator` replaces `~/.cargo/bin/ess` once); a change of the specification's meaning.
- **Surfaces:** `ess/ess-inputs.yaml`, `.github/workflows/check.yml`, `AGENTS.md`, `Taskfile.yml` (reads the version from `ess/ess-inputs.yaml`), the five test files above, and any `ess/` file 0.97.3 refuses.
- **Confidence:** medium. The jump is 20 releases; the unit measures what 0.97.3 says about the specification.

## Order

- **Depends on:** nothing open. Disjoint from story:evaluate-without-thread-per-call (`crates/canon/src/eval/mod.rs`).
- Runs beside it in wave 2026-11-22-w10 on its own branch; one release carries both (0.11.1, or the number the operator names).

## Acceptance

`task ess-gate` and `task check` pass with ESS 0.97.3 first on `PATH`; `git grep -n "0\.77\.0"` over `ess/`, `.github/`, `Taskfile.yml`, `AGENTS.md` and `crates/` finds only history; the test step stays under 120 s; each pin the unit could not move (a place that has no 0.97.3 download) is named.

## Source

- **Request:** `host-operator`, Timo's order of 2026-11-22 14:17 CET.
- **Canon files read:** `ess/ess-inputs.yaml`, `.github/workflows/check.yml:79`, `AGENTS.md:88`, `crates/canon/tests/ess_gate.rs:18`.
- **Not run:** `ess` 0.97.3 against Canon's specification.
