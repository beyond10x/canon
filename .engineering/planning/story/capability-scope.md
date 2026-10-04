---
format: aep.planning-md/3
id: story:capability-scope
kind: story
status: draft
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
scope:
- confidence: cited
  path: conformance/scenarios/capability-scope.yaml
- confidence: cited
  path: crates/canon/src/eval/authority.rs
- confidence: cited
  path: crates/canon/src/ir/
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/validate/
- confidence: cited
  path: ess/
- confidence: cited
  path: fixtures/investigation/capability-scope.yaml
revision: 3
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
(story:action-admissibility), which gains a scope per decision and the value `require_approval`
beside `granted` and `denied`. Canon grants nothing and derives no grant from the subject (canon
design § 33); whoever evaluates the case supplies its grants. `canon-case/1` is not changed, so
`crates/canon/src/eval/case.rs` stays untouched (story:evaluator-skeleton § Rules).

## ESS first

- Specification change, first commit: in `ess/domains/protocol.yaml`, an optional scope on
  `canon.protocol.CapabilityRequirement` (today it holds only `capability`,
  `crates/canon/src/model/mod.rs:120-121`); with the fixture
  `fixtures/investigation/capability-scope.yaml` and the scenario file
  `conformance/scenarios/capability-scope.yaml`.
- Red on that commit: `crates/canon/tests/ess_model_matches.rs` fails naming
  `CapabilityRequirement.scope`; `CANON-SCOPE-001` fails under `canon conform run`.

## Scope

- In: the scope on requirements (model, validator, IR); scoped decisions and `require_approval` in
  `canon-authority/1`; coverage and precedence in the authority check.
- Out: path scopes for telling test edits from code edits (ADR 0084 § Decision 1 names the need;
  this story's scope map can carry a path key, matching paths is not built here); `scope.widen`
  (ADR 0083 § Decision 5), the Connectors catalog and runtime binding (Commission, Connectors);
  classifying scope changes in `canon diff`.
- Surfaces: `ess/`, `crates/canon/src/model/`, `crates/canon/src/validate/`, `crates/canon/src/ir/`,
  `crates/canon/src/eval/authority.rs`, `fixtures/investigation/capability-scope.yaml`,
  `conformance/scenarios/capability-scope.yaml`.

## Order

depends_on story:action-admissibility (it extends that story's `canon-authority/1` and
`eval/authority.rs`), which comes after story:evaluator-skeleton.

depends_on story:invalidation-rules is an ordering edge, stated as one: every story that changes
`protocol/1` shares `ess/`, `model/`, `validate/` and `ir/` and runs one per wave regardless; the
edge keeps the proposed chain that story:semantic-diff is written against in front (reasoning in
story:case-composition § Order).

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
