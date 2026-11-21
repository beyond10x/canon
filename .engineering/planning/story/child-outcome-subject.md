---
format: aep.planning-md/3
id: story:child-outcome-subject
kind: story
status: active
title: Select the child cases a children form reads by the artifact they were opened about
tags:
- from-request
refs:
- provider: sdc
  reference: docs/orchestration.md
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:child-outcome-revision
scope:
- confidence: cited
  path: conformance/requirements.yaml
- confidence: cited
  path: conformance/scenarios/child-outcome-subject.yaml
- confidence: inferred
  path: crates/canon/src/check/space.rs
- confidence: cited
  path: crates/canon/src/eval/actions.rs
- confidence: cited
  path: crates/canon/src/eval/claims.rs
- confidence: cited
  path: crates/canon/src/eval/composition.rs
- confidence: cited
  path: crates/canon/src/eval/read.rs
- confidence: inferred
  path: crates/canon/src/explain/mod.rs
- confidence: cited
  path: crates/canon/src/ir/mod.rs
- confidence: cited
  path: crates/canon/src/model/predicate.rs
- confidence: cited
  path: crates/canon/src/validate/mod.rs
- confidence: cited
  path: ess/domains/protocol.yaml
- confidence: cited
  path: fixtures/investigation/child-outcome-subject.yaml
revision: 17
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T00:43:59Z", actor: "human:timo", revision: 16, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
- {from: "proposed", to: "active", at: "2026-11-21T00:43:59Z", actor: "human:timo", revision: 17, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
---
## Outcome

story:child-outcome-revision covers most of request 14. It does not cover one part: a `children` form cannot name the artifact its children were opened about. This story adds that part. When it is done, a `children` form in `protocol/1` may carry `subject`. `{children: {in: [O, ...], subject: A}}` reads only the `case_outcome` records whose `subject` is `A`. `{children: {in: [O, ...], subject: A, revision: current}}` reads only the records whose `subject` is `A` and whose `subject_revision` is the case snapshot's current revision of `A`. Over the records it reads, `false`, `unknown` and `true` follow CANON-COMPOSE-001. A form without `subject` reads records about every artifact, as it does today. `canon validate` refuses a `subject` the protocol does not declare as `undeclared-artifact`, naming the declaration. `canon-ir/1` writes `"subject"` only when the protocol declares it, so the IR of every existing protocol stays the same byte for byte. A blocked action's reason and the explanation name the form with its `subject`. The protocol reference, generated from `ChildOutcomes`, states two runner duties beside those of story:child-outcome-revision. First, a `case_outcome` record's `subject` is the `case_open` subject of the action that opened the child. Second, the runner writes an `open` record when it opens a child. No new refusal code is added.

## Why

- Request 14 (2026-11-21) wants `children` scoped to the children opened while a named artifact (the wave) was at its current revision. `$HOME/beyond10x/sdc/docs/orchestration.md:228-234` says why the draft avoids the form: one `dropped` child makes it false for good. The draft reads one `wave_result` per wave instead (`:223-226`, `$HOME/beyond10x/sdc/protocol/draft/orchestrate.protocol.yaml:69-74`, `:117-123`). The document says this request extends request item 5, which became story:child-outcome-revision.
- What story:child-outcome-revision already gives (its § Outcome, § Decided, and the runner duty refined on 2026-11-21):
  - `sdc.story` declares `requirements`, `findings`, `spec` and `change`, and not `wave` (`$HOME/beyond10x/sdc/protocol/sdc-story.protocol.yaml:19-30`). Runner duty case (b) therefore applies: a story child's record carries the parent's revision of `wave` at the moment the child was opened. `revision: current` then reads exactly the children opened at the current wave revision.
  - A superseded wave's children are not read. This includes one that ended `dropped` (`sdc-story.protocol.yaml:176`) and one still `open`. The request's "dropped child" is that outcome, and that part is covered.
  - If a child protocol declared the artifact, case (a) would apply: the record carries the child's own revision when it terminates. The form would then mean "computed against the current revision". That equals "opened at the current revision" only when the child's final revision of the artifact equals the parent's revision at opening. Canon compares the value only, so its rule is the same either way. Case (a) does not arise for `wave` in the draft.
- What it does not give. The form names no artifact. story:child-outcome-revision compares each record with the current revision of the record's own `subject`. `ChildOutcomes` has only `in` (`crates/canon/src/model/predicate.rs:262-269`, `ess/domains/protocol.yaml:733-741`). The brief's "form's subject" does not exist.
  - `sdc.orchestrate` opens children about two artifacts: `ideas.dispatch` opens `sdc` about `intake` (`orchestrate.protocol.yaml:179-181`), and `wave.dispatch` opens `sdc.story` about `wave` (`:237-239`).
  - The outcomes of `sdc` are `operating` and `abandoned`, never `passed` (`$HOME/beyond10x/sdc/protocol/sdc.protocol.yaml`, `outcomes:`).
  - So `{children: {in: [passed], revision: current}}` is `false` once an idea child of the current intake revision ends `operating`, and `unknown` while one is `open`. It is not "every story of this wave passed". `$HOME/beyond10x/sdc/docs/protocol-notes.md:163-167` records the same gap: the form "cannot tell a story child from the release child".
- A child that never produced a `case_outcome` record:
  - Canon reads `case_open` in parsing (`eval/read.rs:365`), validate, IR, check and diff. No evaluation stage reads it, and Canon opens nothing itself (`ess/domains/protocol.yaml:186-196`).
  - `composition::value` (`crates/canon/src/eval/composition.rs:31-47`) iterates only the `case_outcome` records it is given.
  - No Canon input says a child was opened. The `wave_dispatched` record is the protocol's own evidence and has no count. Canon cannot compare the records with `arguments.stories`.
  - "Every" therefore ranges over the `case_outcome` records the evidence check accepted, narrowed by the form's filters. Example: a wave of five stories, two records `passed`, three absent. The form is `true`.
  - `ess/domains/protocol.yaml:866-869` says a record is `open` while the child runs. It does not say a record exists from the moment of opening. That duty belongs to the runner.
- Boundary: writing the records, with their `subject` and an `open` record at opening, is the runner's job (Loom). Canon owns which records the form selects. `eval/binding.rs:27-33` already refuses a record whose subject is not declared, so every record's `subject` is a parent artifact.
- Precedent: an evidence match already takes an optional `subject` (`ess/domains/protocol.yaml:541-556`, story:subject-bound-evidence-match).
- The form does not replace `wave_result`. `wave.delivered` also states that the gate passed on the integration branch (`orchestrate.protocol.yaml:69-74`), and no child outcome carries that. Whether to use the form there is the prototype's choice.
- Could not verify:
  - No `canon` binary is on `PATH`, so nothing was run.
  - Loom was not read. That Loom writes the `case_open` subject as the record's `subject`, and writes an `open` record at opening, is unverified.

## Decision needed

| | Option | Change to `ess/` and the model | Cost |
|---|---|---|---|
| A | `subject: A` on the `children` form | `ChildOutcomes.subject: Optional<canon.protocol.ArtifactId>`. The form keeps the records whose `subject` is `A`. | One field. It uses a field every record already carries, with validation code `undeclared-artifact`. Two actions that open children about one artifact stay mixed (none do in the draft). |
| B | `protocol: P` on the form | The form gains a field. `canon-evidence/1` gains `child_protocol` on `case_outcome` records, with refusals like `misplaced-child-case`. | The runner writes a new field that Canon cannot check, and every existing record lacks it. This changes two documents. |
| C | No language change | None. The prototype keeps idea children and story children in separate cases, or keeps `wave_result`. | The SDC restructures its protocol. "Every story of this wave passed" stays a program's record, which Canon cannot tell from a forgery (`orchestration.md:231-233`). |

Recommendation: option A. It is the smallest change that makes the request a Canon claim, and it follows the existing `subject` key on evidence matches.

## ESS first

- First commit:
  - In `ess/domains/protocol.yaml`, add the field `subject: Optional<canon.protocol.ArtifactId>` to `canon.protocol.ChildOutcomes`, with its `# read from:` comment, and extend the type's comment.
  - Add `CANON-COMPOSE-003` to `conformance/requirements.yaml`.
  - Add the fixture `fixtures/investigation/child-outcome-subject.yaml` and the scenario `conformance/scenarios/child-outcome-subject.yaml`.
- Red on that commit:
  - `crates/canon/tests/ess_model_matches.rs` fails, naming `ChildOutcomes.subject`.
  - `canon conform run --requirements conformance/requirements.yaml` fails `CANON-COMPOSE-003` at `compile`, because `#[serde(deny_unknown_fields)]` on `ChildOutcomes` (`predicate.rs:262-263`) refuses `subject`.

## Scope

- In: the `subject` key on the `children` form, with its parsing, validation, IR form, evaluation, reasons and explanation; the two runner-duty sentences in the generated reference; scenario `CANON-COMPOSE-003`. — cited
- Out:
  - The `revision` key and its runner duty (story:child-outcome-revision). — cited
  - Writing records and counting the children opened against `arguments` (runner, Loom). — cited
  - A refusal for a `subject` that no `case_open` names. The request does not ask for one. — inferred
  - A filter by child protocol (option B). — cited
- Surfaces:
  - `ess/domains/protocol.yaml` — cited
  - `crates/canon/src/model/predicate.rs` — cited
  - `crates/canon/src/eval/read.rs`: parses the form at `:476-481`. — cited
  - `crates/canon/src/eval/composition.rs` — cited
  - `crates/canon/src/eval/claims.rs`: `:188`, `:236`. — cited
  - `crates/canon/src/eval/actions.rs`: reasons are keyed by outcome list at `:187`, `:323`. — cited
  - `crates/canon/src/ir/mod.rs`: `:363`, `:565`, `:909-912`. — cited
  - `crates/canon/src/validate/mod.rs` — cited
  - `crates/canon/src/explain/mod.rs` — inferred
  - `crates/canon/src/check/space.rs`: `Read.subject` exists. Whether a filtered form reads fewer dimensions is not established. — inferred
  - `conformance/requirements.yaml` — cited
  - `conformance/scenarios/child-outcome-subject.yaml` — cited
  - `fixtures/investigation/child-outcome-subject.yaml` — cited
  - `website/status.yaml` — inferred
- Confidence: medium. The evaluation and IR sites are cited. The effect on `check` is inferred.

## Order

- Depends on story:child-outcome-revision (wave 7). It adds `revision`, rekeys the reasons by the whole form, and edits the same lines in every evaluation and IR surface listed above. This story lands in a later wave.
- Shares `ess/domains/protocol.yaml`, `model/predicate.rs`, `ir/mod.rs`, `validate/mod.rs` and `conformance/requirements.yaml` with story:revision-ordering (wave 8).
- Shares `eval/composition.rs` with story:evidence-redelivery (wave 8). That story leaves the form receiving distinct records, which this story keeps.
- If either of those stays in wave 8, this story goes in wave 9 or later.
- Shares `check/space.rs` with story:canon-simulate (wave 4). Its walk adds a `case_outcome` record in one dimension of that kind; this story leaves that unchanged.
- Contradicts none of story:supplied-evidence, story:missing-evidence, story:pending-decisions, story:decision-grounds or story:judgement-marking. It adds no evidence kind, no producer and no decision.

## Acceptance

Conformance scenario `CANON-COMPOSE-003` passes under `canon conform run` over `fixtures/investigation/child-outcome-subject.yaml`.

The fixture declares artifacts `intake` and `wave`. Action `ideas.open` has `case_open` about `intake`, and action `stories.open` has `case_open` about `wave`. Claim `wave.passed` is `{children: {in: [passed], subject: wave, revision: current}}`. Claim `all.passed` is the same form without `subject`. Outcome `wave.done` requires `wave.passed`.

1. `compile` writes `"subject": "wave"` for `wave.passed`, and no `subject` key for `all.passed`.
2. Wave is at `w2`. Child `c1` is `passed` about `w1`, `c2` is `dropped` about `w1`, `c3` and `c4` are `passed` about `w2`, and `i1` is `open` about the current `intake`. `wave.passed` is `true` and `all.passed` is `unknown`.
3. As in 2, with `i1` `operating`: `wave.passed` is `true` and `all.passed` is `false`.
4. As in 2, with `c4` `dropped` about `w2`: `wave.passed` is `false`. With `c4` `open` instead: `wave.passed` is `unknown`.
5. Only `c1` and `c2` are given (both about `w1`): `wave.passed` is `unknown` and `wave.done` is blocked. The blocked action's reason names the form with `"subject": "wave"`.
6. Only `c3` is given, with `c4` absent: `wave.passed` is `true`. This pins that Canon counts only the records it is given.
7. `subject: nope` is refused as `undeclared-artifact`, naming claim `wave.passed`.
8. `CANON-COMPOSE-001` and `CANON-COMPOSE-002` still pass unchanged.

`crates/canon/tests/ess_model_matches.rs` passes.

## Source

Request 14 from the session working on the SDC prototype, 2026-11-21 (not an item of `canon-requests.md`). Date of this story: 2026-11-21.

- Files read in the sdc directory: `docs/orchestration.md` § 2.1–2.6, `protocol/draft/orchestrate.protocol.yaml`, `protocol/sdc-story.protocol.yaml`, `protocol/sdc.protocol.yaml` and `docs/protocol-notes.md:158-173`.
- Files read in the canon tree at `c295fae`: `AGENTS.md`, `eval/composition.rs`, `eval/read.rs`, `eval/claims.rs`, `eval/binding.rs`, `model/predicate.rs`, `ir/mod.rs`, `check/mod.rs`, `check/space.rs`, `ess/domains/protocol.yaml` and `conformance/requirements.yaml`.
- Stories read: story:child-outcome-revision in full, plus the child-related passages of the eight stories named in § Order.

## Decided

Option A of § Decision needed: an optional `subject` on the `children` form. Decided on 2026-11-21 by the wave coordinator on the drafter's recommendation; the story was accepted under the operator's standing grant without a choice among the options (`approval-record:accept-sdc-request-14-2026-11-21`). The unit builds this option and does not reopen the choice.
