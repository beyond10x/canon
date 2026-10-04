//! Exhaustive checking of a compiled protocol over its finite state space (story:canon-check).
//!
//! [`check`] enumerates every state of a protocol, evaluates each with Canon's own evaluator
//! ([`crate::eval::evaluate_with`]) and reports what no single evaluation shows. It is a pure
//! function of the compiled protocol and the `canon-properties/1` properties: no clock, network or
//! filesystem, and the same inputs give the same [`Report`], byte for byte.
//!
//! # The state space
//!
//! The dimensions, in this order: each declared evidence kind, in identifier order; each capability
//! some action requires, in identifier order; each decision name some outcome requires, in
//! identifier order.
//!
//! - An evidence kind's classes are each distinct result the protocol's predicates match for the
//!   kind, in code-point order, then a record with no result, which stands for every other result:
//!   against every evidence match the protocol writes, a record with another result behaves as one
//!   with none. The kind takes every subset of its classes, so `2^(results + 1)` values; the empty
//!   set is the kind absent. A protocol that declares no artifact has no record any evidence could
//!   be about, so each of its kinds has the one value absent.
//! - A capability is undecided, granted or denied (3 values).
//! - A decision is not taken or taken (2 values); taken, it is taken for every outcome that
//!   requires it.
//!
//! The space is atemporal: no record carries an observation instant, so none expires (an expired
//! record is an absent one). Obligation status is not a dimension: it is computed from the claims in
//! each state, and no `protocol/1` predicate reads it.
//!
//! A state is evaluated as a case snapshot that lists each declared artifact, and the case itself,
//! at one revision, with one evidence record per present class about the first declared artifact,
//! the decided capabilities as `canon-authority/1` and the taken decisions as `canon-decisions/1`.
//!
//! # The bound
//!
//! A space of more than [`STATE_BOUND`] states (65 536) is refused as `state-space-bound`, before
//! any state is evaluated, naming the count, the bound and each dimension with its number of values:
//! `` protocol `p` revision 1 has 98304 states, more than the bound of 65536: evidence kind `k`
//! 1024, evidence kind `m` 16, capability `c` 3, decision `d` 2 ``.
//!
//! # Findings
//!
//! In this order, each group in identifier order:
//!
//! - `unreachable-outcome`: an outcome legitimate in no state.
//! - `unsatisfiable-precondition`: an action whose precondition is `true` in no state, that is,
//!   neither `admissible` nor `approval-required` in any state.
//! - `unproduced-evidence`: a predicate that reads, directly or through the claims it tests, an
//!   evidence kind no action's `may_produce` lists. Every predicate is read: each claim's
//!   `true_when`, each obligation's `discharged_when`, each action's precondition and each outcome's
//!   requirement, in that order, each section in identifier order; one finding per declaration and
//!   kind, kinds in identifier order.
//! - `authority-bypass`: an outcome whose requirement holds because evidence an authority-requiring
//!   action may produce is present, in a state that needs no authority decision. Governed evidence
//!   is each kind some action that requires a capability may produce. The outcome is legitimate in
//!   a state whose every present evidence kind some action requiring no capability may produce, and
//!   not legitimate in that state with the governed kinds its requirement reads (directly or
//!   through claims) absent. An outcome that holds because governed evidence is absent, such as
//!   `{claim: approved, is: unknown}`, needs no authority decision and is no bypass. Static: action
//!   order and preconditions are not followed. The finding names the witness state.
//! - `property-failed`: a property whose subject has two statuses in two states that agree on every
//!   dimension except the evidence kinds its `independent_of` claim reads (directly or through
//!   claims). The counterexample is that pair of states.
//!
//! A witness state is the one of least weight (present classes, decided capabilities and taken
//! decisions), ties broken by its rendering in code-point order. The counterexample's first state is
//! the witness among the states that have such a partner, and its second the witness among that
//! state's partners. A state is written `{}`, or as its present classes, decided capabilities and
//! taken decisions in dimension order: `` {evidence `k` result `r`, evidence `k2`, capability `c`
//! granted, decision `d` taken} ``.
//!
//! # Refusals
//!
//! Before the bound, the properties are checked against the protocol, as
//! `properties.rs` says: `unsupported-format`, `protocol-mismatch`, `invalid-identifier`,
//! `duplicate-identifier`, `undeclared-action`, `undeclared-outcome`, `undeclared-claim`. Text
//! that is not a `canon-properties/1` document is refused by [`read_properties`] as
//! `malformed-input`. An evaluation the evaluator refuses is refused with the evaluator's code.

mod properties;
mod space;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub use properties::read_properties;

use crate::eval::{self, Supplied};
use crate::ir::Ir;
use crate::model::{
    ActionId, CASE_FORMAT, Case, CaseArtifact, CaseId, ClaimId, Declarations, EvidenceKindId,
    ObligationId, OutcomeId, Properties, PropertyDependency, PropertyId, PropertySubject,
    ProtocolId, Revision, one_line,
};

use space::{Space, State};

/// The most states [`check`] evaluates; a larger space is refused.
pub const STATE_BOUND: u128 = 65_536;

/// Why a check was refused: a stable machine-readable code and a one-line message.
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

/// A declaration whose predicate reads evidence: a claim's `true_when`, an obligation's
/// `discharged_when`, an action's precondition or an outcome's requirement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reader {
    Claim(ClaimId),
    Obligation(ObligationId),
    Action(ActionId),
    Outcome(OutcomeId),
}

/// `` claim `c` ``, `` obligation `o` ``, `` action `a` `` or `` outcome `o` ``.
impl fmt::Display for Reader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (section, id) = match self {
            Reader::Claim(id) => ("claim", id.as_str()),
            Reader::Obligation(id) => ("obligation", id.as_str()),
            Reader::Action(id) => ("action", id.as_str()),
            Reader::Outcome(id) => ("outcome", id.as_str()),
        };
        write!(f, "{section} `{}`", one_line(id))
    }
}

/// One thing a check found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Finding {
    UnreachableOutcome {
        outcome: OutcomeId,
    },
    UnsatisfiablePrecondition {
        action: ActionId,
    },
    UnproducedEvidence {
        reader: Reader,
        kind: EvidenceKindId,
    },
    AuthorityBypass {
        outcome: OutcomeId,
        /// The witness state, as a report writes it.
        state: String,
    },
    PropertyFailed {
        property: PropertyId,
        /// The subject, as a report writes it: `` action `a` `` or `` outcome `o` ``.
        subject: String,
        /// The subject's status in the counterexample's first state, and that state.
        first: (String, String),
        /// The subject's other status in its second state, and that state.
        second: (String, String),
    },
}

impl Finding {
    /// A stable machine-readable code for the kind of finding.
    pub fn code(&self) -> &'static str {
        match self {
            Finding::UnreachableOutcome { .. } => "unreachable-outcome",
            Finding::UnsatisfiablePrecondition { .. } => "unsatisfiable-precondition",
            Finding::UnproducedEvidence { .. } => "unproduced-evidence",
            Finding::AuthorityBypass { .. } => "authority-bypass",
            Finding::PropertyFailed { .. } => "property-failed",
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", self.code())?;
        match self {
            Finding::UnreachableOutcome { outcome } => write!(
                f,
                "outcome `{}` is legitimate in no state",
                one_line(outcome.as_str())
            ),
            Finding::UnsatisfiablePrecondition { action } => write!(
                f,
                "action `{}`: its precondition holds in no state",
                one_line(action.as_str())
            ),
            Finding::UnproducedEvidence { reader, kind } => write!(
                f,
                "{reader} reads evidence kind `{}`, which no action may produce",
                one_line(kind.as_str())
            ),
            Finding::AuthorityBypass { outcome, state } => write!(
                f,
                "outcome `{}` is legitimate without authority in state {state}",
                one_line(outcome.as_str())
            ),
            Finding::PropertyFailed {
                property,
                subject,
                first,
                second,
            } => write!(
                f,
                "property `{}`: {subject} is {} in state {} and {} in state {}",
                one_line(property.as_str()),
                first.0,
                first.1,
                second.0,
                second.1
            ),
        }
    }
}

/// What a check found over the whole state space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub protocol: ProtocolId,
    pub revision: u64,
    /// How many states were evaluated.
    pub states: u128,
    /// How many properties were checked.
    pub properties: usize,
    pub findings: Vec<Finding>,
}

impl Report {
    /// Whether the check found nothing.
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }
}

/// One line per finding, then the summary: `` checked: protocol `p` revision 1: 48 states, 1
/// property, 0 findings ``.
impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for finding in &self.findings {
            writeln!(f, "{finding}")?;
        }
        writeln!(
            f,
            "checked: protocol `{}` revision {}: {}, {}, {}",
            one_line(self.protocol.as_str()),
            self.revision,
            counted(self.states, "state", "states"),
            counted(self.properties as u128, "property", "properties"),
            counted(self.findings.len() as u128, "finding", "findings"),
        )
    }
}

fn counted(count: u128, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

/// One state's evaluation: each action's status and each outcome's, in identifier order.
struct Evaluated {
    actions: Vec<String>,
    outcomes: Vec<String>,
}

/// Checks `ir` over its whole state space, and each of `properties` when given, as the module docs
/// say. Pure and deterministic.
pub fn check(ir: &Ir, properties: Option<&Properties>) -> Result<Report, Refusal> {
    if let Some(properties) = properties {
        properties::validate(ir, properties)?;
    }
    let space = Space::new(ir);
    let size = match space.size() {
        Some(size) if size <= STATE_BOUND => size,
        size => {
            let count = size.map_or_else(|| format!("more than {}", u128::MAX), |n| n.to_string());
            return Err(Refusal::new(
                "state-space-bound",
                format!(
                    "protocol `{}` revision {} has {count} states, more than the bound of \
                     {STATE_BOUND}: {}",
                    one_line(ir.protocol.id.as_str()),
                    ir.protocol.revision,
                    space.dimensions()
                ),
            ));
        }
    };
    let states: Vec<State> = (0..size).map(|index| space.state(index)).collect();
    let evaluated = states
        .iter()
        .map(|state| evaluate(ir, &space, state))
        .collect::<Result<Vec<_>, _>>()?;

    let mut findings = Vec::new();
    for (at, outcome) in ir.outcomes.keys().enumerate() {
        if !evaluated.iter().any(|e| e.outcomes[at] == "legitimate") {
            findings.push(Finding::UnreachableOutcome {
                outcome: outcome.clone(),
            });
        }
    }
    for (at, action) in ir.actions.keys().enumerate() {
        let holds =
            |e: &Evaluated| matches!(e.actions[at].as_str(), "admissible" | "approval-required");
        if !evaluated.iter().any(holds) {
            findings.push(Finding::UnsatisfiablePrecondition {
                action: action.clone(),
            });
        }
    }
    let produced: BTreeSet<&EvidenceKindId> = ir
        .actions
        .values()
        .flat_map(|action| &action.may_produce)
        .collect();
    for (reader, predicate) in space::readers(ir) {
        for kind in space::kinds_read(ir, predicate) {
            if !produced.contains(kind) {
                findings.push(Finding::UnproducedEvidence {
                    reader: reader.clone(),
                    kind: kind.clone(),
                });
            }
        }
    }
    findings.extend(bypasses(ir, &space, &states, &evaluated));
    if let Some(properties) = properties {
        findings.extend(failed_properties(
            ir, &space, &states, &evaluated, properties,
        ));
    }
    Ok(Report {
        protocol: ir.protocol.id.clone(),
        revision: ir.protocol.revision,
        states: size,
        properties: properties.map_or(0, |p| p.properties.len()),
        findings,
    })
}

/// Evaluates one state with the evaluator and keeps each action's and outcome's status.
fn evaluate(ir: &Ir, space: &Space<'_>, state: &State) -> Result<Evaluated, Refusal> {
    let revision = Revision::new(space::REVISION);
    let case = Case {
        format: CASE_FORMAT.to_owned(),
        id: CaseId::new("check"),
        protocol: ir.protocol.id.clone(),
        artifacts: Declarations::new(
            ir.artifacts
                .keys()
                .map(|id| {
                    let current = CaseArtifact {
                        revision: revision.clone(),
                    };
                    (id.clone(), current)
                })
                .collect(),
        ),
        termination: None,
        revision: Some(revision),
    };
    let evidence = space.evidence(state);
    let authority = space.authority(state);
    let decisions = space.decisions(state);
    let supplied = Supplied {
        authority: Some(&authority),
        at: None,
        decisions: Some(&decisions),
    };
    let decision = eval::evaluate_with(ir, &case, &evidence, supplied)
        .map_err(|refusal| Refusal::new(refusal.code(), refusal.to_string()))?;
    let status = |section: &Option<crate::model::Json>, id: &str| {
        section
            .as_ref()
            .and_then(|entries| entries.get(id))
            .and_then(|entry| entry.get("status"))
            .and_then(|status| status.as_str())
            .unwrap_or_default()
            .to_owned()
    };
    Ok(Evaluated {
        actions: ir
            .actions
            .keys()
            .map(|id| status(&decision.actions, id.as_str()))
            .collect(),
        outcomes: ir
            .outcomes
            .keys()
            .map(|id| status(&decision.outcomes, id.as_str()))
            .collect(),
    })
}

/// The index of the state of least weight among the `candidates` indices into `states`, ties
/// broken by rendering.
fn witness(
    space: &Space<'_>,
    states: &[State],
    candidates: impl Iterator<Item = usize>,
) -> Option<usize> {
    candidates.min_by_key(|index| (space.weight(&states[*index]), space.render(&states[*index])))
}

/// The `authority-bypass` findings, in outcome order.
fn bypasses(ir: &Ir, space: &Space<'_>, states: &[State], evaluated: &[Evaluated]) -> Vec<Finding> {
    let governed: BTreeSet<&EvidenceKindId> = ir
        .actions
        .values()
        .filter(|action| !action.requires.is_empty())
        .flat_map(|action| &action.may_produce)
        .collect();
    let free: BTreeSet<&EvidenceKindId> = ir
        .actions
        .values()
        .filter(|action| action.requires.is_empty())
        .flat_map(|action| &action.may_produce)
        .collect();
    let positions = space.kind_positions();
    let legitimate = |state: &State, at: usize| {
        let index = usize::try_from(space.index(state)).expect("the space is within the bound");
        evaluated[index].outcomes[at] == "legitimate"
    };
    let mut findings = Vec::new();
    for (at, (id, outcome)) in ir.outcomes.iter().enumerate() {
        let Some(requirement) = outcome.requires.predicate() else {
            continue;
        };
        // The dimensions of the governed kinds the requirement reads.
        let read_governed: BTreeSet<usize> = space::kinds_read(ir, requirement)
            .into_iter()
            .filter(|kind| governed.contains(kind))
            .filter_map(|kind| positions.get(kind).copied())
            .collect();
        if read_governed.is_empty() {
            continue;
        }
        // Legitimate with only evidence no authority is needed for, and not once the governed
        // evidence is taken away: it holds because that evidence is present.
        let candidates = (0..states.len()).filter(|index| {
            let state = &states[*index];
            let without_authority = space.present(state).iter().all(|kind| free.contains(kind));
            without_authority
                && legitimate(state, at)
                && !legitimate(&space.erase(state, &read_governed), at)
        });
        if let Some(index) = witness(space, states, candidates) {
            findings.push(Finding::AuthorityBypass {
                outcome: id.clone(),
                state: space.render(&states[index]),
            });
        }
    }
    findings
}

/// The `property-failed` findings, in property identifier order.
fn failed_properties(
    ir: &Ir,
    space: &Space<'_>,
    states: &[State],
    evaluated: &[Evaluated],
    properties: &Properties,
) -> Vec<Finding> {
    let positions = space.kind_positions();
    let mut ordered: Vec<_> = properties.properties.iter().collect();
    ordered.sort_by(|a, b| a.0.cmp(b.0));
    let mut findings = Vec::new();
    for (id, property) in ordered {
        // The subject as a report writes it, whether it is an action, and its position.
        let (subject, is_action, at) = match &property.subject {
            PropertySubject::Action(action) => (
                format!("action `{}`", one_line(action.as_str())),
                true,
                ir.actions.keys().position(|a| a == action),
            ),
            PropertySubject::Outcome(outcome) => (
                format!("outcome `{}`", one_line(outcome.as_str())),
                false,
                ir.outcomes.keys().position(|o| o == outcome),
            ),
        };
        let at = at.expect("validated: the subject is declared");
        // The subject's status in each state, by state index.
        let status: Vec<&str> = evaluated
            .iter()
            .map(|e| {
                if is_action {
                    e.actions[at].as_str()
                } else {
                    e.outcomes[at].as_str()
                }
            })
            .collect();
        let PropertyDependency::Claim(claim) = &property.independent_of;
        let true_when = &ir.claims.get(claim).expect("validated").true_when;
        let erased: BTreeSet<usize> = space::kinds_read(ir, true_when)
            .into_iter()
            .filter_map(|kind| positions.get(kind).copied())
            .collect();
        // States that agree outside the erased kinds share a group; a state has a partner exactly
        // when its group holds more than one status.
        let mut groups: BTreeMap<State, Vec<usize>> = BTreeMap::new();
        for (index, state) in states.iter().enumerate() {
            groups
                .entry(space.erase(state, &erased))
                .or_default()
                .push(index);
        }
        let mixed: Vec<&Vec<usize>> = groups
            .values()
            .filter(|members| {
                let statuses: BTreeSet<&str> = members.iter().map(|m| status[*m]).collect();
                statuses.len() > 1
            })
            .collect();
        let Some(first) = witness(space, states, mixed.iter().flat_map(|g| g.iter().copied()))
        else {
            continue;
        };
        let group = &groups[&space.erase(&states[first], &erased)];
        let other = group
            .iter()
            .copied()
            .filter(|m| status[*m] != status[first]);
        let second = witness(space, states, other).expect("the first state has a partner");
        findings.push(Finding::PropertyFailed {
            property: id.clone(),
            subject,
            first: (status[first].to_owned(), space.render(&states[first])),
            second: (status[second].to_owned(), space.render(&states[second])),
        });
    }
    findings
}
