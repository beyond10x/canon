//! The finite state space of a compiled protocol: its dimensions, their values, the evaluation
//! inputs each state stands for, and how a state is written in a report.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

use super::Reader;
use crate::eval::{invalidated_claims, reads};
use crate::ir::Ir;
use crate::model::{
    ArtifactId, CapabilityId, ClaimId, DecisionName, Declarations, EVIDENCE_FORMAT, EvidenceId,
    EvidenceKindId, EvidenceMatch, EvidenceRecord, InputId, InputValue, Instant, OutcomeId,
    OutcomeRequirement, Predicate, Revision, Truth, one_line,
};

/// The revision every artifact and the case itself have in a checked state, and the case revision
/// every explicit decision is taken at.
pub(crate) const REVISION: &str = "r";

/// Who takes an explicit decision in a checked state.
const PRINCIPAL: &str = "canon-check";

/// The revision of an upstream artifact a record recorded when it was observed before that artifact
/// moved: an earlier one than [`REVISION`], the case's current revision of every artifact.
const EARLIER: &str = "r0";

/// What reads the records of an evidence dimension: an evidence match, or an order form for each kind
/// it reads. `result` is the result a match or a `first` form compares a record's with.
#[derive(Clone, Copy)]
struct Read<'a> {
    kind: &'a EvidenceKindId,
    subject: Option<&'a ArtifactId>,
    result: Option<&'a str>,
    /// Whether an order form reads: the dimension then has a history dimension.
    ordered: bool,
}

/// The history dimension's value that adds no record: the dimension's current records, if any, are
/// first observed at its rank's instant.
const CURRENT: u32 = 0;

/// The history dimension's value that leaves the order unknown: no record of the dimension has an
/// `observed_at`, and when it has no current record, one earlier record without a result stands in.
const UNPLACED: u32 = 1;

/// One evidence dimension: the records of one kind about one artifact. Its classes are each
/// result a match that reads such a record matches, in code-point order, then `None`, a record
/// with no result, which stands for every other result. Each class comes in one variant per
/// upstream vector: for each of the dimension's upstream artifacts, whether the record was observed
/// before it moved. A value is a set of (class, vector) variants, one bit each, class-major, so a
/// dimension without upstream artifacts has one bit per class as before, and records of one class
/// observed before and after a move can be present together.
struct Kind<'a> {
    id: &'a EvidenceKindId,
    /// A record of the dimension with no result: every record of it is this one, with its own id
    /// and class. `None` when the protocol declares no artifact, and then there is no class.
    record: Option<EvidenceRecord>,
    /// Whether some match of the kind names the record's artifact as its subject. A dimension that
    /// is not bound stands for records about an artifact no match of the kind names.
    bound: bool,
    classes: Vec<Option<&'a str>>,
    /// The upstream artifacts of the invalidation rules that can keep a record of this dimension
    /// from a claim, in identifier order: those whose rules invalidate a claim that reaches an
    /// evidence match reading it, in its own predicate or a claim it tests. Empty when no rule can.
    upstreams: Vec<&'a ArtifactId>,
    /// Whether an order form reads this dimension, which then has a history dimension.
    ordered: bool,
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

    /// How many upstream vectors each class has: `2^upstreams`.
    fn vectors(&self) -> Option<u32> {
        1u32.checked_shl(self.upstreams.len() as u32)
    }

    /// How many bits a value has: one per class and upstream vector. `None` when it does not fit.
    fn bits(&self) -> Option<u32> {
        (self.classes.len() as u32).checked_mul(self.vectors()?)
    }

    /// The class and upstream vector of bit `bit`.
    fn variant(&self, bit: u32) -> (Option<&str>, u32) {
        let vectors = self
            .vectors()
            .expect("a state exists only when the bits fit");
        (self.classes[(bit / vectors) as usize], bit % vectors)
    }

    /// `` `k` ``, or `` `k` about `b` ``, followed by `` with upstream `a`, `c` `` when the
    /// dimension has upstream artifacts: how the bound refusal names it.
    fn dimension_name(&self) -> String {
        if self.upstreams.is_empty() {
            return self.name();
        }
        let upstreams: Vec<String> = self
            .upstreams
            .iter()
            .map(|artifact| format!("`{}`", one_line(artifact.as_str())))
            .collect();
        format!("{} with upstream {}", self.name(), upstreams.join(", "))
    }

    /// Whether `matching` reads the records of this dimension, by the evaluator's own rule.
    fn read_by(&self, matching: &EvidenceMatch) -> bool {
        self.read_as(&matching.kind, matching.subject.as_ref())
    }

    /// Whether a match or an order form reading `kind`, about `subject` when it names one, reads the
    /// records of this dimension, by the evaluator's own rule.
    fn read_as(&self, kind: &EvidenceKindId, subject: Option<&ArtifactId>) -> bool {
        self.record
            .as_ref()
            .is_some_and(|record| reads(kind, subject, record))
    }

    /// How many values its history dimension has: [`CURRENT`], [`UNPLACED`], and each non-empty
    /// set of classes observed earlier. `None` when it does not fit.
    fn histories(&self) -> Option<u128> {
        1u128
            .checked_shl(u32::try_from(self.classes.len()).ok()?)?
            .checked_add(1)
    }
}

/// The dimensions, in order: the evidence dimensions, each capability, each decision name, each
/// case input, then, only when an order form reads evidence, the history dimension of each
/// evidence dimension an order form reads and, when there are two or more of those, the order
/// dimension.
pub(crate) struct Space<'a> {
    kinds: Vec<Kind<'a>>,
    capabilities: Vec<&'a CapabilityId>,
    /// Each decision name and every outcome that requires it.
    decisions: Vec<(&'a DecisionName, Vec<&'a OutcomeId>)>,
    /// Each case input and the values it may take, in identifier order: its value in a state is
    /// the index of one of them.
    inputs: Vec<(&'a InputId, &'a [InputValue])>,
    /// The position in `kinds` of each evidence dimension an order form reads, in dimension order:
    /// each has a history dimension.
    ordered: Vec<usize>,
    /// How many weak orders of the `ordered` dimensions there are, when there are two or more: the
    /// values of the order dimension. `None` when it does not fit.
    orders: Option<u128>,
    /// Each of those weak orders, as the rank of each `ordered` dimension, in lexicographic order;
    /// listed only when the space is within the bound, empty otherwise.
    rankings: Vec<Vec<u32>>,
}

/// One state: a value for each dimension, in dimension order.
pub(crate) type State = Vec<u32>;

/// A capability's value: not decided, granted or denied.
const UNDECIDED: u32 = 0;
const GRANTED: u32 = 1;

impl<'a> Space<'a> {
    pub(crate) fn new(ir: &'a Ir) -> Self {
        let matches: Vec<Read<'a>> = readers(ir)
            .flat_map(|(_, predicate)| reads_in(predicate))
            .collect();
        let mut kinds = Vec::new();
        for id in ir.evidence_kinds.keys() {
            if ir.artifacts.is_empty() {
                kinds.push(Kind {
                    id,
                    record: None,
                    bound: false,
                    classes: Vec::new(),
                    upstreams: Vec::new(),
                    ordered: false,
                });
                continue;
            }
            let of_kind: Vec<Read<'a>> = matches
                .iter()
                .copied()
                .filter(|matching| matching.kind == id)
                .collect();
            let named: BTreeSet<&ArtifactId> = of_kind
                .iter()
                .filter_map(|matching| matching.subject)
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
                        upstream_revisions: Declarations::default(),
                    }),
                    bound,
                    classes: Vec::new(),
                    upstreams: Vec::new(),
                    ordered: false,
                };
                let reading: Vec<&Read<'a>> = of_kind
                    .iter()
                    .filter(|matching| kind.read_as(matching.kind, matching.subject))
                    .collect();
                let results: BTreeSet<&str> = reading
                    .iter()
                    .filter_map(|matching| matching.result)
                    .collect();
                kind.classes = results.into_iter().map(Some).chain([None]).collect();
                kind.ordered = reading.iter().any(|matching| matching.ordered);
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
        for rule in ir.invalidation.values() {
            for claim in invalidated_claims(ir, &rule.invalidates) {
                let Some(declared) = ir.claims.get(&claim) else {
                    continue;
                };
                for matching in matches_read(ir, &declared.true_when) {
                    for kind in kinds.iter_mut() {
                        if kind.read_by(matching) && !kind.upstreams.contains(&&rule.upstream) {
                            kind.upstreams.push(&rule.upstream);
                        }
                    }
                }
            }
        }
        for kind in &mut kinds {
            kind.upstreams.sort();
        }
        let ordered: Vec<usize> = kinds
            .iter()
            .enumerate()
            .filter(|(_, kind)| kind.ordered)
            .map(|(at, _)| at)
            .collect();
        let orders = (ordered.len() >= 2).then(|| weak_orders(ordered.len()));
        let mut space = Space {
            kinds,
            capabilities: capabilities.into_iter().collect(),
            decisions: decisions.into_iter().collect(),
            inputs: ir
                .inputs
                .iter()
                .map(|(id, input)| (id, input.values.as_slice()))
                .collect(),
            ordered,
            orders: orders.flatten(),
            rankings: Vec::new(),
        };
        if orders.is_some() && space.size().is_some_and(|size| size <= super::STATE_BOUND) {
            space.rankings = rankings(space.ordered.len());
        }
        space
    }

    /// How many values each dimension has, in dimension order; `None` for an evidence kind with
    /// more values than a `u128` holds.
    fn radices(&self) -> Vec<Option<u128>> {
        let kinds = self
            .kinds
            .iter()
            .map(|kind| 1u128.checked_shl(kind.bits()?));
        let capabilities = self.capabilities.iter().map(|_| Some(3));
        let decisions = self.decisions.iter().map(|_| Some(2));
        let inputs = self
            .inputs
            .iter()
            .map(|(_, values)| u128::try_from(values.len()).ok());
        let histories = self.ordered.iter().map(|at| self.kinds[*at].histories());
        let orders = (self.ordered.len() >= 2).then_some(self.orders);
        kinds
            .chain(capabilities)
            .chain(decisions)
            .chain(inputs)
            .chain(histories)
            .chain(orders)
            .collect()
    }

    /// The position of the first history dimension.
    fn first_history(&self) -> usize {
        self.first_input() + self.inputs.len()
    }

    /// The value of the history dimension of the evidence dimension at `at`, which an order form
    /// reads.
    fn history(&self, state: &State, at: usize) -> Option<u32> {
        let index = self.ordered.iter().position(|ordered| *ordered == at)?;
        Some(state[self.first_history() + index])
    }

    /// The rank of each `ordered` dimension in `state`: all `0` when there is only one.
    fn ranks(&self, state: &State) -> Vec<u32> {
        if self.ordered.len() < 2 {
            return vec![0; self.ordered.len()];
        }
        let at = self.first_history() + self.ordered.len();
        self.rankings[state[at] as usize].clone()
    }

    /// Whether the evidence dimension at `at` holds a record in `state`: a current one, or one its
    /// history dimension adds.
    pub(super) fn holds(&self, state: &State, at: usize) -> bool {
        state[at] != 0
            || self
                .history(state, at)
                .is_some_and(|value| value != CURRENT)
    }

    /// The evidence dimensions an order form reads whose first record has a known instant in
    /// `state`, each with its rank: those with a current record and no history, and those with an
    /// earlier record observed at an instant.
    fn placed(&self, state: &State) -> Vec<(usize, u32)> {
        let ranks = self.ranks(state);
        self.ordered
            .iter()
            .zip(ranks)
            .filter(|(at, _)| match self.history(state, **at) {
                Some(CURRENT) => state[**at] != 0,
                Some(UNPLACED) | None => false,
                Some(_) => true,
            })
            .map(|(at, rank)| (*at, rank))
            .collect()
    }

    /// The position of the first case input dimension.
    fn first_input(&self) -> usize {
        self.kinds.len() + self.capabilities.len() + self.decisions.len()
    }

    /// The value of each case input `state` stands for, as a case snapshot gives it.
    pub(crate) fn inputs(&self, state: &State) -> Declarations<InputId, InputValue> {
        Declarations::new(
            self.inputs
                .iter()
                .zip(&state[self.first_input()..])
                .map(|((id, values), value)| ((*id).clone(), values[*value as usize].clone()))
                .collect(),
        )
    }

    /// The number of states, or `None` when it does not fit a `u128`.
    pub(crate) fn size(&self) -> Option<u128> {
        self.radices()
            .into_iter()
            .try_fold(1u128, |total, radix| total.checked_mul(radix?))
    }

    /// Each dimension and its number of values, in dimension order, as the bound refusal names
    /// them.
    pub(super) fn dimensions(&self) -> String {
        let kinds = self.kinds.iter().map(|kind| {
            let count = match kind.bits() {
                Some(bits) => 1u128
                    .checked_shl(bits)
                    .map_or_else(|| format!("2^{bits}"), |n| n.to_string()),
                None => format!("2^({} * 2^{})", kind.classes.len(), kind.upstreams.len()),
            };
            format!("evidence kind {} {count}", kind.dimension_name())
        });
        let capabilities = self
            .capabilities
            .iter()
            .map(|id| format!("capability `{}` 3", one_line(id.as_str())));
        let decisions = self
            .decisions
            .iter()
            .map(|(name, _)| format!("decision `{}` 2", one_line(name.as_str())));
        let inputs = self
            .inputs
            .iter()
            .map(|(id, values)| format!("input `{}` {}", one_line(id.as_str()), values.len()));
        let histories = self.ordered.iter().map(|at| {
            let kind = &self.kinds[*at];
            let count = kind.histories().map_or_else(
                || format!("2^{} + 1", kind.classes.len()),
                |n| n.to_string(),
            );
            format!("history of evidence kind {} {count}", kind.name())
        });
        let orders = (self.ordered.len() >= 2).then(|| {
            let count = self
                .orders
                .map_or_else(|| format!("more than {}", u128::MAX), |n| n.to_string());
            format!("order of first observations {count}")
        });
        kinds
            .chain(capabilities)
            .chain(decisions)
            .chain(inputs)
            .chain(histories)
            .chain(orders)
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// The state at `index`, the first dimension varying fastest. Called only once the size is
    /// known to fit.
    pub(crate) fn state(&self, mut index: u128) -> State {
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
    /// it tests, each with its kind: those whose records an evidence match or an order form it
    /// reaches reads.
    pub(super) fn dimensions_read(
        &self,
        ir: &Ir,
        predicate: &Predicate,
    ) -> BTreeMap<usize, &'a EvidenceKindId> {
        self.dimensions_of(&evidence_reached(ir, predicate))
    }

    /// The positions of the evidence dimensions `predicate` reads positively, each with its kind:
    /// those whose records an evidence match or order form [`positive_reads`] finds reads.
    pub(super) fn dimensions_read_positively(
        &self,
        ir: &Ir,
        predicate: &Predicate,
    ) -> BTreeMap<usize, &'a EvidenceKindId> {
        self.dimensions_of(&positive_reads(ir, predicate))
    }

    /// The positions of the evidence dimensions one of `reached` reads, each with its kind.
    fn dimensions_of(&self, reached: &[Reached<'_>]) -> BTreeMap<usize, &'a EvidenceKindId> {
        self.kinds
            .iter()
            .enumerate()
            .filter(|(_, kind)| {
                reached
                    .iter()
                    .any(|(id, subject)| kind.read_as(id, *subject))
            })
            .map(|(at, kind)| (at, kind.id))
            .collect()
    }

    /// Whether `state` takes any authority decision: some capability granted or denied.
    pub(super) fn decides_authority(&self, state: &State) -> bool {
        state[self.kinds.len()..self.kinds.len() + self.capabilities.len()]
            .iter()
            .any(|value| *value != UNDECIDED)
    }

    /// The evidence kinds present in `state`: a record of the kind, current or earlier, is in it.
    pub(super) fn present(&self, state: &State) -> BTreeSet<&'a EvidenceKindId> {
        self.kinds
            .iter()
            .enumerate()
            .filter(|(at, _)| self.holds(state, *at))
            .map(|(_, kind)| kind.id)
            .collect()
    }

    /// How far `state` is from the empty state: present records, decided capabilities and taken
    /// decisions. A record weighs one, and one more for each upstream artifact it was observed
    /// before the move of, so a witness prefers current records. A case input weighs nothing:
    /// every state gives each input a value. An earlier record a history dimension adds weighs one,
    /// and so does leaving a dimension's order unknown; the order of first observations weighs
    /// nothing. A witness is the state of least weight, ties broken by its rendering.
    pub(super) fn weight(&self, state: &State) -> u32 {
        let kinds = self.kinds.len();
        let capabilities = self.capabilities.len();
        let first_input = self.first_input();
        let first_history = self.first_history();
        state
            .iter()
            .enumerate()
            .map(|(at, value)| {
                if at >= first_history {
                    match *value {
                        CURRENT if at < first_history + self.ordered.len() => 0,
                        UNPLACED if at < first_history + self.ordered.len() => 1,
                        earlier if at < first_history + self.ordered.len() => {
                            (earlier - UNPLACED).count_ones()
                        }
                        _ => 0,
                    }
                } else if at < kinds {
                    let kind = &self.kinds[at];
                    (0..kind.bits().unwrap_or(0))
                        .filter(|bit| value & (1 << bit) != 0)
                        .map(|bit| 1 + kind.variant(bit).1.count_ones())
                        .sum()
                } else if at < kinds + capabilities {
                    u32::from(*value != UNDECIDED)
                } else if at < first_input {
                    *value
                } else {
                    0
                }
            })
            .sum()
    }

    /// The state as a report writes it: `{}`, or each present class, decided capability and taken
    /// decision in dimension order. A record observed before an upstream artifact moved is written
    /// `` evidence `k` observed before upstream `a` moved ``.
    pub(super) fn render(&self, state: &State) -> String {
        let mut items = Vec::new();
        for (kind, value) in self.kinds.iter().zip(state) {
            for bit in 0..kind.bits().unwrap_or(0) {
                if value & (1 << bit) == 0 {
                    continue;
                }
                let (class, vector) = kind.variant(bit);
                let name = kind.name();
                let mut item = match class {
                    Some(result) => format!("evidence {name} result `{}`", one_line(result)),
                    None => format!("evidence {name}"),
                };
                let moved: Vec<String> = kind
                    .upstreams
                    .iter()
                    .enumerate()
                    .filter(|(at, _)| vector & (1 << at) != 0)
                    .map(|(_, artifact)| format!("`{}`", one_line(artifact.as_str())))
                    .collect();
                if !moved.is_empty() {
                    item.push_str(&format!(
                        " observed before upstream {} moved",
                        moved.join(", ")
                    ));
                }
                items.push(item);
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
        for (id, value) in self.inputs(state).iter() {
            items.push(format!(
                "input `{}` `{}`",
                one_line(id.as_str()),
                one_line(value.as_str())
            ));
        }
        for at in &self.ordered {
            let kind = &self.kinds[*at];
            let name = kind.name();
            match self.history(state, *at) {
                Some(UNPLACED) if state[*at] == 0 => {
                    items.push(format!(
                        "evidence {name} earlier, observed at no known instant"
                    ));
                }
                Some(UNPLACED) => {
                    items.push(format!("evidence {name} observed at no known instant"))
                }
                Some(CURRENT) | None => {}
                Some(earlier) => {
                    for (bit, class) in kind.classes.iter().enumerate() {
                        if (earlier - UNPLACED) & (1 << bit) == 0 {
                            continue;
                        }
                        items.push(match class {
                            Some(result) => {
                                format!("evidence {name} result `{}` earlier", one_line(result))
                            }
                            None => format!("evidence {name} earlier"),
                        });
                    }
                }
            }
        }
        let mut placed = self.placed(state);
        if placed.len() >= 2 {
            placed.sort_by_key(|(at, rank)| (*rank, *at));
            let mut order = format!("order {}", self.kinds[placed[0].0].name());
            for pair in placed.windows(2) {
                let tie = if pair[0].1 == pair[1].1 { "=" } else { "<" };
                order.push_str(&format!(" {tie} {}", self.kinds[pair[1].0].name()));
            }
            items.push(order);
        }
        format!("{{{}}}", items.join(", "))
    }

    /// The evidence records `state` stands for: one per present class and upstream vector, each
    /// with its own id, about its dimension's artifact at [`REVISION`], with no observation
    /// instant, so none expires. A record of a dimension with upstream artifacts records a revision
    /// of each: [`EARLIER`] for one its vector marks moved, [`REVISION`] otherwise. The evaluator's own
    /// invalidation stage then decides which claims it is kept from.
    ///
    /// A dimension an order form reads is observed as its history dimension says, at instants the
    /// evaluator only compares, since no evaluation instant is given: with [`CURRENT`], its records
    /// at its rank's instant; with [`UNPLACED`], without `observed_at`, and with one earlier record
    /// without a result when it has no current record; otherwise one earlier record per class the
    /// value names, at its rank's instant, and its current records after every rank's instant. An
    /// earlier record is about the dimension's artifact at [`EARLIER`], so revision binding keeps
    /// it from every evidence match, and records no upstream revision. Earlier records come after
    /// the current ones, in dimension and then class order.
    pub(crate) fn evidence(&self, state: &State) -> Vec<EvidenceRecord> {
        let ranks = self.ranks(state);
        let late = u32::try_from(self.ordered.len()).expect("few ordered dimensions");
        let rank_of = |at: usize| {
            self.ordered
                .iter()
                .position(|ordered| *ordered == at)
                .map(|index| ranks[index])
        };
        let mut records = Vec::new();
        for (at, (kind, value)) in self.kinds.iter().zip(state).enumerate() {
            let observed_at = match (self.history(state, at), rank_of(at)) {
                (Some(CURRENT), Some(rank)) => Some(instant(rank)),
                (Some(UNPLACED), _) | (None, _) | (_, None) => None,
                (Some(_), Some(_)) => Some(instant(late)),
            };
            let Some(record) = &kind.record else {
                continue;
            };
            for bit in 0..kind.bits().unwrap_or(0) {
                if value & (1 << bit) == 0 {
                    continue;
                }
                let (class, vector) = kind.variant(bit);
                let upstream_revisions = Declarations::new(
                    kind.upstreams
                        .iter()
                        .enumerate()
                        .map(|(at, artifact)| {
                            let moved = vector & (1 << at) != 0;
                            let revision = if moved { EARLIER } else { REVISION };
                            ((*artifact).clone(), Revision::new(revision))
                        })
                        .collect(),
                );
                records.push(EvidenceRecord {
                    id: EvidenceId::new(format!("e{}", records.len())),
                    result: class.map(str::to_owned),
                    observed_at: observed_at.clone(),
                    upstream_revisions,
                    ..record.clone()
                });
            }
        }
        for at in &self.ordered {
            let kind = &self.kinds[*at];
            let Some(record) = &kind.record else {
                continue;
            };
            let earlier =
                |result: Option<&str>, observed_at: Option<Instant>, id: usize| EvidenceRecord {
                    id: EvidenceId::new(format!("e{id}")),
                    result: result.map(str::to_owned),
                    subject_revision: Revision::new(EARLIER),
                    observed_at,
                    ..record.clone()
                };
            match self.history(state, *at) {
                Some(CURRENT) | None => {}
                Some(UNPLACED) => {
                    if state[*at] == 0 {
                        records.push(earlier(None, None, records.len()));
                    }
                }
                Some(value) => {
                    let rank = rank_of(*at).expect("an ordered dimension has a rank");
                    for (bit, class) in kind.classes.iter().enumerate() {
                        if (value - UNPLACED) & (1 << bit) != 0 {
                            records.push(earlier(*class, Some(instant(rank)), records.len()));
                        }
                    }
                }
            }
        }
        records
    }

    /// The `canon-authority/1` document `state` stands for, as JSON: each decided capability.
    pub(crate) fn authority(&self, state: &State) -> String {
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
    pub(crate) fn decisions(&self, state: &State) -> String {
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
    /// on everything else share it. An erased evidence dimension an order form reads loses its
    /// history too, and stands first, alone, in the order of first observations, the others
    /// keeping their order behind it: it holds no record, so its rank decides nothing.
    pub(super) fn erase(&self, state: &State, erased: &BTreeSet<usize>) -> State {
        let first_history = self.first_history();
        let mut erased_state: State = state
            .iter()
            .enumerate()
            .map(|(at, value)| {
                let history_of = at
                    .checked_sub(first_history)
                    .and_then(|index| self.ordered.get(index));
                if erased.contains(&at) || history_of.is_some_and(|of| erased.contains(of)) {
                    0
                } else {
                    *value
                }
            })
            .collect();
        if self.ordered.len() >= 2 && self.ordered.iter().any(|at| erased.contains(at)) {
            let ranks = self.ranks(state);
            let kept: BTreeSet<u32> = self
                .ordered
                .iter()
                .zip(&ranks)
                .filter(|(at, _)| !erased.contains(at))
                .map(|(_, rank)| *rank)
                .collect();
            let normalized: Vec<u32> = self
                .ordered
                .iter()
                .zip(&ranks)
                .map(|(at, rank)| {
                    if erased.contains(at) {
                        0
                    } else {
                        1 + u32::try_from(kept.range(..rank).count()).expect("few ranks")
                    }
                })
                .collect();
            let index = self
                .rankings
                .binary_search(&normalized)
                .expect("a normalized ranking is a weak order");
            let at = first_history + self.ordered.len();
            erased_state[at] = u32::try_from(index).expect("within the bound");
        }
        erased_state
    }
}

/// The instant of rank `rank` in a checked state: whole minutes from `2000-01-01T00:00:00Z`.
fn instant(rank: u32) -> Instant {
    Instant::new(format!("2000-01-01T{:02}:{:02}:00Z", rank / 60, rank % 60))
}

/// How many weak orders (orderings with ties) `n` things have: the ordered Bell number, or `None`
/// when it does not fit.
fn weak_orders(n: usize) -> Option<u128> {
    let mut counts: Vec<u128> = vec![1];
    for m in 1..=n {
        let mut total: u128 = 0;
        let mut choose: u128 = 1;
        for k in 1..=m {
            choose = choose.checked_mul((m - k + 1) as u128)? / k as u128;
            total = total.checked_add(choose.checked_mul(counts[m - k])?)?;
        }
        counts.push(total);
    }
    counts.get(n).copied()
}

/// Every weak order of `n` things, as the rank of each, ranks `0` to `k - 1` all used, in
/// lexicographic order. Called only when their number is within the bound.
fn rankings(n: usize) -> Vec<Vec<u32>> {
    let n32 = u32::try_from(n).expect("few ordered dimensions");
    let total = (n as u32).pow(n32);
    (0..total)
        .map(|mut code| {
            let mut ranks = vec![0u32; n];
            for rank in ranks.iter_mut().rev() {
                *rank = code % n32;
                code /= n32;
            }
            ranks
        })
        .filter(|ranks| {
            let used: BTreeSet<u32> = ranks.iter().copied().collect();
            used.iter()
                .copied()
                .eq(0..u32::try_from(used.len()).expect("few ranks"))
        })
        .collect()
}

/// Every predicate the protocol declares, with its declaration: claims, obligations, action
/// preconditions and outcome requirements, each section in identifier order. The one list both the
/// classes and the `unproduced-evidence` findings read, so no reader is in one and not the other.
pub(super) fn readers(ir: &Ir) -> impl Iterator<Item = (Reader, &Predicate)> {
    let claims = ir
        .claims
        .iter()
        .map(|(id, claim)| (Reader::Claim(id.clone()), &claim.true_when));
    // An applicability predicate is not a reader: it tests only claims, which are readers
    // themselves, and case inputs, so it reads no evidence a claim does not.
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

/// What reads evidence in `predicate` itself, in source order: each evidence match, and each kind
/// an order form reads.
fn reads_in(predicate: &Predicate) -> Vec<Read<'_>> {
    let mut found = Vec::new();
    predicate.visit(&mut |node| match node {
        Predicate::Evidence(matching) => found.push(Read {
            kind: &matching.kind,
            subject: matching.subject.as_ref(),
            result: matching.result.as_deref(),
            ordered: false,
        }),
        Predicate::First(form) => found.push(Read {
            kind: &form.kind,
            subject: form.subject.as_ref(),
            result: Some(form.result.as_str()),
            ordered: true,
        }),
        Predicate::Before(_) => {
            for (kind, subject) in node.order_reads() {
                found.push(Read {
                    kind,
                    subject,
                    result: None,
                    ordered: true,
                });
            }
        }
        _ => {}
    });
    found
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

/// What reads evidence: a kind, and the artifact the reader names as its subject, if any.
type Reached<'a> = (&'a EvidenceKindId, Option<&'a ArtifactId>);

/// What `predicate` reads, through the claims it tests: each evidence match it reaches, and each
/// kind an order form it reaches reads.
fn evidence_reached<'a>(ir: &'a Ir, predicate: &'a Predicate) -> Vec<Reached<'a>> {
    let mut reached = Vec::new();
    let mut visited: BTreeSet<&ClaimId> = BTreeSet::new();
    let mut pending = vec![predicate];
    while let Some(next) = pending.pop() {
        next.visit(&mut |node| match node {
            Predicate::Evidence(matching) => {
                reached.push((&matching.kind, matching.subject.as_ref()));
            }
            Predicate::First(_) | Predicate::Before(_) => reached.extend(node.order_reads()),
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
    reached
}

/// Every evidence kind `predicate` reads: the kinds of every evidence match and order form it
/// reaches.
pub(super) fn kinds_read<'a>(ir: &'a Ir, predicate: &'a Predicate) -> BTreeSet<&'a EvidenceKindId> {
    evidence_reached(ir, predicate)
        .into_iter()
        .map(|(kind, _)| kind)
        .collect()
}

/// Every evidence match and order form `predicate` reaches in a positive position, where a record
/// it reads can help the predicate hold: its own and those of every claim it tests, through any
/// number of claim references. `not` flips the position, and so does a claim test `is: false` or
/// `is: unknown`, each of which holds on what the claim's evidence does not establish; `all`,
/// `any` and `is: true` keep it. Each claim is visited once per position. An order form in a
/// positive position counts for every kind it reads.
pub(super) fn positive_reads<'a>(ir: &'a Ir, predicate: &'a Predicate) -> Vec<Reached<'a>> {
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
                    matches.push((&matching.kind, matching.subject.as_ref()));
                }
            }
            Predicate::First(_) | Predicate::Before(_) => {
                if positive {
                    matches.extend(next.order_reads());
                }
            }
            Predicate::Input(_) => {}
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
