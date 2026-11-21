---
format: aep.planning-md/3
id: story:simulate-revise-step
kind: story
status: active
title: Revise artifacts declared in revises as a step of canon simulate, and keep stuck states and cycles complete when a step sets records aside
tags:
- from-request
refs:
- provider: sdc
  reference: request-revise-step
relations:
- decomposes: epic:protocol-analysis
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:canon-simulate
- depends_on: story:judgement-marking
scope:
- confidence: cited
  path: crates/canon-cli/src/lib.rs
- confidence: cited
  path: crates/canon-cli/tests/canon_simulate.rs
- confidence: cited
  path: crates/canon/src/check/space.rs
- confidence: cited
  path: crates/canon/src/simulate/mod.rs
- confidence: inferred
  path: crates/canon/src/simulate/tests.rs
- confidence: cited
  path: ess/domains/cli.yaml
- confidence: inferred
  path: fixtures/investigation/simulate/
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T04:50:38Z", actor: "human:timo", revision: 10, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w5"}
- {from: "proposed", to: "active", at: "2026-11-21T04:50:38Z", actor: "human:timo", revision: 11, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w5"}
---
## Outcome

`canon simulate` gets a fifth step kind, the **revise step**, built from the `revises` field that story:judgement-marking adds to an action. Canon still runs nothing and revises nothing. A revise step says that a runner may move an artifact to a new revision between two evaluations.

1. **Revise step.** It starts from a state in which action `x` is `admissible`. There is one successor for each non-empty subset S of `x`'s `revises`, combined with no record or with one record step `x` offers today. In the successor, every artifact of S is at a new revision, and then the record is added at the current revision.
   - It is written `` action `x` revises `a` `` or `` action `x` revises `a` and produces evidence `k` result `r` ``.
   - It is tried after `x`'s record steps: subsets in order of their sorted ids, records in today's order.
   - An action without `revises` has no revise step. Every protocol that declares no `revises` gets byte-identical output.
2. **What a revision of `a` does to a state.** It uses the check space's own model (`crates/canon/src/check/mod.rs` "The state space"; `space.rs:18-27`, `:1516-1530`).
   - A current record about `a` leaves the state. Revision binding keeps it from every evidence match, and nothing else reads it in a protocol the walk accepts, since order forms are refused.
   - A record of a dimension that a `children` form reads becomes an earlier record at the superseded revision, written `` … earlier ``. `children` without `revision: current` still reads it (`eval/composition.rs:1-13`; probe e6).
   - A record of a dimension with upstream artifact `a` becomes the variant observed before `a` moved, written `` … observed before upstream `a` moved `` (`space.rs:1395-1421`). The invalidation stage keeps that variant from the claims that `a`'s rules invalidate.
   - A state holds no revision number. Two runs that leave the same records reach the same state. So every walk state is a state of `canon check`'s space, and the reachable count is at most the check's whole product.
3. **Case revision.** It does not move. Authority entries stay unbound and decisions stay at `r`, as `space.rs:18-20` and `:1735-1750` write them.
   - A finding with a denial is therefore about a lasting denial. That finding is true of a real case.
   - A denial bound to a case revision lapses when the runner moves the case revision (probe d4). The capability is then undecided, a value the walk already reaches before its authority step, so no dead end is missed.
   - Moving the case revision on each revise step would free every denied state as soon as any admissible action revises something (story 5, release 3). That would leave a lasting denial's dead end unreported.
4. **R3 and R6 restated.**
   - A state is *open* when some sequence of action steps (record and revise), supplied, expiry and authority steps reaches a legitimate outcome.
   - `stuck-state` is unchanged.
   - `cycle`: the state is not open, no action is `approval-required`, some action is admissible, and every **record** step of every admissible action returns the same state. When a revise step of an admissible action changes the state, the finding appends `` ; every revision by action `x` leads to a state no outcome is reachable from ``.
   - *Waiting* (R3) keeps today's rule. A state waits when it is open and every step of every admissible action, record or revise, returns it.
   - Why nothing goes unreported:
     - The states that are not open are closed under the counted steps.
     - Take a bottom strongly connected set reached from such a state, and in it a state m whose records and capability decisions are maximal under inclusion.
     - A record, supplied or authority step from m only adds, and its successor stays in the set, so each such step returns m.
     - So m decides every capability and is a `stuck-state` or a `cycle`.
     - R5 then prints a contained state.
   - Why the old rule fails with revise steps: today's argument needs every action step to only add. A revise step removes records. Under today's cycle rule the loop in sdc.direction 3 (revise context, re-check, approve denied) holds no reported state.
5. **What exit 0 means (with `revises` declared on every write action).**
   - Every reachable state reaches a legitimate non-decision outcome (R4) through those steps.
   - Every action is admissible somewhere, and every outcome is legitimate somewhere.
   - Still reported as protocol defects:
     - a failed check on an artifact that this protocol revises only through an action no state admits (class a3);
     - a failed check on an artifact only another case revises (a2);
     - a lasting denial with no outcome after it (d);
     - a write action that declares no `revises` (it stays a `cycle`);
     - a revise loop that never reaches an outcome.
   - Not reported:
     - a failed check or mixed results left by a declared revision (a1, c);
     - states that wait on supplied evidence or expiry;
     - anything below the class abstraction. The walk assumes that a check may record the needed result on a new revision, so the 4 intake cycles with `scope.answer` denied (28, 30, 32, 34) are not reported. Of the 8 intake cycles that name `scope.answer`, the 4 with it granted (29, 31, 33, 35) are not reported either; they open the same way.
   - Class e findings stay until a separate fix (§ Order). They are false.

## Why

- **The request.** The second request in `requests/15/request.md`: "Revise step for the walk … say what exit 0 would then mean". The classification of the 87 findings is in the CLASSIFICATION block.
  - All 87 findings were reproduced byte for byte by `canon-sim simulate` over the protocols at `f364b6b`, written out with `git show`.
  - The direction protocol in the `sdc` working tree has moved on to revision 7, uncommitted, with 10 findings. It is not classified here.
- **The evaluator rules behind each class**, each confirmed with `canon-sim evaluate` on hand-written cases in `requests/16/cases/` (output in `probes.out`):
  - **Revision binding** leaves a1:
    - intake 6 has `requirements.valid` false (a1). With requirements at r2, `requirements.check` is admissible (a2).
    - plan 6, at plan p2, re-admits every plan check (a7). Its own writer is not admissible at p1 (a6), which is why plan 3-9 are class a3.
  - Revision binding also leaves c: two `context_approval` records leave `context.approved` unknown (c1). At context x2, `context.check` is admissible (c2).
  - **Invalidation by an upstream artifact** leaves intake 10, 18, 21-23, 30 and 31. At questions q2, `requirements.check` is admissible (a3), through the rule `questions.revised`.
  - **An authority entry bound to a case revision lapses:**
    - An unbound denial blocks `context.approve` before and after context x2 (d1, d2).
    - A denial bound to c1 blocks it at c1 (d3) and leaves it `approval-required` at c2 (d4).
  - **A decision bound to a case revision lapses:** `abandoned` is legitimate at c1 and blocked at c2 (x1, x2). So a revise step that moved the case revision would also drop every taken decision.
- **Why sdc 1 is stuck** (the coordinator's class e):
  - `intake.opened` and `roadmap.copied` are true, and `children.passed` is unknown with no `case_outcome` (e1).
  - A `case_outcome` `passed` record makes `direction.open` admissible (e2). epic 2 is the same: adding the child outcome makes `plan.open` admissible (e4, e5).
  - The walk adds a child outcome only as a step of the opening action while it is admissible (`check/mod.rs:1021-1027`, `simulate/mod.rs:637-653`). The opening action's guard closes once its opening record exists.
  - A runner records the outcome when the child ends, so these 4 findings are false of a real case.
  - A revise step would leave them only through an unrelated revision of `idea` or `epic` (e3).
- **Bound.** `canon-sim check` gives the whole product of each SDC space: sdc 4608, release 24576, story 12288, epic 36864, direction 49152, intake 49152, plan 65536, spec 65536. With Outcome 2, none of the eight can be refused as `reachable-state-bound` under any option.
- **Not verified:**
  - the post-revise state and finding counts (no build);
  - the `revises` each SDC action would declare, read from its description ("moves … to a new revision");
  - that the SDC driver moves the case revision on every artifact revision (story:authority-revision-binding § Why, `$HOME/beyond10x/sdc/driver/src/main.rs:280-285`, not read);
  - that sdc reaches `operating` only when `intake.open` adds the child outcome before its opening record (inferred from the step rule).

## Decision needed

Which artifact a step may revise, and when.

| Option | `ess/`, model, walk | Reachable states and bound | Classes left | Not-open rule |
|---|---|---|---|---|
| A. Only `revises` of an admissible action, as a step of it | No model change; walk only | Successors per state go up by (2^k−1)(1+r) per admissible revising action; k ≤ 2 in SDC (`amend.upstream`). At most the check's product | a1, c | Holds with the restated `cycle` (Outcome 4) |
| B. Any declared artifact at any time, as an external step like a supplied record | Walk only | Plus one successor per artifact per state (sdc 3, direction 7, intake 6, plan 5, epic 3, spec 4, story 4, release 2). Same ceiling | a1, a2, a3, c, and e for a wrong reason | Holds. It hides the a3 guard gaps of plan 3-9 and needs no `revises` |
| C. A, plus an external step for an artifact that declares the protocol revising it (`revised_by`, like `supplied_by`) | Language change: model, IR, validate, diff, `protocol.yaml` | A plus one successor per declared artifact | a1, c, and a2 counted as waiting | Holds. Like R3 for supplied kinds |

Recommendation: A. It moves an artifact only where the protocol says an action does. It keeps the a3 guard gaps reported. It needs no language change, and C can follow if `sdc` wants class a2 counted as waiting.

## ESS first

- **First commit.**
  - In `ess/domains/cli.yaml`, `canon.cli.Simulate` (`:238-278` on `impl/canon-simulate`) gets a new summary and new comments on `response` and `rejected`. They name the revise step and the restated `cycle` rule, with `# read from:` this story. No input or outcome changes.
  - New fixtures, each a `protocol.yaml` and a hand-written `report.txt`: `fixtures/investigation/simulate/revise/`, `revise-undeclared/`, `revise-upstream/`, `revise-denied/`, `revise-mixed/` and `revise-child/`.
  - The named test, with `CASES` going from 15 to 21.
- **Red on that commit.** `simulate_revises_declared_artifacts` fails:
  - `revise` exits 1 with `cycle` for `fix`, where the fixture expects exit 0;
  - `revise-denied` prints the cycle without the revision suffix.
- **Model.** Unchanged; `revises` comes from story:judgement-marking. Without it, `every_simulate_fixture_protocol_validates` is red too.

## Scope

- **In:** the revise step, the state change of Outcome 2, the fixed case revision, the restated `cycle` and waiting rules, rendering, fixtures and the test. — cited
- **Out:**
  - options B and C;
  - a child-outcome step (class e);
  - several records per step (story:simulate-record-groups; it composes with Outcome 1 as "one record or one group");
  - order forms;
  - bound authority steps;
  - the SDC `revises` declarations. — cited
- **Surfaces:**
  - `crates/canon/src/simulate/mod.rs` (`Step`, `Layout`, step loop `:635-697`, idle and waiting `:699`, `:759-773`) — cited
  - `crates/canon/src/check/space.rs` (`walk_evidence` `:2451`, `render_walk` `:2489`: upstream vector and earlier records) — cited
  - `crates/canon/src/simulate/tests.rs` — inferred
  - `crates/canon-cli/src/lib.rs` (`Simulate` doc `:154-176`) — cited
  - `crates/canon-cli/tests/canon_simulate.rs` — cited
  - `ess/domains/cli.yaml` — cited
  - `fixtures/investigation/simulate/revise*/` — inferred
- **The coordinator's at integration:** `website/docs/reference/cli.md` and `website/status.yaml`.
- **Confidence:** medium. The state change is read from the space. Whether the `Layout` slots take an upstream vector without a rewrite is not established.

## Order

- **Depends on:** story:canon-simulate (active, `impl/canon-simulate` at `4b36d52`, not merged) and story:judgement-marking (wave 5, in implementation). This is follow-up 4 of story:judgement-marking § Order.
- **Shares files with active stories:**
  - `simulate/mod.rs` with story:simulate-record-groups;
  - `check/space.rs` with story:revision-ordering and story:child-outcome-subject (both inferred);
  - `crates/canon-cli/src/lib.rs` and `ess/domains/cli.yaml` with story:canon-mutate and story:protocol-floor.
- **Wave:** run it in a wave without those stories, after story:simulate-record-groups.
- **Separate story:** class e, a `case_outcome` step after the opening record. It is not drafted here.

## Acceptance

`crates/canon-cli/tests/canon_simulate.rs::simulate_revises_declared_artifacts` passes:

1. `revise`: check `test` may record `pass` or another result about `doc`. `fix` (`revises: [doc]`, no `may_produce`) is admissible while the claim is false. The outcome needs `pass`. Exit 0, summary line only.
2. `revise-undeclared`, the same protocol without `revises`: exit 1, `cycle` naming `fix`, the bytes the current walk gives.
3. `revise-upstream`: a record about `b` under an invalidation rule with upstream `a`, and `fix` revising `a`. Exit 0; the path holds `` action `fix` revises `a` ``.
4. `revise-denied`: approving needs a denied capability. Exactly one `cycle`, ending `` ; every revision by action `fix` leads to a state no outcome is reachable from ``, exit 1.
5. `revise-mixed`: an approve action that may record two results, and `fix` revising its artifact. No finding, exit 0.
6. `revise-child`: a `children` outcome read after its subject is revised. Exit 0, and the reached state renders the outcome `earlier`.
7. The 15 existing fixtures give unchanged bytes, and two runs of each new fixture give identical bytes.

## Source

- **Request:** `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/requests/15/request.md`, request "revise step".
- **Canon trees read:** `AGENTS.md`, story:canon-simulate, story:judgement-marking, story:authority-revision-binding, story:supplied-evidence, story:simulate-record-groups, `review-result:adversary-canon-simulate-pass-1`, `crates/canon/src/simulate/mod.rs`, `check/{mod,space}.rs`, `eval/{mod,composition}.rs`, `canon-cli/src/lib.rs`, `canon-cli/tests/canon_simulate.rs`, `ess/domains/cli.yaml`.
- **SDC protocols:** the eight at `f364b6b`.
- **Runs:** `canon-sim simulate`, `check`, `compile` and `evaluate` (`requests/16/{probes,cases,classify}.py`). 2026-11-21.

## Decided

Option A of § Decision needed. Decided on 2026-11-21 by the session working on the SDC prototype, which the operator named the orchestrator of this plan, after the classification below. Its words:

> Choice 1: A, and with that the story is accepted under the grant (a revise step only for `revises` of an admissible action, case revision fixed, no language change; a3 guard gaps stay visible, which is what I want from the tool). C (`revised_by` on an artifact, a2 becomes "waiting") stays a separate draft, not accepted: I decide it after A is implemented and the a2 count on SDC is measured by the binary, since it is a language change. B is rejected: it hides a3.

The same session fixed the order of the three stories that edit `crates/canon/src/simulate/mod.rs` after story:canon-simulate: the child-outcome step first, then story:simulate-record-groups, then this story. The story was accepted under the operator's standing grant (`approval-record:accept-sdc-revise-step-2026-11-21`). The unit builds option A and does not reopen the choice. When it is built, the unit runs the binary on the SDC protocols and reports the count of class a2 findings, which that session needs to decide option C.

## Classification of the 87 findings, 2026-11-21

The drafter's classification of every finding `canon simulate` (release build of `impl/canon-simulate` at `8fb960c`) prints on the eight SDC protocols at `f364b6b` of beyond10x/sdc, branch `fix/roadmap-read-precondition`. Finding numbers are positions in each output. As the drafter returned it:

```
87 findings, eight files at f364b6b (all reproduced byte for byte)
class | rule that leaves it under A | sdc dir epic intake plan spec story release | total
a1 failed check; an action of this protocol revises the artifact | revision binding; upstream invalidation for intake 10,18,21-23,30,31 | 0 2 0 17 8 15 3 1 | 46
a2 failed check; only another case revises it (idea, roadmap, spec) | not left; B or C (revised_by) leaves it | 0 0 1 18 2 1 1 1 | 24
a3 failed check; its only revisers (plan.decompose, amend.upstream) are guarded on coverage | not left: real guard gap; B hides it | 0 0 0 0 7 0 0 0 | 7
c two context_approval records (approved and other) | revision binding via context.change | 0 2 0 0 0 0 0 0 | 2
d lasting denial: owner.approve, change.merge, release.approve | not left; a bound denial lapses only if the case revision moves | 0 2 0 0 0 0 1 1 | 4
e child outcome only as a step of the closed opening action | no revise rule; a case_outcome record (e2, e5); a false finding | 2 0 2 0 0 0 0 0 | 4
Findings: sdc e 1,2 | dir a1 1,2; d 3,4; c 5,6 | epic a2 1; e 2,3
intake a2 1-5,11-15,24-27,32-35; a1 6-10,16-23,28-31
plan a2 1,2; a3 3-9; a1 10-17 | spec a2 1; a1 2-16 | story a2 1; a1 2-4; d 5 | release a2 1; a1 2; d 3
Corrections to the coordinator's classes:
- (b) is not a class of its own: every a1 and a2 cycle is (b) seen from the action.
- (a) splits into a1, a2 and a3.
- (d) also covers story 5 (change.merge). The 8 intake cycles with scope.answer are a1 or a2, not d.
- (e) also covers epic 2 and 3. sdc 2 and epic 3 also hold an a2 record.
Under A, once sdc declares revises as its descriptions say (expected, not run): a1 and c are not reported; a2, a3, d and e are still reported (d as a cycle with the revision suffix).
```

## Decided in review, after adversary pass 1

The coordinator of wave 2026-11-20-w8 decided these on 2026-11-21, from the unit's report and from `review-result:adversary-simulate-revise-step-pass-1` (green, three notes, all introduced by the unit).

- **D1. The text of a `cycle` finding.** When a revise step of an admissible action changes the state, the finding says `every record step of action `x` returns this state` and ends `; every revision by action `x` leads to a state no outcome is reachable from`. When no revise step changes the state it says `every step of`, as before this story. § Outcome 4 wrote "every step"; that is false of a state a revise step leaves, so the built text stands. A protocol in which no action declares `revises` prints the bytes it printed before: the adversary compared 147 `protocol/1` files under `fixtures/` and `conformance/` with the binary of the base and found 141 equal and 6 different, each of the 6 declaring `revises`.
- **D2. Acceptance 3 and 6.** Each asks for exit 0 together with a printed path or rendering. The walk prints a path and a state only with a finding, so no run gives both. The fixtures `revise-upstream` and `revise-child` exit 0 and hold the summary line; the path `then action `fix` revises `a`` and the ` earlier` rendering are pinned in `crates/canon/src/simulate/tests.rs`. The adversary confirmed from the code that no exit-0 run prints a path.
- **D3. `fixtures/investigation/judgement-marking.yaml`.** Its walk went from one `cycle` and exit 1 to no finding and exit 0, because the action that declares `revises` now leaves the state. The adversary's own enumerator gives the same 19 states and no state that is not open. No test read that walk; the adversary's case `judgement_marking_has_no_finding_once_its_revision_is_walked` pins it (`f159b8ac`).
- **D4. The record of a revise step is added after the revision.** A mutant that added it before passed every test of the unit; the adversary's case `a_revise_step_adds_its_record_after_the_revision` fails on it (`f159b8ac`). The code was right.
- **D5. The help text states the whole waiting rule.** `canon simulate --help` left out that a waiting state has no legitimate outcome and no approval-required action; by its text `revise-child` would count 4 waiting states where the walk counts 2. Corrected at `80168ba9`; the module docs and `ess/domains/cli.yaml` were right.
- **D6. The argument that no state that is not open goes unreported** (§ Outcome 4) is restated in the module docs of `simulate` as a forward argument over record and authority steps, which only add. It uses no revise, child-outcome or expiry step. The adversary's enumerator walked 6,000 random protocols and 8 fixtures and probes, 74,604 states, 393 of them cycles with the revision suffix, and found no disagreement in state count, terminable count, stuck or cycle count, or printed findings.
- **D7. Left to story:child-outcome-recovery:** an unbound `case_outcome` dimension is about the least artifact id (the unit's known limit W0, stated in the module docs).
- **D8. The measurement of class a2** on the requesting project's protocols is not part of this unit (§ Decided named it). That project declares no `revises`: its nine protocols give equal bytes before and after, 71 findings in sum. Its session adds the declarations in a story of its own once a release carries the revise step, and measures then.
