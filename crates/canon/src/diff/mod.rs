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
//! | obligation added | `TIGHTENING` |
//! | obligation removed | `RELAXATION` |
//! | action added | `EXPANSION` |
//! | action removed | `BREAKING` |
//! | capability requirement added to an action | `TIGHTENING` |
//! | capability requirement removed from an action | `RELAXATION` |
//! | action effect changed | `BREAKING` |
//! | evidence kind an action may produce added | `EXPANSION` |
//! | evidence kind an action may produce removed | `BREAKING` |
//! | outcome added | `EXPANSION` |
//! | outcome removed, or its requirement changed | `BREAKING` |
//! | invalidation rule added, or a claim added to what it invalidates | `TIGHTENING` |
//! | invalidation rule removed, or a claim removed from what it invalidates | `RELAXATION` |
//! | invalidation rule's upstream artifact changed | `BREAKING` |
//!
//! A changed predicate — a claim's `true_when`, an obligation's `discharged_when`, an action's
//! `precondition` — is compared by implication. When the new predicate is true only where the old
//! one is, it is strengthened (`TIGHTENING`); when the old one is true only where the new one is,
//! weakened (`RELAXATION`); when both hold, rewritten (`NO SEMANTIC CHANGE`); otherwise changed
//! (`BREAKING`). Implication is decided structurally and soundly, never guessed: a member added
//! under an `all` strengthens it, a member added under an `any` weakens it, `not` reverses the
//! direction, and two evidence matches or claim tests imply each other only when they are equal.
//! What it cannot show is classified as changed. The rules hold in three-valued logic: whenever the
//! stronger predicate is true the weaker is true, and whenever the weaker is false the stronger is
//! false.
//!
//! The protocol's revision is not a change: two revisions are expected to differ in it. Two
//! documents of different protocols are refused (`different-protocol`).
//!
//! Changes come in a fixed order, so the same two inputs always give the same report: the protocol
//! header, then artifacts, evidence kinds, claims, obligations, actions, outcomes and invalidation
//! rules; within a section, by identifier; within a declaration, its description, then its fields
//! in the order the IR declares them (an action's precondition, capability requirements, effect and
//! evidence it may produce), each list in identifier order, removals before additions.
//!
//! Out of scope (story:semantic-diff): live-case impact, which needs case data Canon does not hold
//! (design § 38), and constructs the kernel does not yet have.

use std::collections::{BTreeMap, HashMap};
use std::fmt;

use crate::ir::{
    Action, Claim, Described, EvidenceKind, InvalidationRule, Ir, Obligation, Outcome,
};
use crate::model::{Age, Predicate, ProtocolId, one_line};

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
        if from == to {
            return;
        }
        let mut implication = Implication::default();
        let stronger = implication.implies(to, from);
        let weaker = implication.implies(from, to);
        let (category, how) = match (stronger, weaker) {
            (true, true) => (Category::NoSemanticChange, "rewritten"),
            (true, false) => (Category::Tightening, "strengthened"),
            (false, true) => (Category::Relaxation, "weakened"),
            (false, false) => (Category::Breaking, "changed"),
        };
        self.push(category, section, id, format!("{field} {how}"));
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

fn changes(from: &Ir, to: &Ir) -> Vec<Change> {
    use Category::{Breaking, Expansion, Relaxation, Tightening};

    let mut changes = Changes {
        changes: Vec::new(),
    };
    changes.description(
        "protocol",
        to.protocol.id.as_str(),
        &from.protocol.description,
        &to.protocol.description,
    );
    changes.section(
        "artifact",
        &from.artifacts,
        &to.artifacts,
        Breaking,
        Breaking,
        |changes, id, from: &Described, to: &Described| {
            changes.description("artifact", id, &from.description, &to.description);
        },
    );
    changes.section(
        "evidence kind",
        &from.evidence_kinds,
        &to.evidence_kinds,
        Breaking,
        Expansion,
        evidence_kind,
    );
    changes.section(
        "claim",
        &from.claims,
        &to.claims,
        Breaking,
        Expansion,
        |changes, id, from: &Claim, to: &Claim| {
            changes.description("claim", id, &from.description, &to.description);
            changes.predicate("claim", id, "predicate", &from.true_when, &to.true_when);
        },
    );
    changes.section(
        "obligation",
        &from.obligations,
        &to.obligations,
        Relaxation,
        Tightening,
        |changes, id, from: &Obligation, to: &Obligation| {
            changes.description("obligation", id, &from.description, &to.description);
            changes.predicate(
                "obligation",
                id,
                "discharge predicate",
                &from.discharged_when,
                &to.discharged_when,
            );
        },
    );
    changes.section(
        "action",
        &from.actions,
        &to.actions,
        Breaking,
        Expansion,
        action,
    );
    changes.section(
        "outcome",
        &from.outcomes,
        &to.outcomes,
        Breaking,
        Expansion,
        |changes, id, from: &Outcome, to: &Outcome| {
            changes.description("outcome", id, &from.description, &to.description);
            if from.requires != to.requires {
                changes.push(Breaking, "outcome", id, "requirement changed".to_owned());
            }
        },
    );
    changes.section(
        "invalidation rule",
        &from.invalidation,
        &to.invalidation,
        Relaxation,
        Tightening,
        invalidation_rule,
    );
    changes.changes
}

fn evidence_kind(changes: &mut Changes, id: &str, from: &EvidenceKind, to: &EvidenceKind) {
    const SECTION: &str = "evidence kind";
    changes.description(SECTION, id, &from.description, &to.description);
    let shown = |age: &Age| one_line(age.as_str());
    let (category, what) = match (&from.max_age, &to.max_age) {
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

fn action(changes: &mut Changes, id: &str, from: &Action, to: &Action) {
    const SECTION: &str = "action";
    changes.description(SECTION, id, &from.description, &to.description);
    changes.predicate(
        SECTION,
        id,
        "precondition",
        &from.precondition,
        &to.precondition,
    );
    changes.members(
        SECTION,
        id,
        &from.requires,
        &to.requires,
        (Category::Relaxation, "no longer requires capability"),
        (Category::Tightening, "now requires capability"),
    );
    if from.effect != to.effect {
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
                shown(&from.effect),
                shown(&to.effect)
            ),
        );
    }
    changes.members(
        SECTION,
        id,
        &from.may_produce,
        &to.may_produce,
        (Category::Breaking, "may no longer produce evidence"),
        (Category::Expansion, "may now produce evidence"),
    );
}

fn invalidation_rule(
    changes: &mut Changes,
    id: &str,
    from: &InvalidationRule,
    to: &InvalidationRule,
) {
    const SECTION: &str = "invalidation rule";
    changes.description(SECTION, id, &from.description, &to.description);
    if from.upstream != to.upstream {
        changes.push(
            Category::Breaking,
            SECTION,
            id,
            format!(
                "upstream artifact changed from `{}` to `{}`",
                one_line(from.upstream.as_str()),
                one_line(to.upstream.as_str())
            ),
        );
    }
    changes.members(
        SECTION,
        id,
        &from.invalidates,
        &to.invalidates,
        (Category::Relaxation, "no longer invalidates claim"),
        (Category::Tightening, "now invalidates claim"),
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
    /// 6. two evidence matches, or two claim tests: they are equal.
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
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Category, Implication, diff};
    use crate::ir::{Ir, compile};
    use crate::model::{self, ClaimId, ClaimTest, EvidenceKindId, EvidenceMatch, Predicate, Truth};

    fn evidence(kind: &str) -> Predicate {
        Predicate::Evidence(EvidenceMatch {
            kind: EvidenceKindId::new(kind),
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
}
