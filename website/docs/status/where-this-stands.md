---
title: Where this stands
description: What Canon can do today, what is planned, and the normative requirements it is built against.
---

# Where this stands

Canon is in bootstrap. One library crate, `b10x-canon`, and one command-line crate, `canon-cli`,
hold everything marked shipped below. Nothing has been released; build from source.

| Capability | Status | What it means |
|---|---|---|
| [`protocol/1` source model and parser](../reference/protocol.md) | Shipped | Artifacts, evidence kinds, claims, obligations, actions and outcomes, read strictly. |
| [Validation](../reference/validation.md) | Shipped | Format, identifiers, duplicates, unresolved references and claim cycles, in a stable order. |
| [`canon-ir/1` compilation](../reference/canon-ir.md) | Shipped | Canonical ordering, explicit defaults, one serialization suitable for hashing. |
| [Three-valued claim evaluation](../reference/evaluation.md) | Shipped | `canon evaluate` over a case snapshot and evidence records, writing a `canon-decision/1` document; scenario `CANON-CLAIM-001` (covering `CANON-CLAIM-001` and `-002`). |
| [Evaluation documents](../reference/documents.md) | Shipped | `canon-case/1`, `canon-evidence/1`, `canon-decisions/1` and `canon-decision/1`, and the `canon-properties/1` document `canon check` reads. |
| [`canon conform run`](../reference/conformance.md) | Shipped | Runs scenarios that compile a fixture and evaluate a case, with authority decisions, explicit decisions and an evaluation instant when a step gives them. |
| ESS specification of the data model | Shipped | The model is specified under `ess/` and held to a hard gate; a test fails when the two differ. |
| [Evidence bound to revisions](../reference/evaluation.md#refusals) | Shipped | Evidence applies only to the current revision of the artifact it names; a record bound to another revision is listed as excluded under each claim that reaches its kind (through a match that names a subject, only when it is about that artifact), and one about an undeclared artifact is refused; scenario `CANON-EVIDENCE-001`. |
| [Subject-bound evidence matches](../concepts/evidence-and-revisions.md#evidence-about-one-artifact) | Shipped | An evidence match may name the declared artifact its record must be about (`subject`); a record about another artifact, even at its current revision, does not match it and is not listed as excluded under it, while a match without a subject reads records about any artifact; scenario `CANON-EVIDENCE-003`. |
| [Evidence freshness](../reference/evaluation.md#refusals) | Shipped | An evidence kind may declare a `max_age`; at the instant given with `--at`, a record older than that is listed as excluded (`expired`), never counted as `FALSE`. Without `--at` nothing expires; scenario `CANON-EVIDENCE-002`. |
| [Obligations](../reference/evaluation.md#sections) | Shipped | Each declared obligation is `open` or `discharged` by its discharge predicate over the claim values; scenario `CANON-OBLIGATION-001`. |
| [Action admissibility](../reference/evaluation.md#sections) | Shipped | Each declared action is `admissible`, `approval-required` or `blocked`, from its precondition and the authority decisions given with `--authority`, with the reasons; scenario `CANON-AUTHORITY-001`. |
| [Outcomes](../reference/evaluation.md#sections) | Shipped | Each declared outcome is `legitimate` or `blocked`, with the reasons; a case that terminates through an undeclared or a blocked outcome is refused; scenario `CANON-OUTCOME-001`. |
| [Decision-based outcomes](../reference/evaluation.md#explicit-decisions) | Shipped | An outcome may require an explicit decision (`requires: decision: <name>`) instead of a predicate. It is `legitimate` only with a `canon-decisions/1` decision of that name, for that outcome, taken at the case snapshot's `revision` (given with `--decisions`); a decision taken at a superseded case revision does not apply; scenario `CANON-OUTCOME-002`. |
| [`canon check`](../reference/cli.md#canon-check) | Shipped | Evaluates a protocol in every state of its finite state space and reports outcomes no state reaches, actions whose precondition holds in no state, claims, obligations, preconditions and outcome requirements that read evidence no action produces, outcomes that rest on evidence an authority-requiring action may produce yet hold without any authority decision, and each [`canon-properties/1`](../reference/documents.md) property that fails, with a counterexample; a space of more than 65 536 states is refused. |
| Invalidation rules | Planned | An upstream artifact change invalidates dependent claims. |
| [Structured explanation](../reference/evaluation.md#sections) | Shipped | Every decision carries an `explanation`: what it was computed from (protocol revision, Canon version, case snapshot, evidence set, authority and explicit decisions, evaluation instant), and why each claim that is not `TRUE`, open obligation, action that is not admissible and blocked outcome has its status, down to each evidence record that applied or was excluded and why. The same inputs in any evidence order give the same bytes; scenario `CANON-EXPLAIN-001` (also covering `CANON-DETERMINISM-001`). |
| Semantic diff | Planned | Classify a protocol change as tightening, relaxation, breaking, expansion or no semantic change. |
| Normative conformance suite | Started | The requirements below as runnable scenarios; scenarios cover `CANON-CLAIM-001`, `CANON-CLAIM-002`, `CANON-EVIDENCE-001`, `CANON-EVIDENCE-002`, `CANON-EVIDENCE-003`, `CANON-OBLIGATION-001`, `CANON-AUTHORITY-001`, `CANON-OUTCOME-001`, `CANON-OUTCOME-002`, `CANON-EXPLAIN-001` and `CANON-DETERMINISM-001`. |
| Protocol composition, a shared evidence envelope | Open question | Listed as open in the design; no decision yet. |

## Commands that are not built

The design proposes `canon frontier`, `canon diff`, `canon inspect` and `canon conform synthesize`. None exists in the current binary, and their shape may change. The
[CLI reference](../reference/cli.md) lists only the commands that exist.

## Normative requirements

From the design document. Each is meant to become a conformance scenario. Today the scenarios in
`conformance/scenarios` cover `CANON-CLAIM-001`, `CANON-CLAIM-002`, `CANON-EVIDENCE-001`,
`CANON-EVIDENCE-002`, `CANON-AUTHORITY-001`, `CANON-OUTCOME-001` and `CANON-DETERMINISM-001`;
`CANON-INDEPENDENCE-001` and `CANON-INVALIDATION-001` have none yet. Four more scenarios hold what
the design lists no requirement for: `CANON-OBLIGATION-001`, obligations, `CANON-OUTCOME-002`,
outcomes that require an explicit decision, `CANON-EVIDENCE-003`, evidence matches bound to a
subject, and `CANON-EXPLAIN-001`, the structured explanation, which also covers
`CANON-DETERMINISM-001`.

| Requirement | Statement |
|---|---|
| `CANON-CLAIM-001` | A required positive predicate evaluating to `UNKNOWN` must not be treated as satisfied. |
| `CANON-CLAIM-002` | A required positive predicate evaluating to `FALSE` must not be treated as satisfied. |
| `CANON-EVIDENCE-001` | Evidence bound to a subject revision must not establish a current-revision claim after the subject revision changes. |
| `CANON-EVIDENCE-002` | Expired evidence must become inapplicable or `UNKNOWN`; expiration alone must not imply `FALSE`. |
| `CANON-AUTHORITY-001` | An action or transition with an unsatisfied authority requirement must not be treated as admissible. |
| `CANON-INDEPENDENCE-001` | Evidence failing a declared independence relation must not satisfy a requirement that demands such independence. |
| `CANON-OUTCOME-001` | A case must terminate only through an outcome declared by its governing protocol. |
| `CANON-INVALIDATION-001` | Changing a bound upstream artifact must invalidate dependent claims whose only support was bound to the previous revision. |
| `CANON-DETERMINISM-001` | Equivalent normalized protocol, case, evidence, authority and evaluation-time inputs must produce equivalent normalized outputs. |

## The first milestone

The design sets one goal for the first milestone: a protocol can deterministically derive a useful
frontier and legitimate outcomes from a live case, evidence, authority and time, without owning
execution or persistence. Parsing, validation, the compiled form, claim evaluation, revision
binding, subject-bound evidence matches, freshness, obligations, action admissibility, outcomes,
decision-based outcomes and the structured explanation are done; the rest is the planned work above.
A service, database, UI, scheduling and workflow execution are not part of it.

Read the full
[design proposal](https://github.com/beyond10x/canon/blob/main/docs/design/canon-protocol-calculus-design.md)
for the reasoning.
