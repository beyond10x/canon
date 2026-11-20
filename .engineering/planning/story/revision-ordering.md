---
format: aep.planning-md/3
id: story:revision-ordering
kind: story
status: active
title: Order red and green across descended revisions with the same test id
refs:
- provider: atlas
  reference: adr:0084
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:evidence-revision-binding
- depends_on: story:evidence-order-predicate
- depends_on: story:conformance-suite
scope:
- confidence: cited
  path: conformance/requirements.yaml
- confidence: cited
  path: conformance/scenarios/revision-ordering.yaml
- confidence: inferred
  path: crates/canon-docs/src/landing.rs
- confidence: inferred
  path: crates/canon-docs/src/pages.rs
- confidence: inferred
  path: crates/canon/src/check/space.rs
- confidence: cited
  path: crates/canon/src/diff/mod.rs
- confidence: inferred
  path: crates/canon/src/eval/actions.rs
- confidence: inferred
  path: crates/canon/src/eval/claims.rs
- confidence: inferred
  path: crates/canon/src/eval/depth.rs
- confidence: cited
  path: crates/canon/src/eval/order.rs
- confidence: inferred
  path: crates/canon/src/eval/read.rs
- confidence: cited
  path: crates/canon/src/ir/mod.rs
- confidence: inferred
  path: crates/canon/src/model/case.rs
- confidence: cited
  path: crates/canon/src/model/evidence.rs
- confidence: cited
  path: crates/canon/src/model/predicate.rs
- confidence: cited
  path: crates/canon/src/validate/mod.rs
- confidence: cited
  path: ess/domains/protocol.yaml
- confidence: cited
  path: fixtures/investigation/revision-ordering.yaml
revision: 32
transitions:
- {from: "draft", to: "proposed", at: "2026-11-20T20:58:28Z", actor: "human:timo", revision: 4, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w1"}
- {from: "proposed", to: "active", at: "2026-11-20T20:58:28Z", actor: "human:timo", revision: 5, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w1"}
---
## Outcome

Revision-based ordering (Atlas ADR 0084 § Decision 3, "later"): red on revision R1 (test added, fix
absent) and green on revision R2 descended from R1, with the same test id. It closes the gap AEP's
`test-driven` principle states about itself: it does not check "that the failing test is a test *of
the change*" (aep `principles/development/test-driven.yaml:50`, quoted from ADR 0084).

story:evidence-revision-binding and story:evidence-order-predicate are implemented. The story
waits only on the revision relation, which its first step settles.

## State on 2026-11-20

Read by the scoper at `8f1f452`:

- An evidence record is bound by `subject` and `subject_revision`
  (`ess/domains/protocol.yaml:755,777`; `crates/canon/src/model/evidence.rs:35`). The case snapshot
  holds one current revision per artifact (`protocol.yaml:744-749`;
  `crates/canon/src/model/case.rs:32,58`). Revisions are "opaque text, compared for equality only"
  (`protocol.yaml:705-708`).
- Four order forms exist: `first`, `before`, `after`, `since` (`protocol.yaml:458,462,467,472`;
  `crates/canon/src/model/predicate.rs:122-132`), evaluated by `value(predicate, history)` at
  `crates/canon/src/eval/order.rs:69`, which takes no `Case`.
- No document states descent between revisions. Two module documents name the gap and this story:
  `model/predicate.rs:31-33,42` and `eval/order.rs:56-58`.
- No record carries a test id or any key red and green could be matched on. The record fields are
  `format, id, kind, result, subject, subject_revision, observed_at, upstream_revisions, producer,
  child_case` (`protocol.yaml:755-817`).

## First step: the revision relation

ADR 0084 § Open leaves "the syntax of the evidence-order predicate and of the revision relation"
open (Atlas `architecture/adr/0084-protocols-express-step-order.md:54`). This story settles the
relation in its first step, before the ESS-first commit, and records the choice in this body. The
scoper listed four options the model admits and picked none:

| | the relation | changes | cost |
|---|---|---|---|
| A | `descends_from: List<Revision>` on `CaseArtifact`: every ancestor of the artifact's current revision | `CaseArtifact` in `protocol.yaml`, `model/case.rs:52-59`, `case-1.schema.json` | `order::value` and its 4 callers are given the `Case`. It answers only "does the current revision descend from X", so green is bound to the current revision. The caller writes the transitive closure. |
| B | `subject_descends_from: List<Revision>` on each `EvidenceRecord` | `protocol.yaml:755`, `model/evidence.rs`, `evidence-1.schema.json` | `value(predicate, history)` keeps its signature. Whoever produced a record claims its descent, so two records can disagree and a new refusal is needed. |
| C | a separate `canon-revisions/1` document of parent edges | a new struct and model file, `DOCUMENTS` in `ess_model_matches.rs:84`, `canon evaluate --revisions`, a `revisions:` list in scenarios, a schema, a cycle refusal | The most general (any pair, merges) and the largest. It edits `crates/canon-cli/src/lib.rs`, which story:protocol-imports and story:protocol-floor also edit. |
| D | `revisions: Map<ArtifactId, Map<Revision, List<Revision>>>` on `Case`: parent edges | `Case` (`protocol.yaml:1058`), `model/case.rs`, `case-1.schema.json` | As general as C with no new document or flag. `order.rs` is given the `Case`. The snapshot grows with history. |

Every option also needs a key that red and green are matched on: a new optional per-record field
on `EvidenceRecord`, with a generic name such as `item`, because Canon holds no engineering
vocabulary (AGENTS.md § Boundary).

**The coordinator's recommendation, 2026-11-20: option A with the key `item`.** Descent is a fact
about an artifact's history, so it belongs to the case snapshot, which whoever evaluates the case
supplies, and not to the producer of a record (B). Both acceptance rows read descent only from the
revision the green record is on, which is the current one. A is the smallest change that answers
them. The operator was asked on 2026-11-20; the unit takes A and `item` unless the store records
another answer before its wave starts.

## ESS first

- Specification change, first commit: in `ess/domains/protocol.yaml`, the revision relation, the
  per-record key and the predicate form that uses them, as the first step settles them; with the
  fixture `fixtures/investigation/revision-ordering.yaml`, the scenario file
  `conformance/scenarios/revision-ordering.yaml` and the `CANON-ORDER-005` entry in
  `conformance/requirements.yaml`.
- Red on that commit: `crates/canon/tests/ess_model_matches.rs::specification_and_model_are_equal`
  fails naming the new declarations; and `canon conform run --requirements
  conformance/requirements.yaml` fails on `CANON-ORDER-005`, because the scenario's evidence
  carries the new key and `deny_unknown_fields` (`model/evidence.rs:16`) refuses it.

The scenario id is `CANON-ORDER-005`. The draft named `CANON-ORDER-002`, which
story:evidence-order-after took (`conformance/requirements.yaml:91-97`); `CANON-ORDER-001` to
`CANON-ORDER-004` are in use.

## Scope

Rescoped 2026-11-20 by `story-scoper` against `8f1f452`, written by the wave coordinator. Every
line is **cited** (read from the story or the tree) or **inferred** (a reading that could be
wrong).

- **In:** the revision relation, the per-record key red and green are matched on, and the
  red-on-R1, green-on-descendant predicate form; scenario `CANON-ORDER-005`. — cited
- **Out:** computing descent from a VCS (Canon reads it as input); any `canon diff` rule beyond
  the structural comparison every order form already gets
  (`crates/canon/src/diff/mod.rs:1137-1143`). A new form cannot skip `diff/mod.rs`, whose matches
  at `:465-471` and `:648-654` are exhaustive. — cited
- **Primary surface:** `crates/canon/src/eval/order.rs` — cited
- **Files:** `ess/domains/protocol.yaml`; `crates/canon/src/model/predicate.rs`;
  `crates/canon/src/model/evidence.rs`; `crates/canon/src/ir/mod.rs` (order arms :299-355,
  :450-461, :743-761); `crates/canon/src/validate/mod.rs` (:749-755, :854);
  `crates/canon/src/diff/mod.rs`; `fixtures/investigation/revision-ordering.yaml` (new);
  `conformance/scenarios/revision-ordering.yaml` (new); `conformance/requirements.yaml`. — cited
- **Files, for option A:** `crates/canon/src/model/case.rs` — inferred
- **Also likely:** `crates/canon/src/eval/claims.rs`, `eval/actions.rs`, `eval/depth.rs`,
  `eval/read.rs`, `crates/canon/src/check/space.rs`, `crates/canon-docs/src/landing.rs`,
  `crates/canon-docs/src/pages.rs`: each matches order-form variants and changes if the form is a
  new variant. — inferred
- **Only for option C, not in the typed scope:** `crates/canon-cli/src/lib.rs`,
  `crates/canon/src/conform/mod.rs`, `crates/canon/src/eval/mod.rs`,
  `crates/canon/tests/ess_model_matches.rs`.
- **Coordinator's at integration, not in the typed scope:** the story's entry in
  `website/status.yaml` (the order capabilities are at `status.yaml:126-138`), and the regenerated
  `website/docs/reference/protocol.md`, `evaluation.md`, `canon-ir.md`,
  `website/static/schemas/protocol-1.schema.json`, `evidence-1.schema.json`, `case-1.schema.json`
  and `website/data/status.json`.
- **Confidence:** medium. The core files are cited; the option chosen in the first step decides
  whether `model/case.rs` joins.
- **Would collide with:** any unit on `ess/domains/protocol.yaml`, `model/predicate.rs`,
  `model/evidence.rs`, `ir/mod.rs`, `validate/mod.rs` or `conformance/requirements.yaml`
  (story:capability-scope, story:protocol-imports), and any unit on `eval/claims.rs`
  (story:invalidation-evaluation-cost).
- **Safety fact:** order forms read only `history` and never the exclusion stages
  (`eval/order.rs:9-14`, `:69`), so adding a form changes no existing claim value. Unproven.

## Order

depends_on story:evidence-revision-binding, story:evidence-order-predicate and
story:conformance-suite: all three are implemented.

Shares `ess/domains/protocol.yaml`, `crates/canon/src/ir/mod.rs`, `validate/mod.rs` and
`conformance/requirements.yaml` with story:capability-scope and story:protocol-imports, so the
three run in three different waves.

## Acceptance

Conformance scenario `CANON-ORDER-005` passes under `canon conform run` over
`fixtures/investigation/revision-ordering.yaml`, with four expectations: test T failed on R1 and
passed on R2 descended from R1 is TRUE; T failed on R1 and passed on R3 not descended from R1 is
FALSE; a failure and a pass under different test ids is FALSE; no record is UNKNOWN.
`crates/canon/tests/ess_model_matches.rs` passes.

## Source

Atlas ADR 0084 (accepted 2026-10-04; on Atlas `main`, commit `38266148c` is an ancestor of
`759e16472`) § Decision 3 (`0084-protocols-express-step-order.md:41-43`) and § Open (`:54`); aep
`principles/development/test-driven.yaml:50`; draft `canon-stories-step-order.md`; Atlas ADR 0080;
rescope of 2026-11-20 (`approval-record:accept-drafts-2026-11-20`).
