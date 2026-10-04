---
format: aep.planning-md/3
id: story:canon-check
kind: story
status: draft
title: Check a protocol exhaustively with canon check
relations:
- decomposes: epic:protocol-analysis
- depends_on: story:three-valued-claims
- depends_on: story:obligations
- depends_on: story:action-admissibility
- depends_on: story:outcomes
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:decision-outcomes
scope:
- confidence: cited
  path: crates/canon-cli/src/check.rs
- confidence: cited
  path: crates/canon-cli/src/main.rs
- confidence: cited
  path: crates/canon-cli/tests/canon_check.rs
- confidence: cited
  path: crates/canon/src/check/
- confidence: cited
  path: fixtures/investigation/check/
revision: 5
---
## Outcome

`canon check <protocol>` (clap derive) enumerates the protocol's finite state space — which evidence
kinds are present and with which result values, each obligation's status, each authority decision,
and each explicit decision an outcome may require (`canon-decisions/1`, story:decision-outcomes) —
evaluates every state with Canon's own evaluator, and reports, deterministically:

- outcomes no state reaches;
- actions whose precondition holds in no state;
- claims that need an evidence kind no action's `may_produce` lists;
- outcomes reachable without any action that requires authority (an authority bypass);
- each declared property that fails, with one counterexample state.

It reads no clock and does no IO beyond its inputs. Part 1 of epic:protocol-analysis.

## First step: the two open items

The epic leaves two items open, and this story settles both before its ESS-first commit, recording
the choice in this body:

1. **Property declarations.** The format for declaring properties beside a protocol. The example to
   express is "`emergency.leave` admissibility is independent of `cause.identified`" (Atlas
   `docs/design/governed-autonomy/incident-response-walkthrough.md` § 1). Planned as its own
   document read by `crates/canon/src/check/`, leaving `protocol/1` unchanged, as
   `canon-authority/1` is (story:action-admissibility § ESS first). If the first step finds the
   properties must live in `protocol/1`, the story widens to `ess/` and `crates/canon/src/model/` and
   is re-scoped before it is proposed.
2. **State-space bound.** The size above which `canon check` refuses rather than runs, and how the
   refusal names the dimensions that made the space large.

## ESS first

- Specification change: none in `ess/` while the property document stays beside `protocol/1` (see
  the first step). The first commit is the named test below with its fixtures under
  `fixtures/investigation/check/`, each carrying the report `canon check` must print.
- Red on that commit: `canon_check_reports_investigation_defects` fails, because `canon check` does
  not exist.

## Scope

- In: the state-space enumerator, the five reports, the property document and its validation, the
  state-space bound and its refusal.
- Out: scenario generation (story:scenario-generation) and mutation (story:canon-mutate); time and
  freshness (the state space is atemporal: an expired record is an absent one).
- Surfaces: `crates/canon/src/check/` (new), `crates/canon-cli/src/check.rs` (new),
  `crates/canon-cli/src/main.rs` (the subcommand variant), `fixtures/investigation/check/` (new),
  `crates/canon-cli/tests/canon_check.rs` (new).

## Order

- depends_on story:three-valued-claims (claims are evaluated per state), story:obligations and
  story:action-admissibility (the obligation and authority dimensions of the state space),
  story:outcomes (reachability is outcome evaluation) and story:decision-outcomes (an outcome that
  requires an explicit decision would otherwise be reported unreachable). All come after
  story:evaluator-skeleton.
- It adds no file under `crates/canon/src/eval/`: it calls the evaluator, it does not change it.

## Acceptance

The named test `canon_check_reports_investigation_defects` (in
`crates/canon-cli/tests/canon_check.rs`) passes with seven expectations, each over one fixture in
`fixtures/investigation/check/`: the base investigation protocol reports nothing; a variant with an
outcome whose requirement contradicts itself reports that outcome unreachable; a variant with an
unsatisfiable precondition reports that action; a variant with a claim over an evidence kind no
action produces reports that claim and kind; a variant whose outcome is reachable without any
authority-requiring action reports the bypass; a variant with a declared property that fails
reports it with a counterexample state; and a protocol whose state space exceeds the bound is
refused naming the bound. Two runs give identical bytes.

## Source

epic:protocol-analysis part 1 and § Open; Atlas
`docs/design/governed-autonomy/incident-response-walkthrough.md` § 1; Atlas ADR 0080.
