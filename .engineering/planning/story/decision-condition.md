---
format: aep.planning-md/3
id: story:decision-condition
kind: story
status: draft
title: Let an outcome's explicit decision carry an optional `when` predicate, so the decision counts only while the predicate is true
tags:
- from-request
refs:
- provider: sdc
  reference: request-17-decision-condition
relations:
- decomposes: epic:canon-kernel
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:decision-outcomes
- depends_on: story:pending-decisions
- depends_on: story:decision-grounds
scope:
- confidence: cited
  path: conformance/requirements.yaml
- confidence: inferred
  path: conformance/scenarios/decision-condition.yaml
- confidence: inferred
  path: crates/canon-cli/tests/decision_condition.rs
- confidence: cited
  path: crates/canon-docs/src/landing.rs
- confidence: cited
  path: crates/canon-docs/src/pages.rs
- confidence: cited
  path: crates/canon/src/check/space.rs
- confidence: cited
  path: crates/canon/src/check/tests.rs
- confidence: cited
  path: crates/canon/src/diff/mod.rs
- confidence: cited
  path: crates/canon/src/eval/depth.rs
- confidence: cited
  path: crates/canon/src/eval/outcomes.rs
- confidence: cited
  path: crates/canon/src/eval/read.rs
- confidence: cited
  path: crates/canon/src/ir/mod.rs
- confidence: inferred
  path: crates/canon/src/model/mod.rs
- confidence: cited
  path: crates/canon/src/model/requirement.rs
- confidence: inferred
  path: crates/canon/src/model/write.rs
- confidence: cited
  path: crates/canon/src/validate/mod.rs
- confidence: cited
  path: ess/domains/protocol.yaml
- confidence: inferred
  path: fixtures/investigation/decision-condition.yaml
- confidence: inferred
  path: fixtures/investigation/decision-condition/
revision: 20
---
## Outcome

An outcome that requires an explicit decision may also carry an optional predicate, `when`. The decision counts toward the outcome only while `when` is TRUE.

```yaml
outcomes:
  withdrawn:
    requires:
      decision: withdraw
      when: {claim: publication.refused}
```

- **Model.** `canon.protocol.OutcomeRequirement`'s `decision` variant holds a new struct, `canon.protocol.DecisionRequirement {decision: DecisionName, when: Optional<Predicate>}`.
  - In Rust this is `OutcomeRequirement::Decision(DecisionRequirement)`. `decision()` still returns the name, and a new `when()` returns the predicate.
  - Written without `when`, a requirement parses, compiles and evaluates to the same bytes as today.
- **IR (`canon-ir/1`).** The IR writes `"when"` only when it is present, normalized as `requires` is (`ir/mod.rs:349-353`). So the IR of every existing protocol is byte-identical. `read_ir` reads `{"decision", "when"}` back (`eval/read.rs:426-435`).
- **Evaluation (`outcomes` section, CANON-OUTCOME-003).**

| `when` | a decision applies at the case revision | status | `reasons` | `pending_decisions.decisions` entry |
|---|---|---|---|---|
| TRUE | yes | `legitimate`, with `decided_by` | none | none |
| TRUE | no | `blocked` | `[{"decision": <name>, "present": false}]` | yes, as today |
| FALSE | either | `blocked` | the reasons a precondition that is not `true` gives (`actions.rs`), e.g. `{"claim": c, "value": "false"}` | none |
| UNKNOWN | either | `blocked` | the same, e.g. `{"claim": c, "value": "unknown"}` | none |

- **A decision supplied while `when` is FALSE or UNKNOWN** is neither refused nor counted.
  - The outcome's reasons name only `when`.
  - `explanation.computed_from.decisions` lists the decision as given. That listing exists today (`explain/mod.rs:18-19`).
  - The explanation's `outcomes.<id>.because` holds the `when` reasons. Every claim they name is listed under `claims`, down to the evidence records, by the existing rule (`explain/mod.rs:28-30`).
  - No new status, no new reason value and no new key.
- **Terminating through such an outcome while it is blocked** is refused as `illegitimate-termination`, as it is today.
- **`undeclared-decision` is unchanged.** It depends only on the decision name, never on `when`.
- **Refusals.**
  - `when` written without `decision` (beside a predicate requirement) is refused: `error[parse]`, exit 1.
  - A `decision` inside `when` is refused at parse as `unknown field `decision``. This is today's behaviour for a decision inside `all`; I reproduced it with the binary.
  - Whatever forms `when` may hold (§ Decision needed, D4), its references are resolved by validation step 9, under outcomes. Its nesting is checked by `predicate-too-deep`, naming the outcome and the field `when`.
- **`canon check`** gets no new kind of dimension. An outcome's slice is the union of the decision dimension and `when`'s predicate slice (`check/space.rs:671-686`). Without that, the check would decide the outcome over a slice that does not hold `when`'s claims.
- **`canon simulate`.**
  - The decision step is unchanged: it is offered wherever the decision is not taken (D3).
  - Each walk state is evaluated by the evaluator, so `when` applies there.
  - R4's ending `; decision `d` taken makes outcome `o` legitimate` appears only in states where `when` is TRUE.
  - Terminable counts per outcome drop to the states where both the decision and `when` hold.
- **`canon diff`.** Adding, removing or changing `when` prints `BREAKING: outcome `o` requirement changed`, under the existing rule (`diff/mod.rs:52`, `:825-826`).
  - Claims tested in `when` count as read positively when diff decides the direction of a claim change (`diff/mod.rs:614-618`).

## Why

- **The request.** The session working on the SDC prototype wrote: "Canon lets an outcome require either a condition or a decision, not both, so 'this decision only after the publication was refused' cannot be said in the protocol and SDC's tools enforce it." It asks for an optional `when: <predicate>` on the decision variant. It is a draft and is not accepted now.
- **This is by design today. Verified in the tree at `cf4badbb`:**
  - `model/requirement.rs:13-14`: a decision is "the whole of an outcome's `requires`".
  - `model/requirement.rs:178-180` refuses both together. The binary prints `error[parse]: outcomes.withdrawn: a requirement is either `decision` alone or a predicate, not both`. A `when` key is refused as `unknown field `when``.
  - story:decision-outcomes, coordinator notes: "a decision requirement is its own variant of an outcome requirement, standing alone"; "an outcome cannot require both a decision and a claim (no-op, as designed)".
  - `ess/domains/protocol.yaml:339-355`: "A decision is a requirement on its own".
- **How evaluation works today:**
  - A decision applies only when its name, outcome and `case_revision` equal the snapshot's revision (`eval/decisions.rs:28-33`, `:57-79`).
  - A decision outcome is legitimate exactly when one applies (`eval/outcomes.rs:113-125`). Otherwise it is blocked with `{decision, present: false}` (`:147-152`).
  - `pending_decisions.decisions` takes only a reason with exactly the keys `decision` and `present: false` (`eval/pending.rs:112-121`). The section holds only entries "that would change a status" (story:pending-decisions § Outcome).
  - Probe output: `withdrawn` is blocked, and `pending_decisions.decisions` = `[{case_revision: c1, decision: withdraw, options: [taken], outcome: withdrawn}]`.
- **Precedent for gating:**
  - An action whose precondition is not `true` is `blocked` by the precondition's reasons. "Authority is not consulted" (`eval/actions.rs:7`, `:40`), and pending lists no entry for it (`eval/pending.rs:14-15`).
  - The table above applies the same rule to a decision.
  - The one existing guard predicate is `applicable_when`: claims and case inputs only, refused otherwise as `evidence-in-applicability` (`validate/mod.rs:60-62`, `eval/obligations.rs:8-15`). Unlike `when`, it treats UNKNOWN as "applies".
- **The workaround and what it loses:** an action with a precondition and a capability that `may_produce` a record marked `decided_by: judgement` (story:judgement-marking, `ess/domains/protocol.yaml:322`), with the outcome requiring a claim and the evidence.
  - It loses the `canon-decisions/1` principal (`decided_by.principals`).
  - It loses the binding to the case revision.
  - It loses the R4 ending: `canon simulate` names decision steps only (`simulate/mod.rs:980-1011`).
  - I read this; I did not run it.
- **"Only after" cannot be enforced.** A decision entry has no time, only `case_revision` (`ess/domains/protocol.yaml:984-1010`). So Canon cannot tell a decision taken before `when` turned TRUE from one taken after, at the same case revision.
  - What this story gives: "counts only while `when` is TRUE".
  - Real ordering comes from the caller moving the case revision when evidence arrives (who assigns case revisions is open: story:decision-outcomes, "Open"), or from story:decision-grounds's `ground_evidence` naming the refusal record.
- **Revision binding.** When the record that made `when` TRUE is excluded later (`revision_mismatch`, `expired`, `invalidated`), the claim goes UNKNOWN and the outcome is blocked by `when`.
  - The supplied decision stays in the input, uncounted.
  - If `when` becomes TRUE again at the same case revision, the decision counts again.
  - A terminated case re-evaluated then is refused as `illegitimate-termination`. That is today's rule for any outcome (story:decision-outcomes, terminated-case-reevaluation, option A).
- **`canon simulate` today** (probe `today.yaml`): `stuck-state: state {evidence `publication` result `refused`}: … ; decision `withdraw` taken makes outcome `withdrawn` legitimate`; summary `8 reachable states, 5 terminable (outcome `published` 2, outcome `withdrawn` 4)`.
- **Not verified:**
  - Nothing was built.
  - The binary is `canon 0.9.0`, built 2026-11-21 06:51, and may be older than HEAD.
  - The simulate counts in § Acceptance are derived from R3 to R5, not run.
  - That `ess` 0.77.0 accepts a struct payload in this union: `Predicate` already has one (`protocol.yaml:567-579`).

## Decision needed

| | Choice | What it changes | Cost |
|---|---|---|---|
| D1 A | A decision supplied while `when` is not TRUE is ignored (not counted, not refused) | Nothing beyond the table | The caller must read `computed_from.decisions` to see that it gave one |
| D1 B | Refused with a new code (e.g. `inapplicable-decision`) | New refusal in `eval/outcomes.rs` | Refuses on UNKNOWN (UNKNOWN is not FALSE). The same inputs give a refusal at a later `--at` when a record expires |
| D2 A | When `when` is not TRUE, the reasons name only `when`; no pending entry | Nothing in `eval/pending.rs` | The decision the outcome will later need is not named until `when` is TRUE |
| D2 B | Name `when`'s reasons and `{decision, present: false}` | The pending filter must then skip it, or CANON-PENDING-001 ("would change a status") breaks | Edits `eval/pending.rs` and its requirement |
| D3 A | Decision step offered in every state (as today) | No change to `simulate/mod.rs` steps | States hold decisions taken before `when` was TRUE, which the evaluator accepts |
| D3 B | Decision step offered only where some outcome requiring it has `when` TRUE | `simulate/mod.rs` step loop (`:791-797`) | The walk diverges from the evaluator; shares a file with three ordered simulate stories |
| D4 A | `when` is any predicate, as `requires` is | Readers must include `when`: `check/space.rs:2069-2088`, `simulate/mod.rs:1090-1140` (order forms, `children`) | Adds `simulate/mod.rs` to the scope |
| D4 B | Claims and case inputs only, as `applicable_when` is; an evidence match, order form or `children` form refused as a new `evidence-in-decision-condition`, naming the outcome | New problem in `validate/mod.rs` | The author declares a claim for the condition |

`canon diff` could class adding `when` as `TIGHTENING`. That is not offered here: every other requirement change is `BREAKING` today (`diff/mod.rs:52`).

Recommendations:
- **D1 A:** a refusal would treat UNKNOWN as a contradiction.
- **D2 A:** it is the rule for actions and keeps CANON-PENDING-001 true unchanged.
- **D3 A:** the walk evaluates the evaluator's inputs and gets the R4 ending right without a code change.
- **D4 B:** it reuses the one existing guard form and keeps `simulate/mod.rs` and the check's evidence readers out of scope.

## ESS first

- **First commit (specification only):**
  - In `ess/domains/protocol.yaml`, the `decision` variant of `canon.protocol.OutcomeRequirement` becomes `canon.protocol.DecisionRequirement`.
  - The new struct is declared with `decision: canon.protocol.DecisionName` and `when: Optional<canon.protocol.Predicate>`. Its comment states the table and `# read from:` this story.
  - `CANON-OUTCOME-003` is added to `conformance/requirements.yaml`. The id is free: there is no `CANON-OUTCOME-003` in the tree.
  - New `fixtures/investigation/decision-condition.yaml` and `conformance/scenarios/decision-condition.yaml`.
- **Red on that commit:**
  - `crates/canon/tests/ess_model_matches.rs::specification_and_model_are_equal` names `OutcomeRequirement`'s `decision` variant and `DecisionRequirement`.
  - `canon conform run --requirements conformance/requirements.yaml` fails on `CANON-OUTCOME-003`, because the fixture's `when` is refused: `unknown field `when``.

## Scope

- **In:**
  - The struct and variant, the reader, the IR write and read-back, and the evaluation table — cited
  - Depth and reference checks on `when` — cited
  - The check's outcome slice — cited
  - The diff reader direction — cited
  - The D4 B refusal — inferred
- **Out:**
  - Ordering a decision after evidence (needs a decision instant, or grounds: story:decision-grounds) — cited
  - A finer `canon diff` class — cited
  - `when` on a predicate requirement — cited
  - Any change to `canon-decisions/1` — cited
- **Surfaces:**
  - `ess/domains/protocol.yaml` — cited
  - `crates/canon/src/model/requirement.rs` — cited
  - `crates/canon/src/model/mod.rs` (re-export) — inferred
  - `crates/canon/src/ir/mod.rs` (`:349-353`, `:852-859`) — cited
  - `crates/canon/src/eval/read.rs` (`:426-435`) — cited
  - `crates/canon/src/eval/outcomes.rs` (`:96`, `:113-153`) — cited
  - `crates/canon/src/eval/depth.rs` (`:66-74`) — cited
  - `crates/canon/src/validate/mod.rs` (`:805`, `:966`) — cited
  - `crates/canon/src/check/space.rs` (`:487-492`, `:671-686`) — cited
  - `crates/canon/src/check/tests.rs` (`:2664` builds the variant) — cited
  - `crates/canon/src/diff/mod.rs` (`:614-618`) — cited
  - `crates/canon-docs/src/pages.rs` (`:1478`) — cited
  - `crates/canon-docs/src/landing.rs` (`:314`) — cited
  - `conformance/requirements.yaml` — cited
  - `conformance/scenarios/decision-condition.yaml` — inferred
  - `fixtures/investigation/decision-condition.yaml` — inferred
  - `fixtures/investigation/decision-condition/` (CLI fixtures) — inferred
  - `crates/canon-cli/tests/decision_condition.rs` — inferred
  - `crates/canon/src/model/write.rs` (only if story:protocol-writer lands first) — inferred
- **Not touched under the recommendations:** `crates/canon/src/eval/pending.rs`, `crates/canon/src/simulate/mod.rs`, `crates/canon/src/explain/mod.rs`, `ess/domains/cli.yaml`.
- **The coordinator's at integration:** regenerated `website/docs/reference/`, `website/static/schemas/`, `website/status.yaml`.
- **Confidence:** high for the model, IR, evaluation and diff. Medium for the check slice; `predicate_slice` was not traced through claim references.

## Order

- **Depends on** story:decision-outcomes and story:pending-decisions (both implemented).
- **story:decision-grounds** (active, wave 6) shares `eval/outcomes.rs`, `ess/domains/protocol.yaml` (split by declaration) and `conformance/requirements.yaml`.
  - It adds `stale_grounds` to the `{decision, present: false}` reason.
  - This story lands after it and leaves that reason as decision-grounds leaves it.
  - While `when` is not TRUE, no decision reason and so no `stale_grounds` is written. That is the actions rule decision-grounds already follows for capability reasons.
  - Note: decision-grounds puts a third key on that reason, and `eval/pending.rs:120` takes only a reason with exactly two keys. That is decision-grounds' to settle.
- **story:simulate-record-groups** (wave 7) shares `diff/mod.rs`, `eval/read.rs`, `ir/mod.rs`, `model/mod.rs`, `validate/mod.rs` and `ess/domains/protocol.yaml`.
- **story:simulate-revise-step** (wave 8) shares `check/space.rs`. story:simulate-child-outcome-step (wave 6) shares no file under D3 A and D4 B. Under D4 A or D3 B, all three share `simulate/mod.rs`, which § Replan 9 orders one after another.
- **story:protocol-writer** (wave 7) destructures every model type. Landing after it adds `when` to `model/write.rs`.
- **story:revision-ordering** (wave 9) shares `depth.rs`, `diff/mod.rs`, `ir/mod.rs`, `eval/read.rs`, `validate/mod.rs`, `check/space.rs`, and `canon-docs` `pages.rs` and `landing.rs`.
- **story:child-outcome-subject** and **story:child-outcome-recovery** share `check/space.rs`.
- **Earliest wave:** after wave 8. In wave 9 it meets story:revision-ordering.

## Acceptance

The named test `crates/canon-cli/tests/decision_condition.rs::a_decision_counts_only_while_its_condition_is_true` passes. It covers fixture `decision-condition.yaml`, plus fixtures under `fixtures/investigation/decision-condition/` for steps 7 and 8. The protocol has:
- artifact `a`;
- evidence kind `publication`;
- claims `refused` (a `publication` record with result `refused`) and `accepted`;
- action `submit`, whose precondition is both claims `unknown` and which may produce `publication`;
- outcome `published`, requiring `accepted`;
- outcome `withdrawn`, requiring `{decision: withdraw, when: {claim: refused}}`.

The case is at `revision: c1`.

1. `canon conform run --requirements conformance/requirements.yaml` passes `CANON-OUTCOME-003` and every existing scenario, unchanged.
2. No evidence and no decision: `withdrawn` is `blocked` with `[{"claim": "refused", "value": "unknown"}]`, and `pending_decisions` has no `decisions` list.
3. The same case with `withdraw` for `withdrawn` at `c1`: the same status and reasons, and `computed_from.decisions` lists the entry.
4. A `refused` record and no decision: `blocked` with `[{"decision": "withdraw", "present": false}]`, and pending holds `{outcome: withdrawn, decision: withdraw, case_revision: c1, options: [taken]}`.
5. A `refused` record and the decision at `c1`: `legitimate` with `decided_by: {decision: withdraw, principals: [p]}`. At `c0` it is blocked as in 4.
6. The `refused` record with `subject_revision` not current, plus the decision at `c1`: blocked as in 2. With `termination: withdrawn` the case is refused as `illegitimate-termination`.
7. Validation and compiling:
   - `when` beside `claim:` is refused, `error[parse]`, exit 1.
   - `when: {all: [{decision: withdraw}]}` is refused as `unknown field `decision``.
   - Under D4 B, `when: {evidence: {kind: publication}}` is refused as `evidence-in-decision-condition`, naming `withdrawn`.
   - Every existing fixture compiles to the same bytes.
8. Simulate and diff:
   - `canon simulate` prints the `{evidence `publication` result `refused`}` stuck state with the R4 ending.
   - It prints the `{evidence `publication`}` stuck state without one.
   - The summary gives `outcome `withdrawn` 1`.
   - `canon diff` from the protocol without `when` prints `BREAKING: outcome `withdrawn` requirement changed`.

## Source

- **The request:** relayed from the SDC session on 2026-11-21, with an earlier drafter's notes. Each point was checked again here.
- **Tree:** `canon-wave-2026-11-20-w1`, branch `wave/2026-11-20-w6`, `cf4badbb`.
- **Files read:**
  - `AGENTS.md`, `docs/plan/wave-plan-2026-11-20.md`
  - `crates/canon/src/model/requirement.rs`
  - `crates/canon/src/eval/{outcomes,decisions,pending,actions,obligations,depth,read}.rs`
  - `crates/canon/src/ir/mod.rs`, `crates/canon/src/validate/mod.rs`
  - `crates/canon/src/check/space.rs`, `crates/canon/src/check/mod.rs`
  - `crates/canon/src/simulate/mod.rs`, `crates/canon/src/diff/mod.rs`, `crates/canon/src/explain/mod.rs`
  - `crates/canon/tests/ess_model_matches.rs`
  - `ess/domains/protocol.yaml`, `conformance/requirements.yaml`, `conformance/scenarios/decision-outcomes.yaml`
- **Stories read:** decision-outcomes, pending-decisions, judgement-marking, canon-simulate (to the end, R1 to R7 and R2a), decision-grounds. Typed scopes of the stories under § Order.
- **Commands:**
  - `aep plan artifact show story:<slug>`
  - `target/debug/canon validate`, `compile`, `diff`, `simulate`, `check` and `evaluate` on probe protocols in the scratch directory: `both.yaml`, `when.yaml`, `nested.yaml`, `today.yaml`
- **Date:** 2026-11-21.
