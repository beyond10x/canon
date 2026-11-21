//! A breadth-first walk over the states a case of a compiled protocol can reach from the empty
//! case (story:canon-simulate, with the rules refined after adversary pass 1, R1 to R6).
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
//! observed after every move of an upstream artifact; for each dimension and class the walk holds
//! the down-closed set of profiles its records meet, as the space's values do. It is evaluated as
//! `canon check` evaluates a state, as a case snapshot with every artifact at one revision, no
//! termination and no evaluation instant. The walk never needs a dimension's table of values, so
//! the number of values of a dimension is never a reason to refuse (R1): its bound counts the
//! states it reaches.
//!
//! # Steps
//!
//! - **Action step.** Taken only from a state in which the action is `admissible`. It adds one
//!   current record of one kind the action may produce (its `may_produce`, and `case_outcome` when
//!   it opens a child case), in one dimension of that kind, of one class of that dimension and from
//!   a producer of one of its profiles. Each combination is its own successor; a producer profile
//!   is a free choice of each step, as a runner may use any producer. A record already there, or
//!   one from a producer independent on more, returns the same state. An action that may produce
//!   nothing, or nothing a record can be about, has one step, which returns the same state.
//! - **Supplied step.** A kind that declares `supplied_by` is produced by another protocol's
//!   cases (story:supplied-evidence), and no action step produces it: a record of it may arrive at
//!   any step, from any state, by the combinations an action step takes.
//! - **Expiry step (R2).** A kind that declares `max_age` may lose a record to time: one record
//!   class present of such a kind leaves the state, every producer's record of it. The space
//!   `canon check` covers has no representation of an expired record: it is atemporal, and an
//!   expired record is an absent one to every evidence match (`crate::check`, "The state space"),
//!   so the state without the class evaluates as a case in which those records are past their
//!   `max_age`. With it the step graph can loop.
//! - **Authority step.** A capability some action requires goes from undecided to `granted` or to
//!   `denied`.
//! - **Decision step.** A decision some outcome requires goes from not taken to taken.
//!
//! No step revises an artifact, terminates the case or adds a record at a superseded revision,
//! so the history dimensions of a `children` form stay empty.
//!
//! # Order
//!
//! There is one initial state per assignment of case-input values, in input order, the last input
//! varying fastest. Breadth-first search reaches each state first along a shortest path. From a
//! state, steps are tried in this order: actions in identifier order, then their kinds,
//! dimensions, classes and profiles in the space's own orders; supplied kinds the same way;
//! expiring kinds in dimension order, then classes; capabilities in identifier order, `granted`
//! before `denied`; decisions in name order.
//!
//! # Findings
//!
//! A state is *open* when some sequence of action, supplied, expiry and authority steps leads
//! from it to a state in which an outcome is legitimate (R3); decision steps are not part of that
//! sequence (R4), since a protocol with an outcome that requires only a decision can leave every
//! state by it. In this order:
//!
//! - `stuck-state`: a state that is not open, in which no action is `admissible` or
//!   `approval-required`.
//! - `cycle`: a state that is not open, in which no action is `approval-required`, some action is
//!   `admissible` and every step of every admissible action returns the same state. It names those
//!   actions (R6: with expiry steps the states the steps never leave can be a loop; the finding is
//!   the state of it in which every action step returns the same state).
//! - `dead-action`: an action `admissible` in no reachable state, in identifier order.
//! - `unreached-outcome`: an outcome legitimate in no reachable state, in identifier order; then
//!   `no-outcome-reachable`, once, when the protocol declares outcomes and every one is unreached.
//!
//! A state that meets the second half of the `stuck-state` or the `cycle` rule, in which no
//! outcome is legitimate, and that is open waits: on a supplied record or an expiry, the only
//! steps that can open it. It is no finding; the summary counts it when the protocol declares a
//! supplied or an expiring kind, `` W waiting on supplied evidence or expiry ``.
//!
//! One finding is printed per smallest state (R5): among the states of one kind, `stuck-state`
//! or `cycle`, one is printed when no other of that kind is contained in it. State A is contained
//! in state B when both give every case input the same value and every record of A (each class,
//! with the producer profiles it meets), every capability decision of A and every decision taken
//! in A is in B. Every reachable state that is not open leads by action and authority steps to a
//! state of one of the two kinds, and every state of a kind contains a printed one, so nothing is
//! left out. A printed finding names its state and its shortest path, then, when a decision step
//! leads from its state to a state with a legitimate outcome, each such decision in name order
//! and the outcomes it makes legitimate (R4), `` decision `d` taken makes outcome `o` legitimate
//! ``, and last, when other states of its kind contain its state, how many, `` 3 other stuck
//! states contain it `` (`` 1 other cycle state contains it ``).
//!
//! Printed stuck states and cycles are ordered by the length of their path, then by their
//! rendering. A path is written as its steps, `` action `a` produces evidence `k` result `r` ``,
//! `` protocol `s` supplies evidence `k` ``, `` evidence `k` expires ``, `` capability `c` denied
//! ``, `` decision `d` taken ``, joined by `, then `, or `empty`. The findings are followed by the
//! summary: `` simulated: protocol `p` revision 1: N reachable states, T terminable (outcome `a`
//! x, outcome `b` y), W waiting on supplied evidence or expiry, F findings (S stuck or cycle
//! states) ``: F counts printed findings and S the states the printed stuck states and cycles
//! stand for. A terminable state is one in which some outcome is legitimate; the walk continues
//! through it.
//!
//! # Refusals
//!
//! First, as `canon check` does, an IR nested beyond the evaluator's depth bound is refused as
//! `predicate-too-deep`, and claims that test each other in a cycle as `claim-cycle`. Then a
//! protocol with an order form (`first`, `before`, `after`, `since`) is refused as
//! `ordered-evidence`, naming the first declaration that has one: the space puts a dimension's
//! current records at one instant, so a sequence of records of one dimension has no state there.
//! Then the walk is refused as `reachable-state-bound` when it would visit more than
//! [`check::STATE_BOUND`] states. A walk that stops early reports nothing: no partial walk is
//! offered, since a walk that has not seen every reachable state cannot say that none is stuck.
//! An evaluation the evaluator refuses is refused with its code.
//!
//! # Known limit
//!
//! No artifact is revised. A kind read with `result:` that an action may produce twice with
//! different results gives a `cycle`, because mixed results leave the match `unknown` and the
//! walk cannot set the mixed records aside by a revision.

use std::collections::{BTreeSet, HashMap, VecDeque};
use std::fmt;

use crate::check::{self, Reader, Space, State, Walked};
use crate::eval;
use crate::ir::Ir;
use crate::model::{ActionId, DecisionName, EvidenceKindId, OutcomeId, ProtocolId, one_line};

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
/// the decisions that lead from it to a legitimate outcome and how many other states of its kind
/// contain it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dead {
    /// The state, as a report writes it.
    pub state: String,
    /// Its shortest path, one step each, as a report writes them.
    pub path: Vec<String>,
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
        /// The admissible actions, each of whose steps returns the state.
        actions: Vec<ActionId>,
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

/// `; path: …`, then the decisions and the count of containing states, each after `; `.
fn tail(dead: &Dead, kind: &str) -> String {
    let mut text = if dead.path.is_empty() {
        "; path: empty".to_owned()
    } else {
        format!("; path: {}", dead.path.join(", then "))
    };
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
            Finding::Cycle { dead, actions } => {
                let actions: Vec<String> = actions
                    .iter()
                    .map(|action| format!("action `{}`", one_line(action.as_str())))
                    .collect();
                write!(
                    f,
                    "state {}: no outcome is legitimate, no action is approval-required, and \
                     every step of {} returns this state{}",
                    dead.state,
                    actions.join(", "),
                    tail(dead, "cycle")
                )
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
    /// How many reachable states wait on a supplied record or an expiry; `None` when the protocol
    /// declares no supplied and no expiring kind, the only steps that can open such a state.
    pub waiting: Option<usize>,
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
/// supplied or an expiring kind.
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
            terminable.push_str(&format!(
                ", {waiting} waiting on supplied evidence or expiry"
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

/// One step of the walk, by position: a record an action or a supplier adds (the action's
/// position, `None` for a supplied one, then the dimension, class and profile), a record class
/// that expires, an authority decision or a decision taken.
#[derive(Clone, Copy, Debug)]
enum Step {
    Record {
        action: Option<usize>,
        dimension: usize,
        class: usize,
        profile: usize,
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

/// Where a walk's records sit after the space's state: one slot per evidence dimension and class,
/// each the down-closed set of producer profiles its records meet, as bits in whole words.
struct Layout<'d, 'a> {
    dimensions: &'d [Walked<'a>],
    /// The length of the space's state at the front of a walk's state.
    base: usize,
    /// For each dimension and class, the position of its slot's first word.
    slots: Vec<Vec<usize>>,
    /// For each dimension, how many words a slot of it takes.
    words: Vec<usize>,
}

impl<'d, 'a> Layout<'d, 'a> {
    fn new(dimensions: &'d [Walked<'a>], base: usize) -> Self {
        let mut at = base;
        let mut slots = Vec::new();
        let mut words = Vec::new();
        for walked in dimensions {
            let width = walked.profiles.len().div_ceil(32).max(1);
            words.push(width);
            slots.push(
                (0..walked.classes.len())
                    .map(|_| {
                        let start = at;
                        at += width;
                        start
                    })
                    .collect(),
            );
        }
        Self {
            dimensions,
            base,
            slots,
            words,
        }
    }

    /// The whole length of a walk's state.
    fn length(&self) -> usize {
        self.slots
            .iter()
            .zip(&self.words)
            .map(|(slots, words)| slots.len() * words)
            .sum::<usize>()
            + self.base
    }

    /// Whether profile `profile` is in the slot of `dimension` and `class` of `state`.
    fn has(&self, state: &State, dimension: usize, class: usize, profile: usize) -> bool {
        state[self.slots[dimension][class] + profile / 32] & (1 << (profile % 32)) != 0
    }

    /// `state` with one more record of `class` in `dimension`, from a producer of `profile`.
    fn add(&self, state: &State, dimension: usize, class: usize, profile: usize) -> State {
        let mut walked = state.clone();
        let within = &self.dimensions[dimension].within;
        for inner in 0..within.len() {
            if within[inner][profile] {
                walked[self.slots[dimension][class] + inner / 32] |= 1 << (inner % 32);
            }
        }
        walked
    }

    /// Whether `state` holds a record of `class` in `dimension`.
    fn present(&self, state: &State, dimension: usize, class: usize) -> bool {
        let start = self.slots[dimension][class];
        state[start..start + self.words[dimension]]
            .iter()
            .any(|word| *word != 0)
    }

    /// `state` without any record of `class` in `dimension`.
    fn expire(&self, state: &State, dimension: usize, class: usize) -> State {
        let mut walked = state.clone();
        let start = self.slots[dimension][class];
        walked[start..start + self.words[dimension]].fill(0);
        walked
    }

    /// The records of `state`, as the space writes them: each dimension and class that holds one,
    /// with the profiles of its records, one per greatest member of its set, in profile order.
    fn held(&self, state: &State) -> Vec<(usize, usize, Vec<usize>)> {
        let mut held = Vec::new();
        for (dimension, walked) in self.dimensions.iter().enumerate() {
            for class in 0..walked.classes.len() {
                let members: Vec<usize> = (0..walked.profiles.len())
                    .filter(|profile| self.has(state, dimension, class, *profile))
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
                held.push((dimension, class, greatest));
            }
        }
        held
    }

    /// Whether every record of `smaller` is in `larger`, its producer profiles included.
    fn records_within(&self, smaller: &State, larger: &State) -> bool {
        smaller[self.base..]
            .iter()
            .zip(&larger[self.base..])
            .all(|(a, b)| a & !b == 0)
    }

    /// How many things `state` holds: records' profiles, decided capabilities and taken
    /// decisions, and case inputs off their first value, which two comparable states share. A
    /// state contained in another holds fewer.
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
    /// admissible actions, or neither.
    idle: Option<Vec<usize>>,
    /// The states its action, supplied, expiry and authority steps lead to.
    next: Vec<usize>,
    /// The states its decision steps lead to, with each decision's position.
    decided: Vec<(usize, usize)>,
}

/// Walks every state `ir` reaches from the empty case, as the module docs say. Pure and
/// deterministic.
pub fn simulate(ir: &Ir) -> Result<Report, Refusal> {
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
                 `after` or `since`), which canon simulate does not walk"
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

    let dimensions = space.walked();
    let initial = space.initial_states();
    let base = initial.first().map_or(0, Vec::len);
    let layout = Layout::new(&dimensions, base);
    let of_kinds = |kinds: &BTreeSet<&EvidenceKindId>| -> Vec<usize> {
        (0..dimensions.len())
            .filter(|at| kinds.contains(dimensions[*at].kind))
            .collect()
    };
    // Every record step a set of dimensions offers, in dimension, class and profile order.
    let records = |at: &[usize]| -> Vec<(usize, usize, usize)> {
        at.iter()
            .flat_map(|dimension| {
                let walked = &dimensions[*dimension];
                (0..walked.classes.len()).flat_map(move |class| {
                    (0..walked.profiles.len()).map(move |profile| (*dimension, class, profile))
                })
            })
            .collect()
    };
    let action_records: Vec<Vec<(usize, usize, usize)>> = ir
        .actions
        .values()
        .map(|action| records(&of_kinds(&check::produces(ir, action).collect())))
        .collect();
    let supplied_kinds: BTreeSet<&EvidenceKindId> = check::supplied(ir).collect();
    let supplied_records = records(&of_kinds(&supplied_kinds));
    let expiring_kinds: BTreeSet<&EvidenceKindId> = ir
        .evidence_kinds
        .iter()
        .filter(|(_, kind)| kind.max_age.is_some())
        .map(|(id, _)| id)
        .collect();
    let expiring = of_kinds(&expiring_kinds);
    let capabilities = space.walked_capabilities().len();
    let decisions = space.walked_decisions().len();

    let mut states: Vec<State> = initial
        .into_iter()
        .map(|mut state| {
            state.resize(layout.length(), 0);
            state
        })
        .collect();
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
            space.walk_evidence(&held),
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
        for action in &admissible {
            admissible_somewhere[*action] = true;
            for (dimension, class, profile) in &action_records[*action] {
                let walked = layout.add(&state, *dimension, *class, *profile);
                if walked != state {
                    returns = false;
                    successors.push((
                        walked,
                        Step::Record {
                            action: Some(*action),
                            dimension: *dimension,
                            class: *class,
                            profile: *profile,
                        },
                    ));
                }
            }
        }
        for (dimension, class, profile) in &supplied_records {
            let walked = layout.add(&state, *dimension, *class, *profile);
            if walked != state {
                successors.push((
                    walked,
                    Step::Record {
                        action: None,
                        dimension: *dimension,
                        class: *class,
                        profile: *profile,
                    },
                ));
            }
        }
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
            next: followed,
            decided,
        });
        next += 1;
    }

    // Open states: those from which action, supplied, expiry and authority steps reach a state
    // with a legitimate outcome, found backwards from those states.
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
    let mut stuck: Vec<usize> = Vec::new();
    let mut cycles: Vec<usize> = Vec::new();
    for (at, state) in visited.iter().enumerate() {
        let Some(admissible) = &state.idle else {
            continue;
        };
        if open[at] {
            waiting += 1;
        } else if admissible.is_empty() {
            stuck.push(at);
        } else {
            cycles.push(at);
        }
    }

    let render_step = |step: &Step| -> String {
        let record = |dimension: usize, class: usize| {
            let walked = &dimensions[dimension];
            let mut record = format!("evidence {}", walked.name);
            if let Some(result) = walked.classes[class] {
                record.push_str(&format!(" result `{}`", one_line(result)));
            }
            record
        };
        match *step {
            Step::Record {
                action,
                dimension,
                class,
                profile,
            } => {
                let walked = &dimensions[dimension];
                let record = format!("{}{}", record(dimension, class), walked.profiles[profile]);
                match action {
                    Some(action) => {
                        let id = ir.actions.keys().nth(action).expect("a declared action");
                        format!("action `{}` produces {record}", one_line(id.as_str()))
                    }
                    None => {
                        let supplier = ir
                            .evidence_kinds
                            .get(walked.kind)
                            .and_then(|kind| kind.supplied_by.as_ref())
                            .expect("a supplied kind");
                        format!(
                            "protocol `{}` supplies {record}",
                            one_line(supplier.as_str())
                        )
                    }
                }
            }
            Step::Expiry { dimension, class } => format!("{} expires", record(dimension, class)),
            Step::Capability { at, granted } => format!(
                "capability `{}` {}",
                one_line(space.walked_capabilities()[at].as_str()),
                if granted { "granted" } else { "denied" }
            ),
            Step::Decision { at } => format!(
                "decision `{}` taken",
                one_line(space.walked_decisions()[at].as_str())
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
    let rendered =
        |at: usize| space.render_walk(&states[at][..base].to_vec(), &layout.held(&states[at]));
    let contains = |smaller: usize, larger: usize| {
        space.decided_within(&states[smaller], &states[larger])
            && layout.records_within(&states[smaller], &states[larger])
    };
    // The smallest states of one kind, each with how many other states of the kind contain it.
    let smallest = |of: &[usize]| -> Vec<(usize, usize)> {
        let mut by_size: Vec<usize> = of.to_vec();
        by_size.sort_by_key(|at| (layout.size(&states[*at]), *at));
        let mut found: Vec<usize> = Vec::new();
        for at in &by_size {
            if !found.iter().any(|small| contains(*small, *at)) {
                found.push(*at);
            }
        }
        found
            .into_iter()
            .map(|small| {
                let containing = of
                    .iter()
                    .filter(|other| **other != small && contains(small, **other))
                    .count();
                (small, containing)
            })
            .collect()
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
    for (at, containing) in ordered_findings(&cycles) {
        let actions = visited[at]
            .idle
            .as_ref()
            .expect("a cycle state is idle")
            .iter()
            .map(|action| {
                ir.actions
                    .keys()
                    .nth(*action)
                    .expect("a declared action")
                    .clone()
            })
            .collect();
        findings.push(Finding::Cycle {
            dead: dead(at, containing),
            actions,
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
    Ok(Report {
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
        waiting: (!supplied_kinds.is_empty() || !expiring_kinds.is_empty()).then_some(waiting),
        dead: stuck.len() + cycles.len(),
        findings,
    })
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
