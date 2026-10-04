---
format: aep.planning-md/3
id: story:review-hardening-w7
kind: story
status: draft
title: Fix the independent review findings on the wave-7 evaluator
relations:
- decomposes: epic:canon-kernel
- depends_on: story:decision-outcomes
revision: 1
---
## Outcome

The findings of the independent review of canon main 8fc260a (wave 2026-10-04-w7) are fixed:
- a caller-built IR nested beyond `MAX_IR_DEPTH` is refused, never aborting the process;
- the evaluation reference states the outcome and action reason rules as they are;
- a required field left out of a case, evidence record or authority entry is refused naming the field;
- every evidence refusal names the record, and `canon evaluate` names the file;
- the exit-status table covers usage errors (or usage errors exit with a status of their own);
- `canon-authority/1` is declared in `ess/`, with a schema and a documents-page section.

## Acceptance

The reviewer's five test files (`review_cli_contract.rs`, `review_depth_beyond_bound.rs`,
`review_outcome_reasons_docs.rs`, `review_properties.rs`, `review_refusal_naming.rs`, kept at
`ga-wave-2026-10-04-w9/canon-review-findings/`) pass unedited.

## ESS first

`canon-authority/1` gains its declaration in `ess/domains/protocol.yaml`; the five review files are
the red tests of the first commit.
