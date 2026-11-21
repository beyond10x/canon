//! A breadth-first walk over the states a case of a compiled protocol can reach from the empty
//! case (story:canon-simulate).
//!
//! [`simulate`] walks every reachable state, one step at a time, evaluates each with Canon's own
//! evaluator and reports what no single evaluation and no static check shows: states where a case
//! is stuck, with the shortest path that reaches each, cycles, actions that are never admissible
//! and outcomes that are never legitimate. It is a pure function of the compiled protocol: no
//! clock, network, filesystem or runner, and the same protocol gives the same [`Report`], byte for
//! byte. A step never runs an action.
//!
//! # States
//!
//! A state of the walk is a state of the whole product `canon check` covers (`crate::check`, "The
//! state space"): one value per dimension of the whole space, evaluated as `canon check` evaluates
//! one, as a case snapshot with every artifact at one revision, no termination and no evaluation
//! instant. `canon check` decides each subject over its slice and may enumerate no state of the
//! whole product; the walk enumerates none either, and visits only the states its steps reach, so
//! its bound counts reachable states, not the product. Every dimension keeps the classes,
//! upstream artifacts and producer profiles it has in the whole space.
//!
//! # Steps
//!
//! A step only adds, so the step graph has no cycle except a step that returns its own state.
//!
//! - **Action step.** Taken only from a state in which the action is `admissible`. It adds one
//!   current record of one kind the action may produce (its `may_produce`, and `case_outcome` when
//!   it opens a child case), in one dimension of that kind, of one class of that dimension and from
//!   a producer of one of its profiles: at the current revision, observed after every move of an
//!   upstream artifact. Each combination is its own successor; a producer profile is a free choice
//!   of each step, as a runner may use any producer. A record already there, or one from a
//!   producer independent on more, returns the same state. An action that may produce nothing,
//!   or nothing a record can be about, has one step, which returns the same state.
//! - **Supplied step.** A kind that declares `supplied_by` is produced by another protocol's
//!   cases (story:supplied-evidence), and no action step produces it: a record of it may arrive at
//!   any step, from any state, as a supplied step adds one, by the combinations an action step
//!   takes. A state that would be stuck, or a cycle, except that a supplied step leads out of it
//!   waits on supplied evidence: it is no finding, and the summary counts it, `` W waiting on
//!   supplied evidence ``, whenever the protocol declares a supplied kind.
//! - **Authority step.** A capability some action requires goes from undecided to `granted` or to
//!   `denied`.
//! - **Decision step.** A decision some outcome requires goes from not taken to taken.
//!
//! No step revises an artifact, removes a record, terminates the case or adds a record at a
//! superseded revision, so the history dimensions of a `children` form stay empty.
//!
//! # Order
//!
//! There is one initial state per assignment of case-input values, in input order, the last input
//! varying fastest. Breadth-first search reaches each state first along a shortest path. From a
//! state, steps are tried in this order: actions in identifier order, then their kinds,
//! dimensions, classes and profiles in the space's own orders; supplied kinds the same way;
//! capabilities in identifier order, `granted` before `denied`; decisions in name order.
//!
//! # Findings
//!
//! In this order:
//!
//! - `stuck-state`: a state in which no outcome is legitimate and no action is `admissible` or
//!   `approval-required`. It names the state and its shortest path.
//! - `cycle`: a state in which no outcome is legitimate, no action is `approval-required`, some
//!   action is `admissible` and every step of every admissible action returns the same state. It
//!   names the state, its path and those actions.
//! - `dead-action`: an action `admissible` in no reachable state, in identifier order.
//! - `unreached-outcome`: an outcome legitimate in no reachable state, in identifier order; then
//!   `no-outcome-reachable`, once, when the protocol declares outcomes and every one is unreached.
//!
//! Stuck states and cycles are ordered by the length of their path, then by their rendering. A
//! path is written as its steps, `` action `a` produces evidence `k` result `r` ``, `` protocol
//! `s` supplies evidence `k` ``, `` capability `c` denied ``, `` decision `d` taken ``, joined by
//! `, then `, or `empty`. The findings are followed by the summary: `` simulated: protocol `p`
//! revision 1: N reachable states, T terminable (outcome `a` x, outcome `b` y), F findings ``. A
//! terminable state is one in which some outcome is legitimate; the walk continues through it.
//!
//! # Refusals
//!
//! First, as `canon check` does, an IR nested beyond the evaluator's depth bound is refused as
//! `predicate-too-deep`, and claims that test each other in a cycle as `claim-cycle`. Then a
//! protocol with an order form (`first`, `before`, `after`, `since`) is refused as
//! `ordered-evidence`, naming the first declaration that has one: the space puts a dimension's
//! current records at one instant, so a sequence of records of one dimension has no state there.
//! Then the walk is refused as `reachable-state-bound` when an evidence dimension has more values
//! than a state holds, naming it, or when it would visit more than [`check::STATE_BOUND`] states.
//! A walk that stops early reports nothing: no partial walk is offered, since a walk that has not
//! seen every reachable state cannot say that none is stuck. An evaluation the evaluator refuses
//! is refused with its code.

use std::collections::{BTreeSet, HashMap};
use std::fmt;

use crate::check::{self, Reader, Space, State, Walked};
use crate::eval;
use crate::ir::Ir;
use crate::model::{ActionId, EvidenceKindId, OutcomeId, ProtocolId, one_line};

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

/// One thing a walk found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Finding {
    StuckState {
        /// The state, as a report writes it.
        state: String,
        /// Its shortest path, one step each, as a report writes them.
        path: Vec<String>,
    },
    Cycle {
        state: String,
        path: Vec<String>,
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
            Finding::StuckState { .. } => "stuck-state",
            Finding::Cycle { .. } => "cycle",
            Finding::DeadAction { .. } => "dead-action",
            Finding::UnreachedOutcome { .. } => "unreached-outcome",
            Finding::NoOutcomeReachable { .. } => "no-outcome-reachable",
        }
    }
}

/// A path as a report writes it: `empty`, or its steps joined by `, then `.
fn path_text(path: &[String]) -> String {
    if path.is_empty() {
        "empty".to_owned()
    } else {
        path.join(", then ")
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.code())?;
        match self {
            Finding::StuckState { state, path } => write!(
                f,
                "state {state}: no outcome is legitimate and no action is admissible or \
                 approval-required; path: {}",
                path_text(path)
            ),
            Finding::Cycle {
                state,
                path,
                actions,
            } => {
                let actions: Vec<String> = actions
                    .iter()
                    .map(|action| format!("action `{}`", one_line(action.as_str())))
                    .collect();
                write!(
                    f,
                    "state {state}: no outcome is legitimate, no action is approval-required, and \
                     every step of {} returns this state; path: {}",
                    actions.join(", "),
                    path_text(path)
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
    /// How many reachable states wait on supplied evidence; `None` when the protocol declares no
    /// supplied kind.
    pub waiting: Option<usize>,
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
/// states, 2 terminable (outcome `a` 2), 0 findings ``, with `` , 2 waiting on supplied evidence
/// `` before the findings when the protocol declares a supplied kind.
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
            terminable.push_str(&format!(", {waiting} waiting on supplied evidence"));
        }
        writeln!(
            f,
            "simulated: protocol `{}` revision {}: {}, {terminable}, {}",
            one_line(self.protocol.as_str()),
            self.revision,
            counted(self.states, "reachable state", "reachable states"),
            counted(self.findings.len(), "finding", "findings"),
        )
    }
}

/// One step of the walk, by position: an action's or a supplied record (the action's position,
/// `None` for a supplied one, then the dimension, class and profile), an authority decision or a
/// decision taken.
#[derive(Clone, Copy, Debug)]
enum Step {
    Record {
        action: Option<usize>,
        dimension: usize,
        class: usize,
        profile: usize,
    },
    Capability {
        at: usize,
        granted: bool,
    },
    Decision {
        at: usize,
    },
}

/// What the walk keeps of a reachable state: how it was first reached, and its depth, the length
/// of its shortest path.
struct Visited {
    parent: Option<(usize, Step)>,
    depth: usize,
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
    if let Some(dimension) = space.unwalkable() {
        return Err(Refusal::new(
            "reachable-state-bound",
            format!("{protocol}: {dimension} values, more than a state of the walk holds"),
        ));
    }
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
    let of_kinds = |kinds: &BTreeSet<&EvidenceKindId>| -> Vec<usize> {
        (0..dimensions.len())
            .filter(|at| kinds.contains(dimensions[*at].kind))
            .collect()
    };
    // The dimensions each action's records go into, in dimension order, which is kind order.
    let produced: Vec<Vec<usize>> = ir
        .actions
        .values()
        .map(|action| of_kinds(&check::produces(ir, action).collect()))
        .collect();
    let supplied_kinds: BTreeSet<&EvidenceKindId> = check::supplied(ir).collect();
    let supplied = of_kinds(&supplied_kinds);
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
    let action_records: Vec<Vec<(usize, usize, usize)>> =
        produced.iter().map(|at| records(at)).collect();
    let supplied_records = records(&supplied);
    let capabilities = space.walked_capabilities().len();
    let decisions = space.walked_decisions().len();

    let mut states: Vec<State> = space.initial_states();
    let mut index: HashMap<State, usize> = states
        .iter()
        .enumerate()
        .map(|(at, state)| (state.clone(), at))
        .collect();
    let mut visited: Vec<Visited> = states
        .iter()
        .map(|_| Visited {
            parent: None,
            depth: 0,
        })
        .collect();
    let mut admissible_somewhere = vec![false; ir.actions.len()];
    let mut legitimate_in = vec![0usize; ir.outcomes.len()];
    let (mut terminable, mut waiting) = (0usize, 0usize);
    // Each stuck state and cycle, by state position, with a cycle's actions.
    let mut stuck: Vec<usize> = Vec::new();
    let mut cycles: Vec<(usize, Vec<usize>)> = Vec::new();

    let mut next = 0;
    while next < states.len() {
        let state = states[next].clone();
        let evaluated = check::evaluate(ir, &space, &state)
            .map_err(|refusal| Refusal::new(refusal.code(), refusal.to_string()))?;
        let legitimate: Vec<usize> = (0..ir.outcomes.len())
            .filter(|at| evaluated.outcomes[*at] == "legitimate")
            .collect();
        for at in &legitimate {
            legitimate_in[*at] += 1;
        }
        terminable += usize::from(!legitimate.is_empty());
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
                let walked = space.with_record(&state, *dimension, *class, *profile);
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
        let mut supply = false;
        for (dimension, class, profile) in &supplied_records {
            let walked = space.with_record(&state, *dimension, *class, *profile);
            if walked != state {
                supply = true;
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
        for at in 0..capabilities {
            for granted in [true, false] {
                if let Some(walked) = space.with_capability(&state, at, granted) {
                    successors.push((walked, Step::Capability { at, granted }));
                }
            }
        }
        for at in 0..decisions {
            if let Some(walked) = space.with_decision(&state, at) {
                successors.push((walked, Step::Decision { at }));
            }
        }

        let idle = legitimate.is_empty() && !approval;
        let is_stuck = idle && admissible.is_empty();
        let is_cycle = idle && !admissible.is_empty() && returns;
        if (is_stuck || is_cycle) && supply {
            waiting += 1;
        } else if is_stuck {
            stuck.push(next);
        } else if is_cycle {
            cycles.push((next, admissible.clone()));
        }

        let depth = visited[next].depth + 1;
        for (walked, step) in successors {
            if index.contains_key(&walked) {
                continue;
            }
            if states.len() >= limit {
                return Err(bound());
            }
            index.insert(walked.clone(), states.len());
            states.push(walked);
            visited.push(Visited {
                parent: Some((next, step)),
                depth,
            });
        }
        next += 1;
    }

    let render_step = |step: &Step| -> String {
        match *step {
            Step::Record {
                action,
                dimension,
                class,
                profile,
            } => {
                let walked: &Walked<'_> = &dimensions[dimension];
                let mut record = format!("evidence {}", walked.name);
                if let Some(result) = walked.classes[class] {
                    record.push_str(&format!(" result `{}`", one_line(result)));
                }
                record.push_str(&walked.profiles[profile]);
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
    let by_path = |at: usize| (visited[at].depth, space.rendered(&states[at]));

    let mut found_stuck: Vec<((usize, String), usize)> =
        stuck.into_iter().map(|at| (by_path(at), at)).collect();
    found_stuck.sort();
    let mut found_cycles: Vec<((usize, String), usize, Vec<usize>)> = cycles
        .into_iter()
        .map(|(at, actions)| (by_path(at), at, actions))
        .collect();
    found_cycles.sort();

    let mut findings = Vec::new();
    for ((_, state), at) in found_stuck {
        findings.push(Finding::StuckState {
            state,
            path: path(at),
        });
    }
    for ((_, state), at, actions) in found_cycles {
        findings.push(Finding::Cycle {
            state,
            path: path(at),
            actions: actions
                .into_iter()
                .map(|action| {
                    ir.actions
                        .keys()
                        .nth(action)
                        .expect("a declared action")
                        .clone()
                })
                .collect(),
        });
    }
    for (id, reached) in ir.actions.keys().zip(&admissible_somewhere) {
        if !reached {
            findings.push(Finding::DeadAction { action: id.clone() });
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
        terminable,
        outcomes: ir
            .outcomes
            .keys()
            .cloned()
            .zip(legitimate_in.iter().copied())
            .collect(),
        waiting: (!supplied_kinds.is_empty()).then_some(waiting),
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
