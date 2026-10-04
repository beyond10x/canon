---
title: The protocol/1 language
sidebar_label: The protocol/1 language
description: How a Canon protocol is put together, section by section.
---

# The `protocol/1` language

A protocol is a YAML document. It names itself with an id and a revision, then declares six
sections. Each section is a map from an identifier to a declaration. This page explains what the
sections are for; the [`protocol/1` reference](../reference/protocol.md) lists every key, generated
from Canon's model.

```yaml
format: protocol/1
protocol:
  id: investigation
  revision: 1
artifacts: {}       # what evidence is about
evidence_kinds: {}  # what can be observed
claims: {}          # propositions, established by evidence
obligations: {}     # what is owed
actions: {}         # what may be done
outcomes: {}        # how a case may legitimately end
```

Canon reads a document strictly. A key it does not know is refused. A key written with no value is
refused too, so a forgotten value never silently becomes a default: to take a default, leave the
key out.

## Artifacts

An artifact is a thing that has revisions, such as an explanation, a design or a deployment.
Evidence is bound to the revision of an artifact it was observed on; see
[evidence bound to revisions](./evidence-and-revisions.md).

## Evidence kinds

An evidence kind names a class of observation the protocol admits. Claims match evidence by kind,
and actions say which kinds they may produce. Both must name a declared kind.

An evidence match may also name the artifact its record must be about, with `subject`. Only
records about that artifact then count; a record about another declared artifact does not
match, even at that artifact's current revision. A match without `subject` counts records about
any declared artifact. A `subject` must name a declared artifact.

```yaml
claims:
  explanation.survived:
    true_when:
      evidence: {kind: falsification_attempt, result: survived, subject: explanation}
```

:::shipped[Subject-bound evidence matches]

Conformance scenario `CANON-EVIDENCE-003` holds this; see
[evidence bound to revisions](./evidence-and-revisions.md#evidence-about-one-artifact).

:::

## Claims

A claim is a proposition whose value is `TRUE`, `FALSE` or `UNKNOWN`
([three-valued truth](./three-valued-truth.md)). Its predicate, `true_when`, says what evidence and
which other claims establish it. Claims must not test each other in a cycle.

## Obligations

An obligation is something that must be done before the case can be complete. Its
`discharged_when` predicate says when it is done; it may test only claim values, never evidence
directly.

```yaml
obligations:
  establish.explanation:
    discharged_when:
      claim: explanation.supported
```

:::shipped[Open or discharged]

`canon evaluate` reports every declared obligation as `open` or `discharged`. It is discharged
only when the predicate is `TRUE`; `UNKNOWN` and `FALSE` both leave it open. Whether deciding a
claim or establishing it discharges an obligation is the author's choice, written with the claim
tests of the predicate: `{claim: c}` needs `c` to be `TRUE`, and `{not: {claim: c, is: unknown}}`
needs it decided either way.

:::

## Actions

An action is a semantically named possibility, not a workflow step. It says when it applies
(`precondition`), which capabilities it needs authority for (`requires`), what class of effect it
has (`effect`) and which evidence kinds it may produce (`may_produce`). A person, planner, agent or
workflow engine decides how to carry it out.

```yaml
actions:
  rollback.recent_release:
    precondition:
      claim: impact.mitigated
      is: false
    requires:
      - capability: production.rollback
    effect: reversible_change
    may_produce:
      - evidence: rollback_record
```

Canon checks that capabilities and effect classes are well-formed identifiers. It does not resolve
who holds a capability: whoever runs the evaluation passes the authority decisions in, as a
`canon-authority/1` list of capabilities granted or denied (`canon evaluate --authority`). It gives
effect classes no built-in meaning today.

:::shipped[Action admissibility]

`canon evaluate` reports every declared action as `blocked` when its precondition is not `TRUE`
or a capability it requires is denied, `approval-required` when the precondition is `TRUE`,
nothing is denied and some required capability is not decided, and `admissible` when the
precondition is `TRUE` and every required capability is granted. A status other than
`admissible` comes with the claims, evidence or capabilities that decide it. Conformance scenario
`CANON-AUTHORITY-001` holds this.

:::

## Outcomes

An outcome is a declared terminal interpretation of a case, such as `supported`, `restored` or
`abandoned`, together with what it requires: a predicate, or an explicit decision. A case ends only
through an outcome its protocol declares.

:::shipped[Legitimate or blocked]

`canon evaluate` reports every declared outcome as `legitimate` when its requirement is `TRUE`
and `blocked`, with the reasons, otherwise. A case snapshot that records termination through an
outcome the protocol does not declare is refused (`undeclared-outcome`), and so is one through a
blocked outcome (`illegitimate-termination`). Conformance scenario `CANON-OUTCOME-001` holds this.

:::

:::shipped[Outcomes that require an explicit decision]

An outcome may require an explicit decision instead of a predicate:

```yaml
outcomes:
  inconclusive:
    requires:
      decision: explicitly_inconclusive
```

It is `legitimate` only when `canon evaluate --decisions` is given a `canon-decisions/1` decision
of that name, for that outcome, taken at the case snapshot's `revision`. A decision taken at an
earlier case revision has been superseded and does not apply; the outcome is then `blocked`, naming
the decision. A legitimate decided outcome records who decided: the decision and every principal
whose decision applied. A decision for an outcome that does not require it, or the same decision
given twice, is refused. Conformance scenario `CANON-OUTCOME-002` holds this.

:::

## Predicates

Claims, discharge predicates, preconditions and outcome requirements are written as predicates: a
small, total expression language with five forms, `all`, `any`, `not`, an `evidence` match and a
`claim` test. There are no variables, loops or function calls. An outcome requirement may instead
be a decision requirement, `requires: decision: <name>`: it is not a predicate form, so it stands
alone as the whole requirement and cannot appear inside `all`, `any` or `not`, or in a claim,
obligation or action.

```yaml
true_when:
  any:
    - claim: explanation.supported
    - all:
        - evidence: {kind: falsification_attempt, result: survived}
        - not:
            claim: explanation.refuted
            is: unknown
```

:::shipped[How three values combine]

`all` is `false` when a member is `false`, `true` when every member is `true`, and `unknown`
otherwise; `any` is the mirror image; `not` swaps `true` and `false` and keeps `unknown`. The
[evaluation reference](../reference/evaluation.md) gives every rule, including evidence matches.

:::

## Validation and the compiled form

`canon validate` checks the format, identifiers, duplicates, unresolved references and claim
cycles, and reports every problem in a stable order ([validation](../reference/validation.md)). A
valid document compiles into [`canon-ir/1`](../reference/canon-ir.md): declarations sorted by
identifier, defaults written out, authoring shortcuts removed, so that documents meaning the same
thing compile to the same bytes.
