//! Deterministic evaluation of a case under a compiled protocol (design § 8, § 13).
//!
//! [`evaluate_with`] is a pure function of a compiled protocol ([`Ir`]), a `canon-case/1` case
//! snapshot, a set of `canon-evidence/1` records and the inputs a caller supplies ([`Supplied`]):
//! `canon-authority/1` authority decisions (`--authority`), the evaluation instant (`--at`) and
//! `canon-decisions/1` explicit decisions (`--decisions`).
//! [`evaluate`] is the same with nothing supplied. It reads nothing else: no clock, network,
//! filesystem, credentials or implicit `latest`. Its result is a `canon-decision/1` [`Decision`]
//! giving every declared claim the value `true`, `false` or `unknown`, each declared obligation
//! `open` or `discharged`, each declared action `admissible`, `approval-required` or `blocked`,
//! and each declared outcome `legitimate` or `blocked` (the sections below), or a
//! [`Refusal`] when the inputs do not fit the protocol. [`render`] is the decision's one
//! serialization: canonical JSON, the serialization `canon-ir/1` uses.
//!
//! # Three-valued claims
//!
//! A claim's value is its `true_when` predicate's value over the evidence set:
//!
//! - An evidence match reads the records of its kind. One that names a `subject` reads only the
//!   records about that artifact: a record about another artifact, even at that artifact's
//!   current revision, is not of the match and neither establishes nor contradicts it
//!   (CANON-EVIDENCE-003). One that names no subject reads records about any artifact. One that
//!   demands independence (`independent_on: [D, ...]`) reads only the records that name a
//!   `producer` the case's `independence` map lists on every dimension `D`: any other record of
//!   its kind and subject is not of the match, whatever its result, so it neither establishes nor
//!   contradicts it (CANON-INDEPENDENCE-001, `independence.rs`). "Records of the kind" below are
//!   the records the match reads.
//! - An evidence match without a result is `true` when a record of the kind exists and `unknown`
//!   when none does.
//! - An evidence match with a result is `unknown` when no record of the kind exists, `false` when
//!   records of the kind exist and none has the result, `true` when every record of the kind has
//!   that result, and `unknown` when records of the kind disagree (some with the result, some
//!   without): conflicting evidence neither establishes the predicate nor by itself contradicts it.
//! - `all` is `false` when a member is `false`, otherwise `true` when every member is `true` (an
//!   empty `all` is `true`), otherwise `unknown`.
//! - `any` is `true` when a member is `true`, otherwise `false` when every member is `false` (an
//!   empty `any` is `false`), otherwise `unknown`.
//! - `not` swaps `true` and `false` and keeps `unknown`.
//! - A claim reference is three-valued: `{claim: c}` and `{claim: c, is: true}` are `c`'s own value,
//!   and `{claim: c, is: false}` is `not` of it, so a claim built on `c` is `unknown` exactly when
//!   the evidence leaves `c` `unknown` and is never `false` for want of evidence.
//!   `{claim: c, is: unknown}` asks whether `c` is undecided, and is itself decided: `true` when
//!   `c` is `unknown`, `false` otherwise.
//! - An input test `{input: s, in: [...]}` is `true` when the case's value of the input `s` is one
//!   it lists and `false` otherwise. A case gives every input the protocol declares a value (the
//!   case check refuses one that does not), so an input test is never `unknown`: inputs are
//!   configuration fixed when the case opens, not evidence.
//! - An order form, `{first: {kind: K, result: R}}`, `{before: {first: K1, then: K2}}`,
//!   `{after: {first: K1, then: {kind: K2, result: R}}}` or `{since: {last: K1, then: {kind: K2,
//!   result: R}}}`, reads the evidence history instead: every record of its kinds (about its
//!   `subject` when it names one) the evidence check accepted, the records the exclusion stages set
//!   aside included, ordered by `observed_at`. `first` is whether the first record of `K` has
//!   result `R`; `before` whether the first record of `K1` was observed strictly before the first
//!   of `K2`; `after` whether a record of `K2` with result `R` was observed strictly after the
//!   first record of `K1`; `since` whether a record of `K2` with result `R`, at the subject
//!   revision of the latest record of `K1`, was observed after it. Each is `unknown` while the
//!   order it reads is not established, as with no record or a compared record without
//!   `observed_at`; the predicate's reference in `protocol/1` gives every rule (CANON-ORDER-001,
//!   CANON-ORDER-002, CANON-ORDER-004). No record is listed in `excluded_evidence` because of an
//!   order form.
//! - The order form over revisions, `{descended: {first: {kind: K1, result: R1}, then: {kind:
//!   K2, result: R2}}}` (story:revision-ordering), reads the history the same way and no
//!   `observed_at`: whether a record of `K2` with `R2` about the current revision of its subject
//!   names an `item` that a record of `K1` with `R1` about the same subject names on a revision
//!   the case snapshot lists in that artifact's `descends_from`. It is `true` when one does,
//!   otherwise `unknown` with no such later record or when the snapshot gives no
//!   `descends_from` for its subject, and otherwise `false` (CANON-ORDER-005).
//! - The form over the child cases, `{children: {in: [O, ...]}}` (story:case-composition), reads
//!   every `case_outcome` record the evidence check accepted, as the order forms read the history,
//!   one per child case: its `child_case` names the child and its `result` the child's outcome,
//!   `open` while the child runs. It is `unknown` with no record, `false` when a record's outcome
//!   is neither listed nor `open` (a child that ended outside the set), otherwise `unknown` while a
//!   child is `open` and `true` when every child ended with a listed outcome (CANON-COMPOSE-001).
//!   With `revision: current`, `{children: {in: [O, ...], revision: current}}`
//!   (story:child-outcome-revision), it reads only the records whose `subject_revision` is the
//!   case snapshot's current revision of their `subject`, by the same rules: a record about
//!   another revision neither establishes nor contradicts it, and with no record read it is
//!   `unknown` (CANON-COMPOSE-002). `revision: any` reads what no key reads. With `subject: A`
//!   (story:child-outcome-subject), it reads only the records whose `subject` is `A`, and with
//!   `revision: current` too only those at the current revision of `A`: a record about another
//!   artifact neither establishes nor contradicts it (CANON-COMPOSE-003). No record is listed in
//!   `excluded_evidence` because of it, and a reason names it with its `revision` and its
//!   `subject` when the IR writes them.
//!
//! `unknown` and `false` are never collapsed, and only `true` satisfies a positive requirement.
//!
//! # Refusals
//!
//! Inputs are checked before any claim is evaluated, in this order, and the first problem found is
//! the refusal: the compiled protocol's depth (a predicate nested more than [`MAX_IR_DEPTH`]
//! levels, itself or along the claims it tests, as `predicate-too-deep`, and claims that test each
//! other in a cycle as `claim-cycle`; see "Depth" below), the case snapshot's format, its identifiers (case id, protocol id, the case
//! revision when given, then each artifact id and revision in the order written, each followed by
//! the revisions its `descends_from` lists, in the order written: an identifier, listed once, else
//! `duplicate-identifier`, and not the artifact's current revision, else `self-descent`, each
//! naming the case, the revision and the artifact), its protocol against the compiled protocol's,
//! each artifact it lists (declared by the protocol, listed once), each artifact the protocol
//! declares (listed by the case), each case input it gives, in the order written (its id and
//! value identifiers, then declared by the protocol, else `undeclared-input`, given once, else
//! `duplicate-identifier`, and its value declared by the input, else `undeclared-input-value`,
//! naming the input and the value), each input the protocol declares (given by the case, else
//! `missing-input`, naming it), each producer its `independence` map lists, in the order written
//! (an identifier, listed once, else `duplicate-identifier`, and each dimension listed for it an
//! identifier, listed once for it, else `duplicate-identifier`, and declared by the protocol, else
//! `undeclared-dimension`, naming the producer and the dimension); then each evidence record in
//! the order given: its format, its identifiers (id, kind, subject, subject revision, then each
//! upstream artifact and revision in `upstream_revisions`, then its `producer`, its
//! `child_case` and its `item` when given), its
//! `observed_at` when given (an instant, a fraction of one to nine digits allowed, else
//! `invalid-instant`), its id
//! (used once in the set: a copy equal to the first counts once, and a copy that differs is
//! refused as `duplicate-identifier`, `` evidence `e1` is given more than once with different
//! content ``, story:evidence-redelivery), its kind (declared by the protocol) and each upstream
//! artifact
//! (declared by the protocol, else `undeclared-artifact`, naming the record and the artifact, unless
//! the record's kind declares `supplied_by`: such a record, copied unchanged from a case under its
//! supplier, may name artifacts only the supplier declares, carried and neither checked nor read,
//! story:supplied-evidence; named once, else `duplicate-identifier`), then, for a `case_outcome`
//! record, its child case (named,
//! else `missing-child-case`) and its outcome (a `result`, else `missing-child-outcome`), and for a
//! record of any other kind no child case (else `misplaced-child-case`); then, over the whole set,
//! one `case_outcome` record per child case, whatever its revision (else `duplicate-child-case`,
//! naming the least such child and its records in evidence-id order, citing each; an equal copy of
//! a record is not another record, and counts once). A record's
//! subject and subject revision are checked here as identifiers only; the revision-binding stage
//! reads them. Every refusal of a record names it (`` evidence `e2` format is `canon-evidence/2`,
//! expected `canon-evidence/1` ``, `` evidence `e2` kind identifier `k k` is empty or … ``), and
//! cites it ([`Refusal::evidence`]), here and in the revision-binding stage below, so a caller
//! that read the records from files can name the file of each (`canon evaluate` does).
//!
//! A case snapshot or evidence record that is not a document of its format is refused when it is
//! read (`read_case`, `read_evidence`), before any of this, as `malformed-input`; a required key
//! left out is named (`` missing field `subject` ``), and an evidence record is named by its `id`
//! when it has one (`` evidence `e2` is not a canon-evidence/1 document: missing field `subject`
//! ``).
//!
//! Then the supplied inputs are read, in this order: the authority decisions (`--authority`), the
//! evaluation instant (`--at`) and the explicit decisions. The authority decisions are read by
//! `authority.rs` as `canon-authority/1`, refused as the section "Authority decisions" below
//! says. The instant is read by `freshness.rs` and refused as `invalid-instant` when it is not
//! one. The explicit decisions are read by `decisions.rs` as `canon-decisions/1`, refused as the
//! section "Explicit decisions" below says. Then a ground artifact the protocol does not declare
//! (story:decision-grounds) is refused as `undeclared-artifact`, naming the entry and the artifact
//! (`grounds.rs`): each authority entry's, in capability, scope, case revision and principal order,
//! then each explicit decision's, in the order given, each entry's artifacts in id order.
//!
//! Then the exclusion stages run, in pipeline order: revision binding, freshness, invalidation.
//! Each may refuse the inputs it reads. Revision binding checks each record in the order given: a
//! record whose subject the protocol does not declare is refused as `undeclared-artifact`, naming
//! the record and its subject (`` evidence `e1` is about artifact `x`, which the protocol does not
//! declare ``), and a record bound to a revision of its subject that is not the case snapshot's
//! current one is excluded as `revision_mismatch`. Freshness refuses a `max_age` it cannot read
//! (`invalid-max-age`) and excludes a record older than its kind's `max_age` at the evaluation
//! instant as `expired`. Invalidation refuses a rule whose upstream artifact the case snapshot does
//! not name, which only an IR a caller builds can hold, as `undeclared-artifact`, and invalidates,
//! for the claims each rule names and every claim built on one of them, the records that recorded
//! a revision of the rule's upstream artifact other than the case snapshot's current one
//! (`invalidation.rs`, CANON-INVALIDATION-001).
//!
//! Then claims are evaluated.
//!
//! Last, the `outcomes` section checks the case snapshot's `termination`: one naming an outcome
//! the protocol does not declare is refused as `undeclared-outcome`, naming the outcome; then an
//! explicit decision taken for an outcome the protocol does not declare is refused as
//! `undeclared-outcome`, and one taken for a declared outcome that does not require a decision of
//! its name as `undeclared-decision`, each naming the decision and the outcome; and a
//! `termination` naming a declared outcome that is blocked as `illegitimate-termination`, naming
//! the outcome and its status (CANON-OUTCOME-001).
//!
//! # Depth
//!
//! [`read_ir`] refuses an IR whose arrays and objects nest deeper than [`MAX_IR_DEPTH`]
//! (4096) before parsing it. Reading and claim evaluation recurse along the nesting of the
//! predicates, so they run on a thread of their own with a stack sized for that bound
//! (`DEEP_STACK`): an IR at the bound reads and evaluates from any caller's thread. That thread is
//! a worker which each calling thread starts with its first call and keeps until it ends, so many
//! small calls from one thread share one worker; callers on different threads have a worker each
//! and run at the same time; and work that evaluates again runs the inner call on the worker
//! itself, never waiting for it. A panic in the work reaches the caller and the worker serves the
//! next call. The worker is the one `unsafe` of the crate (`worker.rs`).
//!
//! Claim evaluation also recurses through each claim test into the predicate of the claim it
//! tests, so the bound applies to a claim's effective depth: a predicate is one level and each
//! `all`, `any` or `not` around it one more, and a claim test at level `n` reaches `n` plus the
//! effective depth of the claim it tests. [`evaluate_with`] takes any [`Ir`] a caller builds and
//! checks the bound first, before any step recurses (`depth.rs`), computing each claim's
//! effective depth once, after the claims it tests, without recursion. It refuses as
//! `predicate-too-deep` an obligation, action, outcome or claim whose own predicate nests more
//! than [`MAX_IR_DEPTH`] levels (`` claim `c` true_when nests predicates deeper than 4096 ``), and
//! a claim whose effective depth is more than [`MAX_IR_DEPTH`], naming the claims of the chain that
//! reaches it (`` claim `c1` true_when nests predicates deeper than 4096 through the claims it
//! tests: c1 -> c2 -> c3 ``). Claims that test each other in a cycle — which `canon compile` never
//! produces, but a caller can build — have no effective depth and are refused there as
//! `claim-cycle`, naming the claims from the first one reached again, in the order evaluation
//! reaches them: `claims test each other in a cycle: c -> d -> c`. Each level of the JSON text is
//! at least one predicate level, so [`read_ir`]'s bound holds every predicate of an IR it returns
//! within the bound; a chain of claims, each within it, can still exceed it, and `canon compile`
//! compiles such a protocol, so an IR [`read_ir`] returns can still be refused here. Dropping a
//! caller's IR is the caller's own recursion.
//!
//! # The pipeline
//!
//! [`evaluate_with`] runs the same steps for every evaluation, each in the file that owns its
//! concept; this module holds only the order:
//!
//! 1. check the compiled protocol's depth (`depth.rs`), the case snapshot (`case.rs`) and the
//!    evidence set (`evidence.rs`), which keeps the first of any records equal under one id, so
//!    every later step reads each id once;
//! 2. read the supplied inputs: authority decisions (`authority.rs`), the evaluation instant
//!    (`freshness.rs`) and explicit decisions (`decisions.rs`);
//! 3. run the exclusion stages in order: revision binding (`binding.rs`), freshness
//!    (`freshness.rs`), invalidation (`invalidation.rs`). Each sees only the evidence the stages
//!    before it left. Revision binding and freshness return the records they exclude from every
//!    claim, with their reason; invalidation returns, for each claim, the records it keeps from
//!    that claim alone;
//! 4. evaluate every claim over the evidence left (`claims.rs`), and its order forms and forms over
//!    the child cases over every record the evidence check accepted (`order.rs`,
//!    `composition.rs`), each in its own context: no
//!    evidence match in its evaluation reads a record invalidated for it, including the matches of
//!    every claim it tests, which are evaluated for that use without the records invalidated for
//!    either; the value reported for a tested claim is its own. Each claim's
//!    `excluded_evidence` lists, in evidence-id order, every excluded record of a kind the claim
//!    reaches: a kind an evidence match in its own predicate names, or one a claim it tests
//!    reaches, through any number of claim references. A match that names a subject reaches its
//!    kind only for records about that artifact: a record about another artifact does not match
//!    it, so it is not listed under a claim that reaches its kind only through such a match. A
//!    record excluded from a claim is excluded from every claim built on it. A record invalidated
//!    for a claim is listed under it, with the reason `invalidated`, when an evidence match the
//!    claim reaches reads it, and under no claim it was not invalidated for. Any other record is
//!    listed under a claim, with the reason `not_independent`, when an evidence match the claim
//!    reaches would read it by kind and subject and none of those matches accepts its producer
//!    (`independence.rs`): a match that demands independence keeps it out, and no match demanding
//!    less reads it. Each record is listed once, with the first of these reasons that applies;
//! 5. the `obligations`, `actions` and `outcomes` sections (`obligations.rs`, `actions.rs`,
//!    `outcomes.rs`), then the explanation (`crate::explain`). Each section evaluates its
//!    predicates with the one evaluator claims use (`claims::predicate`): discharge and
//!    applicability predicates over the claim values, action preconditions and outcome
//!    requirements over the claim values and the evidence left after step 3 (their order forms
//!    and forms over the child cases over every record accepted), each with the case's input
//!    values, which every input test reads. The invalidation stage keeps a record from claims
//!    only, so these predicates' own evidence matches read it. An outcome that requires an explicit
//!    decision is decided by the explicit decisions instead. Before the `actions` and `outcomes`
//!    sections, which authority and explicit decisions apply is decided once
//!    (`authority::Authority::applying_at`, `decisions::Decisions::applying_at`): an entry applies
//!    at the case snapshot's `revision` (an authority entry without a case revision at every one)
//!    and only while every ground it rested on holds (`grounds.rs`), which reads the claim entries
//!    of step 4; those read no decision, so computing them first changes nothing. The two
//!    sections read only the entries that apply; the explanation records every decision given.
//!
//! Revision binding excludes a record bound to another revision as `revision_mismatch`, and
//! freshness a record older than its kind's `max_age` at the evaluation instant as `expired`,
//! each listed as excluded under each claim that reaches its kind through a match that reads it, as
//! step 4 says. Invalidation keeps a record from the claims an invalidation rule names and every
//! claim built on one of them, throughout their evaluation, as `invalidated`, listed under each of
//! those claims that reaches a match reading it; every other claim, even one reading the same
//! kind, reads the record and does not list it. The `obligations`, `actions` and `outcomes`
//! sections are written as the next section says, and every decision carries an `explanation`
//! (`crate::explain`).
//!
//! # Sections
//!
//! A section is written only when the protocol declares something it reports on: a protocol that
//! declares no obligation has no `obligations` key, and likewise for actions and outcomes.
//!
//! - `obligations` is an array with one `{"id": <obligation>, "status": <status>}` entry per
//!   declared obligation, in identifier order. An obligation with an `applicable_when` predicate is
//!   `not_applicable` when it is `false` over the claim values and the case's inputs. Otherwise the
//!   status is `discharged` when the obligation's `discharged_when` predicate is `true` over the
//!   claim values, whatever its applicability; `unknown` and `false` both leave it `open`
//!   (CANON-OBLIGATION-001). An open obligation whose applicability is `unknown` has
//!   `"reasons"` naming what is unknown, written as an action's reasons are (CANON-INPUT-001).
//! - `actions` maps each declared action to its `status` and, unless it is admissible, the
//!   `reasons` that decide it. `blocked` when the precondition is not `true`: each claim test and
//!   evidence match that decides it is a reason, `{"claim": <id>, "value": <the claim's value>}`
//!   or `{"evidence": <kind>, "present": <whether a record the match reads applies>}`, with
//!   `"subject": <artifact>` added when the match names one (a record about another artifact
//!   is not read by it), then each input test that decides it, `{"input": <id>, "value": <the
//!   case's value>}`, then each order form that decides it, `{"after": <kind>, "then": <kind>,
//!   "result": <result>, "value": <its value>}`, `{"before": <kind>, "then": <kind>, "value":
//!   <its value>}`, `{"first": <kind>, "result": <result>, "value": <its value>}` or `{"since":
//!   <kind>, "then": <kind>, "result": <result>, "value": <its value>}`, with `"subject"` added
//!   when it names one, then each form over the child cases that decides it, `{"children":
//!   [<outcome>, ...], "value": <its value>}`, with `"revision": "current"` added when the form
//!   reads only the current revision (story:child-outcome-revision) and `"subject": <artifact>`
//!   when it names one (story:child-outcome-subject), or
//!   `{"requirement": "unsatisfiable"}` when none does. `blocked` too when the precondition is
//!   `true` and the authority decisions deny a requirement of the action, each a reason
//!   `{"capability": <id>, "decision": "denied"}`. `approval-required` when the precondition is
//!   `true`, nothing is denied and some requirement requires approval or is not decided, each a
//!   reason `{"capability": <id>, "decision": "require_approval"}` or `{"capability": <id>,
//!   "decision": "none"}`. A requirement with a scope adds `"scope": {<key>: <value>, ...}` to its
//!   reason. `admissible` when the precondition is `true` and every requirement is granted
//!   (CANON-AUTHORITY-001). A requirement is decided by the decisions on its capability that
//!   apply at the case snapshot's `revision`, whose grounds all hold, and cover its scope, `denied`
//!   over `require_approval` over `granted`, as `actions.rs` says (CANON-SCOPE-001,
//!   CANON-AUTHORITY-002). A `"decision": "none"` reason adds `"stale_grounds": [...]`, the failed
//!   grounds of the entries that would have covered it but for their grounds, as "Authority
//!   decisions" below says (CANON-GROUND-001).
//! - `outcomes` maps each declared outcome to `{"status": "legitimate"}` when its `requires`
//!   predicate is `true` over the claim values and the evidence left, and otherwise to
//!   `{"status": "blocked", "reasons": [...]}`. Its reasons follow the one rule an action
//!   precondition's do, and are written the same way: in an `all` that is `false`, only its
//!   `false` members decide it, with polarity carried through `not`.
//!   An outcome that requires an explicit decision (`requires: decision: <name>`) is
//!   `legitimate` only when an explicit decision of that name, for that outcome, was taken at the
//!   case snapshot's `revision` and its grounds all hold, and its entry then records who decided
//!   (design § 37): `{"status": "legitimate", "decided_by": {"decision": <name>, "principals":
//!   [...]}}`, every principal whose decision applied, sorted by Unicode code point; otherwise it
//!   is blocked with the one reason `{"decision": <name>, "present": false}` (CANON-OUTCOME-002),
//!   which adds `"stale_grounds": [...]` when a decision taken at that revision was set aside by
//!   its grounds, as "Explicit decisions" below says (CANON-GROUND-001). A decision requirement
//!   may carry a condition (`requires: {decision: <name>, when: <predicate>}`,
//!   story:decision-condition, CANON-OUTCOME-003): a decision counts when `when` is `true` at
//!   this evaluation, whenever it was taken. While `when` is `false` or `unknown` the outcome is
//!   `blocked` by the reasons a precondition that is not `true` gives, such as `{"claim": <id>,
//!   "value": "false"}`; a decision given is neither counted nor refused, so the same decisions
//!   document counts at a later evaluation once `when` is `true`; no `decision` reason and no
//!   `stale_grounds` is written, and `pending_decisions` lists nothing for it. `when` is
//!   decided over the claim values and case inputs, as `requires` is. The case snapshot's
//!   `termination` is checked against it, as the refusals above say (CANON-OUTCOME-001).
//! - `explanation` is written for every decision: what the decision was computed from (design
//!   § 37), and why each claim that is not `true`, each open obligation, each action that is not
//!   admissible and each blocked outcome has its status, down to the evidence records that applied
//!   or were excluded. Its shape is given by `crate::explain` (CANON-EXPLAIN-001).
//! - `missing_evidence` maps each `admissible` action whose `may_produce` is not empty to the kinds
//!   of that list that have no applicable record, in kind-identifier order; an empty list means
//!   every kind it may produce has one. A record is applicable when the revision-binding and
//!   freshness stages leave it and no claim lists it in `excluded_evidence` (`missing.rs`,
//!   `applicable`). It is written only when some action is listed: an action that is `blocked` or
//!   `approval-required`, and one with an empty `may_produce`, is not. It reads `may_produce` and
//!   the records, so a kind that declares `supplied_by` is listed like any other, and it claims
//!   nothing about running the action (CANON-FRONTIER-001).
//! - `pending_decisions` lists the decisions the evaluation waits on (`pending.rs`). Under
//!   `authority`, one entry per reason `{"capability", "decision": "none" | "require_approval",
//!   "scope"?}` of an `approval-required` action, `{"action", "capability", "scope"?, "current",
//!   "case_revision"?, "options": ["granted", "denied"]}`, by action, then capability, then scope
//!   (no scope first, then by key-value pairs in key order, key before value, a prefix first);
//!   a `blocked` action has none. Under `decisions`, one entry per outcome reason `{"decision":
//!   <name>, "present": false}`, `{"outcome", "decision", "case_revision"?, "options"}`, by
//!   outcome, the options `["taken"]`, or `[]` for a snapshot without a revision. `case_revision`
//!   is the snapshot's `revision`, written when it has one. Each list, and the section, is written
//!   only when it has an entry. It reads the reasons of those two sections and the case revision,
//!   and nothing else, so it decides no coverage or lapse of its own; precedence does not change, so
//!   a `granted` entry decides a `require_approval` requirement only when no `require_approval`
//!   entry still covers it (CANON-PENDING-001).
//!
//! # Authority decisions
//!
//! `--authority` names a `canon-authority/1` document: a YAML (or JSON) list of decisions, each a
//! capability, the scope it covers (optional: absent, every scope), the decision and, optionally,
//! the principal who took it and the case revision it was taken at,
//! `- {capability: metrics.read, scope: {cluster: prod-eu}, decision: granted, principal: ops-lead,
//! case_revision: c2}` (or `require_approval`, or `denied`). An empty list decides nothing. Canon
//! grants nothing and resolves no identity: the caller decides and passes the decisions in, and a
//! principal is recorded as given and changes no decision. A decision with a case revision, whatever
//! it decides, applies only while that revision is the case snapshot's `revision`, and to nothing
//! when the snapshot has none; one without applies at every case revision (CANON-AUTHORITY-002).
//!
//! An entry may name the grounds it rested on (story:decision-grounds, CANON-GROUND-001):
//! `ground_revisions: {<artifact>: <revision>, ...}` and `ground_evidence: [<evidence id>, ...]`,
//! both optional. An artifact ground holds while its revision is the case snapshot's current
//! revision of the artifact, compared for equality; an evidence ground while a record with its id
//! is applicable, as `missing_evidence` defines it (given, left by the revision-binding and
//! freshness stages, listed in no claim's `excluded_evidence`); an id no record has does not hold
//! and is not refused. An entry applies only while every ground holds, whatever it decides, so a
//! denial whose ground moved lapses too. A requirement covered only by entries that do not apply is
//! not decided, and its reason `{"capability": <id>, "decision": "none"}` adds `"stale_grounds"`:
//! every failed ground of the entries on its capability that cover it and are set aside by their
//! grounds alone, each once, artifact grounds first by artifact and then revision, `{"artifact":
//! <id>, "reason": "revision_mismatch", "revision": <the ground's revision>}`, then evidence grounds
//! by id, `{"evidence": <id>, "reason": <reason>}` with the reason the record was excluded with
//! (the revision-binding or freshness stage's, else that of the first claim listing it in claim-id
//! order), or `{"evidence": <id>, "present": false}` when no record has the id. An entry set aside
//! by its case revision adds none of its grounds: it would not apply whatever its grounds do, so
//! naming them would ask for a ground to hold again that restores nothing. So the order of the
//! entries does not change the result (`grounds.rs`). `pending_decisions` lists such a requirement as it lists any other that
//! is not decided, without the grounds.
//!
//! Refused, in this order: text that is not such a list, or an entry with another key, another
//! decision or a missing key, as `malformed-input`, naming the entry by its position, counting
//! from 1, and a missing key by name (`` entry 2: missing field `capability` ``); then, entry by
//! entry, a capability, scope key, scope value, principal, case revision, ground artifact, ground
//! revision or ground evidence id that is not an identifier as `invalid-identifier`, and an entry
//! equal to an earlier one in capability, scope, case revision and principal, whatever its decision
//! and grounds, as `duplicate-identifier` (`authority.rs`); then, after the explicit decisions are
//! read, a ground artifact the protocol does not declare as `undeclared-artifact`.
//!
//! # Explicit decisions
//!
//! `--decisions` names a `canon-decisions/1` document: a YAML (or JSON) list of decisions, each
//! naming the decision, the outcome it was taken for, the principal who took it and the case
//! revision it was taken at, `- {decision: explicitly_inconclusive, outcome: inconclusive,
//! principal: lead-investigator, case_revision: c2}`. An empty list decides nothing. Canon decides
//! nothing and resolves no identity: the caller passes in the decisions that were taken. A
//! decision applies only while its case revision is the case snapshot's `revision`: one taken at a
//! superseded revision, or given for a snapshot without a revision, applies to nothing. A decision
//! may name grounds as an authority entry does, and applies only while every one holds; an outcome
//! left blocked by a decision at the snapshot's `revision` that its grounds set aside adds
//! `"stale_grounds"` to its reason `{"decision": <name>, "present": false}`, by the rules
//! "Authority decisions" gives, and `pending_decisions` lists it as it lists any other outcome
//! waiting on a decision (CANON-GROUND-001). Refused, in this order: text that is not such a list,
//! or an entry with another key or a missing one, as `malformed-input`, naming the entry by its
//! position and a missing key by name (`` entry 1: missing field `case_revision` ``); then, entry
//! by entry, a decision, outcome, principal, case revision, ground artifact, ground revision or
//! ground evidence id that is not an identifier as `invalid-identifier`, and an entry that repeats
//! an earlier one in its decision, outcome, principal and case revision, whatever its grounds, as
//! `duplicate-identifier`, naming it; then a ground artifact the protocol does not declare as
//! `undeclared-artifact`; then, in the `outcomes` section and at any case revision, a decision for
//! an outcome the protocol does not declare as `undeclared-outcome`, and one for an outcome that
//! does not require a decision of its name as `undeclared-decision`.

mod actions;
mod authority;
mod binding;
mod case;
mod claims;
mod composition;
mod decision;
#[cfg(test)]
mod decision_condition;
mod decisions;
mod depth;
mod evidence;
mod freshness;
mod grounds;
mod independence;
mod invalidation;
#[cfg(test)]
mod invalidation_cost;
mod json;
mod missing;
mod obligations;
mod order;
mod outcomes;
mod pending;
mod read;
mod worker;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::ir::Ir;
use crate::model::{
    ArtifactId, Case, ChildOutcomes, ClaimDecision, ClaimId, DECISION_FORMAT, Decision,
    Declarations, EvidenceExclusion, EvidenceId, EvidenceKindId, EvidenceMatch, EvidenceRecord,
    ExclusionReason, Predicate, is_identifier, one_line,
};

pub use case::{case_from_value, read_case};
pub use decision::{render, render_sections};
pub use evidence::{evidence_from_value, read_evidence};
pub use read::{MAX_IR_DEPTH, read_ir};

pub(crate) use authority::Authority;
pub(crate) use claims::{EvidenceSets, Inputs, reads};
pub(crate) use decisions::Decisions;
pub(crate) use depth::check as check_depth;
pub(crate) use independence::Independence;
pub(crate) use invalidation::{Invalidated, invalidated_claims};

/// Why an evaluation was refused: a stable machine-readable code, a one-line message, and the
/// evidence records it refuses, by id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    code: &'static str,
    message: String,
    evidence: Vec<EvidenceId>,
}

impl Refusal {
    fn new(code: &'static str, message: String) -> Self {
        Self {
            code,
            message,
            evidence: Vec::new(),
        }
    }

    /// The same refusal, citing the evidence record `id`.
    fn citing(mut self, id: &EvidenceId) -> Self {
        self.evidence.push(id.clone());
        self
    }

    /// A stable machine-readable code for the kind of refusal; a conformance scenario names it.
    pub fn code(&self) -> &'static str {
        self.code
    }

    /// The evidence records the refusal is about, by id, each named in the message: every refusal
    /// of a record cites it, so a caller that read the records from files can name each one's file
    /// (`canon evaluate` does). Empty for a refusal of anything else.
    pub fn evidence(&self) -> &[EvidenceId] {
        &self.evidence
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Refusal {}

/// What a caller supplies besides the protocol, the case snapshot and the evidence, each as the
/// text it was given, unparsed: the file that owns it reads it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Supplied<'a> {
    /// `canon-authority/1` decisions (`--authority`), read by `authority.rs`.
    pub authority: Option<&'a str>,
    /// The evaluation instant (`--at`), read by `freshness.rs`.
    pub at: Option<&'a str>,
    /// Explicit decisions (`canon-decisions/1`), read by `decisions.rs`.
    pub decisions: Option<&'a str>,
}

/// Evaluates every claim of `ir` for `case` from `evidence`, with nothing else supplied. Pure and
/// deterministic: the same inputs give the same decision, and the order of `evidence` does not
/// matter.
pub fn evaluate(ir: &Ir, case: &Case, evidence: &[EvidenceRecord]) -> Result<Decision, Refusal> {
    evaluate_with(ir, case, evidence, Supplied::default())
}

/// Evaluates `case` under `ir` from `evidence` and the `supplied` inputs, in the order the module
/// docs give. Pure and deterministic.
pub fn evaluate_with(
    ir: &Ir,
    case: &Case,
    evidence: &[EvidenceRecord],
    supplied: Supplied<'_>,
) -> Result<Decision, Refusal> {
    depth::check(ir)?;
    case::check(ir, case)?;
    let distinct = evidence::check(ir, evidence)?;
    let evidence = distinct.as_slice();
    let authority = authority::read(supplied.authority)?;
    let at = freshness::instant(supplied.at)?;
    let decisions = decisions::read(supplied.decisions)?;
    grounds::declared(ir, authority.as_ref(), decisions.as_ref())?;
    on_deep_stack(|| {
        let case_inputs: Inputs = case
            .inputs
            .iter()
            .map(|(input, value)| (input.clone(), value.clone()))
            .collect();
        let mut applicable: Vec<&EvidenceRecord> = evidence.iter().collect();
        let mut excluded: Vec<EvidenceExclusion> = Vec::new();
        let stage = binding::exclude(ir, case, &applicable)?;
        set_aside(&mut applicable, &mut excluded, stage);
        let stage = freshness::exclude(ir, case, &applicable, at.as_ref())?;
        set_aside(&mut applicable, &mut excluded, stage);
        let invalidated = invalidation::exclude(ir, case, &applicable)?;

        let applicable: Vec<EvidenceRecord> = applicable.into_iter().cloned().collect();
        // Evidence matches read what the exclusion stages left; the order forms read every record
        // the evidence check accepted.
        let independence = Independence::of(case);
        let sets = EvidenceSets {
            left: &applicable,
            history: evidence,
            independence: Some(&independence),
            artifacts: Some(&case.artifacts),
        };
        let values = claims::values(ir, sets, &invalidated, &case_inputs)?;
        let obligations = obligations::section(ir, &values, &case_inputs);
        // The claim entries read no authority or explicit decision; they come before the sections
        // that read those, since an evidence ground holds only for a record no claim lists.
        let record_of: BTreeMap<&EvidenceId, &EvidenceRecord> =
            evidence.iter().map(|record| (&record.id, record)).collect();
        let reached = invalidation::reached_by_claim(ir);
        let claims = values
            .iter()
            .map(|(id, value)| {
                let entry = ClaimDecision {
                    value: *value,
                    excluded_evidence: excluded_for(
                        reached.get(id).map_or(&[], Vec::as_slice),
                        id,
                        evidence,
                        &record_of,
                        &excluded,
                        &invalidated,
                        &independence,
                    ),
                };
                (id.clone(), entry)
            })
            .collect();
        let claims = Declarations::new(claims);
        // Which entries apply is decided once, here: at the case revision, with every ground
        // holding. Only those decide a requirement or an outcome; the explanation records every
        // entry as given.
        let standing = grounds::Standing::new(
            case,
            &applicable,
            &excluded,
            claims.iter().map(|(_, entry)| entry),
        );
        let applying = authority
            .as_ref()
            .map(|authority| authority.applying_at(case.revision.as_ref(), &standing));
        let applying_decisions = decisions
            .as_ref()
            .map(|decisions| decisions.applying_at(case.revision.as_ref(), &standing));
        let actions = actions::section(ir, &values, &case_inputs, sets, applying.as_ref());
        let outcomes = outcomes::section(
            ir,
            case,
            &values,
            &case_inputs,
            sets,
            decisions.as_ref(),
            applying_decisions.as_ref(),
        )?;
        // The records the binding and freshness stages left, less those a claim lists.
        let missing_evidence = missing::section(
            ir,
            actions.as_ref(),
            &applicable,
            claims.iter().map(|(_, entry)| entry),
        );
        // Read from the reasons the two stages gave, so coverage and lapse are decided once.
        let pending_decisions =
            pending::section(actions.as_ref(), outcomes.as_ref(), case.revision.as_ref());
        let mut decision = Decision {
            format: DECISION_FORMAT.to_owned(),
            case: case.id.clone(),
            protocol: ir.protocol.id.clone(),
            protocol_revision: ir.protocol.revision,
            claims,
            obligations,
            actions,
            outcomes,
            explanation: None,
            missing_evidence,
            pending_decisions,
        };
        let inputs = crate::explain::Inputs {
            authority: authority.as_ref(),
            decisions: decisions.as_ref(),
            at: supplied.at,
        };
        decision.explanation = crate::explain::explain(
            ir,
            case,
            evidence,
            &excluded,
            &invalidated,
            inputs,
            &decision,
        );
        Ok(decision)
    })
}

/// The reasons `predicate` is not `true` over the claim `values`, the case's `inputs` and the
/// `evidence` (what was left, and the history), as a blocked outcome states them (`outcomes.rs`):
/// the explanation gives an open obligation the same reasons. Called only for a predicate that is
/// not `true`.
pub(crate) fn unmet_reasons(
    predicate: &Predicate,
    values: &BTreeMap<ClaimId, crate::model::Truth>,
    inputs: &Inputs,
    evidence: EvidenceSets<'_>,
) -> Vec<crate::model::Json> {
    outcomes::reasons(predicate, values, inputs, evidence)
}

/// The reason each order form among `forms` gives, as a blocked action names it, with its value
/// over the `history` and, for a `descended` form, the descent of the case's `artifacts`
/// (`order.rs`): each form once, in the order `order.rs` gives.
pub(crate) fn order_reasons<'a>(
    forms: impl IntoIterator<Item = &'a Predicate>,
    history: &[EvidenceRecord],
    artifacts: &Declarations<ArtifactId, crate::model::CaseArtifact>,
) -> Vec<crate::model::Json> {
    order::reasons(forms, history, Some(artifacts))
}

/// The reason each `children` form among `forms` gives, as a blocked action names it, with its
/// value over the records it reads of the `history`, against the current revisions of the case's
/// `artifacts` (`composition.rs`): each form once, by its outcome list, then its `revision`, then
/// its `subject`.
pub(crate) fn children_reasons<'a>(
    forms: impl IntoIterator<Item = &'a ChildOutcomes>,
    history: &[EvidenceRecord],
    artifacts: &Declarations<ArtifactId, crate::model::CaseArtifact>,
) -> Vec<crate::model::Json> {
    composition::reasons(forms, history, Some(artifacts))
}

/// Moves the records one exclusion stage excluded out of `applicable` and into `excluded`. A stage
/// that names a record no longer applicable excludes nothing more.
fn set_aside(
    applicable: &mut Vec<&EvidenceRecord>,
    excluded: &mut Vec<EvidenceExclusion>,
    stage: Vec<EvidenceExclusion>,
) {
    for exclusion in stage {
        if let Some(at) = applicable
            .iter()
            .position(|record| record.id == exclusion.evidence)
        {
            applicable.remove(at);
            excluded.push(exclusion);
        }
    }
}

/// The excluded records listed under `claim`: those an evidence match the claim reaches would read
/// (of its kind and, when it names a subject, about that artifact), in evidence-id order. A claim
/// reaches the evidence matches of its own predicate, and every match a claim it tests reaches,
/// through any number of claim references; each claim is visited once, so a cycle a caller builds
/// into an IR ends. The records the exclusion stages set aside for every claim (`excluded`) are
/// listed with their reasons. A record the invalidation stage kept from this claim
/// (`invalidated`) is listed with the reason `invalidated` when an evidence match the claim reaches
/// reads it ([`invalidation::Invalidated::listed`]). Any other record is listed with the reason
/// `not_independent` when a match the claim reaches would read it by kind and subject and none of
/// those matches accepts its producer ([`independence::rejected`]). Each record is listed once.
///
/// `reached` holds the evidence matches the claim reaches ([`invalidation::reached_by_claim`]) and
/// `record_of` is `evidence` by id, each found once for every claim, so listing a claim costs time
/// linear in the matches it reaches and in the records it lists or reads, not that times the
/// records invalidated for it.
fn excluded_for(
    reached: &[&EvidenceMatch],
    claim: &ClaimId,
    evidence: &[EvidenceRecord],
    record_of: &BTreeMap<&EvidenceId, &EvidenceRecord>,
    excluded: &[EvidenceExclusion],
    invalidated: &invalidation::Invalidated,
    independence: &Independence,
) -> Vec<EvidenceExclusion> {
    let demanding = reached
        .iter()
        .any(|matching| matching.independent_on.is_some());
    if excluded.is_empty() && invalidated.kept(claim).is_empty() && !demanding {
        return Vec::new();
    }
    // Each kind and subject the reached matches read, once.
    let matches: Vec<(&EvidenceKindId, Option<&ArtifactId>)> = reached
        .iter()
        .map(|matching| (&matching.kind, matching.subject.as_ref()))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let read = |record: &EvidenceRecord| {
        matches
            .iter()
            .any(|(kind, subject)| claims::reads(kind, *subject, record))
    };
    let mut listed: Vec<EvidenceExclusion> = excluded
        .iter()
        .filter(|exclusion| {
            record_of
                .get(&exclusion.evidence)
                .is_some_and(|record| read(record))
        })
        .cloned()
        .chain(
            invalidated
                .listed(matches.iter().copied(), claim)
                .map(|id| EvidenceExclusion {
                    evidence: id.clone(),
                    reason: ExclusionReason::Invalidated,
                }),
        )
        .collect();
    // A record is rejected only by a match that demands independence: when none of the reached
    // matches demands it, no record is listed as `not_independent`.
    if demanding {
        let set_aside: BTreeSet<&EvidenceId> = excluded
            .iter()
            .map(|exclusion| &exclusion.evidence)
            .collect();
        listed.extend(
            evidence
                .iter()
                .filter(|record| {
                    !set_aside.contains(&record.id)
                        && !invalidated.excludes(claim, &record.id)
                        && independence::rejected(reached, record, independence)
                })
                .map(|record| EvidenceExclusion {
                    evidence: record.id.clone(),
                    reason: ExclusionReason::NotIndependent,
                }),
        );
    }
    listed.sort();
    listed
}

/// The stack the recursive steps of [`read_ir`] and [`evaluate`] run on. Their depth follows the
/// nesting of the protocol's predicates, which [`MAX_IR_DEPTH`] bounds; a debug build needs
/// up to about 2 KiB per level, so the bound needs about 8 MiB, more than a spawned thread's
/// default 2 MiB. 64 MiB leaves room; it is reserved, not touched, until used.
const DEEP_STACK: usize = 64 << 20;

#[cfg(test)]
thread_local! {
    /// The threads [`on_deep_stack`] started for calls made on this thread; read by the unit test
    /// that holds the count of many small calls.
    static SPAWNED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Runs `work` on a thread with [`DEEP_STACK`], so how deep a protocol's predicates may nest does
/// not depend on the caller's stack. The thread reads nothing and the result is the same as on the
/// caller's thread. A panic in `work` is resumed on the caller's thread, and the next call works.
///
/// The thread is a worker that each calling thread starts with its first call and keeps until that
/// thread ends (`worker.rs`): a caller that makes many small calls pays one thread and a channel
/// round trip per call, not a thread per call. What it costs: a stack of [`DEEP_STACK`] reserved,
/// not touched, for each thread that ever evaluates, for as long as it lives; and one worker per
/// calling thread, so callers on different threads run at the same time (a worker shared by
/// the process would run them one after the other, which `canon check`, `generate` and `simulate`
/// would pay for, since they call from several threads).
///
/// Reentrancy: `work` that evaluates again runs the inner call where it is, on the worker's deep
/// stack. Nothing would deadlock otherwise (the inner call would start another worker for the
/// worker thread); this keeps the nested call on the stack the worker has and the thread count
/// down.
fn on_deep_stack<T: Send>(work: impl FnOnce() -> T + Send) -> T {
    worker::run(work)
}

/// [`on_deep_stack`] with a thread started and joined for this one call: what every call did
/// before the worker, and what a call made while its thread's locals are destroyed still does.
fn on_a_thread_of_its_own<T: Send>(work: impl FnOnce() -> T + Send) -> T {
    #[cfg(test)]
    SPAWNED.with(|spawned| spawned.set(spawned.get() + 1));
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("canon-eval".to_owned())
            .stack_size(DEEP_STACK)
            .spawn_scoped(scope, work)
            .expect("the operating system starts a thread for evaluation");
        match worker.join() {
            Ok(result) => result,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    })
}

fn unsupported_format(found: &str, expected: &str) -> Refusal {
    Refusal::new(
        "unsupported-format",
        format!("format is `{}`, expected `{expected}`", one_line(found)),
    )
}

fn identifier(what: &str, id: &str) -> Result<(), Refusal> {
    if is_identifier(id) {
        return Ok(());
    }
    Err(Refusal::new(
        "invalid-identifier",
        format!(
            "{what} `{}` is empty or contains whitespace or a control character",
            one_line(id)
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Truth;

    const PROTOCOL: &str = "format: protocol/1\n\
        protocol: {id: p, revision: 3}\n\
        artifacts: {a: {}, b: {}}\n\
        evidence_kinds: {k: {}}\n\
        claims:\n\
          \x20\x20base: {true_when: {evidence: {kind: k, result: pass}}}\n\
          \x20\x20is_true: {true_when: {claim: base}}\n\
          \x20\x20is_false: {true_when: {claim: base, is: false}}\n\
          \x20\x20is_unknown: {true_when: {claim: base, is: unknown}}\n\
          \x20\x20negated: {true_when: {not: {claim: base, is: unknown}}}\n";

    const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}, b: {revision: r1}}\n";

    fn ir() -> Ir {
        crate::ir::compile(&crate::model::parse(PROTOCOL).expect("parses")).expect("compiles")
    }

    fn record(id: &str, result: &str) -> String {
        format!(
            "format: canon-evidence/1\nid: {id}\nkind: k\nresult: {result}\nsubject: a\nsubject_revision: r1\n"
        )
    }

    fn decide(case: &str, evidence: &[String]) -> Result<Decision, Refusal> {
        let case = read_case(case)?;
        let evidence = evidence
            .iter()
            .map(|text| read_evidence(text))
            .collect::<Result<Vec<_>, _>>()?;
        evaluate(&ir(), &case, &evidence)
    }

    fn values(evidence: &[String]) -> Vec<(String, Truth)> {
        decide(CASE, evidence)
            .expect("decides")
            .claims
            .iter()
            .map(|(id, entry)| (id.as_str().to_owned(), entry.value))
            .collect()
    }

    #[test]
    fn a_claim_reference_is_three_valued_and_only_is_unknown_is_decided() {
        use Truth::{False as F, True as T, Unknown as U};
        let names = ["base", "is_false", "is_true", "is_unknown", "negated"];
        for (evidence, expected) in [
            (vec![], [U, U, U, T, F]),
            (vec![record("e1", "pass")], [T, F, T, F, T]),
            (vec![record("e1", "fail")], [F, T, F, F, T]),
            (
                vec![record("e1", "pass"), record("e2", "fail")],
                [U, U, U, T, F],
            ),
        ] {
            let found = values(&evidence);
            let expected: Vec<(String, Truth)> = names
                .iter()
                .map(|name| name.to_string())
                .zip(expected)
                .collect();
            assert_eq!(found, expected, "{evidence:?}");
        }
    }

    #[test]
    fn the_order_of_the_evidence_set_does_not_change_the_decision() {
        let forward = [record("e1", "pass"), record("e2", "fail")];
        let backward = [record("e2", "fail"), record("e1", "pass")];
        let render_of = |evidence: &[String]| render(&decide(CASE, evidence).expect("decides"));
        assert_eq!(render_of(&forward), render_of(&backward));
    }

    #[test]
    fn the_decision_renders_every_declared_claim_in_identifier_order() {
        let decision = decide(CASE, &[record("e1", "pass")]).expect("decides");
        // Every section but the explanation, whose shape `crate::explain` tests.
        let listed = ["case", "claims", "format", "protocol", "protocol_revision"]
            .map(str::to_owned)
            .into();
        assert_eq!(
            render_sections(&decision, &listed),
            "{\n  \"case\": \"C-1\",\n  \"claims\": {\n    \"base\": {\n      \"value\": \"true\"\n    },\n    \"is_false\": {\n      \"value\": \"false\"\n    },\n    \"is_true\": {\n      \"value\": \"true\"\n    },\n    \"is_unknown\": {\n      \"value\": \"false\"\n    },\n    \"negated\": {\n      \"value\": \"true\"\n    }\n  },\n  \"format\": \"canon-decision/1\",\n  \"protocol\": \"p\",\n  \"protocol_revision\": 3\n}\n"
        );
    }

    #[test]
    fn inputs_that_do_not_fit_the_protocol_are_refused_with_a_code() {
        let pass = record("e1", "pass");
        for (case, evidence, code, message) in [
            (
                CASE.replace("canon-case/1", "canon-case/2"),
                vec![],
                "unsupported-format",
                "format is `canon-case/2`, expected `canon-case/1`",
            ),
            (
                CASE.replace("id: C-1", "id: 'C 1'"),
                vec![],
                "invalid-identifier",
                "case identifier `C 1` is empty or contains whitespace or a control character",
            ),
            (
                CASE.replace("b: {revision: r1}", "b: {revision: ''}"),
                vec![],
                "invalid-identifier",
                "artifact revision `` is empty or contains whitespace or a control character",
            ),
            (
                CASE.replace("protocol: p", "protocol: q"),
                vec![],
                "protocol-mismatch",
                "case `C-1` is governed by protocol `q`, not by `p`",
            ),
            (
                CASE.replace("b: {revision: r1}", "c: {revision: r1}"),
                vec![],
                "undeclared-artifact",
                "case `C-1` lists artifact `c`, which the protocol does not declare",
            ),
            (
                CASE.replace("b: {revision: r1}", "a: {revision: r2}"),
                vec![],
                "malformed-input",
                "case is not a canon-case/1 document: artifacts: duplicate entry with key \"a\"",
            ),
            (
                CASE.replace(", b: {revision: r1}", ""),
                vec![],
                "missing-artifact",
                "case `C-1` does not give the current revision of artifact `b`",
            ),
            (
                CASE.to_owned(),
                vec![pass.replace("canon-evidence/1", "evidence/1")],
                "unsupported-format",
                "evidence `e1` format is `evidence/1`, expected `canon-evidence/1`",
            ),
            (
                CASE.to_owned(),
                vec![pass.replace("subject_revision: r1", "subject_revision: 'r 1'")],
                "invalid-identifier",
                "evidence `e1` subject revision `r 1` is empty",
            ),
            (
                CASE.to_owned(),
                vec![pass.clone(), record("e1", "fail")],
                "duplicate-identifier",
                "evidence `e1` is given more than once with different content",
            ),
            (
                CASE.to_owned(),
                vec![pass.replace("kind: k", "kind: z")],
                "undeclared-evidence-kind",
                "evidence `e1` is of kind `z`, which the protocol does not declare",
            ),
        ] {
            let refusal = decide(&case, &evidence).expect_err(code);
            assert_eq!(refusal.code(), code, "{refusal}");
            assert!(refusal.to_string().starts_with(message), "{refusal}");
        }
    }

    /// `evaluate` takes any IR a caller builds; claims that test each other in a cycle are
    /// refused, naming the cycle, instead of recursing until the stack overflows.
    #[test]
    fn an_ir_whose_claims_test_each_other_in_a_cycle_is_refused() {
        use crate::model::{ClaimId, ClaimTest, Predicate};
        let case = read_case(CASE).expect("case");
        // Each edit replaces a claim's predicate with a test of another claim.
        let refused = |edits: &[(&str, &str)]| {
            let mut cyclic = ir();
            for (claim, tested) in edits {
                cyclic
                    .claims
                    .get_mut(&ClaimId::new(*claim))
                    .expect("declared")
                    .true_when = Predicate::Claim(ClaimTest {
                    claim: ClaimId::new(*tested),
                    is: Truth::True,
                });
            }
            let refusal = evaluate(&cyclic, &case, &[]).expect_err("a cycle is refused");
            assert_eq!(refusal.code(), "claim-cycle", "{refusal}");
            refusal.to_string()
        };
        assert_eq!(
            refused(&[("base", "base")]),
            "claims test each other in a cycle: base -> base"
        );
        // `is_true` already tests `base`.
        assert_eq!(
            refused(&[("base", "is_true")]),
            "claims test each other in a cycle: base -> is_true -> base"
        );
        assert_eq!(
            refused(&[("base", "is_true"), ("is_true", "is_false")]),
            "claims test each other in a cycle: base -> is_true -> is_false -> base"
        );
        // A cycle that does not pass through the first claim evaluated is found as well.
        assert_eq!(
            refused(&[("negated", "is_unknown"), ("is_unknown", "negated")]),
            "claims test each other in a cycle: is_unknown -> negated -> is_unknown"
        );
    }

    /// YAML refuses a repeated key before evaluation; a caller that builds the case itself can
    /// still repeat an artifact, and the evaluator refuses that too.
    #[test]
    fn a_case_built_with_a_repeated_artifact_is_refused() {
        let mut case = read_case(CASE).expect("case reads");
        let first = case.artifacts.iter().next().expect("an artifact");
        let repeated = (first.0.clone(), first.1.clone());
        let mut entries: Vec<_> = case
            .artifacts
            .iter()
            .map(|(id, entry)| (id.clone(), entry.clone()))
            .collect();
        entries.push(repeated);
        case.artifacts = Declarations::new(entries);
        let refusal = evaluate(&ir(), &case, &[]).expect_err("refused");
        assert_eq!(refusal.code(), "duplicate-identifier");
        assert_eq!(
            refusal.to_string(),
            "case `C-1` lists artifact `a` more than once"
        );
    }

    /// A record about an artifact the protocol does not declare is refused, naming it; one bound
    /// to a revision of its subject that is not current is excluded, so it decides nothing and is
    /// listed under every claim that reaches its kind.
    #[test]
    fn evidence_applies_only_to_the_current_revision_of_a_declared_subject() {
        let elsewhere = record("e1", "pass").replace("subject: a", "subject: nowhere");
        let refusal = decide(CASE, &[elsewhere]).expect_err("refused");
        assert_eq!(refusal.code(), "undeclared-artifact", "{refusal}");
        assert_eq!(
            refusal.to_string(),
            "evidence `e1` is about artifact `nowhere`, which the protocol does not declare"
        );

        let stale = record("e1", "fail").replace("subject_revision: r1", "subject_revision: r0");
        assert_eq!(values(std::slice::from_ref(&stale)), values(&[]));
        let decision = decide(CASE, &[stale, record("e2", "pass")]).expect("decides");
        for (claim, entry) in decision.claims.iter() {
            assert_eq!(
                entry.excluded_evidence,
                [exclusion(
                    "e1",
                    crate::model::ExclusionReason::RevisionMismatch
                )],
                "{}",
                claim.as_str()
            );
        }
        assert_eq!(
            values(&[record("e2", "pass")]),
            decision
                .claims
                .iter()
                .map(|(id, entry)| (id.as_str().to_owned(), entry.value))
                .collect::<Vec<_>>()
        );
    }

    /// One record of `kind` about artifact `a` at `r1`, built directly.
    fn bare(id: &str, kind: &str) -> EvidenceRecord {
        EvidenceRecord {
            format: crate::model::EVIDENCE_FORMAT.to_owned(),
            producer: None,
            id: EvidenceId::new(id),
            kind: crate::model::EvidenceKindId::new(kind),
            result: None,
            subject: crate::model::ArtifactId::new("a"),
            subject_revision: crate::model::Revision::new("r1"),
            observed_at: None,
            upstream_revisions: crate::model::Declarations::default(),
            child_case: None,
            item: None,
        }
    }

    fn exclusion(id: &str, reason: crate::model::ExclusionReason) -> EvidenceExclusion {
        EvidenceExclusion {
            evidence: EvidenceId::new(id),
            reason,
        }
    }

    fn ids(kept: &[&EvidenceRecord]) -> Vec<String> {
        kept.iter()
            .map(|record| record.id.as_str().to_owned())
            .collect()
    }

    /// Each record a stage names leaves the kept set and joins the excluded list once, in the
    /// order named; a record no longer kept (excluded by an earlier stage, or never given) is not
    /// excluded again.
    #[test]
    fn set_aside_moves_each_named_record_out_of_the_kept_set_once() {
        use crate::model::ExclusionReason::{Expired, Invalidated, RevisionMismatch};
        let records = [bare("e1", "k"), bare("e2", "k"), bare("e3", "k")];
        let mut kept: Vec<&EvidenceRecord> = records.iter().collect();
        let mut excluded = Vec::new();
        set_aside(
            &mut kept,
            &mut excluded,
            vec![
                exclusion("e2", RevisionMismatch),
                exclusion("gone", Expired),
            ],
        );
        assert_eq!(ids(&kept), ["e1", "e3"]);
        assert_eq!(excluded, [exclusion("e2", RevisionMismatch)]);
        set_aside(
            &mut kept,
            &mut excluded,
            vec![exclusion("e3", Invalidated), exclusion("e2", Expired)],
        );
        assert_eq!(ids(&kept), ["e1"]);
        assert_eq!(
            excluded,
            [
                exclusion("e2", RevisionMismatch),
                exclusion("e3", Invalidated)
            ]
        );
    }

    /// An excluded record is listed under every claim that reaches its kind, directly or through
    /// a chain of claim references, in evidence-id order, and under no other claim.
    #[test]
    fn excluded_for_lists_the_kinds_a_claim_reaches_in_evidence_id_order() {
        use crate::model::ExclusionReason::{Expired, RevisionMismatch};
        let ir = crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\nevidence_kinds: {k: {}, l: {}}\n\
                 claims:\n\
                 \x20\x20direct: {true_when: {evidence: {kind: k}}}\n\
                 \x20\x20one_level: {true_when: {not: {claim: direct}}}\n\
                 \x20\x20two_levels: {true_when: {any: [{claim: one_level, is: unknown}]}}\n\
                 \x20\x20other: {true_when: {evidence: {kind: l}}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles");
        let evidence = [bare("e1", "k"), bare("e2", "l"), bare("e3", "k")];
        // In the order the stages excluded them, which is not evidence-id order.
        let excluded = [
            exclusion("e3", RevisionMismatch),
            exclusion("e2", Expired),
            exclusion("e1", Expired),
        ];
        let reached = invalidation::reached_by_claim(&ir);
        let listed = |claim: &str| -> Vec<String> {
            excluded_for(
                &reached[&ClaimId::new(claim)],
                &ClaimId::new(claim),
                &evidence,
                &evidence.iter().map(|record| (&record.id, record)).collect(),
                &excluded,
                &invalidation::Invalidated::default(),
                &Independence::default(),
            )
            .iter()
            .map(|exclusion| exclusion.evidence.as_str().to_owned())
            .collect()
        };
        for claim in ["direct", "one_level", "two_levels"] {
            assert_eq!(listed(claim), ["e1", "e3"], "{claim}");
        }
        assert_eq!(listed("other"), ["e2"]);
        assert_eq!(
            excluded_for(
                &reached[&ClaimId::new("direct")],
                &ClaimId::new("direct"),
                &evidence,
                &evidence.iter().map(|record| (&record.id, record)).collect(),
                &[],
                &invalidation::Invalidated::default(),
                &Independence::default(),
            ),
            Vec::new()
        );
    }

    /// A claim is evaluated in its own context: a record invalidated for it is read by no match in
    /// its evaluation, including the match of `helper`, which the rule does not name. So `named`
    /// is `unknown`, not `true` on `e1` through `helper`, and lists `e1`; so do `built` and
    /// `tests_only`, built on it, the second through claim tests alone. `helper` reports its own
    /// value, `true` on `e1`, and lists nothing. The explanation agrees with `excluded_evidence`.
    #[test]
    fn a_claim_is_evaluated_without_its_invalidated_records_in_every_claim_it_tests() {
        let ir = crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
                 artifacts: {a: {}, up: {}}\nevidence_kinds: {k: {}, m: {}}\n\
                 claims: {helper: {true_when: {evidence: {kind: k}}}, \
                 named: {true_when: {all: [{claim: helper}, {evidence: {kind: m}}]}}, \
                 built: {true_when: {all: [{claim: named}, {evidence: {kind: k}}]}}, \
                 tests_only: {true_when: {not: {claim: named}}}}\n\
                 invalidation: {r: {upstream: up, invalidates: [named]}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles");
        let case = read_case(
            "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}, up: {revision: u1}}\n",
        )
        .expect("reads");
        let evidence = [
            read_evidence(
                "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r1\n\
                 upstream_revisions: {up: u0}\n",
            )
            .expect("reads"),
            read_evidence(
                "format: canon-evidence/1\nid: e2\nkind: m\nsubject: a\nsubject_revision: r1\n",
            )
            .expect("reads"),
        ];
        let decision = evaluate(&ir, &case, &evidence).expect("decides");
        let entry = |claim: &str| {
            let entry = decision.claims.get(&ClaimId::new(claim)).expect("declared");
            let listed: Vec<&str> = entry
                .excluded_evidence
                .iter()
                .map(|exclusion| exclusion.evidence.as_str())
                .collect();
            (entry.value, listed)
        };
        assert_eq!(entry("helper"), (Truth::True, vec![]));
        assert_eq!(entry("named"), (Truth::Unknown, vec!["e1"]));
        assert_eq!(entry("built"), (Truth::Unknown, vec!["e1"]));
        assert_eq!(entry("tests_only"), (Truth::Unknown, vec!["e1"]));
        let explained = |claim: &str| -> Vec<String> {
            decision.explanation.as_ref().expect("explained")["claims"][claim]["because"]
                .as_array()
                .expect("because")
                .iter()
                .filter(|reason| reason["evidence"] == "e1")
                .map(ToString::to_string)
                .collect()
        };
        for claim in ["named", "built", "tests_only"] {
            assert_eq!(
                explained(claim),
                [r#"{"evidence":"e1","reason":"invalidated","status":"excluded"}"#],
                "{claim}"
            );
        }
        assert_eq!(
            explained("helper"),
            [r#"{"evidence":"e1","status":"applied"}"#]
        );
    }

    /// The order forms read the history: a record excluded as `revision_mismatch` or `expired`, or
    /// invalidated for the claim, is still the first record of its kind. It is listed under a claim
    /// only because an evidence match the claim reaches reads it, never because of an order form.
    /// The explanation names each order form with its value, as an action's reasons do.
    #[test]
    fn an_order_form_reads_every_record_the_exclusion_stages_set_aside() {
        let ir = crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
                 artifacts: {a: {}, up: {}}\nevidence_kinds: {k: {max_age: 1h}, m: {}}\n\
                 claims: {red_first: {true_when: {first: {kind: k, result: red}}}, \
                 k_before_m: {true_when: {before: {first: k, then: m, subject: a}}}, \
                 green: {true_when: {evidence: {kind: k, result: green}}}, \
                 fixed: {true_when: {all: [{first: {kind: k, result: red}}, \
                 {evidence: {kind: k, result: green}}]}}, \
                 both: {true_when: {all: [{claim: red_first}, {not: {claim: k_before_m, is: false}}]}}}\n\
                 invalidation: {r: {upstream: up, invalidates: [fixed]}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles");
        let case = read_case(
            "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r2}, up: {revision: u2}}\n",
        )
        .expect("reads");
        let record = |id: &str, kind: &str, result: &str, revision: &str, at: &str, up: &str| {
            let result = if result.is_empty() {
                String::new()
            } else {
                format!("result: {result}\n")
            };
            read_evidence(&format!(
                "format: canon-evidence/1\nid: {id}\nkind: {kind}\n{result}subject: a\n\
                 subject_revision: {revision}\nobserved_at: {at}\nupstream_revisions: {{up: {up}}}\n"
            ))
            .expect("reads")
        };
        let decide = |evidence: &[EvidenceRecord]| {
            evaluate_with(
                &ir,
                &case,
                evidence,
                Supplied {
                    at: Some("2026-10-02T12:00:00Z"),
                    ..Supplied::default()
                },
            )
            .expect("decides")
        };
        let green = record("g", "k", "green", "r2", "2026-10-02T11:30:00Z", "u2");
        let diff = record("m1", "m", "", "r2", "2026-10-02T11:40:00Z", "u2");
        for (red, reason) in [
            (
                record("e", "k", "red", "r1", "2026-10-02T11:00:00Z", "u2"),
                crate::model::ExclusionReason::RevisionMismatch,
            ),
            (
                record("e", "k", "red", "r2", "2026-10-01T11:00:00Z", "u2"),
                crate::model::ExclusionReason::Expired,
            ),
        ] {
            let decision = decide(&[red, green.clone(), diff.clone()]);
            let entry = |claim: &str| decision.claims.get(&ClaimId::new(claim)).expect("declared");
            for claim in ["red_first", "k_before_m", "both", "green", "fixed"] {
                assert_eq!(entry(claim).value, Truth::True, "{claim} {reason:?}");
            }
            for claim in ["red_first", "k_before_m", "both"] {
                assert!(entry(claim).excluded_evidence.is_empty(), "{claim}");
            }
            for claim in ["green", "fixed"] {
                assert_eq!(entry(claim).excluded_evidence, [exclusion("e", reason)]);
            }
        }
        // Invalidated for `fixed`: its evidence match does not read the red record, its order form
        // does. `green` reads both records, which disagree.
        let moved = record("e", "k", "red", "r2", "2026-10-02T11:00:00Z", "u1");
        let decision = decide(&[moved, green.clone()]);
        let entry = |claim: &str| decision.claims.get(&ClaimId::new(claim)).expect("declared");
        assert_eq!(entry("fixed").value, Truth::True);
        assert_eq!(
            entry("fixed").excluded_evidence,
            [exclusion("e", crate::model::ExclusionReason::Invalidated)]
        );
        assert_eq!(entry("red_first").value, Truth::True);
        assert_eq!(entry("green").value, Truth::Unknown);
        assert_eq!(entry("k_before_m").value, Truth::Unknown, "no `m` record");
        assert_eq!(entry("both").value, Truth::Unknown);
        let explained = &decision.explanation.as_ref().expect("explained")["claims"];
        assert_eq!(
            explained["k_before_m"],
            serde_json::json!({"because": [
                {"before": "k", "subject": "a", "then": "m", "value": "unknown"},
            ], "value": "unknown"})
        );
        assert_eq!(
            explained["both"]["because"],
            serde_json::json!([
                {"claim": "k_before_m", "value": "unknown"},
                {"claim": "red_first", "value": "true"},
            ])
        );
    }

    /// A record given more than once counts once when every copy equals the first
    /// (story:evidence-redelivery): the decision is the one without the copy, byte for byte, the
    /// explanation included, wherever the copy stands, and the explanation lists the id once.
    #[test]
    fn an_equal_copy_of_a_record_counts_once() {
        let (pass, fail) = (record("e1", "pass"), record("e2", "fail"));
        let single = decide(CASE, &[pass.clone(), fail.clone()]).expect("decides");
        let expected = render(&single);
        for evidence in [
            vec![pass.clone(), fail.clone(), pass.clone()],
            vec![pass.clone(), pass.clone(), fail.clone()],
            vec![fail.clone(), pass.clone(), pass.clone(), pass.clone()],
        ] {
            let decision = decide(CASE, &evidence).expect("an equal copy is not refused");
            assert_eq!(render(&decision), expected, "{evidence:?}");
            assert_eq!(
                decision.explanation.as_ref().expect("explained")["computed_from"]["evidence"],
                serde_json::json!(["e1", "e2"])
            );
        }
    }

    /// Many small calls from a thread with a small stack start at most one thread for the deep
    /// stack, however many calls there are (story:evaluate-without-thread-per-call).
    #[test]
    fn two_thousand_calls_spawn_at_most_one_thread() {
        const CALLS: usize = 2000;
        let spawned = std::thread::Builder::new()
            .stack_size(256 << 10)
            .spawn(|| {
                let (ir, case) = (ir(), read_case(CASE).expect("case"));
                for at in 0..CALLS {
                    let evidence = [read_evidence(&record(&format!("e{at}"), "pass")).expect("e")];
                    let decision =
                        evaluate_with(&ir, &case, &evidence, Supplied::default()).expect("decides");
                    assert_eq!(decision.claims.len(), 5);
                }
                SPAWNED.with(|spawned| spawned.get())
            })
            .expect("the caller's thread starts")
            .join()
            .expect("the caller's thread ends normally");
        assert!(
            spawned <= 1,
            "{CALLS} calls started {spawned} threads for the deep stack"
        );
    }

    /// Runs `body` on a fresh thread with a small stack and fails, rather than hangs, when it does
    /// not end in 20 s.
    fn on_a_small_thread<T: Send + 'static>(body: impl FnOnce() -> T + Send + 'static) -> T {
        let (sent, received) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .stack_size(256 << 10)
            .spawn(move || {
                let _ = sent.send(body());
            })
            .expect("the caller's thread starts");
        received
            .recv_timeout(std::time::Duration::from_secs(20))
            .expect("the call ends in 20 s and does not panic")
    }

    /// Work that recurses far deeper than the caller's stack holds runs, since it runs on the
    /// worker's.
    #[test]
    fn work_runs_on_the_deep_stack_not_the_callers() {
        fn descend(levels: usize) -> usize {
            let frame = std::hint::black_box([1u8; 16 << 10]);
            if levels == 0 {
                return frame[0] as usize;
            }
            descend(levels - 1) + frame[levels % frame.len()] as usize
        }
        // 400 frames of 16 KiB: 6.4 MiB, against the caller's 256 KiB.
        let (name, depth) = on_a_small_thread(|| {
            on_deep_stack(move || {
                (
                    std::thread::current().name().map(str::to_owned),
                    descend(400),
                )
            })
        });
        assert_eq!(name.as_deref(), Some("canon-eval"));
        assert_eq!(depth, 401);
    }

    struct Boom(u8);

    /// A panic in the work reaches the caller with its payload, and the next call works.
    #[test]
    fn a_panic_in_the_work_reaches_the_caller_and_the_next_call_works() {
        let (payload, next, spawned) = on_a_small_thread(|| {
            let caught = std::panic::catch_unwind(|| {
                on_deep_stack(|| -> u8 { std::panic::panic_any(Boom(7)) })
            })
            .expect_err("the panic reaches the caller");
            let payload = caught.downcast_ref::<Boom>().map(|boom| boom.0);
            let next = on_deep_stack(|| 5);
            (payload, next, SPAWNED.with(|spawned| spawned.get()))
        });
        assert_eq!(payload, Some(7));
        assert_eq!(next, 5);
        assert_eq!(spawned, 1, "the worker survived the panic");
    }

    /// Work that calls evaluation again runs it where it is, on the thread of the outer call: the
    /// nested call keeps the deep stack the worker already has and starts no second worker. The
    /// threads are compared, since the sum and the count of started threads (a second worker would
    /// be counted on the worker's own thread, not the caller's) come out the same without it.
    #[test]
    fn work_that_evaluates_again_runs_on_the_thread_of_the_outer_call() {
        let (sum, threads, spawned) = on_a_small_thread(|| {
            let caller = std::thread::current().id();
            let (sum, threads) = on_deep_stack(|| {
                let outer = std::thread::current().id();
                let inner = on_deep_stack(|| std::thread::current().id());
                let deeper = on_deep_stack(|| on_deep_stack(|| std::thread::current().id()));
                (
                    on_deep_stack(|| 20) + on_deep_stack(|| on_deep_stack(|| 1)),
                    (caller, outer, inner, deeper),
                )
            });
            (sum, threads, SPAWNED.with(|spawned| spawned.get()))
        });
        let (caller, outer, inner, deeper) = threads;
        assert_eq!(sum, 21);
        assert_ne!(
            outer, caller,
            "the outer call runs on the worker, not the caller"
        );
        assert_eq!(
            inner, outer,
            "the nested call runs on the outer call's thread"
        );
        assert_eq!(
            deeper, outer,
            "a call nested twice runs on the outer call's thread"
        );
        assert_eq!(spawned, 1);
    }

    /// Callers on different threads do not wait for each other: each work waits, inside, for the
    /// other's to start, which one shared worker could never give.
    #[test]
    fn callers_on_different_threads_run_at_the_same_time() {
        let (to_first, first_hears) = std::sync::mpsc::channel::<()>();
        let (to_second, second_hears) = std::sync::mpsc::channel::<()>();
        let wait = std::time::Duration::from_secs(20);
        let first = std::thread::spawn(move || {
            on_deep_stack(move || {
                to_second.send(()).expect("second is there");
                first_hears.recv_timeout(wait).is_ok()
            })
        });
        let second = std::thread::spawn(move || {
            on_deep_stack(move || {
                to_first.send(()).expect("first is there");
                second_hears.recv_timeout(wait).is_ok()
            })
        });
        assert!(
            first.join().expect("first ends"),
            "first never heard of second"
        );
        assert!(
            second.join().expect("second ends"),
            "second never heard of first"
        );
    }

    /// A copy that differs from the first is refused as `duplicate-identifier`, naming and citing
    /// the record, whichever copy comes first.
    #[test]
    fn a_differing_copy_of_a_record_is_refused() {
        let (pass, fail) = (record("e1", "pass"), record("e1", "fail"));
        for evidence in [vec![pass.clone(), fail.clone()], vec![fail, pass]] {
            let refusal = decide(CASE, &evidence).expect_err("refused");
            assert_eq!(refusal.code(), "duplicate-identifier");
            assert_eq!(
                refusal.to_string(),
                "evidence `e1` is given more than once with different content"
            );
            assert_eq!(refusal.evidence(), [EvidenceId::new("e1")]);
        }
    }
}
