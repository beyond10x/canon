//! Deterministic validation of a `protocol/1` document.
//!
//! Validation resolves every reference and reports every problem it finds, in a stable order that
//! does not depend on the order the sections are written in:
//!
//! 1. the format;
//! 2. malformed identifiers: the protocol id, then the ids declared in artifacts, evidence kinds,
//!    claims, obligations, actions and outcomes, in that section order, then the capabilities and
//!    effect classes actions name, in action order, then the decisions outcomes require, in
//!    outcome order;
//! 3. duplicate identifiers, in the same section order;
//! 4. maximum ages that are not a whole number without leading zeros, followed by `s`, `m`, `h`
//!    or `d`, or that are but are too long to count in seconds (each with its own message), in the
//!    order the evidence kinds are written;
//! 5. unresolved references: those in claims first, then those in obligations (their discharge
//!    predicates, each followed by every evidence match it holds, which a discharge predicate may
//!    not: it tests only claim values), then those in actions, then those in outcomes;
//!    within a section, in the order its declarations are written, and within a declaration, in
//!    the order its references are written;
//! 6. cycles between claims.
//!
//! Within a section, entries come in the order they are written. Every message renders document
//! text on one line. The same document always yields the same problems in the same order.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::model::{
    ActionId, ClaimId, EvidenceKindId, FORMAT, ObligationId, OutcomeId, Predicate, Protocol,
    is_identifier, one_line,
};

/// What an identifier names: the protocol itself, a declaration section (named by the singular noun
/// of what it declares), a capability or effect class an action names, or a decision an outcome
/// requires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Section {
    Protocol,
    Artifact,
    EvidenceKind,
    Claim,
    Obligation,
    Action,
    Outcome,
    Capability,
    EffectClass,
    Decision,
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Section::Protocol => "protocol",
            Section::Artifact => "artifact",
            Section::EvidenceKind => "evidence kind",
            Section::Claim => "claim",
            Section::Obligation => "obligation",
            Section::Action => "action",
            Section::Outcome => "outcome",
            Section::Capability => "capability",
            Section::EffectClass => "effect class",
            Section::Decision => "decision",
        })
    }
}

/// The declaration a reference is written in.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Referrer {
    Claim(ClaimId),
    Obligation(ObligationId),
    Action(ActionId),
    Outcome(OutcomeId),
}

impl fmt::Display for Referrer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Referrer::Claim(id) => write!(f, "claim `{}`", one_line(id.as_str())),
            Referrer::Obligation(id) => write!(f, "obligation `{}`", one_line(id.as_str())),
            Referrer::Action(id) => write!(f, "action `{}`", one_line(id.as_str())),
            Referrer::Outcome(id) => write!(f, "outcome `{}`", one_line(id.as_str())),
        }
    }
}

/// One reason a protocol document is invalid.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Problem {
    /// The document's `format` is not `protocol/1`.
    UnsupportedFormat { found: String },
    /// An identifier is empty or contains whitespace or a control character.
    InvalidIdentifier { section: Section, id: String },
    /// An identifier is declared more than once in one section.
    DuplicateIdentifier { section: Section, id: String },
    /// A predicate tests a claim that is not declared.
    UndeclaredClaim { referrer: Referrer, claim: ClaimId },
    /// A predicate matches, or an action may produce, an evidence kind that is not declared.
    UndeclaredEvidenceKind {
        referrer: Referrer,
        kind: EvidenceKindId,
    },
    /// An evidence kind's `max_age` is not a whole number without leading zeros, followed by `s`,
    /// `m`, `h` or `d`, or is too long to count in seconds.
    InvalidMaxAge {
        kind: EvidenceKindId,
        max_age: String,
        /// Well formed, but too long to count in seconds.
        too_long: bool,
    },
    /// A discharge predicate tests evidence; it may test only claim values. `path` locates the
    /// evidence match inside the predicate, as `discharged_when.all[1].evidence`.
    EvidenceInDischarge {
        obligation: ObligationId,
        kind: EvidenceKindId,
        path: String,
    },
    /// Claims whose predicates test each other in a cycle; the first claim is repeated at the end.
    ClaimCycle { claims: Vec<ClaimId> },
}

impl Problem {
    /// A stable machine-readable code for the kind of problem.
    pub fn code(&self) -> &'static str {
        match self {
            Problem::UnsupportedFormat { .. } => "unsupported-format",
            Problem::InvalidIdentifier { .. } => "invalid-identifier",
            Problem::DuplicateIdentifier { .. } => "duplicate-identifier",
            Problem::UndeclaredClaim { .. } => "undeclared-claim",
            Problem::UndeclaredEvidenceKind { .. } => "undeclared-evidence-kind",
            Problem::InvalidMaxAge { .. } => "invalid-max-age",
            Problem::EvidenceInDischarge { .. } => "evidence-in-discharge",
            Problem::ClaimCycle { .. } => "claim-cycle",
        }
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Problem::UnsupportedFormat { found } => {
                write!(f, "format is `{}`, expected `{FORMAT}`", one_line(found))
            }
            Problem::InvalidIdentifier { section, id } => write!(
                f,
                "{section} identifier `{}` is empty or contains whitespace or a control character",
                one_line(id)
            ),
            Problem::DuplicateIdentifier { section, id } => {
                write!(f, "{section} `{}` is declared more than once", one_line(id))
            }
            Problem::UndeclaredClaim { referrer, claim } => write!(
                f,
                "{referrer} references claim `{}`, which is not declared",
                one_line(claim.as_str())
            ),
            Problem::UndeclaredEvidenceKind { referrer, kind } => write!(
                f,
                "{referrer} references evidence kind `{}`, which is not declared",
                one_line(kind.as_str())
            ),
            Problem::InvalidMaxAge {
                kind,
                max_age,
                too_long,
            } => write!(
                f,
                "evidence kind `{}` has max_age `{}`, which {}",
                one_line(kind.as_str()),
                one_line(max_age),
                if *too_long {
                    "is too long to count in seconds"
                } else {
                    "is not a whole number without leading zeros, followed by s, m, h or d"
                }
            ),
            Problem::EvidenceInDischarge {
                obligation,
                kind,
                path,
            } => write!(
                f,
                "obligation `{}` tests evidence kind `{}` at `{path}`; a discharge predicate tests only claim values",
                one_line(obligation.as_str()),
                one_line(kind.as_str())
            ),
            Problem::ClaimCycle { claims } => {
                let path: Vec<String> = claims.iter().map(|id| one_line(id.as_str())).collect();
                write!(
                    f,
                    "claims test each other in a cycle: {}",
                    path.join(" -> ")
                )
            }
        }
    }
}

/// Validates a parsed protocol. `Ok` when it is valid; otherwise every problem, in a stable order.
pub fn validate(protocol: &Protocol) -> Result<(), Vec<Problem>> {
    let mut problems = Vec::new();

    if protocol.format != FORMAT {
        problems.push(Problem::UnsupportedFormat {
            found: protocol.format.clone(),
        });
    }

    let declared: [(Section, Vec<&str>); 7] = [
        (Section::Protocol, vec![protocol.protocol.id.as_str()]),
        (Section::Artifact, strs(protocol.artifacts.ids())),
        (Section::EvidenceKind, strs(protocol.evidence_kinds.ids())),
        (Section::Claim, strs(protocol.claims.ids())),
        (Section::Obligation, strs(protocol.obligations.ids())),
        (Section::Action, strs(protocol.actions.ids())),
        (Section::Outcome, strs(protocol.outcomes.ids())),
    ];
    let mut named = Vec::new();
    for (_, action) in protocol.actions.iter() {
        for requirement in &action.requires {
            named.push((Section::Capability, requirement.capability.as_str()));
        }
        if let Some(effect) = &action.effect {
            named.push((Section::EffectClass, effect.as_str()));
        }
    }
    for (_, outcome) in protocol.outcomes.iter() {
        if let Some(decision) = outcome.requires.decision() {
            named.push((Section::Decision, decision.as_str()));
        }
    }
    let every_identifier = declared
        .iter()
        .flat_map(|(section, ids)| ids.iter().map(move |id| (*section, *id)))
        .chain(named);
    for (section, id) in every_identifier {
        if !is_identifier(id) {
            problems.push(Problem::InvalidIdentifier {
                section,
                id: id.to_owned(),
            });
        }
    }

    for (section, ids) in &declared {
        duplicates(*section, ids, &mut problems);
    }

    for (id, kind) in protocol.evidence_kinds.iter() {
        if let Some(max_age) = &kind.max_age
            && max_age.seconds().is_none()
        {
            problems.push(Problem::InvalidMaxAge {
                kind: id.clone(),
                max_age: max_age.as_str().to_owned(),
                too_long: max_age.is_well_formed(),
            });
        }
    }

    for (id, claim) in protocol.claims.iter() {
        references(
            protocol,
            &Referrer::Claim(id.clone()),
            &claim.true_when,
            &mut problems,
        );
    }
    for (id, obligation) in protocol.obligations.iter() {
        references(
            protocol,
            &Referrer::Obligation(id.clone()),
            &obligation.discharged_when,
            &mut problems,
        );
        evidence_in_discharge(id, &obligation.discharged_when, &mut problems);
    }
    for (id, action) in protocol.actions.iter() {
        let referrer = Referrer::Action(id.clone());
        if let Some(precondition) = &action.precondition {
            references(protocol, &referrer, precondition, &mut problems);
        }
        for production in &action.may_produce {
            if !protocol.evidence_kinds.contains(&production.evidence) {
                problems.push(Problem::UndeclaredEvidenceKind {
                    referrer: referrer.clone(),
                    kind: production.evidence.clone(),
                });
            }
        }
    }
    for (id, outcome) in protocol.outcomes.iter() {
        if let Some(requires) = outcome.requires.predicate() {
            references(
                protocol,
                &Referrer::Outcome(id.clone()),
                requires,
                &mut problems,
            );
        }
    }

    claim_cycles(protocol, &mut problems);

    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

/// Reports every evidence match in an obligation's discharge predicate, in the order written,
/// with its path from `discharged_when`.
fn evidence_in_discharge(
    obligation: &ObligationId,
    discharged_when: &Predicate,
    problems: &mut Vec<Problem>,
) {
    let mut pending = vec![(discharged_when, "discharged_when".to_owned())];
    while let Some((predicate, path)) = pending.pop() {
        match predicate {
            Predicate::All(members) | Predicate::Any(members) => {
                let form = if matches!(predicate, Predicate::All(_)) {
                    "all"
                } else {
                    "any"
                };
                for (index, member) in members.iter().enumerate().rev() {
                    pending.push((member, format!("{path}.{form}[{index}]")));
                }
            }
            Predicate::Not(inner) => pending.push((inner, format!("{path}.not"))),
            Predicate::Evidence(matching) => problems.push(Problem::EvidenceInDischarge {
                obligation: obligation.clone(),
                kind: matching.kind.clone(),
                path: format!("{path}.evidence"),
            }),
            Predicate::Claim(_) => {}
        }
    }
}

fn strs<'a, T: AsRef<str> + 'a>(ids: impl Iterator<Item = &'a T>) -> Vec<&'a str> {
    ids.map(AsRef::as_ref).collect()
}

/// Reports each identifier once for every declaration after its first.
fn duplicates(section: Section, ids: &[&str], problems: &mut Vec<Problem>) {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(*id) {
            problems.push(Problem::DuplicateIdentifier {
                section,
                id: (*id).to_owned(),
            });
        }
    }
}

/// Reports every claim and evidence kind the predicate references that is not declared, in the
/// order the predicate is written.
fn references(
    protocol: &Protocol,
    referrer: &Referrer,
    predicate: &Predicate,
    problems: &mut Vec<Problem>,
) {
    let mut unresolved = Vec::new();
    predicate.visit(&mut |node| match node {
        Predicate::Claim(test) if !protocol.claims.contains(&test.claim) => {
            unresolved.push(Problem::UndeclaredClaim {
                referrer: referrer.clone(),
                claim: test.claim.clone(),
            });
        }
        Predicate::Evidence(matching) if !protocol.evidence_kinds.contains(&matching.kind) => {
            unresolved.push(Problem::UndeclaredEvidenceKind {
                referrer: referrer.clone(),
                kind: matching.kind.clone(),
            });
        }
        _ => {}
    });
    problems.extend(unresolved);
}

/// Reports cycles among claims whose predicates test other claims: at least one cycle for every
/// set of claims that test each other in a cycle, not every distinct cycle through that set (for
/// `a` testing `b` and `c`, `b` testing `c`, `c` testing `a`, it reports `a -> b -> c -> a` and not
/// `a -> c -> a`). Claims are visited in declaration order and their references in the order
/// written, so the cycles reported are the same on every run.
fn claim_cycles(protocol: &Protocol, problems: &mut Vec<Problem>) {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Mark {
        Unvisited,
        OnPath,
        Done,
    }

    fn visit<'a>(
        protocol: &'a Protocol,
        claim: &'a ClaimId,
        marks: &mut BTreeMap<&'a ClaimId, Mark>,
        path: &mut Vec<&'a ClaimId>,
        problems: &mut Vec<Problem>,
    ) {
        marks.insert(claim, Mark::OnPath);
        path.push(claim);
        if let Some(declared) = protocol.claims.get(claim) {
            for next in declared.true_when.claim_references() {
                match marks.get(next).copied().unwrap_or(Mark::Unvisited) {
                    Mark::Unvisited if protocol.claims.contains(next) => {
                        visit(protocol, next, marks, path, problems);
                    }
                    Mark::OnPath => {
                        let start = path
                            .iter()
                            .position(|on_path| *on_path == next)
                            .expect("a claim marked on the path is on the path");
                        let mut cycle: Vec<ClaimId> =
                            path[start..].iter().map(|id| (*id).clone()).collect();
                        cycle.push(next.clone());
                        problems.push(Problem::ClaimCycle { claims: cycle });
                    }
                    Mark::Unvisited | Mark::Done => {}
                }
            }
        }
        path.pop();
        marks.insert(claim, Mark::Done);
    }

    let mut marks = BTreeMap::new();
    for claim in protocol.claims.ids() {
        if !marks.contains_key(claim) {
            visit(protocol, claim, &mut marks, &mut Vec::new(), problems);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn problems(source: &str) -> Vec<String> {
        let protocol = crate::model::parse(source).expect("parses");
        match validate(&protocol) {
            Ok(()) => Vec::new(),
            Err(problems) => problems
                .iter()
                .map(|problem| format!("{}: {problem}", problem.code()))
                .collect(),
        }
    }

    /// A discharge predicate tests claim values only: each evidence match in it is refused, in the
    /// order written, naming the obligation and where the match is; after the obligation's
    /// unresolved references and before the actions'.
    #[test]
    fn evidence_in_a_discharge_predicate_is_refused_with_its_path() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
            evidence_kinds: {k: {}}\nclaims: {c: {true_when: {evidence: {kind: k}}}}\n\
            actions: {a: {precondition: {claim: y}}}\n\
            obligations:\n  o:\n    discharged_when:\n      all:\n        - {claim: c}\n        - {not: {evidence: {kind: k}}}\n        - {any: [{claim: x}, {evidence: {kind: z}}]}\n";
        assert_eq!(
            problems(source),
            [
                "undeclared-claim: obligation `o` references claim `x`, which is not declared",
                "undeclared-evidence-kind: obligation `o` references evidence kind `z`, which is not declared",
                "evidence-in-discharge: obligation `o` tests evidence kind `k` at `discharged_when.all[1].not.evidence`; a discharge predicate tests only claim values",
                "evidence-in-discharge: obligation `o` tests evidence kind `z` at `discharged_when.all[2].any[1].evidence`; a discharge predicate tests only claim values",
                "undeclared-claim: action `a` references claim `y`, which is not declared",
            ]
        );
        assert_eq!(
            problems(
                &source
                    .replace("{not: {evidence: {kind: k}}}", "{claim: c}")
                    .replace("{evidence: {kind: z}}", "{claim: c}")
                    .replace("{claim: x}", "{claim: c}")
                    .replace("{claim: y}", "{claim: c}")
            ),
            Vec::<String>::new()
        );
    }

    /// Each malformed maximum age is refused, in the order the evidence kinds are written, after
    /// the duplicate identifiers and before the unresolved references; a well-formed one is not.
    #[test]
    fn a_malformed_maximum_age_is_refused_naming_its_kind() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
            evidence_kinds: {z: {max_age: 5 min}, a: {max_age: 1h}, k: {max_age: 05m}, k: {}}\n\
            claims: {c: {true_when: {evidence: {kind: x}}}}\n";
        assert_eq!(
            problems(source),
            [
                "duplicate-identifier: evidence kind `k` is declared more than once",
                "invalid-max-age: evidence kind `z` has max_age `5 min`, which is not a whole number without leading zeros, followed by s, m, h or d",
                "invalid-max-age: evidence kind `k` has max_age `05m`, which is not a whole number without leading zeros, followed by s, m, h or d",
                "undeclared-evidence-kind: claim `c` references evidence kind `x`, which is not declared",
            ]
        );
        assert_eq!(
            problems(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
                 evidence_kinds: {k: {max_age: 0s}, l: {max_age: 30d}}\n"
            ),
            Vec::<String>::new()
        );
        assert_eq!(
            problems(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
                 evidence_kinds: {k: {max_age: 106751991167301d}}\n"
            ),
            [
                "invalid-max-age: evidence kind `k` has max_age `106751991167301d`, which is too long to count in seconds"
            ]
        );
    }

    /// A decision an outcome requires is an identifier, reported after every name actions give;
    /// it references nothing, so it is never undeclared. Written beside a predicate form, or inside
    /// one, it does not parse.
    #[test]
    fn a_decision_requirement_is_an_identifier_and_stands_alone() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
            actions: {act: {effect: 'two words'}}\n\
            outcomes: {o: {requires: {decision: 'a b'}}, q: {requires: {decision: fine}}}\n";
        assert_eq!(
            problems(source),
            [
                "invalid-identifier: effect class identifier `two words` is empty or contains whitespace or a control character",
                "invalid-identifier: decision identifier `a b` is empty or contains whitespace or a control character",
            ]
        );
        for requires in [
            "{decision: d, claim: c}",
            "{all: [{decision: d}]}",
            "{not: {decision: d}}",
        ] {
            let source = format!(
                "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nclaims: {{c: {{true_when: {{all: []}}}}}}\noutcomes: {{o: {{requires: {requires}}}}}\n"
            );
            assert!(crate::model::parse(&source).is_err(), "{requires} parses");
        }
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
            claims: {c: {true_when: {decision: d}}}\n";
        assert!(
            crate::model::parse(source).is_err(),
            "a claim names a decision"
        );
    }
}
