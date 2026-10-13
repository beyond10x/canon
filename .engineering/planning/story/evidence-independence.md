---
format: aep.planning-md/3
id: story:evidence-independence
kind: story
status: implemented
title: Evidence requirements that demand an independent producer
refs:
- provider: atlas
  reference: adr:0084
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:three-valued-claims
- depends_on: story:evaluator-skeleton
- depends_on: story:invalidation-rules
- depends_on: story:ess-pin-0-59
scope:
- confidence: cited
  path: conformance/scenarios/evidence-independence.yaml
- confidence: cited
  path: crates/canon-docs/
- confidence: cited
  path: crates/canon/src/check/
- confidence: cited
  path: crates/canon/src/diff/
- confidence: cited
  path: crates/canon/src/eval/
- confidence: cited
  path: crates/canon/src/explain/
- confidence: cited
  path: crates/canon/src/generate/
- confidence: cited
  path: crates/canon/src/ir/
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/validate/
- confidence: cited
  path: ess/
- confidence: cited
  path: fixtures/investigation/evidence-independence.yaml
- confidence: cited
  path: website/
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-13T11:05:08Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-13T11:05:08Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-10-13T12:38:04Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Outcome

An evidence match can demand independence: it names the independence dimensions its records must
satisfy, and a record satisfies it only when the trusted context says the record's producer is
independent on every one of them. A record that fails the requirement is inapplicable to that match,
whatever its result: a claim resting only on such records is UNKNOWN, not FALSE (CANON-INDEPENDENCE-001,
design § 32; design § 39.4: Canon declares the requirement shape, trusted runtime context resolves
identities and relationships). This carries AEP's `EvidenceRequirement.independent` (aep
`crates/govern/aep-domain/src/requirement.rs:291`) into Canon, as Atlas ADR 0084 § "Also needed"
requires, and is the form the engineering-protocols stories `story:security-independence-rules` and
`story:rollback-verification-rule` declare their rules in.

## First step: where independence is declared (settled 2026-10-13)

ADR 0084 § Open left "on the evidence kind, or on the requirement" open. Settled: on the
requirement, because one kind can be required independent by one claim and not by another.

- `protocol/1`: a top-level `independence_dimensions` section declares the dimension ids the
  protocol uses (each with an optional `description`), like `inputs` declares case inputs. Canon gives
  no dimension a meaning; a domain names them (`different_principal`, `different_agent_run`).
- `protocol/1`: an evidence match (`{evidence: {kind, result, subject}}`) takes an optional
  `independent_on`, a non-empty list of declared dimension ids, without duplicates.
- `canon-evidence/1`: a record takes an optional `producer`, an opaque principal id naming who
  produced it. Canon does not prove it (signing, attestation are out).
- `canon-case/1`: the case takes an optional `independence` map, keyed by producer, whose value is
  the list of declared dimensions on which that producer is independent of the case's work. The
  trusted context that writes the case resolves it; a producer absent from the map, or listed with
  `[]`, is independent on no dimension.
- Evaluation: a record matches an evidence match with `independent_on: D` only when it has a
  `producer` and the case lists every dimension of `D` for that producer. Otherwise the record is
  excluded from that match with the reason `not_independent` in the explanation, beside
  `revision_mismatch`, `expired` and `invalidated`. A match without `independent_on` is unchanged.
  The order forms (`first`, `before`) are unchanged: they read the evidence history.
- Validation refuses an `independent_on` naming an undeclared dimension, an empty or duplicated
  list, and a case `independence` entry naming an undeclared dimension.
- `canon diff` classifies adding a dimension to an evidence match's `independent_on` as TIGHTENING
  and removing one as RELAXATION (design § 31, "stronger independence requirement"), so a changed
  requirement is never reported as no change.

## ESS first

- Specification change, first commit: in `ess/domains/protocol.yaml`, the `independence_dimensions`
  section, `independent_on` on the evidence match of `canon.protocol.Predicate`, `producer` on the
  evidence record and `independence` on the Case; with the fixture
  `fixtures/investigation/evidence-independence.yaml` and the scenario file
  `conformance/scenarios/evidence-independence.yaml`.
- Red on that commit: `crates/canon/tests/ess_model_matches.rs` fails naming the new fields; then
  `CANON-INDEPENDENCE-001` fails under `canon conform run`.

## Scope

- In: the dimension declarations, `independent_on`, `producer` and the case's `independence` in
  model, validator, IR, evaluator, explanation and diff; the generated reference pages and schemas;
  the status page row for CANON-INDEPENDENCE-001.
- Out: proving who produced a record (signing, attestation), which Canon takes as input; resolving
  identities or relationships, which the trusted context does; the requirements catalogue
  (story:conformance-suite).
- Surfaces: `ess/`, `crates/canon/src/model/`, `crates/canon/src/validate/`, `crates/canon/src/ir/`,
  `crates/canon/src/eval/` (`claims.rs`, `evidence.rs`, new `independence.rs`),
  `crates/canon/src/explain/`, `crates/canon/src/diff/`, `crates/canon-docs/`,
  `website/docs/reference/`, `website/static/schemas/`, `website/data/`, `website/status.yaml`,
  `fixtures/investigation/evidence-independence.yaml`,
  `conformance/scenarios/evidence-independence.yaml`.

## Order

depends_on story:three-valued-claims (it changes which records satisfy a match) and
story:evaluator-skeleton (it adds `eval/independence.rs` beside the per-concept files).
depends_on story:invalidation-rules is an ordering edge: every story that changes `protocol/1`
shares `ess/`, `model/`, `validate/` and `ir/` and runs one per wave.

## Acceptance

Conformance scenario `CANON-INDEPENDENCE-001` passes under `canon conform run` over
`fixtures/investigation/evidence-independence.yaml`, for a claim over a `review` match with
`independent_on: [different_principal]`, with five expectations: no record leaves the claim
UNKNOWN; an approving record whose producer the case lists with `[]` leaves it UNKNOWN; an
approving record whose producer the case lists on `different_principal` makes it TRUE; both records
together make it TRUE; an approving record without a `producer` leaves it UNKNOWN. A sixth
expectation: the same claim with `independent_on` removed is TRUE on the non-independent record
alone. `crates/canon/tests/ess_model_matches.rs` passes; `canon diff` of the protocol before and
after adding `independent_on` reports TIGHTENING.

## Source

Atlas ADR 0084 § Also needed and § Open; `docs/design/canon-protocol-calculus-design.md` § 7.1,
§ 31, § 32 (CANON-INDEPENDENCE-001), § 39.4; aep `crates/govern/aep-domain/src/requirement.rs:291`;
Atlas ADR 0080.
