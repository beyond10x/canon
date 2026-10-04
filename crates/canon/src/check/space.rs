//! The finite state space of a compiled protocol: its dimensions, their values, the evaluation
//! inputs each state stands for, and how a state is written in a report.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

use super::Reader;
use crate::eval::reads;
use crate::ir::Ir;
use crate::model::{
    ArtifactId, CapabilityId, ClaimId, DecisionName, EVIDENCE_FORMAT, EvidenceId, EvidenceKindId,
    EvidenceMatch, EvidenceRecord, OutcomeId, OutcomeRequirement, Predicate, Revision, Truth,
    one_line,
};

/// The revision every artifact and the case itself have in a checked state, and the case revision
/// every explicit decision is taken at.
pub(super) const REVISION: &str = "r";

/// Who takes an explicit decision in a checked state.
const PRINCIPAL: &str = "canon-check";

/// One evidence dimension: the records of one kind about one artifact. Its classes are each
/// result a match that reads such a record matches, in code-point order, then `None`, a record
/// with no result, which stands for every other result. A value is a set of classes, one bit each.
struct Kind<'a> {
    id: &'a EvidenceKindId,
    /// A record of the dimension with no result: every record of it is this one, with its own id
    /// and class. `None` when the protocol declares no artifact, and then there is no class.
    record: Option<EvidenceRecord>,
    /// Whether some match of the kind names the record's artifact as its subject. A dimension that
    /// is not bound stands for records about an artifact no match of the kind names.
    bound: bool,
    classes: Vec<Option<&'a str>>,
}

impl Kind<'_> {
    /// `` `k` ``, or `` `k` about `b` `` for a bound dimension.
    fn name(&self) -> String {
        let id = one_line(self.id.as_str());
        match &self.record {
            Some(record) if self.bound => {
                format!("`{id}` about `{}`", one_line(record.subject.as_str()))
            }
            _ => format!("`{id}`"),
        }
    }

    /// Whether `matching` reads the records of this dimension, by the evaluator's own rule.
    fn read_by(&self, matching: &EvidenceMatch) -> bool {
        self.record
            .as_ref()
            .is_some_and(|record| reads(&matching.kind, matching.subject.as_ref(), record))
    }
}

/// The dimensions, in order: the evidence dimensions, each capability, each decision name.
pub(super) struct Space<'a> {
    kinds: Vec<Kind<'a>>,
    capabilities: Vec<&'a CapabilityId>,
    /// Each decision name and every outcome that requires it.
    decisions: Vec<(&'a DecisionName, Vec<&'a OutcomeId>)>,
}

/// One state: a value for each dimension, in dimension order.
pub(super) type State = Vec<u32>;

/// A capability's value: not decided, granted or denied.
const UNDECIDED: u32 = 0;
const GRANTED: u32 = 1;

impl<'a> Space<'a> {
    pub(super) fn new(ir: &'a Ir) -> Self {
        let matches: Vec<&EvidenceMatch> = readers(ir)
            .flat_map(|(_, predicate)| evidence_matches(predicate))
            .collect();
        let mut kinds = Vec::new();
        for id in ir.evidence_kinds.keys() {
            if ir.artifacts.is_empty() {
                kinds.push(Kind {
                    id,
                    record: None,
                    bound: false,
                    classes: Vec::new(),
                });
                continue;
            }
            let of_kind: Vec<&EvidenceMatch> = matches
                .iter()
                .copied()
                .filter(|matching| matching.kind == *id)
                .collect();
            let named: BTreeSet<&ArtifactId> = of_kind
                .iter()
                .filter_map(|matching| matching.subject.as_ref())
                .collect();
            let mut about: Vec<(&ArtifactId, bool)> = named.iter().map(|a| (*a, true)).collect();
            let unbound = named.is_empty() || of_kind.iter().any(|m| m.subject.is_none());
            if unbound && let Some(other) = ir.artifacts.keys().find(|a| !named.contains(a)) {
                about.push((other, false));
            }
            for (artifact, bound) in about {
                let mut kind = Kind {
                    id,
                    record: Some(EvidenceRecord {
                        format: EVIDENCE_FORMAT.to_owned(),
                        id: EvidenceId::new("e"),
                        kind: id.clone(),
                        result: None,
                        subject: artifact.clone(),
                        subject_revision: Revision::new(REVISION),
                        observed_at: None,
                    }),
                    bound,
                    classes: Vec::new(),
                };
                let results: BTreeSet<&str> = of_kind
                    .iter()
                    .filter(|matching| kind.read_by(matching))
                    .filter_map(|matching| matching.result.as_deref())
                    .collect();
                kind.classes = results.into_iter().map(Some).chain([None]).collect();
                kinds.push(kind);
            }
        }
        let capabilities: BTreeSet<&CapabilityId> = ir
            .actions
            .values()
            .flat_map(|action| &action.requires)
            .collect();
        let mut decisions: BTreeMap<&DecisionName, Vec<&OutcomeId>> = BTreeMap::new();
        for (outcome, declared) in &ir.outcomes {
            if let OutcomeRequirement::Decision(name) = &declared.requires {
                decisions.entry(name).or_default().push(outcome);
            }
        }
        Space {
            kinds,
            capabilities: capabilities.into_iter().collect(),
            decisions: decisions.into_iter().collect(),
        }
    }

    /// How many values each dimension has, in dimension order; `None` for an evidence kind with
    /// more values than a `u128` holds.
    fn radices(&self) -> Vec<Option<u128>> {
        let kinds = self
            .kinds
            .iter()
            .map(|kind| 1u128.checked_shl(kind.classes.len() as u32));
        let capabilities = self.capabilities.iter().map(|_| Some(3));
        let decisions = self.decisions.iter().map(|_| Some(2));
        kinds.chain(capabilities).chain(decisions).collect()
    }

    /// The number of states, or `None` when it does not fit a `u128`.
    pub(super) fn size(&self) -> Option<u128> {
        self.radices()
            .into_iter()
            .try_fold(1u128, |total, radix| total.checked_mul(radix?))
    }

    /// Each dimension and its number of values, in dimension order, as the bound refusal names
    /// them.
    pub(super) fn dimensions(&self) -> String {
        let kinds = self.kinds.iter().map(|kind| {
            let count = 1u128
                .checked_shl(kind.classes.len() as u32)
                .map_or_else(|| format!("2^{}", kind.classes.len()), |n| n.to_string());
            format!("evidence kind {} {count}", kind.name())
        });
        let capabilities = self
            .capabilities
            .iter()
            .map(|id| format!("capability `{}` 3", one_line(id.as_str())));
        let decisions = self
            .decisions
            .iter()
            .map(|(name, _)| format!("decision `{}` 2", one_line(name.as_str())));
        kinds
            .chain(capabilities)
            .chain(decisions)
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// The state at `index`, the first dimension varying fastest. Called only once the size is
    /// known to fit.
    pub(super) fn state(&self, mut index: u128) -> State {
        self.radices()
            .into_iter()
            .map(|radix| {
                let radix = radix.expect("the size fits, so every radix does");
                let value = index % radix;
                index /= radix;
                u32::try_from(value).expect("a value within the bound fits u32")
            })
            .collect()
    }

    /// The index of `state`: the inverse of [`Space::state`].
    pub(super) fn index(&self, state: &State) -> u128 {
        self.radices()
            .into_iter()
            .zip(state)
            .rev()
            .fold(0, |index, (radix, value)| {
                index * radix.expect("the size fits, so every radix does") + u128::from(*value)
            })
    }

    /// The positions of the evidence dimensions `predicate` reads, directly or through the claims
    /// it tests, each with its kind: those whose records an evidence match it reaches reads.
    pub(super) fn dimensions_read(
        &self,
        ir: &Ir,
        predicate: &Predicate,
    ) -> BTreeMap<usize, &'a EvidenceKindId> {
        self.dimensions_of(&matches_read(ir, predicate))
    }

    /// The positions of the evidence dimensions `predicate` reads positively, each with its kind:
    /// those whose records an evidence match [`positive_matches`] finds reads.
    pub(super) fn dimensions_read_positively(
        &self,
        ir: &Ir,
        predicate: &Predicate,
    ) -> BTreeMap<usize, &'a EvidenceKindId> {
        self.dimensions_of(&positive_matches(ir, predicate))
    }

    /// The positions of the evidence dimensions one of `matches` reads, each with its kind.
    fn dimensions_of(&self, matches: &[&EvidenceMatch]) -> BTreeMap<usize, &'a EvidenceKindId> {
        self.kinds
            .iter()
            .enumerate()
            .filter(|(_, kind)| matches.iter().any(|matching| kind.read_by(matching)))
            .map(|(at, kind)| (at, kind.id))
            .collect()
    }

    /// Whether `state` takes any authority decision: some capability granted or denied.
    pub(super) fn decides_authority(&self, state: &State) -> bool {
        state[self.kinds.len()..self.kinds.len() + self.capabilities.len()]
            .iter()
            .any(|value| *value != UNDECIDED)
    }

    /// The evidence kinds present in `state`.
    pub(super) fn present(&self, state: &State) -> BTreeSet<&'a EvidenceKindId> {
        self.kinds
            .iter()
            .zip(state)
            .filter(|(_, value)| **value != 0)
            .map(|(kind, _)| kind.id)
            .collect()
    }

    /// How far `state` is from the empty state: present classes, decided capabilities and taken
    /// decisions. A witness is the state of least weight, ties broken by its rendering.
    pub(super) fn weight(&self, state: &State) -> u32 {
        let kinds = self.kinds.len();
        let capabilities = self.capabilities.len();
        state
            .iter()
            .enumerate()
            .map(|(at, value)| {
                if at < kinds {
                    value.count_ones()
                } else if at < kinds + capabilities {
                    u32::from(*value != UNDECIDED)
                } else {
                    *value
                }
            })
            .sum()
    }

    /// The state as a report writes it: `{}`, or each present class, decided capability and taken
    /// decision in dimension order.
    pub(super) fn render(&self, state: &State) -> String {
        let mut items = Vec::new();
        for (kind, value) in self.kinds.iter().zip(state) {
            for (bit, class) in kind.classes.iter().enumerate() {
                if value & (1 << bit) == 0 {
                    continue;
                }
                let name = kind.name();
                items.push(match class {
                    Some(result) => format!("evidence {name} result `{}`", one_line(result)),
                    None => format!("evidence {name}"),
                });
            }
        }
        let capabilities = &state[self.kinds.len()..];
        for (id, value) in self.capabilities.iter().zip(capabilities) {
            if *value != UNDECIDED {
                let decision = if *value == GRANTED {
                    "granted"
                } else {
                    "denied"
                };
                items.push(format!("capability `{}` {decision}", one_line(id.as_str())));
            }
        }
        let decisions = &state[self.kinds.len() + self.capabilities.len()..];
        for ((name, _), value) in self.decisions.iter().zip(decisions) {
            if *value != 0 {
                items.push(format!("decision `{}` taken", one_line(name.as_str())));
            }
        }
        format!("{{{}}}", items.join(", "))
    }

    /// The evidence records `state` stands for: one per present class, about its dimension's
    /// artifact at [`REVISION`], with no observation instant, so none expires.
    pub(super) fn evidence(&self, state: &State) -> Vec<EvidenceRecord> {
        let mut records = Vec::new();
        for (kind, value) in self.kinds.iter().zip(state) {
            let Some(record) = &kind.record else {
                continue;
            };
            for (bit, class) in kind.classes.iter().enumerate() {
                if value & (1 << bit) != 0 {
                    records.push(EvidenceRecord {
                        id: EvidenceId::new(format!("e{}", records.len())),
                        result: class.map(str::to_owned),
                        ..record.clone()
                    });
                }
            }
        }
        records
    }

    /// The `canon-authority/1` document `state` stands for, as JSON: each decided capability.
    pub(super) fn authority(&self, state: &State) -> String {
        let values = &state[self.kinds.len()..];
        let decided: Vec<Value> = self
            .capabilities
            .iter()
            .zip(values)
            .filter(|(_, value)| **value != UNDECIDED)
            .map(|(id, value)| {
                let decision = if *value == GRANTED {
                    "granted"
                } else {
                    "denied"
                };
                json!({"capability": id.as_str(), "decision": decision})
            })
            .collect();
        Value::Array(decided).to_string()
    }

    /// The `canon-decisions/1` document `state` stands for, as JSON: each taken decision, for
    /// every outcome that requires it, at [`REVISION`].
    pub(super) fn decisions(&self, state: &State) -> String {
        let values = &state[self.kinds.len() + self.capabilities.len()..];
        let taken: Vec<Value> = self
            .decisions
            .iter()
            .zip(values)
            .filter(|(_, value)| **value != 0)
            .flat_map(|((name, outcomes), _)| {
                outcomes.iter().map(move |outcome| {
                    json!({
                        "decision": name.as_str(),
                        "outcome": outcome.as_str(),
                        "principal": PRINCIPAL,
                        "case_revision": REVISION,
                    })
                })
            })
            .collect();
        Value::Array(taken).to_string()
    }

    /// `state` with every dimension of the kinds in `erased` set to absent: the states that agree
    /// on everything else share it.
    pub(super) fn erase(&self, state: &State, erased: &BTreeSet<usize>) -> State {
        state
            .iter()
            .enumerate()
            .map(|(at, value)| if erased.contains(&at) { 0 } else { *value })
            .collect()
    }
}

/// Every predicate the protocol declares, with its declaration: claims, obligations, action
/// preconditions and outcome requirements, each section in identifier order. The one list both the
/// classes and the `unproduced-evidence` findings read, so no reader is in one and not the other.
pub(super) fn readers(ir: &Ir) -> impl Iterator<Item = (Reader, &Predicate)> {
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
        let requirement = outcome.requires.predicate()?;
        Some((Reader::Outcome(id.clone()), requirement))
    });
    claims.chain(obligations).chain(actions).chain(outcomes)
}

/// The evidence matches in `predicate` itself, in source order.
fn evidence_matches(predicate: &Predicate) -> Vec<&EvidenceMatch> {
    let mut matches = Vec::new();
    predicate.visit(&mut |node| {
        if let Predicate::Evidence(matching) = node {
            matches.push(matching);
        }
    });
    matches
}

/// Every evidence match `predicate` reaches: its own, and those of every claim it tests, through
/// any number of claim references, each claim visited once.
fn matches_read<'a>(ir: &'a Ir, predicate: &'a Predicate) -> Vec<&'a EvidenceMatch> {
    let mut matches = Vec::new();
    let mut visited: BTreeSet<&ClaimId> = BTreeSet::new();
    let mut pending = vec![predicate];
    while let Some(next) = pending.pop() {
        next.visit(&mut |node| match node {
            Predicate::Evidence(matching) => matches.push(matching),
            Predicate::Claim(test) => {
                if visited.insert(&test.claim)
                    && let Some(claim) = ir.claims.get(&test.claim)
                {
                    pending.push(&claim.true_when);
                }
            }
            _ => {}
        });
    }
    matches
}

/// Every evidence kind `predicate` reads: the kinds of every evidence match it reaches.
pub(super) fn kinds_read<'a>(ir: &'a Ir, predicate: &'a Predicate) -> BTreeSet<&'a EvidenceKindId> {
    matches_read(ir, predicate)
        .into_iter()
        .map(|matching| &matching.kind)
        .collect()
}

/// Every evidence match `predicate` reaches in a positive position, where a record it reads can
/// help the predicate hold: its own and those of every claim it tests, through any number of claim
/// references. `not` flips the position, and so does a claim test `is: false` or `is: unknown`,
/// each of which holds on what the claim's evidence does not establish; `all`, `any` and
/// `is: true` keep it. Each claim is visited once per position.
pub(super) fn positive_matches<'a>(ir: &'a Ir, predicate: &'a Predicate) -> Vec<&'a EvidenceMatch> {
    let mut matches = Vec::new();
    let mut visited: BTreeSet<(&ClaimId, bool)> = BTreeSet::new();
    let mut pending = vec![(predicate, true)];
    while let Some((next, positive)) = pending.pop() {
        match next {
            Predicate::All(members) | Predicate::Any(members) => {
                pending.extend(members.iter().map(|member| (member, positive)));
            }
            Predicate::Not(inner) => pending.push((inner, !positive)),
            Predicate::Evidence(matching) => {
                if positive {
                    matches.push(matching);
                }
            }
            Predicate::Claim(test) => {
                let position = match test.is {
                    Truth::True => positive,
                    Truth::False | Truth::Unknown => !positive,
                };
                if visited.insert((&test.claim, position))
                    && let Some(claim) = ir.claims.get(&test.claim)
                {
                    pending.push((&claim.true_when, position));
                }
            }
        }
    }
    matches
}
