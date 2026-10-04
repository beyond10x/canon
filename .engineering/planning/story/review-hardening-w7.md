---
format: aep.planning-md/3
id: story:review-hardening-w7
kind: story
status: implemented
title: Fix the independent review findings on the wave-7 evaluator
relations:
- decomposes: epic:canon-kernel
- depends_on: story:decision-outcomes
- serves: vision:O2
- serves: vision:governed-autonomy
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: conformance/scenarios/review-hardening-w7.yaml
- confidence: cited
  path: crates/canon-cli/src/evaluate.rs
- confidence: cited
  path: crates/canon-cli/src/lib.rs
- confidence: cited
  path: crates/canon-cli/tests/
- confidence: cited
  path: crates/canon-cli/tests/review_cli_contract.rs
- confidence: cited
  path: crates/canon-docs/src/pages.rs
- confidence: cited
  path: crates/canon/src/check/mod.rs
- confidence: cited
  path: crates/canon/src/eval/
- confidence: cited
  path: crates/canon/src/eval/authority.rs
- confidence: cited
  path: crates/canon/src/eval/depth.rs
- confidence: cited
  path: crates/canon/src/eval/evidence.rs
- confidence: cited
  path: crates/canon/src/model/
- confidence: cited
  path: crates/canon/src/validate/mod.rs
- confidence: cited
  path: crates/canon/tests/
- confidence: cited
  path: crates/canon/tests/ess_model_matches.rs
- confidence: cited
  path: ess/
- confidence: cited
  path: fixtures/investigation/subject-bound-reasons.yaml
- confidence: cited
  path: website/
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T09:51:21Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-04T09:51:21Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-10-04T12:11:09Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"test_result":1,"review_outcome":5,"verification":1}}}
---
## Outcome

The findings of the independent review of canon main 8fc260a (wave 2026-10-04-w7) are fixed:
- a caller-built IR nested beyond `MAX_IR_DEPTH` is refused, never aborting the process;
- the evaluation reference states the outcome and action reason rules as they are;
- a required field left out of a case, evidence record or authority entry is refused naming the field;
- every evidence refusal names the record, and `canon evaluate` names the file;
- the exit-status table covers usage errors (or usage errors exit with a status of their own);
- `canon-authority/1` is declared in `ess/`, with a schema and a documents-page section.

- a conformance scenario holds the subject-bound evidence reason shape `{evidence, present, subject}` in the `actions` and `outcomes` sections (CANON-EVIDENCE-003 has no actions or outcomes step; adversary pass 2 of story:subject-bound-evidence-match, ess/domains/protocol.yaml:434).

## Acceptance

The reviewer's five test files (`review_cli_contract.rs`, `review_depth_beyond_bound.rs`,
`review_outcome_reasons_docs.rs`, `review_properties.rs`, `review_refusal_naming.rs`, kept at
`ga-wave-2026-10-04-w9/canon-review-findings/`) pass unedited.

## ESS first

`canon-authority/1` gains its declaration in `ess/domains/protocol.yaml`; the five review files are
the red tests of the first commit.

## Coordinator decisions (wave 2026-10-04-w14)

- `review_properties.rs` property 5 sets `explanation.computed_from.case.termination` aside before
  comparing: the explanation records the case it read since story:explanation (93f527f); every other
  assertion is unchanged.
- Outcome reasons follow the action rule: in an `all` that is false only its false members decide
  it, with polarity through `not`; the reference states one rule for both sections.
- Scope widened to `eval/`, `model/`, the ESS model test, the canon-docs documents table, `website/`,
  `ess/` and `AGENTS.md`, which the fixes reach.
