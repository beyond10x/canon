---
format: aep.planning-md/3
id: story:simulate-child-outcome-step
kind: story
status: active
title: Let a child case's outcome arrive after its opening action as a step of canon simulate, and drop the printed limit line
tags:
- from-request
refs:
- provider: sdc
  reference: request-16
relations:
- decomposes: epic:protocol-analysis
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:canon-simulate
scope:
- confidence: cited
  path: crates/canon-cli/src/lib.rs
- confidence: inferred
  path: crates/canon-cli/tests/simulate_child_outcome.rs
- confidence: cited
  path: crates/canon/src/simulate/mod.rs
- confidence: inferred
  path: crates/canon/src/simulate/tests.rs
- confidence: cited
  path: ess/domains/cli.yaml
- confidence: inferred
  path: fixtures/investigation/child-outcome/
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T05:02:20Z", actor: "human:timo", revision: 9, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w5"}
- {from: "proposed", to: "active", at: "2026-11-21T05:02:20Z", actor: "human:timo", revision: 10, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w5"}
---
## Outcome

`canon simulate` lets a child case's `case_outcome` record arrive at any step after the action that opened the child was taken. Canon still runs nothing: the step says that a runner may record a child's outcome between two evaluations.

1. **Opening mark.** A walk state holds one mark per action that declares `case_open`. Every step of that action sets its mark: each record step, and the one step of an action that may produce nothing. No step clears a mark. A state writes each set mark after its records and capabilities, in action order: `` action `a` opened a child ``.
2. **Action step.** It adds only records of the action's `may_produce`. `case_outcome` is no longer one of them, so the walk stops reading `check::produces` for it. `canon check` keeps `produces` unchanged.
3. **Child-outcome step.** From a state with at least one mark set, one record of `case_outcome` arrives. Each dimension, class and producer profile of the kind is a successor, as an action step takes them today.
   - A class other than `open`, in a state holding the class `open`, has a second successor that also drops `open`. One record is one child, and `duplicate-child-case` refuses a second record for it, so an open child that ends is replaced, not joined.
   - It is written `` a child of action `a` records evidence `case_outcome` result `r` ``, plus `` , replacing result `open` `` for the second successor. `a` is the least action whose mark is set.
   - It is tried after the supplied steps and before the expiry steps.
4. **Rules R3 to R5 with the new step.**
   - A state is open when some sequence of action, supplied, child-outcome, expiry and authority steps reaches a legitimate outcome.
   - The child-outcome step is not an action step, so `stuck-state` and `cycle` are unchanged. A state that only a child outcome can open waits.
   - When the protocol declares an opening action, the summary prints the waiting count as `` W waiting on supplied evidence, a child outcome or expiry ``. Otherwise the summary is unchanged.
   - R5 containment also requires every mark of A to be set in B. `size` counts set marks.
5. **The limit line goes.** story:canon-simulate prints a line naming this limit whenever a protocol has an opening action. That line is no longer printed, and its paragraph leaves the module docs and the `Simulate` clap doc. A protocol without an opening action gets the same bytes as before.

**Why both claims still hold.**
- *Every printed finding is true of a real case.* A child-outcome step is offered only after a step of the opening action, which is when a runner can record the outcome. A printed state is not open, so no child outcome that may arrive opens it.
- The replacing successor keeps an `open` record from blocking a child forever. Without it, the probe `child-ends-open` gets a false `stuck-state` whose state holds `result `open``.
- *No dead end goes unreported.* Action and authority steps still only add records, marks and decisions. A non-open state therefore leads by them to a state where they all return it, and that state is a `stuck-state` or a `cycle` (R3). The removing successor is external, as an expiry is (R2), so the argument does not use it.
- story:simulate-revise-step's bottom-set argument still holds: its maximal state m is maximal over marks too, and its step rule counts only action steps.
- **Limits:** two classes of `case_outcome` stand for two children, also for an action a real case takes only once. A walk state is no longer a state of `canon check`'s space. It is one such state plus k marks, where k is the number of opening actions. Evaluation reads only the space's part, so two states that differ only in marks evaluate the same.
- **Bound:** at most 2^k × (states without marks) reachable states. `check::STATE_BOUND` (65,536) counts marked states, under R1 as before, and no partial walk is reported.

## Why

- **Request 16** (`$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/requests/17/request.md`) covers class e of the classification in story:simulate-revise-step: sdc findings 1 and 2, epic findings 2 and 3.
- **The defect in the code**, `impl/canon-simulate` at `4b36d52`:
  - `produces` (`crates/canon/src/check/mod.rs:1019-1027`) adds `case_outcome` to an opening action's kinds.
  - `action_records` (`crates/canon/src/simulate/mod.rs:579-583`) takes them from `produces`.
  - The step loop adds them only for admissible actions (`:637-654`). The module docs say so (`:26-32`).
  - The `children` form reads one `case_outcome` dimension. It has no subject, so the space makes one unbound dimension (`space.rs:439-441`).
- **The four findings** (`$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/w5/sdc-run/sdc.txt`, `sdc-epic.txt`; the protocols at `f364b6b`):
  - sdc 1 and 2: `intake.open` has the guard `intake.opened is unknown` and the opening record `intake_opened`. Once `intake_opened` is present the guard is false, so `case_outcome` never arrives. `direction.open` needs `children.passed`.
  - epic 2 and 3: `spec.open` has the guard `spec.opened is unknown` (with `roadmap.approved`) and the opening record `spec_opened`. `plan.open` needs `children.passed`.
- **Confirmed with `canon-sim evaluate`** (`requests/17/work/probes17.py`, output in `probes17.out`):
  - sdc 1 + `passed`: `direction.open` admissible (p2, as e2 of `requests/16`).
  - sdc 1 + `open`: unknown, nothing admissible (p3).
  - sdc 1 + `abandoned`: false (p4).
  - One child with `open` and `passed`: refused `duplicate-child-case` (p5). Two children: unknown (p6).
  - sdc 2 + `passed`: `direction.open` admissible (p8). After `direction_opened` and a passed child, nothing is admissible, because `roadmap.approved` is false (p9).
  - epic 2 + `passed`: `plan.open` admissible (q2, as e5).
  - epic 3 + `passed`: nothing admissible, because `gate.passed` is false (q4).
  - So sdc 1 and epic 2 are false.
  - sdc 2 and epic 3 are false today, since their states are stuck only because no outcome arrives. Under this story they stay stuck for a true reason: a `roadmap_approval` without `approved`, or an `ess_gate` without `passed`, about an artifact only another case revises (class a2).
- **Measured by emulation**, not by a binary. `requests/17/work/emulate_e.py` is a Python walk over the compiled IR, with `children` added to the earlier drafter's `emulate.py`. It reproduces the copied binary exactly for today's rule: sdc 108/55/2/17, epic 114/58/3/20, and the four probe fixtures. Under option A:
  - sdc: 86 reachable, 44 terminable, 5 waiting, 2 findings (19 states).
  - epic: 82 reachable, 42 terminable, 6 waiting, 3 findings (19 states).
  - The sdc 1 and epic 2 states wait. Each protocol gains one true `stuck-state`: the child ended with an outcome `children` does not list. Today that state is hidden by R5 inside sdc 1 or epic 2.
  - Only `sdc` and `sdc-epic` declare `case_open` among the eight, so the other six are unchanged.
- **Not verified:** the walk under any option in Rust (no build), and the wall time.

## Decision needed

When a child's `case_outcome` record may arrive. None of the options changes `ess/` beyond `cli.yaml` or the model. Probe fixtures are in `requests/17/work/fx/`.

| Option | Walk | sdc, epic states | Findings | Cost |
|---|---|---|---|---|
| A. Mark per opening action, child-outcome step while marked, a terminal class also replaces `open` (§ Outcome) | marks and one external step | 86, 82 (today 108, 114) | sdc 1 and epic 2 gone, no false finding in the probes | ≤ 2^k states |
| A0. The coordinator's sketch as written: add only | as A without the replacing successor | 86, 82 | one false `stuck-state` per protocol holding `result `open`` (sdc 3 findings, epic 4) | breaks the first claim |
| A1. A, and the opening step still offers `case_outcome` | as A | 160, 148 | as A | walks an outcome before its child exists |
| B. No mark: an outcome may arrive while a record the opening action may produce is present | no new state | not run | an opening with no `may_produce` (probe `child-repeat`) never gets one | wrong when another action or a supplier produces that kind, or when the record expires |
| C. No mark: an outcome may arrive from any state, like a supplied record | no new state | not run | paths with an outcome before any opening | hides a dead end in which the opening action is never taken |

Recommendation: A, because it is the only option measured with no false finding and every dead end kept.

**Checked against the brief's cases (option A):**
- *An opening action admissible more than once* (`child-repeat`: no `may_produce`, guard `resolved is unknown`): the first step sets the mark, later steps return the state, and the state waits. Each class is one or more children, never two records of one child.
- *`open` then a terminal class*: covered by the replacing successor.
- *`revision: current`* (story:child-outcome-revision): the walk adds only current records, so `current` and `any` read the same in every walk state today.
- *An opening action with other kinds*: each of its record steps sets the mark (`sdc`, `sdc-epic`).
- *An opening action that requires a capability* (`child-denied`): no mark is set until it is granted and taken. The binary today prints a false `stuck-state` for `{evidence `opened`, capability `delegate` granted}`. Option A prints only `{capability `delegate` denied}`.

## ESS first

- **First commit.**
  - In `ess/domains/cli.yaml`, `canon.cli.Simulate` (`:238-278` on `impl/canon-simulate`) gets a new summary and new comments on `response` (waiting on a child outcome) and on `rejected` (the open rule names child-outcome steps), with `# read from:` this story. No input or outcome changes.
  - New fixtures under `fixtures/investigation/child-outcome/`, each a `protocol.yaml` and a hand-written `report.txt`: `closed`, `ends-open`, `repeat`, `denied`, `copied-root` and `copied-epic`. The two copies are SDC's `protocol/sdc.protocol.yaml` and `protocol/sdc-epic.protocol.yaml` at `f364b6b`, unchanged, each with a `source.txt` naming the repository, path and commit.
  - The named test, in a file of its own, so `CASES` of `canon_simulate.rs` stays at 15.
- **Red on that commit.** `simulate_lets_a_child_outcome_arrive_after_its_opening` fails: `closed` exits 1 with `` stuck-state: state {evidence `opened`} `` (the copied binary's output), where `report.txt` expects exit 0. The model does not change.

## Scope

- **In:** the opening marks, the child-outcome step and its replacing successor, the action step without `case_outcome`, the open, waiting and containment rules, rendering, removing the limit line, fixtures and the test. (cited)
- **Out:**
  - binding a child outcome to the opening's subject (story:child-outcome-subject adds `subject` to `children`);
  - a revise step and what it does to a mark (story:simulate-revise-step);
  - `canon check` and `produces`;
  - expiry of a `case_outcome` record (adversary pass 2 of story:canon-simulate);
  - counting children per opening. (cited)
- **Surfaces:**
  - `crates/canon/src/simulate/mod.rs` (`Step`, `Layout`, `action_records` `:579`, step loop `:637-697`, waiting `:759-773`, `render_step` `:775`, module docs) — cited
  - `crates/canon/src/simulate/tests.rs` — inferred
  - `crates/canon-cli/src/lib.rs` (`Simulate` doc `:154-171`) — cited
  - `ess/domains/cli.yaml` — cited
  - `crates/canon-cli/tests/simulate_child_outcome.rs` (new) — inferred
  - `fixtures/investigation/child-outcome/` (new) — inferred
- **The coordinator's at integration:** `website/docs/reference/cli.md`.
- **Confidence:** medium. The step rule and the numbers are read and emulated. Whether `render_walk` (`space.rs:2489`) needs the marks, or `simulate/mod.rs` appends them, is not established.

## Order

- **Depends on:** story:canon-simulate (active, `impl/canon-simulate` at `4b36d52`, not merged; it merges with the limit line this story removes).
- **The order fixed by `sdc`** for `crates/canon/src/simulate/mod.rs`: this story, then story:simulate-record-groups, then story:simulate-revise-step.
- **What the later two read differently:**
  - record-groups: "`case_outcome` … stays a single-kind step" holds as "the child-outcome step is never in a group". `case_outcome` is in no `may_produce`, so `invalid-record-group` refuses grouping it anyway.
  - revise-step, Outcome 2: "every walk state is a state of `canon check`'s space" holds for the state without its marks.
  - revise-step, § Why: "none of the eight can be refused" no longer follows from the product for `sdc.epic` (36,864 × 2^4).
  - revise-step must say what a revision of an opening's subject does to its mark. Under `revision: current`, an old child's late outcome must not arrive as a current record.
- **Shares** `crates/canon-cli/src/lib.rs` and `ess/domains/cli.yaml` with story:simulate-revise-step, story:canon-mutate and story:protocol-floor.

## Acceptance

`crates/canon-cli/tests/simulate_child_outcome.rs::simulate_lets_a_child_outcome_arrive_after_its_opening` passes over `fixtures/investigation/child-outcome/` with the Rust binary. The figures are emulated, and the binary must repeat them:

1. `closed`: exit 0. It prints only `` 9 reachable states, 5 terminable (outcome `declined` 4, outcome `done` 1), 3 waiting on supplied evidence, a child outcome or expiry, 0 findings (0 stuck or cycle states) ``.
2. `ends-open`: exactly one `stuck-state`, whose state holds `` evidence `case_outcome` `` without a result and `` action `open` opened a child ``. No printed state holds `` result `open` ``.
3. `repeat`: exit 0, with `9 reachable states` and `3 waiting`.
4. `denied`: exactly one finding, `` stuck-state: state {capability `delegate` denied} ``. No path holds a child-outcome step before a step of `open`.
5. `copied-root`:
   - summary `` 86 reachable states, 44 terminable (outcome `abandoned` 43, outcome `operating` 2), 5 waiting on supplied evidence, a child outcome or expiry, 2 findings (19 stuck or cycle states) ``;
   - no printed state is sdc 1's records, with or without a mark;
   - sdc 2's records are printed only with the mark of `intake.open`.
6. `copied-epic`:
   - summary `` 82 reachable states, 42 terminable (outcome `abandoned` 41, outcome `passed` 2), 6 waiting on supplied evidence, a child outcome or expiry, 3 findings (19 stuck or cycle states) ``;
   - epic 2's records are not printed;
   - epic 3's records are printed only with the mark of `spec.open`.
7. No output names the limit line. The 15 fixtures of `simulate_reports_reachability_findings` give unchanged bytes, and two runs of each new fixture give identical bytes.
8. `git ls-files '*.py'` is empty. No file under `crates/` gains the string `sdc`. The test names the copies only by directory.

## Source

- **Request:** request 16 in `$HOME/.cache/scratch/140bdba0-2c69-4bdb-acbe-c1b4ecf2c8b8/requests/17/request.md`, and the brief `requests/drafter-brief-2.md`.
- **First tree (`43515423`):** `AGENTS.md`, story:canon-simulate, story:simulate-record-groups, story:simulate-revise-step, story:child-outcome-revision, story:child-outcome-subject.
- **`impl/canon-simulate` at `4b36d52`:** `simulate/mod.rs`, `check/mod.rs`, `check/space.rs`, `eval/composition.rs`, `eval/mod.rs`, `model/mod.rs`, `canon-cli/src/lib.rs`, `canon-cli/tests/canon_simulate.rs`, `ess/domains/cli.yaml`, and the uncommitted `crates/canon/tests/adversary_simulate.rs`.
- **SDC at `f364b6b`:** `sdc` and `sdc-epic`.
- **Run with the copied binary:** `compile`, `simulate` (same bytes as `w5/sdc-run`) and `evaluate`, through `prep.sh`, `fx.sh` and `probes17.py`, plus `emulate_e.py`, all in `requests/17/work/`.
- **Probes reused:** e1, e2, e4 and e5 of `requests/16/probes.out`.
- 2026-11-21.

## Decided

Option A of § Decision needed: one mark per opening action, a child-outcome step while a mark is set, and a second successor in which a terminal class replaces `open`. Decided on 2026-11-21 by the wave coordinator on the drafter's recommendation. The coordinator's own sketch in the request was option A0; the drafter showed that it prints a false `stuck-state` for a child that is open and then ends, so A0 is rejected, as are A1, B and C for the reasons in the table.

The session working on the SDC prototype, which the operator named the orchestrator of this plan, asked for this story as its request 16 and for its acceptance on arrival; it did not choose among the options and may overrule this choice until the unit merges. It fixed the order: this story, then story:simulate-record-groups, then story:simulate-revise-step.

The story was accepted under the operator's standing grant (`approval-record:accept-sdc-request-16-2026-11-21`). The unit builds option A and does not reopen the choice. The figures of § Acceptance come from the drafter's Python walk; where the Rust binary gives another figure, the unit finds out which is right, reports it and does not adjust the walk to the expected number.
