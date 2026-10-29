//! Semantic difference between two revisions of one protocol (design § 31).
//!
//! [`diff`] is a pure function of two compiled protocols, `canon-ir/1` as [`crate::ir::compile`]
//! returns it. It lists every change between them and classifies each one by what it does to
//! legitimate work:
//!
//! - `TIGHTENING`: less work is legitimate, or more is required of it;
//! - `RELAXATION`: more work is legitimate, or less is required of it;
//! - `BREAKING`: neither, or something a case or an evidence record relies on no longer holds;
//! - `EXPANSION`: a new declaration that requires nothing of existing work;
//! - `NO SEMANTIC CHANGE`: documentation, or a rewriting with the same meaning.
//!
//! Each change is classified on its own, by these rules:
//!
//! | Change | Category |
//! |---|---|
//! | any description | `NO SEMANTIC CHANGE` |
//! | artifact added or removed (a case snapshot gives a revision of exactly the declared artifacts) | `BREAKING` |
//! | evidence kind added | `EXPANSION` |
//! | evidence kind removed | `BREAKING` |
//! | maximum age introduced or shortened | `TIGHTENING` |
//! | maximum age removed or lengthened | `RELAXATION` |
//! | maximum age rewritten with the same length (`60m`, `1h`) | `NO SEMANTIC CHANGE` |
//! | claim added | `EXPANSION` |
//! | claim removed | `BREAKING` |
//! | claim's predicate changed | by implication, in the direction the protocol reads the claim (below) |
//! | obligation added | `TIGHTENING` |
//! | obligation removed | `RELAXATION` |
//! | applicability predicate (`applicable_when`) introduced on an obligation | `RELAXATION` |
//! | applicability predicate removed from an obligation | `TIGHTENING` |
//! | applicability predicate changed | `BREAKING` |
//! | action added | `EXPANSION` |
//! | action removed | `BREAKING` |
//! | capability requirement added to an action | `TIGHTENING` |
//! | capability requirement removed from an action | `RELAXATION` |
//! | action effect changed | `BREAKING` |
//! | evidence kind an action may produce added | `EXPANSION` |
//! | evidence kind an action may produce removed | `BREAKING` |
//! | action argument schema introduced or removed (absent, the action takes only `{}`) | `BREAKING` |
//! | action argument schema narrowed, widened, rewritten or changed | by the rules in `diff/arguments.rs` |
//! | outcome added | `EXPANSION` |
//! | outcome removed, or its requirement changed | `BREAKING` |
//! | invalidation rule added, or a claim added to what it invalidates | `TIGHTENING` |
//! | invalidation rule removed, or a claim removed from what it invalidates | `RELAXATION` |
//! | invalidation rule's upstream artifact changed | `BREAKING` |
//! | case input added or removed (a case gives a value to exactly the declared inputs) | `BREAKING` |
//! | value added to a case input | `EXPANSION` |
//! | value removed from a case input | `BREAKING` |
//! | independence dimension added | `EXPANSION` |
//! | independence dimension removed (a case listing a producer on it is refused) | `BREAKING` |
//! | dimension added to an evidence match's `independent_on`, or the list introduced | `TIGHTENING`; `BREAKING` under `not` |
//! | dimension removed from an evidence match's `independent_on`, or the list removed | `RELAXATION`; `BREAKING` under `not` |
//!
//! An applicability predicate is not compared by implication: an obligation that applies in fewer
//! cases requires less of them, but the classifier does not order two guards, so any change to one
//! present in both revisions is `BREAKING`.
//!
//! A changed predicate — a claim's `true_when`, an obligation's `discharged_when`, an action's
//! `precondition` — is compared by implication. When the new predicate is true only where the old
//! one is, it is strengthened (`TIGHTENING`); when the old one is true only where the new one is,
//! weakened (`RELAXATION`); when both hold, rewritten (`NO SEMANTIC CHANGE`); otherwise changed
//! (`BREAKING`). Implication is decided structurally and soundly, never guessed: a member added
//! under an `all` strengthens it, a member added under an `any` weakens it, `not` reverses the
//! direction, and two evidence matches, claim tests or input tests imply each other only when they
//! are equal, so an input test whose `in` list changed implies nothing it did not before. What it
//! cannot show is classified as changed. An order form (`first`, `before`, `after`, `since`) is compared
//! structurally only: a predicate whose order forms, taken as a set, differ between the revisions
//! is changed (`BREAKING`), whatever members around them were added or removed; one whose order
//! forms are the same is classified by implication, each form implying only an equal one.
//!
//! The independence an evidence match demands (`independent_on`) is not compared by implication
//! either: a match that demands more reads fewer records, which can leave records that disagree
//! out of it, so it is not true only where the other is. It is classified by the design's rule for
//! a stronger independence requirement (§ 31): when the two predicates differ only in the
//! dimensions their matches demand, a dimension added to a match read positively tightens
//! (`predicate demands more independence`), one removed from it relaxes (`predicate demands less
//! independence`), and both at once is `BREAKING` (`changed`). A dimension added to or removed from
//! a match under an odd number of `not`s is `BREAKING`, its line saying what the dimensions did
//! (`demands more independence` when added, `demands less independence` when removed): a match
//! that demands more reads fewer records, so it can be `unknown` where it was `false`, and its
//! negation `unknown` where it was `true`, which blocks work it admitted. A predicate whose demands
//! changed along with anything else is `BREAKING` (`changed`); one whose demands are the same is
//! classified by implication, a match implying only an equal one. A claim's predicate is then
//! directed by how the protocol reads the claim, as below, except that a changed demand read in
//! reverse is `BREAKING`, as under `not`.
//!
//! Because a match that reads mixed results is `unknown`, a demand added to a positive match can
//! also turn `unknown` into `true` (an independent approval beside a non-independent rejection), so
//! `TIGHTENING` here follows design § 31 and is not a proof that no work becomes admissible.
//!
//! The rules hold in three-valued logic: whenever the
//! stronger predicate is true the weaker is true, and whenever the weaker is false the stronger is
//! false.
//!
//! A claim's predicate is classified so, and then by the direction the protocol reads the claim,
//! since a claim strengthened is true in fewer cases, which tightens a place that needs it true
//! and relaxes one that needs it false. Every place a predicate is written reads the claims it
//! tests: another claim's `true_when`, an obligation's `discharged_when` and `applicable_when`, an
//! action's precondition and an outcome's requirement. A claim test `{claim: c}` reads `c`
//! positively, `{claim: c, is: false}` in reverse and `{claim: c, is: unknown}` in both
//! directions; a `not` reverses what it holds; an `applicable_when` reverses the whole predicate,
//! since an obligation that applies in fewer cases requires less; and a claim read by another
//! claim is read in each direction that claim is read in, composed, through any number of claims.
//! The directions are taken from both revisions together. A claim read only positively, or read
//! by nothing, is classified as its predicate is; a claim read only in reverse has `TIGHTENING`
//! and `RELAXATION` exchanged, its line ending `, read in reverse` (a changed independence demand
//! read in reverse is `BREAKING` instead, as above); a claim read in both directions
//! is `BREAKING`, its line ending `, read in both directions`. A predicate rewritten or changed is
//! classified as it is, whatever the direction.
//!
//! The protocol's revision is not a change: two revisions are expected to differ in it. Two
//! documents of different protocols are refused (`different-protocol`).
//!
//! Changes come in a fixed order, so the same two inputs always give the same report: the protocol
//! header, then artifacts, evidence kinds, claims, obligations, actions, outcomes, invalidation
//! rules, case inputs and independence dimensions; within a section, by identifier; within a declaration, its description,
//! then its fields in the order the IR declares them (an obligation's discharge predicate and
//! applicability predicate; an action's precondition, capability requirements, effect, evidence
//! it may produce and argument schema; a case input's values), each list in identifier order, removals before
//! additions. Every IR declaration is destructured whole, so a field added to one does not compile
//! here until the classifier says what a change to it is.
//!
//! Out of scope (story:semantic-diff): live-case impact, which needs case data Canon does not hold
//! (design § 38), and constructs the kernel does not yet have.

mod arguments;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;

use crate::ir::{
    Action, CaseInput, Claim, Described, EvidenceKind, Header, InvalidationRule, Ir, Obligation,
    Outcome,
};
use crate::model::{
    Age, ArtifactId, ClaimId, EvidenceKindId, EvidenceMatch, IndependenceDimensionId, Predicate,
    ProtocolId, Truth, one_line,
};

/// What a change does to legitimate work (design § 31).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Category {
    Tightening,
    Relaxation,
    Breaking,
    Expansion,
    NoSemanticChange,
}

impl Category {
    /// The category's name as the report prints it.
    pub fn as_str(self) -> &'static str {
        match self {
            Category::Tightening => "TIGHTENING",
            Category::Relaxation => "RELAXATION",
            Category::Breaking => "BREAKING",
            Category::Expansion => "EXPANSION",
            Category::NoSemanticChange => "NO SEMANTIC CHANGE",
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One classified change: its category, the declaration it touches (by section and identifier),
/// and what changed, which names any further identifier it touches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub category: Category,
    /// The section of the declaration, by the singular noun of what it declares (`claim`,
    /// `evidence kind`), or `protocol` for the header.
    pub section: &'static str,
    /// The identifier of the declaration, as written.
    pub id: String,
    /// What changed: `added`, `description changed`, ``no longer requires capability `c` ``.
    pub what: String,
}

impl fmt::Display for Change {
    /// ``<CATEGORY>: <section> `<id>` <what>``, every identifier on one line.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} `{}` {}",
            self.category,
            self.section,
            one_line(&self.id),
            self.what
        )
    }
}

/// Every change from one revision of a protocol to another, in the order the module documents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diff {
    pub protocol: ProtocolId,
    pub from_revision: u64,
    pub to_revision: u64,
    pub changes: Vec<Change>,
}

impl Diff {
    /// The report: ``diff: protocol `<id>` revision <from> to <to>``, then one line per change, or
    /// `NO SEMANTIC CHANGE: no declaration differs` when there is none.
    pub fn render(&self) -> String {
        let mut report = format!(
            "diff: protocol `{}` revision {} to {}\n",
            one_line(self.protocol.as_str()),
            self.from_revision,
            self.to_revision
        );
        if self.changes.is_empty() {
            report.push_str(Category::NoSemanticChange.as_str());
            report.push_str(": no declaration differs\n");
        }
        for change in &self.changes {
            report.push_str(&change.to_string());
            report.push('\n');
        }
        report
    }
}

/// Why two documents were not compared: a stable machine-readable code and a one-line message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    code: &'static str,
    message: String,
}

impl Refusal {
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

/// Lists and classifies every change from `from` to `to`, two revisions of one protocol. Refused
/// when their protocol identifiers differ. Runs on a thread with a deep stack, so the comparison of
/// predicates nested as deep as an IR may be ([`crate::eval::MAX_IR_DEPTH`]) does not exhaust the
/// caller's.
pub fn diff(from: &Ir, to: &Ir) -> Result<Diff, Refusal> {
    if from.protocol.id != to.protocol.id {
        return Err(Refusal {
            code: "different-protocol",
            message: format!(
                "the earlier revision is of protocol `{}` and the later of protocol `{}`; only \
                 two revisions of one protocol are compared",
                one_line(from.protocol.id.as_str()),
                one_line(to.protocol.id.as_str())
            ),
        });
    }
    let changes = on_deep_stack(|| changes(from, to));
    Ok(Diff {
        protocol: to.protocol.id.clone(),
        from_revision: from.protocol.revision,
        to_revision: to.protocol.revision,
        changes,
    })
}

/// The stack [`diff`] runs on: as deep as the evaluator's, for the same bound on nesting.
const DEEP_STACK: usize = 64 << 20;

fn on_deep_stack<T: Send>(work: impl FnOnce() -> T + Send) -> T {
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .name("canon-diff".to_owned())
            .stack_size(DEEP_STACK)
            .spawn_scoped(scope, work)
            .expect("the operating system starts a thread for the diff");
        match worker.join() {
            Ok(result) => result,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    })
}

/// Collects changes for one declaration at a time.
struct Changes {
    changes: Vec<Change>,
}

impl Changes {
    fn push(&mut self, category: Category, section: &'static str, id: &str, what: String) {
        self.changes.push(Change {
            category,
            section,
            id: id.to_owned(),
            what,
        });
    }

    fn description(
        &mut self,
        section: &'static str,
        id: &str,
        from: &Option<String>,
        to: &Option<String>,
    ) {
        if from != to {
            self.push(
                Category::NoSemanticChange,
                section,
                id,
                "description changed".to_owned(),
            );
        }
    }

    /// Every declaration of one section: removed, added, or present in both and compared.
    fn section<K: AsRef<str> + Ord, T>(
        &mut self,
        section: &'static str,
        from: &BTreeMap<K, T>,
        to: &BTreeMap<K, T>,
        removed: Category,
        added: Category,
        compare: impl Fn(&mut Self, &str, &T, &T),
    ) {
        let mut ids: Vec<&K> = from.keys().chain(to.keys()).collect();
        ids.sort();
        ids.dedup();
        for id in ids {
            let name: &str = (*id).as_ref();
            match (from.get(id), to.get(id)) {
                (Some(_), None) => self.push(removed, section, name, "removed".to_owned()),
                (None, Some(_)) => self.push(added, section, name, "added".to_owned()),
                (Some(from), Some(to)) => compare(self, name, from, to),
                (None, None) => {}
            }
        }
    }

    /// A predicate present in both revisions, classified by implication.
    fn predicate(
        &mut self,
        section: &'static str,
        id: &str,
        field: &str,
        from: &Predicate,
        to: &Predicate,
    ) {
        if let Some((category, how)) = compared(from, to) {
            self.push(category, section, id, format!("{field} {how}"));
        }
    }

    /// A list of identifiers present in both revisions: each removed one, then each added one.
    fn members<T: AsRef<str> + Ord>(
        &mut self,
        section: &'static str,
        id: &str,
        from: &[T],
        to: &[T],
        removed: (Category, &str),
        added: (Category, &str),
    ) {
        for member in from.iter().filter(|member| !to.contains(member)) {
            self.push(
                removed.0,
                section,
                id,
                format!("{} `{}`", removed.1, one_line(AsRef::<str>::as_ref(member))),
            );
        }
        for member in to.iter().filter(|member| !from.contains(member)) {
            self.push(
                added.0,
                section,
                id,
                format!("{} `{}`", added.1, one_line(AsRef::<str>::as_ref(member))),
            );
        }
    }
}

/// A predicate present in both revisions, compared by implication: its category and how it
/// changed, or `None` when it is the same.
fn compared(from: &Predicate, to: &Predicate) -> Option<(Category, &'static str)> {
    if from == to {
        return None;
    }
    // Order forms are compared structurally only: a predicate whose order forms changed is
    // `BREAKING`, whatever else it implies.
    if order_forms(from) != order_forms(to) {
        return Some((Category::Breaking, "changed"));
    }
    if let Some(changed) = independence_changed(from, to) {
        return Some(changed);
    }
    let mut implication = Implication::default();
    let stronger = implication.implies(to, from);
    let weaker = implication.implies(from, to);
    Some(match (stronger, weaker) {
        (true, true) => (Category::NoSemanticChange, "rewritten"),
        (true, false) => (Category::Tightening, "strengthened"),
        (false, true) => (Category::Relaxation, "weakened"),
        (false, false) => (Category::Breaking, "changed"),
    })
}

/// The order forms `predicate` holds, each once, in canonical-IR form: two predicates hold the same
/// order forms exactly when these are equal.
fn order_forms(predicate: &Predicate) -> Vec<&Predicate> {
    let mut forms = predicate.order_forms();
    forms.sort_by_key(|form| format!("{form:?}"));
    forms.dedup();
    forms
}

/// One dimension an evidence match demands independence on, with the match it is demanded by
/// (kind, result, subject) and whether the match is read positively (`true`: outside a `not`, or
/// under an even number of them).
type Demand<'a> = (
    bool,
    &'a EvidenceKindId,
    Option<&'a str>,
    Option<&'a ArtifactId>,
    &'a IndependenceDimensionId,
);

/// Every independence demand `predicate` makes, each once.
fn demands(predicate: &Predicate) -> BTreeSet<Demand<'_>> {
    fn walk<'a>(predicate: &'a Predicate, positive: bool, found: &mut BTreeSet<Demand<'a>>) {
        match predicate {
            Predicate::All(members) | Predicate::Any(members) => {
                for member in members {
                    walk(member, positive, found);
                }
            }
            Predicate::Not(inner) => walk(inner, !positive, found),
            Predicate::Evidence(matching) => {
                for dimension in matching.independent_on.iter().flatten() {
                    found.insert((
                        positive,
                        &matching.kind,
                        matching.result.as_deref(),
                        matching.subject.as_ref(),
                        dimension,
                    ));
                }
            }
            Predicate::Claim(_)
            | Predicate::Input(_)
            | Predicate::First(_)
            | Predicate::Before(_)
            | Predicate::After(_)
            | Predicate::Since(_) => {}
        }
    }
    let mut found = BTreeSet::new();
    walk(predicate, true, &mut found);
    found
}

/// `predicate` with no evidence match demanding independence, in canonical form.
fn without_independence(predicate: &Predicate) -> Predicate {
    fn strip(predicate: &Predicate) -> Predicate {
        match predicate {
            Predicate::All(members) => Predicate::All(members.iter().map(strip).collect()),
            Predicate::Any(members) => Predicate::Any(members.iter().map(strip).collect()),
            Predicate::Not(inner) => Predicate::Not(Box::new(strip(inner))),
            Predicate::Evidence(matching) => Predicate::Evidence(EvidenceMatch {
                independent_on: None,
                ..matching.clone()
            }),
            other => other.clone(),
        }
    }
    crate::ir::normalize(&strip(predicate))
}

/// A predicate whose independence demands changed, classified by the design's rule for a stronger
/// independence requirement (§ 31) rather than by implication: a demand added to a match read
/// positively tightens, one removed from it relaxes; a demand changed on a match under `not`, both
/// added and removed, or a change to anything else in the same predicate, is `BREAKING`. The line
/// says what the dimensions did, whatever the polarity: `demands more independence` when only
/// added, `demands less independence` when only removed, `changed` otherwise. `None` when the
/// demands are the same, so implication decides.
fn independence_changed(from: &Predicate, to: &Predicate) -> Option<(Category, &'static str)> {
    let (before, after) = (demands(from), demands(to));
    if before == after {
        return None;
    }
    if without_independence(from) != without_independence(to) {
        return Some((Category::Breaking, "changed"));
    }
    let added: Vec<_> = after.difference(&before).collect();
    let removed: Vec<_> = before.difference(&after).collect();
    // Under `not` a demand added can turn `false` into `unknown` (only records that fail it decided
    // the match), which `not` makes `true` into `unknown`: less work is legitimate, not more.
    let negative = added
        .iter()
        .chain(&removed)
        .any(|(positive, ..)| !*positive);
    Some(match (added.is_empty(), removed.is_empty()) {
        (false, true) if negative => (Category::Breaking, "demands more independence"),
        (false, true) => (Category::Tightening, "demands more independence"),
        (true, false) if negative => (Category::Breaking, "demands less independence"),
        (true, false) => (Category::Relaxation, "demands less independence"),
        _ => (Category::Breaking, "changed"),
    })
}

/// The directions a claim is read in: positively (it being true helps the place that reads it
/// hold), in reverse (it being true helps it fail), or both.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Reading {
    positive: bool,
    reverse: bool,
}

impl Reading {
    const POSITIVE: Reading = Reading {
        positive: true,
        reverse: false,
    };

    fn reversed(self) -> Reading {
        Reading {
            positive: self.reverse,
            reverse: self.positive,
        }
    }

    fn or(self, other: Reading) -> Reading {
        Reading {
            positive: self.positive || other.positive,
            reverse: self.reverse || other.reverse,
        }
    }

    /// A reading through a reader that is itself read `outer`: each direction composed with each.
    fn through(self, outer: Reading) -> Reading {
        let mut composed = Reading::default();
        if self.positive {
            composed = composed.or(outer);
        }
        if self.reverse {
            composed = composed.or(outer.reversed());
        }
        composed
    }
}

/// How the predicates of `ir` read claims: for each claim, the direction each place outside the
/// claims section reads it in, and each claim that reads it with the direction of that read.
#[derive(Default)]
struct Readers {
    places: BTreeMap<ClaimId, Reading>,
    claims: BTreeMap<ClaimId, Vec<(ClaimId, Reading)>>,
}

impl Readers {
    fn of(ir: &Ir) -> Readers {
        let mut readers = Readers::default();
        for (id, claim) in &ir.claims {
            let mut read = Vec::new();
            reads(&claim.true_when, Reading::POSITIVE, &mut read);
            for (claim, reading) in read {
                readers
                    .claims
                    .entry(claim)
                    .or_default()
                    .push((id.clone(), reading));
            }
        }
        let mut read = Vec::new();
        for obligation in ir.obligations.values() {
            reads(&obligation.discharged_when, Reading::POSITIVE, &mut read);
            if let Some(applicable_when) = &obligation.applicable_when {
                reads(applicable_when, Reading::POSITIVE.reversed(), &mut read);
            }
        }
        for action in ir.actions.values() {
            reads(&action.precondition, Reading::POSITIVE, &mut read);
        }
        for outcome in ir.outcomes.values() {
            if let Some(requires) = outcome.requires.predicate() {
                reads(requires, Reading::POSITIVE, &mut read);
            }
        }
        for (claim, reading) in read {
            let entry = readers.places.entry(claim).or_default();
            *entry = entry.or(reading);
        }
        readers
    }

    /// Every direction `claim` is read in, or `None` when nothing reads it. A claim on `path` is
    /// being read already: a cycle the validator refuses adds no direction.
    fn reading(&self, claim: &ClaimId, path: &mut BTreeSet<ClaimId>) -> Option<Reading> {
        if !path.insert(claim.clone()) {
            return None;
        }
        let mut reading = self.places.get(claim).copied();
        for (reader, read) in self.claims.get(claim).into_iter().flatten() {
            let outer = self.reading(reader, path).unwrap_or(Reading::POSITIVE);
            let through = read.through(outer);
            reading = Some(reading.map_or(through, |known| known.or(through)));
        }
        path.remove(claim);
        reading
    }
}

/// Each claim `predicate` tests, with the direction it reads it in when the predicate itself is
/// read `reading`: a `not` reverses, `is: false` reverses, `is: unknown` reads both ways.
fn reads(predicate: &Predicate, reading: Reading, found: &mut Vec<(ClaimId, Reading)>) {
    match predicate {
        Predicate::All(members) | Predicate::Any(members) => {
            for member in members {
                reads(member, reading, found);
            }
        }
        Predicate::Not(inner) => reads(inner, reading.reversed(), found),
        Predicate::Claim(test) => {
            let read = match test.is {
                Truth::True => reading,
                Truth::False => reading.reversed(),
                Truth::Unknown => reading.or(reading.reversed()),
            };
            found.push((test.claim.clone(), read));
        }
        Predicate::Evidence(_)
        | Predicate::Input(_)
        | Predicate::First(_)
        | Predicate::Before(_)
        | Predicate::After(_)
        | Predicate::Since(_) => {}
    }
}

/// A claim's changed predicate, classified by implication and then by the direction the two
/// revisions read the claim in, as the module docs say.
fn claim_predicate(
    changes: &mut Changes,
    id: &str,
    from: &Predicate,
    to: &Predicate,
    readers: (&Readers, &Readers),
) {
    let Some((category, how)) = compared(from, to) else {
        return;
    };
    let claim = ClaimId::new(id);
    let reading = [readers.0, readers.1]
        .into_iter()
        .filter_map(|readers| readers.reading(&claim, &mut BTreeSet::new()))
        .reduce(Reading::or)
        .unwrap_or(Reading::POSITIVE);
    let directed = matches!(category, Category::Tightening | Category::Relaxation);
    // A changed demand read in reverse is a demand under `not`: `BREAKING`, as there.
    let demand = independence_changed(from, to).is_some();
    let (category, read) = match (directed, reading.positive, reading.reverse) {
        (true, false, true) if demand => (Category::Breaking, ", read in reverse"),
        (true, false, true) => (
            if category == Category::Tightening {
                Category::Relaxation
            } else {
                Category::Tightening
            },
            ", read in reverse",
        ),
        (true, true, true) => (Category::Breaking, ", read in both directions"),
        _ => (category, ""),
    };
    changes.push(category, "claim", id, format!("predicate {how}{read}"));
}

fn changes(from: &Ir, to: &Ir) -> Vec<Change> {
    use Category::{Breaking, Expansion, Relaxation, Tightening};

    // Whole, without `..`: a section added to the IR does not compile here until it is compared.
    let Ir {
        protocol: from_header,
        artifacts: from_artifacts,
        evidence_kinds: from_evidence_kinds,
        claims: from_claims,
        obligations: from_obligations,
        actions: from_actions,
        outcomes: from_outcomes,
        invalidation: from_invalidation,
        inputs: from_inputs,
        independence_dimensions: from_dimensions,
    } = from;
    let Ir {
        protocol: to_header,
        artifacts: to_artifacts,
        evidence_kinds: to_evidence_kinds,
        claims: to_claims,
        obligations: to_obligations,
        actions: to_actions,
        outcomes: to_outcomes,
        invalidation: to_invalidation,
        inputs: to_inputs,
        independence_dimensions: to_dimensions,
    } = to;

    let mut changes = Changes {
        changes: Vec::new(),
    };
    // The identifier is the same in both, or `diff` refused them; the revision is expected to
    // differ and is not a change.
    let Header {
        id: _,
        revision: _,
        description: from_description,
    } = from_header;
    let Header {
        id,
        revision: _,
        description: to_description,
    } = to_header;
    changes.description("protocol", id.as_str(), from_description, to_description);
    changes.section(
        "artifact",
        from_artifacts,
        to_artifacts,
        Breaking,
        Breaking,
        |changes, id, from: &Described, to: &Described| {
            let Described { description: from } = from;
            let Described { description: to } = to;
            changes.description("artifact", id, from, to);
        },
    );
    changes.section(
        "evidence kind",
        from_evidence_kinds,
        to_evidence_kinds,
        Breaking,
        Expansion,
        evidence_kind,
    );
    let readers = (&Readers::of(from), &Readers::of(to));
    changes.section(
        "claim",
        from_claims,
        to_claims,
        Breaking,
        Expansion,
        |changes, id, from: &Claim, to: &Claim| {
            let Claim {
                description: from_description,
                true_when: from_true_when,
            } = from;
            let Claim {
                description: to_description,
                true_when: to_true_when,
            } = to;
            changes.description("claim", id, from_description, to_description);
            claim_predicate(changes, id, from_true_when, to_true_when, readers);
        },
    );
    changes.section(
        "obligation",
        from_obligations,
        to_obligations,
        Relaxation,
        Tightening,
        obligation,
    );
    changes.section(
        "action",
        from_actions,
        to_actions,
        Breaking,
        Expansion,
        action,
    );
    changes.section(
        "outcome",
        from_outcomes,
        to_outcomes,
        Breaking,
        Expansion,
        |changes, id, from: &Outcome, to: &Outcome| {
            let Outcome {
                description: from_description,
                requires: from_requires,
            } = from;
            let Outcome {
                description: to_description,
                requires: to_requires,
            } = to;
            changes.description("outcome", id, from_description, to_description);
            if from_requires != to_requires {
                changes.push(Breaking, "outcome", id, "requirement changed".to_owned());
            }
        },
    );
    changes.section(
        "invalidation rule",
        from_invalidation,
        to_invalidation,
        Relaxation,
        Tightening,
        invalidation_rule,
    );
    // A case gives a value to exactly the declared inputs: one added or removed breaks every case
    // written against the other revision.
    changes.section(
        "case input",
        from_inputs,
        to_inputs,
        Breaking,
        Breaking,
        case_input,
    );
    // A dimension added requires nothing until a match demands it; one removed breaks every case
    // that lists a producer on it.
    changes.section(
        "independence dimension",
        from_dimensions,
        to_dimensions,
        Breaking,
        Expansion,
        |changes, id, from: &Described, to: &Described| {
            let Described { description: from } = from;
            let Described { description: to } = to;
            changes.description("independence dimension", id, from, to);
        },
    );
    changes.changes
}

fn evidence_kind(changes: &mut Changes, id: &str, from: &EvidenceKind, to: &EvidenceKind) {
    const SECTION: &str = "evidence kind";
    let EvidenceKind {
        description: from_description,
        max_age: from_max_age,
    } = from;
    let EvidenceKind {
        description: to_description,
        max_age: to_max_age,
    } = to;
    changes.description(SECTION, id, from_description, to_description);
    let shown = |age: &Age| one_line(age.as_str());
    let (category, what) = match (from_max_age, to_max_age) {
        (None, None) => return,
        (None, Some(to)) => (
            Category::Tightening,
            format!("maximum age introduced: {}", shown(to)),
        ),
        (Some(from), None) => (
            Category::Relaxation,
            format!("maximum age removed: was {}", shown(from)),
        ),
        (Some(from), Some(to)) if from == to => return,
        (Some(from), Some(to)) => {
            let (category, how) = match (from.seconds(), to.seconds()) {
                (Some(old), Some(new)) if new < old => (Category::Tightening, "shortened"),
                (Some(old), Some(new)) if new > old => (Category::Relaxation, "lengthened"),
                (Some(_), Some(_)) => (Category::NoSemanticChange, "rewritten"),
                // An age the validator would refuse: its length is not known, so neither is the
                // direction of the change.
                _ => (Category::Breaking, "changed"),
            };
            (
                category,
                format!("maximum age {how} from {} to {}", shown(from), shown(to)),
            )
        }
    };
    changes.push(category, SECTION, id, what);
}

/// An obligation's discharge predicate is compared by implication. Its applicability predicate is
/// not: introducing one relaxes (the obligation now applies in fewer cases), removing one tightens
/// (it applies in every case), and any change to one present in both is `BREAKING`.
fn obligation(changes: &mut Changes, id: &str, from: &Obligation, to: &Obligation) {
    const SECTION: &str = "obligation";
    let Obligation {
        description: from_description,
        discharged_when: from_discharged_when,
        applicable_when: from_applicable_when,
    } = from;
    let Obligation {
        description: to_description,
        discharged_when: to_discharged_when,
        applicable_when: to_applicable_when,
    } = to;
    changes.description(SECTION, id, from_description, to_description);
    changes.predicate(
        SECTION,
        id,
        "discharge predicate",
        from_discharged_when,
        to_discharged_when,
    );
    let applicability = match (from_applicable_when, to_applicable_when) {
        (None, None) => None,
        (None, Some(_)) => Some((Category::Relaxation, "introduced")),
        (Some(_), None) => Some((Category::Tightening, "removed")),
        (Some(from), Some(to)) if from == to => None,
        (Some(_), Some(_)) => Some((Category::Breaking, "changed")),
    };
    if let Some((category, how)) = applicability {
        changes.push(
            category,
            SECTION,
            id,
            format!("applicability predicate {how}"),
        );
    }
}

fn action(changes: &mut Changes, id: &str, from: &Action, to: &Action) {
    const SECTION: &str = "action";
    let Action {
        description: from_description,
        precondition: from_precondition,
        requires: from_requires,
        effect: from_effect,
        may_produce: from_may_produce,
        arguments: from_arguments,
    } = from;
    let Action {
        description: to_description,
        precondition: to_precondition,
        requires: to_requires,
        effect: to_effect,
        may_produce: to_may_produce,
        arguments: to_arguments,
    } = to;
    changes.description(SECTION, id, from_description, to_description);
    changes.predicate(
        SECTION,
        id,
        "precondition",
        from_precondition,
        to_precondition,
    );
    changes.members(
        SECTION,
        id,
        from_requires,
        to_requires,
        (Category::Relaxation, "no longer requires capability"),
        (Category::Tightening, "now requires capability"),
    );
    if from_effect != to_effect {
        let shown = |effect: &Option<crate::model::EffectClass>| match effect {
            Some(effect) => format!("`{}`", one_line(effect.as_str())),
            None => "none".to_owned(),
        };
        changes.push(
            Category::Breaking,
            SECTION,
            id,
            format!(
                "effect changed from {} to {}",
                shown(from_effect),
                shown(to_effect)
            ),
        );
    }
    changes.members(
        SECTION,
        id,
        from_may_produce,
        to_may_produce,
        (Category::Breaking, "may no longer produce evidence"),
        (Category::Expansion, "may now produce evidence"),
    );
    if let Some((category, what)) =
        arguments::classify(from_arguments.as_ref(), to_arguments.as_ref())
    {
        changes.push(category, SECTION, id, what.to_owned());
    }
}

fn invalidation_rule(
    changes: &mut Changes,
    id: &str,
    from: &InvalidationRule,
    to: &InvalidationRule,
) {
    const SECTION: &str = "invalidation rule";
    let InvalidationRule {
        description: from_description,
        upstream: from_upstream,
        invalidates: from_invalidates,
    } = from;
    let InvalidationRule {
        description: to_description,
        upstream: to_upstream,
        invalidates: to_invalidates,
    } = to;
    changes.description(SECTION, id, from_description, to_description);
    if from_upstream != to_upstream {
        changes.push(
            Category::Breaking,
            SECTION,
            id,
            format!(
                "upstream artifact changed from `{}` to `{}`",
                one_line(from_upstream.as_str()),
                one_line(to_upstream.as_str())
            ),
        );
    }
    changes.members(
        SECTION,
        id,
        from_invalidates,
        to_invalidates,
        (Category::Relaxation, "no longer invalidates claim"),
        (Category::Tightening, "now invalidates claim"),
    );
}

/// A value added to a case input is `EXPANSION`: every case of the earlier revision still gives
/// a declared value. A value removed is `BREAKING`: a case that gave it no longer evaluates.
fn case_input(changes: &mut Changes, id: &str, from: &CaseInput, to: &CaseInput) {
    const SECTION: &str = "case input";
    let CaseInput {
        description: from_description,
        values: from_values,
    } = from;
    let CaseInput {
        description: to_description,
        values: to_values,
    } = to;
    changes.description(SECTION, id, from_description, to_description);
    changes.members(
        SECTION,
        id,
        from_values,
        to_values,
        (Category::Breaking, "no longer takes value"),
        (Category::Expansion, "now takes value"),
    );
}

/// Structural implication between predicates, remembered per pair of sub-predicates so each pair
/// is decided once: the work is bounded by the product of the two predicates' sizes.
#[derive(Default)]
struct Implication {
    decided: HashMap<(usize, usize), bool>,
}

impl Implication {
    /// Whether `left` being true shows `right` true, and `right` being false shows `left` false,
    /// in three-valued logic, by these rules, each sound on its own:
    ///
    /// 1. an `all` on the right: `left` implies every member;
    /// 2. an `any` on the left: every member implies `right`;
    /// 3. an `all` on the left: some member implies `right`;
    /// 4. an `any` on the right: `left` implies some member;
    /// 5. two `not`s: what the right negates implies what the left negates;
    /// 6. two evidence matches, two claim tests, two input tests or two order forms of one kind:
    ///    they are equal.
    fn implies(&mut self, left: &Predicate, right: &Predicate) -> bool {
        let key = (
            std::ptr::from_ref(left) as usize,
            std::ptr::from_ref(right) as usize,
        );
        if let Some(&known) = self.decided.get(&key) {
            return known;
        }
        let holds = self.decide(left, right);
        self.decided.insert(key, holds);
        holds
    }

    fn decide(&mut self, left: &Predicate, right: &Predicate) -> bool {
        if let Predicate::All(members) = right {
            return members.iter().all(|member| self.implies(left, member));
        }
        if let Predicate::Any(members) = left {
            return members.iter().all(|member| self.implies(member, right));
        }
        if let Predicate::All(members) = left
            && members.iter().any(|member| self.implies(member, right))
        {
            return true;
        }
        if let Predicate::Any(members) = right
            && members.iter().any(|member| self.implies(left, member))
        {
            return true;
        }
        match (left, right) {
            (Predicate::Not(left), Predicate::Not(right)) => self.implies(right, left),
            (Predicate::Evidence(left), Predicate::Evidence(right)) => left == right,
            (Predicate::Claim(left), Predicate::Claim(right)) => left == right,
            // Their `in` lists are sorted and kept once in the IR, so equal sets are equal tests.
            // A test with more values is weaker, but a changed `in` list is not ordered.
            (Predicate::Input(left), Predicate::Input(right)) => left == right,
            // Order forms are compared structurally; `compared` has already classified a change to
            // one as `BREAKING`.
            (Predicate::First(left), Predicate::First(right)) => left == right,
            (Predicate::Before(left), Predicate::Before(right)) => left == right,
            (Predicate::After(left), Predicate::After(right)) => left == right,
            (Predicate::Since(left), Predicate::Since(right)) => left == right,
            // Two different kinds of leaf, or an `all`, `any` or `not` the rules above leave
            // undecided: not shown, so not implied.
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Category, Implication, diff};
    use crate::ir::{Ir, compile};
    use crate::model::{
        self, ClaimId, ClaimTest, EvidenceKindId, EvidenceMatch, InputId, InputTest, InputValue,
        Predicate, Truth,
    };

    fn evidence(kind: &str) -> Predicate {
        Predicate::Evidence(EvidenceMatch {
            kind: EvidenceKindId::new(kind),
            independent_on: None,
            result: None,
            subject: None,
        })
    }

    fn claim(id: &str) -> Predicate {
        Predicate::Claim(ClaimTest {
            claim: ClaimId::new(id),
            is: Truth::True,
        })
    }

    fn implies(left: &Predicate, right: &Predicate) -> bool {
        Implication::default().implies(left, right)
    }

    #[test]
    fn implication_follows_all_any_and_not() {
        let (a, b, c) = (evidence("a"), evidence("b"), claim("c"));
        let ab = Predicate::All(vec![a.clone(), b.clone()]);
        let a_or_b = Predicate::Any(vec![a.clone(), b.clone()]);
        assert!(implies(&ab, &a));
        assert!(!implies(&a, &ab));
        assert!(implies(&a, &a_or_b));
        assert!(!implies(&a_or_b, &a));
        assert!(implies(&ab, &a_or_b));
        assert!(!implies(&a_or_b, &ab));
        let not = |p: &Predicate| Predicate::Not(Box::new(p.clone()));
        assert!(implies(&not(&a), &not(&ab)));
        assert!(!implies(&not(&ab), &not(&a)));
        assert!(!implies(&a, &c));
        for p in [&a, &ab, &a_or_b, &not(&a_or_b), &Predicate::All(vec![])] {
            assert!(implies(p, p), "{p:?} implies itself");
        }
        // `all` of nothing is true, `any` of nothing false.
        assert!(implies(&a, &Predicate::All(vec![])));
        assert!(implies(&Predicate::Any(vec![]), &a));
        assert!(!implies(&Predicate::All(vec![]), &a));
    }

    fn ir(body: &str) -> Ir {
        let source = format!("format: protocol/1\nprotocol: {{id: p, revision: 1}}\n{body}");
        compile(&model::parse(&source).expect("parses")).expect("compiles")
    }

    #[test]
    fn identical_revisions_report_no_change() {
        let one = ir("evidence_kinds: {a: {}}\n");
        let report = diff(&one, &one).expect("one protocol");
        assert!(report.changes.is_empty());
        assert_eq!(
            report.render(),
            "diff: protocol `p` revision 1 to 1\nNO SEMANTIC CHANGE: no declaration differs\n"
        );
    }

    #[test]
    fn two_protocols_are_refused() {
        let one = ir("");
        let mut other = one.clone();
        other.protocol.id = model::ProtocolId::new("q");
        assert_eq!(
            diff(&one, &other).expect_err("refused").code(),
            "different-protocol"
        );
    }

    #[test]
    fn an_age_of_the_same_length_is_rewritten_and_a_longer_one_relaxes() {
        let hour = ir("evidence_kinds: {a: {max_age: 1h}}\n");
        let minutes = ir("evidence_kinds: {a: {max_age: 60m}}\n");
        let day = ir("evidence_kinds: {a: {max_age: 1d}}\n");
        let category =
            |from: &Ir, to: &Ir| diff(from, to).expect("one protocol").changes[0].category;
        assert_eq!(category(&hour, &minutes), Category::NoSemanticChange);
        assert_eq!(category(&hour, &day), Category::Relaxation);
        assert_eq!(category(&day, &hour), Category::Tightening);
    }

    #[test]
    fn changes_come_in_section_then_identifier_order() {
        let from =
            ir("evidence_kinds: {a: {}}\nactions: {z: {}, b: {requires: [{capability: k}]}}\n");
        let to = ir("evidence_kinds: {a: {}, c: {}}\nactions: {b: {}, y: {}}\n");
        let lines: Vec<String> = diff(&from, &to)
            .expect("one protocol")
            .changes
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(
            lines,
            [
                "EXPANSION: evidence kind `c` added",
                "RELAXATION: action `b` no longer requires capability `k`",
                "EXPANSION: action `y` added",
                "BREAKING: action `z` removed",
            ]
        );
    }

    /// The report's lines, without the header line.
    fn lines(from: &Ir, to: &Ir) -> Vec<String> {
        diff(from, to)
            .expect("one protocol")
            .changes
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    /// An input `s` and a claim `c` for the protocols of the tests below.
    const INPUTS: &str = "case: {inputs: {s: {values: [high, low]}}}\nevidence_kinds: {k: {}}\n\
                          claims: {c: {true_when: {evidence: {kind: k}}}}\n";

    #[test]
    fn an_applicability_predicate_introduced_relaxes_removed_tightens_and_changed_breaks() {
        let obligation = |guard: &str| {
            ir(&format!(
                "{INPUTS}obligations: {{o: {{discharged_when: {{claim: c}}{guard}}}}}\n"
            ))
        };
        let always = obligation("");
        let high = obligation(", applicable_when: {input: s, in: [high]}");
        let either = obligation(", applicable_when: {input: s, in: [high, low]}");
        let high_and_c =
            obligation(", applicable_when: {all: [{input: s, in: [high]}, {claim: c}]}");
        assert_eq!(
            lines(&always, &high),
            ["RELAXATION: obligation `o` applicability predicate introduced"]
        );
        assert_eq!(
            lines(&high, &always),
            ["TIGHTENING: obligation `o` applicability predicate removed"]
        );
        // A changed guard is not ordered, not even one narrowed by a conjunct.
        for (from, to) in [(&high, &either), (&either, &high), (&high, &high_and_c)] {
            assert_eq!(
                lines(from, to),
                ["BREAKING: obligation `o` applicability predicate changed"]
            );
        }
        assert!(lines(&high, &high).is_empty());
        // An obligation added with a guard is still an obligation added.
        assert_eq!(
            lines(&ir(INPUTS), &high),
            ["TIGHTENING: obligation `o` added"]
        );
    }

    #[test]
    fn a_value_added_to_an_input_expands_and_every_other_input_change_breaks() {
        let inputs = |inputs: &str| ir(&format!("case: {{inputs: {{{inputs}}}}}\n"));
        let two = inputs("s: {values: [high, low]}");
        let three = inputs("s: {values: [high, low, medium]}");
        let reordered = inputs("s: {values: [low, high]}");
        let described = inputs("s: {description: d, values: [high, low]}");
        let another = inputs("s: {values: [high, low]}, t: {values: [a]}");
        assert_eq!(
            lines(&two, &three),
            ["EXPANSION: case input `s` now takes value `medium`"]
        );
        assert_eq!(
            lines(&three, &two),
            ["BREAKING: case input `s` no longer takes value `medium`"]
        );
        assert_eq!(lines(&two, &another), ["BREAKING: case input `t` added"]);
        assert_eq!(lines(&another, &two), ["BREAKING: case input `t` removed"]);
        assert_eq!(
            lines(&two, &described),
            ["NO SEMANTIC CHANGE: case input `s` description changed"]
        );
        assert!(lines(&two, &reordered).is_empty());
        // Inputs come last, after invalidation rules.
        assert_eq!(
            lines(
                &ir("actions: {a: {}}\ncase: {inputs: {s: {values: [high]}}}\n"),
                &two
            ),
            [
                "BREAKING: action `a` removed",
                "EXPANSION: case input `s` now takes value `low`",
            ]
        );
    }

    #[test]
    fn an_input_test_implies_only_an_equal_one() {
        let test = |values: &[&str]| {
            Predicate::Input(InputTest {
                input: InputId::new("s"),
                r#in: values.iter().map(|value| InputValue::new(*value)).collect(),
            })
        };
        let (high, either) = (test(&["high"]), test(&["high", "low"]));
        assert!(implies(&high, &high));
        assert!(!implies(&high, &either));
        assert!(!implies(&either, &high));
        assert!(!implies(&high, &claim("s")));
        assert!(implies(
            &Predicate::All(vec![high.clone(), claim("c")]),
            &high
        ));
        // A changed `in` list in a precondition or a claim's predicate is BREAKING.
        let action = |values: &str| {
            ir(&format!(
                "{INPUTS}actions: {{a: {{precondition: {{input: s, in: [{values}]}}}}}}\n"
            ))
        };
        assert_eq!(
            lines(&action("high"), &action("high, low")),
            ["BREAKING: action `a` precondition changed"]
        );
        assert_eq!(
            lines(&action("high, low"), &action("high")),
            ["BREAKING: action `a` precondition changed"]
        );
        assert!(lines(&action("high, low"), &action("low, high")).is_empty());
    }

    /// An order form is compared structurally (story:evidence-order-predicate): a predicate whose
    /// order forms changed, were added or were removed is changed, `BREAKING`, even where a member
    /// added under an `all` would otherwise strengthen it. With the same order forms, a predicate is
    /// classified by implication as before, and an equal form implies itself.
    #[test]
    fn a_changed_order_form_is_breaking() {
        let claim = |true_when: &str| {
            ir(&format!(
                "artifacts: {{a: {{}}}}\nevidence_kinds: {{k: {{}}, m: {{}}}}\nclaims: {{c: {{true_when: {true_when}}}}}\n"
            ))
        };
        let red = "{first: {kind: k, result: red}}";
        let km = "{before: {first: k, then: m}}";
        let mk = "{before: {first: m, then: k}}";
        let both = format!("{{all: [{red}, {km}]}}");
        for (from, to) in [
            (
                red.to_owned(),
                "{first: {kind: k, result: green}}".to_owned(),
            ),
            (
                red.to_owned(),
                "{first: {kind: k, result: red, subject: a}}".to_owned(),
            ),
            (km.to_owned(), mk.to_owned()),
            (red.to_owned(), both.clone()),
            (both.clone(), red.to_owned()),
            (
                "{evidence: {kind: k}}".to_owned(),
                format!("{{all: [{{evidence: {{kind: k}}}}, {km}]}}"),
            ),
        ] {
            assert_eq!(
                lines(&claim(&from), &claim(&to)),
                ["BREAKING: claim `c` predicate changed"],
                "{from} -> {to}"
            );
        }
        assert_eq!(
            lines(
                &claim(&both),
                &claim(&format!(
                    "{{all: [{red}, {km}, {{evidence: {{kind: k}}}}]}}"
                ))
            ),
            ["TIGHTENING: claim `c` predicate strengthened"]
        );
        assert!(lines(&claim(&both), &claim(&format!("{{all: [{km}, {red}]}}"))).is_empty());
        let first = |result: &str| {
            Predicate::First(crate::model::FirstResult {
                kind: EvidenceKindId::new("k"),
                result: result.to_owned(),
                subject: None,
            })
        };
        assert!(implies(&first("red"), &first("red")));
        assert!(!implies(&first("red"), &first("green")));
        assert!(!implies(&first("red"), &evidence("k")));
    }

    /// An `after` is compared structurally like the other order forms (story:evidence-order-after):
    /// a change to its first kind, later kind, result or subject, or adding or removing one, is
    /// `BREAKING`; with the same order forms, a member added under an `all` strengthens, reordering
    /// members changes nothing, and it implies only an equal one.
    #[test]
    fn a_changed_after_form_is_breaking() {
        let claim = |true_when: &str| {
            ir(&format!(
                "artifacts: {{a: {{}}}}\nevidence_kinds: {{k: {{}}, m: {{}}}}\nclaims: {{c: {{true_when: {true_when}}}}}\n"
            ))
        };
        let red = "{after: {first: k, then: {kind: m, result: red}}}";
        let both = format!("{{all: [{{evidence: {{kind: k}}}}, {red}]}}");
        for to in [
            "{after: {first: m, then: {kind: m, result: red}}}".to_owned(),
            "{after: {first: k, then: {kind: k, result: red}}}".to_owned(),
            "{after: {first: k, then: {kind: m, result: green}}}".to_owned(),
            "{after: {first: k, then: {kind: m, result: red}, subject: a}}".to_owned(),
            "{before: {first: k, then: m}}".to_owned(),
            format!("{{any: [{{before: {{first: k, then: m}}}}, {red}]}}"),
            "{evidence: {kind: m, result: red}}".to_owned(),
        ] {
            assert_eq!(
                lines(&claim(red), &claim(&to)),
                ["BREAKING: claim `c` predicate changed"],
                "{red} -> {to}"
            );
        }
        assert_eq!(
            lines(&claim(red), &claim(&both)),
            ["TIGHTENING: claim `c` predicate strengthened"]
        );
        assert!(
            lines(
                &claim(&both),
                &claim(&format!("{{all: [{red}, {{evidence: {{kind: k}}}}]}}"))
            )
            .is_empty()
        );
        let after = |result: &str| {
            Predicate::After(crate::model::EvidenceAfter {
                first: EvidenceKindId::new("k"),
                then: crate::model::LaterRecord {
                    kind: EvidenceKindId::new("m"),
                    result: result.to_owned(),
                },
                subject: None,
            })
        };
        assert!(implies(&after("red"), &after("red")));
        assert!(!implies(&after("red"), &after("green")));
        assert!(!implies(&after("red"), &evidence("m")));
    }

    /// A `since` is compared structurally like the other order forms (story:evidence-order-since):
    /// a change to its last kind, later kind, result or subject, a change to an `after` over the
    /// same kinds, or adding or removing one, is `BREAKING`; with the same order forms, a member
    /// added under an `all` strengthens, reordering members changes nothing, and it implies only an
    /// equal one.
    #[test]
    fn a_changed_since_form_is_breaking() {
        let claim = |true_when: &str| {
            ir(&format!(
                "artifacts: {{a: {{}}}}\nevidence_kinds: {{k: {{}}, m: {{}}}}\nclaims: {{c: {{true_when: {true_when}}}}}\n"
            ))
        };
        let red = "{since: {last: k, then: {kind: m, result: red}}}";
        let both = format!("{{all: [{{evidence: {{kind: k}}}}, {red}]}}");
        for to in [
            "{since: {last: m, then: {kind: m, result: red}}}".to_owned(),
            "{since: {last: k, then: {kind: k, result: red}}}".to_owned(),
            "{since: {last: k, then: {kind: m, result: green}}}".to_owned(),
            "{since: {last: k, then: {kind: m, result: red}, subject: a}}".to_owned(),
            "{after: {first: k, then: {kind: m, result: red}}}".to_owned(),
            format!("{{any: [{{before: {{first: k, then: m}}}}, {red}]}}"),
            "{evidence: {kind: m, result: red}}".to_owned(),
        ] {
            assert_eq!(
                lines(&claim(red), &claim(&to)),
                ["BREAKING: claim `c` predicate changed"],
                "{red} -> {to}"
            );
        }
        assert_eq!(
            lines(&claim(red), &claim(&both)),
            ["TIGHTENING: claim `c` predicate strengthened"]
        );
        assert!(
            lines(
                &claim(&both),
                &claim(&format!("{{all: [{red}, {{evidence: {{kind: k}}}}]}}"))
            )
            .is_empty()
        );
        let since = |result: &str| {
            Predicate::Since(crate::model::EvidenceSince {
                last: EvidenceKindId::new("k"),
                then: crate::model::LaterRecord {
                    kind: EvidenceKindId::new("m"),
                    result: result.to_owned(),
                },
                subject: None,
            })
        };
        assert!(implies(&since("red"), &since("red")));
        assert!(!implies(&since("red"), &since("green")));
        assert!(!implies(&since("red"), &evidence("m")));
    }
}
