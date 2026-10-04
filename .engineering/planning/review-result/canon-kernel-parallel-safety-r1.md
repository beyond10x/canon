---
format: aep.planning-md/3
id: review-result:canon-kernel-parallel-safety-r1
kind: review-result
status: active
title: Canon kernel decomposition — parallel-safety critic, round 1
relations:
- reviews: epic:canon-kernel
- reviews: story:action-admissibility
- reviews: story:canon-ir
- reviews: story:conformance-suite
- reviews: story:evidence-freshness
- reviews: story:evidence-revision-binding
- reviews: story:explanation
- reviews: story:obligations
- reviews: story:outcomes
- reviews: story:protocol-source-model
- reviews: story:semantic-diff
- reviews: story:three-valued-claims
revision: 1
---
needs-revision

action-admissibility — shares the investigation fixture and `canon evaluate` with obligations, outcomes and evidence-freshness (unordered siblings, inferred), and neither body says so — .engineering/planning/story/action-admissibility.md:23
obligations — is unordered against action-admissibility, outcomes and evidence-revision-binding (all depend only on story:three-valued-claims) and all extend one evaluator, one evaluation-result shape and one conformance-scenario registry (inferred; no body names a file), yet no body names the others or the shared surface — .engineering/planning/story/obligations.md:12
evidence-freshness — the body says it edits the `protocol/1` source model, the IR and a scenario (cited), which protocol-source-model and canon-ir own; it is unordered against obligations, action-admissibility and outcomes and does not say how they stay apart — .engineering/planning/story/evidence-freshness.md:20
conformance-suite — and story:semantic-diff each add a subcommand (`canon conform run`, `canon diff`) to the same clap CLI entry (inferred; neither depends on the other, and the CLI file does not exist yet) without saying so — .engineering/planning/story/conformance-suite.md:21
protocol-source-model — it states no crate layout (replace `crates/canon` or create the design § 28 crates), and this decides every later item's surface; `aep plan artifact waves` records no scope for any story — .engineering/planning/story/protocol-source-model.md:32

Notes on the findings:
- **Shared evaluator (obligations):** the evaluator, the result struct and the scenario registry are named by nothing in the tree. Obligations, actions, outcomes and the evidence-revision-binding story each extend them. Both remedies stay open: an ordering edge recording the shared surface as its reason, or splitting the surface (one module and one scenario file per construct, with the result shape fixed first).
- **Shared fixture (action-admissibility):** the §12 example at `docs/design/canon-protocol-calculus-design.md:625-663` has claims, actions and outcomes only. It has no obligation, no capability requirement and no maximum age. The acceptance of obligations, action-admissibility and evidence-freshness (and of semantic-diff, which sits after freshness) all run "over the investigation fixture", so each must edit that one fixture, and nobody owns who adds what. Remedies: an ordering edge naming the fixture, or one fixture or variant file per story.
- **Source model and IR (evidence-freshness):** I found no body showing that obligations, action-admissibility or outcomes also edit the parser, validator or IR. The collision claim rests on the fixture and the evaluator, and is inferred.
- **CLI entry (conformance-suite):** protocol-source-model creates the CLI (its `canon validate` is a "thin shell"), so the file is not there yet and is created before both items. If the crates are split, the collision moves to the canon-cli entry file and to `Cargo.lock`.
- **Layout and scope (protocol-source-model):** `aep plan artifact waves` prints "0 wave(s), 0 collision(s), 11 unassessed". A wave cannot be derived, and the four-way fan-out above would look parallel-safe by default.

What I read: 15 artifacts (11 stories, the epic, 2 decision-blockers, 2 visions via the graph), using `aep plan artifact list`, `show` for 13, `graph`, `waves` and `validate` (valid). I also read `crates/canon/src/lib.rs`, `crates/canon/Cargo.toml`, `Cargo.toml`, `AGENTS.md`, `Taskfile.yml`, design § 12 and § 28, and `docs/contracts/protocol-core.md`. Surfaces: cited 2 (protocol-source-model and action-admissibility both cite `crates/canon/src/lib.rs`; the latter only as inference for `ApprovalRequired`). Inferred 5 (three-valued-claims, canon-ir, explanation, conformance-suite and semantic-diff, from the CLI commands they name and the design § 28 crate names). Unplaceable by file 4 (obligations, outcomes, evidence-revision-binding, evidence-freshness; I placed them only at "the evaluator", which has no file yet).

Could not establish:
- No story records a scope. The wave command reports 11 unassessed, so every collision above is inferred, not read from a scope.
- Which crates the work lands in: the single `b10x-canon` crate or the §28 split. The pairs above collide in either layout, but the file differs.
- Out of my lane, for the design and acceptance critics:
  - action-admissibility cites `crates/canon/src/lib.rs:36`, a file protocol-source-model replaces.
  - three-valued-claims and evidence-revision-binding both define the evidence record and case snapshot as evaluator input. They are ordered, so there is no concurrency issue, but the ownership overlaps.
  - canon-decision/1 is defined in explanation, yet obligations, action-admissibility and outcomes must already emit their statuses on `canon evaluate`.

```findings
- file: .engineering/planning/story/action-admissibility.md
  line: 23
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: shares the investigation fixture (design §12 has no obligation, capability requirement or max age) and the `canon evaluate` surface with obligations, outcomes and evidence-freshness, which are unordered siblings, and no body says so; surface inferred, no file cited
- file: .engineering/planning/story/obligations.md
  line: 12
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: obligations, action-admissibility, outcomes and evidence-revision-binding all depend only on three-valued-claims and extend one evaluator, one evaluation-result shape and one conformance-scenario registry, with no body naming the others or the shared surface; surface inferred, no body cites a file and none exists yet
- file: .engineering/planning/story/evidence-freshness.md
  line: 20
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: body says it edits the protocol/1 source model, the IR and a scenario (cited), surfaces owned by protocol-source-model and canon-ir, while unordered against obligations, action-admissibility and outcomes, and does not say how they stay apart; the collision with those three is inferred
- file: .engineering/planning/story/conformance-suite.md
  line: 21
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: conformance-suite and semantic-diff each add a subcommand to the same clap CLI entry and are unordered, yet neither says so; surface inferred from the command names, and the CLI file does not exist yet
- file: .engineering/planning/story/protocol-source-model.md
  line: 32
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: states no crate layout (replace crates/canon or create the design §28 crates) and no story records a scope, so `aep plan artifact waves` prints 11 unassessed and no concurrent set can be established; surface not established for any story
```
