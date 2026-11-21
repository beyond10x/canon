---
format: aep.planning-md/3
id: story:child-outcome-recovery
kind: story
status: active
title: 'Set aside only the revised subject''s child outcomes in canon simulate''s revise step, so a parent''s recovery under `revision: current` is walked'
tags:
- from-request
refs:
- provider: sdc
  reference: request-17
relations:
- decomposes: epic:protocol-analysis
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:simulate-revise-step
- depends_on: story:simulate-child-outcome-step
- depends_on: story:child-outcome-subject
scope:
- confidence: cited
  path: conformance/scenarios/child-outcome-revision.yaml
- confidence: inferred
  path: crates/canon-cli/src/lib.rs
- confidence: inferred
  path: crates/canon-cli/tests/canon_simulate.rs
- confidence: inferred
  path: crates/canon/src/check/space.rs
- confidence: cited
  path: crates/canon/src/simulate/mod.rs
- confidence: cited
  path: ess/domains/cli.yaml
- confidence: inferred
  path: fixtures/investigation/simulate/revise-child-current/
- confidence: inferred
  path: fixtures/investigation/simulate/revise-child-late/
- confidence: inferred
  path: fixtures/investigation/simulate/revise-child-other-subject/
- confidence: inferred
  path: fixtures/investigation/simulate/revise-child-two-subjects/
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T07:04:05Z", actor: "human:timo", revision: 13, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w6"}
- {from: "proposed", to: "active", at: "2026-11-21T07:04:05Z", actor: "human:timo", revision: 14, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w6"}
---
## Outcome

The evaluator needs no change. `{children: {in: [...], revision: current}}` already lets a parent recover after a child ends with an outcome the form does not list. Three conditions apply: the parent revises the artifact that child was opened about, opens new children with new ids, and those children end with listed outcomes. This story closes the one gap that is left, which is in `canon simulate`'s revise step (story:simulate-revise-step). It also pins the recovery in conformance.

1. **Walk dimensions of `case_outcome`.** The walk keeps one `case_outcome` dimension for each distinct `case_open` subject. Today it keeps the space's single unbound dimension, which is about the least artifact id. A child-outcome step (story:simulate-child-outcome-step) adds its record about the `case_open` subject of the marked opening action. A `children` form with `subject` (story:child-outcome-subject) reads the dimension of that subject. `canon check` and its space do not change.
2. **Revise step.** A revision of `a` makes earlier only the `case_outcome` records about `a`. Records about any other artifact stay current.
3. **Mark.** The mark of an opening action takes three values: unset, set, and set-before-a-revision-of-its-subject. A revise of `a` moves the set marks of openings about `a` to the third value. From the third value, a child-outcome step offers the record as earlier, written `` … earlier ``, and as current. A step of the opening action leaves the third value as it is.
4. **Conformance.** One step is added to `conformance/scenarios/child-outcome-revision.yaml` (CANON-COMPOSE-002) to pin the request's real case.
5. A protocol without `case_open`, or with exactly one `case_open` subject that is also the least artifact id, gives byte-identical output.

**Why both claims hold under 1 to 3.** These rest on two runner duties. From story:child-outcome-subject: a record's `subject` is the `case_open` subject. From story:child-outcome-revision, refined on 2026-11-21: its revision follows rule (a) or rule (b).
- *Every printed finding is true:* a failed record stays current only while its own subject is unrevised. The evaluator reads it in exactly that case (`crates/canon/src/eval/composition.rs:38-53`).
- *No dead end goes unreported:* a revise of `a` no longer sets aside a child opened about `b`. The current successor of value 3 covers a rule (a) child that ends against the parent's new revision.
- **Limit:** under value 3, a printed state can hold a current failure from an old child. That is a real case only for a runner that shows an open child the parent's new revision.

## Why

- **Request 17** (`$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/requests/19/request.md`) and SDC limitation 4 (`$HOME/beyond10x/sdc/docs/protocol-notes.md:171-175`).
- **The real case by hand.** `requests/19/work/real-case.sh` runs `sdc.epic` at `25e0709`. Output is in `real-case.out`.
  - Records: spec child `passed` about `spec` s1, plan child `passed` about `epic` e1, story child `passed` about `spec` s1, release child `withheld` about `epic` e1.
  - At e1: `children.passed` is `false` and `not_released` is legitimate.
  - The epic moves to e2, and new plan and release children end `passed` about e2.
  - With the protocol as written (no key), `children.passed` stays `false` at e2. This was run.
- **The children each rule covers** (artifact lists read in `$HOME/.local/state/worktree/trees/b10x/sdc/sdc-roadmap-read-precondition/protocol/` at `25e0709`):
  - Rule (b) covers the release child (`sdc.release` declares `spec` and `release`, not `epic`) and the plan child.
  - Rule (a) covers the spec and story children (both protocols declare `spec`).
- **What the key evaluates to after the move.** This is read from code (`composition.rs:38-81`) and from step `an-abandoned-outcome-about-an-earlier-revision` of the scenario (`child-outcome-revision.yaml:232-274`). It was not run, because the copied binary cannot compile the key.
  - **Rule (b):** the `withheld` record carries e1. e1 is not current, so it is not read. Right after the move, the form is `true` over the spec and story children alone. Once the new children end: `true`, and `passed` is legitimate. If the new release child ends `withheld`: `false`.
  - **Rule (a),** if `sdc.release` declared `epic`: the record carries the child's own e1 when it ended before the move, so the result is the same. If the child terminated against e2, the record carries e2 and is read. It then stays `false` until the epic moves again, and that is correct.
  - **The missed case is not in the key.** A failed child opened about another artifact stays read: a story child `dropped` about `spec` s1 keeps the form `false` after the epic moves. Only a revision of `spec` sets it aside. This is the form's stated rule ("current revision of their `subject`"), and it is also what the request words ask for.
- **What SDC's protocols must write** (inferred from the probe):
  - `revision: current` on `children.passed` (`sdc-epic.protocol.yaml:58-62`).
  - A write action with `revises: [epic]` and no `may_produce`. `canon-sim check` gives 36864 states with or without such an action (the probe was written without `revises`, which the binary predates).
  - `not_released` must not end the tree at once. Today it is legitimate as soon as the release fails (`:220-230`).
  - After the move, `spec.opened`, `plan.opened`, `stories.opened` and `release.opened` are `unknown`, because their records are bound to e1. `spec.open` is admissible again (probe), so the spec stage reopens too.
- **The gap in the walk.** In `crates/canon/src/check/space.rs:439-441`, an unbound kind gets one dimension about the first artifact id that no match names. The IR sorts the artifacts as `epic`, `roadmap`, `spec` (compile output), so in `sdc.epic` the walk records every child about `epic`. Read literally, story:simulate-revise-step Outcome 2 then gives:
  - Revising `epic` sets aside the failed spec and story children too. That hides a dead end, against the second claim.
  - In a protocol whose revised artifact is not the least id, the recovery is never reached. That prints a false finding, against the first claim.
- **What `canon check` already covers.** Its space already reaches the recovered state: a history dimension holding the failed class as an earlier record, with current `passed` (`crates/canon/src/check/mod.rs:221-245`). The key adds no dimension (`:233-237`). `STATE_BOUND` stays 65,536 (`:394`).
- **Found, out of scope:** R2a keeps an expired `case_outcome` record at `r0` (`space.rs:2488`). A `revision: current` form then does not read it, while the evaluator does, because history holds expired records (`eval/mod.rs:479`). It matters only when an evidence match also reads `case_outcome`, which no SDC protocol does.

## Decision needed

Language options (point 2 of the request). Each one is weighed against the key.

| Option | `ess/`, model, IR, evaluator | Fits `protocol/1`? | `duplicate-child-case` | Cost |
|---|---|---|---|---|
| L0. Nothing: `revision: current` | none | yes | unchanged; a reopened child is a new child with a new id | covers the real case |
| L1. Form reading children "opened at the current revision" | `canon-evidence/1` gains an opening revision on `case_outcome`, plus a form key, IR, `composition::read` | no: no Canon input records an opening, and the one record per child is replaced at termination (`composition.rs:121-152`) | unchanged | never true for a child that writes its subject, which is why runner duty 1 was withdrawn on 2026-11-20 |
| L2. A refusal on which revision a record carries | evaluator refusal | no: Canon cannot see the child's snapshot | unchanged | unenforceable |
| L3. A key on `case_open` | model, IR, validate, diff | yes | unchanged | it changes no value; the form reads records, not openings |

Walk options (what this story builds).

| Option | Walk | Claims | Cost |
|---|---|---|---|
| W0. Revise-step as written | unbound dimension about the least artifact id | both can fail (see § Why) | none |
| W1. § Outcome 1-3 | one dimension per `case_open` subject, marks with 3 values | both hold, within the stated limit | marks go from 2^k to 3^k, and up to one dimension per subject |
| W2. A revise makes every `case_outcome` record earlier | none new | hides dead ends | none |
| W3. Protocols write `subject` (child-outcome-subject) | bound dimensions in `check` too | holds | `check` product: 2 forms is 72² where 1 form is 72 (`protocol-notes.md:148`); `sdc.epic` goes over 65,536 |

Recommendation: L0 for the language and W1 for the walk. W1 is the only walk option that keeps both claims without growing `canon check`'s space.

## ESS first

- **First commit:**
  - `ess/domains/cli.yaml` `canon.cli.Simulate`: comments on `response` and `rejected` (the revise step sets aside records about the revised subject; marks have 3 values), with `# read from:` this story.
  - Fixtures `fixtures/investigation/simulate/revise-child-current/`, `revise-child-other-subject/`, `revise-child-two-subjects/` and `revise-child-late/`, each a `protocol.yaml` and a hand-written `report.txt`.
  - The scenario step of § Acceptance 5.
- **Red on that commit:** `simulate_sets_aside_child_outcomes_of_the_revised_subject` fails.
  - `revise-child-current` exits 1. Its children are about `b`, and the unbound dimension is about `a`.
  - `revise-child-other-subject` exits 0 where one finding is expected.
  - This is inferred from revise-step Outcome 2. The unit records the run. The model does not change.

## Scope

- **In:** the walk dimensions per `case_open` subject, the revise rule, the 3-valued marks, rendering, fixtures, the test and the scenario step. — cited
- **Out:**
  - any language option (L1 to L3). — cited
  - SDC's protocol changes. — cited
  - the R2a expiry discrepancy (file separately). — cited
  - `canon check`'s space. — cited
- **Surfaces:**
  - `crates/canon/src/simulate/mod.rs` — cited
  - `crates/canon/src/check/space.rs` (`walk_evidence`, `render_walk`) — inferred
  - `crates/canon-cli/src/lib.rs` (`Simulate` doc) — inferred
  - `crates/canon-cli/tests/canon_simulate.rs` — inferred
  - `ess/domains/cli.yaml` — cited
  - `fixtures/investigation/simulate/revise-child-*/` — inferred
  - `conformance/scenarios/child-outcome-revision.yaml` — cited
- **Confidence:** medium. The evaluator is read and partly run. The walk is read from two unbuilt stories.

## Order

- **It must follow all three active stories on this code.**
  - story:simulate-child-outcome-step: it extends that story's marks and step.
  - story:simulate-revise-step: it narrows that story's Outcome 2. That story is in wave 8, and SDC fixed the order child-outcome step, record-groups, revise step. So this story goes in wave 9 or later.
  - story:child-outcome-subject: it relies on that story's runner duty (record subject is the `case_open` subject) and shares `space.rs`.
- **Answers a question left open:** story:simulate-child-outcome-step § Order asks what a revision does to a mark. Point 3 answers it. If story:simulate-revise-step decides it first, this story follows that answer.
- **Shares files:** `crates/canon-cli/src/lib.rs` and `ess/domains/cli.yaml` with story:canon-mutate and story:protocol-floor.

## Acceptance

`crates/canon-cli/tests/canon_simulate.rs::simulate_sets_aside_child_outcomes_of_the_revised_subject` passes. Every fixture declares artifacts `a` and `b`. Action `open` has `case_open` about `b`. Claim `done.current` is `{children: {in: [done], revision: current}}`. Action `fix` is admissible while that claim is `false`. Outcome `built` requires the claim.

1. `revise-child-current` (`fix` revises `b`): exit 0, summary line only. The path to `built` holds `` action `fix` revises `b` `` and, after it, the failed record rendered `` earlier ``.
2. `revise-child-other-subject` (`fix` revises `a`): exit 1, with exactly one finding. Its state holds the failed `case_outcome` record as current.
3. `revise-child-two-subjects` (adds `open_a` about `a`; `fix` revises `b`): a failed child of `open` is never in a printed state. A failed child of `open_a` is in exactly one printed state.
4. `revise-child-late`: some path holds a child-outcome step written `` … earlier `` after `` action `fix` revises `b` ``.
5. Step `recovery-after-a-child-ended-outside-the-set` of CANON-COMPOSE-002 passes under `canon conform run`.
   - Setup: spec at s1, epic at e2. Records: `withheld` about epic e1, `passed` about spec s1, and two `passed` records about epic e2.
   - The `current` form is `true` and the form without the key is `false`.
   - With the spec record `dropped` instead, the `current` form is `false`.
6. Every earlier simulate fixture gives unchanged bytes, and two runs of each new fixture give identical bytes.

## Source

Request 17, `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/requests/19/request.md`, and the brief `requests/drafter-brief-2.md`. Date: 2026-11-21.

- **Canon `wave/2026-11-20-w6` at `4c35a9e`:**
  - Files: `AGENTS.md`, `eval/composition.rs`, `eval/mod.rs`, `check/mod.rs`, `check/space.rs`, `simulate/mod.rs`, `model/requirement.rs`, `model/predicate.rs`, `model/mod.rs`, `conformance/scenarios/child-outcome-revision.yaml`, `fixtures/investigation/child-outcome-revision.yaml`, `conformance/requirements.yaml`, `docs/design/canon-protocol-calculus-design.md`, decision-blocker:outcome-decision-source.
  - Stories: case-composition, child-outcome-revision, child-outcome-subject, simulate-child-outcome-step, simulate-revise-step, decision-outcomes.
- **SDC:** `protocol/*.protocol.yaml` at `25e0709` (moved from `f364b6b`) and `$HOME/beyond10x/sdc/docs/protocol-notes.md`.
- **Run with `canon-sim`:** `compile`, `evaluate` and `check` through `requests/19/work/real-case.sh`.

## Decided

Options L0 and W1 of § Decision needed. Decided on 2026-11-21 by the session working on the SDC prototype, which the operator named the orchestrator of this plan. Its words:

> `story:child-outcome-recovery` is accepted as drafted (L0, no language change; W1 in `canon simulate`, wave 9 or later, after the child-outcome step, the revise step and story:child-outcome-subject). That `{children: {in: [...], revision: current}}` already gives the recovery is the answer I needed; SDC writes its side (the key on `children.passed`, a write action with `revises: [epic]`, `not_released` only after the reworks are spent, new child ids). The uncovered case (a failed child opened about another artifact, e.g. a story `dropped` about `spec` s1) is noted; SDC will revise the artifact the child was about.

The story was accepted under the operator's standing grant (`approval-record:accept-sdc-request-17-2026-11-21`). The unit builds W1 and does not reopen the choice. The scenario step of § Acceptance 5 uses neutral identifiers, as the "Fixture rule" of story:simulate-child-outcome-step says for protocols of another repository.
