# AGENTS.md — canon

What Canon is and how to build it is in [README.md](README.md); this file is what an agent changing
it must know. The cross-repository architecture is Atlas ADRs 0066–0075 and Atlas
`docs/design/governed-autonomy/`.

## Serves

- **O2 — decisions as data, with evidence.** Protocol semantics are declared as data and evaluated
  deterministically by the component that owns them.

## Boundary

- Canon owns generic protocol semantics: case, artifact and revision, fact, claim, predicate,
  evidence, obligation, action, authority requirement, outcome, invalidation and recovery, and
  evaluation with explanation (Atlas ADR 0067).
- Canon contains no engineering vocabulary. Engineering protocols belong in `beyond10x/els`.
- Canon does not know AEP, Commission, Loom or any model provider. Nothing here depends on them.

## Rules

- Evaluation is a pure function of protocol IR, case snapshot, evidence set, authority decisions
  and an evaluation instant passed in explicitly. No hidden clock, network, filesystem, credentials
  or "latest" lookups inside the evaluator.
- `UNKNOWN` is not `FALSE`. Missing or stale evidence is never a contradiction.
- The same normalized input always produces the same normalized evaluation, byte for byte.
- Every semantic rule lands with a test in the conformance suite.
- Anything that runs is Rust; command lines use clap derive.

## ESS

Canon opts out of ESS for its language semantics: Canon's own conformance suite is authoritative for
what a protocol means (Atlas ADR 0067). If the Canon toolchain is later specified as software, that
specification lives under `ess/` and covers the toolchain, not the language.

## Work

- Planned in the AEP store under `.engineering/`, written only through `aep plan artifact`. Body
  drafts go in `.engineering/drafts/` (ignored).
- Build with `CARGO_TARGET_DIR=$HOME/.cache/b10x-target/canon` (the Taskfile sets it).
- Every commit and push is `b10x-bot[bot]`'s through `b10x-gates bot`; every GitHub write goes
  through `b10x-gates api`.
- Use a managed worktree (`worktree create --repo canon --purpose …`) for changes.
