---
format: aep.planning-md/3
id: story:generate-rollback-in-private-staging
kind: story
status: active
title: canon generate stages a run in a private directory so rollback never removes another writer's file
tags:
- from-finding
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon-cli/src/generate.rs
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T21:20:34Z", actor: "human:timo", revision: 3, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
- {from: "proposed", to: "active", at: "2026-11-22T21:20:34Z", actor: "human:timo", revision: 4, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
---
## Outcome

`canon generate` stages each run in a private directory, so a rollback never removes a file another process put in the shared output directory.

## Why

- Review of tag 0.10.0 (security): in a shared output directory, cleanup removes files by pathname. If another process replaces a generated file before rollback, cleanup removes the replacement.
- On main (0.12.0, `crates/canon-cli/src/generate.rs`, byte-identical to 0.10.0): the run records what it created (`file`, `directory`) and `remove` (about line 138) deletes those paths last first.

## Scope

- **In:** write the run's files into a private staging directory created by the run (same filesystem as the output directory, created with a name no other process uses, mode 0700), and move them into place only when the whole run succeeded; the rollback removes the private directory and nothing in the output directory except what the run itself moved there and that still is what it moved (compare a file's identity before removing it). A test replaces a generated file between creation and rollback and holds that the replacement stays. The bytes of everything `canon generate` writes unchanged.
- **Out:** a change of what is generated or where.
- **Surfaces:** `crates/canon-cli/src/generate.rs`, `crates/canon-cli/tests/`.

## Acceptance

- The replacement test is red before and green after, output shown.
- `canon generate` output on every generating fixture is byte-identical to 0.12.0; a failed run leaves the output directory as it was.

## Source

- **Request:** Timo, 2026-11-22 about 22:10 CET, typed (see story:conform-compiles-the-fixture-once).
