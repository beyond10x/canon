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
Evidence is meant to be bound to the revision of an artifact it was observed on; see
[evidence bound to revisions](./evidence-and-revisions.md).

## Evidence kinds

An evidence kind names a class of observation the protocol admits. Claims match evidence by kind,
and actions say which kinds they may produce. Both must name a declared kind.

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
  establish.support:
    discharged_when:
      claim: explanation.supported
```

:::caution[Planned: evaluating obligations]

The validator checks every discharge predicate today. Evaluating each obligation as open or
discharged is not built yet.

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
who holds a capability, and it gives effect classes no built-in meaning today. Deciding whether an
action is admissible is planned.

## Outcomes

An outcome is a declared terminal interpretation of a case, such as `supported`, `restored` or
`abandoned`, together with the predicate it requires. A case is meant to end only through an
outcome its protocol declares. Evaluating outcomes is planned.

## Predicates

Claims, preconditions and outcomes are written as predicates: a small, total expression language
with five forms, `all`, `any`, `not`, an `evidence` match and a `claim` test. There are no
variables, loops or function calls.

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

:::note[Shipped: how three values combine]

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
