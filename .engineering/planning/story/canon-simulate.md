---
format: aep.planning-md/3
id: story:canon-simulate
kind: story
status: active
title: Find stuck states, cycles and dead actions by walking reachable states with canon simulate
tags:
- from-request
refs:
- provider: sdc
  reference: docs/canon-requests.md
relations:
- decomposes: epic:protocol-analysis
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:canon-check
- depends_on: story:ess-command-surface
scope:
- confidence: cited
  path: crates/canon-cli/src/lib.rs
- confidence: cited
  path: crates/canon-cli/src/main.rs
- confidence: cited
  path: crates/canon-cli/src/simulate.rs
- confidence: cited
  path: crates/canon-cli/tests/canon_simulate.rs
- confidence: inferred
  path: crates/canon/src/check/space.rs
- confidence: cited
  path: crates/canon/src/lib.rs
- confidence: inferred
  path: crates/canon/src/simulate/mod.rs
- confidence: cited
  path: ess/domains/cli.yaml
- confidence: inferred
  path: fixtures/investigation/simulate/
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-11-20T22:35:01Z", actor: "human:timo", revision: 13, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
- {from: "proposed", to: "active", at: "2026-11-20T22:35:01Z", actor: "human:timo", revision: 14, executor: "agent:claude-140bdba0", correlation: "wave/2026-11-20-w2"}
---
## Outcome

`canon simulate --path <protocol>` (clap derive, like `canon check --path`) walks every state a case can reach from the empty case, one step at a time. It reports where the protocol leaves a case stuck. `simulate::simulate(ir) -> Result<Report, Refusal>` in the library is a pure function of the compiled protocol: no clock, network, filesystem or runner, and identical input gives identical bytes.

**What a step is.** A step is a function from a state to the next state, computed from the IR. A state is a state of the space `canon check` enumerates (a `State`, `crates/canon/src/check/space.rs`), and Canon evaluates it with `check::evaluate` (`check/mod.rs:635`). There are three kinds of step:
- **Action step.** It is taken only from a state in which the action is `admissible`. It adds one record of one kind the action may produce (`may_produce`, plus `case_outcome` for a `case_open`, as `produces` does, `check/mod.rs:700-708`). The record goes into one dimension of that kind, with one class of that dimension and one producer profile of it. It is at the current revision, observed after every move of an upstream artifact. Each combination of dimension, class and profile is a separate successor. An action that may produce nothing has one step, which leaves the state unchanged.
- **Authority step.** A capability some action requires goes from undecided to `granted` or to `denied`.
- **Decision step.** A decision name some outcome requires goes from not taken to taken.

**What a step is not.** No action runs. No arguments are validated. No artifact is revised, because `protocol/1` has no declaration of an action that revises one. No record is removed and no case is terminated. A step never produces a record at a superseded revision.

Every step only adds, so the step graph has no cycle except a step that returns its own state.

**Initial states and order.** There is one initial state per assignment of case-input values, in input order. Breadth-first search from those states reaches each state first along a shortest path. Steps are tried in this order:
1. actions in identifier order, then kinds, dimensions, classes and profiles in the check's own orders;
2. capabilities in identifier order, `granted` before `denied`;
3. decisions in name order.

**Findings.** They come in this order:
- `stuck-state`: no outcome is legitimate, and no action is `admissible` or `approval-required`. Each one names its state and its shortest path.
- `cycle`: no outcome is legitimate, no action is `approval-required`, at least one action is `admissible`, and every step of every admissible action returns the same state. It names the state, the path and those actions.
- `dead-action`: an action that is admissible in no reachable state.
- `unreached-outcome`: an outcome that is legitimate in no reachable state. `no-outcome-reachable` is printed once when every declared outcome is unreached.
- Stuck states and cycles are ordered by path length, then by rendering.

**Output.** The findings are followed by one line: `` simulated: protocol `p` revision 1: N reachable states, T terminable (outcome `a` x, outcome `b` y), F findings ``. A terminable state is one in which some outcome is legitimate. The walk continues through terminable states, because Canon terminates no case. A path is written as its steps, for example `` action `a` produces evidence `k` result `r` ``, `` capability `c` denied ``, `` decision `d` taken ``.

**Exit status.** 0 when there is no finding, 1 when there is a finding or a refusal.

**Bound and refusals.**
- The walk is refused with `reachable-state-bound` when it would visit more than `check::STATE_BOUND` (65,536) states.
- The bound counts reachable states. The check counts the product of every dimension (`check/mod.rs:249-252`).
- Reachable states are a subset of the check's space. So every protocol `canon check` accepts can be simulated, and so can a protocol the check refuses whose reachable part fits.
- A bounded partial walk is not offered: a walk that stops early and finds nothing cannot report "nothing found" (`UNKNOWN` is not `FALSE`).
- A protocol with an order form (`first`, `before`, `after`, `since`) is refused as `ordered-evidence`, as are the evaluator's `predicate-too-deep` and `claim-cycle`. Order forms are refused because the check's space puts a dimension's current records at one instant (`check/mod.rs:124-133`, `:164-166`), so a walk's same-dimension sequence has no state there.

## Why

**The request.** `$HOME/beyond10x/sdc/docs/canon-requests.md:29-31` (item 4), with the detail in `docs/canon-simulation-request.md:30-35`. That file asks for:
- a breadth-first walk from the empty case;
- stuck states with the shortest sequence that reaches each;
- cycles, dead actions and the reachable state count;
- the determinism of `evaluate`;
- refusal or a stated bound above 65,536.

Its definition of a stuck state is at `:21`. `boundary.md:74` says finding stuck states before a run is Canon's work, and that a runner's verdicts are terminable, awaiting authority and stuck. So `approval-required` is kept apart from stuck here. `boundary.md:164-166` gives the walk the highest value of the checker work.

**How it differs from `canon check`.**
- The check's `unreachable-outcome` and `unsatisfiable-precondition` are judged over every combination of records (`check/mod.rs:258-260`, `:583-598`). `canon generate` walks that same space (`generate/mod.rs:3-4`).
- `canon simulate` judges over the states reachable through steps. Its `dead-action` and `unreached-outcome` include every finding the check makes, plus actions and outcomes that every possible record set reaches but that no order of steps from the empty case reaches. Example: two actions, each needing evidence only the other produces.
- `stuck-state` and `cycle` have no counterpart in the check.

**Verified in the canon tree (at `15cfece`).**
- `EvidenceProduction` holds only `evidence`, a kind, with no result and no subject (`crates/canon/src/model/mod.rs:253-256`). The IR holds a sorted list of kinds (`crates/canon/src/ir/mod.rs:168`, `:263`). So "each result variant" (request `:30`) has to come from the check's classes: each result some match reads, plus a record with no result (`check/mod.rs:24-29`).
- Action statuses are `admissible`, `approval-required` and `blocked` (`crates/canon/src/eval/actions.rs:105-143`). Outcome statuses are `legitimate` and `blocked` (`crates/canon/src/eval/outcomes.rs:7-14`).
- `check::evaluate` builds a case with no `termination` and no instant (`check/mod.rs:654`, `:662`).
- Order rankings are built only within the bound (`space.rs:571-573`).
- The sdc limitation protocols list no order dimension (`docs/limitations/output.txt:2,6,10,26`), so the `ordered-evidence` refusal leaves them in scope.

**Boundary.**
- Running actions, the driver's `revises` binding (`docs/driver.md:50`) and the run-time "useful" rule (`driver.md:56-57`) belong to a runner, and to request item 2.
- An invalidation loop (request `:24`) needs an action that revises an artifact. `protocol/1` cannot declare one.
- Search beyond 65,536 states is request item 3.

**Could not verify.**
- Whether the reachable part of `output.txt:10` (intake) fits the bound. I ran nothing.
- `boundary.md:74`, `:109` and `:161` cite `canon-simulation-request.md:66-81`. That file has 35 lines, so those citations do not resolve.

## Decision needed

How authority and explicit decisions enter the walk. None of the options changes `ess/` beyond `cli.yaml`, or the model.

| Option | What it changes | Cost |
|---|---|---|
| A. Authority and decision steps (as in Outcome) | Paths name grants, denials and decisions | Walk states include capabilities and decisions. A denial that strands a case is found. |
| B. Fixed per walk: each assignment of capabilities and decisions is an initial state | Action steps only | 3^c·2^d initial states. A decision outcome holds from step 0. Paths never show a denial. |
| C. `approval-required` counts as `admissible` | No authority dimension | Finds no stuck state caused by a denial. It hides `approval-required` too. |

Recommendation: A, because every reachable state is then a state of the check's space and denials are findings.

## ESS first

- **First commit.**
  - Adds `canon.cli.Simulate` (wire `simulate`, input `path: String`, outcomes `unreadable`, `rejected` and `nothing-found`) to `ess/domains/cli.yaml`, modelled on `canon.cli.Check` (`cli.yaml:164-198`).
  - Adds the fixture set `fixtures/investigation/simulate/`.
  - Adds the named test.
  - The model does not change.
- **Red on that commit.**
  - `simulate_reports_reachability_findings` fails, because clap refuses `simulate` with exit 64.
  - `every_canon_subcommand_is_declared_in_ess` (`crates/canon-cli/tests/ess_commands_match.rs:458`) fails, naming `simulate` as declared in `ess/` and absent from clap.

## Scope

- **In:** the step relation, the breadth-first walk, the five findings, the two new refusals, the `simulate` subcommand, and the wording for 0 and 1 in `EXIT_STATUSES`. (cited)
- **Out:**
  - order forms (a later story);
  - invalidation loops (they need a language form);
  - an admissible action that changes no claim (request item 2);
  - search beyond the bound (request item 3);
  - authority bypass and properties (`canon check`);
  - running anything. (cited)
- **Surfaces:**
  - `crates/canon/src/simulate/mod.rs` (new) — inferred
  - `crates/canon/src/check/space.rs` (a `pub(crate)` successor that adds one current record of a class and profile to a dimension's value) — inferred
  - `crates/canon/src/lib.rs` (the module line) — cited
  - `crates/canon-cli/src/simulate.rs` (new, laid out like `check.rs:14-48`) — cited
  - `crates/canon-cli/src/lib.rs` (the `Simulate` variant and `EXIT_STATUSES` at `lib.rs:21-43`) — cited
  - `crates/canon-cli/src/main.rs` (the `mod` line and the dispatch arm beside `main.rs:61`) — cited
  - `crates/canon-cli/tests/canon_simulate.rs` (new) — cited
  - `ess/domains/cli.yaml` — cited
  - `fixtures/investigation/simulate/` (new, owned whole) — inferred
- **The coordinator's at integration:** `website/status.yaml`, and the regenerated `website/docs/reference/cli.md` and `website/data/status.json`.
- **Not touched:** `conformance/requirements.yaml`, `ess/domains/protocol.yaml`, `crates/canon/src/model/`.
- **Confidence:** medium. The step relation is read from the check's space. Whether `space.rs` needs more than the successor function is not established.

## Order

- Depends on story:canon-check and story:ess-command-surface. Both are implemented.
- Shares `crates/canon-cli/src/lib.rs`, `main.rs` and `ess/domains/cli.yaml` with story:protocol-imports, story:protocol-floor and story:canon-mutate.
- Shares `crates/canon/src/lib.rs` with story:protocol-floor and story:canon-mutate.
- Shares `crates/canon/src/check/space.rs` with story:revision-ordering (that story's scope line is inferred).
- So it runs in a wave that holds none of those four stories.

## Acceptance

The named test `simulate_reports_reachability_findings` (in `crates/canon-cli/tests/canon_simulate.rs`) passes over `fixtures/investigation/simulate/`:

1. A clean protocol exits 0 and prints only the summary line, with its reachable state count and terminable count per outcome.
2. A protocol with two actions, each needing evidence only the other produces, and an outcome needing both:
   - `canon check` exits 0;
   - `canon simulate` exits 1, printing `stuck-state` for `{}` with an empty path, `dead-action` for both actions, `unreached-outcome` for the outcome and `no-outcome-reachable`.
3. A protocol whose one action requires capability `c` prints `stuck-state` for `` {capability `c` denied} `` after `` capability `c` denied ``. It prints no stuck state for `{}`, where the action is `approval-required`.
4. A protocol in which, after a denial, the one admissible action may produce nothing prints `cycle`, naming that state, its shortest path and the action.
5. A protocol with an order form is refused as `ordered-evidence`, exit 1.
6. Bound behaviour:
   - a protocol with more than 65,536 reachable states is refused as `reachable-state-bound`, exit 1;
   - a protocol whose upstream variants put its space above the bound (`canon check` refuses it as `state-space-bound`) is simulated and exits 0.
7. Two runs give identical bytes, and `every_canon_subcommand_is_declared_in_ess` passes.

## Source

- Request item 4 of `$HOME/beyond10x/sdc/docs/canon-requests.md`. Also read: `docs/canon-simulation-request.md`, `docs/boundary.md`, `docs/driver.md`, `docs/limitations/output.txt`.
- canon tree `15cfece`: `AGENTS.md`, `crates/canon/src/check/{mod,space}.rs`, `generate/mod.rs`, `eval/{actions,outcomes}.rs`, `model/mod.rs`, `ir/mod.rs`, `crates/canon-cli/src/{lib,main,check}.rs`, `ess/domains/cli.yaml`, story:protocol-floor, story:canon-mutate, story:revision-ordering, story:protocol-imports, epic:protocol-analysis.
- 2026-11-20.

## What invalidation loops wait for

§ Scope leaves invalidation loops out because `protocol/1` cannot declare that an action revises
an artifact. Request 10 of the SDC prototype (`docs/plan/wave-plan-2026-11-20.md`, "Requests
received") proposes a `revises: [artifact]` field on an action. That field is the dependency: with
it, a revising step can be added to the walk and a loop of revise, invalidate and re-produce
becomes a finding. Request 10 is a request and no story on 2026-11-20.

## Decided

Option A of § Decision needed: authority steps and decision steps are steps of the walk. Decided on 2026-11-20 by the wave coordinator on the drafter's recommendation, after the operator accepted the story without choosing (`approval-record:accept-sdc-requests-2026-11-20`). The unit builds this option and does not reopen the choice.
