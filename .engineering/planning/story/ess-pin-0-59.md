---
format: aep.planning-md/3
id: story:ess-pin-0-59
kind: story
status: implemented
title: Hold the specification to ess 0.59.0
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
scope:
- confidence: cited
  path: .github/workflows/check.yml
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: crates/canon/tests/adversary2_ess_gate.rs
- confidence: cited
  path: crates/canon/tests/ess_gate.rs
- confidence: cited
  path: crates/canon/tests/ess_model_matches.rs
- confidence: cited
  path: ess/ess-inputs.yaml
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-13T11:05:08Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-13T11:05:08Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-13T12:38:03Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Canon's specification is held to ess 0.59.0, the newest ess release: `ess/ess-inputs.yaml`
requires it, CI installs it (version and SHA-256 of the
`ess-0.59.0-x86_64-unknown-linux-gnu.tar.gz` asset from the release's `SHA256SUMS`), and the gate
tests that name the pin follow it. Under the 0.53.0 pin an installed 0.59.0 dispatcher delegated to
0.53.0 and refused the pin under `--strict-requires` when 0.53.0 was absent.

## ESS first

No behaviour change: the specification sources are unchanged and validate, compile and synthesize
(0 refusals) under 0.59.0 as they did under 0.53.0. Exempt from the red-test rule.

## Scope

- In: `ess/ess-inputs.yaml`, `.github/workflows/check.yml`, `crates/canon/tests/ess_gate.rs`,
  `crates/canon/tests/adversary2_ess_gate.rs`, `crates/canon/tests/ess_model_matches.rs`,
  `AGENTS.md`.
- Out: removing the `UNMAPPED:` scan, which waits on the pinned ess refusing open entries itself.

## Acceptance

`ess specify validate --path ess --strict-requires` with ess 0.59.0 reports the specification
valid; `cargo test -p b10x-canon --test ess_gate --test ess_model_matches --test adversary2_ess_gate`
passes; CI's `task check` installs ess 0.59.0 and passes.
