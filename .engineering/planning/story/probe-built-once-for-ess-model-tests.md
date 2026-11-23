---
format: aep.planning-md/3
id: story:probe-built-once-for-ess-model-tests
kind: story
status: draft
title: The four adversary2_ess_model tests read one probe built once per run
relations:
- decomposes: epic:canon-kernel
revision: 1
---
## Outcome

The four `adversary2_ess_model` tests read one probe package built once per test run, so none of them starts a cargo build and the four together stay under 10 s.

## Why

- Each of the four tests runs `cargo test --offline --no-run --test ess_model_matches` on a probe package under `target/tmp/adversary2-ess-model`, in its own nextest process. Four processes wait on cargo's one build lock on that directory.
- Wave 12 gate, 2026-11-22: the test step failed 5 times on these four (killed at 60 s) and passed on the third rerun at 58.5 s with the four at 47 s each. The budget in the folder's `AGENTS.md` is a test step of at most 120 s and no test over 60 s; an expensive step per test is the named mistake.
- The review of wave 12 skipped these four for the same reason.

## Scope

- **In:** build the probe once (a setup step before the tests, or a cache on disk under `target/` keyed by the digest of its sources) and make the four tests read the result.
- **Out:** what the tests assert.
- **Surfaces:** `crates/canon/tests/adversary2_ess_model.rs`, `.config/nextest.toml`.

## Acceptance

`cargo nextest run -p b10x-canon -E 'test(/^adversary2_ess_model::/)'` on a cold `target/tmp` ends in under 120 s with the 4 tests each under 10 s after the one build, and a probe with a changed source is rebuilt (a test shows both).

## Source

- **Request:** wave 2026-11-22-w12 gate and the folder rule "Tests stay fast".
