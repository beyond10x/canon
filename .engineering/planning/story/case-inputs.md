---
format: aep.planning-md/3
id: story:case-inputs
kind: story
status: draft
title: Declare case inputs in protocol/1 and derive obligations from them
refs:
- provider: taskboard
  reference: C-001
- provider: taskboard
  reference: C-005
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:semantic-diff
scope:
- confidence: inferred
  path: conformance/scenarios/case-inputs.yaml
- confidence: inferred
  path: crates/canon-cli/tests/case_inputs.rs
- confidence: inferred
  path: crates/canon/src/check/
- confidence: inferred
  path: crates/canon/src/diff/
- confidence: inferred
  path: crates/canon/src/eval/
- confidence: inferred
  path: crates/canon/src/explain/
- confidence: inferred
  path: crates/canon/src/ir/
- confidence: inferred
  path: crates/canon/src/model/
- confidence: inferred
  path: crates/canon/src/validate/
- confidence: inferred
  path: ess/domains/protocol.yaml
- confidence: inferred
  path: fixtures/investigation/case-inputs.yaml
- confidence: inferred
  path: fixtures/investigation/invalid/
- confidence: inferred
  path: fixtures/investigation/semantic-diff/
- confidence: inferred
  path: website/data/
- confidence: inferred
  path: website/docs/reference/
- confidence: inferred
  path: website/static/schemas/
- confidence: inferred
  path: website/status.yaml
revision: 4
---
## Outcome

A protocol declares case inputs, a case supplies their values, and obligations and preconditions
depend on them, so one compiled protocol serves several profiles of one kind of work. This closes a
gap the implemented story:protocol-source-model (TASKBOARD C-001) and story:obligations (C-005)
left: `protocol/1` has no case inputs, and an obligation has no rule for when it applies
(`crates/canon/src/model/mod.rs:55-75`, `:122-125`). The engineering-protocols repository's
`story:software-change-profiles` names that gap and waits on it for its `risk` and `change_kind`
inputs.

1. **`case: inputs:` in `protocol/1`.** Optional, keyed by input id, where the design's sketch
   puts it (`docs/design/canon-protocol-calculus-design.md:632`). Each input declares a closed,
   non-empty list of `values` (identifiers) and an optional description. The validator refuses an
   empty list and a value listed twice. The sketch's `type: string` is not taken: a free-text
   input could select no rule.
2. **Values in the case.** `canon-case/1` gains `inputs`, a map from input id to one value. An
   evaluation is refused, naming the input, when the case omits a declared input, names an input
   the protocol does not declare, or gives a value outside the declared list (naming the value as
   well). Inputs are fixed when a case opens; they are configuration, not evidence, so no
   evaluation has an unknown input.
3. **The `input` predicate.** A new predicate leaf, `input: <id>` with `in: [<value>, …]`: TRUE
   when the case's value is in the list, FALSE otherwise. It is usable wherever a predicate is: in
   a claim's `true_when`, an action's `precondition`, and (4) below. The validator refuses an
   unknown input id and a value the input does not declare.
4. **Derived obligations.** An obligation gains an optional `applicable_when` predicate (the name
   the design gives the same guard on actions, `docs/design/canon-protocol-calculus-design.md:541`).
   FALSE: the obligation's status is `not_applicable`, and it holds back no outcome. TRUE: the
   obligation is `open` or `discharged` as today. UNKNOWN (a claim it reads is UNKNOWN): the
   obligation is `open`, with a reason naming what is unknown, because UNKNOWN is not FALSE.
5. **The classifier knows them.** `canon diff` (story:semantic-diff) classifies every change to
   these constructs, so `canon floor` sees them:
   - an obligation that gains an `applicable_when` is `RELAXATION`; one that loses it is
     `TIGHTENING`;
   - a value added to an input's `values` is `EXPANSION`;
   - any other change to an input declaration, an `input` leaf or an `applicable_when` (a value
     removed, an input added or removed, a changed `in` list, a changed guard) is `BREAKING`: the
     classifier does not order those changes, and `BREAKING` makes `canon floor` refuse them.

## Order

- depends_on story:semantic-diff: this story changes `canon-ir/1`'s types (the `case: inputs`
  section, the predicate leaf, `applicable_when`), and story:semantic-diff's classifier is written
  against those types; landing after it, this story extends the classifier with § Outcome 5
  instead of moving the types under it.
- story:protocol-imports depends on this story: both change `protocol/1` in `ess/`,
  `crates/canon/src/model/`, `crates/canon/src/validate/` and `crates/canon/src/ir/`. This story
  goes first because a downstream protocol is blocked on it today.
- Order recorded for https://github.com/beyond10x/canon/issues/5 and this story (2026-10-07):
  story:semantic-diff → story:case-inputs → story:protocol-imports → story:protocol-floor;
  story:ess-command-surface after story:semantic-diff and before story:protocol-imports;
  story:case-composition after story:protocol-imports.
- story:ess-command-surface also edits `ess/`, in files of its own (`ess/domains/cli.yaml`, new,
  and the domain list in `ess/system.yaml`), disjoint from `ess/domains/protocol.yaml`; the two may
  share a wave.
- Other draft stories that change `protocol/1` (story:capability-scope,
  story:evidence-independence, story:evidence-order-predicate, story:revision-ordering) have no edge
  to this one; `aep plan artifact waves` keeps each in a wave of its own by the shared directories.

## Scope

- In: § Outcome 1 to 5; the fixtures, scenario and test below; four revision pairs added to
  `fixtures/investigation/semantic-diff/`.
- Out: which inputs a protocol of a given kind declares (the engineering-protocols repository
  declares `risk` and `change_kind`); inputs whose value changes during a case; numeric or
  free-text inputs; a capability requirement that varies by input (`story:software-change-profiles`
  says profiles change authority requirements too; its acceptance tests obligations and
  preconditions only, and a precondition with an `input` leaf covers those).
- Surfaces: `ess/domains/protocol.yaml`, `crates/canon/src/model/`, `crates/canon/src/validate/`,
  `crates/canon/src/ir/`, `crates/canon/src/eval/` (`obligations.rs`, `case.rs`, the predicate
  evaluation), `crates/canon/src/explain/`, `crates/canon/src/diff/`, `crates/canon/src/check/`
  (`space.rs` matches on every `Predicate` variant and needs the `input` leaf),
  `crates/canon-cli/tests/case_inputs.rs`, `fixtures/investigation/case-inputs.yaml`,
  `fixtures/investigation/invalid/`, `fixtures/investigation/semantic-diff/`,
  `conformance/scenarios/case-inputs.yaml`, `website/docs/reference/`, `website/data/` and
  `website/static/schemas/` (regenerated by `task docs-generate`), `website/status.yaml`.

## ESS first

- Specification change, first commit: in `ess/domains/protocol.yaml`, the optional `case` field on
  `canon.protocol.Protocol` holding `inputs`, keyed by input id, of a new type
  `canon.protocol.CaseInput` (values, description); the `input` predicate variant; the optional
  `applicable_when` on the obligation type; the `inputs` field on the `canon.protocol.Case` entity
  (`canon-case/1`); and `not_applicable` in the obligation status of `canon-decision/1`. With the
  fixtures, the scenario file `conformance/scenarios/case-inputs.yaml` and the named test below.
- Red on that commit: `crates/canon/tests/ess_model_matches.rs` fails naming `inputs`,
  `applicable_when` and `not_applicable`, because the Rust model has none of them;
  `case_inputs_derive_obligations` fails, because `deny_unknown_fields` refuses `case`.

## Acceptance

The named test `case_inputs_derive_obligations` (in `crates/canon-cli/tests/case_inputs.rs`)
passes. Its fixture `fixtures/investigation/case-inputs.yaml` declares the input `severity` (`low`,
`high`); the obligation `peer.review` with `applicable_when` severity in `[high]`; the obligation
`falsification.recorded` with `applicable_when` the claim `explanation.supported`; the outcome
`finding.published`, which requires every applicable obligation discharged; and the action
`publish_finding`, whose precondition is TRUE when severity is `low` or the claim
`explanation.supported` is TRUE. Thirteen expectations:

1. `canon conform run` reports `CANON-INPUT-001` (`conformance/scenarios/case-inputs.yaml`) passed,
   with steps for expectations 2 to 6;
2. severity `low`: `peer.review` is `not_applicable`, and `finding.published` is not held back by it;
3. severity `high`, no evidence: `peer.review` is `open`;
4. severity `high`, with the discharging evidence: `peer.review` is `discharged`;
5. severity `low`, no evidence: `publish_finding` is admissible; severity `high`, no evidence: it is
   blocked naming `explanation.supported`;
6. no evidence: `falsification.recorded` is `open`, its reason naming `explanation.supported`;
7. `canon evaluate` on a case without `severity` exits 1 and its message names `severity`;
8. on a case with severity `medium`, exits 1 and its message names `severity` and `medium`;
9. on a case naming an input `urgency` the protocol does not declare, exits 1 naming `urgency`;
10. `canon validate` on `fixtures/investigation/invalid/input-empty-values.yaml` exits 1 naming
    the input;
11. on `fixtures/investigation/invalid/input-duplicate-value.yaml`, exits 1 naming the value;
12. on `fixtures/investigation/invalid/input-undeclared.yaml` (an `input` leaf naming an input that
    is not declared) and `input-undeclared-value.yaml` (an `in` list naming a value the input does
    not declare), each exits 1 naming the input or the value;
13. `canon diff` over the four revision pairs of § Outcome 5 added to
    `fixtures/investigation/semantic-diff/` prints `RELAXATION`, `TIGHTENING`, `EXPANSION` and
    `BREAKING` respectively.

`crates/canon/tests/ess_model_matches.rs` passes.

## Source

TASKBOARD C-001 and C-005 (Atlas `docs/design/governed-autonomy/TASKBOARD.md`), as the gap the
stories implementing them left; `docs/design/canon-protocol-calculus-design.md` (`applicable_when`
at line 541, `case: inputs:` at line 632); the engineering-protocols repository's
`story:software-change-profiles` § Canon capability; Atlas ADR 0080 (spec first, then red).
