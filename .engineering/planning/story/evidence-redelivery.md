---
format: aep.planning-md/3
id: story:evidence-redelivery
kind: story
status: implemented
title: Count an equal re-delivered evidence record once and refuse a differing one, so an at-least-once runner can resume a crashed run
tags:
- from-request
refs:
- provider: sdc
  reference: docs/canon-research/01-event-stream-and-resume.md
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: conformance/requirements.yaml
- confidence: inferred
  path: conformance/scenarios/evidence-redelivery.yaml
- confidence: inferred
  path: crates/canon-cli/src/evaluate.rs
- confidence: cited
  path: crates/canon-cli/tests/adversary2_review_hardening.rs
- confidence: cited
  path: crates/canon-cli/tests/adversary_conformance_suite_cli.rs
- confidence: cited
  path: crates/canon-cli/tests/adversary_review_hardening.rs
- confidence: cited
  path: crates/canon-cli/tests/conform_requirements.rs
- confidence: cited
  path: crates/canon-cli/tests/evidence_file_named.rs
- confidence: cited
  path: crates/canon/src/eval/evidence.rs
- confidence: cited
  path: crates/canon/src/eval/mod.rs
- confidence: cited
  path: crates/canon/tests/review_refusal_naming.rs
- confidence: cited
  path: ess/domains/protocol.yaml
- confidence: inferred
  path: fixtures/investigation/evidence-redelivery.yaml
- confidence: inferred
  path: website/docs/concepts/evaluation.md
- confidence: inferred
  path: website/docs/reference/evaluation.md
revision: 21
transitions:
- {from: "draft", to: "proposed", at: "2026-11-20T23:28:16Z", actor: "human:timo", revision: 17, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
- {from: "proposed", to: "active", at: "2026-11-20T23:28:16Z", actor: "human:timo", revision: 18, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
- {from: "active", to: "implemented", at: "2026-11-21T11:37:24Z", actor: "human:timo", revision: 21, decided_on: {"recorded":{"test_result":1,"verification":1}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w6"}
---
## Outcome

A `canon-evidence/1` record given more than once in one evaluation counts once when every copy is equal to the first. Copies that differ are still refused. After this, a runner that delivers at least once can re-evaluate its stored inputs after a crash, and Canon does not refuse every later evaluation of the case.

1. **Equal copy.** Two records with one `id` are equal when `format`, `kind`, `result`, `subject`, `subject_revision`, `observed_at`, `producer` and `child_case` are equal as written, and `upstream_revisions` holds the same pairs in any order. `observed_at` is compared as written text: `12:00:00Z` and `12:00:00.000Z` differ, because precision sets the order resolution (`crates/canon/src/eval/order.rs:16-24`).
2. **Evaluation.** `eval::evaluate_with` evaluates the distinct records, keeping the first copy in the order given. Every later stage sees each id once: binding, freshness, invalidation, the order-form history, `composition::one_record_per_child` and `explanation.computed_from.evidence`. A decision computed with an equal copy is byte-identical to the decision computed without it.
3. **Differing copy.** It is refused as `duplicate-identifier` (code unchanged), naming the record: `` evidence `e1` is given more than once with different content ``.
4. **`case_outcome`.** An equal copy of a `case_outcome` record is not refused as `duplicate-child-case`.
5. **`canon evaluate`.** A refusal still names every file that holds a record of the cited id (`crates/canon-cli/src/evaluate.rs:85-97`, unchanged).
6. **Unchanged.** No field, IR byte, command or flag changes. The duplicate rules for `canon-decisions/1` and `canon-authority/1` stay as they are.

## Why

- **The request.** Request 11 is not an item of `$HOME/beyond10x/sdc/docs/canon-requests.md`. It is recorded in `docs/plan/wave-plan-2026-11-20.md:144-153` of the canon tree. The draft body it points to is `$HOME/beyond10x/sdc/docs/canon-research/01-event-stream-and-resume.md`.
- **The boundary.**
  - Canon evaluates one case snapshot as a pure function with an explicit instant (`AGENTS.md:22-26`; `crates/canon/src/eval/mod.rs:3-8`).
  - Canon is not "an event store" or "a workflow engine" (design § 33, `docs/design/canon-protocol-calculus-design.md:1460-1470`).
  - So a run resumes from Canon's side when the runner re-evaluates its stored inputs at an instant it supplies. Appends, the hash chain, locks, the clock, executing actions and reconciling them belong to the runner (Loom, the SDC driver).
- **The one Canon-side blocker found.**
  - `evidence::check` refuses any repeated id, equal or not (`crates/canon/src/eval/evidence.rs:90-97`).
  - Three tests assert that equal copies are refused: `eval/mod.rs:827-832`, `crates/canon-cli/tests/evidence_file_named.rs:50-53` and `crates/canon-cli/tests/adversary2_review_hardening.rs:96-104`.
  - No story and no design text gives the reason for that rule; I searched both.
  - Runners already diverge. Loom's `CanonGovernor` keeps the first record of a repeated id whatever its content, and Loom's `evaluate` refuses the request (`$HOME/beyond10x/loom/crates/loom-governor/src/lib.rs:1974-1977`, `:2033-2043`). Keeping the first copy silently drops a conflicting observation.
- **The draft's findings, checked against the tree at `557e75b`.**
  - "Not a pure fold of evidence": true. The caller supplies the case snapshot (`eval/mod.rs:407-415`). This is by design and is not a defect.
  - "A new instant changes which evidence expired": true (`eval/freshness.rs:6-10`). The instant is already explicit, so the runner logs it. No Canon change is needed.
  - Duplicate id: true. It also applies to the other inputs:
    - an equal `canon-decisions/1` entry is refused (`eval/decisions.rs:23`, `:97-108`);
    - a second `canon-authority/1` decision for one capability at an equal scope is refused whatever its value (`eval/authority.rs:18-26`).
  - No "started, result unknown" status: true. There are three action statuses (`eval/mod.rs:10`). `EffectClass` is only read into the IR (`eval/read.rs:358`; `model/ids.rs:173-176`).
  - `computed_from` lists evidence ids, not content, and carries the Canon version: true (`explain/mod.rs:305-315`).
  - The SDC crash findings are stale. The driver now:
    - writes through a temporary file and a rename (`$HOME/beyond10x/sdc/driver/src/main.rs:100-105`);
    - appends to `events.jsonl` (`:192-202`);
    - takes a lock (`:107-109`);
    - writes the snapshot after the evidence and `action_finished` (`:690-691`; `docs/driver.md:89`).
  - The SDC driver stores one file per id (`main.rs:705`, `:768`), so it cannot give Canon a repeated id. The at-least-once source is Loom's "proposed again after a restart" (`loom-executor/src/workflow.rs:76-77`, as the draft cites it).
- **Not verified.** Nothing was built or run. I did not re-read Loom's `workflow.rs`.

## Decision needed

| | Rule | `ess/` and model | Cost |
|---|---|---|---|
| A | Equal copy counts once; a differing copy is refused `duplicate-identifier`, new message | Comment on `canon.protocol.EvidenceRecord.id`. No field. Equality in `eval/evidence.rs` | 3 tests that assert equal copies are refused are rewritten. Requirement-count tests move |
| B | As A, but a differing copy gets a new code, `conflicting-evidence` (the draft's) | As A, plus a new stable code in the reference | Every runner that matches `duplicate-identifier` for evidence must learn the new code |
| C | No Canon change. The reference states that the runner removes copies | None | Loom's two paths already diverge (keep first or refuse). Each runner re-implements equality |
| D | Byte-identical files only | None | The library never sees bytes. Loom calls `evidence_from_value` (`loom-governor/src/lib.rs:2022-2030`) |

Recommendation: option A, because it is the smallest change that removes the refusal blocking a resumed run while still refusing a conflicting copy, and it adds no new code. The acceptance below is for A.

## ESS first

- **First commit.**
  - The rule, in the comment on field `id` of `canon.protocol.EvidenceRecord` in `ess/domains/protocol.yaml` (`:770-783`).
  - The fixture `fixtures/investigation/evidence-redelivery.yaml`.
  - The scenario file `conformance/scenarios/evidence-redelivery.yaml`.
  - `CANON-EVIDENCE-004` in `conformance/requirements.yaml`, `status: implemented`.
- **Red on that commit.** `canon conform run --requirements conformance/requirements.yaml` fails `CANON-EVIDENCE-004`, step `equal-copy`, with `evaluation refused: duplicate-identifier`, because `evidence.rs:90` refuses the copy.
- The count lines in `conform_requirements.rs:113` and `adversary_conformance_suite_cli.rs:94` (`20 implemented`) also fail. That is expected; the implementation commits fix them.
- `ess_model_matches.rs` stays green because no field changes, so `task ess-gate` stays green.

## Scope

- **In:** Outcome 1 to 6; scenario `CANON-EVIDENCE-004`. — cited
- **Out:** — cited
  - The same rule for `canon-decisions/1` and `canon-authority/1`.
  - An input digest.
  - An action marking for safe re-runs; an `in-doubt` status.
  - A journal format and a fold.
  - Storage, locks, the clock, executing or reconciling effects (runner).
- **Surfaces:**
  - `ess/domains/protocol.yaml` — cited
  - `crates/canon/src/eval/evidence.rs` (`:39-46`, `:90-97`) — cited
  - `crates/canon/src/eval/mod.rs` (docs `:98-100`, `:415-441`, test `:827-832`) — cited
  - `crates/canon-cli/tests/evidence_file_named.rs` — cited
  - `crates/canon-cli/tests/adversary2_review_hardening.rs` — cited
  - `crates/canon-cli/tests/conform_requirements.rs` — cited
  - `crates/canon-cli/tests/adversary_conformance_suite_cli.rs` — cited
  - `crates/canon/tests/review_refusal_naming.rs` (doc comment `:4`) — cited
  - `crates/canon-cli/src/evaluate.rs` (doc comment `:26-31`) — inferred
  - `conformance/requirements.yaml` — cited
  - `conformance/scenarios/evidence-redelivery.yaml` (new) — inferred
  - `fixtures/investigation/evidence-redelivery.yaml` (new) — inferred
  - `website/docs/reference/evaluation.md` (generated, `:100`) — inferred
  - `website/docs/concepts/evaluation.md` (`:51-52`) — inferred
  - `website/status.yaml` — inferred
- **Not edited:** `model/evidence.rs`, `eval/composition.rs`, `explain/mod.rs`. They receive the distinct set. — inferred
- **Confidence:** high for the evaluator and the tests (read); medium for the docs.

## Order

- **Depends on:** none.
- **Shares files without an edge:**
  - `eval/mod.rs`: story:missing-evidence, story:child-outcome-revision, story:authority-revision-binding.
  - `ess/domains/protocol.yaml`, `conformance/requirements.yaml` and the two requirement-count tests: all five stories in flight. story:supplied-evidence shares only these.
  - `model/evidence.rs` (read): story:revision-ordering adds the field `item` to `EvidenceRecord`. The equality destructures the record exhaustively, so whichever story lands second covers `item`.
  - `eval/composition.rs` (edited by story:child-outcome-revision) now receives distinct records.
- **Follow-ups**, one line each:
  1. Equal-copy rule for `canon-decisions/1` entries; for `canon-authority/1` after story:authority-revision-binding.
  2. A content digest of every evaluation input in `computed_from` (ids only today, `explain/mod.rs:313`).
  3. A declared marking on a `protocol/1` action that re-running it is safe, carried to the IR and the decision.
  4. An open-attempt input and an `in-doubt` action status (decision: new document or evidence kind); after 3.
  5. A run-journal document format and a pure fold from a journal prefix to the evaluation inputs (`CANON-RUN-*`). This needs a decision against design § 33 first.

## Acceptance

Scenario `CANON-EVIDENCE-004` passes under `canon conform run`, with these expectations:

1. Evidence with an equal copy of one record gives a decision, explanation included, byte-identical to the step without the copy. `computed_from.evidence` lists the id once.
2. An equal copy whose `upstream_revisions` keys are written in another order gives the same bytes as (1).
3. An equal copy of a `case_outcome` record gives a decision, not `duplicate-child-case`, with the `children` value of the single-record step.
4. A copy that differs only in `result` is refused as `duplicate-identifier`.
5. A copy that differs only in `observed_at` precision (`12:00:00Z`, `12:00:00.000Z`) is refused as `duplicate-identifier`.
6. Placing the copy first or last in the list gives the same bytes.
7. `crates/canon-cli/tests/evidence_file_named.rs`: two files holding equal records exit 0 with the same decision as one file; two differing files exit 1 naming both files.

## Source

- Request 11 (`docs/plan/wave-plan-2026-11-20.md:144-153`), and `$HOME/beyond10x/sdc/docs/canon-research/01-event-stream-and-resume.md`.
- Read in `$HOME/beyond10x/sdc/`: `docs/canon-requests.md`, `docs/driver.md`, `driver/src/main.rs`.
- Read in `$HOME/beyond10x/loom/`: `crates/loom-governor/src/lib.rs`, `src/file_store.rs`.
- Read in the canon tree at `557e75b`:
  - `AGENTS.md`, design § 21.2, § 33, § 37
  - `eval/{mod,evidence,order,freshness,composition,decisions,authority}.rs`, `explain/mod.rs`
  - `model/{evidence,ids,mod}.rs`, `ess/domains/protocol.yaml`, `conformance/requirements.yaml`
  - the stories in flight
- Date: 2026-11-20.

## Decided

Option A of § Decision needed: an equal copy counts once; a differing copy is refused as `duplicate-identifier` with the new message. Decided on 2026-11-21 by the wave coordinator on the drafter's recommendation; the operator accepted the story without choosing (`approval-record:accept-sdc-requests-9-11-2026-11-21`). The unit builds this option and does not reopen the choice.

## Scope as built

Written by the wave coordinator at the close of wave 2026-11-20-w6 from the implementor's report, the adversary's report (`review-result:adversary-evidence-redelivery-pass-1`, green, no finding) and the merge of `impl/evidence-redelivery` (`540ad15`). It corrects the body where the two differ.

- `eval::evidence::check` returns the distinct records: the first copy of each id in the order given. `evaluate_with` evaluates those, so every later stage, `composition::one_record_per_child` included, sees each id once.
- Equality is decided by `same_record`, which names every field of `EvidenceRecord`: a field added to the record later does not compile until the comparison names it. `observed_at` is compared as written text; `upstream_revisions` as sorted pairs. Every field is a string, so the two builds of `serde_json` cannot differ here.
- A differing copy is refused where the id is checked: after the format, the identifiers and `observed_at` of that record, before its kind and its upstream artifacts. A record refused for an earlier reason is refused for it first. This is the reference's existing rule (inputs are checked in the order given and the first problem is the refusal); the unit made the outcome less dependent on order than the base was (`[a, a, x_bad]` and `[x_bad, a, a]` are now both refused for `x`).
- Which equal copy is kept cannot be seen in any output: equal copies differ at most in the order of upstream keys, which nothing prints.
- `canon evaluate` with several evidence files reads them in sorted file-name order and still names, in a refusal, every file that holds a record of the cited id, the file of a dropped equal copy included.
- Scenario `CANON-EVIDENCE-004`. Six existing tests were re-pinned, each because its "repeated id" was an equal copy; two of them are in files § Scope did not list: `crates/canon-cli/tests/review_refusal_naming.rs` and `crates/canon-cli/tests/adversary_review_hardening.rs`.
- The scenario pins the decision provenance version (`"canon": "0.9.0"`), as `explanation.yaml` and `pending-decisions.yaml` do: a release changes it.
