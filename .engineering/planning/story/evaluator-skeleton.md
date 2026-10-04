---
format: aep.planning-md/3
id: story:evaluator-skeleton
kind: story
status: implemented
title: Split the evaluator into one module per concept and land the shared slots
relations:
- decomposes: epic:canon-kernel
- depends_on: story:three-valued-claims
- depends_on: story:conformance-runner
- depends_on: story:ess-hard-gate
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon-cli/src/conform.rs
- confidence: cited
  path: crates/canon-cli/src/diff.rs
- confidence: cited
  path: crates/canon-cli/src/evaluate.rs
- confidence: cited
  path: crates/canon-cli/src/main.rs
- confidence: cited
  path: crates/canon-cli/tests/evaluator_skeleton.rs
- confidence: cited
  path: crates/canon/src/conform/
- confidence: cited
  path: crates/canon/src/eval/
- confidence: cited
  path: crates/canon/src/explain/mod.rs
- confidence: cited
  path: crates/canon/src/ir/
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/validate/
- confidence: cited
  path: ess/
- confidence: cited
  path: fixtures/investigation/evaluator-skeleton.yaml
- confidence: cited
  path: fixtures/investigation/invalid/undeclared-discharge-claim.yaml
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T02:12:47Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-04T03:45:02Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-10-04T04:40:21Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":3,"verification":1}}}
---
## Outcome

The evaluator story:three-valued-claims builds is split into one module per concept, every shared
slot the later evaluator stories fill is landed once, and the one source-model extension whose shape
the plan already settles (the obligation discharge predicate) is landed with its ESS declaration.
After this story, story:evidence-revision-binding, story:obligations, story:action-admissibility,
story:outcomes and story:evidence-freshness each own a disjoint file and can run in one wave.

Nothing evaluates differently: every stub is inert, so the `canon-decision/1` bytes
story:three-valued-claims produces do not change.

## What it lands

1. **`crates/canon/src/eval/` split** (re-plan 2026-10-04). `mod.rs` keeps only the pipeline:
   read inputs, run the exclusion stages in order (revision binding, freshness, invalidation),
   evaluate claims over the applicable evidence, then the obligations, actions and outcomes
   sections, then the explanation. One file per concept:
   - `case.rs` (`canon-case/1`), `evidence.rs` (`canon-evidence/1`), `decision.rs`
     (`canon-decision/1`), `claims.rs` — story:three-valued-claims' code, moved, not changed;
   - `binding.rs`, `freshness.rs`, `invalidation.rs` — exclusion stages, each a stub that
     excludes nothing;
   - `obligations.rs`, `actions.rs`, `authority.rs`, `outcomes.rs`, `decisions.rs` — section and
     input stubs that return no section.
   `explain/mod.rs` gets the placeholder explanation type and a stub that returns none.
2. **Slots in the shared documents**, so no later story edits `case.rs`, `decision.rs` or
   `mod.rs`: `canon-decision/1` gains one optional slot per section named in the story bodies —
   `obligations`, `actions`, `outcomes`, `explanation` — each typed by its concept file's own
   type; each `claims` entry gains an optional `excluded_evidence` slot whose items are an evidence
   id and a reason from `revision_mismatch`, `expired`, `invalidated` (the reasons
   story:evidence-revision-binding, story:evidence-freshness and story:invalidation-rules name). A
   slot that is empty or absent is not serialized. `canon-case/1` gains `termination`, the optional
   outcome id story:outcomes describes; its check stays in `outcomes.rs`.
3. **CLI split.** `crates/canon-cli/src/main.rs` keeps only the subcommand enum and dispatch; each
   subcommand moves to its own file: `evaluate.rs`, `conform.rs`, `diff.rs` (plus the existing
   `validate` and `compile`, which stay in `main.rs`). `canon evaluate` gains `--authority` and
   `--at`, the inputs story:action-admissibility and story:evidence-freshness name; each is read
   and passed through unparsed, and the library refuses it, naming the flag, until the owning story
   parses it in `authority.rs` / `freshness.rs`. `canon diff --from --to` is declared and refuses
   as not built until story:semantic-diff.
4. **Per-section scenario comparison.** `canon conform run` compares an evaluate step section by
   section: only the sections of `canon-decision/1` the expectation lists are compared, each byte
   for byte. Without this, every story that adds a section would have to edit every earlier story's
   scenario file, and two such stories could never share a wave.
5. **Obligation discharge predicate** (moved here from story:obligations, whose body settles it):
   `discharged_when`, a required predicate on every obligation, in the source model, the validator
   (its claim references resolve), `canon-ir/1` and `ess/`.

## Rules every later evaluator story keeps

- A field a story adds to an input document is optional, and its absence keeps the earlier
  behaviour, so no earlier scenario has to change.
- A story edits only its own concept file(s), fixture and scenario file. If it finds it must edit
  `mod.rs`, `case.rs`, `decision.rs` or another story's file, the plan is wrong for it: stop and
  report rather than widen the change.

## ESS first

- Specification change, first commit: in `ess/domains/protocol.yaml`, the field
  `discharged_when: canon.protocol.Predicate` on `canon.protocol.Obligation`; on the case-snapshot
  declaration story:three-valued-claims adds to `ess/`, the field
  `termination: Optional<canon.protocol.OutcomeId>`. Each cites the story body and design line it
  was read from, then the Rust line once it exists.
- Red on that commit: `crates/canon/tests/ess_model_matches.rs` fails naming
  `Obligation.discharged_when`, because the Rust model does not have it yet.
- The module split, slots and CLI split change no behaviour; the named test below holds that.

## Scope

- In: everything under "What it lands"; the fixtures `fixtures/investigation/evaluator-skeleton.yaml`
  (the base plus one obligation, `establish.explanation`, discharged when `explanation.supported`
  is `TRUE`) and `fixtures/investigation/invalid/undeclared-discharge-claim.yaml`; the test file
  `crates/canon-cli/tests/evaluator_skeleton.rs`.
- Out: any evaluation semantics; `max_age`, invalidation rules, decision requirements and
  `canon-decisions/1`, whose shapes the plan does not settle (they stay with their stories);
  `canon-authority/1`'s shape (story:action-admissibility).
- Surfaces: `crates/canon/src/eval/` (every file), `crates/canon/src/explain/mod.rs`,
  `crates/canon/src/conform/`, `crates/canon/src/model/`, `crates/canon/src/validate/`,
  `crates/canon/src/ir/`, `ess/`, `crates/canon-cli/src/main.rs`,
  `crates/canon-cli/src/evaluate.rs`, `crates/canon-cli/src/conform.rs`,
  `crates/canon-cli/src/diff.rs`, the two fixtures, the test file.

## Shared surface and order (re-plan 2026-10-04)

Depends on story:three-valued-claims (it splits that story's code and extends its documents and
its ESS case declaration) and story:conformance-runner (it changes the runner's comparison). Every
later evaluator story depends on this one for its file and slot.

## Acceptance

The named test `evaluator_skeleton_is_inert` (in `crates/canon-cli/tests/evaluator_skeleton.rs`)
passes with four expectations: `canon conform run` over a directory holding only
`conformance/scenarios/three-valued-claims.yaml`, unedited, passes; a scenario whose expected
document omits a section the output carries passes, and one whose listed `claims` section differs
fails naming the scenario, step and section; `canon compile` of
`fixtures/investigation/evaluator-skeleton.yaml` emits `establish.explanation` with its discharge
predicate; and `canon validate` of `fixtures/investigation/invalid/undeclared-discharge-claim.yaml`
is refused naming the undeclared claim. `task ess-gate` stays green.

## Source

Re-plan brief 2026-10-04 (wider waves); story:obligations § Outcome (discharge predicate);
story:outcomes § Extends (termination field); story:action-admissibility and
story:evidence-freshness § Extends (`--authority`, `--at`); Atlas ADR 0080 (draft).


## From wave 2026-10-04-w4 (three-valued-claims, adversary pass 2, F5)

Evaluate steps in `canon conform run` compare the whole decision byte for byte. A new slot inside
each `claims` entry (such as `excluded_evidence`) breaks every existing expectation unless the slot
is left out when empty. This story keeps CANON-CLAIM-001 unedited by omitting empty slots, and says
so in the decision format. The claim-value memo in `eval/claims.rs` must survive the split into
per-concept files (a pass-2 adversary case pins it).


## Coordinator decisions (wave 2026-10-04-w6)

- The case, evidence and decision document types stay in `crates/canon/src/model/`, where
  `ess_model_matches` compares them with `ess/`; only their reading and rendering code moves into the
  `eval/` files. Where this body says the files move, read it as that.
- The decision's five slots are declared in `ess/` in this story's first commit: `excluded_evidence`
  with its settled shape, and `obligations`, `actions`, `outcomes`, `explanation` as
  `Optional<Json>`, each naming the story that later gives it a type. Each of those stories replaces
  its own Json with its type in its own ESS change.

### Adversary decisions (wave 2026-10-04-w6)

- Pass 1 F1: `canon validate` refuses an evidence match inside `discharged_when`
  (`evidence-in-discharge`), since a discharge predicate is over claim values (fixed).
- Pass 1 F2: `claims.rs` exposes `pub(super) fn predicate(predicate, claims, evidence)`, so the
  wave-7 stories do not edit `claims.rs` (fixed).
- Pass 1 F3: `actions::section` and `outcomes::section` receive the evidence left after the
  exclusion stages (fixed).
- Pass 1 F4: direct unit tests for `set_aside` and `excluded_for` kill the three surviving mutants (fixed).
- Pass 1 F5: listed sections are compared each on its own, in any key order (fixed).
- Pass 1 F6: `excluded_for` follows claim references transitively, each claim visited once (fixed).
- Pass 2 F1: the runner writes a scenario authority list back so it always reaches the evaluator
  (flow style where the YAML emitter refuses a mapping key); an entry that reads back differently is
  a step failure, never a panic (fixed; the adversary cases require the entry to pass).
- Pass 2 F2: section comparison keeps every byte, so a `\r` difference fails (fixed).
- Pass 2 J1: the four decision-section slots stay `Json`; comments in `ess/` and the model now say
  each section's shape is fixed by its story's conformance scenarios and typing them is a later
  model story (fixed).
- Pass 2 J2: the evaluator docs list refusals in their real order (fixed).
- Pass 2 J3: a scenario authority stays a YAML list; a format header on `canon-authority/1` would be
  story:action-admissibility's change to `conform/` (no-op).
