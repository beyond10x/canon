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
| [Three-valued claim evaluation](../reference/evaluation.md) | Shipped | `canon evaluate` over a case snapshot and evidence records, writing a `canon-decision/1` document. |
| [Evaluation documents](../reference/documents.md) | Shipped | `canon-case/1`, `canon-evidence/1` and `canon-decision/1`. |
| [`canon conform run`](../reference/conformance.md) | Shipped | Runs scenarios that compile a fixture and evaluate a case. |
| ESS specification of the data model | Shipped | The model is specified under `ess/` and held to a hard gate; a test fails when the two differ. |
| Evidence bound to revisions | Planned | Evidence applies only to the revision it was observed on. Records name a revision today, but it does not affect the result. |
| Evidence freshness | Planned | Evidence expires against the evaluation instant. |
| Obligations | Planned | Each obligation evaluated as open or discharged. |
| Action admissibility | Planned | From preconditions and authority decisions. |
| Outcomes | Planned | Earned or blocked; no completion through an undeclared outcome; decision-based outcomes after that. |
| Invalidation rules | Planned | An upstream artifact change invalidates dependent claims. |
| Structured explanation | Planned | A deterministic account of why each value is what it is. |
| Semantic diff | Planned | Classify a protocol change as tightening, relaxation, breaking, expansion or no semantic change. |
| Normative conformance suite | Started | The requirements below as runnable scenarios; the first scenario covers `CANON-CLAIM-001` and `-002`. |
| Protocol composition, a shared evidence envelope | Open question | Listed as open in the design; no decision yet. |

## Commands that are not built

The design proposes `canon frontier`, `canon diff`, `canon inspect` and `canon conform synthesize`. None exists in the current binary, and their shape may change. The
[CLI reference](../reference/cli.md) lists only the commands that exist.

## Normative requirements

From the design document. Each is meant to become a conformance scenario. Today one scenario in
`conformance/scenarios` covers `CANON-CLAIM-001` and `CANON-CLAIM-002`; the others have none yet.

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
execution or persistence. Parsing, validation, the compiled form and claim evaluation are done; the
rest is the planned work above. A service, database, UI, scheduling and workflow execution are not part of it.

Read the full
[design proposal](https://github.com/beyond10x/canon/blob/main/docs/design/canon-protocol-calculus-design.md)
for the reasoning.
