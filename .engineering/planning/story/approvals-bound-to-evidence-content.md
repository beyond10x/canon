---
format: aep.planning-md/3
id: story:approvals-bound-to-evidence-content
kind: story
status: implemented
title: a persisted approval stops holding when the evidence behind its ID changed, or the immutability requirement is documented and pinned
tags:
- from-finding
relations:
- decomposes: epic:canon-kernel
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: crates/canon/src/eval/decisions.rs
- confidence: cited
  path: crates/canon/src/eval/grounds.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-22T21:20:34Z", actor: "human:timo", revision: 4, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
- {from: "proposed", to: "active", at: "2026-11-22T21:20:34Z", actor: "human:timo", revision: 5, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
- {from: "active", to: "implemented", at: "2026-11-23T03:03:17Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-22-w12"}
---
## Outcome

A persisted approval that rests on evidence is no longer valid after the evidence behind its ID was replaced with other contents, or Canon states in its docs that it checks the ID only and names who must keep IDs immutable.

## Why

- Review of tag 0.10.0 (security): Canon checks whether the evidence ID an approval cites is still applicable. If storage replaces the contents of that ID between evaluations, the original approval still holds. Enforce immutability at ingestion, or bind approvals to a content digest.
- On main (0.12.0, `crates/canon/src/eval/grounds.rs`, byte-identical to 0.10.0, check about line 214): grounds are compared by evidence ID. No digest appears in the file.

## Scope

- **In:** read `grounds.rs`, `decisions.rs` and the evidence and decision models. If an evidence record or a decision's grounds already carry (or can carry, without changing `protocol/1`, `canon-case/1`, `canon-evidence` bytes or `canon-decision/1` bytes of existing cases) a content digest, bind the approval to it and test that replaced contents under one ID invalidate the approval. If binding needs a change of a versioned format, the unit does not make it: it delivers (a) a test that pins today's behaviour, (b) the requirement written in the module docs and the CLI help ("evidence IDs must be immutable once an approval cites them"), and (c) a report with the format options and their cost, for a design story. In both cases the unit says which it did.
- **Out:** a change of a versioned format without Timo's word.
- **Surfaces:** `crates/canon/src/eval/grounds.rs`, `crates/canon/src/eval/decisions.rs`, module docs, `crates/canon-cli/src/lib.rs` (help only).

## Acceptance

- A test replaces the contents of an evidence ID between two evaluations: either the approval no longer holds (bound form, output shown) or the test pins that it holds and the docs say so (documented form).
- Existing decisions' bytes unchanged.

## Source

- **Request:** Timo, 2026-11-22 about 22:10 CET, typed (see story:conform-compiles-the-fixture-once). A change of a versioned format is an architecture change and waits for his word.
