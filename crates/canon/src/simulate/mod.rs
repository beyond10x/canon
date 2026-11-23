//! A breadth-first walk over the states a case of a compiled protocol can reach from the empty
//! case (story:canon-simulate, with the rules refined after adversary pass 1, R1 to R6,
//! story:simulate-child-outcome-step, which lets a child case's outcome arrive after its opening,
//! and story:simulate-revise-step, which lets an admissible action move the artifacts it declares
//! in `revises` to a new revision), or, for a protocol with kinds required at open
//! (story:supplied-evidence-required-at-open), from a case that holds one record of each.
//!
//! [`simulate`] walks every reachable state, one step at a time, evaluates each with Canon's own
//! evaluator and reports what no single evaluation and no static check shows: states no sequence
//! of steps gets out of, with the shortest path that reaches each, actions that are never
//! admissible and outcomes that are never legitimate. It is a pure function of the compiled
//! protocol: no clock, network, filesystem or runner, and the same protocol gives the same
//! [`Report`], byte for byte. A step never runs an action.
//!
//! # States
//!
//! A state of the walk is a set of current records and a state of the space `canon check` covers
//! (`crate::check`, "The state space") with every evidence dimension at no record: its capability
//! decisions, its taken decisions and its case inputs. Each record is of one class of one evidence
//! dimension of that space, from a producer of one of its profiles, at the current revision and
//! observed after every move of an upstream artifact, or, after a revise step, before the move of
//! some upstream artifacts an action revises (the space's upstream vector); for each dimension,
//! class and vector the walk holds the down-closed set of profiles its records meet, as the
//! space's values do. A dimension has a vector per subset of the upstream artifacts some action
//! declares in `revises`, so a protocol that declares none has one per dimension and class. A
//! state holds no revision number: two sequences of steps that leave the same records reach the
//! same state, and a walk state is a state of the space `canon check` covers plus the marks
//! below. It is evaluated as
//! `canon check` evaluates a state, as a case snapshot with every artifact at one revision, no
//! termination and no evaluation instant. The walk never needs a dimension's table of values, so
//! the number of values of a dimension is never a reason to refuse (R1): its bound counts the
//! states it reaches.
//!
//! A state also holds one *mark* per action that declares `case_open`, in action order, kept
//! apart from its records (story:simulate-child-outcome-step, "Decided in review, after
//! adversary pass 1", D1): how many of the children that action's steps opened have not ended,
//! none, one, or two or more. A step of the action raises it by one, up to two or more, which is
//! when a runner has opened a child case; a child's outcome other than `open` lowers it. Mark
//! none is below one, and one below two or more; a state holds one thing per child a mark counts,
//! one or two. Evaluation reads only the records and the space's state, so two states that differ
//! only in marks evaluate the same; a walk state is a state of the space `canon check` covers
//! plus k marks, k the number of opening actions, and the walk reaches at most 3^k times the
//! states it reaches without marks. A protocol with no opening action has no mark.
//!
//! A state that holds a `case_outcome` record of the class `open`, current or held as expired,
//! holds some mark above none ("Decided in review, D5") on an action whose children record in
//! that record's dimension (story:child-outcome-subject): an `open` record is a child that has not
//! ended, and with every such mark at none no child is left to end, so no later outcome could
//! replace it. The reverse holds too (story:simulate-opens-child-with-its-record) in each
//! dimension the action's children record in that has the class and that some reader reads with
//! no independence demand: a runner writes a child's `open` record in the step that opens it, and
//! every producer's record meets that reader, so a mark above none comes with an `open` record
//! there, until an expiry or a revision removes it from a dimension that holds no expired or
//! earlier record, as it leaves every evidence match of a real case. In a dimension whose every
//! reader demands independence a runner's `open` record may meet no demand, which no reader reads
//! and the state without it stands for, so there a mark above none may come without an `open`
//! record. A child-outcome successor that would break either is not a successor: a child that
//! ends replaces its own `open` record, and leaves the one a child that has not ended holds.
//!
//! # Steps
//!
//! - **Action step.** Taken only from a state in which the action is `admissible`. It adds one
//!   current record of one kind of the action's `may_produce`, in one dimension of that kind, of
//!   one class of that dimension and from a producer of one of its profiles. Each combination is
//!   its own successor; a producer profile is a free choice of each step, as a runner may use any
//!   producer. A record already there, or one from a producer independent on more, adds nothing.
//!   An action that may produce nothing, or nothing a record can be about, has one step, which
//!   adds nothing. Every step of an action that opens a child case raises its mark and adds the
//!   child's `case_outcome` record of the class `open` (story:simulate-opens-child-with-its-record),
//!   current, from a producer of the empty profile, in each dimension its children record in that
//!   has the class `open` and that some reader reads with no independence demand (none in a
//!   dimension without the class, D-b, and none in one whose every reader demands independence,
//!   where a runner's record may meet no demand); the step keeps its text (D-a). It is a step when it
//!   only raises the mark or adds that record; a step that adds nothing, raises no mark (none is
//!   raised past two or more) and finds the `open` record there returns the same state. A child's
//!   other `case_outcome` records are not among those an opening action's step adds (`canon
//!   check` still counts the kind as produced, `check::produces`): they arrive by the
//!   child-outcome step.
//! - **Group step** (story:simulate-record-groups). An action that declares `records_together`
//!   records each group's kinds in one run, all of them or none: for each group, one step adds
//!   one current record of each member kind at once, each member in one dimension of its kind, of
//!   one class and from a producer of one profile, and each combination is its own successor, so
//!   a group step has the product of its members' successor counts. A member no record can be
//!   about adds nothing. A kind in a group is never added alone: the action's single-kind steps
//!   are those of its kinds in no group. It is an action step in every other way: it is taken
//!   only where the action is `admissible`, raises the action's mark, and returns the same state
//!   when it adds nothing. The declaration states a fact Canon cannot check, as `independent_on`
//!   does: every finding holds for a runner that records each group together or not at all, and
//!   a state holding one kind of a group alone is not a state of such a case. `canon validate`
//!   refuses a group naming `case_outcome` on an action that opens a child case, so a child's
//!   outcome is never grouped.
//! - **Revise step** (story:simulate-revise-step). An action that declares `revises` may move
//!   those artifacts to a new revision: taken only from a state in which it is `admissible`, one
//!   successor per non-empty subset of its `revises`, each combined with no record, then with
//!   each record of its single-kind steps and each combination of its group steps, as those
//!   steps take them. In the successor every artifact of the subset is at a new revision, then
//!   the record or the group is added at the current one. Canon revises nothing; the step says
//!   that a runner may move an artifact between two evaluations, where the protocol says an
//!   action does. A revision of artifact `a` changes a state as the space `canon check` covers
//!   models one (`crate::check`, "The state space"):
//!   - a current record about `a` leaves the state: revision binding keeps it from every
//!     evidence match, and nothing else reads it in a protocol the walk accepts, since order
//!     forms are refused;
//!   - in a dimension a `children` form reads, a current record about `a` becomes an earlier
//!     record of its class, at a superseded revision: every `children` form without
//!     `revision: current` still reads it (`eval/composition.rs`), and the state writes it
//!     `` evidence `case_outcome` about `a` result `r` earlier ``. It is the bit an expiry holds
//!     a `case_outcome` record in (R2a, below), so a record of one class both expired and set
//!     aside is one earlier record, as in the space's history dimension, and a dimension a revise
//!     step can set aside writes it `earlier` whichever step set it;
//!   - a current record of a dimension with upstream `a` becomes the one observed before `a`
//!     moved, written `` … observed before upstream `a` moved ``: the invalidation stage keeps it
//!     from the claims `a`'s rules invalidate;
//!   - records held as earlier, capability decisions, taken decisions and marks stay. The case
//!     revision does not move: an authority entry or a decision bound to it would lapse when it
//!     moved, so a denial a protocol leaves standing stays standing, and a finding with a denial
//!     is about a lasting denial.
//!
//!   The artifact a record is about is its dimension's: the one its matches name for a bound
//!   dimension, otherwise the first artifact no match of the kind names, as the space takes it.
//!   A revise step is an action step in every other way: it raises the action's mark, and it is
//!   no step when its successor is the state it starts from. An action without `revises` has no
//!   revise step, and a protocol in which no action declares one walks as before.
//! - **Supplied step.** A kind that declares `supplied_by` is produced by another protocol's
//!   cases (story:supplied-evidence), and no action step produces it: a record of it may arrive at
//!   any step, from any state, by the combinations an action step takes. A kind that also declares
//!   `required_at_open: true` (story:supplied-evidence-required-at-open) is in every initial state
//!   already (see Order); its supplied step stays, since a record of it can expire, or be set
//!   aside by a revision, and arrive again, and a record of another class can arrive beside it.
//! - **External revise step** (story:artifact-revised-by). An artifact that declares `revised_by`
//!   is revised by the cases of another protocol, and a case of it may move the artifact to a new
//!   revision between two evaluations: the step is offered in every state, one per such artifact
//!   in identifier order, and its successor is the revision of that artifact, as a revise step
//!   makes it, with no record added. It is no step when its successor is the state it starts from.
//!   It is no action step: it raises no mark, and it is outside the test that every step of every
//!   admissible action returns a state, as a supplied step is. Canon revises nothing, and nothing
//!   checks that the named protocol revises the artifact: the declaration states a fact Canon
//!   cannot see, as `supplied_by` does. Written `` protocol `q` revises `a` ``.
//! - **Child-outcome step.** For each opening action whose mark is above none, one current record
//!   of `case_outcome` arrives from one of its children, in the one dimension `canon check` counts
//!   its action as producing (story:child-outcome-subject): the one about its `case_open` subject
//!   when the protocol reads `case_outcome` about that artifact by name, in a `children` form or
//!   an evidence match, otherwise the one without a subject, and none when neither exists, since
//!   nothing reads the child's record then, by the combinations an action step takes: a runner
//!   records a child's outcome when the child ends, whether the opening action's guard is still
//!   open or not. Canon runs nothing; the step says that a runner may record it
//!   between two evaluations. A record of the class `open` keeps the mark: the child has not
//!   ended; the opening step added it already where it adds one, so it is a step only where an
//!   expiry or a revision removed it, where the opening added none, or with another producer
//!   profile (D-c). Any other class lowers it, from
//!   one to none, and from two or more to one or, since more children may remain, to two or
//!   more; it is a step when it lowers the mark, even with its class already there. A class
//!   other than `open`, in a state holding a record of the class `open` in that dimension, has
//!   more successors that also drop the `open` records: one record is one child
//!   (`duplicate-child-case` refuses a second record for it), so an open child that ends is
//!   replaced, not joined. A record of the class `open` held as expired (R2a, below) is an open
//!   child too, and is replaced the same way: when the state holds `open` both current and held
//!   as expired, the successors replace the current records, the one held as expired, or both,
//!   since either child may end first. A successor that drops the last `open` record of a
//!   dimension the opening step adds it in while a mark above none on an action whose children
//!   record in it is left is not one: that child has not ended, and its record stays (two
//!   children of one action, or of two actions about one subject, share the dimension's `open`
//!   record); in a dimension whose every reader demands independence the child left may be one
//!   whose record meets no demand. The step is written `` a child of action `a`
//!   records evidence `case_outcome` result `r` ``, `a` the action whose mark it lowers or keeps,
//!   followed by `` , replacing result `open` `` (`` result `open` expired ``, `` result `open`
//!   and result `open` expired ``) for a replacing successor. It is no action step: it makes no
//!   state a `cycle` and keeps none from being a `stuck-state`.
//! - **Expiry step (R2).** A kind that declares `max_age` may lose a record to time: one record
//!   class present of such a kind leaves the state, every producer's record of it. The space
//!   `canon check` covers has no representation of an expired record: it is atemporal, and an
//!   expired record is an absent one to every evidence match (`crate::check`, "The state space"),
//!   so the state without the class evaluates as a case in which those records are past their
//!   `max_age`. With it the step graph can loop.
//!
//!   `case_outcome` is the exception (R2a): the evaluator's `children` form reads every accepted
//!   `case_outcome` record, expired ones too (`eval/composition.rs`: an expiry does not undo a
//!   child's outcome), and every evidence match loses an expired one. When no evidence match
//!   reads `case_outcome`, it has no expiry step, which would change nothing the evaluator reads.
//!   When one does and a `children` form reads it too, the step holds the class's record as
//!   expired: one bit after the class's profiles, the record written as the history dimension of
//!   a `children` form writes an earlier one, at a superseded revision of its artifact and with no
//!   producer, which revision binding keeps from every evidence match, as freshness keeps an
//!   expired record, and every `children` form reads. A state can always hold it, so no protocol
//!   is refused for it. It is rendered `` evidence `case_outcome` result `r` expired ``, after the
//!   class's current records; a later record of the class sits beside it, as a second child's
//!   outcome does.
//! - **Authority step.** A capability some action requires goes from undecided to `granted` or to
//!   `denied`.
//! - **Decision step.** A decision some outcome requires goes from not taken to taken.
//!
//! No step terminates the case or adds a record at a superseded revision: a `children` form's
//! earlier records are those an expiry or a revision held as earlier.
//!
//! # Order
//!
//! There is one initial state per assignment of case-input values, in input order, the last input
//! varying fastest, with every mark at none. When kinds declare `required_at_open: true`, each
//! assignment has one initial state per combination of one record of each such kind, in kind-id
//! order, the last varying fastest, each kind's records in the order its supplied steps add them
//! (dimension, class, producer profile): the opener hands one in when it opens the case, of any
//! class. Their path is `empty`, so a record of a class that leads nowhere is a stuck state the
//! walk starts in, and `{}` is reached only when a record can leave (an expiry or a revision).
//! Without such a kind the initial states are as before. Breadth-first search reaches each state first along
//! a shortest path, a group step counting as one step. From a state, steps are tried in this
//! order: actions in identifier order, then their kinds in no group, dimensions, classes and
//! profiles in the space's own orders, then their groups in order of each group's least kind id,
//! each group's combinations with its members in kind-id order and the last member varying
//! fastest, each member's dimensions, classes and profiles in the space's orders, then its revise
//! steps, the subsets of its `revises` in lexicographic order of their sorted ids (`[a]`,
//! `[a, b]`, `[b]`), each with no record, then its records and group combinations in the order
//! above, a revise step counting as one step; supplied kinds the same way; the external revise
//! steps, by artifact in identifier order; the child-outcome
//! step, by opening action, dimension, class and profile, each plain successor before its
//! replacing ones, and each with the lower mark before the kept one; expiring kinds in dimension
//! order, then classes; capabilities in identifier order, `granted` before `denied`; decisions in
//! name order.
//!
//! # Findings
//!
//! A *record step* is an action's single-kind or group step (and the step of an action that adds
//! nothing); an action's steps are its record steps and its revise steps. A state is *open* when
//! some sequence of action (record and revise), supplied, child-outcome, expiry and authority
//! steps leads from it to a state in which an outcome is legitimate (R3); decision steps are not
//! part of that sequence (R4), since a protocol with an outcome that requires only a decision can
//! leave every state by it. A state that is not open, in which a denied capability blocks an
//! action whose precondition is true and a decision step leads to a state with a legitimate
//! outcome, ends by that decision (story:refused-approval-is-not-stuck, option B): it is none of
//! the findings below, and the summary counts it. A refused approval is ended by an explicit
//! decision, and a protocol that declares one for it needs no workaround action. In this order:
//!
//! - `stuck-state`: a state that is not open, in which no action is `admissible` or
//!   `approval-required`.
//! - `cycle`: a state that is not open, in which no action is `approval-required`, some action is
//!   `admissible` and every record step of every admissible action returns the same state. It
//!   names those actions (R6: with expiry steps the states the steps never leave can be a loop;
//!   the finding is the state of it in which every record step returns the same state). When a
//!   revise step of an admissible action changes the state, the finding says `` every record step
//!   of action `x` `` where it says `` every step of action `x` `` otherwise, and ends with
//!   `` ; every revision by action `y` leads to a state no outcome is reachable from ``, naming
//!   each such action: the state is not open, so neither is any state a step leads to.
//! - `dead-action`: an action `admissible` in no reachable state, in identifier order.
//! - `unreached-outcome`: an outcome legitimate in no reachable state, in identifier order; then
//!   `no-outcome-reachable`, once, when the protocol declares outcomes and every one is unreached.
//!
//! A state that meets the second half of the `stuck-state` or the `cycle` rule and ends by a
//! decision after a denial is neither.
//!
//! A state that meets the second half of the `stuck-state` or the `cycle` rule, in which no
//! outcome is legitimate, that is open and in which every step of every admissible action, record
//! or revise, returns it, waits: on a supplied record, a child's outcome or an expiry, the only
//! steps that can open it. It is no finding; the summary counts it when the protocol declares a
//! supplied or an expiring kind, `` W waiting on supplied evidence or expiry ``, and when it
//! declares an action that opens a child case, `` W waiting on supplied evidence, a child outcome
//! or expiry ``. An open state a revise step leads out of neither waits nor is a finding. A state
//! only an external revise step opens waits too, and a protocol with an artifact that declares
//! `revised_by` says so: `` W waiting on supplied evidence, expiry or a revision by another
//! protocol `` (`` supplied evidence, a child outcome, expiry or a revision by another protocol ``
//! with an action that opens a child case).
//!
//! One finding is printed per smallest state (R5): among the states of one kind, `stuck-state`
//! or `cycle`, one is printed when no other of that kind is contained in it. State A is contained
//! in state B when both give every case input the same value and every record of A (each class
//! and upstream vector, with the producer profiles it meets, and each record held as earlier),
//! every capability decision of A, every decision taken in A and every mark of A is in B, none
//! below one below two or more.
//!
//! Nothing that is not open goes unreported (story:simulate-revise-step, § Outcome 4, read with
//! the child-outcome and expiry steps). Record steps only add: a record, or a mark raised, and
//! authority steps only decide a capability. Take a reachable state that is not open and follow
//! record and authority steps from it, each changing the state, for as long as one does: each
//! successor holds more than the one before, so this ends, at a state m whose record steps all
//! return it and whose capabilities are all decided, since an authority step would change it.
//! Each state on the way is not open, as no step leads from a state that is not open to one that
//! is. In m no action is `approval-required`, every capability being decided, so m is a
//! `stuck-state` when no action is admissible and a `cycle` otherwise, unless it ends by a
//! decision after a denial, which the summary counts, and every state of a kind contains a
//! printed one. The argument uses no revise, child-outcome or expiry step, the three
//! that remove a record or lower a mark: they are what the old `cycle` rule, which asked every
//! step of every admissible action to return the state, could not cover, since a state a revise
//! step leaves never met it. A printed state is not open, so no child outcome that may arrive in
//! it and no revision opens it. A state that ends by a decision after a denial leaves the kind it
//! would be of, and a state of the kind that contains it is printed in its place unless it
//! contains another printed one. A printed finding names its state and its shortest path, then,
//! when a denied capability blocks an action whose precondition is true in its state (option A),
//! each such block and the two existing forms that end the case after the denial, `` capability
//! `c` denied blocks action `a`, whose precondition is true: to end the case after the denial,
//! declare an outcome with `requires: {decision: ..., when: ...}` or an action without capability
//! `c` whose record an outcome reads `` (the actions one set of denied capabilities blocks named
//! together, `` capability `c` and capability `d` denied block … ``, `` whose preconditions are
//! true `` and `` after the denials `` for more), then,
//! when a decision step leads from its state to a state with a legitimate outcome, each such
//! decision in name order and the outcomes it makes legitimate (R4), `` decision `d` taken makes
//! outcome `o` legitimate ``, then, when other states of its kind contain its state, how many,
//! `` 3 other stuck states contain it `` (`` 1 other cycle state contains it ``), and last the
//! revision suffix of a `cycle`.
//!
//! A state is written as `canon check` writes one, its records, capabilities, decisions and case
//! inputs, followed by each mark above none in action order, `` action `a` opened a child `` for
//! one and `` action `a` opened 2 or more children `` for two or more. A record observed before an
//! upstream artifact moved is written `` evidence `k` about `b` observed before upstream `a`
//! moved ``, after its class's records observed after every move. Printed
//! stuck states and cycles are ordered by the length of their path, then by their rendering. A
//! path is written as its steps, `` action `a` produces evidence `k` result `r` `` (`` action `a`
//! produces nothing `` for the step of an opening action that adds no record; a group step on one
//! line, its records in kind-id order joined by ` and `, `` action `a` produces evidence `k`
//! and evidence `l` result `r` ``), `` action `a` revises `b`, `c` `` (`` action `a` revises `b`
//! and produces evidence `k` `` with a record or a group), `` protocol `s`
//! supplies evidence `k` ``, `` protocol `q` revises `a` ``, `` a child of action `a` records evidence `case_outcome` result `r`
//! ``, `` evidence `k` expires ``, `` capability `c` denied ``, `` decision `d` taken ``, joined
//! by `, then `, or `empty`. The findings are followed by the summary: `` simulated: protocol `p`
//! revision 1: N reachable states, T terminable (outcome `a` x, outcome `b` y), W waiting on
//! supplied evidence or expiry, D ending by a decision after a denial, F findings (S stuck or
//! cycle states) ``: D counts the states that end by a decision after a denial, written only when
//! it is above 0, F counts printed
//! findings and S the states the printed stuck states and cycles stand for. A terminable state is
//! one in which some outcome is legitimate; the walk continues through it.
//!
//! # Refusals
//!
//! First, as `canon check` does, an IR nested beyond the evaluator's depth bound is refused as
//! `predicate-too-deep`, and claims that test each other in a cycle as `claim-cycle`. Then a
//! protocol with an order form (`first`, `before`, `after`, `since`, `descended`) is refused as
//! `ordered-evidence`, naming the first declaration that has one: the space puts a dimension's
//! current records at one instant, so a sequence of records of one dimension has no state there.
//! Then the walk is refused as `reachable-state-bound` when it would visit more than
//! [`check::STATE_BOUND`] states, states that differ only in marks counted apart. A walk that
//! stops early reports nothing: no partial walk is
//! offered, since a walk that has not seen every reachable state cannot say that none is stuck.
//! An evaluation the evaluator refuses is refused with its code.
//!
//! # Known limits
//!
//! Only an action's own `revises` moves an artifact, and only where the action is admissible: an
//! artifact another case revises, or one the protocol revises only through an action no state
//! admits, never moves, so a failed check on it is still a finding, and a kind read with
//! `result:` that an action may produce twice with different results gives a `cycle` unless an
//! admissible action revises the record's artifact. The case revision never moves, so a lasting
//! denial is reported, and a denial a runner binds to a case revision is walked as lasting. The
//! records of a dimension no match names an artifact for are about the first artifact no match
//! of the kind names, as the space takes them, so a revision of that artifact sets every such
//! record aside and a revision of another artifact sets none of them aside; the exception is
//! the `case_outcome` records of children that actions open about several artifacts, which the
//! walk keeps about the `case_open` subject of the action (`recovery`,
//! story:child-outcome-recovery), so a revision sets aside only the child outcomes about its
//! artifact. A child that has not ended when its subject is revised may end against the earlier
//! revision or the new one: from the third value of its action's mark its outcome arrives as an
//! earlier record as well as a current one.
//!
//! # Cost
//!
//! Each reachable state is evaluated once by Canon's evaluator, which dominates a walk with few
//! findings: 65536 states and no finding take 12.4 to 14.5 s in a release build (measured
//! 2026-11-21 on a shared workstation). Finding the smallest states (R5) compares states of one
//! kind only within a group of equal case inputs, each with those of a smaller size and then of
//! a greater one, stopping at the first that contains it: 65536 stuck states, each with case
//! inputs of its own, take 9.0 s, against 157.7 s when every state was compared with every
//! other; 16384 take 2.1 s, against 19.3 s. Within one group it stays quadratic: n stuck or cycle
//! states with equal inputs of which none contains another take up to n² comparisons, which for
//! 65536 such states is the cost of the search before the grouping.

use std::collections::{BTreeSet, HashMap, VecDeque};
use std::fmt;

use crate::check::{self, Reader, Space, State, Walked};
use crate::eval;
use crate::ir::Ir;
use crate::model::{
    ActionId, ArtifactId, CASE_OUTCOME, CapabilityId, DecisionName, EvidenceKindId, OPEN_OUTCOME,
    OutcomeId, Predicate, ProtocolId, one_line,
};

mod recovery;
#[cfg(test)]
mod tests;

/// Why a walk was refused: a stable machine-readable code and a one-line message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    code: &'static str,
    message: String,
}

impl Refusal {
    fn new(code: &'static str, message: String) -> Self {
        Self { code, message }
    }

    /// A stable machine-readable code for the kind of refusal.
    pub fn code(&self) -> &'static str {
        self.code
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Refusal {}

/// What a printed `stuck-state` or `cycle` names besides its kind: its state, its shortest path,
/// the denials that block an action whose precondition is true, the decisions that lead from it
/// to a legitimate outcome and how many other states of its kind contain it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dead {
    /// The state, as a report writes it.
    pub state: String,
    /// Its shortest path, one step each, as a report writes them.
    pub path: Vec<String>,
    /// Each action, in identifier order, that a denial blocks in the state while its
    /// precondition is true, with the denied capabilities in identifier order
    /// (story:refused-approval-is-not-stuck, option A).
    pub blocks: Vec<(ActionId, Vec<CapabilityId>)>,
    /// Each decision, in name order, whose step leads from the state to one in which an outcome
    /// is legitimate, with those outcomes in identifier order.
    pub decisions: Vec<(DecisionName, Vec<OutcomeId>)>,
    /// How many other reachable states of its kind contain the state.
    pub containing: usize,
}

/// One thing a walk found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Finding {
    StuckState(Dead),
    Cycle {
        dead: Dead,
        /// The admissible actions, each of whose record steps returns the state.
        actions: Vec<ActionId>,
        /// The admissible actions some revise step of which changes the state, each to a state
        /// no outcome is reachable from (story:simulate-revise-step); empty when every revise
        /// step returns it.
        revising: Vec<ActionId>,
    },
    DeadAction {
        action: ActionId,
    },
    UnreachedOutcome {
        outcome: OutcomeId,
    },
    NoOutcomeReachable {
        protocol: ProtocolId,
        revision: u64,
    },
}

impl Finding {
    /// A stable machine-readable code for the kind of finding.
    pub fn code(&self) -> &'static str {
        match self {
            Finding::StuckState(_) => "stuck-state",
            Finding::Cycle { .. } => "cycle",
            Finding::DeadAction { .. } => "dead-action",
            Finding::UnreachedOutcome { .. } => "unreached-outcome",
            Finding::NoOutcomeReachable { .. } => "no-outcome-reachable",
        }
    }
}

/// The clause of option A (story:refused-approval-is-not-stuck): each set of denied capabilities
/// with the actions it blocks, in order of their first action, and the two existing forms that
/// end the case after the denial, `` capability `c` denied blocks action `a`, whose precondition
/// is true: to end the case after the denial, declare an outcome with `requires: {decision: ...,
/// when: ...}` or an action without capability `c` whose record an outcome reads ``.
fn blocked(blocks: &[(ActionId, Vec<CapabilityId>)]) -> String {
    let capability = |id: &CapabilityId| format!("capability `{}`", one_line(id.as_str()));
    let action = |id: &ActionId| format!("action `{}`", one_line(id.as_str()));
    let mut groups: Vec<(&[CapabilityId], Vec<&ActionId>)> = Vec::new();
    for (id, capabilities) in blocks {
        match groups
            .iter_mut()
            .find(|(denied, _)| *denied == capabilities.as_slice())
        {
            Some((_, actions)) => actions.push(id),
            None => groups.push((capabilities, vec![id])),
        }
    }
    let named: Vec<String> = groups
        .iter()
        .map(|(denied, actions)| {
            format!(
                "{} denied {} {}",
                denied
                    .iter()
                    .map(capability)
                    .collect::<Vec<_>>()
                    .join(" and "),
                if denied.len() == 1 { "blocks" } else { "block" },
                actions
                    .iter()
                    .map(|id| action(id))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
        .collect();
    let denied: BTreeSet<&CapabilityId> = blocks.iter().flat_map(|(_, denied)| denied).collect();
    format!(
        "{}, {}: to end the case after the {}, declare an outcome with `requires: {{decision: ..., \
         when: ...}}` or an action without {} whose record an outcome reads",
        named.join(", "),
        if blocks.len() == 1 {
            "whose precondition is true"
        } else {
            "whose preconditions are true"
        },
        if denied.len() == 1 {
            "denial"
        } else {
            "denials"
        },
        denied
            .into_iter()
            .map(capability)
            .collect::<Vec<_>>()
            .join(" or ")
    )
}

/// `; path: …`, then the denials that block an action, the decisions and the count of containing
/// states, each after `; `.
fn tail(dead: &Dead, kind: &str) -> String {
    let mut text = if dead.path.is_empty() {
        "; path: empty".to_owned()
    } else {
        format!("; path: {}", dead.path.join(", then "))
    };
    if !dead.blocks.is_empty() {
        text.push_str(&format!("; {}", blocked(&dead.blocks)));
    }
    if !dead.decisions.is_empty() {
        let decisions: Vec<String> = dead
            .decisions
            .iter()
            .map(|(name, outcomes)| {
                let outcomes: Vec<String> = outcomes
                    .iter()
                    .map(|outcome| format!("outcome `{}`", one_line(outcome.as_str())))
                    .collect();
                format!(
                    "decision `{}` taken makes {} legitimate",
                    one_line(name.as_str()),
                    outcomes.join(", ")
                )
            })
            .collect();
        text.push_str(&format!("; {}", decisions.join(", ")));
    }
    match dead.containing {
        0 => {}
        1 => text.push_str(&format!("; 1 other {kind} state contains it")),
        n => text.push_str(&format!("; {n} other {kind} states contain it")),
    }
    text
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.code())?;
        match self {
            Finding::StuckState(dead) => write!(
                f,
                "state {}: no outcome is legitimate and no action is admissible or \
                 approval-required{}",
                dead.state,
                tail(dead, "stuck")
            ),
            Finding::Cycle {
                dead,
                actions,
                revising,
            } => {
                let named = |actions: &[ActionId]| -> String {
                    actions
                        .iter()
                        .map(|action| format!("action `{}`", one_line(action.as_str())))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                write!(
                    f,
                    "state {}: no outcome is legitimate, no action is approval-required, and \
                     every {}step of {} returns this state{}",
                    dead.state,
                    if revising.is_empty() { "" } else { "record " },
                    named(actions),
                    tail(dead, "cycle")
                )?;
                if !revising.is_empty() {
                    write!(
                        f,
                        "; every revision by {} leads to a state no outcome is reachable from",
                        named(revising)
                    )?;
                }
                Ok(())
            }
            Finding::DeadAction { action } => write!(
                f,
                "action `{}` is admissible in no reachable state",
                one_line(action.as_str())
            ),
            Finding::UnreachedOutcome { outcome } => write!(
                f,
                "outcome `{}` is legitimate in no reachable state",
                one_line(outcome.as_str())
            ),
            Finding::NoOutcomeReachable { protocol, revision } => write!(
                f,
                "protocol `{}` revision {revision} reaches no state in which an outcome is \
                 legitimate",
                one_line(protocol.as_str())
            ),
        }
    }
}

/// What a walk found over every reachable state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub protocol: ProtocolId,
    pub revision: u64,
    /// How many states the walk reached, initial states included.
    pub states: usize,
    /// How many of them are terminable: some outcome is legitimate there.
    pub terminable: usize,
    /// Each outcome, in identifier order, with how many reachable states it is legitimate in.
    pub outcomes: Vec<(OutcomeId, usize)>,
    /// How many reachable states wait on a supplied record, a child's outcome, an expiry or a
    /// revision by another protocol; `None` when the protocol declares no supplied kind, no
    /// expiring kind, no action that opens a child case and no artifact with `revised_by`, the
    /// only steps that can open such a state.
    pub waiting: Option<usize>,
    /// Whether some action opens a child case, so a child's outcome may open a waiting state and
    /// the summary names it among what the waiting states wait on.
    pub child_outcomes: bool,
    /// Whether some artifact declares `revised_by`, so a revision by another protocol may open a
    /// waiting state and the summary names it among what the waiting states wait on.
    pub revisions: bool,
    /// How many reachable states that are not open end by a decision after a denial: a denial
    /// blocks an action whose precondition is true there, and a decision step leads to a state
    /// with a legitimate outcome (story:refused-approval-is-not-stuck, option B). None of them is
    /// a finding.
    pub ending: usize,
    /// How many reachable states the printed stuck states and cycles stand for.
    pub dead: usize,
    pub findings: Vec<Finding>,
}

impl Report {
    /// Whether the walk found nothing.
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }
}

fn counted(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

/// One line per finding, then the summary: `` simulated: protocol `p` revision 1: 4 reachable
/// states, 2 terminable (outcome `a` 2), 0 findings (0 stuck or cycle states) ``, with `` , 1
/// waiting on supplied evidence or expiry `` before the findings when the protocol declares a
/// supplied or an expiring kind, and `` , 1 waiting on supplied evidence, a child outcome or
/// expiry `` when it declares an action that opens a child case; then `` , 1 ending by a decision
/// after a denial `` when some state ends that way.
impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for finding in &self.findings {
            writeln!(f, "{finding}")?;
        }
        let mut terminable = format!("{} terminable", self.terminable);
        if !self.outcomes.is_empty() {
            let outcomes: Vec<String> = self
                .outcomes
                .iter()
                .map(|(id, count)| format!("outcome `{}` {count}", one_line(id.as_str())))
                .collect();
            terminable.push_str(&format!(" ({})", outcomes.join(", ")));
        }
        if let Some(waiting) = self.waiting {
            let on = match (self.child_outcomes, self.revisions) {
                (false, false) => "supplied evidence or expiry",
                (true, false) => "supplied evidence, a child outcome or expiry",
                (false, true) => "supplied evidence, expiry or a revision by another protocol",
                (true, true) => {
                    "supplied evidence, a child outcome, expiry or a revision by another protocol"
                }
            };
            terminable.push_str(&format!(", {waiting} waiting on {on}"));
        }
        if self.ending > 0 {
            terminable.push_str(&format!(
                ", {} ending by a decision after a denial",
                self.ending
            ));
        }
        writeln!(
            f,
            "simulated: protocol `{}` revision {}: {}, {terminable}, {} ({})",
            one_line(self.protocol.as_str()),
            self.revision,
            counted(self.states, "reachable state", "reachable states"),
            counted(self.findings.len(), "finding", "findings"),
            counted(self.dead, "stuck or cycle state", "stuck or cycle states"),
        )
    }
}

/// One record a step adds, by position: its dimension, class and producer profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Added {
    dimension: usize,
    class: usize,
    profile: usize,
}

/// Which records of the class `open` a child-outcome step replaces: the current ones, the one held
/// as expired (R2a), or both.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Replacing {
    current: bool,
    expired: bool,
}

/// What a revise step adds after its revision (story:simulate-revise-step): no record, one
/// record of a kind in no group, or one of the action's group combinations, by position.
#[derive(Clone, Copy, Debug)]
enum Adds {
    Nothing,
    Record(Added),
    Group(usize),
}

/// One step of the walk, one variant per kind of step, by position: an action step (the action,
/// and the record it adds, `None` for the step of an action that adds none), an action's group
/// step (the action, and which of its group combinations it adds), an action's revise step (the
/// action, which subset of its `revises` it moves and what it adds), a supplied record, a
/// child's outcome (the action whose mark it lowers or keeps, the record, and the `open` records
/// it replaces, if any), a record class that expires, an authority decision or a decision taken.
#[derive(Clone, Copy, Debug)]
enum Step {
    Action {
        action: usize,
        record: Option<Added>,
    },
    /// A group step of `action`: its combination `at`, one record of each member kind of one of
    /// its groups (story:simulate-record-groups).
    Group {
        action: usize,
        at: usize,
    },
    /// A revise step of `action` (story:simulate-revise-step): the artifacts of its subset
    /// `subset` of `revises` move to a new revision, then `adds` is added at the current one.
    Revise {
        action: usize,
        subset: usize,
        adds: Adds,
    },
    Supplied(Added),
    /// The artifact `at` of those that declare `revised_by` moves to a new revision, a case of
    /// the protocol that revises it having revised it (story:artifact-revised-by).
    External {
        at: usize,
    },
    Child {
        action: usize,
        record: Added,
        replacing: Option<Replacing>,
    },
    /// A child's outcome that arrives as an earlier record, from the third value of the mark of
    /// `action` (story:child-outcome-recovery).
    Late {
        action: usize,
        record: Added,
        replacing: Option<Replacing>,
    },
    Expiry {
        dimension: usize,
        class: usize,
    },
    Capability {
        at: usize,
        granted: bool,
    },
    Decision {
        at: usize,
    },
}

/// Where a walk's records sit after the space's state: one slot per evidence dimension, class and
/// upstream vector, each the down-closed set of producer profiles its records meet, as bits in
/// whole words, and, in a dimension that holds expired records, one more bit after the profiles'
/// of the class's first slot for a record of the class held as expired (R2a) or set aside as
/// earlier by a revision (story:simulate-revise-step). A dimension has one vector per subset of
/// the upstream artifacts a revise step can move, the first observed after every move, so a
/// protocol that revises nothing has one slot per dimension and class. The marks follow the
/// records: two bits per action that opens a child case, in action order, in whole words, none
/// when no action opens one.
struct Layout<'d, 'a> {
    dimensions: &'d [Walked<'a>],
    /// The length of the space's state at the front of a walk's state.
    base: usize,
    /// For each dimension, class and upstream vector, the position of its slot's first word.
    slots: Vec<Vec<Vec<usize>>>,
    /// For each dimension, whether an expiry or a revision holds its records as earlier instead
    /// of removing them.
    holds: Vec<bool>,
    /// For each dimension, the positions in its `upstreams` of those a revise step can move: bit
    /// `i` of a vector's position is `upstreams[moving[i]]`.
    moving: Vec<Vec<usize>>,
    /// The position of the first word of the marks, after every slot.
    marks: usize,
    /// The position of each action that opens a child case, in action order: mark `i` is
    /// `openers[i]`'s.
    openers: Vec<usize>,
    /// For each opener, the `case_open` subject when its mark has a third value
    /// (story:child-outcome-recovery), otherwise `None`; empty before [`Layout::flagging`].
    flagged: Vec<Option<ArtifactId>>,
    /// For each opener, the `open` records a step of it adds with its child
    /// (story:simulate-opens-child-with-its-record); empty before [`Layout::opening`].
    opening: Vec<Vec<Added>>,
}

impl<'d, 'a> Layout<'d, 'a> {
    #[cfg(test)]
    fn new(dimensions: &'d [Walked<'a>], base: usize) -> Self {
        Self::holding(dimensions, base, &vec![false; dimensions.len()])
    }

    /// The layout with marks for `openers`, the positions of the actions that open a child case,
    /// in action order.
    fn marking(mut self, openers: Vec<usize>) -> Self {
        self.openers = openers;
        self
    }

    /// The layout in which each dimension `holds` marks keeps a record held as expired.
    #[cfg(test)]
    fn holding(dimensions: &'d [Walked<'a>], base: usize, holds: &[bool]) -> Self {
        Self::revising(dimensions, base, holds, vec![Vec::new(); dimensions.len()])
    }

    /// The layout in which each dimension `holds` marks keeps a record held as earlier, and each
    /// dimension has a vector per subset of its upstream artifacts at the positions `moving`
    /// gives it.
    fn revising(
        dimensions: &'d [Walked<'a>],
        base: usize,
        holds: &[bool],
        moving: Vec<Vec<usize>>,
    ) -> Self {
        let mut at = base;
        let mut slots = Vec::new();
        for ((walked, held), moving) in dimensions.iter().zip(holds).zip(&moving) {
            let width = (walked.profiles.len() + usize::from(*held))
                .div_ceil(32)
                .max(1);
            slots.push(
                (0..walked.classes.len())
                    .map(|_| {
                        (0..1usize << moving.len())
                            .map(|_| {
                                let start = at;
                                at += width;
                                start
                            })
                            .collect()
                    })
                    .collect(),
            );
        }
        Self {
            dimensions,
            base,
            slots,
            holds: holds.to_vec(),
            moving,
            marks: at,
            openers: Vec::new(),
            flagged: Vec::new(),
            opening: Vec::new(),
        }
    }

    /// The layout in which a step of opener `i` adds the records `opening[i]` as it raises its
    /// mark: the child's `open` record, as a runner writes it in the step that opens the child.
    fn opening(mut self, opening: Vec<Vec<Added>>) -> Self {
        self.opening = opening;
        self
    }

    /// The upstream vector of position `vector` of `dimension`, as [`Space::walk_evidence`] reads
    /// one: bit `i` for the dimension's `upstreams[i]`.
    fn vector(&self, dimension: usize, vector: usize) -> u32 {
        self.moving[dimension]
            .iter()
            .enumerate()
            .filter(|(bit, _)| vector & (1 << bit) != 0)
            .map(|(_, upstream)| 1u32 << upstream)
            .sum()
    }

    /// `state` after a revision of each of `artifacts` (story:simulate-revise-step, § Outcome 2):
    /// every current record about one of them leaves, held as earlier in a dimension a `children`
    /// form reads, and every current record of a dimension with one of them upstream is observed
    /// before it moved. Records held as earlier, decisions, capabilities and marks stay.
    fn revise(&self, state: &State, artifacts: &[&ArtifactId]) -> State {
        let mut walked = state.clone();
        for (dimension, kind) in self.dimensions.iter().enumerate() {
            let profiles = kind.profiles.len();
            let about = kind
                .subject
                .as_ref()
                .is_some_and(|subject| artifacts.contains(&subject));
            if about {
                for class in 0..kind.classes.len() {
                    let present = self.present(&walked, dimension, class);
                    for &start in &self.slots[dimension][class] {
                        for profile in 0..profiles {
                            walked[start + profile / 32] &= !(1 << (profile % 32));
                        }
                    }
                    if present && kind.history && self.holds[dimension] {
                        let start = self.slots[dimension][class][0];
                        walked[start + profiles / 32] |= 1 << (profiles % 32);
                    }
                }
                continue;
            }
            let moved: usize = self.moving[dimension]
                .iter()
                .enumerate()
                .filter(|(_, upstream)| artifacts.contains(&kind.upstreams[**upstream]))
                .map(|(bit, _)| 1 << bit)
                .sum();
            if moved == 0 {
                continue;
            }
            for slots in &self.slots[dimension] {
                for (vector, &from) in slots.iter().enumerate() {
                    let to = slots[vector | moved];
                    if to == from {
                        continue;
                    }
                    for profile in 0..profiles {
                        let bit = 1 << (profile % 32);
                        if walked[from + profile / 32] & bit != 0 {
                            walked[from + profile / 32] &= !bit;
                            walked[to + profile / 32] |= bit;
                        }
                    }
                }
            }
        }
        self.after_revision(walked, artifacts)
    }

    /// The whole length of a walk's state.
    fn length(&self) -> usize {
        self.marks + (2 * self.openers.len()).div_ceil(32) + self.flag_words()
    }

    /// The mark of opener `at` (`openers[at]`) in `state`: 0 for none, 1 for one child that has
    /// not ended, 2 for two or more. It is written in two bits, `01` for one and `11` for two or
    /// more, so a smaller mark has a subset of a greater one's bits and as many as it counts.
    fn mark(&self, state: &State, at: usize) -> u8 {
        let bits = (state[self.marks + 2 * at / 32] >> (2 * at % 32)) & 0b11;
        u8::try_from(bits.count_ones()).expect("two bits")
    }

    /// `state` with the mark of opener `at` at `mark` (0, 1 or 2).
    fn marked(&self, state: &State, at: usize, mark: u8) -> State {
        let mut marked = state.clone();
        let word = &mut marked[self.marks + 2 * at / 32];
        *word &= !(0b11 << (2 * at % 32));
        *word |= [0b00, 0b01, 0b11][usize::from(mark)] << (2 * at % 32);
        if mark == 0 {
            return self.settled(marked, at);
        }
        marked
    }

    /// `state` after a step of the action at `action`: when it opens a child case, its mark is
    /// raised by one child, up to two or more, and the child's `open` record is added
    /// ([`Layout::opening`]).
    fn raise(&self, state: &State, action: usize) -> State {
        match self.openers.iter().position(|opener| *opener == action) {
            Some(at) => {
                let raised = self.marked(state, at, (self.mark(state, at) + 1).min(2));
                self.opening.get(at).map_or(raised.clone(), |records| {
                    records.iter().fold(raised, |added, record| {
                        self.add(&added, record.dimension, record.class, record.profile)
                    })
                })
            }
            None => state.clone(),
        }
    }

    /// Each opener's position and action with its mark in `state`, when above none, in action
    /// order.
    fn raised(&self, state: &State) -> Vec<(usize, usize, u8)> {
        self.openers
            .iter()
            .enumerate()
            .map(|(at, action)| (at, *action, self.mark(state, at)))
            .filter(|(_, _, mark)| *mark > 0)
            .collect()
    }

    /// Whether `state` holds a record of `class` in `dimension` held as expired.
    fn held_expired(&self, state: &State, dimension: usize, class: usize) -> bool {
        let profiles = self.dimensions[dimension].profiles.len();
        self.holds[dimension]
            && state[self.slots[dimension][class][0] + profiles / 32] & (1 << (profiles % 32)) != 0
    }

    /// `state` without every current record of `class` in `dimension`, of every upstream vector.
    fn remove(&self, state: &State, dimension: usize, class: usize) -> State {
        let mut walked = state.clone();
        let profiles = self.dimensions[dimension].profiles.len();
        for &start in &self.slots[dimension][class] {
            for profile in 0..profiles {
                walked[start + profile / 32] &= !(1 << (profile % 32));
            }
        }
        walked
    }

    /// `state` without the records of `class` in `dimension` that `replacing` names: every
    /// current one, the one held as expired, or both.
    fn replace(
        &self,
        state: &State,
        dimension: usize,
        class: usize,
        replacing: Replacing,
    ) -> State {
        let mut walked = if replacing.current {
            self.remove(state, dimension, class)
        } else {
            state.clone()
        };
        let start = self.slots[dimension][class][0];
        let profiles = self.dimensions[dimension].profiles.len();
        if replacing.expired && self.holds[dimension] {
            walked[start + profiles / 32] &= !(1 << (profiles % 32));
        }
        walked
    }

    /// Whether profile `profile` is in the slot of `dimension`, `class` and upstream vector
    /// position `vector` of `state`.
    fn has(
        &self,
        state: &State,
        dimension: usize,
        class: usize,
        vector: usize,
        profile: usize,
    ) -> bool {
        state[self.slots[dimension][class][vector] + profile / 32] & (1 << (profile % 32)) != 0
    }

    /// `state` with one more record of `class` in `dimension`, from a producer of `profile`,
    /// observed after every move of an upstream artifact.
    fn add(&self, state: &State, dimension: usize, class: usize, profile: usize) -> State {
        let mut walked = state.clone();
        let within = &self.dimensions[dimension].within;
        for inner in 0..within.len() {
            if within[inner][profile] {
                walked[self.slots[dimension][class][0] + inner / 32] |= 1 << (inner % 32);
            }
        }
        walked
    }

    /// Whether `state` holds a current record of `class` in `dimension`, of any upstream vector.
    fn present(&self, state: &State, dimension: usize, class: usize) -> bool {
        (0..self.slots[dimension][class].len()).any(|vector| {
            (0..self.dimensions[dimension].profiles.len())
                .any(|profile| self.has(state, dimension, class, vector, profile))
        })
    }

    /// `state` without any current record of `class` in `dimension`; in a dimension that holds
    /// expired records, with one of the class held as expired.
    fn expire(&self, state: &State, dimension: usize, class: usize) -> State {
        let mut walked = self.remove(state, dimension, class);
        let start = self.slots[dimension][class][0];
        let profiles = self.dimensions[dimension].profiles.len();
        if self.holds[dimension] {
            walked[start + profiles / 32] |= 1 << (profiles % 32);
        }
        walked
    }

    /// Each dimension and class of `state` that holds a record held as expired, in that order.
    fn expired(&self, state: &State) -> Vec<(usize, usize)> {
        let mut expired = Vec::new();
        for (dimension, walked) in self.dimensions.iter().enumerate() {
            if !self.holds[dimension] {
                continue;
            }
            let profiles = walked.profiles.len();
            for (class, slots) in self.slots[dimension].iter().enumerate() {
                if state[slots[0] + profiles / 32] & (1 << (profiles % 32)) != 0 {
                    expired.push((dimension, class));
                }
            }
        }
        expired
    }

    /// The records of `state`, as the space writes them: each dimension, class and upstream
    /// vector that holds one, with the vector as [`Space::walk_evidence`] reads it and the
    /// profiles of its records, one per greatest member of its set, in profile order.
    fn held(&self, state: &State) -> Vec<(usize, usize, u32, Vec<usize>)> {
        let mut held = Vec::new();
        for (dimension, walked) in self.dimensions.iter().enumerate() {
            for class in 0..walked.classes.len() {
                for vector in 0..self.slots[dimension][class].len() {
                    let members: Vec<usize> = (0..walked.profiles.len())
                        .filter(|profile| self.has(state, dimension, class, vector, *profile))
                        .collect();
                    if members.is_empty() {
                        continue;
                    }
                    let greatest = members
                        .iter()
                        .copied()
                        .filter(|member| {
                            !members
                                .iter()
                                .any(|other| other != member && walked.within[*member][*other])
                        })
                        .collect();
                    held.push((dimension, class, self.vector(dimension, vector), greatest));
                }
            }
        }
        held
    }

    /// Whether every record of `smaller` is in `larger`, its producer profiles included, and
    /// every mark of `smaller` is at most `larger`'s, its bits a subset (R5): both sit after the
    /// space's state.
    fn records_within(&self, smaller: &State, larger: &State) -> bool {
        smaller[self.base..]
            .iter()
            .zip(&larger[self.base..])
            .all(|(a, b)| a & !b == 0)
    }

    /// How many things `state` holds: records' profiles, the children marks count (a mark's set
    /// bits), decided capabilities and taken decisions, and case inputs off their first value, which two comparable states share.
    /// A state contained in another holds fewer.
    fn size(&self, state: &State) -> u32 {
        state[..self.base]
            .iter()
            .map(|value| u32::from(*value != 0))
            .sum::<u32>()
            + state[self.base..]
                .iter()
                .map(|word| word.count_ones())
                .sum::<u32>()
    }
}

/// What the walk keeps of a reachable state.
struct Visited {
    /// How it was first reached: the state it was reached from, and the step.
    parent: Option<(usize, Step)>,
    /// The length of its shortest path.
    depth: usize,
    /// The outcomes legitimate in it, by position.
    legitimate: Vec<usize>,
    /// Whether it meets the second half of the `stuck-state` rule, or of the `cycle` rule with the
    /// admissible actions, or neither: the `cycle` rule asks only that every record step of every
    /// admissible action return it.
    idle: Option<Vec<usize>>,
    /// The admissible actions some revise step of which leads out of it, in action order.
    revisers: Vec<usize>,
    /// The states its action (record and revise), supplied, child-outcome, expiry and authority
    /// steps lead to.
    next: Vec<usize>,
    /// The states its decision steps lead to, with each decision's position.
    decided: Vec<(usize, usize)>,
    /// Each action a denial blocks in it while its precondition is true, by position, with the
    /// denied capabilities.
    denied: Vec<(usize, Vec<CapabilityId>)>,
}

/// Walks every state `ir` reaches from the empty case, as the module docs say. Pure and
/// deterministic.
pub fn simulate(ir: &Ir) -> Result<Report, Refusal> {
    walk(ir, true).map(|walked| walked.report)
}

/// What [`walk`] gives: the report, and for the tests every state it reached.
struct Walk {
    report: Report,
    /// Every reachable state, in the order the walk first reached it.
    #[cfg(test)]
    states: Vec<State>,
    /// The positions of the reachable states in which an `open` record and the marks of the
    /// actions that produce its dimension do not pair: an `open` record of the dimension, current
    /// or held as earlier, with every such mark at none, or, in a dimension the opening step adds
    /// the record in, some such mark above none with no `open` record.
    #[cfg(test)]
    unpaired: Vec<usize>,
}

/// [`simulate`]. With `with_record` false it is the walk before
/// story:simulate-opens-child-with-its-record, in which the step of an opening action adds no
/// `open` record and a child that ends may drop the `open` record a child that has not ended
/// still holds; the tests hold the walk's states within that walk's.
fn walk(ir: &Ir, with_record: bool) -> Result<Walk, Refusal> {
    eval::check_depth(ir).map_err(|refusal| Refusal::new(refusal.code(), refusal.to_string()))?;
    let protocol = format!(
        "protocol `{}` revision {}",
        one_line(ir.protocol.id.as_str()),
        ir.protocol.revision
    );
    if let Some(reader) = ordered(ir) {
        return Err(Refusal::new(
            "ordered-evidence",
            format!(
                "{protocol}: {reader} reads evidence with an order form (`first`, `before`, \
                 `after`, `since` or `descended`), which canon simulate does not walk"
            ),
        ));
    }
    let space = Space::new(ir);
    let bound = || {
        Refusal::new(
            "reachable-state-bound",
            format!(
                "{protocol} reaches more than the bound of {} states; a partial walk is not \
                 reported",
                check::STATE_BOUND
            ),
        )
    };
    if space
        .initial_count()
        .is_none_or(|count| count > check::STATE_BOUND)
    {
        return Err(bound());
    }
    let limit = usize::try_from(check::STATE_BOUND).expect("the bound fits usize");

    let (dimensions, split) = recovery::split(ir, &space, space.walked());
    let initial = space.initial_states();
    let base = initial.first().map_or(0, Vec::len);
    // R2a: an expired `case_outcome` record still counts for a `children` form. With no evidence
    // match reading the kind it has no expiry step; with one and a `children` form, an expiry
    // holds the record as expired.
    let (children_read, matched) = case_outcome_readers(ir);
    let is_case_outcome = |dimension: &Walked<'_>| dimension.kind.as_str() == CASE_OUTCOME;
    // story:simulate-revise-step: the artifacts some action declares in `revises`. A record about
    // one of them that a `children` form reads is held as earlier after a revision, and is
    // written `… earlier`; a dimension with one of them upstream has a vector per subset of them.
    // An artifact another protocol revises (story:artifact-revised-by) is revisable too.
    let external: Vec<(&ArtifactId, &ProtocolId)> = ir
        .artifacts
        .iter()
        .filter_map(|(id, artifact)| artifact.revised_by.as_ref().map(|by| (id, by)))
        .collect();
    let revisable: BTreeSet<&ArtifactId> = ir
        .actions
        .values()
        .flat_map(|action| &action.revises)
        .chain(external.iter().map(|(artifact, _)| *artifact))
        .collect();
    let revised_earlier: Vec<bool> = dimensions
        .iter()
        .map(|dimension| {
            dimension.history
                && dimension
                    .subject
                    .as_ref()
                    .is_some_and(|subject| revisable.contains(subject))
        })
        .collect();
    let holds: Vec<bool> = dimensions
        .iter()
        .zip(&revised_earlier)
        .map(|(dimension, earlier)| {
            (is_case_outcome(dimension) && children_read && matched) || *earlier
        })
        .collect();
    let moving: Vec<Vec<usize>> = dimensions
        .iter()
        .map(|dimension| {
            dimension
                .upstreams
                .iter()
                .enumerate()
                .filter(|(_, upstream)| revisable.contains(*upstream))
                .map(|(at, _)| at)
                .collect()
        })
        .collect();
    // One mark per action that opens a child case, in action order.
    let openers: Vec<usize> = ir
        .actions
        .values()
        .enumerate()
        .filter(|(_, action)| action.case_open.is_some())
        .map(|(at, _)| at)
        .collect();
    let child_outcomes = !openers.is_empty();
    // A child of an opening action records only in the `case_outcome` dimension its action
    // produces (`Space::produced_by`, story:child-outcome-subject D1): the one about its
    // `case_open` subject, or the one without a subject. Opener `i` is `openers[i]`.
    let produced: Vec<Vec<usize>> = openers
        .iter()
        .map(|action| {
            let action = ir
                .actions
                .values()
                .nth(*action)
                .expect("an opener is an action");
            space
                .produced_by(ir, action)
                .into_iter()
                .filter(|at| is_case_outcome(&dimensions[*at]))
                .collect()
        })
        .collect();
    let produced = split.redirect(&openers, produced);
    // For each dimension, the position of its class `open`, which a child that ends replaces.
    let open_class: Vec<Option<usize>> = dimensions
        .iter()
        .map(|walked| {
            walked
                .classes
                .iter()
                .position(|class| *class == Some(OPEN_OUTCOME))
        })
        .collect();
    // For each dimension, whether the step that opens a child adds its `open` record there: it has
    // the class `open` and some reader demands no independence, so the empty profile is its first
    // one and every producer's record meets that reader. When every reader demands independence a
    // runner's `open` record may meet no demand, which no reader reads and the state without it
    // stands for, so the opening adds none and a child-outcome step adds one that meets a demand.
    let paired: Vec<bool> = dimensions
        .iter()
        .zip(&open_class)
        .map(|(walked, open)| {
            open.is_some() && walked.profiles.first().is_some_and(String::is_empty)
        })
        .collect();
    // story:simulate-opens-child-with-its-record: the step that opens a child adds its `open`
    // record, from a producer of the empty profile, in each dimension its children record in that
    // is `paired` (D-b: a dimension without the class gets no record, and neither does one whose
    // every reader demands independence).
    let opening: Vec<Vec<Added>> = produced
        .iter()
        .map(|at| {
            at.iter()
                .filter(|_| with_record)
                .filter(|dimension| paired[**dimension])
                .filter_map(|dimension| {
                    open_class[*dimension].map(|class| Added {
                        dimension: *dimension,
                        class,
                        profile: 0,
                    })
                })
                .collect()
        })
        .collect();
    let layout = Layout::revising(&dimensions, base, &holds, moving)
        .marking(openers)
        .flagging(ir, &split)
        .opening(opening);
    // Each action's revise subsets: the non-empty subsets of its `revises`, in order of their
    // sorted ids; none for an action that declares no `revises`.
    let action_subsets: Vec<Vec<Vec<&ArtifactId>>> = ir
        .actions
        .values()
        .map(|action| {
            let sorted: BTreeSet<&ArtifactId> = action.revises.iter().collect();
            subsets(&sorted.into_iter().collect::<Vec<_>>())
        })
        .collect();
    let of_kinds = |kinds: &BTreeSet<&EvidenceKindId>| -> Vec<usize> {
        (0..split.originals)
            .filter(|at| kinds.contains(dimensions[*at].kind))
            .collect()
    };
    // Every record step a set of dimensions offers, in dimension, class and profile order.
    let records = |at: &[usize]| -> Vec<Added> {
        at.iter()
            .flat_map(|dimension| {
                let walked = &dimensions[*dimension];
                (0..walked.classes.len()).flat_map(move |class| {
                    (0..walked.profiles.len()).map(move |profile| Added {
                        dimension: *dimension,
                        class,
                        profile,
                    })
                })
            })
            .collect()
    };
    // An action step adds a record of its `may_produce` only: a child's `case_outcome` arrives
    // by the child-outcome step, which `check::produces` does not know. A kind in one of the
    // action's groups (`records_together`) is never added alone.
    let action_records: Vec<Vec<Added>> = ir
        .actions
        .values()
        .map(|action| {
            let grouped: BTreeSet<&EvidenceKindId> =
                action.records_together.iter().flatten().collect();
            records(&of_kinds(
                &action
                    .may_produce
                    .iter()
                    .filter(|kind| !grouped.contains(kind))
                    .collect(),
            ))
        })
        .collect();
    // Each action's group steps, in group order (by least kind): for each group, every
    // combination of one record of each member kind, the members in kind-id order and the last
    // varying fastest. A member no record can be about adds nothing and takes no part.
    let action_groups: Vec<Vec<Vec<Added>>> = ir
        .actions
        .values()
        .map(|action| {
            action
                .records_together
                .iter()
                .flat_map(|group| {
                    let members: Vec<Vec<Added>> = group
                        .iter()
                        .map(|kind| records(&of_kinds(&BTreeSet::from([kind]))))
                        .filter(|member| !member.is_empty())
                        .collect();
                    if members.is_empty() {
                        return Vec::new();
                    }
                    let mut combinations: Vec<Vec<Added>> = vec![Vec::new()];
                    for member in &members {
                        combinations = combinations
                            .into_iter()
                            .flat_map(|prefix| {
                                member.iter().map(move |record| {
                                    let mut combination = prefix.clone();
                                    combination.push(*record);
                                    combination
                                })
                            })
                            .collect();
                    }
                    combinations
                })
                .collect()
        })
        .collect();
    let supplied_kinds: BTreeSet<&EvidenceKindId> = check::supplied(ir).collect();
    let supplied_records = records(&of_kinds(&supplied_kinds));
    let child_records: Vec<Vec<Added>> = produced.iter().map(|at| records(at)).collect();
    // Whether `state` holds a record of the class `open` in `dimension`, current or held as
    // expired or earlier.
    let holds_open = |state: &State, dimension: usize| {
        open_class[dimension].is_some_and(|open| {
            layout.present(state, dimension, open) || layout.held_expired(state, dimension, open)
        })
    };
    // Whether `state` holds a mark above none on an action whose children record in `dimension`.
    let marked_on = |state: &State, dimension: usize| {
        produced
            .iter()
            .enumerate()
            .any(|(opener, at)| at.contains(&dimension) && layout.mark(state, opener) > 0)
    };
    // Whether `walked`, a child-outcome successor of `state`, pairs its `open` records with the
    // marks. Every `case_outcome` record of the class `open`, current or held as expired, has a
    // child that has not ended (D5): a mark above none on an action that produces the record's
    // dimension. And the reverse (story:simulate-opens-child-with-its-record), in a `paired`
    // dimension: a child that ends replaces its own `open` record, so `walked` drops no `open`
    // record of a dimension `state` held while a mark above none on an action that produces the
    // dimension is left. In a dimension whose every reader demands independence the child left
    // may be one whose `open` record meets no demand.
    let open_held = |state: &State, walked: &State| {
        (0..dimensions.len()).all(|dimension| {
            !is_case_outcome(&dimensions[dimension])
                || ((!holds_open(walked, dimension) || marked_on(walked, dimension))
                    && (!with_record
                        || !paired[dimension]
                        || holds_open(walked, dimension)
                        || !holds_open(state, dimension)
                        || !marked_on(walked, dimension)))
        })
    };
    let expiring_kinds: BTreeSet<&EvidenceKindId> = ir
        .evidence_kinds
        .iter()
        .filter(|(_, kind)| kind.max_age.is_some())
        .map(|(id, _)| id)
        .collect();
    let expiring: Vec<usize> = (0..dimensions.len())
        .filter(|at| expiring_kinds.contains(dimensions[*at].kind))
        .filter(|at| matched || !is_case_outcome(&dimensions[*at]))
        .collect();
    let capabilities = space.walked_capabilities().len();
    let decisions = space.walked_decisions().len();

    // story:supplied-evidence-required-at-open: each kind required at open, in identifier order,
    // with the records a walk may start with of it, those its supplied step adds; a kind no record
    // can be about adds none. Each assignment of case inputs starts with one record of each such
    // kind, the last kind varying fastest.
    let at_open: Vec<Vec<Added>> = space
        .at_open()
        .iter()
        .map(|kind| records(&of_kinds(&BTreeSet::from([*kind]))))
        .filter(|records| !records.is_empty())
        .collect();
    let mut states: Vec<State> = Vec::new();
    for mut state in initial {
        state.resize(layout.length(), 0);
        let mut starts = vec![state];
        for records in &at_open {
            starts = starts
                .iter()
                .flat_map(|start| {
                    records.iter().map(|record| {
                        layout.add(start, record.dimension, record.class, record.profile)
                    })
                })
                .collect();
        }
        states.extend(starts);
    }
    let mut index: HashMap<State, usize> = states
        .iter()
        .enumerate()
        .map(|(at, state)| (state.clone(), at))
        .collect();
    let mut visited: Vec<Visited> = Vec::with_capacity(states.len());
    let mut parents: Vec<(Option<(usize, Step)>, usize)> =
        states.iter().map(|_| (None, 0)).collect();
    let mut admissible_somewhere = vec![false; ir.actions.len()];

    let mut next = 0;
    while next < states.len() {
        let state = states[next].clone();
        let held = layout.held(&state);
        let evaluated = check::evaluate_records(
            ir,
            &space,
            &state[..base].to_vec(),
            split.evidence(&space, &dimensions, &held, &layout.expired(&state)),
        )
        .map_err(|refusal| Refusal::new(refusal.code(), refusal.to_string()))?;
        let legitimate: Vec<usize> = (0..ir.outcomes.len())
            .filter(|at| evaluated.outcomes[*at] == "legitimate")
            .collect();
        let approval = evaluated
            .actions
            .iter()
            .any(|status| status == "approval-required");
        let admissible: Vec<usize> = (0..ir.actions.len())
            .filter(|at| evaluated.actions[*at] == "admissible")
            .collect();

        let mut successors: Vec<(State, Step)> = Vec::new();
        let mut returns = true;
        let mut revisers: Vec<usize> = Vec::new();
        for action in &admissible {
            admissible_somewhere[*action] = true;
            // Each step raises the action's mark when it opens a child case: its single-kind steps,
            // then its group steps, each adding its records at once.
            let mut steps: Vec<(State, Step)> = action_records[*action]
                .iter()
                .map(|record| {
                    let added = layout.add(&state, record.dimension, record.class, record.profile);
                    (
                        layout.raise(&added, *action),
                        Step::Action {
                            action: *action,
                            record: Some(*record),
                        },
                    )
                })
                .collect();
            for (at, combination) in action_groups[*action].iter().enumerate() {
                let added = combination.iter().fold(state.clone(), |added, record| {
                    layout.add(&added, record.dimension, record.class, record.profile)
                });
                steps.push((
                    layout.raise(&added, *action),
                    Step::Group {
                        action: *action,
                        at,
                    },
                ));
            }
            if steps.is_empty() {
                steps.push((
                    layout.raise(&state, *action),
                    Step::Action {
                        action: *action,
                        record: None,
                    },
                ));
            }
            for (walked, step) in steps {
                if walked != state {
                    returns = false;
                    successors.push((walked, step));
                }
            }
            // Its revise steps, after its record steps: each subset of its `revises`, then no
            // record, each single-kind record and each group combination, in that order. They
            // do not count for `returns`: the `cycle` rule reads record steps only.
            let mut revising = false;
            for (subset, artifacts) in action_subsets[*action].iter().enumerate() {
                let revised = layout.revise(&state, artifacts);
                let adds = std::iter::once(Adds::Nothing)
                    .chain(action_records[*action].iter().copied().map(Adds::Record))
                    .chain((0..action_groups[*action].len()).map(Adds::Group));
                for adds in adds {
                    let added = match adds {
                        Adds::Nothing => revised.clone(),
                        Adds::Record(record) => {
                            layout.add(&revised, record.dimension, record.class, record.profile)
                        }
                        Adds::Group(at) => action_groups[*action][at].iter().fold(
                            revised.clone(),
                            |added, record| {
                                layout.add(&added, record.dimension, record.class, record.profile)
                            },
                        ),
                    };
                    let walked = layout.raise(&added, *action);
                    if walked != state {
                        revising = true;
                        successors.push((
                            walked,
                            Step::Revise {
                                action: *action,
                                subset,
                                adds,
                            },
                        ));
                    }
                }
            }
            if revising {
                revisers.push(*action);
            }
        }
        for record in &supplied_records {
            let walked = layout.add(&state, record.dimension, record.class, record.profile);
            if walked != state {
                successors.push((walked, Step::Supplied(*record)));
            }
        }
        // After the supplied steps, before the expiry steps: the artifacts another protocol
        // revises, in identifier order. No action step: it counts neither for `returns` nor for
        // `revisers`.
        for (at, (artifact, _)) in external.iter().enumerate() {
            let walked = layout.revise(&state, &[*artifact]);
            if walked != state {
                successors.push((walked, Step::External { at }));
            }
        }
        for (at, action, mark) in layout.raised(&state) {
            for record in &child_records[at] {
                let Added {
                    dimension,
                    class,
                    profile,
                } = *record;
                // A child that ends replaces its `open` record: the current ones, the one held
                // as expired, or both. It lowers the mark by one child, and from two or more
                // may leave two or more; a child that records `open` has not ended.
                let open = open_class[dimension].filter(|open| *open != class);
                let lowered: &[u8] = match (open_class[dimension] == Some(class), mark) {
                    (true, _) => &[mark],
                    (false, 1) => &[0],
                    (false, _) => &[1, 2],
                };
                let mut replacings = vec![None];
                if let Some(open) = open {
                    let current = layout.present(&state, dimension, open);
                    let expired = layout.held_expired(&state, dimension, open);
                    replacings.extend(
                        [(true, false), (false, true), (true, true)]
                            .into_iter()
                            .filter(|(c, e)| (!c || current) && (!e || expired))
                            .map(|(current, expired)| Some(Replacing { current, expired })),
                    );
                }
                for replacing in replacings {
                    let replaced = match (open, replacing) {
                        (Some(open), Some(replacing)) => {
                            layout.replace(&state, dimension, open, replacing)
                        }
                        _ => state.clone(),
                    };
                    let added = layout.add(&replaced, dimension, class, profile);
                    for lowered in lowered {
                        let walked = layout.marked(&added, at, *lowered);
                        // D5: a state that holds an `open` record holds a child that has not
                        // ended, so it keeps a mark above none on an action that produces it;
                        // and a child that ends leaves the `open` record of one not ended.
                        if walked == state || !open_held(&state, &walked) {
                            continue;
                        }
                        successors.push((
                            walked,
                            Step::Child {
                                action,
                                record: *record,
                                replacing,
                            },
                        ));
                    }
                }
            }
        }
        successors.extend(recovery::late(
            &layout,
            &state,
            &child_records,
            &open_class,
            &open_held,
        ));
        for dimension in &expiring {
            for class in 0..dimensions[*dimension].classes.len() {
                if layout.present(&state, *dimension, class) {
                    successors.push((
                        layout.expire(&state, *dimension, class),
                        Step::Expiry {
                            dimension: *dimension,
                            class,
                        },
                    ));
                }
            }
        }
        for at in 0..capabilities {
            for granted in [true, false] {
                if let Some(walked) = space.with_capability(&state[..base].to_vec(), at, granted) {
                    let mut walked = walked;
                    walked.extend_from_slice(&state[base..]);
                    successors.push((walked, Step::Capability { at, granted }));
                }
            }
        }
        for at in 0..decisions {
            if let Some(walked) = space.with_decision(&state[..base].to_vec(), at) {
                let mut walked = walked;
                walked.extend_from_slice(&state[base..]);
                successors.push((walked, Step::Decision { at }));
            }
        }

        let idle = (legitimate.is_empty() && !approval && (admissible.is_empty() || returns))
            .then(|| admissible.clone());
        let mut followed = Vec::new();
        let mut decided = Vec::new();
        let depth = parents[next].1 + 1;
        for (walked, step) in successors {
            let at = match index.get(&walked) {
                Some(at) => *at,
                None => {
                    if states.len() >= limit {
                        return Err(bound());
                    }
                    let at = states.len();
                    index.insert(walked.clone(), at);
                    states.push(walked);
                    parents.push((Some((next, step)), depth));
                    at
                }
            };
            match step {
                Step::Decision { at: decision } => decided.push((decision, at)),
                _ => followed.push(at),
            }
        }
        visited.push(Visited {
            parent: parents[next].0,
            depth: parents[next].1,
            legitimate,
            idle,
            revisers,
            next: followed,
            decided,
            denied: evaluated.denied,
        });
        next += 1;
    }

    // Open states: those from which action, supplied, external revise, child-outcome, expiry and
    // authority steps reach a state with a legitimate outcome, found backwards from those states.
    let mut previous: Vec<Vec<usize>> = vec![Vec::new(); states.len()];
    for (at, state) in visited.iter().enumerate() {
        for to in &state.next {
            previous[*to].push(at);
        }
    }
    let mut open = vec![false; states.len()];
    let mut pending: VecDeque<usize> = VecDeque::new();
    for (at, state) in visited.iter().enumerate() {
        if !state.legitimate.is_empty() {
            open[at] = true;
            pending.push_back(at);
        }
    }
    while let Some(at) = pending.pop_front() {
        for from in &previous[at] {
            if !open[*from] {
                open[*from] = true;
                pending.push_back(*from);
            }
        }
    }

    let mut waiting = 0;
    let mut ending = 0;
    let mut stuck: Vec<usize> = Vec::new();
    let mut cycles: Vec<usize> = Vec::new();
    for (at, state) in visited.iter().enumerate() {
        let Some(admissible) = &state.idle else {
            continue;
        };
        // An open state waits only when every step of every admissible action, record or
        // revise, returns it; one a revise step leads out of is neither waiting nor a finding.
        // One that is not open, in which a denial blocks an action whose precondition is true
        // and a decision step leads to a legitimate outcome, ends by that decision
        // (story:refused-approval-is-not-stuck, option B, which narrows R4).
        if open[at] {
            if state.revisers.is_empty() {
                waiting += 1;
            }
        } else if !state.denied.is_empty()
            && state
                .decided
                .iter()
                .any(|(_, to)| !visited[*to].legitimate.is_empty())
        {
            ending += 1;
        } else if admissible.is_empty() {
            stuck.push(at);
        } else {
            cycles.push(at);
        }
    }

    let action_id = |action: usize| {
        one_line(
            ir.actions
                .keys()
                .nth(action)
                .expect("a declared action")
                .as_str(),
        )
    };
    let render_step = |step: &Step| -> String {
        let record = |dimension: usize, class: usize| {
            let walked = &dimensions[dimension];
            let mut record = format!("evidence {}", walked.name);
            if let Some(result) = walked.classes[class] {
                record.push_str(&format!(" result `{}`", one_line(result)));
            }
            record
        };
        let added = |added: &Added| {
            format!(
                "{}{}",
                record(added.dimension, added.class),
                dimensions[added.dimension].profiles[added.profile]
            )
        };
        match step {
            Step::Action {
                action,
                record: Some(record),
            } => format!("action `{}` produces {}", action_id(*action), added(record)),
            Step::Action {
                action,
                record: None,
            } => format!("action `{}` produces nothing", action_id(*action)),
            Step::Group { action, at } => {
                let records: Vec<String> = action_groups[*action][*at].iter().map(added).collect();
                format!(
                    "action `{}` produces {}",
                    action_id(*action),
                    records.join(" and ")
                )
            }
            Step::Revise {
                action,
                subset,
                adds,
            } => {
                let artifacts: Vec<String> = action_subsets[*action][*subset]
                    .iter()
                    .map(|artifact| format!("`{}`", one_line(artifact.as_str())))
                    .collect();
                let mut text = format!(
                    "action `{}` revises {}",
                    action_id(*action),
                    artifacts.join(", ")
                );
                match adds {
                    Adds::Nothing => {}
                    Adds::Record(record) => {
                        text.push_str(&format!(" and produces {}", added(record)));
                    }
                    Adds::Group(at) => {
                        let records: Vec<String> =
                            action_groups[*action][*at].iter().map(added).collect();
                        text.push_str(&format!(" and produces {}", records.join(" and ")));
                    }
                }
                text
            }
            Step::Supplied(record) => {
                let supplier = ir
                    .evidence_kinds
                    .get(dimensions[record.dimension].kind)
                    .and_then(|kind| kind.supplied_by.as_ref())
                    .expect("a supplied kind");
                format!(
                    "protocol `{}` supplies {}",
                    one_line(supplier.as_str()),
                    added(record)
                )
            }
            Step::External { at } => format!(
                "protocol `{}` revises `{}`",
                one_line(external[*at].1.as_str()),
                one_line(external[*at].0.as_str())
            ),
            Step::Child {
                action,
                record,
                replacing,
            }
            | Step::Late {
                action,
                record,
                replacing,
            } => {
                let mut text = format!(
                    "a child of action `{}` records {}{}",
                    action_id(*action),
                    added(record),
                    if matches!(step, Step::Late { .. }) {
                        " earlier"
                    } else {
                        ""
                    }
                );
                let open = format!("result `{OPEN_OUTCOME}`");
                match replacing {
                    None => {}
                    Some(Replacing {
                        current: true,
                        expired: false,
                    }) => text.push_str(&format!(", replacing {open}")),
                    Some(Replacing {
                        current: false,
                        expired: true,
                    }) => text.push_str(&format!(", replacing {open} expired")),
                    Some(Replacing { .. }) => {
                        text.push_str(&format!(", replacing {open} and {open} expired"));
                    }
                }
                text
            }
            Step::Expiry { dimension, class } => {
                format!("{} expires", record(*dimension, *class))
            }
            Step::Capability { at, granted } => format!(
                "capability `{}` {}",
                one_line(space.walked_capabilities()[*at].as_str()),
                if *granted { "granted" } else { "denied" }
            ),
            Step::Decision { at } => format!(
                "decision `{}` taken",
                one_line(space.walked_decisions()[*at].as_str())
            ),
        }
    };
    let path = |mut at: usize| -> Vec<String> {
        let mut steps = Vec::new();
        while let Some((parent, step)) = &visited[at].parent {
            steps.push(render_step(step));
            at = *parent;
        }
        steps.reverse();
        steps
    };
    // A state as the space writes it, then each mark above none in action order.
    let rendered = |at: usize| {
        let mut text = split.render(
            &space,
            &dimensions,
            &states[at][..base].to_vec(),
            &layout.held(&states[at]),
            &layout.expired(&states[at]),
            &revised_earlier,
        );
        for (_, action, mark) in layout.raised(&states[at]) {
            text.pop();
            if text.len() > 1 {
                text.push_str(", ");
            }
            let children = if mark == 1 {
                "a child"
            } else {
                "2 or more children"
            };
            text.push_str(&format!(
                "action `{}` opened {children}}}",
                action_id(action)
            ));
        }
        text
    };
    let contains = |smaller: usize, larger: usize| {
        space.decided_within(&states[smaller], &states[larger])
            && layout.records_within(&states[smaller], &states[larger])
    };
    // The smallest states of one kind, each with how many other states of the kind contain it.
    // A state is contained only in one with the same case inputs and a greater size (a state
    // contained in another of its size is that state), so the states are compared only within a
    // group of equal inputs, and each only with those of a smaller size, then a greater one.
    let sizes: Vec<u32> = states.iter().map(|state| layout.size(state)).collect();
    let smallest = |of: &[usize]| -> Vec<(usize, usize)> {
        let mut groups: HashMap<&[u32], Vec<usize>> = HashMap::new();
        for at in of {
            groups
                .entry(space.walk_inputs(&states[*at]))
                .or_default()
                .push(*at);
        }
        let mut printed = Vec::new();
        for mut group in groups.into_values() {
            group.sort_by_key(|at| (sizes[*at], *at));
            let mut found: Vec<usize> = Vec::new();
            for at in &group {
                if !found
                    .iter()
                    .any(|small| sizes[*small] < sizes[*at] && contains(*small, *at))
                {
                    found.push(*at);
                }
            }
            for small in found {
                let larger = group.partition_point(|other| sizes[*other] <= sizes[small]);
                let containing = group[larger..]
                    .iter()
                    .filter(|other| contains(small, **other))
                    .count();
                printed.push((small, containing));
            }
        }
        printed.sort_unstable();
        printed
    };
    let decision_exits = |at: usize| -> Vec<(DecisionName, Vec<OutcomeId>)> {
        let mut exits: Vec<(usize, usize)> = visited[at]
            .decided
            .iter()
            .copied()
            .filter(|(_, to)| !visited[*to].legitimate.is_empty())
            .collect();
        exits.sort();
        exits
            .into_iter()
            .map(|(decision, to)| {
                (
                    space.walked_decisions()[decision].clone(),
                    visited[to]
                        .legitimate
                        .iter()
                        .map(|outcome| {
                            ir.outcomes
                                .keys()
                                .nth(*outcome)
                                .expect("a declared outcome")
                                .clone()
                        })
                        .collect(),
                )
            })
            .collect()
    };
    let dead = |at: usize, containing: usize| Dead {
        state: rendered(at),
        path: path(at),
        blocks: visited[at]
            .denied
            .iter()
            .map(|(action, capabilities)| {
                (
                    ir.actions
                        .keys()
                        .nth(*action)
                        .expect("a declared action")
                        .clone(),
                    capabilities.clone(),
                )
            })
            .collect(),
        decisions: decision_exits(at),
        containing,
    };
    let ordered_findings = |of: &[usize]| -> Vec<(usize, usize)> {
        let mut printed = smallest(of);
        printed.sort_by_cached_key(|(at, _)| (visited[*at].depth, rendered(*at)));
        printed
    };

    let mut findings = Vec::new();
    for (at, containing) in ordered_findings(&stuck) {
        findings.push(Finding::StuckState(dead(at, containing)));
    }
    let action_ids = |actions: &[usize]| -> Vec<ActionId> {
        actions
            .iter()
            .map(|action| {
                ir.actions
                    .keys()
                    .nth(*action)
                    .expect("a declared action")
                    .clone()
            })
            .collect()
    };
    for (at, containing) in ordered_findings(&cycles) {
        findings.push(Finding::Cycle {
            dead: dead(at, containing),
            actions: action_ids(visited[at].idle.as_ref().expect("a cycle state is idle")),
            revising: action_ids(&visited[at].revisers),
        });
    }
    for (id, reached) in ir.actions.keys().zip(&admissible_somewhere) {
        if !reached {
            findings.push(Finding::DeadAction { action: id.clone() });
        }
    }
    let mut legitimate_in = vec![0usize; ir.outcomes.len()];
    for state in &visited {
        for outcome in &state.legitimate {
            legitimate_in[*outcome] += 1;
        }
    }
    for (id, count) in ir.outcomes.keys().zip(&legitimate_in) {
        if *count == 0 {
            findings.push(Finding::UnreachedOutcome {
                outcome: id.clone(),
            });
        }
    }
    if !ir.outcomes.is_empty() && legitimate_in.iter().all(|count| *count == 0) {
        findings.push(Finding::NoOutcomeReachable {
            protocol: ir.protocol.id.clone(),
            revision: ir.protocol.revision,
        });
    }
    #[cfg(test)]
    let unpaired: Vec<usize> = (0..states.len())
        .filter(|at| {
            (0..dimensions.len()).any(|dimension| {
                let (open, marked) = (
                    holds_open(&states[*at], dimension),
                    marked_on(&states[*at], dimension),
                );
                is_case_outcome(&dimensions[dimension])
                    && open_class[dimension].is_some()
                    && ((open && !marked) || (marked && !open && paired[dimension]))
            })
        })
        .collect();
    let report = Report {
        protocol: ir.protocol.id.clone(),
        revision: ir.protocol.revision,
        states: states.len(),
        terminable: visited
            .iter()
            .filter(|state| !state.legitimate.is_empty())
            .count(),
        outcomes: ir
            .outcomes
            .keys()
            .cloned()
            .zip(legitimate_in.iter().copied())
            .collect(),
        waiting: (!supplied_kinds.is_empty()
            || !expiring_kinds.is_empty()
            || child_outcomes
            || !external.is_empty())
        .then_some(waiting),
        child_outcomes,
        revisions: !external.is_empty(),
        ending,
        dead: stuck.len() + cycles.len(),
        findings,
    };
    Ok(Walk {
        report,
        #[cfg(test)]
        states,
        #[cfg(test)]
        unpaired,
    })
}

/// The non-empty subsets of `items`, each in the order of `items`, in lexicographic order of
/// those sequences: for `[a, b, c]`, `[a]`, `[a, b]`, `[a, b, c]`, `[a, c]`, `[b]`, `[b, c]`,
/// `[c]` (story:simulate-revise-step, "in order of their sorted ids").
fn subsets<T: Copy>(items: &[T]) -> Vec<Vec<T>> {
    fn extend<T: Copy>(items: &[T], from: usize, prefix: &mut Vec<T>, found: &mut Vec<Vec<T>>) {
        for at in from..items.len() {
            prefix.push(items[at]);
            found.push(prefix.clone());
            extend(items, at + 1, prefix, found);
            prefix.pop();
        }
    }
    let mut found = Vec::new();
    extend(items, 0, &mut Vec::new(), &mut found);
    found
}

/// Whether a `children` form reads `case_outcome`, and whether an evidence match does, in any
/// claim, obligation, precondition or outcome requirement of `ir`.
fn case_outcome_readers(ir: &Ir) -> (bool, bool) {
    let predicates = ir
        .claims
        .values()
        .map(|claim| &claim.true_when)
        .chain(
            ir.obligations
                .values()
                .map(|obligation| &obligation.discharged_when),
        )
        .chain(ir.actions.values().map(|action| &action.precondition))
        .chain(
            ir.outcomes
                .values()
                .filter_map(|outcome| outcome.requires.predicate()),
        );
    let (mut children, mut matched) = (false, false);
    for predicate in predicates {
        predicate.visit(&mut |node| match node {
            Predicate::Children(_) => children = true,
            Predicate::Evidence(matching) if matching.kind.as_str() == CASE_OUTCOME => {
                matched = true;
            }
            _ => {}
        });
    }
    (children, matched)
}

/// The first declaration whose predicate holds an order form: each claim's `true_when`, each
/// obligation's `discharged_when`, each action's precondition and each outcome's requirement, in
/// that order, each section in identifier order.
fn ordered(ir: &Ir) -> Option<Reader> {
    let claims = ir
        .claims
        .iter()
        .map(|(id, claim)| (Reader::Claim(id.clone()), &claim.true_when));
    let obligations = ir
        .obligations
        .iter()
        .map(|(id, obligation)| (Reader::Obligation(id.clone()), &obligation.discharged_when));
    let actions = ir
        .actions
        .iter()
        .map(|(id, action)| (Reader::Action(id.clone()), &action.precondition));
    let outcomes = ir.outcomes.iter().filter_map(|(id, outcome)| {
        Some((Reader::Outcome(id.clone()), outcome.requires.predicate()?))
    });
    claims
        .chain(obligations)
        .chain(actions)
        .chain(outcomes)
        .find(|(_, predicate)| !predicate.order_forms().is_empty())
        .map(|(reader, _)| reader)
}
