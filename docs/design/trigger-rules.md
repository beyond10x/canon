# Trigger rules: when the world starts a protocol

**Status:** Draft (epic:trigger-rules)
**Document format:** `canon-trigger/1` (one rule)
**Evaluation inputs:** `canon-trigger-snapshot/1`, `canon-trigger-history/1`
**Evaluation output:** `canon-trigger-plan/1`
**Date:** 2026-11-26

## 1. Problem

A protocol says what a case does once it is open. Nothing in `protocol/1` says when the world opens
one. Consumers that want "entities of kind K in state S where X holds start protocol P" each write
their own rule file and their own matching, and none of them can test a rule set without a live
engine.

The owner decided (2026-11-26) that the rule declaration and its semantics live in Canon, so a rule
set can be simulated, and that the evaluator that acts on it lives in the consumer's daemon
(`beyond10x/runbook`). Canon owns the format, its checks and one pure operation; it starts nothing.

## 2. Boundary

- Canon: the `canon-trigger/1` document, its validation, the publication check over a rule set,
  and `evaluate(rule_set, snapshot, protocols, history, now) -> plan`.
- Not Canon: building a snapshot from a source, storing firings or approvals, deciding who may
  approve, submitting a run, the clock. The evaluator supplies every one of them as an argument.
- The kernel holds the rules of `AGENTS.md`: a pure function of its arguments, no clock, network,
  filesystem or randomness inside it; the same normalized input gives the same bytes; `UNKNOWN` is
  not `FALSE`.

## 3. The rule document, `canon-trigger/1`

One rule per document, a separate top-level document beside `protocol/1` (answer to open question
1). A rule is deployment policy (owner, mode, cap) about a protocol; a protocol file stays one
protocol's behaviour, and one protocol may have many rules.

```yaml
format: canon-trigger/1
id: stale-invoice-reminder
version: 3
owner: billing-operations
subject:
  kind: invoice
  state: {field: status, is: overdue}
where: '$row.amount > 100 and not contains(["test", "internal"], $row.segment)'
start:
  protocol: send-reminder
  input:
    invoice: {field: id}
    channel: {const: email}
mode: auto            # offer | auto
access: write         # read_only | write
refire: once_per_state_entry   # once_per_state_entry | once_per_subject
cap: 50               # starts per rule per UTC day
```

| Field | Rule |
|---|---|
| `id`, `version` | `id` is a Canon identifier; `version` a positive integer. The pair is the rule key, written `<id>@<version>`. |
| `owner` | required, non-empty. Canon does not resolve it. |
| `subject.kind` | required; a kind name the snapshot uses. |
| `subject.state` | required; `field` names a row field, `is` a string value it must equal. |
| `where` | optional; one `canon-expr/1` expression of type Bool (§ 4). Absent: `true`. |
| `start.protocol` | required; a protocol id. |
| `start.input` | map from a case input of P to `{field: <row field>}` or `{const: <string>}`; nothing else. |
| `mode` | required; `offer` or `auto`. |
| `access` | required; `read_only` or `write`. |
| `refire` | required; `once_per_state_entry` or `once_per_subject`. |
| `cap` | required; a positive integer. |

Unknown fields are refused. `canon trigger validate <file>` checks one document and names the
field of each error (acceptance 1). A rule's lifecycle state (`draft`, `active`, `disabled`) is not
in the document: it is deployment state, passed with the rule to `evaluate` (§ 6).

## 4. The `where` language

`where` reuses `canon-expr/1` (answer to open question 3): comparisons, `and`/`or`/`not`, list
literals, and the pure builtins `contains` (membership) and `present` (presence). The kernel
compiles it against a catalog that holds only those `Pure` builtins and no `Observation` provider,
so a rule can name no call that reaches outside the row; `canon-expr/1` has no loops or assignment.
A reference to any other function is refused at validation with the field `where` named.

The row is the context variable `$row`, a record of the row's fields. A field the row lacks reads
as absent; a comparison over an absent value is `UNKNOWN` under `canon-expr/1`'s three-valued
rules. A row whose `where` is `UNKNOWN` is not a match and is not silently dropped: it is listed
with the verdict `undetermined` (§ 7). That verdict is the one addition to the request's five, and
it follows Canon's rule that `UNKNOWN` is not `FALSE`.

## 5. Protocol access class and guard

Answer to open question 2: the access class is declared in the protocol and derived, and the rule's
`access` is a check against it, never its source.

- Each action of `protocol/1` gets an optional `access: read_only | write`. Absent means `write`.
  `canon validate` refuses `read_only` on an action that `revises` an artifact or has a
  `case_open`.
- The protocol's class, in `canon-ir/1`, is `read_only` when every action is `read_only`, else
  `write`.
- A protocol is `guarded` when one of its actions has a non-empty `requires`: it cannot run that
  action until an authority decision grants the capability, so it is not started unattended.

The `protocols` argument of `evaluate` is a map from protocol id to `{class, guarded, inputs}`,
which `canon trigger protocols <protocol files>` produces from compiled protocols. The evaluator may
build it otherwise; the kernel only reads it.

## 6. Evaluation

```
evaluate(rule_set, snapshot, protocols, history, now) -> plan
```

- `rule_set`: rules, each with its state `draft`, `active` or `disabled`.
- `snapshot` (`canon-trigger-snapshot/1`): per kind, rows with `id`, the fields, and an optional
  `entered_at` (RFC 3339), the instant the row entered its current state.
- `protocols`: § 5.
- `history` (`canon-trigger-history/1`): firings (`firing_id`, `rule`, `at`), activations (`rule`,
  `at`, optional `baseline` list of `<kind>:<row id>`), approvals (`rule`, `by`, `at`). Canon reads
  approvals and never decides who may write one (answer to open question 5).
- `now`: an instant; it is the only time the operation reads.

A row matches a rule when its kind is `subject.kind`, its `subject.state.field` equals `is`, and
`where` is `true`. For each match the plan lists the rule key, the rule's state, the kind, the row
id, the protocol, the input built from `start.input`, the firing id and the verdict, ordered by rule
key, kind, row id. Row and rule order in the input does not change the output bytes.

**Firing id.** `<rule key>/<kind>:<row id>/<entered_at>` for `once_per_state_entry`;
`<rule key>/<kind>:<row id>` for `once_per_subject`. A row that leaves the state and re-enters it
has a new `entered_at` and so a new firing id. `entered_at` is optional per row (answer to open
question 4): a row without it under `once_per_state_entry` is `refused` with reason
`entered-at-missing`.

**Verdicts**, the first that applies:

1. `refused`: the rule is `disabled`; P is not in `protocols`; P is `guarded` and `mode` is `auto`;
   `access` differs from P's class; `access` is `write`, `mode` is `auto` and the history holds no
   approval for this exact rule key; a `once_per_state_entry` row without `entered_at`. Each
   refusal names its reason.
2. `undetermined`: `where` is `UNKNOWN`.
3. `baseline`: the history's activation of this rule key is at or after the row's `entered_at`, or
   lists the row in `baseline`. Listed, never started.
4. `held`: a firing with this firing id is in the history (`already-fired`), or the rule's firings
   on the UTC day of `now`, counted from the history and then from this plan's own earlier
   `would-start` entries in plan order, have reached `cap` (`cap-reached`).
5. `offer`: `mode` is `offer`.
6. `would-start`: `mode` is `auto`.

A `draft` rule is evaluated by the same steps; its entries carry `rule_state: draft`, so a rule
under test is simulated against real state. The evaluator's contract: it submits only `would-start`
entries whose `rule_state` is `active`, with the firing id as the idempotency key, and records the
firing. Evaluating again with that firing in the history lists the match as `held`.

`canon trigger evaluate --rules … --snapshot … --protocols … --history … --now …` prints the plan.

## 7. Publication

A published rule is immutable: a change is a new version. `canon trigger check-set <rule files>
--published <index>` reads the index of published keys with the digest of each document and fails
when a document reuses a published key with a different digest (acceptance 2), when a rule in
`auto` names a guarded protocol (acceptance 7, at publication), when `start.input` names an input P
does not declare, and when two documents in the set share a key. The index is the caller's; Canon
stores nothing.

## 8. ESS and conformance

A new ESS domain `canon.trigger` declares the four documents; the `canon trigger` subcommands are
declared in `ess/domains/cli.yaml`. Each verdict and each refusal reason lands with a scenario in
Canon's conformance suite. Acceptance 9 (no write, no network) is checked by running `canon trigger
evaluate` with a read-only working directory and no network namespace.

## 9. Stories

| Story | Delivers | After |
|---|---|---|
| story:trigger-rule-document | `canon-trigger/1`, the `where` catalog, `canon trigger validate` | — |
| story:protocol-access-class | action `access`, protocol class and guard in `canon-ir/1`, `canon trigger protocols` | — |
| story:trigger-evaluate | `evaluate`, firing ids, `offer`/`would-start`/`baseline`/`undetermined`, determinism, no IO | trigger-rule-document |
| story:trigger-holds-and-refusals | `held` and `refused` with reasons | trigger-evaluate, protocol-access-class |
| story:trigger-rule-publication | `canon trigger check-set` | trigger-rule-document, protocol-access-class |
