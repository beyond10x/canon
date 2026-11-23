#![forbid(unsafe_code)]

//! The `canon` command line's definition: its commands, their arguments and its exit statuses.
//!
//! The binary in `main.rs` parses and dispatches it; `canon-docs` walks it to generate the CLI
//! reference, so the reference cannot drift from the commands that exist.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// A document was read and rejected.
pub const REJECTED: u8 = 1;
/// A document could not be read.
pub const UNREADABLE: u8 = 2;
/// The command line was not understood: an unknown command or option, a missing required one, or
/// a value clap refuses (`EX_USAGE` of BSD `sysexits.h`).
pub const USAGE: u8 = 64;

/// Every exit status `canon` uses and what it means.
pub const EXIT_STATUSES: [(u8, &str); 4] = [
    (
        0,
        "Success: the document is valid or compiled, the case was evaluated, every conformance \
         scenario ran and passed, a check or a simulation found nothing, the scenarios were \
         generated, a protocol stays above its floor, or a mutation run reported no weak \
         point.",
    ),
    (
        REJECTED,
        "Rejected: a document was read and rejected, an evaluation, a check, a simulation or a \
         mutation run was refused, a conformance scenario failed or was unreadable, no scenario \
         ran, a requirement check failed, a check or a simulation found something, generation was \
         refused, a protocol is below its floor, or a mutation run reported a weak point.",
    ),
    (
        UNREADABLE,
        "Unreadable: an input file or directory could not be read, or an output file could not \
         be written.",
    ),
    (
        USAGE,
        "Usage error: the command line was not understood (an unknown command or option, a \
         missing required option, or a value that cannot be parsed); nothing was read.",
    ),
];

#[derive(Debug, Parser)]
#[command(
    name = "canon",
    version,
    about = "Evidence-governed protocol toolchain"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Parse and validate a `protocol/1` document, composed with the documents it imports.
    Validate {
        /// The protocol document to validate.
        #[arg(long)]
        path: PathBuf,
        /// A `protocol/1` document the protocol imports, directly or through another import.
        /// Repeat it once per document, in any order; each import is checked against its pinned
        /// revision and digest. Without it, a protocol with imports is refused.
        #[arg(long)]
        import: Vec<PathBuf>,
    },
    /// Compile a valid `protocol/1` document, composed with the documents it imports, into one
    /// closed `canon-ir/1` and print it.
    Compile {
        /// The protocol document to compile.
        #[arg(long)]
        path: PathBuf,
        /// A `protocol/1` document the protocol imports, directly or through another import.
        /// Repeat it once per document, in any order: the order changes no byte of the IR.
        #[arg(long)]
        import: Vec<PathBuf>,
    },
    /// Evaluate a case under a compiled protocol from an evidence set: every claim, and each
    /// declared obligation, action and outcome. Print the `canon-decision/1` document.
    Evaluate {
        /// The compiled protocol: `canon-ir/1` exactly as `canon compile` prints it.
        #[arg(long)]
        ir: PathBuf,
        /// The `canon-case/1` case snapshot.
        #[arg(long)]
        case: PathBuf,
        /// A directory holding only `canon-evidence/1` records, one per `*.yaml` or `*.json` file.
        #[arg(long)]
        evidence: PathBuf,
        /// A `canon-authority/1` document: which capabilities are granted, granted only with
        /// approval or denied, each at a scope or at every scope. An action is admissible only when
        /// every capability it requires is granted at the scope it requires it; without it, none
        /// is granted. A decision that gives a case revision applies only while the case snapshot
        /// is at that revision; its principal is recorded as given. A `ground_evidence` id is
        /// checked by id and by the `subject_revision`, `observed_at`, `upstream_revisions` and,
        /// where a claim demands independence, `producer` of the record under it, not by the rest
        /// of its contents: with other contents the entry still applies, and the decision changes
        /// only where a claim reads them. Evidence IDs must be immutable once an approval cites
        /// them.
        #[arg(long)]
        authority: Option<PathBuf>,
        /// A `canon-decisions/1` document: the explicit decisions taken, each naming the decision,
        /// the outcome, the principal and the case revision it was taken at. An outcome that requires
        /// a decision is legitimate only with one taken at the case snapshot's revision. A
        /// `ground_evidence` id is checked by id and by the `subject_revision`, `observed_at`,
        /// `upstream_revisions` and, where a claim demands independence, `producer` of the record
        /// under it, not by the rest of its contents: with other contents the entry still applies,
        /// and the decision changes only where a claim reads them. Evidence IDs must be immutable
        /// once an approval cites them.
        #[arg(long)]
        decisions: Option<PathBuf>,
        /// The evaluation instant, in UTC as `YYYY-MM-DDTHH:MM:SSZ`, or `YYYY-MM-DDTHH:MM:SS.fffZ`
        /// with one to nine fraction digits. Evidence older than its kind's `max_age` at this
        /// instant, counted in whole seconds, does not apply; without it, no evidence expires.
        #[arg(long)]
        at: Option<String>,
        /// Strict freshness, for a caller that relies on it: refuse a record of a kind with a
        /// `max_age` that gives no `observed_at` (`undated-evidence`) or was observed later than
        /// `--at` by more than SKEW (`future-evidence`), and a run without `--at` of a protocol
        /// with such a kind (`missing-instant`). SKEW is the clock skew allowed, written as a
        /// `max_age` is (`30s`, `5m`), `0s` when not given; other text is refused as
        /// `invalid-skew`. Without this flag, such records apply and nothing expires without
        /// `--at`.
        #[arg(long, value_name = "SKEW", num_args = 0..=1, default_missing_value = "0s")]
        strict: Option<String>,
    },
    /// Check a `protocol/1` document over its whole finite state space: report outcomes no state
    /// reaches, actions whose precondition holds in no state, claims, obligations, preconditions and
    /// outcome requirements that read evidence no action produces and no other protocol supplies,
    /// outcomes that rest on evidence an authority-requiring action may produce yet hold without
    /// any authority decision, each declared property that fails, with a counterexample, and each
    /// action that declares `runs_only_when` and is admissible or approval-required in a state
    /// where that predicate is not true (`ordering-guard-failed`, last), with that state: the
    /// guard is proved per state, not along an order of actions, and the summary counts the states
    /// without the guards. An
    /// evidence kind that declares `supplied_by` counts as produced without authority and stays
    /// governed when an action that requires a capability may also produce it, so an outcome a
    /// supplied record alone makes legitimate is reported as a bypass. A state is a set of evidence records,
    /// authority decisions and explicit decisions; a record an invalidation rule can keep from a
    /// claim is there in any combination of observed before and after each upstream artifact
    /// moved, one record each. Evidence an order form reads (`first`, `before`, `after`, `since`, `descended`)
    /// is also there with earlier records at a superseded revision, or with no known instant, in
    /// every order of first observations; evidence a `descended` form and another order form read
    /// is also there in three more histories: records that name no item and keep their instants,
    /// earlier records observed after the current ones, and earlier records of two results
    /// observed at two instants, the lowest or the highest result first; evidence an `after` or a
    /// `since` compares with its result
    /// also with a later record of that result, placed anywhere in that order or at no known
    /// instant; and evidence a `since` reads as its last kind also with a last record at a
    /// superseded revision, placed in that order on its own, with or just after its own first
    /// observations or a later record it is compared with. Evidence an evidence match
    /// demanding independence reads (`independent_on`) is there from a producer independent on
    /// exactly each demand, in every combination of demands met, and without a producer when
    /// something that demands nothing reads it. Evidence a `children` form reads (`case_outcome`)
    /// is also there with earlier records at a superseded revision, each record the outcome of a
    /// child case of its own, and with no place in the order, which the form does not read; an
    /// action that opens a child case (`case_open`) counts as producing `case_outcome` about its
    /// `case_open` subject when the protocol reads `case_outcome` about that artifact by name, in
    /// a `children` form or an evidence match, otherwise without a subject, and a `children` form
    /// naming a `subject` reads only records about it; and a protocol that reads `case_outcome`
    /// with both a `children` form and an order form is refused (`ordered-child-outcomes`). Where
    /// a `children` form declares `count: N` of 2 or more, its `case_outcome` evidence is also
    /// there with up to N more records, each a child case of its own, written in a state as
    /// `` evidence `case_outcome` about `b` result `r` 2 more times `` and in the bound refusal
    /// as `` more records of evidence kind `case_outcome` about `b` N ``. Each
    /// outcome, action and property is decided over the
    /// evidence, authority, decisions and case inputs it reads (its slice), with the findings and
    /// witness states the whole state space gives, and the summary counts the whole state space; a
    /// check in which one slice holds more than 65536 states is refused, naming that outcome,
    /// action, property or `runs_only_when` and the slice's dimensions, and a whole state space too large to count
    /// is refused naming the whole state space and its dimensions.
    Check {
        /// The protocol document to check.
        #[arg(long)]
        path: PathBuf,
        /// A `canon-properties/1` document: properties declared beside the protocol, each saying an
        /// action's or outcome's status is independent of a claim. Each is checked in every state.
        #[arg(long)]
        properties: Option<PathBuf>,
        /// A `protocol/1` document the protocol imports, directly or through another import.
        /// Repeat it once per document, in any order; the protocol is checked composed with them.
        #[arg(long)]
        import: Vec<PathBuf>,
    },
    /// Walk every state a case of a `protocol/1` document can reach from the empty case, one step
    /// at a time (or, when evidence kinds declare `required_at_open: true`, from each case that
    /// holds one record of every such kind, of any class, which the opener hands in, its path
    /// `empty`), and report the states no sequence of steps gets out of, each action admissible
    /// in no reachable state, and each outcome legitimate in no reachable state. A step adds one
    /// current record an admissible action may produce (`may_produce`), or one of each kind of a
    /// group the action records together (`records_together`), never one of those kinds alone,
    /// moves a non-empty subset of the artifacts an admissible action declares in `revises` to a
    /// new revision, alone or with one such record or group added at the new revision (a revise
    /// step, tried after the action's record steps, subsets in order of their sorted ids,
    /// written `action x revises a` or `action x revises a and produces …`), adds one record of
    /// a kind that declares `supplied_by` (at any step), moves an artifact that declares
    /// `revised_by` to a new revision with no record added (the external revise step, written
    /// `protocol q revises a`, tried after the supplied steps and before the child-outcome and
    /// expiry steps: a case of the named protocol may revise it between two evaluations; Canon
    /// checks nothing about that protocol), adds one child case's `case_outcome` record, lets one record class of a kind that declares `max_age` expire,
    /// grants or denies one capability some action requires, or takes one decision some outcome
    /// requires; no action runs. Since a step adds one record outside a group, where an action's
    /// guard closes on its first record the walk never holds two records of one taking together
    /// unless the action declares them a group, and an outcome that needs both is reported as
    /// legitimate in no reachable state. Per action that opens a child case
    /// (`case_open`) the walk counts the children that have not ended: none, one, or two or more.
    /// A step of the action raises the count and adds the child's `case_outcome` record `open`,
    /// as a runner writes it when it opens the child, where the protocol reads the result `open`
    /// of that record and some reader of the record demands no independence; where every reader
    /// demands independence (`independent_on`), the runner's record may meet no demand and the
    /// step adds none. A child's outcome arrives while the count is above none,
    /// and an outcome other than `open` lowers it, or from two or more may also leave it at two
    /// or more, and replaces an `open` record, since one record is one child, but, where the step
    /// adds the record, never the last one while a child that has not ended is left. A child of an
    /// opening action records `case_outcome` about the artifact the action's `case_open` names
    /// when the protocol reads `case_outcome` about that artifact by name, otherwise without a
    /// subject. A state holding an `open` record, current or held as expired, keeps the count of
    /// some action that produces that record above none, because an `open` record is a child that
    /// has not ended. A state names each action with a count above none
    /// (`action a opened a child`, `action a opened 2 or more children`). Where a `children` form
    /// declares `count: N`, the walk also counts the children it opened per `case_outcome`
    /// dimension the form reads, up to N, and a state writes the count
    /// (`` 2 children in evidence `case_outcome` about `b` ``, `1 child`, `3 or more children` at
    /// N of 3, with `(1 at the current revision)` when a form says `revision: current`). An
    /// expired
    /// `case_outcome` record still counts for a `children` form: with no evidence match reading
    /// `case_outcome` it does not expire, and otherwise it is held as expired, absent to every
    /// evidence match. A revision of an artifact removes the current records about it, except
    /// that a `case_outcome` record a `children` form reads becomes an earlier record (written
    /// `earlier`), which a `children` form without `revision: current` still reads, and turns a
    /// record with that artifact upstream into one observed before it moved (written `observed
    /// before upstream a moved`), which its invalidation rules keep from their claims. A state
    /// holds no revision number, and the case revision never moves: capability and taken
    /// decisions stay, so a denial lasts. The records of a kind no match names an artifact for
    /// are taken to be about the first artifact no match of the kind names, except the
    /// `case_outcome` records of children that actions about several artifacts open
    /// (`case_open`): each is about its action's `case_open` subject (written `evidence
    /// `case_outcome` about b`), so a revision of an artifact sets aside only the child outcomes
    /// about it. The records are split that way only when some action `revises` an artifact or
    /// some artifact declares `revised_by`; otherwise no revision can set a child's outcome
    /// aside, and the protocol is walked and written as without the split.
    /// A child that has not ended when its subject is revised may end against the
    /// earlier revision: its outcome then arrives as an earlier record (a step written `… earlier`)
    /// as well as a current one, whatever artifact the child was opened about; for the artifact
    /// the space already uses, only in a protocol whose `children` form says `revision: current`
    /// or whose evidence match reads `case_outcome`, since no other claim tells the two apart. An
    /// action without `revises` has no revise step. A state is open when action (record and revise), supplied,
    /// external revise, child-outcome, expiry and authority steps lead from it to a legitimate
    /// outcome; decisions do not count. A state only an external revise step opens waits, and
    /// when some artifact declares `revised_by` the summary counts the waiting states as waiting
    /// on supplied evidence, expiry or a revision by another protocol (`supplied evidence, a child
    /// outcome, expiry or a revision by another protocol` when an action opens a child case). A state that is not open is a stuck state when no action is admissible or
    /// approval-required, and a cycle when no action is approval-required and every record step
    /// of every admissible action returns the same state; when a revise step leads out of a
    /// cycle, it says every record step and ends `every revision by action x leads to a state no
    /// outcome is reachable from`. Each is printed once, at its smallest state, with the shortest
    /// path that reaches it, each denied capability that blocks an action whose precondition is
    /// true there with the two forms that end the case after the denial (an outcome with
    /// `requires: {decision: ..., when: ...}`, or an action without the capability whose record
    /// an outcome reads), each decision that would make an outcome legitimate there, and how
    /// many other such states contain it. A state that is not open, in which a denial blocks an
    /// action whose precondition is true and a decision step leads to a legitimate outcome, ends
    /// by that decision: it is no finding, and the summary counts it (`D ending by a decision
    /// after a denial`, only when D is above 0). An open state in which no outcome is legitimate, no
    /// action is approval-required, and every step of every admissible action, record or revise,
    /// returns the state is counted as waiting. The summary
    /// counts the reachable states, per outcome those in which it is legitimate, the waiting
    /// states, the findings and the stuck and cycle states they stand for. A protocol with an
    /// order form (`first`, `before`, `after`, `since`, `descended`) is refused (`ordered-evidence`),
    /// and so is a protocol whose `children` form counts more than 64 children
    /// (`child-count-bound`), naming the first declaration that holds one, and
    /// a walk that would visit more than 65536 states (`reachable-state-bound`), which reports
    /// nothing of what it saw.
    Simulate {
        /// The protocol document to simulate.
        #[arg(long)]
        path: PathBuf,
    },
    /// Generate `canon-conformance/1` scenarios into an empty directory: one minimal witness per
    /// declared outcome and per action blocked in some state, over the whole state space that
    /// `canon check` covers. Each names the protocol path, as given, as its fixture, so
    /// `canon conform run` runs them from the working directory `canon generate` ran in. Refused,
    /// writing nothing: a protocol path that resolves outside the working directory, an outcome no
    /// state reaches, a protocol that yields no scenario, an outcome or action identifier holding
    /// a character a file name cannot hold (`/ \ : * ? " < > |` or a control character), two
    /// scenario file names equal up to letter case or Unicode normalisation, a protocol with
    /// imports (a scenario names one fixture), a non-empty `--out`, and a whole state space of
    /// more than 65536 states, which `canon check` may still decide slice by slice. It reads no
    /// `runs_only_when`: the scenarios, the output and the exit status are the same with or
    /// without it. On failure, nothing this run created is left.
    Generate {
        /// The protocol document, as a relative path without `..` components.
        #[arg(long)]
        path: PathBuf,
        /// The directory to write the scenarios to: absent or empty. When absent it is created, with
        /// any missing parents; an existing one, or a symbolic link to one, is written into as it
        /// is.
        #[arg(long)]
        out: PathBuf,
    },
    /// The semantic difference between two compiled revisions of one protocol: every change, each
    /// classified as `TIGHTENING`, `RELAXATION`, `BREAKING`, `EXPANSION` or `NO SEMANTIC CHANGE`
    /// and naming the identifiers it touches. Two revisions of different protocols are refused.
    Diff {
        /// The earlier revision, as `canon compile` prints it.
        #[arg(long)]
        from: PathBuf,
        /// The later revision, as `canon compile` prints it.
        #[arg(long)]
        to: PathBuf,
    },
    /// Check that a compiled protocol stays above a compiled organization floor: every change from
    /// the floor to the protocol, classified as `canon diff` classifies it, is a `TIGHTENING`, an
    /// `EXPANSION` or `NO SEMANTIC CHANGE`. Prints `floor: holds` and the three counts and exits 0;
    /// otherwise prints `floor: breached` and each `RELAXATION` and `BREAKING` change, naming the
    /// identifiers it touches, and exits 1. The floor and the protocol keep their own ids,
    /// revisions and descriptions, which are not compared. The protocol does not name its floor;
    /// pass a compiled import as the floor to check that a refinement only tightens it. An added
    /// action is a `RELAXATION`, not an `EXPANSION`, when it repeats an action the floor guards
    /// with a capability and lacks that capability: its precondition is implied by the floor
    /// action's (the same, the always-true one, or some of its conjuncts) and it produces and
    /// revises at least what that action does (an action that does neither is repeated only by
    /// one that does neither). A floor does not pin a guard: `canon diff` classifies a removed
    /// `runs_only_when` as `NO SEMANTIC CHANGE`, so the floor holds for a protocol that drops a
    /// guard the floor declares.
    Floor {
        /// The organization's floor, a `protocol/1` document compiled as `canon compile` prints it.
        #[arg(long)]
        floor: PathBuf,
        /// The protocol checked, `canon-ir/1` as `canon compile` prints it; compile a protocol
        /// that imports others with its `--import` documents first.
        #[arg(long)]
        ir: PathBuf,
    },
    /// Measure how strong a scenario set is: apply five mutation operators to a `protocol/1`
    /// document (drop a member of an `all`, turn an `all` into an `any`, remove an action's
    /// precondition, remove one of its capability requirements, change the result an evidence
    /// match names to another result the protocol names for that kind), run each mutant against
    /// the `canon-conformance/1` scenarios of the directory whose fixture is the protocol file,
    /// handed in as that fixture (a scenario that names the protocol only as an import pins it by
    /// digest and is not run), and classify it with the semantic difference from the protocol
    /// (`canon diff`). Prints one line per mutant: `killed` with the first scenario that fails,
    /// `survived`, `weak point` for a survivor whose difference holds a `RELAXATION`, or
    /// `invalid` with the code of each problem when the mutant does not compile; each compiled
    /// mutant with its categories, then a summary line. Exits 1 when at least one weak point is
    /// reported, and 0 otherwise. Refused, exit 1, running nothing: no scenario names the protocol
    /// file as its fixture, a scenario file does not parse, the protocol does not compile, or a
    /// scenario whose fixture it is does not pass on the protocol itself. A protocol, scenario
    /// directory or scenario file that cannot be read exits 2.
    Mutate {
        /// The protocol document to mutate, the fixture of the scenarios that test it.
        #[arg(long)]
        path: PathBuf,
        /// The directory of scenarios each mutant runs against; every `*.yaml` file in it is one
        /// scenario.
        #[arg(long)]
        scenarios: PathBuf,
    },
    /// Check a set of `protocol/1` documents against each other, the names one gives another that
    /// `canon check` of one document never resolves. Prints one line per finding, its stable code
    /// first, then a summary line counting the protocols, the openings and the supplied kinds
    /// checked: `case-open-cycle`, the set's case openings form a cycle; `unknown-opened-protocol`,
    /// an action's `case_open` names a protocol not in the set; `undeclared-child-outcome`, a
    /// protocol names an outcome of a child case about an artifact (an outcome a `children` form
    /// lists, or the result an evidence match or an order form reads of `case_outcome`, about the
    /// form's `subject`) that no protocol it opens about that artifact declares, a form without a
    /// subject checked against every protocol it opens, `open` never one, and a name about an
    /// artifact one of whose opened protocols is not in the set not checked; `unknown-supplier`, an
    /// evidence kind's `supplied_by` names a protocol not in the set; `unproduced-supplied-kind`,
    /// no action of that supplier lists a kind of the same id in `may_produce`;
    /// `duplicate-protocol-id`, two or more files of the set declare one protocol id, naming the id
    /// and each file's path: a runner resolves an id to one protocol and `case_open` names no
    /// revision, so the files are not pooled and nothing is resolved against that id;
    /// `unsupplied-at-open`, last, an action opens a case under a protocol of the set that declares
    /// an evidence kind with `required_at_open: true`, and the action's protocol declares no kind
    /// of that id, or the action's precondition, when true, does not imply a record of it (an
    /// evidence match of the kind does, at either value; `all` when one member does, `any` when
    /// every member does; a claim test `is: true` or `is: false` through the claim's `true_when`;
    /// an order form naming the kind, when true; never `not`, `is: unknown`, an input test, a
    /// `children` form or the action's `runs_only_when`), named per protocol, action and kind. Each finding
    /// names the protocol, the action, declaration or kind, and the missing id. Exits 0 when the
    /// check found nothing, 1 when it found something. Each document is compiled on its own: one
    /// that does not parse or compile, or that has imports, is refused, exit 1, and a file that
    /// cannot be read exits 2, each naming its path; the files are read in sorted order of their
    /// paths, and when more than one is refused, the first of them in sorted order of the paths
    /// decides the exit status.
    CheckSet {
        /// A protocol document of the set. Repeat it once per document, in any order: the order
        /// changes no byte of the report, and a path given twice, under two spellings or through
        /// a symbolic link, is read once.
        #[arg(long, required = true)]
        path: Vec<PathBuf>,
    },
    /// Conformance scenarios.
    Conform {
        #[command(subcommand)]
        command: ConformCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConformCommand {
    /// Run every `canon-conformance/1` scenario of a registry directory and report each.
    Run {
        /// The registry directory; every `*.yaml` file in it is one scenario.
        #[arg(long, default_value = "conformance/scenarios")]
        scenarios: PathBuf,
        /// A `canon-requirements/1` catalogue of the normative requirements the registry answers
        /// for. After the run, every implemented requirement must be covered by a passing
        /// scenario, no scenario may cover one the catalogue does not list, no planned one may be
        /// covered, and every evaluate step must give byte-identical output when run again and with
        /// its `evidence`, `authority` and `decisions` lists reversed. Adds a `requirements:`
        /// summary line.
        #[arg(long)]
        requirements: Option<PathBuf>,
    },
}
