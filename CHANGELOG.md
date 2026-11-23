# Changelog

## [0.15.0] - 2026-11-24

A minor release: three new optional keys in `protocol/1`, each closing a gap the runbook consumer
named. Each key is absent from every existing protocol, and when absent nothing is
written to `canon-ir/1`: `canon validate`, `compile`, `check`, `simulate` and `check-set` give the
same bytes as 0.14.0 on every existing fixture (852 comparisons; only the wave's new fixtures
differ), and so does `conform run` on the 30 existing scenarios.
Install the CLI or depend on the libraries from Git tag `0.15.0`.

What changes, and for whom (read against the diff since 0.14.0):

- Anyone whose action must follow another step. The action key `runs_only_when: <predicate>` names
  a guard, and `canon check` proves that every state in which the action can run satisfies it; a
  state that does not is the finding `ordering-guard-failed`. A guard that is the literal `all: []`
  is refused as `vacuous-runs-only-when`; other always-true guards are not detected. `canon evaluate`,
  `simulate` and `generate` never read it. `canon floor` does not pin a guard: a refinement that
  drops one still holds. `check-set` and `unproduced-evidence` do not read guards yet.
- Anyone who supplies evidence to a child case when it opens. The evidence-kind field
  `required_at_open: true` (it needs `supplied_by` on the same kind, else `at-open-without-supplier`)
  states that the child needs a current record of the kind from the moment it opens. `canon
  check-set` reports `unsupplied-at-open` for an action that opens such a protocol when its
  precondition does not imply a current record of the kind (an `evidence` match, `all`, `any`, a
  `claim` that is true by such a match, an order form; `not`, `input` and `children` do not).
  `canon simulate` starts each case with one current record of every required kind. The library
  function `eval::missing_at_open` returns the required kinds with no applying record; `canon
  evaluate` bytes do not change. Adding the field is a tightening in `canon diff`, removing it a
  relaxation.
- Anyone whose protocol waits for several children. The key `count: N` on a `children` form (an
  integer of 1 or more; 0 is refused as `zero-child-count`) waits for N child cases that ended with
  a listed outcome: the claim stays unknown while fewer than N records are read or a child is
  open, is false when a child ended outside `in`, and is true at N. It combines with `subject`
  and `revision`. Five parallel readers are one claim instead of a chain of openers: on a copy of the
  consumer's `refresh-read`, `canon check` reads 665600 states in 5.4 s (133120 before) and
  `canon simulate` 2963 states, with 0 findings. `canon check` grows by a factor of N for each counted
  dimension. `canon simulate` refuses a count above 64 as `child-count-bound`. An opener whose
  guard stays open can open two children and fake the count: guard each opener with its `*_opened`
  record.

Other changes:

- `canon simulate` never counts more children than were opened when a revision and an earlier
  record meet in one step (found by the wave review).
- The published `protocol-1.schema.json` and the reference give `count` a minimum of 1.
- `canon check` and `canon simulate` help describe the count dimension and the children count a state
  prints.

Format note: a 0.14.0 reader refuses a protocol that uses any of the three keys (`unknown field`),
and refuses a `canon-ir/1` document that holds them (not canonical text). Protocols without the keys
read the same in both versions.

Not covered: `count` forms with different counts over one dimension in `canon check`; supplied
`case_outcome` records count once per class in `canon simulate`; with `revision: current`, two
cases count too low in `canon simulate` (never above the true count).

## [0.14.0] - 2026-11-23

A minor release: one new command, `canon check-set`, and two changes to what `canon simulate`
reports. They close three gaps the runbook consumer (`bf-runbook`) named. There is no change to
`protocol/1`, `canon-ir/1`, `canon-case/1` or the `canon-decision/1` model. `canon check`,
`compile`, `generate`, `conform run` and `evaluate` give the same bytes as 0.13.0 on every fixture
(808 runs). Install the CLI or depend on the libraries from Git tag `0.14.0`.

What changes, and for whom (read against the diff since 0.13.0):

- Anyone who keeps one protocol per file. `canon check-set --path <file>`, repeatable, checks the
  references between the files that `canon check` reads one at a time. Finding codes:
  `unknown-opened-protocol`, `undeclared-child-outcome`, `unknown-supplier`,
  `unproduced-supplied-kind`, `duplicate-protocol-id` and `case-open-cycle`; a file with `imports:`
  is refused as `error[protocol-imports]`. A supplied kind counts as produced when the supplier has
  an action whose `may_produce` lists a kind of that id. Exit 0 clean, 1 on a finding, 2 on a file
  that cannot be read. A file named twice, also as `./a.yaml` or through a symbolic link, is read
  once. On the consumer's 30 protocol files: `checked: 30 protocols, 32 openings, 1 supplied kind,
  0 findings`, exit 0.
- Anyone who runs `canon simulate` on a protocol with children. A child is opened together with its
  running `case_outcome` record `open`, as the runner writes it. An outcome that requires a `none`
  record while a `children` claim is not yet true is now reported as reached (`unreached-outcome`
  no longer fires for it), and the walk is smaller: the consumer's `needs-you` protocol goes
  from 24410 states (12.9 s) to 1431 (0.5 s), `refresh` from 1607 to 717. 23 fixtures print a
  different `simulate` report (each `report.txt` is in the commit).
- Anyone whose protocol refuses an approval by capability. `canon simulate` no longer reports a
  stuck state for a state in which a denied capability blocks an action and a decision step leads to
  a legitimate outcome: the summary counts it as `N ending by a decision after a denial`. A finding
  that holds a denial now names the action it blocks and the two forms that end the case. This
  reverses part of rule R4 of the wave 9 simulate review (a decision step does not decide whether a
  state is open) for states with a denial.

Breaking, for the output of `canon simulate` only:

- A protocol whose only findings were such states now exits 0 where 0.13.0 exited 1. Three
  fixtures lose findings (`decision-grounds` 2 to 1, `pending-decisions` 4 to 2,
  `check/authority-bypass` 2 to 1); four more gain the denial clause (`capability-scope`,
  `supplied-evidence/bypass`, `supplied-evidence/bypass-unsupplied`, `record-group-large/chain`).
- The state counts and the `terminable` counts of protocols with several opening actions change.
  A script that parses the `simulate` report by line must read the new clause and summary count.

## [0.13.0] - 2026-11-23

A minor release: one new option, `canon evaluate --strict`, and four commands that do less work for
the same bytes. There is no change to `protocol/1`, `canon-ir/1`, `canon-case/1` or the
`canon-decision/1` model. Install the CLI or depend on the libraries from Git tag `0.13.0`.

What changes, and for whom (read against the diff since 0.12.0):

- Anyone who runs `canon evaluate` on evidence they do not fully trust. `--strict [<SKEW>]` refuses
  a decision instead of evaluating it when evidence of a kind with `max_age` has no `observed_at`,
  has one after `--at` plus the skew, or when `--at` is missing. The refusal reasons are
  `missing-instant`, `undated-evidence`, `future-evidence` and `invalid-skew`. It covers every
  accepted record, including the ones the order forms (`first`, `before`, `after`, `since`) read.
  Without `--strict` every byte is as in 0.12.0; with it, where nothing is refused, the bytes are
  the same too.
- Anyone who runs `canon conform`, `canon check`, `canon simulate` or `canon evaluate` on large
  inputs. Output is byte-identical to 0.12.0 on the 173 `protocol/1` fixtures and the hand-written
  edge cases of the review. Measured on the reviewer's inputs: conform compiles a fixture once per
  scenario instead of once per step (23 compiles to 1, 31 % fewer instructions); `canon check`
  builds no explanation (46 to 0 on the 48-state fixture, 33 % fewer); `canon evaluate` reads each
  timestamp of a history once per claim walk (10,240 reads to 160, 49 % fewer instructions on 256
  records); authority lookup reads one capability's grants (CPU at 2,048 grants 114.8 ms to
  62.1 ms).
- Anyone who runs `canon generate` where another process can write to `--out`. Files are written
  into a private staging directory `--out/.canon-generate-<pid>-<n>` (mode 0700), placed by hard
  link and removed on failure only while they still have the identity the run gave them. A
  filesystem without hard links gets a direct write. A rerun after a killed run exits 1 with
  `out-not-empty` and now names the leftover staging directory.
- Anyone who relies on persisted approvals. The documentation of `canon evaluate --help` and of the
  evidence grounds now states that an approval cites evidence by ID and that the contents under an
  ID must not change; it also states which fields decide whether a ground holds (`subject_revision`,
  `observed_at`, `upstream_revisions`, `producer`). No behaviour changed. An optional digest per
  ground is planned as its own story.
- Tooling that compares `canon-decision/1` bytes. `computed_from.semantics.canon` reads `0.13.0`.
  The conformance expectations that pin the version moved to `0.13.0`; other expected output did
  not change.
- Rust consumers: `eval::evaluate_with_freshness` is new; no existing public type or function
  changed. `canon-cli` depends on `libc` on Unix (constants only, already in the lock file).
- Contributors: the CI gate runs the slow lane after `task check` and fails on it; the four new
  slow tests of `check` are in it.

## [0.12.0] - 2026-11-22

A minor release: `canon floor` and `canon simulate` give answers they did not give in 0.11.1, and
`canon check` reaches states it did not reach. There is no change to `protocol/1`, `canon-ir/1`,
`canon-case/1` or the `canon-decision/1` model. Install the CLI or depend on the libraries from Git
tag `0.12.0`.

What changes, and for whom (read against the diff since 0.11.1):

- Anyone who runs `canon floor` in CI. It exits 1 and prints `floor: breached` for an added action
  that repeats a guarded floor action without the floor's capability; in 0.11.1 it printed
  `floor: holds` and `EXPANSION`.
- Anyone who runs `canon check` on a protocol with a `descended` form beside a `first` form over one
  kind. The check reaches a state it missed (earlier records of two results at two instants), so it
  can report a finding 0.11.1 did not. A protocol of this shape has more states: the reviewer's
  protocol went from 41,472 to 53,760 (the bound is 65,536).
- Anyone who runs `canon simulate` on a protocol that opens a child about the space's own
  artifact. The child can end against the revision it was opened at, where the protocol has a
  predicate that tells an earlier record from a current one; the walk has more states and can
  report dead ends it did not. Protocols without such a predicate, and every fixture of the
  repository, give the same bytes.
- Anyone who runs `canon mutate` on a protocol with a decision outcome. The report prints a
  `skipped:` line for each, before its summary line.
- Tooling that compares `canon-decision/1` bytes. `computed_from.semantics.canon` reads `0.12.0`.
  The conformance expectations that pin the version moved to `0.12.0`; other expected output did
  not change.
- Rust consumers: no public type or function changed.

`canon floor`.

- An added action repeats a guarded floor action when its precondition is implied by the guarded
  one's (the same after unwrapping a one-member `all` or `any`, the always-true one, or a subset of
  its conjuncts) and it may produce and revise at least what the guarded one does. A repeat without
  a capability the guarded action requires is a `RELAXATION`: ``RELAXATION: action `a` added without
  capability `c`, which the floor requires of `b` ``. A guarded action that produces and revises
  nothing is repeated only by an added action that does the same. The sentence in the help that
  documented this as a limit is gone.

`canon check`.

- Two history modes (`LOW_FIRST`, `HIGH_FIRST`) put the earlier records of the lowest, or the
  highest, result at the rank's instant and the others 15 seconds after, so a `descended` true
  beside a `first` true over earlier records of two results is reached. They enumerate only the
  earlier sets of two or more classes (35 history values for three classes, not 45). With three or
  more results the middle one is not observed alone first (`story:check-descended-three-result-history`).
  `canon check --help` names the modes.
- `unproduced-evidence` names a decision outcome whose condition reads evidence nothing produces.
- `canon check` holds one slice's states at a time; `canon generate` keeps all, as before. Peak
  memory of a 4-outcome protocol with 32,768 states a slice: 67 MB before, 23.7 MB after.

`canon simulate`.

- The limit sentence of the help is replaced by one clause stating the condition above. One pinned
  unit test now expects 11 states, not 9.

Validation.

- `validate` looks claim, evidence-kind, artifact and dimension references up in one index. Compile
  and check of 40,000 chained claims took 60 s of CPU and now take 0.35 s; errors and their order
  are unchanged.

Tests and gates (not shipped to users).

- ESS moves from 0.97.3 to 0.97.4 (`ess/ess-inputs.yaml`, `check.yml`, with the archive's checksum,
  which `ess_gate.rs` now compares).
- The scenarios the generator writes for the investigation fixtures are committed
  (`crates/canon/tests/generated/investigation-scenarios.txt`, 248 scenarios, SHA-256 digest of the
  fixtures on its first line). The gate lane reads them and regenerates all fixtures but five that
  take over 5 s, then compares bytes (3.9 to 6.7 s); the slow lane regenerates all.
  `task regenerate-scenarios` writes the file. Mutant M3 of the generator now fails the gate lane.
- The invalidation cost tests count claim evaluations and marks instead of comparing two wall-clock
  times: 1,275 evaluations for a chain of 50 and 80,200 for 400. They pass 20 runs in a row at host
  load 95 to 112.
- The slow lane (`task test-slow`) ended 62 of 62 within the bound at host load about 44, and its CI
  step no longer has `continue-on-error`.
- The layout test flags an undeclared `.rs` file in a tests subdirectory, the classifier step of
  `check.yml` refuses an answer outside its three words, and the `canon-docs` layout test is pinned.

Known limits.

- A change that makes one claim evaluation slower, without evaluating more often, is not seen by the
  invalidation cost tests (`story:count-tests-see-a-slower-evaluation`).
- The `Documentation site` workflow can fail on its first run after a release and needs a manual
  dispatch (`story:pages-deploy-uses-the-artifact-it-uploaded`).

## [0.11.1] - 2026-11-22

A patch release: evaluation no longer starts a thread for every call, and the ESS pin moves. There
is no change to the public API, to `protocol/1`, `canon-ir/1`, `canon-case/1` or the
`canon-decision/1` model, or to the command line. Install the CLI or depend on the libraries from
Git tag `0.11.1`.

What changes, and for whom (read against the diff since 0.11.0):

- Tooling that compares `canon-decision/1` bytes. `computed_from.semantics.canon` reads `0.11.1`.
  The conformance expectations that pin the version moved to `0.11.1`; other expected output did
  not change.
- Rust consumers: no public type or function changed. `b10x_canon::eval::evaluate`,
  `evaluate_with` and `read_ir` keep their signatures.

Evaluation.

- `evaluate`, `evaluate_with` and `read_ir` run their recursive steps on a deep-stack thread
  (`DEEP_STACK`, 64 MiB reserved) so the depth a protocol may nest does not depend on the caller's
  stack. 0.11.0 started and joined one such thread for each call. Each calling thread now starts
  one worker thread with its first call and reuses it until the calling thread ends: a unit test
  holds that 2,000 calls from a thread with a 256 KiB stack start at most one thread
  (`eval::tests::two_thousand_calls_spawn_at_most_one_thread`), and
  `crates/canon/tests/evaluate_many_small_cases.rs` holds that many small cases from such a thread
  give the decisions of the large-stack caller. Results and refusals are unchanged, and a protocol
  nested to the bound still evaluates from a small stack.
- Measured on the `loom` harness (release build, 2,000 one-record `evaluate_with` calls, host load
  140 to 175, 5 runs each): 186 to 769 µs a call in 0.11.0 and 21.7 to 25.5 µs in 0.11.1. With 8
  calling threads of 2,000 calls each, a worker per calling thread cost 4.9 to 7.1 µs a call across
  threads; a single worker for the process, tried and not kept, cost 16.6 to 24.8 µs.
- Callers on different threads have a worker each and run at the same time. A call made by work
  already on the worker runs where it is and waits for nothing. A panic in the work reaches the
  caller with its payload and the worker serves the next call. A call made while the calling
  thread's locals are destroyed (from a destructor, after the worker handle is gone) runs on a
  thread of its own, as every call did in 0.11.0. A child of `fork` starts its own worker and never
  uses the one it copied from its parent.
- What it costs: a stack of 64 MiB, reserved and not touched, for each thread that ever evaluates,
  for as long as that thread lives.
- `unsafe`: the crate's one `unsafe` is a lifetime erasure of the job handed to the worker, in
  `crates/canon/src/eval/worker.rs`, with the reason it is sound written beside it. `lib.rs` now
  reads `#![deny(unsafe_code)]` where it read `#![forbid(unsafe_code)]` (a `forbid` cannot be
  allowed in a module), and `worker.rs` alone allows it. A test (`unsafe_confined`) scans every
  file under `crates/canon/src` and fails when another file allows or holds `unsafe`.
  `website/docs/reference/evaluation.md` describes the worker.

ESS.

- The pin moves from 0.77.0 to 0.97.3: `ess/ess-inputs.yaml`, the `ESS_VERSION` and `ESS_SHA256` of
  `.github/workflows/check.yml`, `AGENTS.md`, and the constants and messages of the tests that name
  it. The specification (`ess/`) changes only in that `requires:` line.

For contributors.

- New tests: `adversary_eval_worker` (calls evaluated where the thread's worker handle is being
  destroyed, in each of the three orders; a thousand short-lived callers leave no worker behind; a
  call in a `fork` child of a thread that evaluated returns), `evaluate_many_small_cases` and
  `unsafe_confined`, all in `crates/canon/tests/main.rs`, and five unit tests in `eval`.
- The ESS negative control `an_older_requires_fails_step_1` names its release with
  `ESS_TOOLCHAIN`: ESS 0.97.3 refuses a `requires:` that differs from its own release when no
  toolchain of that release is at hand, and under `ESS_TOOLCHAIN` it runs as the pinned release and
  fails an older `requires:` only under `--strict-requires`.

## [0.11.0] - 2026-11-22

Protocols order evidence by revision, an outcome's decision can carry a condition, an artifact can
name the protocol that revises it, and an action can record several kinds at once. `canon floor`
checks a protocol against an organization floor, `canon mutate` measures a scenario set, and
`canon simulate` walks revisions. Canon is also tested faster and the gate reads one pinned
compiler. Install the CLI or depend on the libraries from Git tag `0.11.0`.

There is no `protocol/1` break: the new keys are optional, and a protocol that uses none of them
parses, compiles and evaluates to the same bytes as under 0.10.0.

What breaks, and for whom (read against the diff of `crates/canon/src/model` and
`crates/canon/src/ir` since 0.10.0):

- Rust consumers that build these types with struct literals or match them exhaustively.
  `Artifact` gains `revised_by`, `Action` gains `records_together` (a list of the new `RecordGroup`,
  which holds `kinds`), `EvidenceRecord` gains `item` (the new identifier type `ItemId`),
  `CaseArtifact` gains `descends_from`, and `Predicate` gains the variant `Descended`, which holds
  the new `EvidenceDescended` (`first`, `then`, `subject`). `OutcomeRequirement::Decision` no
  longer holds a `DecisionName`: it holds the new `DecisionRequirement` (`decision`, `when`), and
  `OutcomeRequirement::decision()` still returns the name. `validate::Problem` gains
  `EvidenceInDecisionCondition`, `InvalidRecordGroup` (with the new `RecordGroupFault`) and
  `SelfRevisedArtifact`. In `ir`, `Ir::artifacts` is now a map to the new `ir::Artifact`
  (`description`, `revised_by`) and no longer to `Described`, and `ir::Action` gains
  `records_together`. `simulate::Report` gains `revisions`.
- Tooling that compares `canon-decision/1` bytes. `computed_from.semantics.canon` reads `0.11.0`,
  and `computed_from.case.artifacts.<artifact>` adds `descends_from` as the case wrote it (`[]`
  when given empty) when the case gives one. The conformance expectations that pin the version
  moved to `0.11.0`; other expected output did not change.
- Tooling that reads `canon simulate` output for a protocol where an action declares `revises`, an
  artifact declares `revised_by`, or an action declares `records_together`. 0.10.0 walked no
  revision; the walk now has a revise step and a group step, so the states, the findings and the
  summary of such a protocol can differ, and a walk that was under the bound of 65,536 states can
  now refuse as `reachable-state-bound`. A protocol with none of them, and the festival fixtures,
  walk as before. The refusal `ordered-evidence` now names `descended` among the order forms.
- Tooling that reads `canon check` state counts. For a protocol that reads a `descended` form
  beside another order form over one evidence kind, the state count grows: the protocol of the
  review that found it, 72 to 216 states. A protocol with `descended` over one kind and `first`
  over another stays at 4,320 states.
- A `protocol/1` document, case or evidence record that uses a new key (`when`, `revised_by`,
  `records_together`, `descended`, `item`, `descends_from`) is refused by Canon 0.10.0 and earlier
  as an unknown field: every structure it sits in refuses unknown fields.
- `canon diff` classes for the new constructs: a `when` added to a decision or removed is a changed
  outcome requirement, `BREAKING`; a `descended` form changed in its records, results or subject
  is `BREAKING`, and added or removed as a member of an `all` it is `TIGHTENING` or `RELAXATION`
  as for any member; `revised_by` introduced, removed or replaced, and a `records_together` group
  added or removed, are each `NO SEMANTIC CHANGE`.

Writing protocols.

- `{descended: {first: {kind: K1, result: R1}, then: {kind: K2, result: R2}, subject: A}}`
  orders by revision and not by time (`CANON-ORDER-005`,
  `conformance/scenarios/revision-ordering.yaml`). It is `true` when an item that had `R1` in a
  record of `K1`, on a revision the case snapshot lists in the artifact's `descends_from`, has
  `R2` in a record of `K2` about the artifact's current revision; `unknown` with no such later
  record or when the snapshot gives no `descends_from` for the subject of one; otherwise `false`.
  Records pair on equal `item`; a record without an `item` takes no part, and `observed_at` is not
  read. One item is enough for `true`: an item that failed on an earlier revision, passed on the
  current one and also failed on the current one is `true`; the model's documentation shows how to
  add a second form to exclude it. `subject` is optional and names a declared artifact.
- `item` on an evidence record is opaque text without whitespace or a control character
  (`invalid-identifier` otherwise), compared for equality only. It takes part in the comparison of
  two copies of one record id. `descends_from` on a `canon-case/1` artifact lists every revision
  the current one descends from. Canon reads it and does not compute it; each entry is refused as
  `invalid-identifier`, `duplicate-identifier` when listed twice, or `self-descent` when it is the
  current revision.
- `requires: {decision: d, when: <predicate>}` on an outcome (`CANON-OUTCOME-003`,
  `conformance/scenarios/decision-condition.yaml`). The decision counts only while `when` is
  `true` at the evaluation, whenever it was taken. While `when` is `false` or `unknown`, the
  outcome is blocked by `when`'s reasons, and a decision given is neither counted nor refused, so
  the same decisions document counts at a later evaluation once `when` is `true`. `when` holds
  claims and case inputs only; an evidence match, an order form or a `children` form in it is
  refused as `evidence-in-decision-condition`. `when` beside any requirement but `decision` is a
  parse error. `canon-ir/1` writes `"when"` only when set.
- `revised_by: <protocol id>` on an artifact names the protocol whose cases revise it. `canon
  validate` refuses the protocol's own id as `self-revised-artifact` and checks the id as a
  protocol identifier. `canon-ir/1` writes it after the artifact's `description` only when set.
  `canon evaluate`, `canon check` and `canon generate` read nothing from it; `canon simulate` does,
  below.
- `records_together: [{kinds: [K1, K2, ...]}]` on an action names kinds its `may_produce` lists
  that one run records all together or not at all. `canon validate` refuses a group of fewer than
  two distinct kinds, a kind the action does not produce, the `case_outcome` kind of an action
  that opens a child case, and a kind already in an earlier group, all as `invalid-record-group`.
  It has no evaluation meaning; only `canon simulate` reads it. `canon-ir/1` writes it only when
  set, groups sorted.
- `b10x_canon::model::write` writes a `protocol/1` model back out as canonical JSON text that
  `parse` reads to an equal model (story:protocol-writer). It is pure and infallible, and
  `canon validate` stays the judge of validity. The round trip holds for a model in which every key
  is at most 1022 bytes as written; the module documentation lists what it does not keep
  (comments, YAML style, key order inside bodies, how a scalar was spelled) and the depth it needs.
- Compiled `canon-ir/1` of a protocol that uses none of the new keys is unchanged.

The command line.

- `canon floor --floor <protocol> --ir <canon-ir>` checks that a compiled protocol stays above a
  compiled organization floor: every change from the floor to the protocol, classified as `canon
  diff` classifies it, is `TIGHTENING`, `EXPANSION` or `NO SEMANTIC CHANGE`. It prints `floor:
  holds` and the three counts and exits 0, or prints `floor: breached` with each `RELAXATION` and
  `BREAKING` change and exits 1. The two keep their own ids, revisions and descriptions. The
  protocol does not name its floor; passing a compiled import as the floor checks that a
  refinement only tightens it. Library: `b10x_canon::floor`.
- `canon mutate --path <protocol> --scenarios <directory>` applies five operators to a protocol
  (drop a member of an `all`, turn an `all` into an `any`, remove an action's precondition, remove
  one of its capability requirements, change the result an evidence match names), runs each mutant
  against the scenarios of the directory whose fixture is the protocol file, and classifies it with
  `canon diff`. It prints one line per mutant (`killed`, `survived`, `weak point` for a survivor
  whose difference holds a `RELAXATION`, or `invalid`) and a summary, and exits 1 when at least one
  weak point is reported. Refused, running nothing: no scenario names the file as its fixture, a
  scenario does not parse, the protocol does not compile, or a scenario does not pass on the
  protocol itself. Library: `b10x_canon::mutate`.
- `canon simulate` gains these steps. A group step adds one record of each kind of a
  `records_together` group, never one of them alone. A revise step moves a non-empty subset of the
  artifacts an admissible action declares in `revises` to a new revision, alone or with one record
  or group of that action (written `action x revises a`). An external revise step moves an
  artifact that declares `revised_by` with no record added (written `protocol q revises a`);
  a state only that step opens is counted as waiting, and the summary then reads `waiting on
  supplied evidence, expiry or a revision by another protocol`. A revision removes the current
  records about the artifact, except that a `case_outcome` record a `children` form reads becomes
  an `earlier` record. Where some action revises an artifact or some artifact declares
  `revised_by`, the walk has one `case_outcome` dimension for each `case_open` subject (written
  `evidence `case_outcome` about b`), so a revision sets aside only the child outcomes about it, and
  a child that has not ended when its subject is revised may end against the earlier revision (a
  step written `… earlier`). A parent that revises the artifact a failed child was about and opens
  new children is walked to the outcome a `{children: {in: [...], revision: current}}` form reads.
  `CANON-COMPOSE-002` gains two steps. A protocol with a `descended` form is refused
  (`ordered-evidence`), as with the other order forms.
- `canon check` decides a `descended` form: its state space gives each record one `item` and each
  artifact its descent, and tests pin that it reaches every value of the form beside evidence
  matches, beside a `first` over its kind and beside another order form of one kind.
  `canon generate` writes the `item` and `descends_from: [r0]` a
  scenario for such a protocol needs. `canon generate` takes the state spaces `canon check` has
  already enumerated and no longer enumerates the whole product a second time; its code and the
  commit that made it state that every scenario and refusal is the same.
- Exit statuses: 0 also covers a protocol above its floor and a mutation run with no weak point; 1
  also covers a protocol below its floor, a refused or weak mutation run. `ess/domains/cli.yaml`
  declares `floor` and `mutate`, and `task ess-gate` fails when they differ from clap.

Documented limits.

- `canon floor` guards only what the protocol requires of an action it declares. An added action is
  an `EXPANSION` and every `EXPANSION` passes, so an action added to the protocol that produces a
  finding the floor guards, without the capability the floor requires, is not caught. A protocol
  that adds an artifact, a case input or an order form under an `all`, or changes the case an
  action opens, is `BREAKING` and below the floor though it may tighten.
- `canon simulate`: a child opened about the artifact the space already uses does not end against
  an earlier revision, and the walk of such a protocol is unchanged. It checks nothing about the
  protocol named in `revised_by`.
- `canon mutate` makes no mutant at a conditioned decision, and does not say it skipped the site.
- The `slow` lane of the test step may fail: 10 of its 63 tests were killed at 60 s on the
  workstation (see below).

For contributors.

- The test step of `task check` runs every test once under cargo-nextest: a test is killed at 60 s
  and counted as failed, the step prints its ten slowest tests and fails over 120 s
  (`.config/nextest.toml`, `Taskfile.yml`). Each crate has one integration-test binary
  (`tests/main.rs`), and test builds carry no debug information (`.cargo/config.toml`). Tests
  measured over 5 s run in `task test-slow`, which CI runs after `task check` with
  `continue-on-error`: the lane is not yet under the 60 s bound (`story:slow-lane-under-bound`).
  On the workstation, the step took 55.4 s on its own line (1,275 tests, slowest 16.9 s) and the
  slow lane killed 10 of 63; before the story, 972.7 s for 1,255 tests in 163 test binaries.
- `rust-toolchain.toml` pins rustc 1.100.0, read by both workflows. `scripts/ci-watch`
  (`task ci-watch -- <pull request or branch>`) exits non-zero unless every check passed and the
  job `canon / task check` reported, and exits 2 at once for a branch with no pull request.
- A pull request that changes only planning or prose files passes `canon / task check` without
  building or testing; a change under `website/` or a story file runs the documentation check only.
  The one path list is `scripts/classify-changes.sh`, tested in
  `crates/canon/tests/planning_only_classifier.rs`.

## [0.10.0] - 2026-11-21

Protocols import pinned protocols, capabilities carry a scope, authority decisions can be bound to
a case revision and to the grounds they rested on, a decision lists what it waits on, a
re-delivered evidence record counts once, a runner can check a call's arguments with Canon's own
validator, and `canon simulate` walks every state a case can reach. Install the CLI or depend on
the libraries from Git tag `0.10.0`.

What breaks, and for whom:

- Rust consumers that build these types with struct literals or match them exhaustively.
  `Protocol` gains `imports`, `EvidenceKind` gains `supplied_by`, `Action` gains `author` and
  `revises`, `CapabilityRequirement` gains `scope`, `EvidenceProduction` gains `decided_by`, the
  `children` form's `ChildOutcomes` gains `revision` and `subject`, `AuthorityDecision` gains
  `scope`, `principal`, `case_revision`, `ground_revisions` and `ground_evidence`,
  `ExplicitDecision` gains `ground_revisions` and `ground_evidence`, `Grant` gains
  `RequireApproval`, and `Decision` gains `missing_evidence` and `pending_decisions`. In `ir`,
  `Action` gains `scopes`, `author`, `judgement_decided` and `revises`, and `EvidenceKind` gains
  `supplied_by`. `conform::Scenario` gains `imports`, and `validate::Problem` gains a variant for
  each new refusal of a protocol document: `DuplicateScopeKey`, `SelfSuppliedEvidence`,
  `MissingImport`, `UnusedImport`, `DuplicateImportDocument`, `ImportCycle`,
  `ImportRevisionMismatch`, `ImportDigestMismatch`, `UndeclaredRefinement`, `ConflictingImports`,
  `UnrefinedRedeclaration`, `InvalidImport` and `InvalidImportDigest`.
- Tooling that compares `canon-decision/1` bytes. A decision may carry two new top-level
  sections, `missing_evidence` and `pending_decisions`, each written only when it has an entry,
  and `computed_from.semantics.canon` reads `0.10.0`. A reason names a scoped requirement with
  its `scope` and a bound `children` form with `"revision": "current"`, a `children` form that
  names a subject with `"subject"`, and a capability reason with `"decision": "none"` or an
  outcome reason `{"decision": …, "present": false}` adds `"stale_grounds"` when an entry set
  aside by its grounds would have covered it. In `computed_from`, an authority or explicit
  decision that names grounds adds `"ground_revisions"` and `"ground_evidence"`, each only when
  not empty. A protocol that uses none of the new fields compiles to the same `canon-ir/1` bytes
  as before.
- Tooling that relies on a repeated evidence id being refused. An evidence record given more
  than once now counts once when every copy equals the first, and the decision is the one
  without the copy. Only a copy that differs is refused, as `duplicate-identifier`, and the
  message changed from `` evidence `<id>` is given more than once `` to `` evidence `<id>` is
  given more than once with different content ``.
- Protocols whose argument schemas list numbers a double cannot tell apart. `canon validate` and
  `canon diff` compare argument-schema numbers by exact value: `9007199254740993` and
  `9007199254740992.0` were one `enum` value in 0.9.0, a duplicate `canon validate` refused, and
  are two now; `1` and `1.0` are still one.
- `canon diff` classes. A changed scope on a capability requirement is one requirement removed
  and one added. `supplied_by` introduced is `EXPANSION`; removed, or changed to another protocol,
  `BREAKING`. An action's `author`, or a production's resolved `decided_by`, turned to judgement
  is `TIGHTENING` and turned to program `RELAXATION`; an artifact added to `revises` is
  `RELAXATION` and one removed `TIGHTENING`. A `children` form whose `revision` or `subject`
  changes is a changed `children` form, `BREAKING`. An `enum` value of an argument schema is
  added or removed by exact value.

Writing protocols.

- Imports: a `protocol/1` document may list `imports`, each pinning a protocol's `revision` and the
  `sha256:` digest of its compiled `canon-ir/1`. `canon validate`, `canon compile` and
  `canon check` take the imported documents with a repeatable `--import <path>` and compose them
  into one closed `canon-ir/1` that names only the root protocol and holds no import reference;
  imports are transitive, a part reached twice is inlined once, and the order of the `--import`
  flags changes no byte. A root may redeclare an imported declaration only when the import lists
  its identifier under `refines`. Refused: `missing-import`, `unused-import`,
  `duplicate-import-document`, `import-cycle`, `import-revision-mismatch`,
  `import-digest-mismatch` (printing both digests), `undeclared-refinement`,
  `conflicting-imports` (two imports whose `canon-ir/1` entries for one identifier differ),
  `unrefined-redeclaration`, `invalid-import` and `invalid-import-digest`. `canon generate`
  refuses a root with imports, naming its first import. A conformance scenario names its own
  `imports`. Scenario `CANON-IMPORT-001`, `conformance/scenarios/imports.yaml`.
- Scoped capabilities: a capability requirement may carry a `scope`
  (`{capability: metrics.read, scope: {cluster: prod-eu, namespace: foo}}`); `canon validate`
  refuses a key named twice (`duplicate-scope-key`). `canon-ir/1` writes `"scope"` only on a
  requirement that has one (Atlas ADR 0083).
- `supplied_by` on an evidence kind names the protocol whose cases produce its records.
  `canon validate` refuses the protocol's own id (`self-supplied-evidence`, fixture
  `fixtures/investigation/invalid/supplied-by-self.yaml`). Adding it changes the protocol's IR
  digest, so a protocol that imports it pins a new one.
- Judgement marks, with no evaluation meaning: `author: program | judgement` and `revises: [...]`
  on an action, `decided_by: program | judgement` on a `may_produce` entry (absent, the action's
  `author`). `canon evaluate`, `canon check` and `canon generate` give the same bytes with or
  without them. `canon validate` refuses an artifact in `revises` that is not declared
  (`undeclared-artifact`, naming the action). `canon-ir/1` writes `"author"` and `"decided_by"`
  only for judgement and `"revises"` only when non-empty; the IR read-back refuses any other
  spelling as `malformed-input`.
- `{children: {in: [O, ...], revision: current}}` reads only the `case_outcome` records whose
  `subject_revision` is the snapshot's current revision of their `subject`; a record about
  another revision neither establishes nor contradicts the form, and with none read it is
  `unknown`. `revision: any`, or no key, reads every accepted record as before. Scenario
  `CANON-COMPOSE-002`, `conformance/scenarios/child-outcome-revision.yaml`.
- `{children: {in: [O, ...], subject: A}}` reads only the `case_outcome` records about the
  artifact `A`; a record about another artifact neither establishes nor contradicts the form. With
  `revision: current` as well, it reads only those among them at the current revision of `A`.
  Without `subject` it reads the records about every artifact, and compiles to the bytes it had
  before. `canon validate` refuses a subject that is not declared (`undeclared-artifact`).
  `canon-ir/1` writes `"subject"` only when the form names one, and two forms that differ only in
  `subject` are two members. Canon does not write these records: the runner that opens a child
  case writes one about the opening action's `case_open` subject, with the outcome `open` when it
  opens the child. Scenario `CANON-COMPOSE-003`, `conformance/scenarios/child-outcome-subject.yaml`.

Evaluating cases.

- A `canon-authority/1` entry may carry a `scope`, the decision `require_approval` beside
  `granted` and `denied`, a `principal` and a `case_revision`. Default deny: a requirement no
  applying entry covers is not granted. A grant without a value for a scope key covers every
  value of it. Among the covering entries `denied` wins over `require_approval`, which wins over
  `granted`. An entry with `case_revision` applies only while the snapshot's `revision` equals it,
  and to nothing when the snapshot has none; an entry without one applies at every revision. The
  principal is recorded as given and matched against nothing. Two entries equal in capability,
  scope, case revision and principal are refused (`duplicate-identifier`). Scenarios
  `CANON-SCOPE-001` and `CANON-AUTHORITY-002`.
- Decision grounds: an entry of `canon-authority/1` or `canon-decisions/1` may name the grounds
  it rested on, `ground_revisions` (artifact revisions keyed by artifact) and `ground_evidence`
  (evidence record ids). An artifact ground holds while its revision is the snapshot's current
  revision of the artifact; an evidence ground holds while a record with that id is applicable,
  the rule `missing_evidence` reads, so expiry and invalidation reach the decision. An id no
  record has does not hold and is not refused. An entry applies only while every ground holds,
  whatever it decided, so a denial whose ground moved lapses too. A requirement or an outcome
  left undecided by an entry set aside by its grounds alone lists the failed grounds under
  `stale_grounds`, each once: artifact grounds by artifact then revision,
  `{"artifact", "reason": "revision_mismatch", "revision"}`, then evidence grounds by id,
  `{"evidence", "reason"}` with the reason the record was excluded for, or
  `{"evidence", "present": false}`. An entry set aside by its case revision adds none. Grounds
  are not part of either document's duplicate key. Refused: a ground artifact, revision or
  evidence id that is not an identifier (`invalid-identifier`) and a ground artifact the protocol
  does not declare (`undeclared-artifact`). An entry without grounds evaluates as before. Scenario
  `CANON-GROUND-001`, `conformance/scenarios/decision-grounds.yaml`.
- An evidence record given more than once counts once when every copy equals the first: every
  field as written, `observed_at` compared as text (`12:00:00Z` and `12:00:00.000Z` differ), and
  `upstream_revisions` in any order. Every later stage reads each id once, so an equal copy of a
  `case_outcome` record is not refused as `duplicate-child-case`. A copy that differs is refused
  as `duplicate-identifier`. `canon evaluate` with several evidence files still names, in that
  refusal, every file that holds a record of the id. Scenario `CANON-EVIDENCE-004`,
  `conformance/scenarios/evidence-redelivery.yaml`.
- `missing_evidence` maps each admissible action whose `may_produce` is not empty to the kinds of
  that list with no applicable record, in identifier order: a record is applicable when the
  revision-binding and freshness stages leave it and no claim lists it in `excluded_evidence`.
  It claims nothing about running the action. Scenario `CANON-FRONTIER-001`.
- `pending_decisions` lists, under `authority`, one entry per requirement with decision `none` or
  `require_approval` that makes an action `approval-required`, with `options: ["granted",
  "denied"]`, and under `decisions`, one entry per outcome blocked on a decision not taken, or
  taken and set aside by its grounds, with `options: ["taken"]`, or `[]` when the snapshot has no
  `revision`. An entry does not copy `stale_grounds`. It is computed from the decision's own
  `actions` and `outcomes` sections and the case revision. Scenario `CANON-PENDING-001`.
- A record of a kind that declares `supplied_by` may name, in `upstream_revisions`, artifacts the
  protocol does not declare; they are carried and not read. Any other record naming one is still
  refused as `undeclared-artifact`. Scenario `CANON-SUPPLY-001`.
- Evaluation under invalidation rules is at most quadratic in the claim count, with every output
  byte-identical: a 400-claim chain with one rule per claim took 54.2 s before and 59 ms after in
  a debug build. `canon validate` finds the claims that reach evidence once, not once per rule.

Checking a call's arguments.

- `b10x_canon::validate::check_arguments(schema, arguments)` checks one call's arguments against
  an action's argument schema, `None` for an action that declares none, which takes only `{}`.
  It is pure and returns every refusal, `ArgumentsRefused::Arguments` with one `ArgumentRefusal`
  each (`pointer`, `keyword`, `reason`) in document order, or `ArgumentsRefused::Schema` for a
  schema outside the dialect, with the pairs `canon validate` reports. Canon itself still
  evaluates no call. The reference page on validation states each keyword's meaning.
- Numbers are compared by exact value, and the value Canon reads depends on how `serde_json` is
  built. A build of the workspace holds number text, because `b10x-canon-expr` turns on
  `arbitrary_precision`; every command-line test runs such a build. The `canon` you install with
  `cargo install --locked --git … canon-cli` or `--path crates/canon-cli`, and a runner whose own
  build does not turn on `arbitrary_precision`, hold doubles and 64-bit integers. The two
  answer differently, and only, here: a literal that is not exactly the integer or double read
  without `arbitrary_precision` (`9007199254740992.5`, `9007199254740993.0`,
  `-9223372036854775809`, `1e400`, `1e-400`), in arguments or in a schema a caller builds; a
  protocol literal the YAML reader rounds (`enum: [9007199254740993.0]` refuses the arguments
  `9007199254740993.0` where the text is held and accepts them where a double is); a schema
  double that is an integer beyond 64 bits (a protocol's `1e30` is 10^30 with text,
  1000000000000000019884624838656 with doubles); and a bound or an `integer` `enum` value written
  `-0`, 0 with text and refused by `canon validate` as not an integer with doubles.

Checking protocols.

- `canon check` decides each outcome, action and property over the dimensions it reads, so the
  bound of 65,536 states applies to each slice, not to the whole product. The summary still gives
  the whole product's state count, and every report of a protocol within the bound is
  byte-identical. A slice above the bound refuses the whole check as `state-space-bound`, naming
  the subject, the slice's count and its dimensions. `canon generate` keeps the whole-product
  bound.
- `canon check` counts a kind that declares `supplied_by` as produced, so it reports no
  `unproduced-evidence` for it; the kind stays governed when an action that requires a
  capability may also produce it.
- An action that declares `case_open` produces the `case_outcome` record about its `case_open`
  subject when the protocol reads `case_outcome` about that artifact by name, in a `children`
  form or an evidence match, and otherwise the record without a subject; never one about another
  artifact. `canon check` and `canon simulate` both follow this rule, and a `children` form that
  names a subject reads only the records about it.

The command line.

- `canon simulate --path <protocol>` walks every state a case can reach from the empty case. A
  step adds one record an admissible action may produce, adds one record of a supplied kind, adds
  one child case's `case_outcome` record, lets a record of a kind with `max_age` expire, grants or
  denies a capability, or takes a decision; no action runs. Per action that declares `case_open`
  the walk counts the children that have not ended, none, one, or two or more: a step of the
  action raises the count, a child's outcome arrives while it is above none, and an outcome other
  than `open` lowers it (from two or more it may also stay there) and replaces an `open` record
  the state holds, since one record is one child. A state names each action with a count
  above none (`` action `a` opened a child ``, `` action `a` opened 2 or more children ``), and a
  path writes the child's step as `` a child of action `a` records evidence `case_outcome` result
  `r` ``. It reports `stuck-state` and `cycle` (each once, at its smallest state, with its
  shortest path), `dead-action`, `unreached-outcome` and `no-outcome-reachable`, then a summary
  line, which counts states waiting `on supplied evidence, a child outcome or expiry` when an
  action opens a child case, and exits 1 on a finding. Refused: a protocol with an order form
  (`ordered-evidence`), `predicate-too-deep`, `claim-cycle`, and a walk of more than 65,536
  reachable states (`reachable-state-bound`), which reports nothing.
  `b10x_canon::simulate::simulate` is the library function. Known limits: no artifact is
  revised, and a step adds one record, so where an action's guard closes on its first record an
  outcome that needs two records of one taking is reported as legitimate in no reachable state.
- The `canon` command surface is declared in `ess/domains/cli.yaml` (domain `canon.cli`).
  `crates/canon-cli/tests/ess_commands_match.rs`, run by `task ess-gate`, fails when a subcommand
  or flag exists only in clap or only in `ess/`, or when requiredness or repeatability differ.

## [0.9.0] - 2026-10-30

Cases compose at run time: an action may open a child case, and a parent claim reads its
children's outcomes (Atlas ADR 0081). Breaking for Rust consumers that match exhaustively over
`Predicate` or build `Action` or `EvidenceRecord` values with struct literals. Install the CLI or
depend on the libraries from Git tag `0.9.0`.

- The ESS specification under `ess/` requires ESS 0.77.0; CI installs it.

Opening a child case.

- An action may declare `case_open: {protocol: P, subject: A}`: performing it opens a child case
  under protocol `P` for artifact `A`. Its authority is declared like any other action's; Canon
  opens nothing itself. `canon validate` refuses a malformed protocol id (`invalid-identifier`)
  and an opening that names the protocol's own id, naming the action (`self-opened-case`,
  fixture `fixtures/investigation/invalid/case-open-self.yaml`). `canon-ir/1` writes the opening
  on its action; the IR of a protocol without one is unchanged.
- `b10x_canon::ir::check_openings` checks a set of compiled protocols and refuses one whose
  openings form a cycle (`OpeningCycle`), named from its least protocol id along the openings
  back to it, for example `audit -> review -> audit`. A chain without a cycle passes. There is no
  run-time check.
- One commission per case: a child case is how work passes to another party; the reference says
  so.

Reading the children.

- Evidence kind `case_outcome`: a record names its child in `child_case` and gives the child's
  outcome as its result, `open` while the child runs. Refused: a `case_outcome` record without
  `child_case` (`missing-child-case`) or a result (`missing-child-outcome`), `child_case` on any
  other kind (`misplaced-child-case`), and two records for one child (`duplicate-child-case`,
  naming the child and both records).
- Predicate form `{children: {in: [O, ...]}}`: `true` once every child's outcome is in the set,
  `false` once any child ended outside it, `unknown` while a child is `open` or when the case has
  no child record. It reads every accepted `case_outcome` record, at any revision. `canon validate`
  refuses an empty set (`empty-child-outcomes`), a repeated outcome (`duplicate-child-outcome`)
  and `open` in the set (`open-child-outcome`). Requirement CANON-COMPOSE-001, scenario
  `conformance/scenarios/case-composition.yaml`.
- `canon diff` takes a change to a `children` form or to a `case_open` as `BREAKING`.
- In `canon check`, an action with `case_open` counts as producing `case_outcome`; evidence a
  `children` form reads gets a history dimension (earlier records at a superseded revision) and no
  place in the order. A protocol that reads `case_outcome` with both a `children` form and an
  order form is refused (`ordered-child-outcomes`). A protocol without either construct builds the
  same space as before.
- Breaking for Rust consumers: `Predicate` gains `Children(ChildOutcomes)`, `Action` gains
  `case_open: Option<CaseOpening>` and `EvidenceRecord` gains `child_case: Option<CaseId>`.

## [0.8.0] - 2026-10-29

Sub-second order and an order form that reads the latest record of a kind. Breaking for Rust
consumers that match exhaustively over `Predicate`. Install the CLI or depend on the libraries
from Git tag `0.8.0`.

- The ESS specification under `ess/` requires ESS 0.76.0; CI installs it.

Order forms compare `observed_at` at the resolution it is written in.

- An instant (`observed_at`, `--at`, a conformance step's `at`) may carry a fraction of one to nine
  digits after the seconds: `YYYY-MM-DDTHH:MM:SS.fffZ`. An empty fraction (`12:00:00.Z`), more than
  nine digits and a comma stay refused as `invalid-instant`, whose message now names both forms:
  `… is not an instant in UTC written YYYY-MM-DDTHH:MM:SSZ or YYYY-MM-DDTHH:MM:SS.fffZ (one to nine
  fraction digits)`.
- `first`, `before` and `after` compare every instant of one evaluation at one resolution: the
  coarsest precision written among the records with an `observed_at`. Each instant is read with
  its finer digits dropped; different instants are ordered and equal ones are not, and the 0.7.0
  rules apply to those instants as they did to seconds. Evidence all in milliseconds is ordered
  within a second (`12:00:00.200Z` before `12:00:00.500Z`); one record at `12:00:05Z` makes the
  evaluation compare seconds, so the same two records tie. One resolution keeps the order a weak
  order, so `canon check`'s state space, which writes whole seconds, still reaches every value the
  evaluator gives. Over whole seconds every value is the 0.7.0 value. Requirement CANON-ORDER-003,
  scenario `conformance/scenarios/evidence-order-subsecond.yaml`.
- Expiry (`max_age`) still counts whole seconds: it reads an instant's whole seconds and ignores
  its fraction.
- `Instant::digits` gives the number of fraction digits written and `Instant::ticks(digits)` the
  instant at that resolution; `Instant::interval` gives the interval the last written digit spans,
  in nanoseconds; `Instant::seconds` reads a fractional instant as its whole seconds.
- The published `Instant` JSON Schema pattern admits the fraction:
  `^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}(\.[0-9]{1,9})?Z$`.

An order form that reads the latest record of a kind.

- Add the order form `since` to `protocol/1` predicates:
  `{since: {last: K1, then: {kind: K2, result: R}, subject?: A}}`. It tests whether a record of
  `K2` with result `R` was observed after the latest record of `K1`, about that record's subject
  and subject revision. `K1` and `K2` may be one kind. It reads the evidence history as the other
  order forms do, at the same resolution:
  - `unknown` with no record of `K1`;
  - `false` with no record of `K2` with `R`;
  - `unknown` when a compared record has no `observed_at`.

  Otherwise each latest record of `K1` gives a value from the records of `K2` with `R` at its
  subject and subject revision, other than itself:
  - `true` when one is later;
  - `false` when every one is earlier, or there is none;
  - `unknown` otherwise.

  The form is `true` or `false` when every latest record agrees, and `unknown` otherwise. A
  `since` over one kind is never `true`. Requirement CANON-ORDER-004, scenario
  `conformance/scenarios/evidence-order-since.yaml`.
- `canon-ir/1` writes it as authored,
  `{"since": {"last": …, "then": {"kind": …, "result": …}}}`, with `subject` when named. Among the
  members of an `all` or `any`, `since` sorts after `not`, and two compare by last kind, later
  kind, result and subject. The IR of a protocol without `since` is unchanged.
- An evaluation's reasons list each `since` after every `after`, `before` and `first` reason:
  `{"since": K1, "then": K2, "result": R, "value": …}`, with `"subject"` when named.
- `canon validate` resolves both kinds and the subject, and refuses `since` in a discharge or
  applicability predicate. `canon diff` compares it structurally, so a changed `since` is
  `BREAKING`.
- In `canon check`, a protocol with `since` gains:
  - a later-record dimension for each result it compares, as for `after`;
  - a last-record dimension for each dimension it reads as `last`: none, or a record at a
    superseded revision placed in the order on its own, tied with or just after the dimension's
    rank or a later record a `since` compares, with no result or, for a one-kind `since`, its
    result (5 values for a two-kind `since`, 9 for a one-kind one).

  It counts `since`'s kinds for `authority-bypass` in every position. A protocol without `since`
  builds the same space, states and reports as before. A protocol whose space exceeds 65536
  states is refused as before; the `since` conformance fixture is such a protocol (210600 states).
- `canon generate` writes the last record at `r0`. The reference pages and the `protocol/1` JSON
  Schema name the form.
- Breaking for Rust consumers: `Predicate` gains the variant `Since(EvidenceSince)`, so an
  exhaustive match over it no longer compiles.

## [0.7.0] - 2026-10-29

An order form that reads a later record. Install the CLI or depend on the libraries from Git tag
`0.7.0`.

- Add the order form `after` to `protocol/1` predicates:
  `{after: {first: K1, then: {kind: K2, result: R}, subject?: A}}`, whether a record of `K2` with
  result `R` was observed strictly after the first record of `K1` (`K1` and `K2` may be one kind).
  It reads the evidence history as `first` and `before` do, records the exclusion stages keep from
  evidence matches included. `unknown` with no record of `K1`; `false` with no record of `K2` with
  `R`; otherwise `unknown` when a compared record has no `observed_at`, `true` when a record of `K2`
  with `R` is later than the first instant of `K1`, `unknown` when one is at that instant and none
  later (a record is not later than itself), `false` otherwise. Records of `K2` with another result
  take no part. Requirement CANON-ORDER-002, scenario
  `conformance/scenarios/evidence-order-after.yaml`.
- `canon-ir/1` writes it as authored, `{"after": {"first": …, "then": {"kind": …, "result": …}}}`
  with `subject` when named; among the members of an `all` or `any`, `after` sorts first, and two
  compare by first kind, later kind, result and subject. The IR of a protocol without `after` is
  unchanged.
- An evaluation's reasons list each `after` before the other order forms:
  `{"after": K1, "then": K2, "result": R, "value": …}`, with `"subject"` when named.
- `canon validate` resolves both kinds and the subject, and refuses `after` in a discharge or
  applicability predicate; `canon diff` compares it structurally like `first` and `before`.
- `canon check` gives a protocol with `after` a later-record dimension per compared result and
  dimension, places later records in the order with the first observations, and counts `after`'s
  kinds for `authority-bypass` in every position. A protocol without `after` checks as before.
- `ess/ess-inputs.yaml` and CI pin ess 0.75.0 (was 0.60.0); the ESS gate tests name the new pin.

## [0.6.0] - 2026-10-15

A normative requirements catalogue the conformance suite is checked against. Install the CLI or
depend on the libraries from Git tag `0.6.0`.

- Add the `canon-requirements/1` catalogue, `conformance/requirements.yaml`: the 15 `CANON-*`
  requirements Canon implements, each with its status (`implemented` or `planned`), statement and
  source. ESS declares it in the new domain `canon.conform`.
- `canon conform run --requirements <catalogue>` checks the registry against it after the run:
  every implemented requirement covered by a passing scenario (`uncovered: …`), no scenario
  covering a requirement the catalogue omits (`unlisted: …`), no planned requirement covered, and
  every evaluate step giving byte-identical output when run again and with its `evidence`,
  `authority` and `decisions` lists reversed (`nondeterministic: …`). The report ends with
  `requirements: <n> implemented, <n> covered, <n> planned`. Without the flag the report is
  unchanged. `task check` runs the registry with the catalogue.
- A catalogue that is refused is named by kind and position only: a wrong `format` or a key that
  is not a `CANON-` id is not quoted back.
- The specification is held to ess 0.60.0, which CI installs.

## [0.5.0] - 2026-10-15

One argument schema per action. Install the CLI or depend on the libraries from Git tag `0.5.0`.

- Each `actions:` entry of `protocol/1` takes an optional `arguments`: a JSON Schema in Canon's
  argument-schema dialect, a subset of JSON Schema 2020-12 with no references. Every schema names
  one `type`, the top level `type: object`; the dialect has `title`, `description`, `enum`,
  `properties`, `required`, a boolean `additionalProperties`, `items`, `minItems`, `maxItems`,
  `minLength`, `maxLength` and integer `minimum` and `maximum`, nested at most 32 deep. Absent, the
  action takes no arguments: a runner accepts only `{}`. Canon validates no call itself.
- `canon validate` refuses a schema outside the dialect as `action-arguments`, one problem per
  offending keyword, naming the action and the keyword's JSON Pointer.
- `canon compile` carries each schema unchanged into `canon-ir/1`, and writes no `arguments` for an
  action without one.
- `canon diff`: a schema introduced or removed is `BREAKING`; one changed only in `title` or
  `description` is `NO SEMANTIC CHANGE`; one that accepts only documents the old one accepted is
  `TIGHTENING` (`arguments narrowed`), the reverse `RELAXATION` (`arguments widened`), anything
  else `BREAKING` (`arguments changed`). Every change is made under a new `protocol.revision`.

## [0.4.0] - 2026-10-13

Evidence requirements that demand an independent producer. Install the CLI or depend on the
libraries from Git tag `0.4.0`.

- Add `independence_dimensions` to `protocol/1`: the dimension ids a protocol demands independence
  on, each with an optional description. Canon gives no dimension a meaning.
- An evidence match takes `independent_on`, a non-empty list of declared dimensions. A record
  satisfies it only when it names a `producer` (new on `canon-evidence/1`) and the case's
  `independence` map (new on `canon-case/1`, written by the trusted context) lists every demanded
  dimension for that producer. A record that fails the demand is inapplicable to that match,
  never contrary: a claim resting only on such records is `unknown`, not `false`
  (`CANON-INDEPENDENCE-001`). Explanations list it as `not_independent`.
- Validation refuses an empty or duplicated `independent_on`, and an undeclared dimension in a
  match or in the case's `independence`.
- `canon diff`: a dimension added to a match read positively is `TIGHTENING`, removed is
  `RELAXATION`; a changed demand under `not` or in a claim read in reverse is `BREAKING`; a
  dimension declaration added is `EXPANSION`, removed is `BREAKING`.
- `canon check` and `canon generate` give records producer profiles for demanded kinds, one value
  per set of demands a record class meets; protocols without demands check as before.
- The specification is held to ess 0.59.0.

## [0.3.0] - 2026-10-13

Predicates over evidence order. Install the CLI or depend on the libraries from Git tag `0.3.0`.

- Add two predicate forms to `protocol/1`: `first: {kind, result}` holds when the first record of a
  kind has that result, and `before: {first, then}` holds when one kind's first record was observed
  strictly before another's; each may name a `subject`. Records are ordered on `canon-evidence/1`'s
  existing `observed_at`; no evidence field is added.
- Order forms read the case's evidence history, including records that revision binding,
  freshness or invalidation keep from evidence matches, so a failing result recorded at an earlier
  revision counts after a fix moves the subject to a later one. No records, a record without
  `observed_at`, a tie at the earliest instant or conflicting earliest results leave a form
  `unknown`; `before` is `false` once the later kind has a record and the earlier has none.
- Validation refuses an order form naming an undeclared kind or subject, and a `before` naming one
  kind twice (`self-ordered-evidence`). Action reasons and explanations name the forms; `canon diff`
  classifies a changed order form as `BREAKING`.
- `canon check` gains history and ordering dimensions for protocols that use order forms;
  protocols without them check as before. Conformance scenario `CANON-ORDER-001`.
- Known limit: a record on any earlier revision counts, whether or not the current revision
  descends from it.

## [0.2.0] - 2026-10-12

Case inputs and semantic diff. Install the CLI or depend on the libraries from Git tag `0.2.0`.

- Add `canon diff --from --to`: compare two protocol revisions on their compiled `canon-ir/1` and
  classify every change as `TIGHTENING`, `RELAXATION`, `BREAKING`, `EXPANSION` or
  `NO SEMANTIC CHANGE`, with the identifiers it touches. Changed predicates are ordered by sound
  structural implication, and a changed claim by the direction each reader reads it.
- Add case inputs to `protocol/1`: `case: inputs:` declares closed value lists, the `input`
  predicate leaf tests a case's value, and an obligation's `applicable_when` makes it
  `not_applicable` when FALSE, so one compiled protocol serves several profiles. `canon-case/1`
  gains `inputs`; a case missing a declared input, naming an undeclared one or giving an undeclared
  value is refused. Conformance scenario `CANON-INPUT-001`.
- `canon check` enumerates input values as a state dimension and names them in its summary.
- Existing protocols without `case` or `applicable_when` evaluate as before.

## [0.1.0] - 2026-10-05

The first source release. Install the CLI or depend on the libraries from Git tag `0.1.0`.

- Include the `b10x-canon` protocol model, strict validation, canonical compilation, three-valued
  evidence evaluation, authority and outcome decisions, invalidation and structured explanations.
- Include the `canon` CLI for validation, compilation, evaluation, finite-state checks, generated
  conformance scenarios and conformance execution.
- Add the `b10x-canon-expr` library: typed catalog expressions, Logos lexing, Pratt parsing,
  hygienic recipes, exact numeric comparisons, deduplicated acquisition plans and pure
  three-valued evaluation with retained-context replay.
- Specify expression wire contracts in ESS and enforce model parity. Raise the ESS gate and
  verified CI installation to 0.53.0.
- Add an assertion concept guide and generated contract/limit reference. Existing protocol
  evaluation and command-line behavior are unchanged.
