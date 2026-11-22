---
format: aep.planning-md/3
id: story:evidence-digest-binds-approvals
kind: story
status: active
title: an approval's evidence ground can carry a sha256 digest and lapses as content_changed when the content differs
tags:
- from-finding
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: conformance
- confidence: cited
  path: crates/canon-cli/src/lib.rs
- confidence: cited
  path: crates/canon/src/eval/authority.rs
- confidence: cited
  path: crates/canon/src/eval/decisions.rs
- confidence: cited
  path: crates/canon/src/eval/grounds.rs
- confidence: cited
  path: crates/canon/src/model/authority.rs
- confidence: cited
  path: crates/canon/src/model/explicit.rs
- confidence: cited
  path: ess/domains/cli.yaml
- confidence: cited
  path: ess/domains/protocol.yaml
- confidence: cited
  path: website/static/schemas
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T22:17:48Z", actor: "human:timo", revision: 12, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
- {from: "proposed", to: "active", at: "2026-11-22T22:17:48Z", actor: "human:timo", revision: 13, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
---
## Outcome

An approval or grant that cites evidence can carry a sha256 digest of that evidence's content; when the evidence record Canon reads under that ID has other contents, the ground fails as `content_changed` and the approval no longer holds. Inputs without a digest give the bytes they give today.

## Why

- Review of tag 0.10.0 (security finding 2): approvals depend on immutable evidence IDs; enforce immutability at ingestion or bind approvals to a content digest.
- Wave 12 (`story:approvals-bound-to-evidence-content`) found that no input format has a place for a digest: `canon-authority/1` and `canon-decisions/1` cite evidence as `ground_evidence: Vec<EvidenceId>` (`crates/canon/src/model/explicit.rs`, `crates/canon/src/model/authority.rs`) and reject unknown fields. It pinned today's behaviour (`crates/canon/tests/approvals_cite_evidence_by_id.rs`) and documented that evidence IDs must be immutable once an approval cites them.
- Decided by Timo, 2026-11-22 23:17:01 CET, relayed by `host-operator`: "A: optional digest (Recommended)".

## Scope

- **In:**
  - `ground_evidence` entries in `canon-authority/1` and `canon-decisions/1` accept either a bare ID (as today) or an object with the ID and an optional `digest: sha256:<hex>`; the JSON Schemas under `website/static/schemas/` and the ESS model `ess/domains/protocol.yaml` follow.
  - Define which bytes are hashed (a canonical form of the `canon-evidence/1` record as Canon reads it) and write it in the reference.
  - `crates/canon/src/eval/grounds.rs`: a record whose digest differs fails the ground with a new reason `content_changed`, shown in the decision's stale-grounds output beside `present: false`.
  - A conformance scenario and requirement for the new rule; a test that replaces the contents under one ID and shows the approval lapses; the pinning test of wave 12 stays green for inputs without a digest.
  - A way for a producer to compute the digest (a `canon` subcommand or a documented command), so the field can be used.
- **Out:** a new version of any format (`/2`); a change to inputs without the field; ingestion checks in a store.
- **Surfaces:** `crates/canon/src/model/explicit.rs`, `crates/canon/src/model/authority.rs`, `crates/canon/src/eval/grounds.rs`, `crates/canon/src/eval/decisions.rs`, `crates/canon/src/eval/authority.rs`, `crates/canon-cli/src/lib.rs`, `ess/domains/protocol.yaml`, `ess/domains/cli.yaml`, `website/static/schemas/`, `conformance/`.

## Acceptance

- Every existing input (fixtures, scenarios, tests) gives byte-identical output (cmp against the base build).
- An approval with a digest holds while the record's content is the one hashed, and lapses with `content_changed` after the content under the same ID changes; red test first.
- A Canon without this change refuses a document that uses the field (the formats reject unknown fields); the release notes say so.
- Generated reference and schemas current (`task docs-check`); the conformance run covers the new requirement.

## Source

- **Request:** Timo's pick "A: optional digest" at 23:17:01 CET, relayed by `host-operator`; the options were put by the coordinator after unit 6 of wave 12 reported.
