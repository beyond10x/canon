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

Canon's ESS specification lives under `ess/`. Today it specifies only the `protocol/1` source
model (`crates/canon/src/model/`), with no commands. Atlas ADR 0076 still owes, each with the story
that introduces it: the `canon` CLI's commands (`canon validate` and its successors), the case
snapshot, and the evidence and decision documents.

`ess/` is held to a hard gate (Atlas ADR 0076): `ess specify validate --path ess
--strict-requires`, `ess specify compile --path ess`, `ess verify conform synthesize --path ess`
with 0 refusals, and no `UNMAPPED:` anywhere under `ess/`. `task ess-gate` enforces all four
(`crates/canon/tests/ess_gate.rs`) and runs first in `task check`. `ess/ess-inputs.yaml` pins the
ess release, which `ess specify toolchain which` reports when run from `ess/`; CI installs the same
one. No story in this repository is implemented while the gate is red. The `UNMAPPED:` scan exists because ess
0.52.0 does not see open questions; remove it once the pinned ess release refuses open entries
itself (beyond10x/ess `epic:typed-open-questions`).

The meaning of a protocol stays in Canon's own conformance suite, not in ESS (Atlas ADR 0067 as
amended by ADR 0076).

Spec first, then red, then implement (Atlas ADR 0080, draft): a unit's first commit changes only
the specification (`ess/`, and Canon's own scenario and fixture for the meaning it adds); a named
test fails on that commit and the failing run is recorded; later commits make it pass without
changing the specification. Only a change with no behaviour change is exempt, and its story says
so. Each story's `## ESS first` section names the change and the red test.

The Rust model in `crates/canon/src/model/` is hand-written. `crates/canon/tests/ess_model_matches.rs`
fails, naming the difference, when its fields, field types or variants differ from `ess/`, so a story
that changes the model updates `ess/` in the same change. Each declaration in `ess/` carries a
`# read from: <file>:<line>` comment; keep those current.

## Work

- Planned in the AEP store under `.engineering/`, written only through `aep plan artifact`. Body
  drafts go in `.engineering/drafts/` (ignored).
- Build with `CARGO_TARGET_DIR=$HOME/.cache/b10x-target/canon` (the Taskfile sets it).
- Every commit and push is `b10x-bot[bot]`'s through `b10x-gates bot`; every GitHub write goes
  through `b10x-gates api`.
- Use a managed worktree (`worktree create --repo canon --purpose …`) for changes.
