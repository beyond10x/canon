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
  fields:
    id: {type: string}
    status: {type: string}
    amount: {type: decimal}
    segment: {type: string}
    note: {type: string, optional: true}
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
| `format` | required; `canon-trigger/1`. |
| `id`, `version` | `id` is a Canon identifier; `version` a positive integer. The pair is the rule key, written `<id>@<version>`. |
| `owner` | required, non-empty. Canon does not resolve it. |
| `subject.kind` | required; a kind name the snapshot uses. |
| `subject.fields` | required; the row fields the rule reads, each `{type: string \| integer \| decimal \| bool, optional: true \| false}`, `optional` `false` when left out. |
| `subject.state` | required; `field` names a declared field of type `string`, `is` a string value it must equal. |
| `where` | optional; one `canon-expr/1` expression of type Bool over the declared fields (§ 4). Absent: `true`. |
| `start.protocol` | required; a protocol id. |
| `start.input` | map from a case input of P to `{field: <declared field>}` or `{const: <string>}`; nothing else. Absent: no input. |
| `mode` | required; `offer` or `auto`. |
| `access` | required; `read_only` or `write`. |
| `refire` | required; `once_per_state_entry` or `once_per_subject`. |
| `cap` | required; a positive integer. |

The rule declares the row fields it reads (decided 2026-11-26, instead of an open record in
`canon-expr/1`): `$row` is then a closed record that `canon-expr/1` types as it stands, and a field
the snapshot names but the rule does not is not read. Unknown fields are refused. `canon trigger
validate --path <file>` checks one document and names the field of each error (acceptance 1); the
file is a `--path` flag, as every input of the `canon` command line is a long flag (`AGENTS.md`
§ ESS). Each error is one line `error[<code>]: <field>: <why>`:

| Code | Field named |
|---|---|
| `parse` | none: the text is not well-formed YAML or not a mapping. |
| `missing-field` | the required field left out (`owner`, `subject.state`, `start.protocol`). |
| `unknown-field` | the field `canon-trigger/1` does not have (`subject.colour`). |
| `invalid-value` | the field whose value is not one the table allows (`mode`, `version`, `cap`). |
| `undeclared-field` | `subject.state.field`, or `start.input.<name>`, naming a field `subject.fields` does not declare, or a state field that is not a required `string`. |
| `invalid-input` | `start.input.<name>`, for anything but `{field}` or `{const}` (an expression, both keys, neither). |
| `invalid-where` | `where`, with the `canon-expr/1` diagnostic code and message. |

A document without the shape of the table (a field missing or unknown, a value of the wrong kind
or outside its enumeration, an input that is not `{field}` or `{const}`) is reported with its first
error. One with the shape is reported with every error, in the order `format`, `id`, `version`,
`owner`, `subject.kind`, `subject.state.field`, `where`, `start.protocol`, `start.input` (in input
order), `cap`. `subject.kind` and `start.protocol` are Canon identifiers. A rule's lifecycle
state (`draft`, `active`, `disabled`) is not in the document: it is deployment state, passed with
the rule to `evaluate` (§ 6).

## 4. The `where` language

`where` reuses `canon-expr/1` unchanged (answer to open question 3): comparisons, `and`/`or`/`not`,
list literals, and two kinds of `Pure` builtin. The kernel compiles it with `canon_expr::check`
against a catalog it builds for each rule from the declared fields, holding only `Pure` builtins and
no `Observation` provider or recipe, so a rule can name no call that reaches outside the row;
`canon-expr/1` has no loops or assignment. The catalog, format `canon-catalog/1`, holds exactly:

| Name | Signature | Implementation | Present when |
|---|---|---|---|
| `contains` | `contains(list: List<String>, item: String) -> Bool` | `Pure(Contains)` | always |
| `present_string` | `present_string(value: Optional<String>) -> Bool` | `Pure(Present)` | an optional field has type `string` |
| `present_integer` | `present_integer(value: Optional<Integer>) -> Bool` | `Pure(Present)` | an optional field has type `integer` |
| `present_decimal` | `present_decimal(value: Optional<Decimal>) -> Bool` | `Pure(Present)` | an optional field has type `decimal` |
| `present_bool` | `present_bool(value: Optional<Bool>) -> Bool` | `Pure(Present)` | an optional field has type `bool` |

`canon-expr/1` gives each catalog function one signature, so presence is one function per field
type, and `contains` tests strings only. A call to any other name, a parameter of another type, or
a result that is not Bool is refused at validation with the field `where` named.

The row is the one context variable, `$row`: a `canon-expr/1` `Record` with one entry per declared
field, its type `String`, `Integer`, `Decimal` or `Bool` for `string`, `integer`, `decimal` or
`bool`, and `Optional` of that type when the field is optional. A `where` that reads a field the
rule does not declare (`$row.colour`) or any other variable is refused naming `where`. An optional
field is tested with its `present_<type>` function; `canon-expr/1` compares it with nothing else.
`$row.<field>` reaches only a field whose name is letters, digits and `_`, not starting with a
digit; a declared field with another name may still be the state field or an input.

A missing or wrongly typed value is the kernel's to decide, not `canon-expr/1`'s: at evaluation
(story:trigger-evaluate) a row that lacks a required declared field, or holds a value of another
type than its declaration, is listed with the verdict `undetermined` and the reason `field-missing`
or `field-type`, without calling `canon-expr/1`. A value has its declaration's type when it is a
string for `string`, a number with no fraction or exponent for `integer`, any number for
`decimal`, and `true` or `false` for `bool`; an optional field may be absent or null. Every other
row is bound as `$row`, and its `where` is `true` or `false`. A row that cannot be decided is
listed, not silently dropped (§ 6). That verdict is the one addition to the request's five, and it
follows Canon's rule that `UNKNOWN` is not `FALSE`.

## 5. Protocol access class and guard

Answer to open question 2: the access class is declared in the protocol and derived, and the rule's
`access` is a check against it, never its source.

- Each action of `protocol/1` gets an optional `access: read_only | write`. Absent means `write`.
  `canon validate` refuses `read_only` on an action that `revises` an artifact or has a
  `case_open`.
- The protocol's class, in `canon-ir/1` under `access_class`, is `read_only` when every action is
  `read_only` (a protocol without actions too), else `write`. An action's `access` is written in
  `canon-ir/1` only when it is `read_only`.
- A protocol is `guarded`, in `canon-ir/1` under `guarded`, when one of its actions has a
  non-empty `requires`: it cannot run that action until an authority decision grants the
  capability, so it is not started unattended.

The `protocols` argument of `evaluate` is a map from protocol id to `{class, guarded, inputs}`.
`canon trigger protocols --path <file> ...` prints it from compiled protocols as the member
`protocols` of a `canon-trigger-protocols/1` document, `{format, protocols}`; it refuses two files
of one protocol id (`duplicate-protocol-id`) and a protocol with imports. The evaluator may build
the map otherwise; the kernel only reads it.

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

**As built** (story:trigger-evaluate), where it is more precise than the text above:

- `--rules` names a `canon-trigger-set/1` document, `{format, rules: [{path, state}]}`, each
  `path` a `canon-trigger/1` file relative to the set file's directory.
- A snapshot row is `{id, entered_at?, fields}`: `id` is an identifier unique within its kind,
  and the values are under `fields`, keyed by field name; the row id is not a field. Every
  instant (`now`, `entered_at`, `at`) is a Canon instant, `YYYY-MM-DDTHH:MM:SSZ` in UTC with an
  optional fraction, not every RFC 3339 form.
- `undetermined` also lists a row of the kind whose state field is absent or not a string, and a
  row in the state that lacks a required declared field or holds a value of another type (§ 4),
  with the reason `field-missing` or `field-type` and the field in the entry's `field`;
  `where-unknown` is the reason for a `where` that is `UNKNOWN` or that `canon-expr/1` cannot
  evaluate (a row over its value bounds).
- A `{field}` input is the row's value as read, null for an absent optional field. The firing id
  writes `entered_at` as the snapshot writes it; an `entered-at-missing` entry has none.
- `baseline` reads the latest activation of the rule key, and the rows any activation of it lists;
  instants are compared by the start of the interval each names.
- The entries are ordered by rule key, kind and row id, each by code point.
- The input is refused as a whole, rather than listed: an instant that is not one
  (`invalid-instant`), a document of another format or a kind or row id with whitespace
  (`malformed-input`), a rule that is not valid (`invalid-rule`), two rules of one rule key
  (`duplicate-rule`), a kind or a row id listed twice (`duplicate-row`).

**As built** (story:trigger-holds-and-refusals):

- The refusals apply in the order the text lists them, `entered-at-missing` last, and each is the
  entry's `reason`: `disabled`, `unknown-protocol`, `guarded-auto`, `access-mismatch`,
  `write-unapproved`. A refusal comes before `undetermined`, so every entry of a `disabled` rule,
  an undetermined row included, is refused `disabled`.
- An approval is `{rule, by, at}`; its `at` is a Canon instant and is not compared with `now`.
- `held` comes before `offer` too: an offer whose firing id is recorded, or whose rule is at its
  cap, is held. Only `would-start` entries count against the cap.
- The cap counts the distinct firing ids of the rule key's firings whose `at` (its whole seconds)
  falls on the UTC day of `now`, before or after `now`; a firing recorded twice counts once.
  `already-fired` comes before `cap-reached`.
- Two more whole-input refusals: a protocol id listed twice in the protocols
  (`duplicate-protocol-id`), and an approval's `at` that is not an instant (`invalid-instant`).

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
