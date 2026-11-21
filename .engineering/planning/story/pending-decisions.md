---
format: aep.planning-md/3
id: story:pending-decisions
kind: story
status: active
title: 'Derive the pending decisions of an evaluation: per approval-required requirement and per decision-blocked outcome, the entry that would change it'
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
- depends_on: story:capability-scope
- depends_on: story:decision-outcomes
scope:
- confidence: cited
  path: conformance/requirements.yaml
- confidence: inferred
  path: conformance/scenarios/pending-decisions.yaml
- confidence: inferred
  path: crates/canon-cli/tests/conform_requirements.rs
- confidence: inferred
  path: crates/canon-docs/src/source.rs
- confidence: cited
  path: crates/canon/src/eval/decision.rs
- confidence: cited
  path: crates/canon/src/eval/mod.rs
- confidence: inferred
  path: crates/canon/src/eval/pending.rs
- confidence: cited
  path: crates/canon/src/model/decision.rs
- confidence: cited
  path: ess/domains/protocol.yaml
- confidence: inferred
  path: fixtures/investigation/pending-decisions.yaml
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T00:21:19Z", actor: "human:timo", revision: 13, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
- {from: "proposed", to: "active", at: "2026-11-21T00:21:19Z", actor: "human:timo", revision: 14, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
---
## Outcome

`canon evaluate` writes a new top-level section, `pending_decisions`, in `canon-decision/1`. It lists the decisions the evaluation waits on. Each one is a `canon-authority/1` or `canon-decisions/1` entry that would change a status in the same decision. The section is computed from the inputs `canon evaluate` already takes, and nothing else. It is written only when it has at least one entry. The other sections do not change.

- **`authority`**: one entry per requirement that makes an action `approval-required`, meaning every reason of that action with `"decision": "none"` or `"decision": "require_approval"` (`crates/canon/src/eval/actions.rs:43-46`). The entry is `{"action", "capability", "scope", "current", "case_revision", "options"}`:
  - `scope` is written only when the requirement has one.
  - `current` is the reason's `none` or `require_approval`.
  - `case_revision` is the snapshot's `revision`, written only when the snapshot has one.
  - `options` is always `["granted", "denied"]`.
  - Entries are ordered by action id, then capability, then scope (no scope first).
  - An action that is `blocked` gets no entry, whether its precondition failed or a requirement is denied. Neither is changed by asking.
- **`decisions`**: one entry per outcome that is `blocked` with the reason `{"decision": <name>, "present": false}` (`crates/canon/src/eval/outcomes.rs:36-38`, `:149`). The entry is `{"outcome", "decision", "case_revision", "options"}`:
  - `options` is `["taken"]` when the snapshot has a `revision`.
  - With no `revision`, `case_revision` is left out and `options` is `[]`, because no `canon-decisions/1` entry can apply then (`eval/decisions.rs:63`).
  - Entries are ordered by outcome id.
- **What an option means.** An option is the value of the entry a caller would add, together with its own `principal`. For an authority entry, add `capability`, `scope` and, when present, `case_revision`. For a decision entry, add `decision`, `outcome` and `case_revision`. Precedence does not change. For a requirement whose `current` is `require_approval`, a `granted` entry decides it only when no `require_approval` entry still covers it (`actions.rs:56-57`). The reference page says this.
- **Not computed.** Who decides, a principal, a holder, an expiry, routing, an inbox, a recommendation, a question text, a cost or grounds. These belong to Loom.
- **Model and ESS.** `canon.protocol.Decision` in `ess/domains/protocol.yaml` and `crates/canon/src/model/decision.rs` gain `pending_decisions: Optional<Json>`. Its shape is fixed by `CANON-PENDING-001`, as the other section slots are (`model/decision.rs:12-13`). No command or flag changes.

## Why

**The request.** Request 12, item 1, from the session working on the SDC prototype, 2026-11-21. It is drafted at `$HOME/beyond10x/sdc/docs/decisions.md:221-225`.
- `decisions.md:197-203` (§ Ownership) gives Canon "the pending decision as a fact derived from a case".
- `:204-209` give Loom the recommendation, routing, the autonomy policy, expiry and escalation. This story takes only Canon's part.
- The driver turns each `next` entry into one `Request` (`decisions.md:153-159`). Its contract needs, from Canon:
  - `case_id`;
  - `unblocks.kind`, `action`, `capability` and `outcome` (`$HOME/beyond10x/sdc/ess/domains/decision.yaml:69-78`);
  - `subject_artifact` and `subject_revision` (`:228-229`);
  - the grant an option writes (`Consequence.grant`, `:86-95`).
- That directory is another session's research. It was read only, and none of it was run.

**How the driver builds `next` today** (`$HOME/beyond10x/sdc/driver/src/main.rs:509-553`):
- It takes the `approval-required` action ids from the `actions` section (`:397`).
- It takes the capabilities from the IR's `requires` (`:520-521`) and ignores scopes.
- It takes the subject from its own bindings file (`bindings[a]["subject"]`, `:529`), not from Canon.
- It takes the artifact revision from `case.json` (`:531-532`).
- It builds no entry for an outcome that waits on a decision. `position` passes only `legitimate_outcomes` (`:398`).

**What `canon-decision/1` already carries, verified in the tree at `6f44325`:**

| Requested field | In the decision today | Where |
|---|---|---|
| action that is `approval-required` | yes | `actions.<id>.status` |
| capability | yes, per requirement, with `scope` and `decision: none \| require_approval` | `actions.rs:43-46` |
| subject artifact of the action | **no**: `protocol/1` declares no subject on an action | `ir/mod.rs:188-208`, `model/mod.rs:218-242`, `:263-271` |
| artifact's current revision | only as input echo, for every artifact | `explanation.computed_from.case.artifacts` (`explain/mod.rs:8-13`) |
| outcome that requires a decision | yes, as a blocked outcome with `{"decision", "present": false}` | `outcomes.rs:36-38` |
| case revision | only as input echo; no top-level key | `computed_from.case.revision`; `Decision` has no such field (`model/decision.rs:18-46`) |
| options | **no** | — |

- **"Requires an explicit decision".** An outcome requires one when it declares `requires: decision: <name>` (`OutcomeRequirement::Decision`, `model/requirement.rs:27-32`). That is the whole requirement: nothing else blocks such an outcome. A decision applies only at the snapshot's `revision` (`eval/decisions.rs:57-79`).
- **Scopes.** Since story:capability-scope, a requirement is `(capability, scope)`. A scope is a map of free identifiers (`ir/mod.rs:210-211`). No scope key or value is checked against an artifact. So a scope does not name a subject artifact.
- **The revision a grant applies at** is the case revision, under story:authority-revision-binding (§ Decided, option A). It is not an artifact revision.
- **Byte effects** (verified in code, not by a run):
  - The conformance runner compares only the sections an expectation lists (`crates/canon/src/conform/mod.rs:93-101`).
  - The two whole-output constants in `crates/canon-cli/tests/adversary_skel_cli.rs:18,24` contain no `approval-required` action and no decision-blocked outcome, so their bytes stay the same.
  - Other whole-output comparisons were not all read.
- **Not verified:**
  - Whether Loom links `b10x_canon` as a library. The SDC driver runs the `canon` binary (`main.rs:243`).
  - Whether Loom reads the section.

## Decision needed

**D1: where the view lives.**

| | Change in `ess/` and the model | Cost |
|---|---|---|
| A | New section `pending_decisions` on `canon.protocol.Decision`, `Optional<Json>` | It repeats facts the `actions` and `outcomes` reasons hold, and the case revision. Bytes change only for a decision that has something pending. |
| B | New format `canon-pending/1` and a command `canon pending`, with the same flags as `evaluate`, declared in `ess/domains/cli.yaml` | A new command, schema and reference page. A runner calls `canon` twice per step. |
| C | Library function `eval::pending(&Decision)` | No CLI surface, so the SDC driver, which runs the binary, gets nothing. It parses untyped sections and reads the case revision from `explanation`. |

**D2: the subject artifact of an action.**

| | Change | Cost |
|---|---|---|
| A | None. The entry gives `case_revision`; the caller maps the action to its artifact, as the driver does now (`main.rs:529`). | The request's `subject_artifact` stays with the caller. |
| B | An optional `subject` on a `protocol/1` action; the entry writes it and its snapshot revision | A language change in `model/mod.rs`, `ir/mod.rs`, `validate`, `diff` and the IR read-back. These are the files where story:judgement-marking adds `author` and `revises`. |
| C | Derive it from the evidence-match subjects of the precondition | Zero or several subjects per action. A guess. |

Recommendation: D1 A and D2 A. D1 A is the placement story:missing-evidence chose (§ Decided, D2 option B), with one evaluation and the same inputs. D2 A uses the revision Canon actually binds a grant to (story:authority-revision-binding, option A) and adds no language.

## ESS first

- **Commit 1 (specification only):**
  - `ess/domains/protocol.yaml`: `pending_decisions: Optional<Json>` on `canon.protocol.Decision` (`:1028`), with the shape and the rules of § Outcome in its comment and `# read from:` this story.
  - The new fixture `fixtures/investigation/pending-decisions.yaml`. It has an action requiring `metrics.read{cluster, namespace}`, an action requiring two capabilities, an action whose precondition can fail, and an outcome with `requires: decision: explicitly_inconclusive`.
  - The new scenario `conformance/scenarios/pending-decisions.yaml`.
  - `CANON-PENDING-001` in `conformance/requirements.yaml`. That id is free: it is not in that file and no story names it.
- **Red on that commit:**
  - `crates/canon/tests/ess_model_matches.rs::specification_and_model_are_equal` names `Decision.pending_decisions`.
  - `canon conform run --requirements conformance/requirements.yaml` fails `CANON-PENDING-001`, because each step lists a section the decision does not carry (`conform/mod.rs:100-101`).

## Scope

- **In:** the section, the model field, the ESS declaration, the fixture, the scenario and the requirement. — cited
- **In:** the reference text for `Decision` and the precedence note on `require_approval`. — inferred
- **Out:** principal, holder, eligibility, expiry, escalation, inbox, recommendation, question text, option cost (Loom, `decisions.md:204-209`). — cited
- **Out:** grounds, which the other half of request 12 covers. — cited
- **Out:** a `subject` on an action (D2 B). — inferred
- **Out:** handed-off questions (`UnblockKind::question`), which are SDC's `questions.yaml`. — cited
- **Out:** the snapshot's `termination`, which is not read; actions are evaluated on a terminated case today too. — inferred
- **Out:** explaining the section in `explanation`; `canon check`, `canon generate` and `canon simulate`. — inferred
- **Surfaces:**
  - `ess/domains/protocol.yaml` — cited
  - `crates/canon/src/model/decision.rs` — cited
  - `crates/canon/src/eval/decision.rs` (slot list and its tests) — cited
  - `crates/canon/src/eval/mod.rs` (`evaluate_with` builds `Decision`) — cited
  - `crates/canon/src/eval/pending.rs` (new) — inferred
  - `conformance/requirements.yaml` — cited
  - `conformance/scenarios/pending-decisions.yaml` (new) — inferred
  - `fixtures/investigation/pending-decisions.yaml` (new) — inferred
  - `crates/canon-cli/tests/conform_requirements.rs` (requirement count) — inferred
  - `crates/canon-docs/src/source.rs` — inferred
- **Coordinator's at integration:** regenerated `website/docs/reference/documents.md`, `website/data/status.json`, and an entry in `website/status.yaml`.
- **Confidence:** high for what the decision carries and for the model change, because both were read. Medium for whole-output tests outside `adversary_skel_cli.rs`.

## Order

- **Depends on:**
  - story:authority-revision-binding (wave 3). The `case_revision` written in an authority option, and `current: none` for a requirement covered only by lapsed entries, come from it.
  - story:missing-evidence (wave 4). It adds the other new top-level field to the same struct and the same slot writer.
  - story:capability-scope and story:decision-outcomes, which are implemented.
- **Shares files with:**
  - `ess/domains/protocol.yaml` and `conformance/requirements.yaml`: story:supplied-evidence (wave 3), story:child-outcome-revision (wave 5), story:evidence-redelivery and story:judgement-marking (wave 6).
  - `crates/canon/src/eval/mod.rs`: story:child-outcome-revision (inferred in its scope) and story:evidence-redelivery.
  - `crates/canon-docs/src/source.rs`: story:judgement-marking.
- **No shared file:** story:canon-simulate. Its authority steps (`granted`, `denied`) and decision steps (`taken`) are this story's option values, and nothing here contradicts them.
- **Neighbour:** the grounds story of request 12, item 2, drafted separately. If it adds a field to `canon-decision/1`, it shares every surface above.
- **Wave:** 5 at the earliest, in a wave without story:child-outcome-revision.

## Acceptance

Conformance scenario `CANON-PENDING-001` passes under `canon conform run` over `fixtures/investigation/pending-decisions.yaml`. Every step lists `pending_decisions`, and the snapshot has `revision: c2` unless a step says otherwise:

1. With no authority and no decisions:
   - `authority` holds `{action: inspect, capability: metrics.read, scope: {cluster: prod-eu, namespace: foo}, current: none, case_revision: c2, options: [granted, denied]}`.
   - `decisions` holds `{outcome: inconclusive, decision: explicitly_inconclusive, case_revision: c2, options: [taken]}`.
2. With a `require_approval` entry covering the requirement, `current` is `require_approval`.
3. With a covering `granted` entry, `inspect` has no entry. With a covering `denied` entry, it has none either.
4. The action whose precondition is not `true` has no entry, although it requires a capability.
5. Of the action that requires two capabilities with one of them granted, only the other is listed. Entries follow action, capability, scope order.
6. With a `canon-decisions/1` entry at `c2`, `decisions` is absent. With one at `c1`, the entry stays.
7. With a `granted` entry bound to `c1`, `inspect` is listed with `current: none`.
8. With a snapshot without `revision`, no entry has `case_revision`, and the outcome entry has `options: []`.
9. With every requirement granted and the decision taken, there is no `pending_decisions` key. A step listing every other section gives the same bytes as on the commit before this story.

## Source

- Request 12, item 1, passed on by the session working on the SDC prototype on 2026-11-21. Not an item of `canon-requests.md`.
- SDC files read:
  - `$HOME/beyond10x/sdc/docs/decisions.md` (whole)
  - `$HOME/beyond10x/sdc/ess/domains/decision.yaml:1-380`
  - `$HOME/beyond10x/sdc/driver/src/main.rs:360-420,505-553`
- Canon tree at `6f44325` (branch `wave/2026-11-20-w2`):
  - `AGENTS.md`
  - `crates/canon/src/model/{decision,requirement,mod}.rs`
  - `crates/canon/src/eval/{actions,outcomes,decisions,decision,mod}.rs`
  - `crates/canon/src/explain/mod.rs`, `crates/canon/src/ir/mod.rs`
  - `ess/domains/{protocol,cli}.yaml`, `conformance/requirements.yaml`
  - `website/docs/reference/documents.md`, `crates/canon-cli/tests/adversary_skel_cli.rs`
- Stories read: story:capability-scope, story:authority-revision-binding, story:missing-evidence, story:child-outcome-revision, story:evidence-redelivery, story:judgement-marking, story:supplied-evidence and story:canon-simulate. Wave numbers are from commit `3043d4d`.
- Date: 2026-11-21.

## Decided

D1 option A and D2 option A of § Decision needed: the view is the section `pending_decisions` of `canon-decision/1`; an entry carries the case revision and no subject artifact. The four points under COULD NOT ESTABLISH that the draft settled provisionally stand as drafted: `options` lists `granted` for a requirement whose `current` is `require_approval`, and `termination` is not read. Decided on 2026-11-21 by the wave coordinator on the drafter's recommendation; the story was accepted under the operator's standing grant without a choice among the options (`approval-record:accept-sdc-requests-12-13-2026-11-21`). The unit builds this option and does not reopen the choice.
