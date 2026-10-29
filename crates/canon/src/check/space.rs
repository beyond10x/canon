//! The finite state space of a compiled protocol: its dimensions, their values, the evaluation
//! inputs each state stands for, and how a state is written in a report.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

use super::Reader;
use crate::eval::{invalidated_claims, reads};
use crate::ir::Ir;
use crate::model::{
    ArtifactId, CASE_OUTCOME, CapabilityId, CaseId, ClaimId, DecisionName, Declarations,
    EVIDENCE_FORMAT, EvidenceId, EvidenceKindId, EvidenceMatch, EvidenceRecord,
    IndependenceDimensionId, InputId, InputValue, Instant, OPEN_OUTCOME, OutcomeId,
    OutcomeRequirement, Predicate, Principal, Revision, Truth, one_line,
};

/// The revision every artifact and the case itself have in a checked state, and the case revision
/// every explicit decision is taken at.
pub(crate) const REVISION: &str = "r";

/// Who takes an explicit decision in a checked state.
const PRINCIPAL: &str = "canon-check";

/// The revision of an upstream artifact a record recorded when it was observed before that artifact
/// moved: an earlier one than [`REVISION`], the case's current revision of every artifact.
const EARLIER: &str = "r0";

/// How the producer of a record in a checked state is named: this, a dash and the producer's
/// position among the space's producers, from 1.
const PRODUCER: &str = "producer";

/// The dimensions a producer is independent on in a checked state: exactly the dimensions some
/// evidence match demands. The empty set is a record without a producer, which no demand accepts,
/// and stands for every producer independent on no demanded dimension.
type Profile<'a> = BTreeSet<&'a IndependenceDimensionId>;

/// The records of one class and upstream vector a value of an evidence dimension holds: the
/// positions in [`Kind::profiles`] of their producers' profiles, an antichain under the subset
/// order, in position order. Empty is no record.
type Producers = Vec<usize>;

/// What reads the records of an evidence dimension: an evidence match, or an order form for each kind
/// it reads. `result` is the result a match or a `first` form compares a record's with.
#[derive(Clone, Copy)]
struct Read<'a> {
    kind: &'a EvidenceKindId,
    subject: Option<&'a ArtifactId>,
    result: Option<&'a str>,
    /// The independence an evidence match demands; `None` for a match that demands none and for an
    /// order form.
    independent_on: Option<&'a [IndependenceDimensionId]>,
    /// Whether an order form reads: the dimension then has a history dimension and a place in the
    /// order.
    ordered: bool,
    /// Whether a `children` form reads (story:case-composition): the dimension then has a history
    /// dimension, whose earlier records the form reads and no evidence match does, and no place
    /// in the order, since the form does not read when a record was observed.
    history: bool,
}

/// The history dimension's value that adds no record: the dimension's current records, if any, are
/// first observed at its rank's instant.
const CURRENT: u32 = 0;

/// The history dimension's value that leaves the order unknown: no record of the dimension has an
/// `observed_at`, and when it has no current record, one earlier record without a result stands in.
/// In a protocol with an `after` or a `since` form, the earlier record without a result and
/// without an `observed_at` stands in whatever the current records are, and they are observed at
/// the rank's instant.
const UNPLACED: u32 = 1;

/// A later-record dimension's value that adds no record.
const NO_LATER: u32 = 0;

/// A later-record dimension's value that adds one record of its result, about the dimension's
/// artifact at [`EARLIER`], observed at the instant of its own place in the order.
const LATER: u32 = 1;

/// A later-record dimension's value that adds one record of its result, about the dimension's
/// artifact at [`EARLIER`], with no `observed_at`.
const LATER_UNPLACED: u32 = 2;

/// A later-record dimension's value, when its result is one of the dimension's classes, that adds
/// no record and observes the dimension's current records of that class at its rank's instant,
/// with its first observations, in place of an earlier record of the class; its other current
/// records still come after them.
const CURRENT_FIRST: u32 = 3;

/// A last-record dimension's value that adds no record.
const NO_LAST: u32 = 0;

/// Where a last record is observed, relative to an item of the order (an ordered dimension's rank
/// or a later record's): at that item's instant, tied with it, or forty-five seconds after it,
/// after it and after every record thirty seconds past it, and before the next rank's instant.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Place {
    At(usize),
    Past(usize),
}

/// A record of a dimension a `since` reads as `last` (story:evidence-order-since), about the
/// dimension's artifact at [`EARLIER`], that a last-record dimension adds at one of its places: it
/// can be observed after the dimension's current records, which no other record of the dimension
/// is, so a latest record of the kind can be about another revision than the current one, and it
/// takes its own place in the order, tied with or just after its dimension's rank or a later
/// record its `since` compares, so that the records a `since` judges by can come before it, with
/// it or after it.
#[derive(Clone)]
struct Last<'a> {
    /// The position of the evidence dimension in `kinds`.
    at: usize,
    /// The results it can have: `None`, and, for a `since` whose `then` reads the same dimension,
    /// its `then` result too, so that latest records of the kind can be compared with each other.
    results: Vec<Option<&'a str>>,
    /// Where it can be observed: at and past its dimension's own item, and at and past the item of
    /// each later record a `since` anchored at the dimension compares.
    places: Vec<Place>,
}

impl<'a> Last<'a> {
    /// How many values its last-record dimension has: none, and each result at each place.
    fn radix(&self) -> usize {
        1 + self.results.len() * self.places.len()
    }

    /// The result and place of value `value` of its last-record dimension, `None` for no record:
    /// value `1 + r * places + p` is result `r` at place `p`.
    fn decode(&self, value: u32) -> Option<(Option<&'a str>, Place)> {
        let index = value.checked_sub(1)? as usize;
        let place = *self.places.get(index % self.places.len())?;
        let result = *self.results.get(index / self.places.len())?;
        Some((result, place))
    }
}

/// A result an `after` or a `since` form compares the records of an evidence dimension its `then`
/// reads with (story:evidence-order-after, story:evidence-order-since): each has a later-record
/// dimension and a place in the order.
#[derive(Clone, Copy)]
struct Later<'a> {
    /// The position of the evidence dimension in `kinds`.
    at: usize,
    result: &'a str,
    /// Whether the result is one of the dimension's classes: the dimension's current records can
    /// then have it, and the later-record dimension has the value [`CURRENT_FIRST`].
    class: bool,
}

impl Later<'_> {
    /// How many values its later-record dimension has.
    fn radix(&self) -> u32 {
        3 + u32::from(self.class)
    }
}

/// One evidence dimension: the records of one kind about one artifact. Its classes are each
/// result a match that reads such a record matches, in code-point order, then `None`, a record
/// with no result, which stands for every other result. Each class comes in one variant per
/// upstream vector: for each of the dimension's upstream artifacts, whether the record was observed
/// before it moved. A value gives each variant, class-major, then vector, one digit: the position
/// in [`Kind::values`] of the producers its records have, read as a number in base
/// `values.len()`, the first variant least significant. A dimension no demanding match reads has
/// the values no record and one record without a producer, one bit per variant as before
/// independence existed; records of one class observed before and after a move, or by producers
/// independent on different dimensions, can be present together.
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
    /// Whether an order form reads this dimension, which then has a history dimension and a place
    /// in the order.
    ordered: bool,
    /// Whether a `children` form reads this dimension, which then has a history dimension and,
    /// unless an order form reads it too, no place in the order.
    history: bool,
    /// The producer profiles of its records, in set order: each distinct demand of the evidence
    /// matches that read it, and the empty profile when a match that demands nothing or an order
    /// form reads it. Only the empty profile when no match that reads it demands independence.
    profiles: Vec<Profile<'a>>,
    /// The producers one variant's records can have, one per down-closed set of `profiles` (the
    /// demands those records meet), as the antichain of its greatest members, no record first.
    /// `None` when there are more than a state space within the bound could hold.
    values: Option<Vec<Producers>>,
}

impl<'a> Kind<'a> {
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

    /// How many variants a value gives a digit: one per class and upstream vector. `None` when it
    /// does not fit.
    fn variants(&self) -> Option<u32> {
        (self.classes.len() as u32).checked_mul(self.vectors()?)
    }

    /// How many values the dimension has: `values^variants`. `None` when it does not fit.
    fn radix(&self) -> Option<u128> {
        let values = u128::try_from(self.values.as_ref()?.len()).ok()?;
        values.checked_pow(self.variants()?)
    }

    /// The records `value` holds, each as its class, upstream vector and producer profile: by
    /// variant, class-major, then vector, then profile in set order. Called only once the size is
    /// known to fit.
    fn records(&self, value: u32) -> Vec<(Option<&'a str>, u32, &Profile<'a>)> {
        let values = self
            .values
            .as_ref()
            .expect("a state exists only when the values fit");
        let vectors = self
            .vectors()
            .expect("a state exists only when the values fit");
        let base = values.len() as u32;
        let mut rest = value;
        let mut records = Vec::new();
        for variant in 0..self.variants().unwrap_or(0) {
            let digit = rest % base;
            rest /= base;
            for profile in &values[digit as usize] {
                records.push((
                    self.classes[(variant / vectors) as usize],
                    variant % vectors,
                    &self.profiles[*profile],
                ));
            }
        }
        records
    }

    /// `` `k` ``, or `` `k` about `b` ``, followed by `` with upstream `a`, `c` `` when the
    /// dimension has upstream artifacts, and by `` with producers independent on [`d`], [`d`, `e`]
    /// `` (each profile but the empty one) when a match that reads it demands independence: how the
    /// bound refusal names it.
    fn dimension_name(&self) -> String {
        let mut name = self.name();
        if !self.upstreams.is_empty() {
            let upstreams: Vec<String> = self
                .upstreams
                .iter()
                .map(|artifact| format!("`{}`", one_line(artifact.as_str())))
                .collect();
            name.push_str(&format!(" with upstream {}", upstreams.join(", ")));
        }
        if self.profiles.iter().any(|profile| !profile.is_empty()) {
            let profiles: Vec<String> = self
                .profiles
                .iter()
                .filter(|profile| !profile.is_empty())
                .map(|profile| format!("[{}]", listed(profile)))
                .collect();
            name.push_str(&format!(
                " with producers independent on {}",
                profiles.join(", ")
            ));
        }
        name
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
/// evidence dimension an order form reads, then, only when a `children` form reads evidence, the
/// history dimension of each other evidence dimension it reads, then the later-record dimension
/// of each result an `after` or a `since` compares the records of a dimension with, the
/// last-record dimension of each evidence dimension a `since` reads as its `last` kind and, when
/// there are two or more of the histories an order form reads and later records together, the
/// order dimension. A `children` form adds no place in the order: its value does not depend on
/// when a record was observed.
pub(crate) struct Space<'a> {
    kinds: Vec<Kind<'a>>,
    capabilities: Vec<&'a CapabilityId>,
    /// Each decision name and every outcome that requires it.
    decisions: Vec<(&'a DecisionName, Vec<&'a OutcomeId>)>,
    /// Each case input and the values it may take, in identifier order: its value in a state is
    /// the index of one of them.
    inputs: Vec<(&'a InputId, &'a [InputValue])>,
    /// The position in `kinds` of each evidence dimension an order form reads, in dimension order:
    /// each has a history dimension and a place in the order.
    ordered: Vec<usize>,
    /// The position in `kinds` of each other evidence dimension a `children` form reads, in
    /// dimension order (story:case-composition): each has a history dimension, after those of the
    /// `ordered` dimensions, and no place in the order, so its records have no `observed_at`.
    children: Vec<usize>,
    /// Each result an `after` or a `since` compares the records of an evidence dimension with, in
    /// dimension and then result order: each has a later-record dimension and a place in the
    /// order, after the `ordered` dimensions. Empty when the protocol has neither, which then
    /// checks as it did before `after` existed.
    later: Vec<Later<'a>>,
    /// Each evidence dimension a `since` reads as `last`, in dimension order: each has a
    /// last-record dimension, after the later-record dimensions, whose record is observed at or
    /// just after one of its places in the order and is itself a thing in the order, after the
    /// `ordered` dimensions and the `later` records. Empty when the protocol has no `since`.
    last: Vec<Last<'a>>,
    /// How many weak orders of the `ordered` dimensions and the `later` records there are, when
    /// there are two or more: the values of the order dimension. `None` when it does not fit.
    orders: Option<u128>,
    /// Each of those weak orders, as the rank of each `ordered` dimension and then each `later`
    /// record, in lexicographic order; listed only when the space is within the bound, empty
    /// otherwise.
    rankings: Vec<Vec<u32>>,
    /// Every non-empty producer profile of every evidence dimension, in set order: the producer of
    /// a record of profile `P` is named [`PRODUCER`]`-n`, `n` the position of `P` here from 1, and
    /// the case lists it as independent on exactly `P`.
    producers: Vec<Profile<'a>>,
    /// The result a `case_outcome` record of the class without a result carries: `other`, primed
    /// until no predicate of the protocol names it, nor `open` (story:case-composition). A
    /// `case_outcome` record must give its child's outcome, and one no predicate names behaves as
    /// a record without a result towards each of them.
    unlisted: String,
}

/// One state: a value for each dimension, in dimension order.
pub(crate) type State = Vec<u32>;

/// A capability's value: not decided, granted or denied.
const UNDECIDED: u32 = 0;
const GRANTED: u32 = 1;

impl<'a> Space<'a> {
    pub(crate) fn new(ir: &'a Ir) -> Self {
        let matches: Vec<Read<'a>> = readers(ir)
            .flat_map(|(_, predicate)| reads_in(ir, predicate))
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
                    history: false,
                    profiles: vec![Profile::new()],
                    values: Some(vec![Vec::new(), vec![0]]),
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
                        producer: None,
                        id: EvidenceId::new("e"),
                        kind: id.clone(),
                        result: None,
                        subject: artifact.clone(),
                        subject_revision: Revision::new(REVISION),
                        observed_at: None,
                        upstream_revisions: Declarations::default(),
                        child_case: None,
                    }),
                    bound,
                    classes: Vec::new(),
                    upstreams: Vec::new(),
                    ordered: false,
                    history: false,
                    profiles: Vec::new(),
                    values: None,
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
                kind.history = reading.iter().any(|matching| matching.history);
                kind.profiles = profiles(reading.iter().map(|m| m.independent_on));
                kind.values = down_sets(&kind.profiles);
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
        let children: Vec<usize> = kinds
            .iter()
            .enumerate()
            .filter(|(_, kind)| kind.history && !kind.ordered)
            .map(|(at, _)| at)
            .collect();
        let mut compared: BTreeSet<(usize, &'a str)> = BTreeSet::new();
        // Each dimension a `since` reads as `last`, with the results its last record can have.
        let mut anchored: BTreeMap<usize, BTreeSet<Option<&'a str>>> = BTreeMap::new();
        // Each anchor dimension a `since` reads, with each dimension and result it compares.
        let mut pairs: BTreeSet<(usize, usize, &'a str)> = BTreeSet::new();
        for (_, predicate) in readers(ir) {
            for form in predicate.order_forms() {
                let (then, subject) = match form {
                    Predicate::After(form) => (&form.then, form.subject.as_ref()),
                    Predicate::Since(form) => {
                        for (at, kind) in kinds.iter().enumerate() {
                            if kind.read_as(&form.last, form.subject.as_ref()) {
                                let compared = kind.read_as(&form.then.kind, form.subject.as_ref());
                                let results = anchored.entry(at).or_default();
                                results.insert(None);
                                if compared {
                                    results.insert(Some(form.then.result.as_str()));
                                }
                                for (then_at, then_kind) in kinds.iter().enumerate() {
                                    if then_kind.read_as(&form.then.kind, form.subject.as_ref()) {
                                        pairs.insert((at, then_at, form.then.result.as_str()));
                                    }
                                }
                            }
                        }
                        (&form.then, form.subject.as_ref())
                    }
                    _ => continue,
                };
                for (at, kind) in kinds.iter().enumerate() {
                    if kind.read_as(&then.kind, subject) {
                        compared.insert((at, then.result.as_str()));
                    }
                }
            }
        }
        let later: Vec<Later<'a>> = compared
            .into_iter()
            .map(|(at, result)| Later {
                at,
                result,
                class: kinds[at].classes.contains(&Some(result)),
            })
            .collect();
        let item_of = |at: usize| ordered.iter().position(|o| *o == at);
        let later_item = |at: usize, result: &str| {
            later
                .iter()
                .position(|l| l.at == at && l.result == result)
                .map(|index| ordered.len() + index)
        };
        let last: Vec<Last<'a>> = anchored
            .into_iter()
            .map(|(at, results)| {
                let results: Vec<Option<&'a str>> = results.into_iter().collect();
                let mut items: BTreeSet<usize> = item_of(at).into_iter().collect();
                for (_, then_at, then_result) in pairs.iter().filter(|pair| pair.0 == at) {
                    items.extend(later_item(*then_at, then_result));
                }
                let places: Vec<Place> = items
                    .into_iter()
                    .flat_map(|item| [Place::At(item), Place::Past(item)])
                    .collect();
                Last {
                    at,
                    results,
                    places,
                }
            })
            .collect();
        let placeable = ordered.len() + later.len();
        let orders = (placeable >= 2).then(|| weak_orders(placeable));
        let producers: BTreeSet<Profile<'a>> = kinds
            .iter()
            .flat_map(|kind| kind.profiles.iter().filter(|p| !p.is_empty()).cloned())
            .collect();
        let mut space = Space {
            producers: producers.into_iter().collect(),
            kinds,
            capabilities: capabilities.into_iter().collect(),
            decisions: decisions.into_iter().collect(),
            inputs: ir
                .inputs
                .iter()
                .map(|(id, input)| (id, input.values.as_slice()))
                .collect(),
            ordered,
            children,
            later,
            last,
            orders: orders.flatten(),
            rankings: Vec::new(),
            unlisted: unlisted(ir),
        };
        if orders.is_some() && space.size().is_some_and(|size| size <= super::STATE_BOUND) {
            space.rankings = rankings(space.placeable());
        }
        space
    }

    /// How many things have a place in the order: the `ordered` dimensions and the `later`
    /// records.
    fn placeable(&self) -> usize {
        self.ordered.len() + self.later.len()
    }

    /// Whether the protocol has an `after` or a `since` form reading evidence, which observes
    /// records as [`UNPLACED`] and [`Space::evidence`] say for such a protocol.
    fn after(&self) -> bool {
        !self.later.is_empty()
    }

    /// The evidence dimensions with a history dimension, in the order of their history
    /// dimensions: the `ordered` ones, then the `children` ones.
    fn with_history(&self) -> impl Iterator<Item = &usize> {
        self.ordered.iter().chain(&self.children)
    }

    /// The position of the first later-record dimension.
    fn first_later(&self) -> usize {
        self.first_history() + self.ordered.len() + self.children.len()
    }

    /// The position of the first last-record dimension.
    fn first_last(&self) -> usize {
        self.first_later() + self.later.len()
    }

    /// `` `k` last ``, or `` `k` result `r` last `` for a last record with a result.
    fn last_name(&self, last: &Last<'_>, result: Option<&str>) -> String {
        let name = self.kinds[last.at].name();
        match result {
            Some(result) => format!("{name} result `{}` last", one_line(result)),
            None => format!("{name} last"),
        }
    }

    /// The position of the order dimension, when there is one.
    fn order_at(&self) -> usize {
        self.first_last() + self.last.len()
    }

    /// Whether `state` observes the current records of `class` of the evidence dimension at `at`
    /// first ([`CURRENT_FIRST`]).
    fn current_first(&self, state: &State, at: usize, class: Option<&str>) -> bool {
        let first_later = self.first_later();
        class.is_some_and(|class| {
            self.later.iter().enumerate().any(|(index, later)| {
                later.at == at
                    && later.result == class
                    && state[first_later + index] == CURRENT_FIRST
            })
        })
    }

    /// Whether the earlier record of `class` of the evidence dimension at `at` is left out of
    /// `state` because current records of the class it holds are observed first in its place
    /// ([`CURRENT_FIRST`]).
    fn stood_in(&self, state: &State, at: usize, class: Option<&str>) -> bool {
        self.current_first(state, at, class)
            && self.kinds[at]
                .records(state[at])
                .iter()
                .any(|(held, _, _)| *held == class)
    }

    /// The name of the thing at `item` in the order: an `ordered` dimension, `` `k` ``, a
    /// `later` record, `` `k` result `r` later ``, or, after those, a `last` record, `` `k` last ``
    /// (`` `k` result `r` last `` with a result).
    fn item_name(&self, state: &State, item: usize) -> String {
        if let Some(index) = item.checked_sub(self.placeable()) {
            let (result, _, _) = self
                .last_place(state, index)
                .expect("a last record in the order is added");
            return self.last_name(&self.last[index], result);
        }
        match item.checked_sub(self.ordered.len()) {
            None => self.kinds[self.ordered[item]].name(),
            Some(index) => {
                let later = &self.later[index];
                format!(
                    "{} result `{}` later",
                    self.kinds[later.at].name(),
                    one_line(later.result)
                )
            }
        }
    }

    /// The last record at `index` in `last` in `state`, when it is added: its result, its place,
    /// and its instant as seconds after `2000-01-01T00:00:00Z`.
    fn last_place(&self, state: &State, index: usize) -> Option<(Option<&'a str>, Place, u32)> {
        let last = &self.last[index];
        let (result, place) = last.decode(state[self.first_last() + index])?;
        let ranks = self.ranks(state);
        let seconds = match place {
            Place::At(item) => ranks[item] * 60,
            Place::Past(item) => ranks[item] * 60 + 45,
        };
        Some((result, place, seconds))
    }

    /// The kind and artifact of the records each evidence dimension holds, in dimension order;
    /// `None` for a dimension of a protocol that declares no artifact.
    #[cfg(test)]
    pub(super) fn evidence_dimensions(&self) -> Vec<Option<(&EvidenceKindId, &ArtifactId)>> {
        self.kinds
            .iter()
            .map(|kind| {
                kind.record
                    .as_ref()
                    .map(|record| (&record.kind, &record.subject))
            })
            .collect()
    }

    /// How the last record at `index` in `last` stands against every other record of `state`:
    /// earlier than, at or later than its instant, in record order, `None` for a record without
    /// one. Empty when the record is not added.
    fn standing(&self, state: &State, index: usize) -> Vec<Option<std::cmp::Ordering>> {
        let Some((_, _, seconds)) = self.last_place(state, index) else {
            return Vec::new();
        };
        let own = at_seconds(seconds);
        let records = self.evidence(state);
        // The last records come last, in `last` order, each when it is added.
        let added = (0..self.last.len())
            .filter(|other| self.last_place(state, *other).is_some())
            .count();
        let before = (0..index)
            .filter(|other| self.last_place(state, *other).is_some())
            .count();
        let position = records.len() - added + before;
        records
            .iter()
            .enumerate()
            .filter(|(at, _)| *at != position)
            .map(|(_, record)| {
                let at = record.observed_at.as_ref()?;
                Some(own.as_str().cmp(at.as_str()))
            })
            .collect()
    }

    /// How many values each dimension has, in dimension order; `None` for an evidence kind with
    /// more values than a `u128` holds.
    fn radices(&self) -> Vec<Option<u128>> {
        let kinds = self.kinds.iter().map(Kind::radix);
        let capabilities = self.capabilities.iter().map(|_| Some(3));
        let decisions = self.decisions.iter().map(|_| Some(2));
        let inputs = self
            .inputs
            .iter()
            .map(|(_, values)| u128::try_from(values.len()).ok());
        let histories = self.with_history().map(|at| self.kinds[*at].histories());
        let later = self
            .later
            .iter()
            .map(|later| Some(u128::from(later.radix())));
        let last = self
            .last
            .iter()
            .map(|last| u128::try_from(last.radix()).ok());
        let orders = (self.placeable() >= 2).then_some(self.orders);
        kinds
            .chain(capabilities)
            .chain(decisions)
            .chain(inputs)
            .chain(histories)
            .chain(later)
            .chain(last)
            .chain(orders)
            .collect()
    }

    /// The position of the first history dimension.
    fn first_history(&self) -> usize {
        self.first_input() + self.inputs.len()
    }

    /// The value of the history dimension of the evidence dimension at `at`, which an order form
    /// or a `children` form reads.
    fn history(&self, state: &State, at: usize) -> Option<u32> {
        let index = self.with_history().position(|with| *with == at)?;
        Some(state[self.first_history() + index])
    }

    /// The rank of each `ordered` dimension and then each `later` record in `state`: all `0` when
    /// there is only one.
    fn ranks(&self, state: &State) -> Vec<u32> {
        if self.placeable() < 2 {
            return vec![0; self.placeable()];
        }
        self.rankings[state[self.order_at()] as usize].clone()
    }

    /// Whether the evidence dimension at `at` holds a record in `state`: a current one, or one its
    /// history dimension, one of its later-record dimensions or its last-record dimension adds.
    pub(super) fn holds(&self, state: &State, at: usize) -> bool {
        let first_later = self.first_later();
        state[at] != 0
            || self
                .history(state, at)
                .is_some_and(|value| value != CURRENT)
            || self.later.iter().enumerate().any(|(index, later)| {
                later.at == at && matches!(state[first_later + index], LATER | LATER_UNPLACED)
            })
            || self.holds_last(state, at)
    }

    /// Whether the last-record dimension of a record of the evidence dimension at `at` adds it in
    /// `state`.
    fn holds_last(&self, state: &State, at: usize) -> bool {
        let first_last = self.first_last();
        self.last
            .iter()
            .enumerate()
            .any(|(index, last)| last.at == at && state[first_last + index] != NO_LAST)
    }

    /// The things in the order with a known instant in `state`, each as its position (the
    /// `ordered` dimensions, then the `later` records) and its rank: the dimensions an order form
    /// reads with a current record and no history (or, in a protocol with an `after` or a `since`,
    /// an unknown order), those with an earlier record observed at an instant or a last record,
    /// which is observed relative to its dimension's rank, and the later records placed in the
    /// order.
    fn placed(&self, state: &State) -> Vec<(usize, u32)> {
        let ranks = self.ranks(state);
        let first_later = self.first_later();
        let dimensions = self.ordered.iter().enumerate().filter_map(|(item, at)| {
            let placed = match self.history(state, *at) {
                Some(CURRENT) => state[*at] != 0,
                Some(UNPLACED) => self.after() && state[*at] != 0,
                None => false,
                Some(_) => true,
            };
            placed.then_some(item)
        });
        let later = (0..self.later.len())
            .filter(|index| state[first_later + index] == LATER)
            .map(|index| self.ordered.len() + index);
        let placeable = self.placeable();
        let last = (0..self.last.len()).filter_map(|index| {
            let (_, _, seconds) = self.last_place(state, index)?;
            Some((placeable + index, seconds))
        });
        dimensions
            .chain(later)
            .map(|item| (item, ranks[item] * 60))
            .chain(last)
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
            let count = match (kind.radix(), &kind.values, kind.variants()) {
                (Some(n), ..) => n.to_string(),
                (None, Some(values), Some(variants)) => format!("{}^{variants}", values.len()),
                (None, Some(values), None) => format!(
                    "{}^({} * 2^{})",
                    values.len(),
                    kind.classes.len(),
                    kind.upstreams.len()
                ),
                (None, None, _) => format!(
                    "more than {}^({} * 2^{})",
                    super::STATE_BOUND,
                    kind.classes.len(),
                    kind.upstreams.len()
                ),
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
        let histories = self.with_history().map(|at| {
            let kind = &self.kinds[*at];
            let count = kind.histories().map_or_else(
                || format!("2^{} + 1", kind.classes.len()),
                |n| n.to_string(),
            );
            format!("history of evidence kind {} {count}", kind.name())
        });
        let later = self.later.iter().map(|later| {
            format!(
                "later evidence kind {} result `{}` {}",
                self.kinds[later.at].name(),
                one_line(later.result),
                later.radix()
            )
        });
        let last = self.last.iter().map(|last| {
            let results: Vec<String> = last
                .results
                .iter()
                .flatten()
                .map(|result| format!("`{}`", one_line(result)))
                .collect();
            let with = if results.is_empty() {
                String::new()
            } else {
                format!(" with or without result {}", results.join(", "))
            };
            format!(
                "last evidence kind {}{with} {}",
                self.kinds[last.at].name(),
                last.radix()
            )
        });
        let orders = (self.placeable() >= 2).then(|| {
            let count = self
                .orders
                .map_or_else(|| format!("more than {}", u128::MAX), |n| n.to_string());
            if self.later.is_empty() {
                format!("order of first observations {count}")
            } else {
                format!("order of first observations and later records {count}")
            }
        });
        kinds
            .chain(capabilities)
            .chain(decisions)
            .chain(inputs)
            .chain(histories)
            .chain(later)
            .chain(last)
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
    /// those whose records an evidence match or order form [`positive_reads`] finds reads (an
    /// order form in any position).
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
    /// and so does leaving a dimension's order unknown; a later record weighs one, and so do
    /// observing current records first and a last record; the order weighs nothing. A witness is
    /// the state of least
    /// weight, ties broken by its rendering.
    pub(super) fn weight(&self, state: &State) -> u32 {
        let kinds = self.kinds.len();
        let capabilities = self.capabilities.len();
        let first_input = self.first_input();
        let first_history = self.first_history();
        let first_later = self.first_later();
        state
            .iter()
            .enumerate()
            .map(|(at, value)| {
                if at >= first_later {
                    let first_last = self.first_last();
                    if at < first_last {
                        u32::from(*value != NO_LATER)
                    } else {
                        u32::from(at < self.order_at() && *value != NO_LAST)
                    }
                } else if at >= first_history {
                    match *value {
                        CURRENT => 0,
                        UNPLACED => 1,
                        earlier => (earlier - UNPLACED).count_ones(),
                    }
                } else if at < kinds {
                    self.kinds[at]
                        .records(*value)
                        .iter()
                        .map(|(_, vector, _)| 1 + vector.count_ones())
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
            for (class, vector, profile) in kind.records(*value) {
                let name = kind.name();
                let mut item = match class {
                    Some(result) => format!("evidence {name} result `{}`", one_line(result)),
                    None => format!("evidence {name}"),
                };
                if !profile.is_empty() {
                    item.push_str(&format!(
                        " by a producer independent on {}",
                        listed(profile)
                    ));
                }
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
        for at in self.with_history() {
            let kind = &self.kinds[*at];
            let name = kind.name();
            match self.history(state, *at) {
                Some(UNPLACED) if state[*at] == 0 || self.after() => {
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
                        if (earlier - UNPLACED) & (1 << bit) == 0
                            || self.stood_in(state, *at, *class)
                        {
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
        let first_later = self.first_later();
        for (index, later) in self.later.iter().enumerate() {
            let name = format!(
                "evidence {} result `{}`",
                self.kinds[later.at].name(),
                one_line(later.result)
            );
            match state[first_later + index] {
                LATER => items.push(format!("{name} later")),
                LATER_UNPLACED => {
                    items.push(format!("{name} later, observed at no known instant"));
                }
                CURRENT_FIRST => items.push(format!("{name} observed first")),
                _ => {}
            }
        }
        for (index, last) in self.last.iter().enumerate() {
            if let Some((result, _, _)) = self.last_place(state, index) {
                items.push(format!("evidence {}", self.last_name(last, result)));
            }
        }
        let mut placed = self.placed(state);
        if placed.len() >= 2 {
            placed.sort_by_key(|(item, rank)| (*rank, *item));
            let mut order = format!("order {}", self.item_name(state, placed[0].0));
            for pair in placed.windows(2) {
                let tie = if pair[0].1 == pair[1].1 { "=" } else { "<" };
                order.push_str(&format!(" {tie} {}", self.item_name(state, pair[1].0)));
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
    /// the current ones, in dimension and then class order. A dimension only a `children` form
    /// reads has the same history values and no rank: none of its records, current or earlier, has
    /// an `observed_at`, which the form does not read.
    ///
    /// In a protocol with an `after` or a `since` form (story:evidence-order-after,
    /// story:evidence-order-since), a dimension with earlier records has its current records
    /// thirty seconds after its rank's instant instead, before the next rank's, and those of a
    /// class [`CURRENT_FIRST`] names at its rank's instant, in place of the earlier record of the
    /// class; with [`UNPLACED`], its current records are at its rank's
    /// instant and the earlier record without a result and without `observed_at` is always there.
    /// Each later record comes last, in later-record order: one record of its result about its
    /// dimension's artifact at [`EARLIER`], at the instant of its own rank with [`LATER`] and
    /// without `observed_at` with [`LATER_UNPLACED`].
    ///
    /// A `case_outcome` record (story:case-composition) is the outcome of a child case of its own:
    /// it names its own id as its `child_case`, so no two records name one child, and a record of
    /// the class without a result carries [`Space::unlisted`] instead, an outcome no predicate
    /// names, since a `case_outcome` record must give one.
    pub(crate) fn evidence(&self, state: &State) -> Vec<EvidenceRecord> {
        let ranks = self.ranks(state);
        let late = u32::try_from(self.ordered.len()).expect("few ordered dimensions");
        let after = self.after();
        let rank_of = |at: usize| {
            self.ordered
                .iter()
                .position(|ordered| *ordered == at)
                .map(|index| ranks[index])
        };
        let mut records = Vec::new();
        for (at, (kind, value)) in self.kinds.iter().zip(state).enumerate() {
            let observed_at = |class: Option<&str>| match (self.history(state, at), rank_of(at)) {
                (Some(CURRENT), Some(rank)) => Some(instant(rank)),
                (Some(UNPLACED), Some(rank)) if after => Some(instant(rank)),
                (Some(UNPLACED), _) | (None, _) | (_, None) => None,
                (Some(_), Some(rank)) if after => Some(if self.current_first(state, at, class) {
                    instant(rank)
                } else {
                    half_past(rank)
                }),
                (Some(_), Some(_)) => Some(instant(late)),
            };
            let Some(record) = &kind.record else {
                continue;
            };
            for (class, vector, profile) in kind.records(*value) {
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
                    observed_at: observed_at(class),
                    upstream_revisions,
                    producer: self.producer(profile),
                    ..record.clone()
                });
            }
        }
        for at in self.with_history() {
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
                    if state[*at] == 0 || after {
                        records.push(earlier(None, None, records.len()));
                    }
                }
                Some(value) => {
                    // A dimension only a `children` form reads has no rank and no instant.
                    let observed_at = rank_of(*at).map(instant);
                    for (bit, class) in kind.classes.iter().enumerate() {
                        if (value - UNPLACED) & (1 << bit) == 0 {
                            continue;
                        }
                        if !self.stood_in(state, *at, *class) {
                            records.push(earlier(*class, observed_at.clone(), records.len()));
                        }
                    }
                }
            }
        }
        let first_later = self.first_later();
        for (index, later) in self.later.iter().enumerate() {
            let observed_at = match state[first_later + index] {
                LATER => Some(instant(ranks[self.ordered.len() + index])),
                LATER_UNPLACED => None,
                _ => continue,
            };
            let Some(record) = &self.kinds[later.at].record else {
                continue;
            };
            records.push(EvidenceRecord {
                id: EvidenceId::new(format!("e{}", records.len())),
                result: Some(later.result.to_owned()),
                subject_revision: Revision::new(EARLIER),
                observed_at,
                ..record.clone()
            });
        }
        for (index, last) in self.last.iter().enumerate() {
            let Some(record) = &self.kinds[last.at].record else {
                continue;
            };
            let Some((result, _, seconds)) = self.last_place(state, index) else {
                continue;
            };
            let observed_at = at_seconds(seconds);
            records.push(EvidenceRecord {
                id: EvidenceId::new(format!("e{}", records.len())),
                result: result.map(str::to_owned),
                subject_revision: Revision::new(EARLIER),
                observed_at: Some(observed_at),
                ..record.clone()
            });
        }
        for record in &mut records {
            if record.kind.as_str() == CASE_OUTCOME {
                record.child_case = Some(CaseId::new(record.id.as_str()));
                if record.result.is_none() {
                    record.result = Some(self.unlisted.clone());
                }
            }
        }
        records
    }

    /// The producer of a record of `profile`: none for the empty profile, otherwise
    /// [`PRODUCER`]`-n` for the profile's position `n` among the space's producers, from 1.
    fn producer(&self, profile: &Profile<'a>) -> Option<Principal> {
        if profile.is_empty() {
            return None;
        }
        let at = self
            .producers
            .binary_search(profile)
            .expect("every non-empty profile is a producer");
        Some(Principal::new(format!("{PRODUCER}-{}", at + 1)))
    }

    /// The case's `independence` map for `records`, as the space builds them: each producer they
    /// name, in producer order, listed on exactly the dimensions of its profile, in identifier
    /// order. Empty when no record names a producer.
    pub(crate) fn independence(
        &self,
        records: &[EvidenceRecord],
    ) -> Declarations<Principal, Vec<IndependenceDimensionId>> {
        let named: BTreeSet<&Principal> = records
            .iter()
            .filter_map(|record| record.producer.as_ref())
            .collect();
        Declarations::new(
            self.producers
                .iter()
                .enumerate()
                .filter_map(|(at, profile)| {
                    let producer = Principal::new(format!("{PRODUCER}-{}", at + 1));
                    named.contains(&producer).then(|| {
                        let dimensions = profile.iter().map(|d| (*d).clone()).collect();
                        (producer, dimensions)
                    })
                })
                .collect(),
        )
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
    /// on everything else share it. An erased evidence dimension an order form or a `children`
    /// form reads loses its history too. One an order form reads loses its later and last records
    /// with it, and they stand first, together and alone, in the order, the others keeping their
    /// order behind them: they hold no record, so their rank decides nothing. A last record that
    /// stays but is placed against a thing that goes is moved to a place against a thing that
    /// stays from which it stands as it did against every record that stays; when there is none,
    /// the thing it was placed against keeps its rank.
    pub(super) fn erase(&self, state: &State, erased: &BTreeSet<usize>) -> State {
        let first_history = self.first_history();
        let first_later = self.first_later();
        let first_last = self.first_last();
        let mut erased_state: State = state
            .iter()
            .enumerate()
            .map(|(at, value)| {
                let history_of = at
                    .checked_sub(first_history)
                    .and_then(|index| self.with_history().nth(index));
                let later_of = at
                    .checked_sub(first_later)
                    .and_then(|index| self.later.get(index))
                    .map(|later| &later.at);
                let last_of = at
                    .checked_sub(first_last)
                    .and_then(|index| self.last.get(index))
                    .map(|last| &last.at);
                if erased.contains(&at)
                    || history_of.is_some_and(|of| erased.contains(of))
                    || later_of.is_some_and(|of| erased.contains(of))
                    || last_of.is_some_and(|of| erased.contains(of))
                {
                    0
                } else {
                    *value
                }
            })
            .collect();
        // The evidence dimension each thing in the order belongs to.
        let items: Vec<&usize> = self
            .ordered
            .iter()
            .chain(self.later.iter().map(|later| &later.at))
            .collect();
        // A last record that stays, placed at or after a thing in the order that goes, takes the
        // first of its values with the same result, placed against a thing that stays, that
        // stands as it did against every record that stays; when none does, the thing it is placed
        // against keeps its place in the order.
        let mut pinned: BTreeSet<usize> = BTreeSet::new();
        for (index, last) in self.last.iter().enumerate() {
            let at = first_last + index;
            let Some((result, place)) = last.decode(erased_state[at]) else {
                continue;
            };
            let gone = |place: Place| match place {
                Place::At(item) | Place::Past(item) => erased.contains(items[item]).then_some(item),
            };
            let Some(item) = gone(place) else {
                continue;
            };
            let standing = self.standing(&erased_state, index);
            let moved = (1..u32::try_from(last.radix()).expect("few places")).find(|value| {
                let Some((other, place)) = last.decode(*value) else {
                    return false;
                };
                let mut candidate = erased_state.clone();
                candidate[at] = *value;
                other == result
                    && gone(place).is_none()
                    && self.standing(&candidate, index) == standing
            });
            match moved {
                Some(value) => erased_state[at] = value,
                None => {
                    pinned.insert(item);
                }
            }
        }
        let goes = |item: usize| erased.contains(items[item]) && !pinned.contains(&item);
        if items.len() >= 2 && (0..items.len()).any(goes) {
            let ranks = self.ranks(state);
            let kept: BTreeSet<u32> = (0..items.len())
                .filter(|item| !goes(*item))
                .map(|item| ranks[item])
                .collect();
            let normalized: Vec<u32> = (0..items.len())
                .map(|item| {
                    if goes(item) {
                        0
                    } else {
                        1 + u32::try_from(kept.range(..ranks[item]).count()).expect("few ranks")
                    }
                })
                .collect();
            let index = self
                .rankings
                .binary_search(&normalized)
                .expect("a normalized ranking is a weak order");
            let at = self.order_at();
            erased_state[at] = u32::try_from(index).expect("within the bound");
        }
        erased_state
    }
}

/// The producer profiles of an evidence dimension whose readers demand `demands` (`None` for a
/// match that demands nothing and for an order form): each distinct demand, as a set, and the
/// empty profile, a record without a producer, when some reader demands nothing or none demands
/// anything; in set order.
///
/// A record without a producer is read only by a reader that demands nothing: when every reader
/// demands independence it is read by none, and the state without it evaluates the same way, so
/// it is left out. (Its only trace would be the kind's presence, which `authority-bypass` reads
/// only to exclude a state, so a state with an unread record is never a witness the state without
/// it is not.)
fn profiles<'a>(
    demands: impl Iterator<Item = Option<&'a [IndependenceDimensionId]>>,
) -> Vec<Profile<'a>> {
    let mut profiles: BTreeSet<Profile<'a>> = BTreeSet::new();
    let mut undemanding = false;
    for demand in demands {
        match demand {
            Some(demand) => {
                profiles.insert(demand.iter().collect());
            }
            None => undemanding = true,
        }
    }
    if undemanding || profiles.is_empty() {
        profiles.insert(Profile::new());
    }
    profiles.into_iter().collect()
}

/// What the records of one class and upstream vector can be, as seen by the readers of the
/// dimension: one value per down-closed set of `profiles` under the subset order, written as the
/// antichain of its greatest members (one record each), in order of how many records, then of
/// their positions; the empty set, no record, first. `None` when there are more than
/// [`super::STATE_BOUND`], which no space within the bound could hold.
///
/// That is exactly what a state must decide. A record from a producer independent on any set `S`
/// meets the demands within `S`, and a reader reads a record exactly when its demand is met (a
/// reader that demands nothing is the empty profile, met by every record); the demands records of
/// a class and vector meet together are the union of such sets, which is down-closed: a demand
/// within a met one is met. A match's value depends only on the classes of the records it reads,
/// and the evaluator never asks which record a class came from, so every evidence set has a state
/// whose every reader reads the same classes and vectors, and two values differ in what some reader
/// reads. A down-closed set is met by one record per greatest member, from a producer independent
/// on exactly that demand, and by no fewer.
fn down_sets(profiles: &[Profile<'_>]) -> Option<Vec<Producers>> {
    // Each profile after every profile within it, so a profile is decided after all it contains.
    let mut order: Vec<usize> = (0..profiles.len()).collect();
    order.sort_by_key(|at| (profiles[*at].len(), *at));
    let within =
        |inner: usize, outer: usize| inner != outer && profiles[inner].is_subset(&profiles[outer]);
    let limit = usize::try_from(super::STATE_BOUND).unwrap_or(usize::MAX);
    let mut found: Vec<Producers> = Vec::new();
    let mut included = vec![false; profiles.len()];
    // Depth-first: at `depth`, decide `order[depth]`, out first, then in when every profile within
    // it is in.
    fn walk(
        depth: usize,
        order: &[usize],
        included: &mut [bool],
        within: &dyn Fn(usize, usize) -> bool,
        found: &mut Vec<Producers>,
        limit: usize,
    ) -> bool {
        if found.len() > limit {
            return false;
        }
        let Some(&at) = order.get(depth) else {
            let greatest: Producers = (0..included.len())
                .filter(|member| {
                    included[*member]
                        && !(0..included.len())
                            .any(|other| included[other] && within(*member, other))
                })
                .collect();
            found.push(greatest);
            return true;
        };
        if !walk(depth + 1, order, included, within, found, limit) {
            return false;
        }
        if (0..included.len()).all(|inner| !within(inner, at) || included[inner]) {
            included[at] = true;
            let fits = walk(depth + 1, order, included, within, found, limit);
            included[at] = false;
            return fits;
        }
        true
    }
    if !walk(0, &order, &mut included, &within, &mut found, limit) {
        return None;
    }
    found.sort_by(|a, b| (a.len(), a).cmp(&(b.len(), b)));
    Some(found)
}

/// `` `d`, `e` ``: the dimensions of `profile`, in identifier order.
fn listed(profile: &Profile<'_>) -> String {
    profile
        .iter()
        .map(|dimension| format!("`{}`", one_line(dimension.as_str())))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The instant of rank `rank` in a checked state: whole minutes from `2000-01-01T00:00:00Z`,
/// written without a fraction, so the evaluation's resolution is whole seconds and two ranks, and
/// [`half_past`], are ordered as written (story:evidence-order-subsecond).
fn instant(rank: u32) -> Instant {
    Instant::new(format!("2000-01-01T{:02}:{:02}:00Z", rank / 60, rank % 60))
}

/// Thirty seconds after the instant of rank `rank`: after it, and before the next rank's.
fn half_past(rank: u32) -> Instant {
    Instant::new(format!("2000-01-01T{:02}:{:02}:30Z", rank / 60, rank % 60))
}

/// The instant `seconds` after `2000-01-01T00:00:00Z`, written without a fraction: [`instant`] of
/// rank `r` is `r * 60` of them, [`half_past`] it `r * 60 + 30`.
fn at_seconds(seconds: u32) -> Instant {
    Instant::new(format!(
        "2000-01-01T{:02}:{:02}:{:02}Z",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    ))
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

/// `other`, primed until it is no result a predicate of the protocol names (an evidence match's, a
/// `first`'s, the `then` of an `after` or a `since`, an outcome a `children` form lists) and not
/// `open`: the result of a `case_outcome` record of the class without a result.
fn unlisted(ir: &Ir) -> String {
    let mut named: BTreeSet<&str> = BTreeSet::from([OPEN_OUTCOME]);
    for (_, predicate) in readers(ir) {
        predicate.visit(&mut |node| match node {
            Predicate::Evidence(matching) => named.extend(matching.result.as_deref()),
            Predicate::First(form) => {
                named.insert(form.result.as_str());
            }
            Predicate::After(form) => {
                named.insert(form.then.result.as_str());
            }
            Predicate::Since(form) => {
                named.insert(form.then.result.as_str());
            }
            Predicate::Children(form) => named.extend(form.r#in.iter().map(String::as_str)),
            _ => {}
        });
    }
    let mut unlisted = String::from("other");
    while named.contains(unlisted.as_str()) {
        unlisted.push('\'');
    }
    unlisted
}

/// The evidence kind `case_outcome` as the protocol declares it, which a `children` form reads;
/// `None` when it does not declare it, which the validator refuses of a protocol with the form.
pub(super) fn case_outcome(ir: &Ir) -> Option<&EvidenceKindId> {
    ir.evidence_kinds
        .get_key_value(&EvidenceKindId::new(CASE_OUTCOME))
        .map(|(kind, _)| kind)
}

/// `case_outcome` when both a `children` form and an order form read it, which the space does not
/// cover together: the form reads records an order form would place first or last.
pub(super) fn ordered_child_outcomes(ir: &Ir) -> Option<&EvidenceKindId> {
    let kind = case_outcome(ir)?;
    let (mut children, mut ordered) = (false, false);
    for (_, predicate) in readers(ir) {
        predicate.visit(&mut |node| {
            if matches!(node, Predicate::Children(_)) {
                children = true;
            } else if node.order_reads().iter().any(|(read, _)| *read == kind) {
                ordered = true;
            }
        });
    }
    (children && ordered).then_some(kind)
}

/// What reads evidence in `predicate` itself, in source order: each evidence match, each kind an
/// order form reads, and, for a `children` form, the kind `case_outcome` with each outcome it lists
/// and `open`, about any artifact (story:case-composition).
fn reads_in<'a>(ir: &'a Ir, predicate: &'a Predicate) -> Vec<Read<'a>> {
    let mut found = Vec::new();
    predicate.visit(&mut |node| match node {
        Predicate::Children(form) => {
            if let Some(kind) = case_outcome(ir) {
                let outcomes = form.r#in.iter().map(String::as_str);
                for result in outcomes.chain([OPEN_OUTCOME]) {
                    found.push(Read {
                        kind,
                        subject: None,
                        result: Some(result),
                        independent_on: None,
                        ordered: false,
                        history: true,
                    });
                }
            }
        }
        Predicate::Evidence(matching) => found.push(Read {
            kind: &matching.kind,
            subject: matching.subject.as_ref(),
            result: matching.result.as_deref(),
            independent_on: matching.independent_on.as_deref(),
            ordered: false,
            history: false,
        }),
        Predicate::First(form) => found.push(Read {
            kind: &form.kind,
            subject: form.subject.as_ref(),
            result: Some(form.result.as_str()),
            independent_on: None,
            ordered: true,
            history: false,
        }),
        Predicate::Before(_) | Predicate::After(_) | Predicate::Since(_) => {
            for (kind, subject) in node.order_reads() {
                found.push(Read {
                    kind,
                    subject,
                    result: None,
                    independent_on: None,
                    ordered: true,
                    history: false,
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
/// kind an order form it reaches reads, and `case_outcome`, about any artifact, for each
/// `children` form it reaches.
fn evidence_reached<'a>(ir: &'a Ir, predicate: &'a Predicate) -> Vec<Reached<'a>> {
    let mut reached = Vec::new();
    let mut visited: BTreeSet<&ClaimId> = BTreeSet::new();
    let mut pending = vec![predicate];
    while let Some(next) = pending.pop() {
        next.visit(&mut |node| match node {
            Predicate::Evidence(matching) => {
                reached.push((&matching.kind, matching.subject.as_ref()));
            }
            Predicate::First(_)
            | Predicate::Before(_)
            | Predicate::After(_)
            | Predicate::Since(_) => {
                reached.extend(node.order_reads());
            }
            Predicate::Children(_) => {
                reached.extend(case_outcome(ir).map(|kind| (kind, None)));
            }
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
/// reaches, and `case_outcome` for every `children` form it reaches.
pub(super) fn kinds_read<'a>(ir: &'a Ir, predicate: &'a Predicate) -> BTreeSet<&'a EvidenceKindId> {
    evidence_reached(ir, predicate)
        .into_iter()
        .map(|(kind, _)| kind)
        .collect()
}

/// Every evidence match `predicate` reaches in a positive position, where a record it reads can
/// help the predicate hold, and every order form it reaches in any position: its own and those of
/// every claim it tests, through any number of claim references. `not` flips the position, and so
/// does a claim test `is: false` or `is: unknown`, each of which holds on what the claim's evidence
/// does not establish; `all`, `any` and `is: true` keep it. Each claim is visited once per
/// position. An order form counts for every kind it reads whatever its position: with no record of
/// its kinds it is `unknown` and so is its negation, so a record can help either hold. So does a
/// `children` form for `case_outcome`, about any artifact.
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
            Predicate::First(_)
            | Predicate::Before(_)
            | Predicate::After(_)
            | Predicate::Since(_) => {
                matches.extend(next.order_reads());
            }
            Predicate::Children(_) => {
                matches.extend(case_outcome(ir).map(|kind| (kind, None)));
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
