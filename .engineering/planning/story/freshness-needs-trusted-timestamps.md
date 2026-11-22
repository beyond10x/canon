---
format: aep.planning-md/3
id: story:freshness-needs-trusted-timestamps
kind: story
status: active
title: canon evaluate has an opt-in strict freshness mode for undated, future-dated and missing-at evidence
tags:
- from-finding
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon-cli/src/lib.rs
- confidence: cited
  path: crates/canon/src/eval/freshness.rs
- confidence: cited
  path: crates/canon/src/eval/mod.rs
- confidence: cited
  path: ess/domains/cli.yaml
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T21:20:33Z", actor: "human:timo", revision: 6, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
- {from: "proposed", to: "active", at: "2026-11-22T21:20:33Z", actor: "human:timo", revision: 7, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
---
## Outcome

A caller that relies on freshness can have Canon refuse, rather than silently accept, evidence of a kind with `max_age` that is undated or dated after the evaluation instant, and an evaluation of a protocol with such a kind that is given no `--at`.

## Why

- Review of tag 0.10.0 (security): the `max_age` rule leaves undated and future-dated evidence eligible, and omitting `--at` skips expiry altogether. Callers relying on freshness need both timestamps and a bound on clock skew.
- On main (0.12.0, `crates/canon/src/eval/freshness.rs`, byte-identical to 0.10.0): `exclude` returns an empty set when `at` is absent, and counts a record as expired only when `max_age` and an observed time both exist and `at - observed > max_age` (a future-dated record gives a saturated 0 and stays eligible).

## Scope

- **In:** an opt-in strict freshness mode. Existing default behaviour, `canon-decision/1` bytes and every conformance scenario stay unchanged. In the strict mode an undated record of a `max_age` kind, a record dated after `at` (by more than an allowed skew the caller names, default 0) and a missing `at` for a protocol with a `max_age` kind each give a named refusal or a named exclusion reason. The unit chooses refusal or exclusion by which the module docs lean to, and says so ("decided by the unit"). The strict mode is reachable from `canon evaluate` and documented in the module docs, the CLI help and the generated reference (run `task docs-generate`).
- **Out:** a change of the default; a change to `protocol/1` or `canon-case/1`.
- **Surfaces:** `crates/canon/src/eval/freshness.rs`, `crates/canon/src/eval/mod.rs` (a `Supplied` field), `crates/canon-cli/src/lib.rs`, `ess/domains/cli.yaml`.

## Acceptance

- Tests: undated, future-dated (skew 0 and skew allowed) and missing `--at` are each accepted by default (as today, pinning it) and refused or excluded in the strict mode, with the output shown; every existing test unchanged.
- `canon evaluate --help` names the mode; the generated reference is current (`task docs-check`).

## Source

- **Request:** Timo, 2026-11-22 about 22:10 CET, typed (see story:conform-compiles-the-fixture-once). The opt-in shape was chosen by the coordinator so that no existing caller changes; reversible.
