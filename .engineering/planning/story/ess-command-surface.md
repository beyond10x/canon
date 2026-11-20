---
format: aep.planning-md/3
id: story:ess-command-surface
kind: story
status: active
title: Declare the canon command surface in ESS
refs:
- provider: atlas
  reference: adr:0076
relations:
- decomposes: epic:canon-kernel
- depends_on: story:ess-hard-gate
- depends_on: story:semantic-diff
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:conformance-suite
scope:
- confidence: cited
  path: AGENTS.md
- confidence: inferred
  path: Taskfile.yml
- confidence: inferred
  path: crates/canon-cli/tests/ess_commands_match.rs
- confidence: cited
  path: ess/domains/cli.yaml
- confidence: cited
  path: ess/ess-inputs.yaml
- confidence: cited
  path: ess/system.yaml
revision: 20
transitions:
- {from: "draft", to: "proposed", at: "2026-11-20T20:58:27Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w1"}
- {from: "proposed", to: "active", at: "2026-11-20T20:58:27Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w1"}
---
## Outcome

The `canon` command surface is declared in `ess/` as commands of a new domain `canon.cli`,
completing ADR 0076's "data model and command surface". There is one command per leaf subcommand
on `main` at `8f1f452`: `validate`, `compile`, `evaluate`, `check`, `generate`, `diff` and
`conform run` (`crates/canon-cli/src/lib.rs:57-182`), each with every flag as an input. Protocol
semantics stay in Canon's own conformance suite.

From then on a story that adds or changes a command declares the change in `ess/` in its first
commit. Three accepted stories depend on this one for that reason: story:protocol-imports
(`--import` on `validate`, `compile` and `check`), story:protocol-floor (`canon floor`) and
story:canon-mutate (`canon mutate`). The first two edges were recorded on 2026-10-07 for
https://github.com/beyond10x/canon/issues/5; the third was added by the rescope of 2026-11-20.

A parity test holds the rule: it compares the clap definition with the compiled ESS commands in
both directions, so a flag or subcommand that exists in only one of them fails `task ess-gate`.

## ESS first

Exempt from "spec first, then red" under AGENTS.md § ESS ("Only a change with no behaviour change
is exempt, and its story says so"). This story changes no behaviour of `canon`: all seven commands
already exist, so no later commit could turn a specification-first red test green without changing
the specification. It is test first instead:

- Commit 1 adds only the parity test
  `crates/canon-cli/tests/ess_commands_match.rs::every_canon_subcommand_is_declared_in_ess` and its
  three controls (`a_flag_only_in_clap_is_named`, `a_command_only_in_ess_is_named`,
  `a_changed_requiredness_is_named`). It is red on that commit: the compiled specification declares
  0 commands (`"commands": {}`, observed by the scoper with ess 0.77.0) against 7 leaf subcommands
  in clap. The failing run is recorded.
- Commit 2 adds `ess/domains/cli.yaml`, lists it in `ess/system.yaml` (`domains:`) and in
  `ess/ess-inputs.yaml` (`specification:`), adds the test to `task ess-gate` in `Taskfile.yml`, and
  rewrites AGENTS.md § ESS. The test passes without being changed.

AGENTS.md § ESS is reworded in commit 2, not before:
`crates/canon/tests/adversary2_ess_gate.rs:65-96` asserts that the specification has more than 0
commands once AGENTS.md § ESS contains "command surface".

The mapping the test holds: clap path `conform run` is wire `conform-run`; `--flag` is an input of
the same name; an `Option<_>` flag is an optional input.

## Acceptance

With ess 0.77.0 first on `PATH`, `task ess-gate` exits 0 and its run includes
`every_canon_subcommand_is_declared_in_ess` and its three controls passing: every leaf subcommand
and every long flag of `canon_cli::Cli` has a command and an input in the compiled `canon.cli`
domain, and the reverse. `ess verify conform synthesize --path ess` reports 0 refusals and at least
one scenario per declared command; the parity test counts the scenarios per command, because
`crates/canon/tests/ess_gate.rs` does not.

## Source

Atlas ADR 0076; gap named while drafting story:ess-hard-gate (2026-10-04); Atlas ADR 0080 (draft).

## Scope

Rescoped 2026-11-20 by `story-scoper` against `8f1f452`, written by the wave coordinator. Every
line is **cited** (read from the story or the tree) or **inferred** (a reading that could be
wrong).

- **In:** one ESS command per leaf `canon` subcommand, with every flag as an input, in a new domain
  `canon.cli`; the domain registered in `ess/system.yaml` and `ess/ess-inputs.yaml`; the clap
  versus ESS parity test, run by `task ess-gate`; AGENTS.md § ESS rewritten so it no longer says
  "no commands". — cited
- **Out:** protocol semantics (they stay in `conformance/`); `canon-docs`, which has no ESS
  declaration (`AGENTS.md:125`); new subcommands (story:protocol-floor, story:canon-mutate); new
  flags (story:protocol-imports); any edit to `crates/canon-cli/src/*.rs`;
  `crates/canon/tests/ess_model_matches.rs`, which skips types outside its `DOMAINS`
  (`ess_model_matches.rs:50,229-233,325`). — cited
- **Primary surface:** `ess/domains/cli.yaml` (new) — cited
- **Files:** `ess/system.yaml:13-17` (domain list) — cited
- **Files:** `ess/ess-inputs.yaml:3-8` (the `specification:` list; a file it does not name is not
  read) — cited
- **Files:** `crates/canon-cli/tests/ess_commands_match.rs` (new, the red test) — inferred, proposed
  by the scoper
- **Files:** `Taskfile.yml:18-22` (`ess-gate` gains
  `cargo test -p canon-cli --test ess_commands_match --locked`) — inferred
- **Documents:** `AGENTS.md:33-40` (today "with no commands … still owes the `canon` CLI's
  commands") — cited
- **Read, not written:** `crates/canon-cli/src/lib.rs`; the parity test walks `canon_cli::Cli`
  through `clap::CommandFactory`. clap and `serde_yaml_ng` are already dependencies of `canon-cli`.
  — cited
- **Coordinator's at integration, not in the typed scope:** the story's entry in
  `website/status.yaml` and the regenerated `website/data/status.json`. Every story of the plan
  adds one entry there, so the wave coordinator writes them once on the integration branch.
- **Confidence:** medium. The ESS files and documents were read. The test file and the Taskfile
  line are the scoper's proposal.
- **Would collide with:** any unit that adds or changes a `canon` subcommand or flag (it must edit
  `ess/domains/cli.yaml` once this lands); any unit that adds an ESS domain (`ess/system.yaml`,
  `ess/ess-inputs.yaml`); any unit editing AGENTS.md § ESS.
- **Safety fact:** subjectless commands compile and synthesize under the pinned ess 0.77.0 at
  `format: ess/20`: `crates/canon/tests/ess_gate.rs:27-60,343-358` appends one to a copy of `ess/`
  and synthesizes it. Unproven for a `canon.cli` domain in its own file.
- **Not established by the scoper:** the field layout of a compiled command in the 0.77.0 IR
  (the `commands` map is empty today); whether a pure, file-reading command needs `emits:` events
  or can declare a `response:`; whether ADR 0076 expects command inputs typed by the
  `canon.protocol` model (the CLI takes file paths only).
- **Order:** story:ess-hard-gate, story:semantic-diff and story:conformance-suite are implemented,
  so nothing holds this story. story:protocol-imports, story:protocol-floor and story:canon-mutate
  depend on it.
- **Gate environment:** from the repository root the `ess` 0.96.0 dispatcher refuses `--path ess`;
  the gate needs `~/.cache/ess/toolchains/0.77.0` first on `PATH` (measured 2026-11-20: 7 of 11
  `ess_gate` tests fail without it, 11 of 11 pass with it).
