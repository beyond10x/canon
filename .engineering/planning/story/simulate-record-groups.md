---
format: aep.planning-md/3
id: story:simulate-record-groups
kind: story
status: active
title: Walk one run of an action that records several kinds as one step with records_together in canon simulate
tags:
- from-request
refs:
- provider: sdc
  reference: request-15
relations:
- decomposes: epic:protocol-analysis
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:canon-simulate
- depends_on: story:judgement-marking
scope:
- confidence: inferred
  path: crates/canon-cli/tests/simulate_record_groups.rs
- confidence: inferred
  path: crates/canon-docs/src/source.rs
- confidence: cited
  path: crates/canon/src/diff/mod.rs
- confidence: cited
  path: crates/canon/src/eval/read.rs
- confidence: cited
  path: crates/canon/src/ir/mod.rs
- confidence: cited
  path: crates/canon/src/model/mod.rs
- confidence: inferred
  path: crates/canon/src/simulate/mod.rs
- confidence: cited
  path: crates/canon/src/validate/mod.rs
- confidence: cited
  path: ess/domains/protocol.yaml
- confidence: inferred
  path: fixtures/investigation/record-group/
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T04:43:47Z", actor: "human:timo", revision: 13, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w5"}
- {from: "proposed", to: "active", at: "2026-11-21T04:43:47Z", actor: "human:timo", revision: 14, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w5"}
---
## Outcome

A protocol can declare which kinds one run of an action records together, and `canon simulate` walks that run as one step.

1. **`records_together` on an action in `protocol/1`.**
   - It is an optional list of groups. Each group is `{kinds: [<kind id>, ...]}`.
   - `canon validate` refuses a group with fewer than two distinct kinds, a kind not in the action's `may_produce`, or a kind in two groups of one action. The code is `invalid-record-group`, and it names the action.
   - Absent means no group.
   - The declaration says that one run records the group's kinds all together or none of them. Canon gives it no evaluation meaning: `canon evaluate`, `canon check` and `canon generate` give the same bytes with or without it.
2. **`canon-ir/1`.**
   - An action writes `"records_together"` only when the list is non-empty. Groups are sorted, and so are the kinds within each group.
   - The IR of every existing protocol stays byte-identical, and the IR read-back reads the field.
   - `canon diff` reports adding, removing or changing a group as `NO SEMANTIC CHANGE`, as it does for `supplied_by`.
3. **Action step of `canon simulate`.** An admissible action offers two sorts of step.
   - One record of one kind that is in no group, as today.
   - For each group, one record of each member kind, added in a single step. Each member takes one dimension, one class and one producer profile.
   - A kind in a group is never offered alone. A group step has the product of its members' successor counts, one successor per combination.
   - The `case_outcome` kind a `case_open` action produces is not groupable. It stays a single-kind step.
4. **Order and print.**
   - Within an action, single-kind steps come first in today's order. Group steps follow, in order of each group's least kind id.
   - Within a group, the combinations are the product of the members' dimension, class and profile, with the members in kind-id order and the last member varying fastest.
   - A path prints one group step as one line: `` action `a` produces evidence `k1` ... and evidence `k2` ... ``. The members appear in kind-id order, joined by ` and `.
   - Breadth-first search counts a group step as one step. It still reaches each state first along a shortest path.
5. **Nothing else changes.** The summary line has the same format. `ess/domains/cli.yaml` is untouched, because no command or flag changes. A protocol that declares no group gets byte-identical `canon simulate` output.

**Why both rules hold.** Every finding the walk prints holds for a runner that records each group together or not at all. That is what the declaration states, in the same way `independent_on` states a fact Canon cannot check. A state holding one kind of a group is not a state of such a case, so omitting it hides no real dead end. A runner that records a group's kinds one at a time is outside the declaration and the walk cannot see it.

## Why

**The request (request 15, `sdc` session, 2026-11-21).** `questions.check` records two kinds in one run. A guard that requires both kinds to have no record makes `passed` unreachable, because a walk step adds one record. The request counts 88 of intake's 177 as caused by this.

**The action.**
- In `$HOME/.local/state/worktree/trees/b10x/sdc/sdc-roadmap-read-precondition/protocol/sdc-intake.protocol.yaml:200-212` (head `f364b6b`), `questions.check` has the guard `any: [claim questions.checked is unknown, claim interview.checked is unknown]`.
- Its `may_produce` is `[{evidence: questions_shape}, {evidence: questions_settled}]`.
- It is the only action with more than one kind among all eight SDC protocols. I compiled all eight with `canon-sim compile` to check.

**What the output shows** (`$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w5/sdc-run/sdc-intake.txt`).
- The last line reads `35 findings (177 stuck or cycle states)`.
- All 35 printed paths contain two `questions.check` steps in a row, one per kind, because a step adds one record. Under a group step each path is one step shorter.
- None of the 88 states is a printed line. R5 prints only the smallest state, and the other states appear as the `N other ... states contain it` counts at the end of those lines.
- The 88 are states that hold two records of one kind with different results, the second one of the class "another result". There are 44 per kind: 20+24 with a doubled `questions_shape` and 20+24 with a doubled `questions_settled`.
- The guard `any` allows the second run, because the other kind is still unknown.
- Under the guard `all` (both unknown) the doubles are unreachable, but so is `passed`.

**Verified by running the copied binary** (probe files are in `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/requests/15/work/`).
- With the guard changed to `all`, `intake-both.protocol.yaml` gives `405 reachable states, 0 terminable, 42 findings (88 stuck or cycle states)`, with `unreached-outcome` and `no-outcome-reachable`. That confirms the request.
- The probe protocol `pair.protocol.yaml` is two kinds, one action and the guard `all` (both unknown). It gives `3 reachable states, 0 terminable, 4 findings (2 stuck or cycle states)`.

**Not verified.**
- No binary implements option A or B. Their numbers come from `emulate.py`, a Python walk I wrote over the compiled IR.
- It reproduces the binary exactly for today's walk: intake 1215/3/35/177, plan 85/1/17/17, spec 120/61/16/16, and the `all` variant 405/0/40 plus 2.
- Walk time at the 65,536 bound is not measured. One run of the binary on intake (1,215 states) took 1.2 s wall.

**How the walk enumerates successors today** (`crates/canon/src/simulate/mod.rs` at `4b36d52`).
- `action_records` (`:579-583`) lists, for each action, every dimension, class and profile of each kind it produces.
- The step loop (`:637-654`) adds one such triple per admissible action. A successor equal to the state is dropped.
- The bound is checked at `:551-556` for initial states and at `:708-709` for each new state.

## Decision needed

| Option | `ess/`, model, IR | Walk | Cost |
|---|---|---|---|
| A. `records_together` (this story) | `protocol.yaml`: type `RecordGroup`, field `Action.records_together`. Model, validate, IR write and read-back, diff, docs. | One step per group, and a grouped kind is not offered alone. | A language change. It shares the `Action` struct with story:judgement-marking. |
| B. No language change | None. | An action step adds one record of each kind of any non-empty subset of `may_produce`. | Successors per action grow as shown below. Partial subsets stay reachable. |
| C. No language change | None. | An action step adds one record of every kind of `may_produce`. | Partial records are unreachable for every action, declared or not. A runner that records one kind and stops is not reported. |

**Successors of one action step.** Today an action has the sum of n_k successors. Under B it has the product of (1+n_k), minus 1. Here n_k = dimensions x classes x profiles of kind k. Every SDC kind has 1 dimension, because each match names its artifact. Each has 1 profile, because `spec_review` has one demand and the others none. Classes are the results read plus "another result", so 2, except `spec_source` with 1. These are maxima, in a state with no record of the action's kinds.

| Protocol | Actions | Kinds in `may_produce` | Today | Under B |
|---|---|---|---|---|
| sdc-intake | `questions.check` | `questions_settled`, `questions_shape` | 4 | 8 |
| sdc-intake | `dissection.check`, `idea.check`, `requirements.check` | one each | 2 | 2 |
| sdc-intake | `amend.upstream`, `idea.dissect`, `questions.answer`, `questions.write`, `requirements.normalise`, `scope.answer` | none | 0 | 0 |
| sdc-plan | `plan.cover`, `plan.scope`, `plan.status`, `plan.validate`, `roadmap.read`, `spec.gate` | one each | 2 | 2 |
| sdc-plan | `amend.upstream`, `plan.decompose` | none | 0 | 0 |
| sdc-spec | `roadmap.read`, `spec.compile`, `spec.review`, `spec.scan`, `spec.synthesize`, `spec.trace`, `spec.validate` | one each | 2 | 2 |
| sdc-spec | `spec.author`, `spec.reconstruct`, `spec.reuse` | `spec_source` | 1 | 1 |
| sdc-spec | `amend.upstream` | none | 0 | 0 |

Over all admissible actions, the largest successor count in one state is 12 today and 16 under B on intake, 8 and 8 on plan, and 7 and 7 on spec. Beyond SDC, an action with m kinds of n_k = 2 has 3^m - 1 successors under B: 8 for m=2, 80 for m=4. At the bound of 65,536 states, intake's 16 means at most about 1.05 million constructed successors.

**Reachable states.**
- Every single-kind step is also a B step, so B reaches every state the walk reaches today and may reach more.
- A state B reaches in one step is reachable today in several steps when no guard blocks the intermediate state. A guard that does block it makes the state new, and the reachable set grows.
- A, by contrast, drops the single-kind steps of grouped kinds, so its reachable set can shrink.
- The bound is unchanged. `check::STATE_BOUND` (65,536) counts visited states (R1), so the walk is finite. It returns `reachable-state-bound` when a new state would pass the bound, and never reports a partial walk.

**sdc-intake under each option** (rows 1 and 4 run on the binary, the others on `emulate.py`):

| Guard | Step | Reachable | `passed` reachable | Stuck or cycle findings | States they stand for |
|---|---|---|---|---|---|
| any | today | 1215 | yes | 35 | 177 |
| any | B | 1215 | yes | 35 | 177 |
| any | A | 405 | yes | 35 | 89 |
| all | today | 405 | no | 40 | 88 |
| all | B | 729 | yes | 41 | 177 |
| all | A | 405 | yes | 35 | 89 |

**What B removes.**
- With the guard `any`, B changes nothing. Both kinds in one step were already reachable in two steps.
- With the guard `all`, B makes `passed` reachable. It replaces the 88 double-record states by 88 states with one questions kind only, and it adds a printed stuck state (41 findings instead of 35).
- Of the 177 states under B and `all`, 88 hold one kind only and 89 hold both once. Option A leaves 89.

**Recommendation: A**, because the measured successor growth is small enough for B (at most 8 per action, 16 per state), but B still leaves 88 stuck states that the request asked to remove. Only a declaration lets the walk leave out partial records without hiding a runner that really stops after one kind.

**How option B would interact with R1 to R6.** The `sdc` session may still choose it.
- Single-kind steps come first. Larger subsets follow by size, then by kind-id order, and their combinations are ordered as in Outcome 4. Paths print as in Outcome 4.
- R3: a step returns the same state exactly when each record it adds is present. So "every step returns the same state" holds as today.
- R5 is unchanged, because containment is by records.
- The single-kind steps in B keep every `cycle` and `stuck-state` that exists today for protocols with one kind per action.

**How A interacts with the rules.**
- R1: the bound counts visited states only.
- R2: expiry steps stay single-record and are tried after all action and supplied steps. A group step has no clock, because order forms are refused (`ordered-evidence`).
- R3 to R6: group steps are action steps, so the open-state computation, the one finding per smallest state and the decision exits are unchanged.
- Supplied kinds stay single-record steps of their own.
- `decided_by` from story:judgement-marking is read by neither the group rule nor the step, and the path does not print it. A group may mix program-decided and judgement-decided kinds.

## ESS first

- **First commit** changes only `ess/domains/protocol.yaml` (with `# read from:` comments) and the fixtures and test below.
  - It adds the type `canon.protocol.RecordGroup {kinds: List<EvidenceKindId>}`. A named type is used because I could not establish whether ESS 0.77.0 accepts a list of lists.
  - It adds the field `canon.protocol.Action.records_together: List<RecordGroup>` beside `may_produce` (`:181`).
  - It adds `fixtures/investigation/record-group/` (the `pair` probe with and without the field, and a refused group for each of the three refusal rules) and the test `simulate_records_a_group_in_one_step` in `crates/canon-cli/tests/simulate_record_groups.rs`.
- **Red on that commit:**
  - `crates/canon/tests/ess_model_matches.rs::specification_and_model_are_equal` fails, naming `RecordGroup` and `Action.records_together`.
  - `simulate_records_a_group_in_one_step` fails. `Action` is `deny_unknown_fields` (`crates/canon/src/model/mod.rs:227`), so `canon simulate` rejects the grouped protocol and exits 1 where the test expects 0.

## Scope

- **In:** the `records_together` field in the model, ESS, validator, IR write, IR read-back and diff, and the group step in the walk. (cited)
- **Out:** a revise step (a later story, listed in story:judgement-marking § Order). Order forms. Grouping `case_outcome`. Changing any SDC protocol. A partial walk beyond the bound. (cited)
- **Surfaces:**
  - `ess/domains/protocol.yaml` (cited)
  - `crates/canon/src/model/mod.rs` (`Action` at `:228`) (cited)
  - `crates/canon/src/validate/mod.rs` (beside `:932`) (cited)
  - `crates/canon/src/ir/mod.rs` (`Action.may_produce` at `:210`, compile at `:307`, write at `:704`) (cited)
  - `crates/canon/src/eval/read.rs` (`:349-362`) (cited)
  - `crates/canon/src/diff/mod.rs` (`:1005-1056`) (cited)
  - `crates/canon/src/simulate/mod.rs` (`Step::Record`, `action_records`, the step loop and the renderer) (inferred)
  - `crates/canon-docs/src/source.rs` (inferred)
  - `crates/canon-cli/tests/simulate_record_groups.rs` (new) (inferred)
  - `fixtures/investigation/record-group/` (new) (inferred)
- **Confidence:** medium. The step relation and the numbers are read and run. The validator and the IR sites are read in the first tree but not exercised, and `check/space.rs` may need no change.

## Order

- Depends on story:canon-simulate. Its code is on `impl/canon-simulate` and not yet on `main`.
- Shares `crates/canon/src/simulate/mod.rs` with story:canon-simulate only.
- Shares `Action` in `model/mod.rs`, `ir/mod.rs`, `eval/read.rs`, `validate/mod.rs`, `diff/mod.rs` and `protocol.yaml` with story:judgement-marking, which also adds fields to `Action`.
- Shares some of those files with story:argument-validator, story:revision-ordering, story:protocol-floor, story:canon-mutate, story:protocol-writer, story:child-outcome-subject and story:pending-decisions.
- So it runs after story:judgement-marking, in a wave that holds none of the others.

## Acceptance

The named test `simulate_records_a_group_in_one_step` (in `crates/canon-cli/tests/simulate_record_groups.rs`) passes over `fixtures/investigation/record-group/`:

1. The `pair` protocol without the declaration exits 1 and prints `3 reachable states, 0 terminable (outcome `done` 0), 4 findings (2 stuck or cycle states)`.
2. With `records_together: [{kinds: [fit, shape]}]` it exits 0 and prints only `2 reachable states, 1 terminable (outcome `done` 1), 0 findings (0 stuck or cycle states)`, so no state holds one of the two kinds alone.
3. A grouped protocol with a result read has a stuck state whose path is one line naming both records in kind-id order, joined by ` and `.
4. A group with one kind, a kind outside `may_produce`, and a kind in two groups are each refused as `invalid-record-group`, naming the action.
5. The IR of a protocol without the field is byte-identical to its IR before the change, the grouped IR holds `"records_together"` sorted and reads back equal, and `canon diff` reports adding a group as `NO SEMANTIC CHANGE`.
6. `simulate_reports_reachability_findings` still passes, and two runs on the grouped fixture give identical bytes.

## Source

- Request 15 and its two replies in `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/requests/15/request.md`.
- Read: `AGENTS.md`, story:canon-simulate, story:judgement-marking and story:supplied-evidence in the first tree at `619fd03`. Also `simulate/mod.rs` and `check/space.rs` at `4b36d52`, `eval/claims.rs`, `eval/outcomes.rs`, `eval/actions.rs`, and the SDC `sdc-intake`, `sdc-plan` and `sdc-spec` protocols at `f364b6b`.
- Run: `canon-sim compile` on all eight SDC protocols, `canon-sim simulate` on `sdc-intake`, on `intake-both.protocol.yaml` and on `pair.protocol.yaml`, and the scratch scripts `emulate.py`, `variants.py`, `classify88.py`, `partial.py`, `table.py` and `pairrun.py` in the work directory.
- 2026-11-21.

## Decided

Option A of § Decision needed. Decided on 2026-11-21 by the session working on the SDC prototype, which the operator named the orchestrator of this plan, after the drafter's figures. Its words:

> A. Store the story as drafted (`records_together` on an action, no evaluation meaning; the walk adds a group in one step and never offers a grouped kind alone), accepted under the grant, after story:judgement-marking. The measurement settles it: B leaves 177 states on intake with either guard, A leaves 89 and makes `passed` reachable with guard `all`; my condition for B was about cost, and the numbers show B does not do the job. Record B and C as rejected alternatives with these figures.

Rejected alternatives, with the figures of § Decision needed (sdc.intake at `f364b6b`):

- **B** (no language change; a step may add any non-empty subset of `may_produce`): with the guard `any`, 1215 reachable states and 35 findings standing for 177 states, as today; with the guard `all`, 729 reachable states, `passed` reachable, 41 findings standing for 177 states. Its cost is small (4 to 8 successors for the one action concerned), and it leaves the states that hold one of the two kinds alone.
- **C** (a step always adds every kind of `may_produce`): it hides a runner that records one kind and stops, for every action.

Two conditions the same session set, which bind the unit:

1. `canon validate` refuses a `records_together` group that names a kind not in the action's `may_produce`, and a kind in two groups of one action; `canon diff` classifies adding or removing a group. (§ Outcome 1 and 2 state both.)
2. The figures for A came from the drafter's Python walk. The story's acceptance repeats them from the Rust binary: sdc.intake at `f364b6b` with the declaration on `questions.check` and the guard `all` gives 405 reachable states and 35 findings standing for 89 states. This is expectation 7 of § Acceptance, added here. The unit takes a copy of that protocol as a fixture with the two edits, names its source, and holds no SDC vocabulary in Canon's code. No Python stays in the repository.

The story was accepted under the operator's standing grant (`approval-record:accept-sdc-request-15-2026-11-21`). The unit builds option A and does not reopen the choice.

## Correction, 2026-11-21

§ Outcome 2 says `canon diff` reports a change of a group as `NO SEMANTIC CHANGE` "as it does for `supplied_by`". That precedent is wrong: since the review of story:supplied-evidence, adding `supplied_by` is `EXPANSION` and removing or replacing it is `BREAKING`, because the evaluator reads it. The class `NO SEMANTIC CHANGE` for `records_together` stands on its own reason: `canon evaluate`, `canon check` and `canon generate` read nothing of it. Noted by the drafter of story:artifact-revised-by and recorded here by the wave coordinator.
