---
format: aep.planning-md/3
id: story:decision-grounds
kind: story
status: implemented
title: Ground authority and explicit decisions on artifact revisions and evidence records, and set a decision aside when a ground moves
tags:
- from-request
refs:
- provider: sdc
  reference: docs/decisions.md
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:authority-revision-binding
- depends_on: story:missing-evidence
scope:
- confidence: cited
  path: conformance/requirements.yaml
- confidence: inferred
  path: conformance/scenarios/decision-grounds.yaml
- confidence: inferred
  path: crates/canon-cli/tests/adversary_conformance_suite_cli.rs
- confidence: inferred
  path: crates/canon-cli/tests/conform_requirements.rs
- confidence: inferred
  path: crates/canon/src/eval/actions.rs
- confidence: cited
  path: crates/canon/src/eval/authority.rs
- confidence: cited
  path: crates/canon/src/eval/decisions.rs
- confidence: cited
  path: crates/canon/src/eval/grounds.rs
- confidence: cited
  path: crates/canon/src/eval/mod.rs
- confidence: cited
  path: crates/canon/src/eval/outcomes.rs
- confidence: cited
  path: crates/canon/src/eval/pending.rs
- confidence: cited
  path: crates/canon/src/explain/mod.rs
- confidence: cited
  path: crates/canon/src/model/authority.rs
- confidence: cited
  path: crates/canon/src/model/explicit.rs
- confidence: cited
  path: crates/canon/tests/adversary_decision_grounds.rs
- confidence: cited
  path: crates/canon/tests/decision_grounds.rs
- confidence: cited
  path: ess/domains/protocol.yaml
- confidence: inferred
  path: fixtures/investigation/decision-grounds.yaml
revision: 27
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T00:21:35Z", actor: "human:timo", revision: 18, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
- {from: "proposed", to: "active", at: "2026-11-21T00:21:35Z", actor: "human:timo", revision: 19, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
- {from: "active", to: "implemented", at: "2026-11-21T11:37:25Z", actor: "human:timo", revision: 27, decided_on: {"recorded":{"test_result":1,"review_outcome":5,"verification":1}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w6"}
---
## Outcome

An entry of `canon-authority/1` or `canon-decisions/1` can name the grounds it rested on. `canon evaluate` sets the entry aside when one of its grounds no longer holds, and says which ground and why.

- `canon.protocol.AuthorityDecision` and `canon.protocol.ExplicitDecision` each gain two fields. Like `scope`, each is a collection, and an absent field reads as empty:
  - `ground_revisions: Map<ArtifactId, Revision>`, the artifact revisions the decision rested on. It has the same form as `EvidenceRecord.upstream_revisions`.
  - `ground_evidence: List<EvidenceId>`, the evidence records it rested on.
- **Artifact ground.** It holds when its revision equals the case snapshot's current revision of that artifact. Revisions are compared for equality, which is the rule `binding.rs` applies to `subject_revision`.
- **Evidence ground.** It holds when a record with that id is in the evidence set, the revision-binding and freshness stages leave the record, and no claim lists it in `excluded_evidence`. That is the "applicable record" of story:missing-evidence (its Decided D1).
  - Invalidation rules and expiry therefore reach a decision through its evidence grounds.
  - An id that no given record has means the ground does not hold. That id is not refused.
- **Applicability.** An entry applies only when every ground holds, and, where it has a `case_revision`, only when that revision is current (story:authority-revision-binding). An entry that does not apply takes no part in coverage, precedence or `decided_by`. This holds whatever the entry decided, so a set-aside `denied` lapses as well.
- **"Stale" uses the statuses and reasons that exist today. No status and no reason value is added.**
  - A requirement covered only by set-aside entries is not decided. The action is `approval-required` with reason `{"capability", "decision": "none"}`.
  - An outcome whose only decision was set aside is `blocked`, `{"decision": <name>, "present": false}`.
  - Either reason gains one key, `stale_grounds`, listing each failed ground of the set-aside entries:
    - `{"artifact", "revision", "reason": "revision_mismatch"}` for a moved artifact;
    - `{"evidence", "reason": <ExclusionReason>}` for an excluded record;
    - `{"evidence", "present": false}` for a record that was not given.
  - These reuse `ExclusionReason` (`documents.md:98-108`) and the `present: false` form (`eval/outcomes.rs:149`).
- **Explanation.** `computed_from.authority` and `computed_from.decisions` record the grounds of each entry as given.
- **Refusals.**
  - A ground artifact the protocol does not declare is refused as `undeclared-artifact`.
  - A ground artifact or revision that is not an identifier is refused as `invalid-identifier`, and so is an evidence id that is not one.
  - Grounds do not enter the duplicate key. Two entries equal in every other key are refused as they are today.
- An input without grounds evaluates to the same bytes as today. No command or flag changes.

## Why

- **The request.** It is request 13 from the SDC session, 2026-11-21, item 2 of `$HOME/beyond10x/sdc/docs/decisions.md:226-229`: grounds as a citation to `artifact@revision` or an evidence id, applied only while current and invalidated by the evidence rules, "so a recommendation whose grounds moved reads as stale from the evaluation". The canon tree records it at `docs/plan/wave-plan-2026-11-20.md:163-165`. That page says the coordinator did not hear the operator's decision behind it.
- **The SDC design.** It is unverified research and I only read it:
  - A ground is a source kind, a citation, the option it supports and a sentence (`$HOME/beyond10x/sdc/ess/domains/decision.yaml:156-164`).
  - A citation is one of three forms (`:146-154`): `path_line` with a digest, `record_id` in one of five stores (`:119-123`), or `artifact_revision` (`:139-144`).
  - The reference check (`$HOME/beyond10x/sdc/tools/decision-grounds.sh:41-46`) accepts `artifact@revision` only when it is current in `case.json`.
  - For a `canon_evidence` id it checks only that a file holds the id (`:34`), not that the record applies.
  - Rule 4 reuses a decided request while its grounds resolve unchanged (`decisions.md:109-112`).
  - ADR 0079 as cited there says "a decision record keeps the digests of the evidence it relied on" (`decisions.md:25`). I did not open ADR 0079.
- **Boundary.** Ownership is at `decisions.md:192-215`.
  - Loom's part: the recommendation, its producer, routing, holders, expiry, `path_line` digests (Canon reads no files), the `aep_artifact` and `sdc_decisions` stores, and grounds in another case.
  - Canon's part: the decided record and whether it applies, which is evaluation.
  - A pending recommendation is not a Canon input. Canon reports a stale recommendation once the entry it produced is evaluated. Before that, Loom's check reads the same two rules.
- **What exists, verified at `6f44325`:**
  - **Binding.** A record whose `subject_revision` is not current is excluded for every claim as `revision_mismatch` (`crates/canon/src/eval/binding.rs:38-47`).
  - **Freshness.** It excludes a record as `expired` only when the record has `observed_at`, its kind has `max_age` and `--at` is given (`crates/canon/src/eval/freshness.rs:6-10`).
  - **Invalidation.** A moved upstream revision sets a record aside only through a protocol-declared rule watching that artifact. It does so only for the claims the rule names and the claims built on them (`crates/canon/src/eval/invalidation.rs:1-14`, `:84-114`).
  - **A record cannot cite another record.** `EvidenceRecord` has no field holding an `EvidenceId` (`crates/canon/src/model/evidence.rs:17-53`).
  - **Unread records go unreported.** `excluded_evidence` lists a record only under claims that reach its kind (`crates/canon/src/eval/mod.rs:552-582`). A record that no claim reads is never reported as excluded.
  - **Undeclared kinds are refused.** A record of a kind the protocol does not declare is refused (`crates/canon/src/eval/evidence.rs:99-108`).
  - **`canon-decisions/1` entries.** An entry is `decision, outcome, principal, case_revision`. It applies only at its case revision (`crates/canon/src/model/explicit.rs:24-37`, `crates/canon/src/eval/decisions.rs:57-79`).
  - **`canon-authority/1` entries.** An entry is `capability, scope, decision` (`crates/canon/src/model/authority.rs:37-48`) and reads no revision (`crates/canon/src/eval/authority.rs:53-59`).
  - **Neither document has grounds.**
- **How grounds differ from story:authority-revision-binding.**
  - That story binds an entry to the case revision it was taken at: one value, the whole case, which moves with any artifact in the SDC driver.
  - Grounds bind the entry to what it rested on: named artifacts, which may differ from the subject, and evidence records, which can expire or be invalidated.
  - An unbound entry, such as a deny meant to last, still lapses when a ground moves.
  - The two compose. Grounds follow that story's Decided option A: a set-aside entry of any decision takes no part.
- **Not verified:** nothing was built or run. `ess_model_matches.rs` was not checked for a defaulted `List<EvidenceId>` field. The function story:missing-evidence will add for "applicable" does not exist yet.

## Decision needed

| | Where grounds live | `ess/` and model | Cost |
|---|---|---|---|
| A | `ground_revisions`, `ground_evidence` on `AuthorityDecision` and `ExplicitDecision`; evaluation sets the entry aside | Four fields on two existing structs; reason key `stale_grounds` | Grounds count only once a decision is recorded. An approval written as `canon-evidence/1` (`decisions.md:182`) gets no evidence-id ground; its artifact grounds are `upstream_revisions` |
| B | A recommendation is a `canon-evidence/1` record: `subject_revision` binds it to the subject, `upstream_revisions` to other artifacts | Gap: a `ground_evidence` field on `EvidenceRecord`, a dependency between records, an order among exclusion stages and a cycle refusal | Every protocol must declare a recommendation kind, a claim reading it and an invalidation rule per cited artifact, or staleness is never reported (`eval/mod.rs:552-582`). A proposal becomes an observation that gates can read |
| C | A new document, for example `canon-grounds/1` with `canon evaluate --grounds`: each ground reported held or stale | New struct, model file, entry in `DOCUMENTS` of `ess_model_matches.rs`, a flag in `ess/domains/cli.yaml`, a schema, a decision section | Covers pending recommendations, but Canon then evaluates a Loom entity that no protocol rule reads. Adds a third decision-side input |

Sub-choice for an evidence ground under A: "holds" means applicable in story:missing-evidence's sense (recommended, one definition in Canon). The alternative is "left by binding and freshness" only, which ignores invalidation.

Recommendation: A, because the decided record is what evaluation already applies, and grounds become one more applicability condition beside `case_revision`, with no new status, document or flag.

## ESS first

- **First commit (specification only).**
  - In `ess/domains/protocol.yaml`, `canon.protocol.ExplicitDecision` (`:888`) and `canon.protocol.AuthorityDecision` (`:928`) each gain `ground_revisions: Map<canon.protocol.ArtifactId, canon.protocol.Revision>` and `ground_evidence: List<canon.protocol.EvidenceId>`.
  - Comments on both structs state the rule; `# read from:` points to this story.
  - `CANON-GROUND-001` is added to `conformance/requirements.yaml`. That id is free: no requirement or story uses `CANON-GROUND`.
  - New `fixtures/investigation/decision-grounds.yaml` and `conformance/scenarios/decision-grounds.yaml`.
- **Red on that commit.**
  - `crates/canon/tests/ess_model_matches.rs::specification_and_model_are_equal` names the four fields.
  - `canon conform run --requirements conformance/requirements.yaml` fails on `CANON-GROUND-001`, because `deny_unknown_fields` (`model/authority.rs:36`, `model/explicit.rs:23`) refuses `ground_revisions` as `malformed-input`.

## Scope

- **In:**
  - The four fields, the readers, identifier checks and `undeclared-artifact` — cited
  - The applicability filter for grounds in both documents — cited
  - `stale_grounds` on the two existing reasons — inferred
  - Grounds in `computed_from` — cited
- **Out:**
  - The recommendation, its producer, routing, holders, expiry and the inbox (Loom) — cited
  - `path_line` digests and the `record_id` stores other than evidence — cited
  - Grounds in another case — inferred
  - Descent between revisions (story:revision-ordering) — inferred
  - Evidence-id grounds on `canon-evidence/1` (option B) — inferred
  - The derived view of pending decisions (request 12) — cited
  - `canon check`/`canon generate`, which write entries without grounds (`check/space.rs:1388-1407`, per story:authority-revision-binding) — inferred
- **Surfaces:**
  - `ess/domains/protocol.yaml` — cited
  - `crates/canon/src/model/authority.rs` — cited
  - `crates/canon/src/model/explicit.rs` — cited
  - `crates/canon/src/eval/authority.rs` — cited
  - `crates/canon/src/eval/decisions.rs` — cited
  - `crates/canon/src/eval/mod.rs` — cited
  - `crates/canon/src/eval/outcomes.rs` — cited
  - `crates/canon/src/eval/actions.rs` — inferred
  - `crates/canon/src/eval/missing.rs` — inferred
  - `crates/canon/src/explain/mod.rs` — cited
  - `conformance/requirements.yaml` — cited
  - `conformance/scenarios/decision-grounds.yaml` — inferred
  - `fixtures/investigation/decision-grounds.yaml` — inferred
  - `crates/canon-cli/tests/conform_requirements.rs` — inferred
  - `crates/canon-cli/tests/adversary_conformance_suite_cli.rs` — inferred
- **Coordinator's at integration:** `website/docs/reference/documents.md`, `authority-1.schema.json`, `decisions-1.schema.json`, `website/status.yaml`.
- **Confidence:** high for the model, ESS and readers. Medium for where the grounds filter runs (`eval/mod.rs` before `actions::section`, or inside it).

## Order

- **Lands after:**
  - story:authority-revision-binding (wave 3): the same reader, applicability filter, duplicate key and `computed_from.authority`.
  - story:missing-evidence (wave 4): the applicable-record definition and `eval/mod.rs`.
- **Shares files:**
  - `ess/domains/protocol.yaml` and `conformance/requirements.yaml` with every story named here. Both are split by declaration.
  - `eval/mod.rs` with story:child-outcome-revision (inferred) and story:evidence-redelivery (wave 6).
  - `eval/actions.rs` with story:child-outcome-revision and story:revision-ordering (inferred).
  - `explain/mod.rs` with story:child-outcome-revision (inferred).
  - story:supplied-evidence shares only the two split files.
- **Earliest wave:** the first wave with no other shared file comes after wave 6. In wave 7 it meets story:revision-ordering on `eval/actions.rs` (both inferred).
- **Neighbour:** the story drafted from request 12 (derived view of pending decisions). Neither depends on the other. That view reads `stale_grounds` as written once this lands.

## Acceptance

Conformance scenario `CANON-GROUND-001` passes under `canon conform run` over `fixtures/investigation/decision-grounds.yaml`. The fixture has action `act` requiring capability `x`, outcome `o` requiring decision `d`, artifact `spec` at `r2`, record `e1` of kind `k` (with `max_age`) read by claim `c`, and an invalidation rule watching `up` that names `c`. The case is at `revision: c2`.

1. An unbound `granted` entry for `x` with `ground_revisions: {spec: r2}` gives `act` `admissible`.
2. The same entry with `{spec: r1}` gives `approval-required`, reason `{"capability": "x", "decision": "none", "stale_grounds": [{"artifact": "spec", "reason": "revision_mismatch", "revision": "r1"}]}`.
3. With `ground_evidence: [e1]`: `admissible` when `e1` is given and applicable; `stale_grounds: [{"evidence": "e1", "present": false}]` when it is not given; `[{"evidence": "e1", "reason": "expired"}]` with `--at` past its `max_age`.
4. With `e1` recording `upstream_revisions: {up: u1}` and the case at `up: u2`, `stale_grounds` is `[{"evidence": "e1", "reason": "invalidated"}]`.
5. A `denied` entry with a moved ground beside an unbound `granted` entry gives `admissible`.
6. A `canon-decisions/1` entry for `o` at `c2` with `{spec: r1}` gives `o` `blocked`, with `{"decision": "d", "present": false, "stale_grounds": [...]}`. With `{spec: r2}`, `o` is `legitimate` with `decided_by`.
7. Every existing scenario passes unchanged. A ground artifact `nowhere` is refused as `undeclared-artifact`, and an evidence id `'e 1'` as `invalid-identifier`.

## Source

- Request 13, passed on by the SDC session on 2026-11-21, item 2 of `$HOME/beyond10x/sdc/docs/decisions.md:217-231`.
- SDC files read: `$HOME/beyond10x/sdc/docs/decisions.md`, `$HOME/beyond10x/sdc/ess/domains/decision.yaml`, `$HOME/beyond10x/sdc/tools/decision-grounds.sh`.
- Canon tree `wave/2026-11-20-w2` at `6f44325`:
  - `AGENTS.md`, `website/docs/reference/documents.md`
  - `crates/canon/src/eval/{binding,freshness,invalidation,evidence,decisions,authority,mod}.rs`
  - `crates/canon/src/model/{decision,authority,explicit,evidence}.rs`
  - `crates/canon/src/explain/mod.rs`, `ess/domains/protocol.yaml`, `conformance/requirements.yaml`, `docs/plan/wave-plan-2026-11-20.md`
- Stories read: story:authority-revision-binding, story:child-outcome-revision, story:missing-evidence, story:evidence-redelivery, story:supplied-evidence, story:revision-ordering.
- Date: 2026-11-21.

## Decided

Option A of § Decision needed: `ground_revisions` and `ground_evidence` on the entries of `canon-authority/1` and `canon-decisions/1`. An evidence ground holds when the record is applicable in the sense of story:missing-evidence. Decided on 2026-11-21 by the wave coordinator on the drafter's recommendation; the story was accepted under the operator's standing grant without a choice among the options (`approval-record:accept-sdc-requests-12-13-2026-11-21`). The unit builds this option and does not reopen the choice.

## Decided in review, after adversary pass 1

The adversary pass (`review-result:adversary-decision-grounds-pass-1`) was green: the six claims
held, four mutants the unit's suite let through are now held by the adversary's cases, and it
asked for one rule to be confirmed or reversed. Decided on 2026-11-21 by the wave coordinator;
the session the operator named orchestrator of this plan may overrule it.

- **D2, an entry set aside by its case revision adds none of its grounds to `stale_grounds`.**
  The unit built this and pinned it in CANON-GROUND-001 (step
  `an-entry-at-another-case-revision-adds-no-ground`); § Outcome said only "each failed ground of
  the set-aside entries". Confirmed. `stale_grounds` answers "which ground must hold again for a
  decision already given to count". An entry bound to another case revision does not count
  whatever its grounds do (story:authority-revision-binding), so naming its grounds would tell a
  runner to restore something that restores nothing. § Outcome reads from now on: `stale_grounds`
  lists each failed ground of the entries that every other rule would let apply.
- In `computed_from`, an entry's `ground_evidence` is written sorted and without repeats and an
  empty ground field is left out. § Outcome said "as given"; the bytes as built stand, because
  the same input in another order must give the same bytes.
- The note on `ess/domains/protocol.yaml` is corrected in the unit: the `read from` range of the
  authority module and the comments of `AuthorityDecision` and `ExplicitDecision`, which state
  applicability and the refusal order without grounds.

## Scope as built

Written by the wave coordinator at the close of wave 2026-11-20-w6 from the implementor's report, the adversary's report (`review-result:adversary-decision-grounds-pass-1`, green) and the merge of `impl/decision-grounds` (`3c0d0a0`). It corrects the body where the two differ.

- Applicability is decided once, in `evaluate_with`, through `Authority::applying_at` and `Decisions::applying_at`, before the action and outcome sections are computed. Whether a ground holds is computed in the new `crates/canon/src/eval/grounds.rs` (`Standing`); nothing else calls it.
- The claim section is computed before the action and outcome sections. It reads no decision; 152 of 152 evaluate steps of the scenarios without grounds are byte-identical to the base.
- `stale_grounds` lists each failed ground once across the entries set aside by their grounds alone: artifact grounds by artifact then revision, then evidence grounds by id. A failed evidence ground carries the reason the binding or freshness stage gave the record, otherwise the reason of the first claim that lists it, in claim-id order. An entry set aside by its case revision adds none (Decided in review, D2).
- The key appears only on a `"decision": "none"` capability reason and on the `{"decision", "present": false}` outcome reason. A `require_approval` reason and a denial never carry it.
- An evidence ground holds on a record of a kind no claim reads, when the record is given and the binding and freshness stages leave it.
- In `computed_from`, an entry's `ground_evidence` is written sorted and without repeats and an empty ground field is left out.
- `crates/canon/src/eval/pending.rs`, which § Scope did not name, changed: its filter takes a reason with no key other than `decision`, `present` and `stale_grounds`. A pending entry does not copy the grounds.
- `canon check` and `canon generate` write entries without grounds; `canon simulate` reads no authority document.
- In § Scope and not edited: `crates/canon/src/eval/missing.rs` (a reader only). Added by the unit: `crates/canon/src/eval/grounds.rs`, `crates/canon/tests/decision_grounds.rs`. Added by the adversary: `crates/canon/tests/adversary_decision_grounds.rs` (6 cases; four hold rules the unit's suite let a mutant through on).
- Scenario `CANON-GROUND-001`. With `CANON-EVIDENCE-004` the catalogue has 28 requirements at this merge.
