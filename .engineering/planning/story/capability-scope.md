---
format: aep.planning-md/3
id: story:capability-scope
kind: story
status: active
title: Scope capability requirements and grants, default deny, deny over approval over allow
refs:
- provider: atlas
  reference: adr:0083
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:action-admissibility
- depends_on: story:invalidation-rules
- depends_on: story:conformance-suite
scope:
- confidence: cited
  path: conformance/requirements.yaml
- confidence: cited
  path: conformance/scenarios/capability-scope.yaml
- confidence: inferred
  path: crates/canon-cli/src/lib.rs
- confidence: inferred
  path: crates/canon/src/check/space.rs
- confidence: inferred
  path: crates/canon/src/diff/mod.rs
- confidence: cited
  path: crates/canon/src/eval/actions.rs
- confidence: cited
  path: crates/canon/src/eval/authority.rs
- confidence: cited
  path: crates/canon/src/eval/read.rs
- confidence: inferred
  path: crates/canon/src/explain/mod.rs
- confidence: cited
  path: crates/canon/src/ir/mod.rs
- confidence: cited
  path: crates/canon/src/model/authority.rs
- confidence: cited
  path: crates/canon/src/model/mod.rs
- confidence: cited
  path: crates/canon/src/validate/mod.rs
- confidence: inferred
  path: crates/canon/tests/missing_keys.rs
- confidence: inferred
  path: crates/canon/tests/review_refusal_naming.rs
- confidence: cited
  path: ess/domains/protocol.yaml
- confidence: cited
  path: fixtures/investigation/capability-scope.yaml
revision: 35
transitions:
- {from: "draft", to: "proposed", at: "2026-11-20T20:58:27Z", actor: "human:timo", revision: 5, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w1"}
- {from: "proposed", to: "active", at: "2026-11-20T20:58:27Z", actor: "human:timo", revision: 6, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w1"}
---
## Outcome

Capabilities carry a scope (Atlas ADR 0083 § Decision 1). A protocol action's capability
requirement may name a scope (`metrics.inspect` requires `metrics.read{cluster, namespace}`), and
the case's grants are scoped (`metrics.read{cluster: prod-eu, namespace: foo}`). Canon decides each
requirement from the grants with AEP's capability model (aep
`crates/govern/aep-domain/src/capability.rs`, ADR 0083 cites `c0d5f852`):

- **Default deny** (`:8-11`): a requirement no grant covers is not granted, and the action is not
  admissible (story:action-admissibility reports it `approval-required`, unchanged).
- **Coverage** (`:13-19`): a grant with no value for a scope key covers every value of it; coverage
  widens from the wildcard outwards, never inwards, so a grant for `{cluster: prod-eu, namespace:
  foo}` does not cover a requirement for `{cluster: prod-eu}`.
- **Precedence** (`:21-28`): among the grants covering a requirement,
  `deny > require_approval > allow > (nothing) = not granted`. Deny wins unconditionally and
  reports the action `blocked`; `require_approval` reports it `approval-required`.

Where the case's grants live is decided here: they are the case's `canon-authority/1` input
(introduced by story:review-hardening-w7, read for admissibility by story:action-admissibility),
which gains a scope per decision and the value `require_approval` beside `granted` and `denied`. Canon grants nothing and derives no grant from the subject (canon
design § 33); whoever evaluates the case supplies its grants. `canon-case/1` is not changed, so
`crates/canon/src/eval/case.rs` stays untouched (story:evaluator-skeleton § Rules).

## ESS first

- Specification change, first commit: in `ess/domains/protocol.yaml`, an optional scope on
  `canon.protocol.CapabilityRequirement` (today it holds only `capability`,
  `crates/canon/src/model/mod.rs:237-242`), a scope on `canon.protocol.AuthorityDecision`, and the
  value `require_approval` on `canon.protocol.Grant` (today `granted` and `denied`,
  `protocol.yaml:879-881`); with the fixture `fixtures/investigation/capability-scope.yaml`, the
  scenario file `conformance/scenarios/capability-scope.yaml` and the `CANON-SCOPE-001` entry in
  `conformance/requirements.yaml`.
- Red on that commit: `crates/canon/tests/ess_model_matches.rs::specification_and_model_are_equal`
  fails naming `CapabilityRequirement.scope` and the `Grant` variant `require_approval`; and
  `canon conform run --requirements conformance/requirements.yaml` fails on `CANON-SCOPE-001`,
  because `deny_unknown_fields` (`model/mod.rs:239`, `model/authority.rs:20`) refuses the fixture's
  scope and the authority document's `require_approval`.

## Scope

Rescoped 2026-11-20 by `story-scoper` against `8f1f452`, written by the wave coordinator. Every
line is **cited** (read from the story or the tree) or **inferred** (a reading that could be
wrong).

- **In:** an optional scope on `CapabilityRequirement` in the model, the validator and the IR,
  including the `canon-ir/1` read-back; scoped decisions and `require_approval` in
  `canon-authority/1`; coverage and precedence in the admissibility check; the explanation's
  record of the authority decisions. — cited
- **Out:** path scopes for telling test edits from code edits (ADR 0084 § Decision 1 names the
  need; this story's scope map can carry a path key, matching paths is not built here);
  `scope.widen` (ADR 0083 § Decision 5), the Connectors catalog and runtime binding (Commission,
  Connectors); classifying scope changes in `canon diff`, which only has to keep compiling. — cited
- **Out, decided by the rescope:** `canon check` and `canon generate` do not enumerate scoped
  decisions or `require_approval`. They keep 3 values per capability id
  (`crates/canon/src/check/mod.rs:30`, `check/space.rs:359-361`), and the authority documents they
  write stay unscoped `granted` and `denied` (`check/space.rs:1388-1407`). An unscoped decision
  covers every scope, so their findings stay sound. — inferred, the draft was silent
- **Primary surface:** `crates/canon/src/eval/actions.rs:93-131` (`admissibility`, where the
  authority check is today; not `eval/authority.rs`, which only reads and keys decisions) — cited
- **Files:** `ess/domains/protocol.yaml` (`CapabilityRequirement` :226-231, `AuthorityDecision`
  :862-873, `Grant` :879-881; the `# read from:` comments at :225-229 cite
  `model/mod.rs:110-114` and are refreshed here) — cited
- **Files:** `crates/canon/src/model/mod.rs:237-242` (`CapabilityRequirement`) — cited
- **Files:** `crates/canon/src/model/authority.rs:16-48` (`AUTHORITY_FORMAT`,
  `AuthorityDecision`, `Grant`) — cited
- **Files:** `crates/canon/src/eval/authority.rs:32-79` (the reader; one decision per capability
  today, a duplicate refused at :67-75) — cited
- **Files:** `crates/canon/src/ir/mod.rs` (`Action.requires: Vec<CapabilityId>` :155, compile
  :219-225, write :555-566) — cited
- **Files:** `crates/canon/src/eval/read.rs:337-345` (rebuilds requirements from `canon-ir/1` and
  refuses a byte mismatch, `read.rs:4-8`; a scope it dropped would make every scoped IR
  unreadable) — cited
- **Files:** `crates/canon/src/validate/mod.rs:501-504` — cited
- **Files:** `fixtures/investigation/capability-scope.yaml` (new),
  `conformance/scenarios/capability-scope.yaml` (new), `conformance/requirements.yaml`
  (`CANON-SCOPE-001` catalogued; `Taskfile.yml:14` checks it) — cited
- **Also likely:** `crates/canon/src/explain/mod.rs:371-382` (writes `{capability, decision}`
  only); `crates/canon/src/check/space.rs:437-441` (collects `&CapabilityId` from the IR's
  requires, changes if the field type changes); `crates/canon/src/diff/mod.rs:933-967` (exhaustive
  `Action` destructure); `crates/canon-cli/src/lib.rs:82` (the `--authority` help says "granted or
  denied"); the tests `crates/canon/tests/review_refusal_naming.rs:91` and
  `crates/canon/tests/missing_keys.rs:183`, which read the duplicate refusal. — inferred
- **Coordinator's at integration, not in the typed scope:** the story's entry in
  `website/status.yaml`, and the regenerated `website/docs/reference/protocol.md`, `documents.md`,
  `evaluation.md`, `canon-ir.md`, `cli.md`, `website/static/schemas/authority-1.schema.json`,
  `protocol-1.schema.json` and `website/data/status.json`.
- **Confidence:** high for model, ESS, evaluation and IR (read); medium for check, diff and
  explain, which depend on the IR shape the implementor picks.
- **Would collide with:** any unit on `ess/domains/protocol.yaml`,
  `crates/canon/src/model/mod.rs`, `validate/mod.rs`, `ir/mod.rs` or `conformance/requirements.yaml`
  (story:protocol-imports, story:revision-ordering), and any unit on
  `crates/canon/src/explain/mod.rs` (story:invalidation-evaluation-cost).
- **Safety fact:** `canon-case/1` is untouched, and an absent `scope` is the wildcard, so every
  existing protocol and authority document reads unchanged, and the `canon-ir/1` of a protocol
  without a scope stays byte-identical. Unproven.

Decided by the rescope of 2026-11-20, where the draft was silent:

- **The scope's type** is a map from a key to one text value, both non-empty.
- **Duplicates.** The reader refuses two decisions with the same capability and an equal scope, as
  it refuses two decisions for one capability today; two decisions for one capability with
  different scopes are the normal case.
- **Multi-key coverage extends aep.** aep's capability model
  (`crates/govern/aep-domain/src/capability.rs:8-28`, identical at `c0d5f852` and at aep `HEAD`
  `70b3179ca` on 2026-11-20) scopes a capability by one resource-class value. The rule of
  § Outcome for several keys ("a grant with no value for a scope key covers every value of it") is
  this story's, read key by key; aep does not state it.
- **The IR shape** (a scoped requirement type, or a side map) is the implementor's choice.

## Order

Every dependency is implemented (story:action-admissibility, story:invalidation-rules,
story:conformance-suite), so the edges hold nothing back. The draft's ordering edge to
story:invalidation-rules kept a chain in front that story:semantic-diff was written against; that
chain has landed.

Shares `ess/domains/protocol.yaml`, `crates/canon/src/model/mod.rs`, `validate/mod.rs`,
`ir/mod.rs` and `conformance/requirements.yaml` with story:protocol-imports and
story:revision-ordering, so the three run in three different waves.

## Acceptance

Conformance scenario `CANON-SCOPE-001` passes under `canon conform run` over
`fixtures/investigation/capability-scope.yaml`, for one action requiring
`metrics.read{cluster: prod-eu, namespace: foo}`, with five expectations: no grant is
`approval-required`; an `allow` for `{cluster: prod-eu}` is `admissible`; an `allow` for
`{cluster: prod-us}` is `approval-required`; an `allow` and a `require_approval` both covering it is
`approval-required`; and a `deny` for unscoped `metrics.read` beside an `allow` for the exact scope
is `blocked` naming the denied capability. `crates/canon/tests/ess_model_matches.rs` passes.

## Source

Atlas ADR 0083 (accepted 2026-10-04) § Decision 1, 5, 6 and § Open; Atlas ADR 0084 § Decision 1;
aep `crates/govern/aep-domain/src/capability.rs:8-28`; canon design § 33; Atlas ADR 0080.
