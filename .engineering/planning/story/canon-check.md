---
format: aep.planning-md/3
id: story:canon-check
kind: story
status: implemented
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
  path: AGENTS.md
- confidence: cited
  path: crates/canon-cli/src/check.rs
- confidence: cited
  path: crates/canon-cli/src/lib.rs
- confidence: cited
  path: crates/canon-cli/src/main.rs
- confidence: cited
  path: crates/canon-cli/tests/canon_check.rs
- confidence: cited
  path: crates/canon-docs/
- confidence: cited
  path: crates/canon/src/check/
- confidence: cited
  path: crates/canon/src/lib.rs
- confidence: cited
  path: crates/canon/src/model/mod.rs
- confidence: cited
  path: crates/canon/src/model/properties.rs
- confidence: cited
  path: crates/canon/tests/ess_model_matches.rs
- confidence: cited
  path: ess/domains/check.yaml
- confidence: cited
  path: ess/ess-inputs.yaml
- confidence: cited
  path: ess/system.yaml
- confidence: cited
  path: fixtures/investigation/check/
- confidence: cited
  path: website/
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T07:10:25Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-10-04T07:10:25Z", actor: "human:timo", revision: 7}
- {from: "active", to: "implemented", at: "2026-10-04T08:26:15Z", actor: "human:timo", revision: 15, decided_on: {"recorded":{"test_result":1,"review_outcome":3,"verification":1}}}
---
## Outcome

`canon check <protocol>` (clap derive) enumerates the protocol's finite state space — which evidence
kinds are present and with which result values, each obligation's status, each authority decision,
and each explicit decision an outcome may require (`canon-decisions/1`, story:decision-outcomes) —
evaluates every state with Canon's own evaluator, and reports, deterministically:

- outcomes no state reaches;
- actions whose precondition holds in no state;
- claims, obligations, action preconditions and outcome requirements that read, directly or
  through the claims they test, an evidence kind no action's `may_produce` lists;
- outcomes legitimate without authority (an authority bypass): the requirement reads, in a position
  where its presence helps (under an even number of `not`, `is: false` or `is: unknown`), evidence
  an authority-requiring action may produce, and the outcome is legitimate in a state with no
  authority decision and only evidence some action without a capability may produce, where removing
  one positively read dimension alone blocks it, such as an alternative path beside the governed
  one; an outcome that holds only because governed evidence is absent is not one. Not reported
  (later decision): an outcome resting on `not {evidence g result r}` where g has a producer that
  needs no authority;
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

## First step: decisions taken (wave 2026-10-04-w10, phase 1)

## First step: settled (wave 2026-10-04-w10, phase 1)

Both open items are settled outside `protocol/1`; `ess/` is unchanged, as `## ESS first` planned.

1. **Property declarations: `canon-properties/1`**, a document of its own passed as
   `canon check --path <protocol> --properties <file>`:

   ```yaml
   format: canon-properties/1
   protocol: investigation          # must equal the protocol's id
   properties:
     <property-id>:
       description: <text, optional>
       subject:                     # exactly one of
         action: <action-id>        #   its status: admissible | approval-required | blocked
         outcome: <outcome-id>      #   its status: legitimate | blocked
       independent_of:
         claim: <claim-id>
   ```

   One property form, independence: the subject's status is the same in every two states that
   agree on every dimension except the evidence kinds the claim reads (directly or through claims
   it tests). The walkthrough example is `subject: {action: emergency.leave}`,
   `independent_of: {claim: cause.identified}`. Refused: another format, another protocol id, an
   undeclared action, outcome or claim. A failure reports the pair of states as the counterexample.

2. **State space and its bound.** Dimensions, in this order: each declared evidence kind in
   identifier order; each capability some action requires, in identifier order; each decision name
   some outcome requires, in identifier order. Values: an evidence kind takes every subset of its
   classes (each distinct result the protocol's predicates match for that kind, plus "a record with
   no result", which stands for every other result), so 2^(results+1) values; a capability is
   undecided, granted or denied (3); a decision is not taken or taken (2). The space is atemporal:
   records carry no `observed_at`, so nothing expires. Obligation status is computed per state, not
   enumerated (protocol/1 predicates cannot read an obligation). Bound: 65 536 states (2^16). Above
   it `canon check` refuses, exit 1:
   `error[state-space-bound]: protocol `<id>` revision <n> has <N> states, more than the bound of
   65536: <each dimension and its value count, in dimension order>`.

Reports, stdout, exit 0 when there are no findings and 1 when there are, groups in this order,
each in identifier order, then a summary `checked: protocol `<id>` revision <n>: <N> states,
<P> properties, <F> findings`:

- `unreachable-outcome`: an outcome legitimate in no state.
- `unsatisfiable-precondition`: an action whose precondition is `true` in no state.
- `unproduced-evidence`: a claim reading (directly or through claims) an evidence kind no action's
  `may_produce` lists, one line per claim and kind.
- `authority-bypass`: an outcome that reads evidence of a kind some capability-requiring action
  may produce, and is legitimate in a state whose every present evidence kind is produced by some
  action requiring no capability. Static: action order and preconditions are not followed.
- `property-failed`: as above.

A witness state is the one with the fewest present classes, decided capabilities and taken
decisions, ties broken by its rendering in code-point order. Rendering: `{}` or
`{item, item}` with items `evidence `k``, `evidence `k` result `r``, `capability `c` granted|denied`,
`decision `d` taken`, in dimension order; absent, undecided and untaken are omitted.

Scope correction for phase 2: the `Check` subcommand variant lives in
`crates/canon-cli/src/lib.rs` (the clap definition `canon-docs` walks), not `main.rs`, and
`crates/canon/src/lib.rs` needs `pub mod check;`. Neither file is in this story's scope.

- Coordinator: `canon-properties/1` is a new input document, so it is declared in `ess/` (its own
  domain file) with a model type under `crates/canon/src/model/`, before the implementation.
