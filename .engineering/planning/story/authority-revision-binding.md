---
format: aep.planning-md/3
id: story:authority-revision-binding
kind: story
status: active
title: Bind authority decisions to a case revision and record the principal who took them
tags:
- from-request
refs:
- provider: sdc
  reference: docs/canon-requests.md
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:capability-scope
- depends_on: story:decision-outcomes
scope:
- confidence: cited
  path: conformance/requirements.yaml
- confidence: cited
  path: conformance/scenarios/authority-binding.yaml
- confidence: inferred
  path: crates/canon-cli/tests/adversary_conformance_suite_cli.rs
- confidence: inferred
  path: crates/canon-cli/tests/conform_requirements.rs
- confidence: cited
  path: crates/canon/src/eval/actions.rs
- confidence: cited
  path: crates/canon/src/eval/authority.rs
- confidence: cited
  path: crates/canon/src/eval/mod.rs
- confidence: cited
  path: crates/canon/src/explain/mod.rs
- confidence: cited
  path: crates/canon/src/model/authority.rs
- confidence: cited
  path: ess/domains/protocol.yaml
revision: 16
transitions:
- {from: "draft", to: "proposed", at: "2026-11-20T22:35:01Z", actor: "human:timo", revision: 14, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
- {from: "proposed", to: "active", at: "2026-11-20T22:35:01Z", actor: "human:timo", revision: 15, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
---
## Outcome

A `canon-authority/1` entry can be bound to a case revision and can name the principal who took it. `canon evaluate` then applies a bound grant only while the case is at that revision.

- `canon.protocol.AuthorityDecision` gains two optional fields:
  - `principal` (`Principal`): who took the decision.
  - `case_revision` (`Revision`): the case revision it was taken at.
- **Applicability.** An entry without `case_revision` is unbound and applies at every case revision, which is today's behaviour. So every existing authority document reads unchanged and evaluates to the same bytes.
  - An entry with `case_revision` applies only when that revision equals the `canon-case/1` snapshot's `revision`, compared for equality.
  - It applies to nothing when the snapshot has no `revision`.
  - This is the rule `canon-decisions/1` already uses (`Decisions::decided_by`, `crates/canon/src/eval/decisions.rs:57-79`).
- **Order of the rules.** Applicability filters first. Coverage (`covers`) and precedence (`denied` > `require_approval` > `granted`) then run over the entries that apply, unchanged. An entry that does not apply takes no part in either.
  - A requirement covered only by entries that do not apply is not decided. Its reason is `{"capability", "decision": "none"}`, with its `scope` when it has one.
- **Principal.** Canon records the principal as given. It is not matched against anything, because Canon resolves no identity (design § 23, `AGENTS.md` § Boundary). It does not change admissibility.
- **Duplicates.** Two entries equal in `capability`, `scope`, `case_revision` and `principal` are refused as `duplicate-identifier`, whatever the decision. The same capability at the same scope at two case revisions is normal input.
  - The refusal text appends `` at case revision `<r>` `` and `` by `<p>` `` when those fields are given.
  - Today's refusal text for unbound entries is unchanged.
- **Identifiers.** A principal or case revision that is not an identifier is refused as `invalid-identifier`. These checks run after the scope checks, principal first and then case revision, as in `canon-decisions/1`.
- **Explanation.** `computed_from.authority` records every entry as given, with `"principal"` and `"case_revision"` added when present. Entries are ordered by capability, then scope, then case revision (none first), then principal (none first).

## Why

**The request.** Item 1 of `$HOME/beyond10x/sdc/docs/canon-requests.md:12-16`:
- `canon-authority/1` has only `capability` and `decision` (`documents.md:116-121` at `8f1f452`). I verified this: the table lists those two keys.
- `canon-decisions/1` has `principal` and `case_revision` (`documents.md:140-148`). Verified.
- Evidence of the gap: "A `case.open` granted at spec r1 still holds at r2" (`$HOME/beyond10x/sdc/docs/driver.md:110-112`).
- `$HOME/beyond10x/sdc/docs/phases.md:45-46` requires human approvals to be "Canon authority decisions bound to the revision they approved".
- `$HOME/beyond10x/sdc/docs/boundary.md:147-153` asks that a grant apply only at its revision, "as `canon-decisions/1` already does".

**What "r1 still holds at r2" means in the evaluator today.**
- In the SDC driver, revising an artifact also moves the case revision (`case["revision"] = c{n}`, `$HOME/beyond10x/sdc/driver/src/main.rs:280-285`). So spec r1 to r2 is case revision `c0` to `c1`.
- `Authority` keys decisions by `(CapabilityId, Scope)` only (`crates/canon/src/eval/authority.rs:45`).
- `Authority::decide` (`:53`) reads no revision.
- `actions::section` is not given the case (`crates/canon/src/eval/actions.rs:78-84`, called at `crates/canon/src/eval/mod.rs:444`).
- So any grant passed in applies at every case revision.
- The driver writes `{capability, decision}` and keeps the principal only in its own event log (`main.rs:366-376`).

**How `canon-decisions/1` is checked today.**
- `case_revision` and `principal` are required (`crates/canon/src/model/explicit.rs:33,36`).
- A decision applies only when its `case_revision` equals `Case.revision`. None applies when the snapshot has no revision (`eval/decisions.rs:28-33,57-79`; `ess/domains/protocol.yaml:837-865,1133-1140`).
- This story reuses that rule. Both fields are optional here so that unbound documents keep today's meaning.

**Boundary.**
- The format and the applicability rule are Canon's. Whether a grant applies decides admissibility, which is evaluation.
- Recording `case_revision` when a grant is taken is the runner's job (the SDC `approve` command). The SDC driver is not in this story.
- Resolving the principal belongs to Mandate (design § 23).

**Not verified.** I did not read takt's workaround (`takt/crates/takt/src/replay.rs:7-9`, cited in `boundary.md:82`).

## Decision needed

A bound `denied` given at `c1`: does it still block at `c2`?

| | Rule | `ess/` and model | Cost |
|---|---|---|---|
| A | Every bound entry, whatever its decision, applies only at its case revision. | One comment on `AuthorityDecision` in `protocol.yaml`. | A bound deny lapses when the case is revised. A caller who wants a lasting deny leaves `case_revision` out. |
| B | Bound `granted` and `require_approval` lapse. A bound `denied` holds at every revision. | Comments on `AuthorityDecision` and `Grant`. A branch in `decide`. | `case_revision` means two things depending on the decision. The caller cannot withdraw a deny by revising the case. |

Recommendation: A. It is the `canon-decisions/1` rule unchanged, and a deny written without `case_revision` already holds at every revision.

## ESS first

**First commit, specification only:**
- `ess/domains/protocol.yaml`: `canon.protocol.AuthorityDecision` (`:885`) gains
  - `principal: Optional<canon.protocol.Principal>`
  - `case_revision: Optional<canon.protocol.Revision>`
- Comments updated on `Principal` (`:298-306`), `Revision` (`:723-728`), `Case.revision` (`:1133-1140`) and `AuthorityDecision`, with `# read from:` refreshed.
- `CANON-AUTHORITY-002` added to `conformance/requirements.yaml`. That id is free: the file catalogues 20 ids and no story names this one.
- New scenario `conformance/scenarios/authority-binding.yaml`, over the existing fixture `fixtures/investigation/capability-scope.yaml`.

**Red on that commit:**
- `crates/canon/tests/ess_model_matches.rs::specification_and_model_are_equal` names `AuthorityDecision.principal` and `AuthorityDecision.case_revision`.
- `canon conform run --requirements conformance/requirements.yaml` fails on `CANON-AUTHORITY-002`. The reason: `#[serde(deny_unknown_fields)]` (`crates/canon/src/model/authority.rs:36`) refuses `case_revision` as `malformed-input`.

## Scope

- **In:** the two optional fields in the model, the reader and the parity test. — cited
- **In:** the applicability filter before coverage and precedence. — cited
- **In:** the extended duplicate key and refusal text. — inferred
- **In:** the explanation record. — cited
- **In:** module docs of `eval/authority.rs`, `eval/actions.rs` and `eval/mod.rs`. — cited
- **Out:** binding a grant to an artifact revision instead of the case revision (the request names the case revision). — inferred
- **Out:** descent between revisions. A grant does not hold at a descendant revision. story:revision-ordering adds `descends_from` on `CaseArtifact` for order forms, not authority. — cited
- **Out:** matching the principal against an acting principal. — inferred
- **Out:** the SDC driver writing `case_revision`. — cited
- **Out:** `canon check` and `canon generate` enumerating bound decisions. They write unbound `{capability, decision}` (`crates/canon/src/check/space.rs:1388-1407`), which keeps today's meaning. — inferred
- **Surfaces:**
  - `ess/domains/protocol.yaml` — cited
  - `crates/canon/src/model/authority.rs` — cited
  - `crates/canon/src/eval/authority.rs` — cited
  - `crates/canon/src/eval/actions.rs` — cited
  - `crates/canon/src/eval/mod.rs` — cited
  - `crates/canon/src/explain/mod.rs` — cited
  - `conformance/requirements.yaml` — cited
  - `conformance/scenarios/authority-binding.yaml` (new) — cited
  - `crates/canon-cli/tests/conform_requirements.rs` (requirement count) — inferred
  - `crates/canon-cli/tests/adversary_conformance_suite_cli.rs` — inferred
  - `crates/canon-cli/src/lib.rs` (`--authority` help, `:82-85`) — inferred
- **Coordinator's at integration:** regenerated `website/docs/reference/documents.md`, `evaluation.md`, `website/static/schemas/authority-1.schema.json`, `website/data/status.json`, and an entry in `website/status.yaml`.
- **Confidence:** high for the model, ESS and reader; medium for how the case revision reaches `actions::section` (a new parameter, or filtering in `eval/mod.rs:416-444`), which is the implementor's choice.

## Order

**Depends on:** story:capability-scope (implemented) and story:decision-outcomes (implemented, source of the rule).

**Shares files with these in-flight stories:**
- `ess/domains/protocol.yaml` and `conformance/requirements.yaml`: story:protocol-imports and story:revision-ordering.
- `crates/canon/src/eval/actions.rs`: story:revision-ordering (inferred in its scope).
- `crates/canon/src/eval/mod.rs` and `crates/canon/src/explain/mod.rs`: story:invalidation-evaluation-cost.
- `crates/canon-cli/src/lib.rs`, only if the help is edited: story:protocol-floor, story:protocol-imports and story:canon-mutate.

Run it in a wave without story:revision-ordering and story:invalidation-evaluation-cost.

## Acceptance

Conformance scenario `CANON-AUTHORITY-002` passes under `canon conform run` over `fixtures/investigation/capability-scope.yaml`. In it, action `inspect` requires `metrics.read{cluster: prod-eu, namespace: foo}` and the case snapshot has `revision: c2` unless stated otherwise:

1. A `granted` entry for `{cluster: prod-eu}` with `case_revision: c1` gives `approval-required`, with reason `{"capability": "metrics.read", "decision": "none", "scope": {…}}`.
2. The same entry with `case_revision: c2` gives `admissible`.
3. An unbound `granted` entry with `principal: p` gives `admissible`, the same bytes as without `principal`.
4. An unbound `granted` entry beside a `denied` entry bound to `c1` gives `admissible`, because the stale deny takes no part (option A).
5. A `denied` entry bound to `c2` beside an unbound `granted` entry for the exact scope gives `blocked`, naming the denied capability with its scope.
6. A `granted` entry bound to `c2`, with a snapshot that has no `revision`, gives `approval-required`.
7. The same capability and scope granted at `c1` and at `c2` is read. Two entries equal in capability, scope, `case_revision` and `principal` are refused as `duplicate-identifier`.

## Source

- Request item 1, `$HOME/beyond10x/sdc/docs/canon-requests.md:12-16`.
- SDC files read: `$HOME/beyond10x/sdc/docs/driver.md`, `docs/boundary.md`, `docs/phases.md`, `driver/src/main.rs`.
- Canon tree at `15cfece`: `AGENTS.md`, `crates/canon/src/model/{authority,explicit,case,evidence}.rs`, `crates/canon/src/eval/{authority,actions,decisions,mod}.rs`, `crates/canon/src/explain/mod.rs`, `crates/canon/src/check/space.rs`, `crates/canon/src/conform/mod.rs`, `ess/domains/protocol.yaml`, `conformance/requirements.yaml`, `conformance/scenarios/capability-scope.yaml`.
- Design § 23, § 33, § 37.
- Stories: story:capability-scope, story:revision-ordering, and the scopes of story:protocol-floor, story:protocol-imports, story:invalidation-evaluation-cost and story:canon-mutate.
- `documents.md` at `8f1f452`.
- Date: 2026-11-20.

## Decided

Option A of § Decision needed: every bound entry, whatever its decision, applies only at its case revision; a deny that must last is written without `case_revision`. Decided on 2026-11-20 by the wave coordinator on the drafter's recommendation, after the operator accepted the story without choosing (`approval-record:accept-sdc-requests-2026-11-20`). The unit builds this option and does not reopen the choice.
