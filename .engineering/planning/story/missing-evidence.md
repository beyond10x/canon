---
format: aep.planning-md/3
id: story:missing-evidence
kind: story
status: active
title: Report per admissible action which evidence kinds it may produce still lack an applicable record
tags:
- from-request
refs:
- provider: sdc
  reference: docs/canon-requests.md
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:action-admissibility
- depends_on: story:evidence-freshness
- depends_on: story:invalidation-rules
- depends_on: story:evidence-independence
- depends_on: story:explanation
- depends_on: story:invalidation-evaluation-cost
scope:
- confidence: cited
  path: conformance/requirements.yaml
- confidence: cited
  path: conformance/scenarios/explanation.yaml
- confidence: inferred
  path: conformance/scenarios/missing-evidence.yaml
- confidence: cited
  path: crates/canon-cli/tests/adversary_skel_cli.rs
- confidence: cited
  path: crates/canon-cli/tests/explanation.rs
- confidence: inferred
  path: crates/canon-docs/src/source.rs
- confidence: cited
  path: crates/canon/src/eval/decision.rs
- confidence: inferred
  path: crates/canon/src/eval/missing.rs
- confidence: cited
  path: crates/canon/src/eval/mod.rs
- confidence: cited
  path: crates/canon/src/model/decision.rs
- confidence: cited
  path: crates/canon/tests/adversary_explain_trace.rs
- confidence: cited
  path: crates/canon/tests/adversary_obligations_docs.rs
- confidence: cited
  path: crates/canon/tests/ess_model_matches.rs
- confidence: cited
  path: ess/domains/protocol.yaml
- confidence: inferred
  path: fixtures/investigation/missing-evidence.yaml
revision: 19
transitions:
- {from: "draft", to: "proposed", at: "2026-11-20T22:35:01Z", actor: "human:timo", revision: 18, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
- {from: "proposed", to: "active", at: "2026-11-20T22:35:01Z", actor: "human:timo", revision: 19, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
---
## Outcome

`canon evaluate` writes a new top-level section, `missing_evidence`, in `canon-decision/1`. It maps each **admissible** action whose `may_produce` is not empty to the kinds in that list that have no applicable record. Kinds are sorted in identifier order. An empty list means every kind the action may produce has an applicable record. Example: `{"attempt_falsification": ["falsification_attempt"], "inspect": []}`.

- A kind K lacks an applicable record when no record of kind K is in the set the revision-binding and freshness stages leave (`left`, `crates/canon/src/eval/mod.rs:423-427`), excluding every record that some claim lists in its `excluded_evidence` (as `invalidated` or `not_independent`). See Decision needed, D1.
- Not listed in the section: an action that is `blocked` or `approval-required`, and an action with an empty `may_produce`.
- The section is written only when at least one action qualifies. Other sections do not change.
- The field claims nothing else. It does not predict that running the action succeeds, that the action produces K, which result K would carry, that a new record would pass an independence demand, or that any claim would become `true`. It reads no run history (`max_runs`, "succeeded against this snapshot"). It says nothing about an action that produces no evidence. It selects and orders nothing.
- `canon.protocol.Decision` in `ess/domains/protocol.yaml` and `crates/canon/src/model/decision.rs` gain the field `missing_evidence`. No command or flag changes.

## Why

- **Request.** Item 2 of `$HOME/beyond10x/sdc/docs/canon-requests.md:18-21`.
- **Driver.** It re-derives applicability at `$HOME/beyond10x/sdc/driver/src/main.rs:162-176`: a record counts when its subject revision is the case's current one and no claim lists it in `excluded_evidence`. Lines 190-191 then apply the rule "some `may_produce` kind not present".
- **First driver run.** It looped on `attempt_falsification`, because `inspect` stays admissible after its evidence exists (`$HOME/beyond10x/sdc/docs/driver.md:56-57`).
- **Boundary.** `$HOME/beyond10x/sdc/docs/boundary.md:69` classes this part as Canon kernel (pure). Row 70 classes the history part (`max_runs`, success per snapshot) as runtime. Moving `until` into the SDC protocols (`canon-requests.md`, "Not Canon") is an SDC edit. Only the pure part is in this story.
- **Design.** It lists "useful missing evidence" in the frontier (`docs/design/canon-protocol-calculus-design.md:282-291`) and asks "admissible and useful" (`:604`). Verified.
- **`may_produce`.** It is a list of kinds only: `canon.protocol.EvidenceProduction` has the one field `evidence` (`ess/domains/protocol.yaml:252-257`). In the IR it is sorted, each kind once (`crates/canon/src/ir/mod.rs:167-168`). It names no subject, result or producer, so the field cannot be more specific than per kind.
- **Applicability depends on the stage.** Binding and freshness set a record aside for every claim. Invalidation keeps a record "from claims only" (`eval/mod.rs:200-206`). Independence works per match (`eval/independence.rs:1-21`). An action precondition's `present` reads `left` and ignores invalidation (`eval/actions.rs:203-206`).
- **Placement.** The `actions` section is untyped `Optional<Json>` (`protocol.yaml:1017-1020`).
- **Byte effects** were verified in the code, not by a run:
  - The conformance runner compares only the sections an expectation lists, and "a later story that adds a section leaves earlier expectations passing" (`crates/canon/src/conform/mod.rs:93-101`, `:711-724`).
  - `canon generate` lists only `actions` or `outcomes` (`crates/canon/src/generate/mod.rs:223,244,354`).
  - Two tests compare whole `canon evaluate` stdout. `crates/canon-cli/tests/adversary_skel_cli.rs:18,24` pins `NO_EVIDENCE_DECISION` and `SKELETON_NO_EVIDENCE_DECISION`, where `inspect` and `attempt_falsification` are admissible with no evidence. `crates/canon-cli/tests/explanation.rs:284-306` compares stdout with step 0 of `conformance/scenarios/explanation.yaml`, where both records are `revision_mismatch`.
  - In all three decisions the new section is non-empty.
- **Answer: the field cannot be added without changing existing bytes.** As a new section (recommended), only those three decisions change: the two constants and the `explanation.yaml` steps that `explanation.rs` reads. The generated `website/docs/reference/investigation-example.md` and `website/data/canon.terminal.json` also change. As a key inside each `actions` entry, every expectation listing `actions` with such an admissible action changes as well: 33 `"status": "admissible"` lines across 10 scenario files, plus the library tests that compare the `actions` Json (`crates/canon/tests/adversary2_admissibility_reasons.rs:55-64` and others). Only an opt-in flag changes no bytes.
- **Not verified:** whether `ess_model_matches.rs` maps `Option<Declarations<ActionId, Vec<EvidenceKindId>>>` to `Optional<Map<…, List<…>>>`. No `Optional<Map` exists under `ess/` today. Also not verified: whether any scenario expectation is a non-object (whole-decision) comparison. None was found.

## Decision needed

D1: what "an applicable record" means for a kind that has no subject.

| Option | Change in `ess/` and the model | Cost |
|---|---|---|
| 1. In `left` (binding and freshness), the `present` rule of action reasons | field and comment only | After an upstream change, an invalidated record still counts, so the action looks useless and a runner stops (`stuck`) where the driver goes on |
| 2. In `left` and listed in no claim's `excluded_evidence` | field and comment only | Same as the driver. A `not_independent` producer re-run stays "missing", so a runner needs its own run budget |
| 3. Per claim: some claim reaching K has no record its own match accepts (subject, independence, invalidation) | field, plus a per-match rule in the comment | Most precise. New logic beside `excluded_for`, larger tests |

D2: placement. Options: (A) a key per `actions` entry; (B) a new top-level section; (C) B behind a new `canon evaluate` flag declared in `ess/domains/cli.yaml`. Costs are under Why.

Recommendation: D1 option 2 and D2 option B. Option 2 removes the driver's second applicability check without changing its behaviour. B re-pins 2 test constants and 1 scenario file instead of 10 scenario files and the library tests.

## ESS first

- **Commit 1 (specification only):**
  - `ess/domains/protocol.yaml`: the field `missing_evidence`, `Optional<Map<canon.protocol.ActionId, List<canon.protocol.EvidenceKindId>>>`, on `canon.protocol.Decision`, with the D1 rule in its comment and a `# read from:` pointing at this story.
  - The new `fixtures/investigation/missing-evidence.yaml`.
  - The new `conformance/scenarios/missing-evidence.yaml` (id `CANON-FRONTIER-001`).
  - `CANON-FRONTIER-001` in `conformance/requirements.yaml`.
- **Red on that commit:**
  - `crates/canon/tests/ess_model_matches.rs::specification_and_model_are_equal` names `Decision.missing_evidence`.
  - `canon conform run --requirements conformance/requirements.yaml` fails `CANON-FRONTIER-001`, because each step lists a section the decision does not carry (`conform/mod.rs:100-101`).

## Scope

- **In:** the section, D1 option 2, the model field, ESS, the fixture, the scenario and the requirement. Re-pinning the three decisions under Why. — cited
- **Out:**
  - `case_outcome` for an action with `case_open`, which `check/mod.rs:700-707` counts as produced (item 5 and Loom). — inferred
  - Usefulness of actions with no `may_produce`, `until`, and `max_runs` (runtime). — cited
  - Explaining the section in `explanation`. — inferred
  - `canon check` and `canon generate`. — inferred
- **Surfaces:**
  - `ess/domains/protocol.yaml` — cited
  - `crates/canon/src/model/decision.rs` — cited
  - `crates/canon/src/eval/decision.rs` — cited
  - `crates/canon/src/eval/mod.rs` — cited
  - `crates/canon/src/eval/missing.rs` (new) — inferred
  - `crates/canon/tests/ess_model_matches.rs` — cited
  - `crates/canon-cli/tests/adversary_skel_cli.rs` — cited
  - `crates/canon-cli/tests/explanation.rs` — cited
  - `conformance/scenarios/explanation.yaml` — cited
  - `conformance/scenarios/missing-evidence.yaml` — inferred
  - `conformance/requirements.yaml` — cited
  - `fixtures/investigation/missing-evidence.yaml` — inferred
  - `crates/canon/tests/adversary_obligations_docs.rs` (builds `Decision {…}`) — cited
  - `crates/canon/tests/adversary_explain_trace.rs` (builds `Decision {…}`) — cited
  - `crates/canon-docs/src/source.rs` — inferred
- **Confidence:** high for the model, ESS and byte effects (read); medium for the parity test's type mapping and for `canon-docs`.

## Order

- Depends on story:action-admissibility, story:evidence-revision-binding, story:evidence-freshness, story:invalidation-rules, story:evidence-independence and story:explanation. All are implemented.
- Shares `crates/canon/src/eval/mod.rs` with story:invalidation-evaluation-cost (active), which requires byte-identical output: land after it.
- Shares `ess/domains/protocol.yaml` and `conformance/requirements.yaml` with story:revision-ordering and story:protocol-imports. Run in a different wave from both.

## Acceptance

Conformance scenario `CANON-FRONTIER-001` passes under `canon conform run` over `fixtures/investigation/missing-evidence.yaml`. Every step lists `missing_evidence`:

1. With no evidence, `inspect` maps to `[supporting_observation]` and `attempt_falsification` to `[falsification_attempt]`. The blocked `publish_finding` and an admissible action with no `may_produce` are not keys.
2. With both records at the current revision, both lists are `[]`.
3. With a falsification record whose result does not satisfy the claim, at the current revision, `attempt_falsification` maps to `[]`.
4. With both records bound to r1 and the case at r2, both kinds are listed.
5. With a record older than its kind's `max_age` at `--at`, its kind is listed.
6. With a record invalidated for a claim by an upstream revision change, its kind is listed.
7. With a record whose producer fails an `independent_on` demand, its kind is listed.

## Source

Request item 2 of `$HOME/beyond10x/sdc/docs/canon-requests.md`. Files read:
- `$HOME/beyond10x/sdc/driver/src/main.rs:130-196`
- `$HOME/beyond10x/sdc/docs/driver.md:40-70,185-212`
- `$HOME/beyond10x/sdc/docs/boundary.md:15-30,45-55,69-76,150-160`
- In the canon tree at `15cfece`:
  - `AGENTS.md`
  - the design `:270-300,595-615`
  - `eval/mod.rs`, `eval/actions.rs`, `eval/decision.rs`, `eval/binding.rs`, `eval/independence.rs`
  - `model/decision.rs`, `ir/mod.rs`, `check/mod.rs`, `conform/mod.rs`, `generate/mod.rs`
  - `ess/domains/protocol.yaml`
  - the scenarios and CLI tests named above
  - the stories capability-scope, invalidation-evaluation-cost, revision-ordering, protocol-imports, protocol-floor and canon-mutate

Date: 2026-11-20.

## Decided

D1 option 2 and D2 option B of § Decision needed: a record is applicable when the revision-binding and freshness stages leave it and no claim lists it in `excluded_evidence`; the field is a new top-level section `missing_evidence`. Decided on 2026-11-20 by the wave coordinator on the drafter's recommendation, after the operator accepted the story without choosing (`approval-record:accept-sdc-requests-2026-11-20`). The unit builds this option and does not reopen the choice.
