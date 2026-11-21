---
format: aep.planning-md/3
id: story:protocol-floor
kind: story
status: active
title: Check that a protocol stays above an organization floor with canon floor
refs:
- provider: atlas
  reference: adr:0086
- provider: github
  reference: https://github.com/beyond10x/canon/issues/5
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:semantic-diff
- depends_on: story:protocol-imports
- depends_on: story:ess-command-surface
- depends_on: story:conformance-suite
scope:
- confidence: cited
  path: crates/canon-cli/src/floor.rs
- confidence: cited
  path: crates/canon-cli/src/lib.rs
- confidence: cited
  path: crates/canon-cli/src/main.rs
- confidence: cited
  path: crates/canon-cli/tests/floor.rs
- confidence: inferred
  path: crates/canon/src/floor/mod.rs
- confidence: cited
  path: crates/canon/src/lib.rs
- confidence: inferred
  path: ess/domains/cli.yaml
- confidence: cited
  path: fixtures/investigation/floor/
revision: 29
transitions:
- {from: "draft", to: "proposed", at: "2026-11-20T20:58:27Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":4}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w1"}
- {from: "proposed", to: "active", at: "2026-11-20T20:58:27Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":4}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w1"}
---
## Outcome

`canon floor --floor <path> --ir <path>` (clap derive) answers whether a protocol stays above an
organization-owned floor (Atlas ADR 0086 § Decision 2: "`canon diff` against the floor shows no
relaxation and no breaking change; only tightening and expansion pass"). Both paths are compiled
`canon-ir/1`, as `canon compile` prints it and as `canon diff` reads it; a composed protocol is
compiled with its imports first (story:protocol-imports). The command classifies every change from
the floor to the protocol with story:semantic-diff's classifier. It exits 0 when every change is
`TIGHTENING`, `EXPANSION` or `NO SEMANTIC CHANGE`, and 1 otherwise, printing each `RELAXATION` and
`BREAKING` change with the identifiers it touches, in the classifier's deterministic order.

A floor is an ordinary `protocol/1` document, for example `floor.code-change/1`. Its id, revision
and description differ from the protocol's by design. The classifier refuses two documents with
different ids as `different-protocol` (`crates/canon/src/diff/mod.rs:258-268`), ignores the
revision (`mod.rs:727-738`) and reports a description change as `NO SEMANTIC CHANGE`
(`mod.rs:739`). The IR header is required (`crates/canon/src/ir/mod.rs:86`), so it cannot be
dropped: the floor module gives both IRs one shared header before it calls `diff::diff`, so
story:semantic-diff needs no rule for differing ids and nothing is printed about the header.

On exit 0 the command prints one line, `floor: holds`, followed by the counts of `TIGHTENING`,
`EXPANSION` and `NO SEMANTIC CHANGE` changes (decided by the rescope of 2026-11-20; the draft was
silent). It prints the changes itself and does not call `Diff::render`, whose first line names a
protocol (`mod.rs:212-228`).

The protocol does not name its own floor. The caller supplies it, because a protocol that declared
its floor could drop the declaration, and the floor exists to bind protocols that were generated
or revised without a human writing them. The same command checks that a refinement
(story:protocol-imports § Outcome 3) only tightens what it imports: pass the compiled import as the
floor.

## Order

- depends_on story:semantic-diff (implemented): the check is that story's classification over a
  floor and a protocol; this story adds no category of its own. Expectation 6 reads the
  classification of claim predicates; expectations 3 and 4 read the capability rule and the outcome
  rule.
- depends_on story:protocol-imports: ADR 0086's generated protocols are compositions of imported
  parts, and expectation 2 and the import form of expectations 5 and 6 compile one. The scoper
  found that expectations 1, 3 and 4 need no imports; the story is not split, because
  story:protocol-imports runs one wave earlier in the plan of 2026-11-20.
- depends_on story:ess-command-surface: `canon floor` is a new command, declared in
  `ess/domains/cli.yaml` in this story's first commit.
- Order recorded for https://github.com/beyond10x/canon/issues/5 (2026-10-07):
  story:semantic-diff → story:case-inputs → story:protocol-imports → story:protocol-floor;
  story:ess-command-surface after story:semantic-diff and before story:protocol-imports. The first
  two have landed. story:case-composition landed in 0.9.0, before story:protocol-imports.
- Shares `crates/canon-cli/src/lib.rs`, `main.rs` and `ess/domains/cli.yaml` with
  story:protocol-imports and story:canon-mutate, and `crates/canon/src/lib.rs` with
  story:canon-mutate, so the three run in three different waves.

## Scope

Rescoped 2026-11-20 by `story-scoper` against `8f1f452`, written by the wave coordinator. Every
line is **cited** (read from the story or the tree) or **inferred** (a reading that could be
wrong).

- **In:** the `canon floor --floor --ir` subcommand; the floor module, which gives both IRs one
  shared header and then calls `diff::diff`; the floor verdict over the classifier's output; the
  0 and 1 wording of the exit-status table; the fixtures and the named test. — cited
- **Out:** which floor applies to which kind of work, and adopting a protocol as a case
  (`protocol.adopt/1`): both belong to the engineering-protocols repository. `canon check` on the
  protocol (ADR 0086 § Decision 3) is an existing command. Any change to the classifier: the floor
  adds no category and does not edit `crates/canon/src/diff/mod.rs`. — cited
- **Primary surface:** `crates/canon-cli/src/floor.rs` (new, modelled on
  `crates/canon-cli/src/diff.rs:16-46`) — cited
- **Files:** `crates/canon-cli/src/lib.rs` (the `Command` variant, `lib.rs:57-164`, and the 0 and
  1 text of `EXIT_STATUSES`, `lib.rs:21-43`, which `usage_exit.rs` and `review_cli_contract.rs`
  read) — cited
- **Files:** `crates/canon-cli/src/main.rs` (the `mod` line and the dispatch arm) — cited
- **Files:** `crates/canon-cli/tests/floor.rs` (new) — cited
- **Files:** `crates/canon/src/lib.rs` (the module line, `lib.rs:8-16`) — cited
- **Files:** `crates/canon/src/floor/mod.rs` (new; one file is enough) — inferred
- **Files:** `fixtures/investigation/floor/` (new, owned whole; the base of
  `fixtures/investigation/semantic-diff/removed-capability/from.yaml` already requires
  `explanation.supported` and `finding.publish`) — cited
- **Files:** `ess/domains/cli.yaml` (the `floor` declaration; the file is created by
  story:ess-command-surface) — inferred
- **Coordinator's at integration, not in the typed scope:** the story's entry in
  `website/status.yaml`, and the regenerated `website/docs/reference/cli.md` and
  `website/data/status.json`.
- **Not touched:** `conformance/requirements.yaml`, `ess/system.yaml`,
  `ess/domains/protocol.yaml`, `website/static/schemas/`.
- **Confidence:** high. The body names every new file and the tree shows the wiring sites; only
  the floor module's file name and the ESS file are inferred.
- **Would collide with:** the `Command` enum, the dispatch and the exit table in
  `crates/canon-cli/src/lib.rs` and `main.rs`, and `ess/domains/cli.yaml`: story:protocol-imports
  and story:canon-mutate.
- **Safety fact:** the floor needs no classifier change. `diff::diff`
  (`crates/canon/src/diff/mod.rs:257`) already returns all five categories (`mod.rs:144-164`), and
  the only header field that blocks a floor comparison is the id check at `mod.rs:258`. Unproven.

## Known limits of the verdict

Read from the classifier by the scoper on 2026-11-20; the floor inherits them and this story does
not change them:

- A protocol that adds an artifact or a case input is below the floor: both are `BREAKING`
  (`diff/mod.rs:744-745`, `:831-832`).
- A protocol that adds an order form (`first`, `before`, `after`, `since`) under an `all` is below
  the floor although it tightens: any change to an order form is `BREAKING`, compared structurally
  (`diff/mod.rs:402`, `:421`).
- A `case_open` change is `BREAKING` (`diff/mod.rs:998-1016`).

A floor that should admit these needs a classifier rule, which is a new story.

## ESS first

- Specification change, first commit: the `floor` command and its two inputs added to
  `ess/domains/cli.yaml`, the command surface story:ess-command-surface declares; the fixture set
  `fixtures/investigation/floor/`; and the named test with its expected verdicts. The floor is a
  `protocol/1` document, so the model does not change.
- Red on that commit: `floor_admits_only_tightening_and_expansion` fails, because `canon floor` is
  not a subcommand and clap refuses it with exit 64; and
  `every_canon_subcommand_is_declared_in_ess` (story:ess-command-surface) fails naming `floor`,
  declared in `ess/` and absent from clap.

## Acceptance

`task check` exits 0 on the wave's integration branch with this story merged (the wave coordinator
regenerates the documentation there), and its run includes the named test
`floor_admits_only_tightening_and_expansion` (in `crates/canon-cli/tests/floor.rs`) passing with six
expectations over `fixtures/investigation/floor/`, whose floor requires `explanation.supported` and
the capability `finding.publish` to publish a finding:

1. a protocol that adds an obligation to the floor exits 0;
2. a protocol composed by importing the floor and adding an action exits 0;
3. a protocol that drops the floor's capability requirement exits 1 and prints `RELAXATION` with
   the action's identifier;
4. a protocol that changes an outcome's requirement exits 1 and prints `BREAKING` with the
   outcome's identifier;
5. in expectations 1 and 2, the protocol's id and revision differ from the floor's and nothing is
   printed about them;
6. a protocol that imports the floor and refines `explanation.supported` exits 0 when the
   refinement adds a conjunct under its `all`, and exits 1 printing `RELAXATION` with the claim's
   identifier when the refinement adds an alternative under an `any`.

## Source

https://github.com/beyond10x/canon/issues/5 (gap 3); Atlas ADR 0086 § Decision 2 and § Open ("The
`floor` construct"); `docs/design/canon-protocol-calculus-design.md` § 31.

## Decided in review, after adversary pass 1

The coordinator of wave 2026-11-20-w9 decided these on 2026-11-21, from the unit's report on bringing its wave 3 branch up to the wave branch and from `review-result:adversary-protocol-floor-pass-1`.

- **D1. The pass's verdict is red, and no code changes.** The adversary found no case in which `canon floor` says something other than the classifier of `canon diff`: 35 forms compared with exact bytes, all equal, among them the forms the language gained after the unit was built (`records_together`, `revises`, `decided_by`, `supplied_by`, `max_age`, `independent_on`, action arguments, `case_open`, `children`, order forms, case inputs, invalidation rules, capability scope, `applicable_when`). Its three findings are mutants of the floor that the unit's test left green: a floor that refuses a `NO SEMANTIC CHANGE`, one that counts it as an `EXPANSION`, and one that prints the changes below the floor out of the classifier's order. The pass's three cases in `crates/canon-cli/tests/adversary_floor.rs` fail on each (`4df0f19a`). They stay in the repository; that is the fix the findings name.
- **D2. No second pass.** Nothing changed after the pass but its own test file. The pass ran the unit's five gate steps with its cases in: 164 test summaries, 1240 passed, 0 failed, 3 ignored.
- **D3. The shared header hides nothing.** The header of `canon-ir/1` holds `id`, `revision` and `description` today, and the classifier ignores the first two, so giving the protocol the floor's header drops no change of meaning. If the header gains a field with meaning, the floor module must compare it; the module docs of `crates/canon/src/floor/mod.rs` name the three.
- **D4. Three fixtures need an import** (`composed`, `refines-conjunct`, `refines-alternative`). The tests over every investigation fixture read them without it: the generation test expects `MissingImport`, the check's fixture test skips them, and the validator's and the writer's corpus tests parse them. Only `crates/canon-cli/tests/floor.rs` composes them. That is as the unit built it in wave 3.
- **D5. Not compared by the floor:** decisions and their grounds are records of a case, not declarations of a protocol, so neither `canon diff` nor `canon floor` reads them.
