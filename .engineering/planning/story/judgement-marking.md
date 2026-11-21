---
format: aep.planning-md/3
id: story:judgement-marking
kind: story
status: implemented
title: Mark judgement-authored actions, revised artifacts and judgement-decided evidence in protocol/1
tags:
- from-request
refs:
- provider: sdc
  reference: docs/loom-sdk-and-agent-node.md
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:canon-ir
- depends_on: story:semantic-diff
scope:
- confidence: cited
  path: crates/canon/src/diff/mod.rs
- confidence: cited
  path: crates/canon/src/eval/read.rs
- confidence: cited
  path: crates/canon/src/ir/mod.rs
- confidence: cited
  path: crates/canon/src/model/mod.rs
- confidence: cited
  path: crates/canon/src/validate/mod.rs
- confidence: inferred
  path: crates/canon/tests/judgement_marking.rs
- confidence: cited
  path: ess/domains/protocol.yaml
- confidence: inferred
  path: fixtures/investigation/judgement-marking.yaml
- confidence: inferred
  path: website/docs/concepts/protocols.md
revision: 19
transitions:
- {from: "draft", to: "proposed", at: "2026-11-20T23:28:16Z", actor: "human:timo", revision: 14, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
- {from: "proposed", to: "active", at: "2026-11-20T23:28:16Z", actor: "human:timo", revision: 15, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
- {from: "active", to: "implemented", at: "2026-11-21T06:20:18Z", actor: "human:timo", revision: 19, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w5"}
---
## Outcome

`protocol/1` gets three optional fields. They mark where a protocol's results depend on judgement rather than on a reproducible procedure. Canon gives them no evaluation meaning in this story: `canon evaluate`, `canon check` and `canon generate` give the same bytes with or without them.

1. **`author` on an action.** The value is `program` or `judgement`; absent means `program`. `judgement` says that the action's arguments, and the content of the artifacts it revises, come from a judgement (a person or a model) and are not a function of the action's inputs.
2. **`revises` on an action.** A list of artifact ids: the artifacts the action may move to a new revision; absent means none. `canon validate` refuses an id that is not a declared artifact as `undeclared-artifact`, naming the action. The validator reports it after the action's `may_produce` kinds and before the subject of its `case_open` (validation step 8, `crates/canon/src/validate/mod.rs:51-54`).
3. **`decided_by` on each `may_produce` entry.** The value is `program` or `judgement`; absent means the action's `author`. It says who decides the record's result. `decided_by: program` on a judgement action covers records the runner writes from the effect it applied, such as provenance.
4. **`canon-ir/1`.** An action writes `"author": "judgement"` only when it is judgement, and `"revises"` only when the list is non-empty, sorted, each id once. A production writes `"decided_by": "judgement"` only when its resolved value is judgement. When one action lists a kind twice, judgement wins. So `author: program` written out compiles to the same bytes as no `author`, and the IR of every existing protocol is byte-identical. `ir::Action` keeps `may_produce: Vec<EvidenceKindId>` (`crates/canon/src/ir/mod.rs:168`) and adds `author`, `revises` and a set of judgement-decided kinds. The IR read-back (`crates/canon/src/eval/read.rs:336-372`) reads all three.
5. **`canon diff`** (`crates/canon/src/diff/mod.rs:32-41`, `action` at `:951`):
   - `author` from `program` to `judgement` is `TIGHTENING`; the reverse is `RELAXATION`.
   - A production becoming judgement-decided is `TIGHTENING`; becoming program-decided is `RELAXATION`.
   - An artifact added to `revises` is `RELAXATION`; one removed is `TIGHTENING`.

   Under the follow-up checks, a judgement mark only adds obligations, and `revises` widens what may move.

## Why

- **Request.** This is request 10 of the SDC prototype: `docs/plan/wave-plan-2026-11-20.md:140-143` in the canon tree, where it is still a request and no story.
- **Design.** The design is `$HOME/beyond10x/sdc/docs/loom-sdk-and-agent-node.md:148-166`. That file is another session's research and was not verified as a whole. `$HOME/beyond10x/sdc/docs/canon-research/02-branching-and-universes.md:24-27` and `:198-202` say every metric that separates judgement edges from program edges needs these fields first. `.engineering/planning/story/canon-simulate.md:189-193` names `revises` as what its invalidation loops wait for.
- **Verified in the canon tree.**
  - `effect` is collected only as a name, at `crates/canon/src/validate/mod.rs:505-507` at `8f1f452` (`:535-536` here).
  - `EvidenceProduction` holds only `evidence`, at `crates/canon/src/model/mod.rs:248-251` at `8f1f452` (`:250-256` here).
  - `Action` and `EvidenceProduction` are `deny_unknown_fields` (`model/mod.rs:194`, `:252`), so a protocol using the fields is refused today.
  - Nothing in `crates/canon/src/eval/` reads `may_produce`.
  - `canon-case/1` carries one revision per artifact (`CaseArtifact`). It does not record which action moved it.
- **Boundary.**
  - The fields are Canon's. `judgement` covers a person as well as a model. The fields name no executor, harness, model, provider or tool. Canon cannot prove that the runner honours them, which is already true of `independent_on` (`crates/canon/src/eval/independence.rs:4-12`).
  - Not Canon's:
    - which agent or program runs an action, with its budget, sandbox, principal and result tables (Loom, `loom.bindings/1`, `loom-sdk-and-agent-node.md:174-191`);
    - Loom's checks when it loads a binding;
    - refusing a runtime that moves an artifact outside `revises`, which needs a record Canon never receives.
  - ADR 0083:31-43 (`$HOME/beyond10x/atlas/architecture/adr/0083-capabilities-and-runtime-binding.md`) puts capabilities in Canon and runtime binding in Commission. It says nothing about judgement. The design applies it by analogy.
- **How `authority-bypass` is computed** (`crates/canon/src/check/mod.rs:710-761`):
  - Governed kinds are those an action with `requires` may produce.
  - A witness is a state with no authority decision and only ungoverned kinds present, in which the outcome is legitimate, and where erasing one present, positively read dimension blocks it.
  - The check evaluates every state, so the bound applies.
- **How independence works** (`independence.rs:56-74`). A record meets a demand only when its producer is listed on every dimension demanded. A record that fails the demand is not of the match, so the match is `unknown`. Only an evidence match carries `independent_on`; order forms and `children` cannot (`validate/mod.rs:27-36`).
- **The three checks:**
  - `self-judged-evidence` is static over predicates, like `unproduced-evidence` (`check/mod.rs:599-613`). It needs a ruling for order forms and `children` that read a judgement-decided kind.
  - `judgement-bypass` is a state search with `authority-bypass`'s machinery: judgement-decided kinds instead of governed ones, and readers that demand no independence.
  - `unchecked-judgement` as designed (`loom-sdk-and-agent-node.md:163`) requires a positive read of a program-decided match. It does not prove the stated property: under an `any`, the outcome can hold through another member. The exact form is a state search: the outcome is legitimate in a state with no program-decided record about the revised artifact. The check space puts every artifact at one revision (`check/mod.rs:241-245`), so "current revision" holds by construction.
  - `self-judged-evidence` and `judgement-bypass` overlap. If every match on a judgement kind demands independence, `judgement-bypass` can fire only through an order form or `children`.
- **What the checks need from story:check-subject-slices** (`impl/check-subject-slices` at `8d8dc31`).
  - `decided_by` adds no dimension; it only partitions kinds. So the two state checks are decided over the outcome's slice, as `authority-bypass` is, with no change to slicing.
  - The design says the static checks escape the 65,536 bound (`loom-sdk-and-agent-node.md:163-164`). In `canon check` they do not. The refusal comes before any finding (`check/mod.rs:559-575` here). On the branch, a slice over the bound refuses the whole check "and nothing else is reported" (its `check/mod.rs:281-282`). They escape only in `canon validate`, or if `check` reports static findings before refusing.
- **Not verified:**
  - The design's `driver.md:122-131` and `:128-129`; that content is at `$HOME/beyond10x/sdc/docs/driver.md:233-243`.
  - Every Loom citation; Loom was not read.

## Decision needed

| Option | `ess/` and model | Cost |
|---|---|---|
| A. Three fields, as requested | Enum `canon.protocol.Author` {program, judgement}; `Action.author: Optional<Author>`, `Action.revises: List<ArtifactId>`, `EvidenceProduction.decided_by: Optional<Author>` | Model, validator, IR write and read-back, diff. No reader of `ir::Action.may_produce` changes. |
| B. `revises` and `decided_by` only: an artifact counts as judgement-written when its action has a judgement-decided production | Two fields | Cannot mark the SDC `spec.draft` case, which is a judgement action whose only production is program-decided (`loom-sdk-and-agent-node.md:203-210`). |
| C. Reserve `effect: judgement`, plus `revises` and `decided_by` | `EffectClass` stops being a free name | `spec.draft` is `effect: write` and judgement at once, which one field cannot say. Existing protocols may already use the name. |

Recommendation: A, keeping the names `author`, `revises`, `decided_by`, `program` and `judgement`, because each states a generic class and B and C cannot express the case that motivates the request.

## ESS first

- **Specification change, first commit:**
  - In `ess/domains/protocol.yaml`: the enum `canon.protocol.Author`; the fields `author` and `revises` on `canon.protocol.Action` (`:146-200`); `decided_by` on `canon.protocol.EvidenceProduction` (`:252-257`), with `# read from:` comments.
  - The fixture `fixtures/investigation/judgement-marking.yaml` and the test file `crates/canon/tests/judgement_marking.rs`.
- **Red on that commit:**
  - `crates/canon/tests/ess_model_matches.rs::specification_and_model_are_equal` fails, naming `Action.author`, `Action.revises`, `EvidenceProduction.decided_by` and `Author`.
  - `judgement_marking_compiles_and_means_nothing_yet` fails because `deny_unknown_fields` refuses `author`.
- No conformance scenario: the story adds no evaluation meaning (AGENTS.md:66-67).

## Scope

- **In:** the three fields in the model, ESS, validator, IR and IR read-back; the diff rows; the fixture and the named test. — cited
- **Out:**
  - The three checks (§ Order).
  - Any evaluation meaning.
  - A revise step in `canon simulate`.
  - Loom bindings, and refusing a move outside `revises`.
  - The design's rule that a judgement action with neither `revises` nor a production is an error.
  - `decided_by` for `case_outcome`.
  - Static per-action numbers (`02-branching-and-universes.md:203-205`). — cited
- **Surfaces:**
  - `ess/domains/protocol.yaml` — cited
  - `crates/canon/src/model/mod.rs` — cited
  - `crates/canon/src/validate/mod.rs` — cited
  - `crates/canon/src/ir/mod.rs` (module doc exception list `:18-28`, `Action` `:155`, compile `:248-272`, write `:609-674`) — cited
  - `crates/canon/src/eval/read.rs` — cited
  - `crates/canon/src/diff/mod.rs` — cited
  - `crates/canon/tests/judgement_marking.rs` (new) — inferred
  - `fixtures/investigation/judgement-marking.yaml` (new) — inferred
  - `crates/canon/tests/ess_model_matches.rs` (red only; may need no edit) — inferred
  - `crates/canon-docs/src/source.rs` (reads a lowercase enum, `:58-59`) — inferred
  - `website/docs/concepts/protocols.md` — inferred
  - Coordinator's at integration, not in the typed scope: the regenerated `website/docs/reference/protocol.md` and `canon-ir.md`, `website/static/schemas/protocol-1.schema.json`, and `website/status.yaml`.
- **Confidence:** high for the model, validator and IR (read). Medium for the diff directions, which are this draft's.

## Order

- Depends on story:canon-ir and story:semantic-diff, both implemented.
- Shares files with active stories:
  - `ess/domains/protocol.yaml`, `model/mod.rs`, `validate/mod.rs` and `ir/mod.rs` with story:protocol-imports (wave 2, in flight), story:supplied-evidence, story:revision-ordering, story:child-outcome-revision and story:argument-validator.
  - `diff/mod.rs` and `eval/read.rs` with story:supplied-evidence and story:revision-ordering.
  - `ess/domains/protocol.yaml` with story:missing-evidence.
  - It runs in a wave with none of them.
- story:missing-evidence reads `may_produce`; the side set keeps its type.
- Follow-up stories, one each:
  1. `self-judged-evidence`: static; in `canon validate`, or a `canon check` finding reported before any bound refusal; with a ruling for order forms and `children`.
  2. `unchecked-judgement`: the exact form, a state search over the outcome's slice, after story:check-subject-slices.
  3. `judgement-bypass`: `authority-bypass` over judgement-decided kinds, or folded into 1 because of the overlap in § Why.
  4. A revise step from `revises` in `canon simulate`, and invalidation loops as a finding (story:canon-simulate § "What invalidation loops wait for").

## Acceptance

`crates/canon/tests/judgement_marking.rs::judgement_marking_compiles_and_means_nothing_yet` passes over `fixtures/investigation/judgement-marking.yaml`:

1. A judgement action with `revises: [spec]` and `{evidence: spec_source, decided_by: program}` compiles. Its IR has `"author": "judgement"` and `"revises": ["spec"]`, and the production carries no `decided_by` key.
2. A production on that action without `decided_by` has `"decided_by": "judgement"` in the IR.
3. `author: program` written out gives the same IR bytes as no `author`.
4. `canon compile` of `fixtures/investigation/protocol.yaml` gives the bytes it gives at `557e75b`.
5. `revises: [missing]` is refused as `undeclared-artifact` naming the action, and `author: llm` is refused when the document is read.
6. The compiled IR reads back to the same bytes through `canon evaluate --ir`.
7. `canon evaluate` and `canon check` give the same bytes for the fixture with the three fields and with them removed.
8. `canon diff` from the unmarked fixture to the marked one prints `TIGHTENING` for `author` and `RELAXATION` for `revises`, each naming the action.

## Source

Request 10 (`docs/plan/wave-plan-2026-11-20.md:140-143`); `$HOME/beyond10x/sdc/docs/loom-sdk-and-agent-node.md`; `$HOME/beyond10x/sdc/docs/canon-research/02-branching-and-universes.md`; Atlas ADR 0083; the canon tree at `557e75b` and `impl/check-subject-slices` at `8d8dc31`. 2026-11-20.

## Decided

Option A of § Decision needed: the three fields `author`, `revises` and `decided_by`, with the enum `canon.protocol.Author`. The `canon diff` directions of § Outcome 5 stand unless the unit finds that a precedent in the tree contradicts them; it then reports the precedent and builds what the precedent says. Decided on 2026-11-21 by the wave coordinator on the drafter's recommendation; the operator accepted the story without choosing (`approval-record:accept-sdc-requests-9-11-2026-11-21`). The unit builds this option and does not reopen the choice.

## Scope as built

Written by the wave coordinator at the close of wave 2026-11-20-w5 from the implementor's reports, the adversary's report (`review-result:adversary-judgement-marking-pass-1`) and the merge of `impl/judgement-marking` (`5b37414`). It corrects the body where the two differ.

- The `revises` check is validation step 9, not 8; its place in the order is the one § Outcome 2 states: after the action's `may_produce` kinds, each listing of `revises` in turn, then the subject of `case_open`.
- A change of `author` also prints one `canon diff` line for each kind of that action that declares no `decided_by` and is produced in both revisions, because `canon-ir/1` holds the resolved value; the lines come in kind order after the `author` line. Writing the old value as `decided_by` on each production leaves only the `author` line. § Outcome 5 does not name these lines; the concept page does.
- A kind added to `may_produce` under a judgement action is `EXPANSION` only, a removed one `BREAKING` only, as before the fields existed.
- The IR read-back refuses every spelling the writer does not write as `malformed-input`: `author` or `decided_by` written as `program`, any other value, `revises` empty, unsorted, repeated or naming an undeclared artifact, a production listed twice. A `decided_by` on a kind the action may not produce cannot be written in `canon-ir/1`, because the key sits on the production entry.
- Imports: an imported action keeps the three fields. Two imports of one action that differ in a resolved value are `conflicting-imports`; two spellings of one resolved value are one declaration. A root action may revise an artifact only an import declares; an import that revises an artifact only the root declares is refused as `invalid-import`, as a `case_open` subject is.
- No precedent in `crates/canon/src/diff/mod.rs` contradicts the directions of § Outcome 5; they are built as written.
- Only the story's named test was written before the code; four more tests in `crates/canon/tests/judgement_marking.rs` and four unit tests were written after it. The adversary's 98 compiled documents, 31 hand-edited IRs and 20 source forms found nothing they miss.
- In § Scope and not edited: `crates/canon-docs/src/source.rs`, `crates/canon/tests/ess_model_matches.rs`.
- The follow-up "a revise step from `revises` in `canon simulate`" of § Order is story:simulate-revise-step.
