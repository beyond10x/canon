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

Canon's ESS specification lives under `ess/`. It specifies the `protocol/1` source model
(`crates/canon/src/model/`), the evaluation documents `canon-case/1` (the Case entity),
`canon-evidence/1`, `canon-decisions/1` and `canon-decision/1` (story:three-valued-claims,
story:decision-outcomes) and `canon-authority/1` (story:review-hardening-w7) in the domain
`canon.protocol`, and the `canon-properties/1` document
`canon check` reads (story:canon-check) in the domain `canon.check`, with no commands. Atlas ADR
0076 still owes the `canon` CLI's commands (`canon validate` and its successors), with the story
that introduces them.

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

The `b10x-canon-expr` crate owns generic catalog expressions. `ess/domains/expr.yaml` specifies
its wire types; `crates/canon-expr/tests/model_matches.rs::every_wire_declaration_matches_ess`
checks every field, field type and variant against the Rust model, with mutation controls.
Native checked types deliberately continue Canon's parity-backed model convention; ESS structural
Rust generation is available, and is not represented as a missing capability.
`task ess-gate` runs both model parity guards. CI and the manifest select ESS 0.53.0.

Keep the Logos lexer and Pratt parser, checker/planner, exact-number implementation and pure
evaluator separate. Catalog recipes and observation providers extend names, never parser
operators. `crates/canon-expr/tests/conformance.rs::complete_three_valued_tables` holds Boolean
semantics; `generated_parse_format_parse_and_malformed_corpus` holds parser/formatter behavior.
`crates/canon-expr/tests/adversary.rs::checked_optional_absence_plan_roundtrips` holds retained
context schemas; `acquired_argument_cannot_exceed_value_payload_budget` holds composed argument
bounds. Every intermediate value is validated. Do not add IO or implicit time to this crate.

Expression contracts and resource limits in `website/docs/reference/assertions.md` are generated
by `canon-docs` from the embedded ESS model and library constants. Update the concept guide with
compatibility decisions and keep unreleased capability claims explicit.

- Planned in the AEP store under `.engineering/`, written only through `aep plan artifact`. Body
  drafts go in `.engineering/drafts/` (ignored).
- Build with `CARGO_TARGET_DIR=$HOME/.cache/b10x-target/canon` (the Taskfile sets it).
- Every commit and push is `b10x-bot[bot]`'s through `b10x-gates bot`; every GitHub write goes
  through `b10x-gates api`.
- Use a managed worktree (`worktree create --repo canon --purpose …`) for changes.

## Releases

Releases are source releases at bare-version tags, starting with `0.1.0`. Update the workspace
version, Cargo.lock and CHANGELOG.md, refresh installation/status prose and regenerate documentation.
Update the expected decision provenance version in `conformance/scenarios/explanation.yaml` and
`crates/canon-cli/tests/adversary_skel_cli.rs`; keep their other exact-output expectations unchanged.
Land the bot-authored preparation on main with `task check` and CI green, then create an annotated
bot tag on that exact commit and its GitHub Release through `b10x-gates`. Verify the peeled tag,
required source checks, Release and downloadable source archives. There is no binary packaging
workflow or crates.io publication. Documentation delivery runs asynchronously; report it pending
unless its live publication has been verified. A source release does not include Atlas, Website
or downstream consumer changes.

## Public documentation

`website/` is the public documentation site, written for people, not agents, and built with
Docusaurus (the only Node in this repository). Hand-written pages are concepts, guides and status
only. Everything derivable is generated by the Rust `canon-docs` crate: `task docs-generate` writes
`website/docs/reference/` (the CLI reference walked from the clap definition in
`crates/canon-cli/src/lib.rs`, the `protocol/1` reference from `crates/canon/src/model/`, the
validation, `canon-ir/1` and conformance pages from module docs, and the investigation example run
through Canon), the JSON Schemas in `website/static/schemas/`, and the whole of `website/data/`: the
`b10x-protocol-graph/1` of the investigation fixture, a `b10x-terminal/1` recording of real
`canon` runs and the `b10x-status/1` status document (those formats forbid a marker line, so the directory is owned whole). Never edit
those files; `task docs-check` (part of `task check`) fails when they drift. A change to the CLI, the model or
those module docs regenerates them in the same change. The landing page is `website/product.json`
(`b10x-product-landing/1`), rendered by the docs-system product-site template through
`withProductSite`. The landing status strip and the status page's `<StatusTable/>` both read `data/status.json`, which `canon-docs` builds from the hand-written `website/status.yaml`: each capability names the AEP story that delivers it, and its status is read from that story (implemented is shipped; otherwise decided when the story cites an Atlas ADR, else planned), never typed. Docs live under
`/docs/`; the client-redirects plugin keeps every older `/canon/<page>` address working.
`pages.yml` builds the site without credentials and `canon-docs site-manifest` binds it to the commit in `.well-known/b10x-site.json`;
`b10x-docs-site.yml` calls the immutable Website project-site publisher for `/canon/`. Mark every
feature a hand-written page describes as shipped or planned, and add a capability to `website/status.yaml` when a
story lands. `canon-docs` is documentation tooling, not Canon's command surface, so it has no ESS
declaration. Verify the live site provenance before reporting it published.
