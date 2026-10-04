//! Exhaustive checking of a compiled protocol over its finite state space (story:canon-check).
//!
//! [`check`] enumerates every state of a protocol, evaluates each with Canon's own evaluator
//! ([`crate::eval::evaluate_with`]) and reports what no single evaluation shows. It is a pure
//! function of the compiled protocol and the `canon-properties/1` properties: no clock, network or
//! filesystem, and the same inputs give the same [`Report`], byte for byte.
//!
//! # The state space
//!
//! The dimensions, in this order: the evidence dimensions of each declared evidence kind, in
//! identifier order; each capability some action requires, in identifier order; each decision name
//! some outcome requires, in identifier order.
//!
//! - An evidence dimension is the records of one kind about one artifact. A kind has one bound
//!   dimension per artifact some evidence match of the kind names as its `subject`, in identifier
//!   order. After them it has one unbound dimension, about an artifact no match of the kind names
//!   (the first such in identifier order), when such an artifact exists and either some match of
//!   the kind names no subject or none names one (a kind no predicate reads keeps its one
//!   dimension). Only a match that names no subject reads the unbound dimension. A match reads a
//!   dimension's records by the evaluator's own rule (`crate::eval::reads`): a match that names a
//!   subject reads the bound dimension about it, and a match that names none reads every dimension
//!   of its kind.
//! - A dimension's classes are each distinct result a match that reads it matches, in code-point
//!   order, then a record with no result, which stands for every other result: against every
//!   evidence match that reads the dimension, a record with another result behaves as one with
//!   none. The dimension takes every subset of its classes, so `2^(results + 1)` values; the empty
//!   set is no record. A protocol that declares no artifact has no record any evidence could be
//!   about, so each of its kinds has one dimension with the one value absent.
//! - A capability is undecided, granted or denied (3 values).
//! - A decision is not taken or taken (2 values); taken, it is taken for every outcome that
//!   requires it.
//! - An evidence dimension has upstream artifacts when an invalidation rule can keep its records
//!   from a claim: the upstream artifact of each rule that invalidates a claim (one it names, or
//!   one built on those, by the evaluator's own rule, `crate::eval::invalidated_claims`) that
//!   reaches an evidence match reading the dimension, in its own predicate or a claim it tests:
//!   the evaluator keeps such a record from every match in that claim's evaluation. Each class of
//!   such a dimension then comes in one variant per upstream vector, which says for each upstream
//!   artifact whether the record was observed before it moved, and the dimension takes every
//!   subset of those variants: a class can be present in any combination of its `2^m` variants, as
//!   one record each. A dimension of `c` classes and `m` upstream artifacts has `2^(c * 2^m)`
//!   values. Rules that share an upstream
//!   artifact share its variants, since one recorded revision moves for all of them. A record
//!   observed before a move records an earlier revision of the artifact than the case's current
//!   one, and the evaluator's invalidation stage decides which claims it is kept from. A dimension
//!   without upstream artifacts, and every dimension of a protocol that declares no invalidation
//!   rule, has its classes alone, as before rules existed.
//!
//! The space is atemporal: no record carries an observation instant, so none expires (an expired
//! record is an absent one). Invalidation is not absence: a record invalidated for some claims
//! still applies to the others, so it has variants of its own. Obligation status is not a
//! dimension: it is computed from the claims in each state, and no `protocol/1` predicate reads it.
//!
//! A state is evaluated as a case snapshot that lists each declared artifact, and the case itself,
//! at one revision, with one evidence record per present class and variant about its dimension's
//! artifact, the decided capabilities as `canon-authority/1` and the taken decisions as
//! `canon-decisions/1`.
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
//! - `authority-bypass`: an outcome that reads governed evidence where its presence helps it hold,
//!   and that is legitimate without authority because of evidence present. Governed evidence is
//!   each kind some action that requires a capability may produce. A requirement reads an evidence
//!   match positively unless it sits under an odd number of flips, through the claims it tests: a
//!   `not`, a claim test `is: false` and a claim test `is: unknown` each flip, and `all`, `any` and
//!   `is: true` do not. The outcome is a bypass when some evidence dimension it reads positively is
//!   of a governed kind, and it is legitimate in a state that takes no authority decision, whose
//!   every present evidence kind some action requiring no capability may produce, and from which
//!   taking away one positively read dimension that is present blocks it. Each such dimension is
//!   judged alone, so the outcome may rest on governed evidence produced without authority or on
//!   other evidence beside it. An outcome that holds only because governed evidence is absent,
//!   such as `{claim: approved, is: unknown}`, needs no authority decision and is no bypass.
//!   Static: action order and preconditions are not followed. The finding names the witness
//!   state.
//! - `property-failed`: a property whose subject has two statuses in two states that agree on every
//!   dimension except the evidence dimensions its `independent_of` claim reads (directly or through
//!   claims). The counterexample is that pair of states.
//!
//! A witness state is the one of least weight (present records, decided capabilities and taken
//! decisions; a record weighs one more for each upstream artifact it was observed before the move
//! of, so a witness prefers current records), ties broken by its rendering in code-point order. The
//! counterexample's first state is the witness among the states that have such a partner, and its
//! second the witness among that state's partners. A state is written `{}`, or as its present
//! classes, decided capabilities and taken decisions in dimension order: `` {evidence `k` result
//! `r`, evidence `k2`, evidence `k3` about `b` result `r`, capability `c` granted, decision `d`
//! taken} ``. A record observed before
//! an upstream artifact moved is written `` evidence `k` observed before upstream `a` moved ``, and
//! a dimension with upstream artifacts is named with them in the bound refusal (`` evidence kind
//! `k` with upstream `a` 16 ``). The numbers the refusal lists multiply to the state count. A bound
//! dimension is written with its subject, in a state and in the bound refusal (`` evidence kind
//! `k3` about `b` 4 ``); an unbound one without, and stands for records about an artifact no match
//! of its kind names.
//!
//! # Refusals
//!
//! First, an IR a caller builds is held to the evaluator's depth bound, before anything walks its
//! predicates: one nested beyond `eval::MAX_IR_DEPTH`, itself or along its claim references, is
//! refused as `predicate-too-deep`, and claims that test each other in a cycle as `claim-cycle`
//! (the module docs of `eval`, "Depth"). Then, before the bound, the properties are checked
//! against the protocol, as `properties.rs` says: `unsupported-format`, `protocol-mismatch`, `invalid-identifier`,
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

pub(crate) use space::{REVISION, Space, State};

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
pub(crate) struct Evaluated {
    pub(crate) actions: Vec<String>,
    pub(crate) outcomes: Vec<String>,
}

/// Checks `ir` over its whole state space, and each of `properties` when given, as the module docs
/// say. Pure and deterministic.
pub fn check(ir: &Ir, properties: Option<&Properties>) -> Result<Report, Refusal> {
    eval::check_depth(ir).map_err(|refusal| Refusal::new(refusal.code(), refusal.to_string()))?;
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
pub(crate) fn evaluate(ir: &Ir, space: &Space<'_>, state: &State) -> Result<Evaluated, Refusal> {
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
pub(crate) fn witness(
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
    let legitimate = |state: &State, at: usize| {
        let index = usize::try_from(space.index(state)).expect("the space is within the bound");
        evaluated[index].outcomes[at] == "legitimate"
    };
    let mut findings = Vec::new();
    for (at, (id, outcome)) in ir.outcomes.iter().enumerate() {
        let Some(requirement) = outcome.requires.predicate() else {
            continue;
        };
        // The evidence dimensions the requirement reads where a record helps it hold. It is
        // meant to rest on authority only when one of them is of a governed kind.
        let positive = space.dimensions_read_positively(ir, requirement);
        if !positive.values().any(|kind| governed.contains(kind)) {
            continue;
        }
        // Legitimate with no authority decision and only evidence no authority is needed for, and
        // because of evidence present there: taking away one positively read dimension that is
        // present blocks it. A state where it holds only on absent evidence is no witness.
        let candidates = (0..states.len()).filter(|index| {
            let state = &states[*index];
            let without_authority = !space.decides_authority(state)
                && space.present(state).iter().all(|kind| free.contains(kind));
            without_authority
                && legitimate(state, at)
                && positive.keys().any(|dimension| {
                    state[*dimension] != 0
                        && !legitimate(&space.erase(state, &BTreeSet::from([*dimension])), at)
                })
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
        let erased: BTreeSet<usize> = space.dimensions_read(ir, true_when).into_keys().collect();
        // States that agree outside the erased dimensions share a group; a state has a partner exactly
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
