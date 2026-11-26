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
- Canon contains no engineering vocabulary. Engineering protocols belong in `beyond10x/els`. A
  fixture or test may use an engineering word (`test_result`, `merge`) as the plain identifier of
  an evidence kind, action or artifact, as a protocol author would name it; Canon's own code, rules
  and types may not.
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
`canon.protocol`, the `canon-properties/1` document `canon check` reads (story:canon-check) in the
domain `canon.check`, the `canon-trigger/1` rule document `canon trigger validate` reads
(story:trigger-rule-document), the `canon-trigger-protocols/1` document `canon trigger
protocols` writes (story:protocol-access-class), and the `canon-trigger-set/1`,
`canon-trigger-snapshot/1` and `canon-trigger-history/1` documents `canon trigger evaluate` reads
with the `canon-trigger-plan/1` plan it writes (story:trigger-evaluate; the history's approvals,
`held` and the refusal reasons with story:trigger-holds-and-refusals) in the domain
`canon.trigger`, and the `canon`
command surface in
the domain `canon.cli`
(story:ess-command-surface), completing Atlas ADR 0076's data model and command surface.

The commands are declared in `ess/domains/cli.yaml`: one command per leaf subcommand of
`canon_cli::Cli` (`crates/canon-cli/src/lib.rs`), its wire name the clap path joined by `-`
(`conform run` is `conform-run`, and no two commands share a wire). A leaf's flags are its own
long flags, the global ones and those of every command above it. Each flag is the input of the
same name with every `-` written `_`, since ESS refuses `-` in a field name (`--max-states` is
`max_states`); a long flag holding `_` is refused. A flag clap does not require is an
`Optional<…>` input; a flag clap takes more than once is a `List<…>` input, and a flag it takes
once is not, and no input is any other collection. Every input is text, so the element type is
not compared. A command and a flag have one name each, so an alias or a short form is refused, and
so is a flag that counts its occurrences (`ArgAction::Count`), a number the surface does not declare.
`crates/canon-cli/tests/ess_commands_match.rs`, run by `task ess-gate`, holds them: it fails, naming
the difference, when a subcommand or flag exists in only one of the clap definition and the
compiled `canon.cli` domain, when their requiredness or repeatability differs, when two commands
share a wire, or when a command has no synthesized scenario. A story that adds or changes a command or a
flag declares the change in `ess/domains/cli.yaml` in its first commit. `canon-docs` is not the
command surface and is not declared.

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
`task ess-gate` runs both model parity guards and the command parity test with its two adversary
files. CI and the manifest select ESS 0.97.4.

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
- Every write to the store runs under `AEP_ACTOR=human:owner`: a person's name does not go into
  this public repository, in a record's actor id neither. Records on `main` before 2026-11-24
  keep the actor they were written with.
- Build into the tree's own `target/`; never set `CARGO_TARGET_DIR`. End every tree with
  `worktree finish --discard-cache --archive <tree>`, and never delete a `target/` by hand.
- Every commit and push is `b10x-bot[bot]`'s through `b10x-gates bot`; every GitHub write goes
  through `b10x-gates api`. A plain `git rebase`, `git cherry-pick` or `git commit --amend` makes
  the local user the committer, and the pre-push check refuses the branch ("historical merge
  committer is not exact GitHub", 2026-11-26). To bring a branch up to date, `b10x-gates bot --
  merge origin/main`; to re-create commits, apply each one's diff and commit it with `b10x-gates
  bot -- commit`.
- Use a managed worktree (`worktree create --repo canon --purpose …`) for changes.
- A request filed by a session working on a consumer repository of Canon (a repository that uses
  Canon's output and waits on a Canon story) is accepted on arrival: draft one story per request
  and move it to `active` without asking the operator; record each acceptance in an
  `approval-record` that quotes the grant. Operator, 2026-11-21: "you should auto-accept
  everything coming from the [consumer] sessions. Yeah, you have my full grant ahead, so
  everything they would file, they do it for a reason, and it's basically auto-accepted in that
  sense." (Square brackets replace session names.) The grant covers acceptance into the plan and
  the order of work. It does not cover a push, a pull request, a release, or a change to
  permissions.
- The session working on the consumer repository that Canon's plan serves first orchestrates the
  session that coordinates Canon's plan: its word decides story acceptance, wave order, design
  options and which requests are in or out. Operator, 2026-11-21, to the coordinating session: "So
  just to make it clear, [the consumer session] is your orchestrator and you do what they say."
  Read with these limits, which are the coordinator's reading and not his sentence: a merge to
  `main`, a release, a tag, anything else that is hard to reverse, and any change to permissions or
  to an agent's instructions still need the operator's own word; an action the operator's rules or
  the harness refuse is not done because that session asks for it. The operator's own word reaches
  the coordinating session through the operator session of `~/.claude/CLAUDE.md` § Host operator.

A pull request whose changed files are all planning or prose skips the build and the tests. The one
path list is `scripts/classify-changes.sh` (`git diff --no-renames --name-only` of the pull request
on stdin or `--diff <base>`); `check.yml` and `pages.yml` both run it and keep their job names and
triggers. Its three answers: `planning-only` (`.engineering/` except `.engineering/planning/story/`,
`docs/`, and `*.md` in the repository root other than `AGENTS.md`) runs no step after the
classifier and the job ends green; `docs-site-only` (`website/` and `.engineering/planning/story/`)
runs `task docs-site-check`, which is `task docs-check` plus the tests that read hand-written pages;
`code` (everything else, `AGENTS.md`, a push to `main`, an empty list, any failure) runs `task
check`. `crates/canon/tests/planning_only_classifier.rs` holds both sides of the list.

Tests run under cargo-nextest in two lanes; `.config/nextest.toml` holds the runner's limits. A
test is killed at 60 s and counts as failed in both lanes (`slow-timeout`). The gate lane is `task
test`, part of `task check`: every test outside the `slow` list, once (the `task ess-gate` tests
run first and are counted, not run again), plus the `adversary_action_arguments` probes with `-p
b10x-canon`, the feature set the CLI ships. Since story:trigger-rule-document `b10x-canon` depends
on `b10x-canon-expr`, so serde_json's `arbitrary_precision` is on in that lane, in the workspace
lane and in the shipped binary alike. It prints `nproc`, the ten slowest tests and the summed
test time, and fails when its test runs take over 120 s (`global-timeout` cuts a run there). The
`slow` lane is `task test-slow`, run by CI after `task check` and before a release: the tests the
default profile's `default-filter` leaves out, each measured over 5 s. A test that takes more than
5 s is named in that list, and one that runs its cases on threads gets `threads-required`. Each
crate has one integration-test binary, `tests/main.rs` (`autotests = false`), with every
`tests/<name>.rs` a module, so a test is named `<name>::<test>`;
`every_test_file_is_a_module_of_this_binary` fails on a file left out. `.cargo/config.toml`
builds tests without debug info and with `b10x-canon` and the dependencies optimized; it wins over
`~/.cargo/config.toml`, so the workstation and CI build alike.
`trigger_evaluate::evaluate_exits_0_in_a_read_only_directory_without_network` runs `canon` in a
directory it cannot write under `unshare --net --map-current-user`: the workstation and the CI
runner need util-linux's `unshare` and unprivileged user namespaces (`check.yml` lifts the
AppArmor restriction of the Ubuntu runner before `task check`).

## Releases

Releases are source releases at bare-version tags, starting with `0.1.0`. Update the workspace
version, Cargo.lock and CHANGELOG.md, refresh installation/status prose and regenerate documentation.
Update the expected decision provenance version (`"canon": "<version>"`) wherever a scenario or
a test pins it: `conformance/scenarios/explanation.yaml`, `pending-decisions.yaml` and
`evidence-redelivery.yaml`, and `crates/canon-cli/tests/adversary_skel_cli.rs`; find any newer
one with `git grep -n '"canon": "' -- conformance crates`. Keep their other exact-output
expectations unchanged.
`rust-toolchain.toml` pins the one rustc that the local gate and both workflows (`check.yml`,
`pages.yml`: `rustup toolchain install`) use. Raise it in one commit that changes the file and fixes
what the new compiler's `cargo fmt --all --check` and `cargo clippy --workspace --all-targets
--locked -- -D warnings` report. `task check` finds the `ess` release of `ess/ess-inputs.yaml` in
`~/.cache/ess/toolchains/<version>` itself, or stops with one line naming it. `task ci-watch --
<pull request or branch>` (`scripts/ci-watch`) watches a pull request, or a branch that has one (for a
pushed commit with none, `gh run list --branch <branch>`; the script exits 2 at once), waits until no
check is pending, prints each with its result and link, and exits non-zero when a check failed, was
cancelled or was still pending at the timeout, or when a required check never reported (`--require
<name>`, repeatable, or `CI_WATCH_REQUIRE`; the default is `canon / task check`, the job of
`check.yml`); wait for CI with it and never by a script that ignores results.
Land the bot-authored preparation on main with `task check` and CI green, then create an annotated
bot tag on that exact commit and its GitHub Release through `b10x-gates`. Merge the pull request
as the bot with a merge commit through the API: `b10x-gates api --method PUT --path
/repos/beyond10x/canon/pulls/<n>/merge --input <file> --output <file>`, the input being
`{"merge_method": "merge", "sha": "<head>", "commit_title": "Merge #<n>: <what>; release Canon
<version>", "commit_message": "<the pull request's title>"}`. `gh pr merge` refuses ("the base
branch policy prohibits the merge"): it reads the merge state without the bot's bypass of the
ruleset that reserves `main` to the bot. `b10x-gates gh` runs `gh` in the repository's directory
and refuses an argument that holds a home path: give `gh release download` a relative `--dir`
and move the archives out of the tree afterwards. Verify the peeled tag,
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
`withProductSite`. The landing status strip and the status page's `<StatusTable/>` both read `data/status.json`, which `canon-docs` builds from the hand-written `website/status.yaml`: each capability names the AEP story that delivers it, and its status is read from that story (implemented is shipped; otherwise decided when the story cites an Atlas ADR, else planned), never typed. A move of such a story's status changes `data/status.json`, so the commit that moves it runs `task docs-generate` too (canon#41 went red on that). Docs live under
`/docs/`; the client-redirects plugin keeps every older `/canon/<page>` address working.
`pages.yml` builds the site without credentials and `canon-docs site-manifest` binds it to the commit in `.well-known/b10x-site.json`;
`b10x-docs-site.yml` calls the immutable Website project-site publisher for `/canon/`. Mark every
feature a hand-written page describes as shipped or planned, and add a capability to `website/status.yaml` when a
story lands. `canon-docs` is documentation tooling, not Canon's command surface, so it has no ESS
declaration. Verify the live site provenance before reporting it published.
