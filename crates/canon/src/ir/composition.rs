//! The static check over a set of compiled protocols that their case openings form no cycle
//! (story:case-composition, D206).
//!
//! An action's `case_open` names the protocol a child case is governed by, never an existing case,
//! so a case cannot be checked against its ancestors at run time; a protocol whose openings can
//! reach it again is refused statically instead. `canon validate` refuses an opening under the
//! protocol's own id, one protocol at a time. [`check_openings`] takes a set of protocols and
//! refuses it when their openings form a cycle: protocol `a` opens a case under `b` when an action
//! of `a` declares `case_open` with protocol `b`, and a cycle is a path of such openings that
//! comes back to where it began. Pure: no file, no clock, no run-time state.
//!
//! The cycle named is the first one found from the least protocol id on any cycle: a depth-first
//! walk from that protocol, through protocols with greater ids only, each opening taken in
//! protocol-id order, a protocol walked through once, closing as soon as a protocol on the walk
//! opens a case under the first. It is written in the order its openings go, from its least id
//! back to it (`a -> c -> b -> a`), a protocol that opens a case under itself as `a -> a`. An
//! opening under a protocol outside the set is an edge out of the set: it closes no cycle. Two
//! protocols of the set with one id are one protocol, with the openings of both.
//!
//! [`check_set`] (story:check-across-protocol-files, `canon check-set`) resolves the names one
//! protocol gives another, which nothing else resolves, over a set of compiled protocols, and
//! reports each finding with a stable code:
//!
//! - `case-open-cycle`: the set's openings form a cycle, as [`check_openings`] names it.
//! - `unknown-opened-protocol`: an action's `case_open` names a protocol that is not in the set.
//! - `undeclared-child-outcome`: a protocol names an outcome of a child case about an artifact `s`
//!   (an outcome a `children` form lists, the result of an evidence match of `case_outcome`, or
//!   the result an order form, `first`, `after`, `since` or `descended`, reads of `case_outcome`,
//!   each about the form's `subject`) that no protocol it opens about `s` (`case_open` with
//!   subject `s`) declares among its outcomes. A form without a subject is checked against every
//!   protocol the protocol opens. `open`, the record of a child that runs, is never one. When a
//!   protocol it opens about `s` is not in the set, its outcomes are not known and the name is
//!   not checked: the opening is a finding of its own. A name is reported once per subject, at
//!   the first declaration that holds it: claims, action preconditions and outcome requirements
//!   in that order, each in id order (an obligation and a decision condition read no evidence).
//! - `unknown-supplier`: an evidence kind's `supplied_by` names a protocol that is not in the set.
//! - `unproduced-supplied-kind`: the supplier is in the set, but no action of it may produce a
//!   kind of the same id (`may_produce`).
//! - `duplicate-protocol-id`: two or more files of the set declare one protocol id, each file
//!   named as the caller names it. A runner resolves an id to one protocol and `case_open` names
//!   no revision, so the files are not pooled: no child outcome and no supplied kind is resolved
//!   against that id, as for a protocol outside the set. One file given twice under one name is
//!   that file once, not a duplicate. The cycle check reads the openings of every file of the id.
//!
//! A protocol's own openings decide which protocols it opens. Pure: the report is a function of
//! the set, whatever its order.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::Ir;
use crate::model::{
    ActionId, ArtifactId, CASE_OUTCOME, ClaimId, EvidenceKindId, OPEN_OUTCOME, OutcomeId,
    OutcomeRequirement, Predicate, ProtocolId, one_line,
};

/// A cycle of case openings among a set of protocols: [`check_openings`] refuses the set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpeningCycle {
    /// The protocols on the cycle in the order their openings go, from the least id back to it:
    /// the first and the last entry are that id.
    pub protocols: Vec<ProtocolId>,
}

impl OpeningCycle {
    /// The stable identifier of the refusal.
    pub fn code(&self) -> &'static str {
        "case-open-cycle"
    }
}

impl fmt::Display for OpeningCycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path: Vec<String> = self
            .protocols
            .iter()
            .map(|protocol| one_line(protocol.as_str()))
            .collect();
        write!(
            f,
            "protocols open cases under each other in a cycle: {}",
            path.join(" -> ")
        )
    }
}

/// Refuses `protocols` when their case openings form a cycle, naming it as the module docs say;
/// accepts a set without one, a chain of openings included.
pub fn check_openings<'a>(protocols: impl IntoIterator<Item = &'a Ir>) -> Result<(), OpeningCycle> {
    let mut opens: BTreeMap<&ProtocolId, BTreeSet<&ProtocolId>> = BTreeMap::new();
    for ir in protocols {
        let opened = opens.entry(&ir.protocol.id).or_default();
        for action in ir.actions.values() {
            if let Some(opening) = &action.case_open {
                opened.insert(&opening.protocol);
            }
        }
    }
    for start in opens.keys() {
        if let Some(path) = cycle_from(start, &opens) {
            return Err(OpeningCycle {
                protocols: path.into_iter().cloned().collect(),
            });
        }
    }
    Ok(())
}

/// The first cycle a depth-first walk from `start` finds through protocols with greater ids,
/// `start` repeated at its end; `None` when no walk from `start` comes back to it.
fn cycle_from<'a>(
    start: &'a ProtocolId,
    opens: &BTreeMap<&'a ProtocolId, BTreeSet<&'a ProtocolId>>,
) -> Option<Vec<&'a ProtocolId>> {
    let successors = |protocol: &ProtocolId| opens.get(protocol).into_iter().flatten().copied();
    let mut walked: BTreeSet<&ProtocolId> = BTreeSet::from([start]);
    // The walk: each protocol on it, with the openings of it not yet taken.
    let mut path: Vec<(&ProtocolId, Vec<&ProtocolId>)> = Vec::new();
    let further = |protocol: &ProtocolId| -> Vec<&'a ProtocolId> {
        let mut next: Vec<&ProtocolId> =
            successors(protocol).filter(|next| *next > start).collect();
        next.reverse();
        next
    };
    path.push((start, further(start)));
    while let Some((protocol, _)) = path.last() {
        if successors(protocol).any(|next| next == start) {
            let mut cycle: Vec<&ProtocolId> = path.iter().map(|(protocol, _)| *protocol).collect();
            cycle.push(start);
            return Some(cycle);
        }
        let next = path.last_mut().and_then(|(_, next)| {
            while let Some(candidate) = next.pop() {
                if !walked.contains(candidate) {
                    return Some(candidate);
                }
            }
            None
        });
        match next {
            Some(next) => {
                walked.insert(next);
                path.push((next, further(next)));
            }
            None => {
                path.pop();
            }
        }
    }
    None
}

/// What [`check_set`] found over a set of compiled protocols, and how much it checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetReport {
    /// The files of the set, each name once: two files with one id count two.
    pub protocols: usize,
    /// The actions of the set that open a child case (`case_open`).
    pub openings: usize,
    /// The evidence kinds of the set that declare `supplied_by`.
    pub supplied: usize,
    /// Every finding, sorted and each once: each protocol id of more than one file first, then a
    /// cycle, then each opening under a protocol outside the set, each undeclared child outcome, each supplier outside the set and each
    /// supplied kind its supplier does not produce, each by protocol id.
    pub findings: Vec<SetFinding>,
}

impl SetReport {
    /// Whether the check found nothing.
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }
}

/// Where a protocol names the outcome of a child case: the declaration whose predicate holds the
/// form. An obligation and an outcome's decision condition read claims and case inputs only (the
/// validator refuses evidence there), so neither ever holds one.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NamedIn {
    Claim(ClaimId),
    Action(ActionId),
    Outcome(OutcomeId),
}

impl fmt::Display for NamedIn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (section, id) = match self {
            NamedIn::Claim(id) => ("claim", id.as_str()),
            NamedIn::Action(id) => ("action", id.as_str()),
            NamedIn::Outcome(id) => ("outcome", id.as_str()),
        };
        write!(f, "{section} `{}`", one_line(id))
    }
}

/// One finding of [`check_set`]. Each has a stable code ([`SetFinding::code`]) and names the
/// protocol, the action, declaration or kind, and the id that is missing.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SetFinding {
    /// Two or more files of the set declare one protocol id (`duplicate-protocol-id`); `sources`
    /// are the files' names, sorted, each once.
    DuplicateProtocolId {
        protocol: ProtocolId,
        sources: Vec<String>,
    },
    /// The set's case openings form a cycle (`case-open-cycle`), as [`check_openings`] names it.
    Cycle(Vec<ProtocolId>),
    /// An action opens a case under a protocol that is not in the set (`unknown-opened-protocol`).
    UnknownOpenedProtocol {
        protocol: ProtocolId,
        action: ActionId,
        opened: ProtocolId,
    },
    /// A protocol names an outcome of a child case about `subject` (every artifact when `None`)
    /// that no protocol it opens about it declares (`undeclared-child-outcome`); `opened` are the
    /// protocols it opens about it, sorted, each once.
    UndeclaredChildOutcome {
        protocol: ProtocolId,
        subject: Option<ArtifactId>,
        outcome: String,
        named_in: NamedIn,
        opened: Vec<ProtocolId>,
    },
    /// An evidence kind is supplied by a protocol that is not in the set (`unknown-supplier`).
    UnknownSupplier {
        protocol: ProtocolId,
        kind: EvidenceKindId,
        supplier: ProtocolId,
    },
    /// An evidence kind is supplied by a protocol of the set no action of which may produce a
    /// kind of the same id (`unproduced-supplied-kind`).
    UnproducedSuppliedKind {
        protocol: ProtocolId,
        kind: EvidenceKindId,
        supplier: ProtocolId,
    },
}

impl SetFinding {
    /// The stable identifier of the finding.
    pub fn code(&self) -> &'static str {
        match self {
            SetFinding::DuplicateProtocolId { .. } => "duplicate-protocol-id",
            SetFinding::Cycle(_) => "case-open-cycle",
            SetFinding::UnknownOpenedProtocol { .. } => "unknown-opened-protocol",
            SetFinding::UndeclaredChildOutcome { .. } => "undeclared-child-outcome",
            SetFinding::UnknownSupplier { .. } => "unknown-supplier",
            SetFinding::UnproducedSuppliedKind { .. } => "unproduced-supplied-kind",
        }
    }
}

/// A protocol id as a finding writes it: in backticks, deceptive characters escaped.
fn quoted(id: &str) -> String {
    format!("`{}`", one_line(id))
}

impl fmt::Display for SetFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SetFinding::DuplicateProtocolId { protocol, sources } => {
                let mut names: Vec<String> = sources.iter().map(|source| quoted(source)).collect();
                let last = names.pop().unwrap_or_default();
                let files = if names.is_empty() {
                    last
                } else {
                    format!("{} and {last}", names.join(", "))
                };
                write!(
                    f,
                    "protocol {} is declared by more than one file of the set: {files}",
                    quoted(protocol.as_str())
                )
            }
            SetFinding::Cycle(protocols) => OpeningCycle {
                protocols: protocols.clone(),
            }
            .fmt(f),
            SetFinding::UnknownOpenedProtocol {
                protocol,
                action,
                opened,
            } => write!(
                f,
                "protocol {} action {} opens a case under protocol {}, which is not in the set",
                quoted(protocol.as_str()),
                quoted(action.as_str()),
                quoted(opened.as_str())
            ),
            SetFinding::UndeclaredChildOutcome {
                protocol,
                subject,
                outcome,
                named_in,
                opened,
            } => {
                let about = match subject {
                    Some(subject) => format!("about {}", quoted(subject.as_str())),
                    None => "about any artifact".to_owned(),
                };
                let opens = if opened.is_empty() {
                    "it opens none".to_owned()
                } else {
                    let names: Vec<String> = opened.iter().map(|id| quoted(id.as_str())).collect();
                    format!("it opens {}", names.join(", "))
                };
                let which = match subject {
                    Some(_) => format!("no protocol it opens {about} declares"),
                    None => "no protocol it opens declares".to_owned(),
                };
                write!(
                    f,
                    "protocol {} {named_in} names outcome {} of a child case {about}, which {which} ({opens})",
                    quoted(protocol.as_str()),
                    quoted(outcome),
                )
            }
            SetFinding::UnknownSupplier {
                protocol,
                kind,
                supplier,
            } => write!(
                f,
                "protocol {} evidence kind {} is supplied by protocol {}, which is not in the set",
                quoted(protocol.as_str()),
                quoted(kind.as_str()),
                quoted(supplier.as_str())
            ),
            SetFinding::UnproducedSuppliedKind {
                protocol,
                kind,
                supplier,
            } => write!(
                f,
                "protocol {} evidence kind {} is supplied by protocol {}, no action of which may produce {}",
                quoted(protocol.as_str()),
                quoted(kind.as_str()),
                quoted(supplier.as_str()),
                quoted(kind.as_str())
            ),
        }
    }
}

/// `n` and the noun, the noun in its plural unless `n` is 1.
fn counted(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

impl fmt::Display for SetReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for finding in &self.findings {
            writeln!(f, "{}: {finding}", finding.code())?;
        }
        writeln!(
            f,
            "checked: {}, {}, {}, {}",
            counted(self.protocols, "protocol", "protocols"),
            counted(self.openings, "opening", "openings"),
            counted(self.supplied, "supplied kind", "supplied kinds"),
            counted(self.findings.len(), "finding", "findings"),
        )
    }
}

/// Every outcome of a child case `predicate` names, as `(subject, outcome)`: the outcomes a
/// `children` form lists, the result of an evidence match of [`CASE_OUTCOME`], and the result an
/// order form (`first`, `after`, `since`, `descended`) reads of a [`CASE_OUTCOME`] record, each
/// with the form's subject. [`OPEN_OUTCOME`] is never one: it is the record of a child that runs.
fn named_outcomes(predicate: &Predicate) -> Vec<(Option<&ArtifactId>, &str)> {
    let mut found = Vec::new();
    predicate.visit(&mut |form| {
        // `(kind, result)` pairs the form reads, and its subject.
        let (reads, subject): (Vec<(&str, &str)>, &Option<ArtifactId>) = match form {
            Predicate::Children(children) => (
                children
                    .r#in
                    .iter()
                    .map(|outcome| (CASE_OUTCOME, outcome.as_str()))
                    .collect(),
                &children.subject,
            ),
            Predicate::Evidence(matching) => (
                matching
                    .result
                    .iter()
                    .map(|result| (matching.kind.as_str(), result.as_str()))
                    .collect(),
                &matching.subject,
            ),
            Predicate::First(order) => (
                vec![(order.kind.as_str(), order.result.as_str())],
                &order.subject,
            ),
            Predicate::After(order) => (
                vec![(order.then.kind.as_str(), order.then.result.as_str())],
                &order.subject,
            ),
            Predicate::Since(order) => (
                vec![(order.then.kind.as_str(), order.then.result.as_str())],
                &order.subject,
            ),
            Predicate::Descended(order) => (
                vec![
                    (order.first.kind.as_str(), order.first.result.as_str()),
                    (order.then.kind.as_str(), order.then.result.as_str()),
                ],
                &order.subject,
            ),
            _ => return,
        };
        for (kind, result) in reads {
            if kind == CASE_OUTCOME && result != OPEN_OUTCOME {
                found.push((subject.as_ref(), result));
            }
        }
    });
    found
}

/// Every predicate of `ir` that may read evidence, with the declaration that holds it: claims,
/// action preconditions and outcome requirements that are predicates, in that order, each section
/// in id order.
fn predicates(ir: &Ir) -> Vec<(NamedIn, &Predicate)> {
    let mut found = Vec::new();
    for (id, claim) in &ir.claims {
        found.push((NamedIn::Claim(id.clone()), &claim.true_when));
    }
    for (id, action) in &ir.actions {
        found.push((NamedIn::Action(id.clone()), &action.precondition));
    }
    for (id, outcome) in &ir.outcomes {
        if let OutcomeRequirement::Predicate(predicate) = &outcome.requires {
            found.push((NamedIn::Outcome(id.clone()), predicate));
        }
    }
    found
}

/// Checks a set of compiled protocols against each other, as the module docs say, and reports
/// every finding with what it checked. Each member is `(source, protocol)`, `source` the name of
/// the file it was read from as a finding writes it; a source given twice is one member, the
/// first given. Pure: the report is a function of the set, whatever its order; two files with one
/// protocol id are the finding `duplicate-protocol-id`, and nothing is resolved against that id.
pub fn check_set<'a>(members: impl IntoIterator<Item = (&'a str, &'a Ir)>) -> SetReport {
    let mut by_source: BTreeMap<&str, &Ir> = BTreeMap::new();
    for (source, ir) in members {
        by_source.entry(source).or_insert(ir);
    }
    let protocols: Vec<&Ir> = by_source.values().copied().collect();
    // Per protocol id: the files that declare it.
    let mut sources: BTreeMap<&ProtocolId, Vec<&str>> = BTreeMap::new();
    for (source, ir) in &by_source {
        sources.entry(&ir.protocol.id).or_default().push(*source);
    }
    let mut findings: BTreeSet<SetFinding> = BTreeSet::new();
    // Per protocol id of one file: the outcomes it declares and the kinds its actions may produce.
    // An id of more than one file is a finding and resolves nothing.
    let mut outcomes: BTreeMap<&ProtocolId, BTreeSet<&str>> = BTreeMap::new();
    let mut produces: BTreeMap<&ProtocolId, BTreeSet<&str>> = BTreeMap::new();
    for ir in &protocols {
        let id = &ir.protocol.id;
        if let Some(files) = sources.get(id).filter(|files| files.len() > 1) {
            findings.insert(SetFinding::DuplicateProtocolId {
                protocol: id.clone(),
                sources: files.iter().map(|file| (*file).to_owned()).collect(),
            });
            continue;
        }
        outcomes.insert(id, ir.outcomes.keys().map(OutcomeId::as_str).collect());
        produces.insert(
            id,
            ir.actions
                .values()
                .flat_map(|action| action.may_produce.iter().map(EvidenceKindId::as_str))
                .collect(),
        );
    }
    if let Err(cycle) = check_openings(protocols.iter().copied()) {
        findings.insert(SetFinding::Cycle(cycle.protocols));
    }
    let mut openings = 0;
    let mut supplied = 0;
    for ir in &protocols {
        let id = &ir.protocol.id;
        // The protocols this one opens about each artifact.
        let mut opened: BTreeMap<&ArtifactId, BTreeSet<&ProtocolId>> = BTreeMap::new();
        for (action_id, action) in &ir.actions {
            let Some(opening) = &action.case_open else {
                continue;
            };
            openings += 1;
            opened
                .entry(&opening.subject)
                .or_default()
                .insert(&opening.protocol);
            if !sources.contains_key(&opening.protocol) {
                findings.insert(SetFinding::UnknownOpenedProtocol {
                    protocol: id.clone(),
                    action: action_id.clone(),
                    opened: opening.protocol.clone(),
                });
            }
        }
        let every: BTreeSet<&ProtocolId> = opened.values().flatten().copied().collect();
        let none: BTreeSet<&ProtocolId> = BTreeSet::new();
        // Each outcome named about one subject once, at the first declaration that names it.
        let mut checked: BTreeSet<(Option<&ArtifactId>, &str)> = BTreeSet::new();
        for (named_in, predicate) in predicates(ir) {
            for (subject, outcome) in named_outcomes(predicate) {
                if !checked.insert((subject, outcome)) {
                    continue;
                }
                let children: &BTreeSet<&ProtocolId> = match subject {
                    Some(subject) => opened.get(subject).unwrap_or(&none),
                    None => &every,
                };
                // A child outside the set, or of an id more than one file declares, is already a
                // finding, and its outcomes are not known.
                if children.iter().any(|child| !outcomes.contains_key(child)) {
                    continue;
                }
                let declared = children
                    .iter()
                    .any(|child| outcomes.get(child).is_some_and(|set| set.contains(outcome)));
                if !declared {
                    findings.insert(SetFinding::UndeclaredChildOutcome {
                        protocol: id.clone(),
                        subject: subject.cloned(),
                        outcome: outcome.to_owned(),
                        named_in: named_in.clone(),
                        opened: children.iter().map(|child| (*child).clone()).collect(),
                    });
                }
            }
        }
        for (kind, declared) in &ir.evidence_kinds {
            let Some(supplier) = &declared.supplied_by else {
                continue;
            };
            supplied += 1;
            if !sources.contains_key(supplier) {
                findings.insert(SetFinding::UnknownSupplier {
                    protocol: id.clone(),
                    kind: kind.clone(),
                    supplier: supplier.clone(),
                });
                continue;
            }
            // A supplier of an id more than one file declares is already a finding.
            if produces
                .get(supplier)
                .is_some_and(|produced| !produced.contains(kind.as_str()))
            {
                findings.insert(SetFinding::UnproducedSuppliedKind {
                    protocol: id.clone(),
                    kind: kind.clone(),
                    supplier: supplier.clone(),
                });
            }
        }
    }
    SetReport {
        protocols: by_source.len(),
        openings,
        supplied,
        findings: findings.into_iter().collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::compile;
    use crate::model::parse;

    /// A protocol `id` whose actions open a case under each protocol of `opens`.
    fn protocol(id: &str, opens: &[&str]) -> Ir {
        let actions: String = opens
            .iter()
            .enumerate()
            .map(|(n, opened)| {
                format!("  open{n}: {{case_open: {{protocol: '{opened}', subject: thing}}}}\n")
            })
            .collect();
        let actions = if actions.is_empty() {
            String::new()
        } else {
            format!("actions:\n{actions}")
        };
        let text = format!(
            "format: protocol/1\nprotocol: {{id: '{id}', revision: 1}}\nartifacts: {{thing: {{}}}}\n{actions}"
        );
        compile(&parse(&text).expect("parses")).expect("compiles")
    }

    fn cycle(protocols: &[Ir]) -> Option<String> {
        check_openings(protocols).err().map(|cycle| {
            assert_eq!(cycle.code(), "case-open-cycle");
            cycle.to_string()
        })
    }

    /// D206: two protocols that open each other are refused, the cycle named from its least id;
    /// a chain without a cycle is accepted, and so is an empty set.
    #[test]
    fn two_protocols_that_open_each_other_are_refused_and_a_chain_is_accepted() {
        assert_eq!(
            cycle(&[
                protocol("review", &["audit"]),
                protocol("audit", &["review"])
            ]),
            Some(
                "protocols open cases under each other in a cycle: audit -> review -> audit".into()
            )
        );
        assert_eq!(
            cycle(&[
                protocol("audit", &["software.change"]),
                protocol("software.change", &["review"]),
                protocol("review", &[]),
            ]),
            None
        );
        assert_eq!(cycle(&[]), None);
    }

    /// The cycle goes the way its openings go, from its least id, whatever order the set is given
    /// in; ids not on the cycle and openings out of the set take no part.
    #[test]
    fn the_cycle_is_written_in_opening_order_from_its_least_id() {
        let set = [
            protocol("c", &["b", "outside"]),
            protocol("b", &["d"]),
            protocol("a", &["c"]),
            protocol("d", &["a"]),
            protocol("e", &["a"]),
        ];
        let expected = "protocols open cases under each other in a cycle: a -> c -> b -> d -> a";
        assert_eq!(cycle(&set).as_deref(), Some(expected));
        let mut reversed = set.clone();
        reversed.reverse();
        assert_eq!(cycle(&reversed).as_deref(), Some(expected));
    }

    /// Of several cycles, the one through the least id on any cycle is named, and of those
    /// through it, the first a walk in id order finds; a protocol opening itself, which `canon
    /// validate` refuses on its own, is the cycle `p -> p` when a caller builds one.
    #[test]
    fn the_least_cycle_is_named_and_a_self_opening_is_a_cycle_of_one() {
        let set = [
            protocol("m", &["n"]),
            protocol("n", &["m"]),
            protocol("b", &["z", "c"]),
            protocol("c", &["b"]),
            protocol("z", &["b"]),
        ];
        assert_eq!(
            cycle(&set).as_deref(),
            Some("protocols open cases under each other in a cycle: b -> c -> b")
        );
        let mut own = protocol("p", &[]);
        let mut opening = protocol("q", &["p"]);
        let (id, action) = opening.actions.pop_first().expect("an action");
        own.actions.insert(id, action);
        assert_eq!(
            cycle(&[own]).as_deref(),
            Some("protocols open cases under each other in a cycle: p -> p")
        );
    }

    /// Two protocols of the set with one id are one protocol, with the openings of both.
    #[test]
    fn protocols_with_one_id_pool_their_openings() {
        assert_eq!(
            cycle(&[
                protocol("a", &["b"]),
                protocol("a", &[]),
                protocol("b", &["a"]),
            ])
            .as_deref(),
            Some("protocols open cases under each other in a cycle: a -> b -> a")
        );
    }

    /// A walk does not pass through a protocol twice, and one that came back to a protocol other
    /// than the first is not a cycle through the first: a long chain of openings with an inner
    /// loop names the inner loop, from its own least id.
    #[test]
    fn an_inner_loop_is_named_from_its_own_least_id() {
        let set = [
            protocol("a", &["b"]),
            protocol("b", &["c"]),
            protocol("c", &["d"]),
            protocol("d", &["c"]),
        ];
        assert_eq!(
            cycle(&set).as_deref(),
            Some("protocols open cases under each other in a cycle: c -> d -> c")
        );
    }

    /// A protocol compiled from the sections written after its header.
    fn written(id: &str, sections: &str) -> Ir {
        let text = format!("format: protocol/1\nprotocol: {{id: '{id}', revision: 1}}\n{sections}");
        compile(&parse(&text).expect("parses")).expect("compiles")
    }

    /// [`check_set`] over `set`, each protocol read from a file named by its id: a set whose ids
    /// differ, so no member is a duplicate of another.
    fn checked<'a>(set: impl IntoIterator<Item = &'a Ir>) -> SetReport {
        let set: Vec<&Ir> = set.into_iter().collect();
        check_set(set.iter().map(|ir| (ir.protocol.id.as_str(), *ir)))
    }

    /// A child protocol `id` about `thing` that declares the outcomes `outcomes`.
    fn child(id: &str, outcomes: &[&str]) -> Ir {
        let outcomes: String = outcomes
            .iter()
            .map(|outcome| format!("  '{outcome}': {{requires: {{all: []}}}}\n"))
            .collect();
        written(
            id,
            &format!("artifacts: {{thing: {{}}}}\noutcomes:\n{outcomes}"),
        )
    }

    /// A parent `p` that opens `change` about `a` and `check` about `b`, with the claims written.
    fn parent(claims: &str) -> Ir {
        written(
            "p",
            &format!(
                "artifacts: {{a: {{}}, b: {{}}}}\nevidence_kinds: {{case_outcome: {{}}, other: {{}}}}\n\
                 claims:\n{claims}actions:\n  \
                 open.a: {{case_open: {{protocol: change, subject: a}}}}\n  \
                 open.b: {{case_open: {{protocol: check, subject: b}}}}\n"
            ),
        )
    }

    /// The report's lines over `p` with `claims`, `change` (accepted, declined) and `check`
    /// (passed, failed).
    fn lines(claims: &str) -> Vec<String> {
        let set = [
            parent(claims),
            child("change", &["accepted", "declined"]),
            child("check", &["passed", "failed"]),
        ];
        checked(&set)
            .to_string()
            .lines()
            .map(str::to_owned)
            .collect()
    }

    /// Story acceptance 2: `open` is never a finding, a name about a subject is checked against
    /// the protocols opened about it, and a form without a subject against every protocol opened.
    #[test]
    fn open_and_declared_names_are_clean_and_a_form_without_subject_reads_every_child() {
        let clean = lines(
            "  c1: {true_when: {children: {in: [accepted, declined], subject: a}}}\n  \
             c2: {true_when: {evidence: {kind: case_outcome, subject: a, result: open}}}\n  \
             c3: {true_when: {children: {in: [declined, passed]}}}\n  \
             c4: {true_when: {evidence: {kind: case_outcome, result: failed}}}\n  \
             c5: {true_when: {evidence: {kind: other, subject: a, result: lost}}}\n",
        );
        assert_eq!(
            clean,
            ["checked: 3 protocols, 2 openings, 0 supplied kinds, 0 findings"]
        );
        assert_eq!(
            lines("  c1: {true_when: {children: {in: [lost]}}}\n"),
            [
                "undeclared-child-outcome: protocol `p` claim `c1` names outcome `lost` of a child case about any artifact, which no protocol it opens declares (it opens `change`, `check`)",
                "checked: 3 protocols, 2 openings, 0 supplied kinds, 1 finding",
            ]
        );
    }

    /// Story acceptance 2: an outcome named about `a` that only the child opened about `b`
    /// declares is a finding; so is one about an artifact the protocol opens nothing about.
    #[test]
    fn an_outcome_of_the_child_about_another_subject_is_a_finding() {
        assert_eq!(
            lines(
                "  c1: {true_when: {evidence: {kind: case_outcome, subject: a, result: passed}}}\n"
            ),
            [
                "undeclared-child-outcome: protocol `p` claim `c1` names outcome `passed` of a child case about `a`, which no protocol it opens about `a` declares (it opens `change`)",
                "checked: 3 protocols, 2 openings, 0 supplied kinds, 1 finding",
            ]
        );
        let set = [
            written(
                "q",
                "artifacts: {a: {}, z: {}}\nevidence_kinds: {case_outcome: {}}\n\
                 claims: {c: {true_when: {children: {in: [accepted], subject: z}}}}\n\
                 actions: {open: {case_open: {protocol: change, subject: a}}}\n",
            ),
            child("change", &["accepted"]),
        ];
        assert_eq!(
            checked(&set).findings[0].to_string(),
            "protocol `q` claim `c` names outcome `accepted` of a child case about `z`, which no protocol it opens about `z` declares (it opens none)"
        );
    }

    /// Every form that names a child's outcome is read: a `children` form, an evidence match and
    /// the order forms `first`, `after`, `since` and `descended`; `before` names no result. Each
    /// name is reported once per subject, at the first declaration holding it.
    #[test]
    fn every_form_naming_a_child_outcome_is_read_once_per_subject() {
        let found = lines(
            "  c1: {true_when: {first: {kind: case_outcome, result: f1, subject: a}}}\n  \
             c2: {true_when: {after: {first: other, then: {kind: case_outcome, result: f2}, subject: a}}}\n  \
             c3: {true_when: {since: {last: other, then: {kind: case_outcome, result: f3}, subject: a}}}\n  \
             c4: {true_when: {descended: {first: {kind: case_outcome, result: f4}, then: {kind: case_outcome, result: f5}, subject: a}}}\n  \
             c5: {true_when: {before: {first: case_outcome, then: other, subject: a}}}\n  \
             c6: {true_when: {not: {children: {in: [f1], subject: a}}}}\n",
        );
        let named: Vec<&str> = found
            .iter()
            .filter_map(|line| line.split(" names outcome `").nth(1)?.split('`').next())
            .collect();
        assert_eq!(named, ["f1", "f2", "f3", "f4", "f5"]);
        assert!(found[0].contains("claim `c1`"), "{found:?}");
        assert_eq!(
            found.last().map(String::as_str),
            Some("checked: 3 protocols, 2 openings, 0 supplied kinds, 5 findings")
        );
    }

    /// Action preconditions and outcome requirements are read too, and the place named is the
    /// declaration holding the form.
    #[test]
    fn actions_and_outcomes_are_read() {
        let set = [
            written(
                "p",
                "artifacts: {a: {}}\nevidence_kinds: {case_outcome: {}}\n\
                 claims: {o: {true_when: {children: {in: [o1], subject: a}}}}\n\
                 actions:\n  open: {case_open: {protocol: change, subject: a}}\n  \
                 go: {precondition: {children: {in: [a1], subject: a}}}\n\
                 outcomes: {done: {requires: {evidence: {kind: case_outcome, subject: a, result: d1}}}}\n",
            ),
            child("change", &["accepted"]),
        ];
        let found: Vec<String> = checked(&set)
            .findings
            .iter()
            .map(|finding| finding.to_string())
            .collect();
        let places: Vec<&str> = found
            .iter()
            .filter_map(|line| line.strip_prefix("protocol `p` ")?.split(" names").next())
            .collect();
        assert_eq!(places, ["action `go`", "outcome `done`", "claim `o`"]);
    }

    /// The four other codes, each naming the protocol, the action or kind and the missing id; a
    /// name about a subject whose opened protocol is outside the set is not a second finding.
    #[test]
    fn missing_protocols_and_suppliers_are_named_with_their_codes() {
        let set = [
            written(
                "p",
                "artifacts: {a: {}}\n\
                 evidence_kinds: {case_outcome: {}, k1: {supplied_by: ghost}, k2: {supplied_by: s}, k3: {supplied_by: s}}\n\
                 claims: {c: {true_when: {children: {in: [lost], subject: a}}}}\n\
                 actions: {open: {case_open: {protocol: absent, subject: a}}}\n",
            ),
            written(
                "s",
                "artifacts: {a: {}}\nevidence_kinds: {k3: {}}\nactions: {make: {may_produce: [{evidence: k3}]}}\n",
            ),
        ];
        let report = checked(&set);
        let codes: Vec<&str> = report.findings.iter().map(SetFinding::code).collect();
        assert_eq!(
            codes,
            [
                "unknown-opened-protocol",
                "unknown-supplier",
                "unproduced-supplied-kind"
            ]
        );
        assert_eq!(
            report.to_string(),
            "unknown-opened-protocol: protocol `p` action `open` opens a case under protocol `absent`, which is not in the set\n\
             unknown-supplier: protocol `p` evidence kind `k1` is supplied by protocol `ghost`, which is not in the set\n\
             unproduced-supplied-kind: protocol `p` evidence kind `k2` is supplied by protocol `s`, no action of which may produce `k2`\n\
             checked: 2 protocols, 1 opening, 3 supplied kinds, 3 findings\n"
        );
        assert!(!report.is_clean());
    }

    /// The cycle of the set is reported with its code, first; the report is the same whatever the
    /// order of the set. Two files with one id are not pooled (they were before the wave review):
    /// they are the finding `duplicate-protocol-id`, and neither the child outcome `late` nor the
    /// supplied kind `k` is resolved against the id `s`, as for a protocol outside the set.
    #[test]
    fn a_cycle_is_reported_and_the_order_and_shared_ids_change_nothing() {
        let set = [
            protocol("audit", &["review"]),
            protocol("review", &["audit"]),
        ];
        let report = checked(&set);
        assert_eq!(
            report.to_string(),
            "case-open-cycle: protocols open cases under each other in a cycle: audit -> review -> audit\n\
             checked: 2 protocols, 2 openings, 0 supplied kinds, 1 finding\n"
        );
        let parent = written(
            "p",
            "artifacts: {a: {}}\nevidence_kinds: {case_outcome: {}, k: {supplied_by: s}}\n\
             claims: {c: {true_when: {children: {in: [late], subject: a}}}}\n\
             actions: {open: {case_open: {protocol: s, subject: a}}}\n",
        );
        let producer = written(
            "s",
            "artifacts: {a: {}}\nevidence_kinds: {k: {}}\nactions: {make: {may_produce: [{evidence: k}]}}\n",
        );
        let ender = child("s", &["late"]);
        let forward = check_set([
            ("p.yaml", &parent),
            ("s.yaml", &producer),
            ("s-late.yaml", &ender),
        ]);
        let backward = check_set([
            ("s-late.yaml", &ender),
            ("s.yaml", &producer),
            ("p.yaml", &parent),
        ]);
        assert_eq!(forward, backward);
        assert_eq!(
            forward.to_string(),
            "duplicate-protocol-id: protocol `s` is declared by more than one file of the set: `s-late.yaml` and `s.yaml`\n\
             checked: 3 protocols, 1 opening, 1 supplied kind, 1 finding\n"
        );
        assert!(checked([&parent, &producer]).findings[0].code() == "undeclared-child-outcome");
    }

    /// One file given twice under one name is that file once, not a duplicate; three files with
    /// one id are one finding naming the three, sorted; and an id of more than one file is still in
    /// the set, so an opening under it and a supplier of it are no finding of their own.
    #[test]
    fn a_file_named_twice_is_one_and_files_with_one_id_are_one_finding() {
        let parent = written(
            "p",
            "artifacts: {a: {}}\nevidence_kinds: {case_outcome: {}, k: {supplied_by: s}}\n\
             claims: {c: {true_when: {children: {in: [late], subject: a}}}}\n\
             actions: {open: {case_open: {protocol: s, subject: a}}}\n",
        );
        let producer = written(
            "s",
            "artifacts: {a: {}}\nevidence_kinds: {k: {}}\nactions: {make: {may_produce: [{evidence: k}]}}\n",
        );
        let once = check_set([
            ("s.yaml", &producer),
            ("p.yaml", &parent),
            ("s.yaml", &producer),
        ]);
        assert_eq!(
            once.to_string(),
            "undeclared-child-outcome: protocol `p` claim `c` names outcome `late` of a child case about `a`, which no protocol it opens about `a` declares (it opens `s`)\n\
             checked: 2 protocols, 1 opening, 1 supplied kind, 1 finding\n"
        );
        let three = check_set([
            ("c/s.yaml", &producer),
            ("a/s.yaml", &child("s", &["late"])),
            ("b/s.yaml", &child("s", &["early"])),
            ("p.yaml", &parent),
        ]);
        assert_eq!(
            three.to_string(),
            "duplicate-protocol-id: protocol `s` is declared by more than one file of the set: `a/s.yaml`, `b/s.yaml` and `c/s.yaml`\n\
             checked: 4 protocols, 1 opening, 1 supplied kind, 1 finding\n"
        );
        assert_eq!(three.findings[0].code(), "duplicate-protocol-id");
    }

    /// The supplier `s` of the kinds `k` and `l`, and a child `c` about `thing` that requires `k`
    /// at open and only reads `l`, with the outcome `done`.
    fn at_open_family() -> [Ir; 2] {
        [
            written(
                "s",
                "artifacts: {thing: {}}\nevidence_kinds: {k: {}, l: {}}\n\
                 actions: {make: {may_produce: [{evidence: k}, {evidence: l}]}}\n",
            ),
            written(
                "c",
                "artifacts: {thing: {}}\n\
                 evidence_kinds: {k: {supplied_by: s, required_at_open: true}, l: {supplied_by: s}}\n\
                 claims: {c.k: {true_when: {evidence: {kind: k}}}}\n\
                 outcomes: {done: {requires: {claim: c.k}}}\n",
            ),
        ]
    }

    /// The `unsupplied-at-open` lines of a set of `p`, written from `sections`, with the
    /// [`at_open_family`].
    fn unsupplied(sections: &str) -> Vec<String> {
        let [supplier, child] = at_open_family();
        let parent = written("p", sections);
        checked([&parent, &supplier, &child])
            .findings
            .iter()
            .filter(|finding| finding.code() == "unsupplied-at-open")
            .map(ToString::to_string)
            .collect()
    }

    /// A parent `p` that declares `k`, `other` and `case_outcome`, the input `mode`, the claims
    /// written and one action per `(id, precondition)` of `actions`, each opening `c` about
    /// `thing`.
    fn opener(claims: &str, actions: &[(&str, &str)]) -> String {
        let actions: String = actions
            .iter()
            .map(|(id, precondition)| {
                format!(
                    "  {id}: {{precondition: {precondition}, case_open: {{protocol: c, subject: thing}}}}\n"
                )
            })
            .collect();
        format!(
            "case: {{inputs: {{mode: {{values: [fast, slow]}}}}}}\n\
             artifacts: {{thing: {{}}}}\n\
             evidence_kinds: {{k: {{supplied_by: s}}, other: {{}}, case_outcome: {{}}}}\n\
             claims:\n{claims}actions:\n{actions}"
        )
    }

    /// The actions of `p` that [`unsupplied`] names, in finding order.
    fn unproved(claims: &str, actions: &[(&str, &str)]) -> Vec<String> {
        unsupplied(&opener(claims, actions))
            .iter()
            .map(|line| {
                line.strip_prefix("protocol `p` action `")
                    .and_then(|rest| rest.split('`').next())
                    .unwrap_or(line)
                    .to_owned()
            })
            .collect()
    }

    const CLAIMS: &str = "  has.k: {true_when: {evidence: {kind: k}}}\n  \
         k.approved: {true_when: {evidence: {kind: k, result: approved}}}\n  \
         has.other: {true_when: {evidence: {kind: other}}}\n  \
         both: {true_when: {all: [{evidence: {kind: k}}, {evidence: {kind: other}}]}}\n  \
         either: {true_when: {any: [{evidence: {kind: k, result: approved}}, {evidence: {kind: other}}]}}\n  \
         chained: {true_when: {claim: has.k}}\n";

    /// story:supplied-evidence-required-at-open: an evidence match of the kind implies a record of
    /// it at either value, so a claim test `is: true` or `is: false` implies one through the
    /// claim's `true_when`; `all` implies it when one member does, `any` when every member does;
    /// an order form naming the kind implies it. `not`, `is: unknown`, an input test, a `children`
    /// form, a match of another kind and the always-true precondition do not, and neither does an
    /// action's `runs_only_when`, which no evaluation reads.
    #[test]
    fn the_precondition_of_an_opening_action_implies_a_record_of_the_kind() {
        let found = unproved(
            CLAIMS,
            &[
                ("match", "{evidence: {kind: k}}"),
                ("match.result", "{evidence: {kind: k, result: approved}}"),
                ("claim.true", "{claim: has.k}"),
                ("claim.false", "{claim: k.approved, is: \"false\"}"),
                ("claim.chained", "{claim: chained}"),
                ("claim.unknown", "{claim: has.k, is: unknown}"),
                ("all.one", "{all: [{claim: has.other}, {claim: has.k}]}"),
                ("any.one", "{any: [{claim: has.other}, {claim: has.k}]}"),
                (
                    "any.every",
                    "{any: [{claim: has.k}, {evidence: {kind: k, result: approved}}]}",
                ),
                ("false.all", "{claim: both, is: \"false\"}"),
                ("false.any", "{claim: either, is: \"false\"}"),
                ("not", "{not: {claim: has.k, is: unknown}}"),
                (
                    "not.match",
                    "{not: {evidence: {kind: k, result: approved}}}",
                ),
                ("children", "{children: {in: [done], subject: thing}}"),
                ("input", "{input: mode, in: [fast]}"),
                ("order", "{first: {kind: k, result: approved}}"),
                ("order.before", "{before: {first: other, then: k}}"),
                ("order.other", "{first: {kind: other, result: x}}"),
                ("order.not", "{not: {first: {kind: k, result: approved}}}"),
                ("other", "{evidence: {kind: other}}"),
                ("always", "{all: []}"),
            ],
        );
        assert_eq!(
            found,
            [
                "always",
                "any.one",
                "children",
                "claim.unknown",
                "false.all",
                "input",
                "not",
                "not.match",
                "order.not",
                "order.other",
                "other",
            ]
        );
    }

    /// A guard of story:ordering-guards-provable-by-check is no part of the proof: the precondition
    /// is what decides whether the action is admissible.
    #[test]
    fn a_runs_only_when_guard_proves_nothing() {
        let sections = opener(CLAIMS, &[]).replace(
            "actions:\n",
            "actions:\n  guarded: {precondition: {claim: has.other}, runs_only_when: {claim: has.k}, \
             case_open: {protocol: c, subject: thing}}\n",
        );
        assert_eq!(
            unsupplied(&sections),
            [
                "protocol `p` action `guarded` opens a case under protocol `c`, which requires evidence kind `k` at open, and the action's precondition does not imply a record of it"
            ]
        );
    }

    /// An opener that does not declare the kind cannot hold a record of it: the finding says so.
    /// One finding per action and kind, a kind the child only reads (`l`) asks nothing.
    #[test]
    fn an_opener_that_declares_no_such_kind_is_a_finding() {
        assert_eq!(
            unsupplied(
                "artifacts: {thing: {}}\nevidence_kinds: {l: {}}\n\
                 claims: {has.l: {true_when: {evidence: {kind: l}}}}\n\
                 actions:\n  \
                 b: {precondition: {claim: has.l}, case_open: {protocol: c, subject: thing}}\n  \
                 a: {case_open: {protocol: c, subject: thing}}\n"
            ),
            [
                "protocol `p` action `a` opens a case under protocol `c`, which requires evidence kind `k` at open, and protocol `p` declares no evidence kind `k`",
                "protocol `p` action `b` opens a case under protocol `c`, which requires evidence kind `k` at open, and protocol `p` declares no evidence kind `k`",
            ]
        );
    }

    /// The finding comes after every other finding, by protocol, action and kind. A child outside
    /// the set, or of an id two files declare, requires nothing known, and a protocol nothing in
    /// the set opens is held to nothing.
    #[test]
    fn findings_come_last_and_only_a_known_child_requires_anything() {
        let [supplier, child] = at_open_family();
        let two = written(
            "c2",
            "artifacts: {thing: {}}\n\
             evidence_kinds: {k2: {supplied_by: s, required_at_open: true}, k1: {supplied_by: s, required_at_open: true}}\n",
        );
        let parent = written(
            "p",
            "artifacts: {thing: {}}\nevidence_kinds: {k: {supplied_by: ghost}}\n\
             actions:\n  \
             open.c: {case_open: {protocol: c, subject: thing}}\n  \
             open.c2: {case_open: {protocol: c2, subject: thing}}\n  \
             open.absent: {case_open: {protocol: absent, subject: thing}}\n",
        );
        let report = checked([&parent, &supplier, &child, &two]);
        let codes: Vec<&str> = report.findings.iter().map(SetFinding::code).collect();
        assert_eq!(
            codes,
            [
                "unknown-opened-protocol",
                "unknown-supplier",
                "unproduced-supplied-kind",
                "unproduced-supplied-kind",
                "unsupplied-at-open",
                "unsupplied-at-open",
                "unsupplied-at-open",
            ]
        );
        let at_open: Vec<String> = report.findings[4..]
            .iter()
            .map(|finding| {
                let line = finding.to_string();
                let (action, rest) = line
                    .strip_prefix("protocol `p` action `")
                    .and_then(|rest| rest.split_once('`'))
                    .expect("a finding of p");
                let kind = rest
                    .split("evidence kind `")
                    .nth(1)
                    .and_then(|rest| rest.split('`').next())
                    .expect("a kind");
                format!("{action} {kind}")
            })
            .collect();
        assert_eq!(at_open, ["open.c k", "open.c2 k1", "open.c2 k2"]);

        let duplicate = written("c", "artifacts: {thing: {}}\n");
        let pooled = check_set([
            ("p.yaml", &parent),
            ("s.yaml", &supplier),
            ("c.yaml", &child),
            ("c-other.yaml", &duplicate),
        ]);
        assert!(
            pooled
                .findings
                .iter()
                .all(|finding| finding.code() != "unsupplied-at-open"),
            "{pooled}"
        );
        let alone = checked([&supplier, &child]);
        assert!(alone.is_clean(), "{alone}");
        assert_eq!(
            alone.to_string(),
            "checked: 2 protocols, 0 openings, 2 supplied kinds, 0 findings\n"
        );
    }
}
