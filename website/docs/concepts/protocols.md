---
title: The protocol/1 language
sidebar_label: The protocol/1 language
description: How a Canon protocol is put together, section by section.
status: shipped
lede: A protocol is a YAML document. It names itself with an id and a revision, then declares six sections.
source: crates/canon/src/model/, the protocol/1 source model
source_url: https://github.com/beyond10x/canon/tree/main/crates/canon/src/model
---

# The `protocol/1` language

Each section is a map from an identifier to a declaration. This page explains what the
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
has (`effect`), which evidence kinds it may produce (`may_produce`) and what arguments it takes
(`arguments`, [below](#argument-schemas)). A person, planner, agent or
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
`canon-authority/1` list of capabilities granted, granted only with approval or denied, each at a
scope or at every scope (`canon evaluate --authority`; shipped, scenario `CANON-SCOPE-001`). It gives
effect classes no built-in meaning today.

:::shipped[Action admissibility]

`canon evaluate` reports every declared action as `blocked` when its precondition is not `TRUE`
or a capability it requires is denied, `approval-required` when the precondition is `TRUE`,
nothing is denied and some required capability is not decided, and `admissible` when the
precondition is `TRUE` and every required capability is granted. A status other than
`admissible` comes with the claims, evidence or capabilities that decide it. Conformance scenario
`CANON-AUTHORITY-001` holds this.

:::

### Argument schemas

An action may declare one argument schema, `arguments`: a JSON Schema saying what arguments the
action is called with. A runner reads it from the protocol it loads and validates each call's
arguments against it. Absent: the action takes no arguments; a runner accepts only `{}`. Canon
evaluates no call, but its library function `b10x_canon::validate::check_arguments` checks one
call's arguments against the schema, reporting every refusal, and a runner may use it or any
2020-12 validator.

```yaml
actions:
  inspect:
    arguments:
      type: object
      properties:
        source: {type: string, minLength: 1}
        limit: {type: integer, minimum: 1, maximum: 50}
      required: [source]
      additionalProperties: false
```

The schema is written in Canon's argument-schema dialect, a subset of JSON Schema 2020-12. Every
schema in it is a valid 2020-12 schema with the same meaning, so a runner can hand it to any
2020-12 validator. Protocols load offline, so the dialect has no reference at all.

- Every schema, the top level and each nested one, names exactly one `type`: `object`, `array`,
  `string`, `integer`, `number`, `boolean` or `null`. The top level is `type: object`.
- `title` and `description`, strings, anywhere.
- `enum`, on any type: a non-empty list of distinct JSON scalars, each of the schema's type.
- `object`: `properties` (a map of name to schema), `required` (a list of distinct names, each
  declared in `properties`), `additionalProperties` (a boolean only).
- `array`: `items` (one schema), `minItems`, `maxItems`.
- `string`: `minLength`, `maxLength`.
- `integer` and `number`: `minimum`, `maximum`, integers.
- Every length and count bound is a non-negative integer, and a minimum is at most its maximum.
- Schemas nest at most 32 deep: the top level is one level, and each schema in `properties` or `items` one more.

Anything else is refused: `$ref`, `$defs`, `$schema`, `$id`, `allOf`, `anyOf`, `oneOf`, `not`,
`if`/`then`/`else`, `patternProperties`, `pattern`, `format`, `const`, `exclusiveMinimum`,
`exclusiveMaximum`, a list-valued `type`, a schema-valued `additionalProperties`, a keyword on a
type it does not apply to, and any unknown keyword. `canon validate` reports each offending keyword
as its own `action-arguments` problem, naming the action, the keyword's JSON Pointer inside the
schema and why it is refused:

```text
action `inspect` arguments at `/properties/source/pattern`: `pattern` is outside the dialect
```

`canon compile` carries each schema into `canon-ir/1` as declared, its object keys sorted like
every object's, and writes no `arguments` for an action without one.

Every change to an action's `arguments` is made under a new `protocol.revision`, like every other
change to a protocol. `canon diff` tells a runner what the change does to calls:

| Change | Category | Line |
|---|---|---|
| `arguments` introduced on an action (it took only `{}`) | `BREAKING` | `arguments introduced` |
| `arguments` removed from an action (it now takes only `{}`) | `BREAKING` | `arguments removed` |
| only `title` or `description` changed, anywhere in the schema | `NO SEMANTIC CHANGE` | `arguments rewritten` |
| the new schema accepts only argument documents the old one accepted | `TIGHTENING` | `arguments narrowed` |
| the old schema accepts only argument documents the new one accepts | `RELAXATION` | `arguments widened` |
| anything else, or both | `BREAKING` | `arguments changed` |

Narrowing and widening are decided structurally and soundly, never guessed. Per keyword: `integer`
to `number` widens and the reverse narrows, and any other type change is changed; an `enum`
introduced or a value removed narrows, and removed or a value added widens; a lower bound
introduced or raised, or an upper bound introduced or lowered, narrows, and the reverse widens; a
name added to `required` narrows, and one removed widens. For an object, each property name is
compared by its effective schema: the declared one, else any value when `additionalProperties` is
absent or true and no value when it is false; `items` and each property recurse. The directions of
all keywords are combined: none is `rewritten`, only narrowing is `narrowed`, only widening is
`widened`, both is `changed`.

:::shipped[Action argument schemas]

Shipped in Canon 0.5.0 (story:action-arguments): `arguments` on an action, its dialect check in
`canon validate`, its carry into `canon-ir/1` and its classification by `canon diff`.

:::

### Judgement marks

Three optional fields mark where a protocol's results depend on a judgement, by a person or a
model, rather than on a reproducible procedure:

- `author` on an action, `program` or `judgement`; absent means `program`. `judgement` says that
  the action's arguments, and the content of the artifacts it revises, are not a function of its
  inputs.
- `revises` on an action: the declared artifacts it may move to a new revision; absent means none.
  `canon validate` refuses an artifact that is not declared as `undeclared-artifact`, naming the
  action.
- `decided_by` on each `may_produce` entry, `program` or `judgement`: who decides the record's
  result; absent means the action's `author`. `decided_by: program` on a judgement action covers a
  record the runner writes from the effect it applied, such as provenance.

```yaml
actions:
  draft_spec:
    author: judgement
    effect: write
    revises: [spec]
    may_produce:
      - evidence: spec_source
        decided_by: program
      - evidence: spec_rationale
```

Canon gives the fields no evaluation meaning: `canon evaluate`, `canon check` and `canon generate`
print the same with or without them. They name no executor, model or tool, and Canon cannot prove
that a runner honours them; it never refuses a runner that moves an artifact outside `revises`.
`canon compile` writes `"author": "judgement"` only on a judgement action, `"revises"` only when
it lists an artifact, sorted, each once, and `"decided_by": "judgement"` only on a production a
judgement decides, so `author: program` written out compiles to the same bytes as no `author`.
`canon diff` names the action in each line:

| Change | Category | Line |
|---|---|---|
| `author` from `program` to `judgement` | `TIGHTENING` | ``author changed from program to judgement`` |
| `author` from `judgement` to `program` | `RELAXATION` | ``author changed from judgement to program`` |
| a kind it may produce becomes judgement-decided | `TIGHTENING` | ``now decides evidence `k` by judgement`` |
| a kind it may produce becomes program-decided | `RELAXATION` | ``now decides evidence `k` by program`` |
| an artifact added to `revises` | `RELAXATION` | ``now revises artifact `a` `` |
| an artifact removed from `revises` | `TIGHTENING` | ``no longer revises artifact `a` `` |

`canon-ir/1` holds who decides each production after resolving it against the action's `author`,
so a change of `author` also prints a `now decides evidence` line for each kind of that action that
declares no `decided_by` and is produced in both revisions, in kind order, after the `author` line
and any kind added or removed. To get only the `author` line, write the old value as `decided_by`
on each production of the action.

:::shipped[Judgement marks]

On `main` (story:judgement-marking), not yet in a release: the three fields in `protocol/1`,
`revises` checked by `canon validate`, carried into `canon-ir/1` and classified by `canon diff`.
No check reads them yet; checks that do are planned.

:::

### Ordering guards

`runs_only_when` on an action asserts when it may run: the action is `admissible` or
`approval-required` only in a state where this predicate is `TRUE`. It is written like a
precondition, and `canon validate` checks its references the same way, naming it (an undeclared
claim is `` action `report.open` runs_only_when references claim `x`, which is not declared ``).
`all: []`, which is true in every state, asserts nothing and is refused as
`vacuous-runs-only-when`.

```yaml
actions:
  report.open:
    precondition:
      all:
        - {claim: store.committed}
        - {claim: report.opened, is: unknown}
    runs_only_when: {claim: store.committed}
```

The field is an assertion, not a condition: `canon evaluate`, `canon simulate` and `canon generate`
never read it, and give the same output with or without it. `canon check` proves it in every state
and reports each broken guard, after every other finding, with the state of least weight in which
the action runs while the guard is not `TRUE`:

```text
ordering-guard-failed: action `publish_finding` is approval-required in state {evidence `falsification_attempt`}, where its runs_only_when is not true
```

`UNKNOWN` is not `TRUE`, so an action that can run while the guard is unknown breaks it. An action
that is `blocked`, by its precondition or by a denied capability, does not run.

The check proves the guard per state, not along an order of actions. Every combination of records
is a state, whether or not a sequence of actions reaches it, so a guard that holds only because
the actions that ran earlier never produced some record is reported as broken. Name the guard in
the action's own precondition, as above, and the check proves it.

`canon compile` writes `"runs_only_when"` only for an action that declares it, so the `canon-ir/1`
of every protocol without it is unchanged. `canon diff` reports the guard introduced, removed or
changed as `NO SEMANTIC CHANGE`, naming the action. A Canon release before the field existed
refuses an IR that holds it.

:::shipped[Ordering guards]

On `main` (story:ordering-guards-provable-by-check), not yet in a release: `runs_only_when` in
`protocol/1` and `canon-ir/1`, its references checked by `canon validate`, and its proof by
`canon check` (`ordering-guard-failed`).

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

`canon validate` checks the format, identifiers, duplicates, argument schemas, unresolved
references and claim cycles, and reports every problem in a stable order ([validation](../reference/validation.md)). A
valid document compiles into [`canon-ir/1`](../reference/canon-ir.md): declarations sorted by
identifier, defaults written out, authoring shortcuts removed, so that documents meaning the same
thing compile to the same bytes.
