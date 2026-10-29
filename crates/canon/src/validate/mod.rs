//! Deterministic validation of a `protocol/1` document.
//!
//! Validation resolves every reference and reports every problem it finds, in a stable order that
//! does not depend on the order the sections are written in:
//!
//! 1. the format;
//! 2. malformed identifiers: the protocol id, then the ids declared in artifacts, evidence kinds,
//!    claims, obligations, actions, outcomes, invalidation rules, case inputs and independence
//!    dimensions, in that section order, then the capabilities and effect classes actions name, in action order, then the
//!    decisions outcomes require, in outcome order, then the values case inputs declare, in input
//!    order;
//! 3. duplicate identifiers, in the same section order;
//! 4. maximum ages that are not a whole number without leading zeros, followed by `s`, `m`, `h`
//!    or `d`, or that are but are too long to count in seconds (each with its own message), in the
//!    order the evidence kinds are written;
//! 5. case inputs that declare no value (`empty-input-values`, naming the input) or list a value
//!    more than once (`duplicate-input-value`, naming the input and the value, once for every
//!    listing after its first), in the order the inputs are written;
//! 6. action argument schemas outside the argument-schema dialect (`action-arguments`; the dialect,
//!    and the order within one schema, are stated with it), each naming the action, the JSON
//!    Pointer of the keyword inside the schema and why it is refused, one per offending keyword,
//!    in the order the actions are written;
//! 7. unresolved references and malformed input tests and order forms (a claim, an evidence kind,
//!    the artifact an evidence match names as its subject, which is reported after the kind it is
//!    written beside, then the evidence match's `independent_on`: an empty list,
//!    `empty-independent-on`, or a dimension it lists more than once, `duplicate-independent-on`,
//!    once for every listing after its first, then each dimension it lists that is not declared,
//!    `undeclared-dimension`, once, in the order listed, every one naming the declaration the
//!    match is written in; for an order form, each kind it names, `first` (or `last`) before
//!    `then`, then its subject, then a `before` whose `first` and `then` name one kind,
//!    `self-ordered-evidence`, naming it (an `after` or a `since` may name one kind as both: a
//!    record of a kind may follow the first, or the latest, record of that kind);
//!    for an input test, first an empty `in` list, `empty-input-test`, or a value it lists more
//!    than once, `duplicate-input-test-value`, once for every listing after its first, then an
//!    input it names that is not declared, `undeclared-input`, or a value it lists that its input
//!    does not declare, `undeclared-input-value`, each value once, in the order listed, every one
//!    naming the declaration the test is written in and the input): those in claims first, then those
//!    in obligations (each obligation's discharge predicate, followed by every evidence match and
//!    every order form it holds, which a discharge predicate may not: it tests only claim values,
//!    each kind an order form names once; then its applicability predicate, followed likewise by
//!    every evidence match and order form it holds, `evidence-in-applicability`: it tests only
//!    claim values and case inputs), then those in
//!    actions, then those in outcomes, then those in invalidation rules (the upstream artifact,
//!    then each claim the rule invalidates); within a section, in the order its declarations are
//!    written, and within a declaration, in the order its references are written;
//! 8. invalidation rules that name a claim reaching no evidence match, in its own predicate or in a
//!    claim it tests through any number of claim references (`inert-invalidation`), in the order
//!    the rules and their claims are written. An order form is not an evidence match here: it reads
//!    the history, which an invalidation rule does not change;
//! 9. cycles between claims.
//!
//! Within a section, entries come in the order they are written. Every message renders document
//! text on one line. The same document always yields the same problems in the same order.

pub(crate) mod arguments;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::model::{
    ActionId, ArtifactId, ClaimId, EvidenceKindId, FORMAT, IndependenceDimensionId, InputId,
    InputValue, InvalidationRuleId, ObligationId, OutcomeId, Predicate, Protocol, is_identifier,
    one_line,
};

/// What an identifier names: the protocol itself, a declaration section (named by the singular noun
/// of what it declares), a capability or effect class an action names, a decision an outcome
/// requires, or a value a case input declares.
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
    InvalidationRule,
    Input,
    InputValue,
    IndependenceDimension,
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
            Section::InvalidationRule => "invalidation rule",
            Section::Capability => "capability",
            Section::EffectClass => "effect class",
            Section::Decision => "decision",
            Section::Input => "input",
            Section::InputValue => "input value",
            Section::IndependenceDimension => "independence dimension",
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
    InvalidationRule(InvalidationRuleId),
}

impl fmt::Display for Referrer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Referrer::Claim(id) => write!(f, "claim `{}`", one_line(id.as_str())),
            Referrer::Obligation(id) => write!(f, "obligation `{}`", one_line(id.as_str())),
            Referrer::Action(id) => write!(f, "action `{}`", one_line(id.as_str())),
            Referrer::Outcome(id) => write!(f, "outcome `{}`", one_line(id.as_str())),
            Referrer::InvalidationRule(id) => {
                write!(f, "invalidation rule `{}`", one_line(id.as_str()))
            }
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
    /// A predicate tests, or an invalidation rule invalidates, a claim that is not declared.
    UndeclaredClaim { referrer: Referrer, claim: ClaimId },
    /// A `before` order form names one evidence kind as both its `first` and its `then`: no
    /// record is observed strictly before itself.
    SelfOrderedEvidence {
        referrer: Referrer,
        kind: EvidenceKindId,
    },
    /// A predicate matches or orders, or an action may produce, an evidence kind that is not
    /// declared.
    UndeclaredEvidenceKind {
        referrer: Referrer,
        kind: EvidenceKindId,
    },
    /// An evidence match or an order form names as its subject, or an invalidation rule as its
    /// upstream artifact, an artifact that is not declared.
    UndeclaredArtifact {
        referrer: Referrer,
        artifact: ArtifactId,
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
    /// evidence match or order form inside the predicate, as `discharged_when.all[1].evidence` or
    /// `discharged_when.first`.
    EvidenceInDischarge {
        obligation: ObligationId,
        kind: EvidenceKindId,
        path: String,
    },
    /// An applicability predicate tests evidence; it may test only claim values and case inputs.
    /// `path` locates the evidence match or order form inside the predicate, as
    /// `applicable_when.not.evidence` or `applicable_when.before`.
    EvidenceInApplicability {
        obligation: ObligationId,
        kind: EvidenceKindId,
        path: String,
    },
    /// A case input declares no value, so no case could give it one.
    EmptyInputValues { input: InputId },
    /// A case input lists a value more than once.
    DuplicateInputValue { input: InputId, value: InputValue },
    /// An input test lists no value, so it is false in every case.
    EmptyInputTest { referrer: Referrer, input: InputId },
    /// An input test lists a value more than once.
    DuplicateInputTestValue {
        referrer: Referrer,
        input: InputId,
        value: InputValue,
    },
    /// An input test names an input that is not declared.
    UndeclaredInput { referrer: Referrer, input: InputId },
    /// An input test lists a value its input does not declare.
    UndeclaredInputValue {
        referrer: Referrer,
        input: InputId,
        value: InputValue,
    },
    /// An evidence match demands independence on no dimension: `independent_on` is empty.
    EmptyIndependentOn {
        referrer: Referrer,
        kind: EvidenceKindId,
    },
    /// An evidence match lists a dimension in `independent_on` more than once.
    DuplicateIndependentOn {
        referrer: Referrer,
        kind: EvidenceKindId,
        dimension: IndependenceDimensionId,
    },
    /// An evidence match demands independence on a dimension that is not declared.
    UndeclaredDimension {
        referrer: Referrer,
        dimension: IndependenceDimensionId,
    },
    /// An invalidation rule names a claim that reaches no evidence match, in its own predicate or
    /// in a claim it tests, so no record could be kept from it.
    InertInvalidation {
        rule: InvalidationRuleId,
        claim: ClaimId,
    },
    /// Claims whose predicates test each other in a cycle; the first claim is repeated at the end.
    ClaimCycle { claims: Vec<ClaimId> },
    /// An action's argument schema is outside the argument-schema dialect: `pointer` is the JSON
    /// Pointer of the offending keyword inside the schema (`""` for the schema itself) and `reason`
    /// why it is refused.
    ActionArguments {
        action: ActionId,
        pointer: String,
        reason: String,
    },
}

impl Problem {
    /// A stable machine-readable code for the kind of problem.
    pub fn code(&self) -> &'static str {
        match self {
            Problem::UnsupportedFormat { .. } => "unsupported-format",
            Problem::InvalidIdentifier { .. } => "invalid-identifier",
            Problem::DuplicateIdentifier { .. } => "duplicate-identifier",
            Problem::UndeclaredClaim { .. } => "undeclared-claim",
            Problem::SelfOrderedEvidence { .. } => "self-ordered-evidence",
            Problem::UndeclaredEvidenceKind { .. } => "undeclared-evidence-kind",
            Problem::UndeclaredArtifact { .. } => "undeclared-artifact",
            Problem::InvalidMaxAge { .. } => "invalid-max-age",
            Problem::EvidenceInDischarge { .. } => "evidence-in-discharge",
            Problem::EvidenceInApplicability { .. } => "evidence-in-applicability",
            Problem::EmptyInputValues { .. } => "empty-input-values",
            Problem::DuplicateInputValue { .. } => "duplicate-input-value",
            Problem::EmptyInputTest { .. } => "empty-input-test",
            Problem::DuplicateInputTestValue { .. } => "duplicate-input-test-value",
            Problem::UndeclaredInput { .. } => "undeclared-input",
            Problem::UndeclaredInputValue { .. } => "undeclared-input-value",
            Problem::EmptyIndependentOn { .. } => "empty-independent-on",
            Problem::DuplicateIndependentOn { .. } => "duplicate-independent-on",
            Problem::UndeclaredDimension { .. } => "undeclared-dimension",
            Problem::InertInvalidation { .. } => "inert-invalidation",
            Problem::ClaimCycle { .. } => "claim-cycle",
            Problem::ActionArguments { .. } => "action-arguments",
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
            Problem::SelfOrderedEvidence { referrer, kind } => write!(
                f,
                "{referrer} orders evidence kind `{}` before itself",
                one_line(kind.as_str())
            ),
            Problem::UndeclaredEvidenceKind { referrer, kind } => write!(
                f,
                "{referrer} references evidence kind `{}`, which is not declared",
                one_line(kind.as_str())
            ),
            Problem::UndeclaredArtifact { referrer, artifact } => write!(
                f,
                "{referrer} references artifact `{}`, which is not declared",
                one_line(artifact.as_str())
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
            Problem::EvidenceInApplicability {
                obligation,
                kind,
                path,
            } => write!(
                f,
                "obligation `{}` tests evidence kind `{}` at `{path}`; an applicability predicate tests only claim values and case inputs",
                one_line(obligation.as_str()),
                one_line(kind.as_str())
            ),
            Problem::EmptyInputValues { input } => {
                write!(f, "input `{}` declares no value", one_line(input.as_str()))
            }
            Problem::DuplicateInputValue { input, value } => write!(
                f,
                "input `{}` lists value `{}` more than once",
                one_line(input.as_str()),
                one_line(value.as_str())
            ),
            Problem::EmptyInputTest { referrer, input } => write!(
                f,
                "{referrer} tests input `{}` for no value",
                one_line(input.as_str())
            ),
            Problem::DuplicateInputTestValue {
                referrer,
                input,
                value,
            } => write!(
                f,
                "{referrer} tests input `{}` for value `{}` more than once",
                one_line(input.as_str()),
                one_line(value.as_str())
            ),
            Problem::UndeclaredInput { referrer, input } => write!(
                f,
                "{referrer} references input `{}`, which is not declared",
                one_line(input.as_str())
            ),
            Problem::UndeclaredInputValue {
                referrer,
                input,
                value,
            } => write!(
                f,
                "{referrer} tests input `{}` for value `{}`, which the input does not declare",
                one_line(input.as_str()),
                one_line(value.as_str())
            ),
            Problem::EmptyIndependentOn { referrer, kind } => write!(
                f,
                "{referrer} demands independence on no dimension of evidence kind `{}`",
                one_line(kind.as_str())
            ),
            Problem::DuplicateIndependentOn {
                referrer,
                kind,
                dimension,
            } => write!(
                f,
                "{referrer} demands independence on dimension `{}` more than once of evidence kind `{}`",
                one_line(dimension.as_str()),
                one_line(kind.as_str())
            ),
            Problem::UndeclaredDimension {
                referrer,
                dimension,
            } => write!(
                f,
                "{referrer} references independence dimension `{}`, which is not declared",
                one_line(dimension.as_str())
            ),
            Problem::InertInvalidation { rule, claim } => write!(
                f,
                "invalidation rule `{}` invalidates claim `{}`, which reaches no evidence match",
                one_line(rule.as_str()),
                one_line(claim.as_str())
            ),
            Problem::ClaimCycle { claims } => {
                let path: Vec<String> = claims.iter().map(|id| one_line(id.as_str())).collect();
                write!(
                    f,
                    "claims test each other in a cycle: {}",
                    path.join(" -> ")
                )
            }
            Problem::ActionArguments {
                action,
                pointer,
                reason,
            } => {
                write!(f, "action `{}` arguments", one_line(action.as_str()))?;
                if !pointer.is_empty() {
                    write!(f, " at `{}`", one_line(pointer))?;
                }
                write!(f, ": {}", one_line(reason))
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

    let declared: [(Section, Vec<&str>); 10] = [
        (Section::Protocol, vec![protocol.protocol.id.as_str()]),
        (Section::Artifact, strs(protocol.artifacts.ids())),
        (Section::EvidenceKind, strs(protocol.evidence_kinds.ids())),
        (Section::Claim, strs(protocol.claims.ids())),
        (Section::Obligation, strs(protocol.obligations.ids())),
        (Section::Action, strs(protocol.actions.ids())),
        (Section::Outcome, strs(protocol.outcomes.ids())),
        (Section::InvalidationRule, strs(protocol.invalidation.ids())),
        (
            Section::Input,
            protocol.inputs().map(|(id, _)| id.as_str()).collect(),
        ),
        (
            Section::IndependenceDimension,
            strs(protocol.independence_dimensions.ids()),
        ),
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
    for (_, input) in protocol.inputs() {
        for value in &input.values {
            named.push((Section::InputValue, value.as_str()));
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

    for (id, input) in protocol.inputs() {
        if input.values.is_empty() {
            problems.push(Problem::EmptyInputValues { input: id.clone() });
        }
        let mut seen = BTreeSet::new();
        for value in &input.values {
            if !seen.insert(value) {
                problems.push(Problem::DuplicateInputValue {
                    input: id.clone(),
                    value: value.clone(),
                });
            }
        }
    }

    for (id, action) in protocol.actions.iter() {
        if let Some(schema) = &action.arguments {
            for (pointer, reason) in arguments::check(schema) {
                problems.push(Problem::ActionArguments {
                    action: id.clone(),
                    pointer,
                    reason,
                });
            }
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
        for (kind, path) in evidence_at("discharged_when", &obligation.discharged_when) {
            problems.push(Problem::EvidenceInDischarge {
                obligation: id.clone(),
                kind,
                path,
            });
        }
        if let Some(applicable_when) = &obligation.applicable_when {
            references(
                protocol,
                &Referrer::Obligation(id.clone()),
                applicable_when,
                &mut problems,
            );
            for (kind, path) in evidence_at("applicable_when", applicable_when) {
                problems.push(Problem::EvidenceInApplicability {
                    obligation: id.clone(),
                    kind,
                    path,
                });
            }
        }
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
    for (id, rule) in protocol.invalidation.iter() {
        let referrer = Referrer::InvalidationRule(id.clone());
        if !protocol.artifacts.contains(&rule.upstream) {
            problems.push(Problem::UndeclaredArtifact {
                referrer: referrer.clone(),
                artifact: rule.upstream.clone(),
            });
        }
        for claim in &rule.invalidates {
            if !protocol.claims.contains(claim) {
                problems.push(Problem::UndeclaredClaim {
                    referrer: referrer.clone(),
                    claim: claim.clone(),
                });
            }
        }
    }

    for (id, rule) in protocol.invalidation.iter() {
        for claim in &rule.invalidates {
            if protocol.claims.contains(claim) && !reaches_evidence(protocol, claim) {
                problems.push(Problem::InertInvalidation {
                    rule: id.clone(),
                    claim: claim.clone(),
                });
            }
        }
    }

    claim_cycles(protocol, &mut problems);

    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

/// Whether `claim` reaches an evidence match: in its own predicate, or in a claim it tests through
/// any number of claim references. Each claim is visited once, so a cycle ends.
fn reaches_evidence(protocol: &Protocol, claim: &ClaimId) -> bool {
    let mut visited = BTreeSet::new();
    let mut pending = vec![claim];
    while let Some(next) = pending.pop() {
        if !visited.insert(next) {
            continue;
        }
        let Some(declared) = protocol.claims.get(next) else {
            continue;
        };
        let mut found = false;
        declared.true_when.visit(&mut |node| match node {
            Predicate::Evidence(_) => found = true,
            Predicate::Claim(test) => pending.push(&test.claim),
            _ => {}
        });
        if found {
            return true;
        }
    }
    false
}

/// Every evidence match and order form in an obligation's discharge or applicability predicate, in
/// the order written, each with its kind and its path from `root` (`discharged_when`,
/// `applicable_when`); a `before`, an `after` or a `since` gives each kind it names once, `first`
/// (or `last`) before `then`.
fn evidence_at(root: &str, predicate: &Predicate) -> Vec<(EvidenceKindId, String)> {
    let mut found = Vec::new();
    let mut pending = vec![(predicate, root.to_owned())];
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
            Predicate::Evidence(matching) => {
                found.push((matching.kind.clone(), format!("{path}.evidence")));
            }
            Predicate::First(form) => {
                found.push((form.kind.clone(), format!("{path}.first")));
            }
            Predicate::Before(form) => {
                found.push((form.first.clone(), format!("{path}.before")));
                if form.then != form.first {
                    found.push((form.then.clone(), format!("{path}.before")));
                }
            }
            Predicate::After(form) => {
                found.push((form.first.clone(), format!("{path}.after")));
                if form.then.kind != form.first {
                    found.push((form.then.kind.clone(), format!("{path}.after")));
                }
            }
            Predicate::Since(form) => {
                found.push((form.last.clone(), format!("{path}.since")));
                if form.then.kind != form.last {
                    found.push((form.then.kind.clone(), format!("{path}.since")));
                }
            }
            Predicate::Claim(_) | Predicate::Input(_) => {}
        }
    }
    found
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

/// Reports every claim, evidence kind and evidence subject the predicate references that is not
/// declared, every input test that lists no value, a value twice, an undeclared input or a
/// value its input does not declare, and every `before` that orders a kind before itself, in the
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
        Predicate::Evidence(matching) => {
            if !protocol.evidence_kinds.contains(&matching.kind) {
                unresolved.push(Problem::UndeclaredEvidenceKind {
                    referrer: referrer.clone(),
                    kind: matching.kind.clone(),
                });
            }
            if let Some(subject) = &matching.subject
                && !protocol.artifacts.contains(subject)
            {
                unresolved.push(Problem::UndeclaredArtifact {
                    referrer: referrer.clone(),
                    artifact: subject.clone(),
                });
            }
            if let Some(dimensions) = &matching.independent_on {
                if dimensions.is_empty() {
                    unresolved.push(Problem::EmptyIndependentOn {
                        referrer: referrer.clone(),
                        kind: matching.kind.clone(),
                    });
                }
                // Each dimension at its first listing, in the order written.
                let mut listed = Vec::new();
                for dimension in dimensions {
                    if listed.contains(&dimension) {
                        unresolved.push(Problem::DuplicateIndependentOn {
                            referrer: referrer.clone(),
                            kind: matching.kind.clone(),
                            dimension: dimension.clone(),
                        });
                    } else {
                        listed.push(dimension);
                    }
                }
                for dimension in listed {
                    if !protocol.independence_dimensions.contains(dimension) {
                        unresolved.push(Problem::UndeclaredDimension {
                            referrer: referrer.clone(),
                            dimension: dimension.clone(),
                        });
                    }
                }
            }
        }
        Predicate::First(_) | Predicate::Before(_) | Predicate::After(_) | Predicate::Since(_) => {
            let reads = node.order_reads();
            let mut kinds: Vec<&EvidenceKindId> = Vec::new();
            for (kind, _) in &reads {
                if !kinds.contains(kind) {
                    kinds.push(kind);
                }
            }
            for kind in kinds {
                if !protocol.evidence_kinds.contains(kind) {
                    unresolved.push(Problem::UndeclaredEvidenceKind {
                        referrer: referrer.clone(),
                        kind: kind.clone(),
                    });
                }
            }
            if let Some((_, Some(subject))) = reads.first()
                && !protocol.artifacts.contains(subject)
            {
                unresolved.push(Problem::UndeclaredArtifact {
                    referrer: referrer.clone(),
                    artifact: (*subject).clone(),
                });
            }
            if let Predicate::Before(form) = node
                && form.first == form.then
            {
                unresolved.push(Problem::SelfOrderedEvidence {
                    referrer: referrer.clone(),
                    kind: form.first.clone(),
                });
            }
        }
        Predicate::Input(test) => {
            if test.r#in.is_empty() {
                unresolved.push(Problem::EmptyInputTest {
                    referrer: referrer.clone(),
                    input: test.input.clone(),
                });
            }
            // Each value at its first listing, in the order written.
            let mut listed = Vec::new();
            for value in &test.r#in {
                if listed.contains(&value) {
                    unresolved.push(Problem::DuplicateInputTestValue {
                        referrer: referrer.clone(),
                        input: test.input.clone(),
                        value: value.clone(),
                    });
                } else {
                    listed.push(value);
                }
            }
            let declared = protocol
                .inputs()
                .find(|(id, _)| **id == test.input)
                .map(|(_, input)| input);
            let Some(declared) = declared else {
                unresolved.push(Problem::UndeclaredInput {
                    referrer: referrer.clone(),
                    input: test.input.clone(),
                });
                return;
            };
            for value in listed {
                if !declared.values.contains(value) {
                    unresolved.push(Problem::UndeclaredInputValue {
                        referrer: referrer.clone(),
                        input: test.input.clone(),
                        value: value.clone(),
                    });
                }
            }
        }
        _ => {}
    });
    problems.extend(unresolved);
}

/// Reports cycles among claims whose predicates test other claims: at least one cycle for every
/// set of claims that test each other in a cycle, not every distinct cycle through that set (for
/// `a` testing `b` and `c`, `b` testing `c`, `c` testing `a`, it reports `a -> b -> c -> a` and not
/// `a -> c -> a`). Claims are visited in declaration order and their references in the order
/// written, so the cycles reported are the same on every run. The walk keeps its own stack, so a
/// chain of any length is walked without recursion.
fn claim_cycles(protocol: &Protocol, problems: &mut Vec<Problem>) {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Mark {
        OnPath,
        Done,
    }

    // The claims a claim tests, in the order written; none for a claim not declared.
    let references = |claim: &ClaimId| {
        protocol
            .claims
            .get(claim)
            .map(|declared| declared.true_when.claim_references())
            .unwrap_or_default()
            .into_iter()
    };

    // A depth-first walk with a stack of its own, not recursion, so a long chain of claims cannot
    // overflow the caller's stack: `path` holds the claims being walked, outermost first, and
    // `pending` the references each of them has still to follow.
    let mut marks: BTreeMap<&ClaimId, Mark> = BTreeMap::new();
    for root in protocol.claims.ids() {
        if marks.contains_key(root) {
            continue;
        }
        marks.insert(root, Mark::OnPath);
        let mut path = vec![root];
        let mut pending = vec![references(root)];
        while let Some(next) = pending.last_mut().map(Iterator::next) {
            let Some(next) = next else {
                pending.pop();
                let done = path.pop().expect("a claim for each list of references");
                marks.insert(done, Mark::Done);
                continue;
            };
            match marks.get(next).copied() {
                None if protocol.claims.contains(next) => {
                    marks.insert(next, Mark::OnPath);
                    path.push(next);
                    pending.push(references(next));
                }
                Some(Mark::OnPath) => {
                    let start = path
                        .iter()
                        .position(|on_path| *on_path == next)
                        .expect("a claim marked on the path is on the path");
                    let mut cycle: Vec<ClaimId> =
                        path[start..].iter().map(|id| (*id).clone()).collect();
                    cycle.push(next.clone());
                    problems.push(Problem::ClaimCycle { claims: cycle });
                }
                None | Some(Mark::Done) => {}
            }
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

    /// Case inputs: an input id is an identifier declared once; an input declares a value, each
    /// once, reported after the maximum ages and before the references; an input test names a
    /// declared input and only values it declares, wherever a predicate is written; an
    /// applicability predicate tests no evidence. A protocol that fits is valid.
    #[test]
    fn case_inputs_and_input_tests_are_checked_in_order() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
            case: {inputs: {s: {values: [a, b, a]}, e: {values: []}, 'bad id': {values: [ok]}}}\n\
            evidence_kinds: {k: {}}\n\
            claims: {c: {true_when: {input: s, in: [a, z]}}}\n\
            obligations: {o: {discharged_when: {claim: c}, \
            applicable_when: {all: [{input: x, in: [a]}, {evidence: {kind: k}}]}}}\n";
        assert_eq!(
            problems(source),
            [
                "invalid-identifier: input identifier `bad id` is empty or contains whitespace or a control character",
                "duplicate-input-value: input `s` lists value `a` more than once",
                "empty-input-values: input `e` declares no value",
                "undeclared-input-value: claim `c` tests input `s` for value `z`, which the input does not declare",
                "undeclared-input: obligation `o` references input `x`, which is not declared",
                "evidence-in-applicability: obligation `o` tests evidence kind `k` at `applicable_when.all[1].evidence`; an applicability predicate tests only claim values and case inputs",
            ]
        );
        assert_eq!(
            problems(
                &source
                    .replace("[a, b, a]", "[a, b]")
                    .replace(
                        "e: {values: []}, 'bad id': {values: [ok]}",
                        "e: {values: [ok]}"
                    )
                    .replace("[a, z]", "[a]")
                    .replace("input: x", "input: s")
                    .replace("{evidence: {kind: k}}", "{claim: c}")
            ),
            Vec::<String>::new()
        );
        for written in [
            "{input: s}",
            "{in: [a]}",
            "{input: s, in: [a], claim: c}",
            "{claim: c, in: [a]}",
        ] {
            let source = format!(
                "format: protocol/1\nprotocol: {{id: p, revision: 1}}\ncase: {{inputs: {{s: {{values: [a]}}}}}}\nclaims: {{c: {{true_when: {written}}}}}\n"
            );
            assert!(crate::model::parse(&source).is_err(), "{written} parses");
        }
    }

    /// An input test lists at least one value, each once: an empty list and every repeated listing
    /// are refused before the input and its values are resolved, a value not declared and listed
    /// twice is refused as undeclared once, and an undeclared input with an empty list is refused
    /// for both.
    #[test]
    fn an_input_test_lists_values_each_once() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
            case: {inputs: {s: {values: [a, b]}}}\n\
            claims: {c: {true_when: {input: s, in: []}}, \
            d: {true_when: {input: s, in: [b, z, b, z, b]}}, \
            e: {true_when: {input: x, in: []}}}\n";
        assert_eq!(
            problems(source),
            [
                "empty-input-test: claim `c` tests input `s` for no value",
                "duplicate-input-test-value: claim `d` tests input `s` for value `b` more than once",
                "duplicate-input-test-value: claim `d` tests input `s` for value `z` more than once",
                "duplicate-input-test-value: claim `d` tests input `s` for value `b` more than once",
                "undeclared-input-value: claim `d` tests input `s` for value `z`, which the input does not declare",
                "empty-input-test: claim `e` tests input `x` for no value",
                "undeclared-input: claim `e` references input `x`, which is not declared",
            ]
        );
    }

    /// An evidence match's subject resolves to a declared artifact in every section a predicate
    /// appears in; one that does not is refused after the kind written beside it, naming the
    /// declaration and the artifact. A declared subject is not refused.
    #[test]
    fn an_undeclared_evidence_subject_is_refused_naming_it() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
            artifacts: {a: {}}\nevidence_kinds: {k: {}}\n\
            claims: {c: {true_when: {all: [{evidence: {kind: z, subject: x}}, {evidence: {kind: k, subject: a}}]}}}\n\
            actions: {act: {precondition: {evidence: {kind: k, subject: y}}}}\n\
            outcomes: {o: {requires: {not: {evidence: {kind: k, subject: w}}}}}\n";
        assert_eq!(
            problems(source),
            [
                "undeclared-evidence-kind: claim `c` references evidence kind `z`, which is not declared",
                "undeclared-artifact: claim `c` references artifact `x`, which is not declared",
                "undeclared-artifact: action `act` references artifact `y`, which is not declared",
                "undeclared-artifact: outcome `o` references artifact `w`, which is not declared",
            ]
        );
        assert_eq!(
            problems(
                &source
                    .replace("kind: z, subject: x", "kind: k, subject: a")
                    .replace("subject: y", "subject: a")
                    .replace("subject: w", "subject: a")
            ),
            Vec::<String>::new()
        );
    }

    /// An invalidation rule's id is an identifier declared once, its upstream artifact is declared
    /// and so is every claim it invalidates. Its references are reported after those of outcomes,
    /// the upstream artifact first, each naming the rule and what it names.
    #[test]
    fn an_invalidation_rule_resolves_its_upstream_artifact_and_claims() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
            artifacts: {a: {}}\nevidence_kinds: {k: {}}\n\
            claims: {c: {true_when: {evidence: {kind: k}}}}\n\
            outcomes: {o: {requires: {claim: gone}}}\n\
            invalidation: {r: {upstream: x, invalidates: [c, y]}, r: {upstream: a, invalidates: []}, \
            'bad id': {upstream: a, invalidates: [c]}}\n";
        assert_eq!(
            problems(source),
            [
                "invalid-identifier: invalidation rule identifier `bad id` is empty or contains whitespace or a control character",
                "duplicate-identifier: invalidation rule `r` is declared more than once",
                "undeclared-claim: outcome `o` references claim `gone`, which is not declared",
                "undeclared-artifact: invalidation rule `r` references artifact `x`, which is not declared",
                "undeclared-claim: invalidation rule `r` references claim `y`, which is not declared",
            ]
        );
        assert_eq!(
            problems(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
                 evidence_kinds: {k: {}}\nclaims: {c: {true_when: {evidence: {kind: k}}}}\n\
                 invalidation: {r: {upstream: a, invalidates: [c]}}\n"
            ),
            Vec::<String>::new()
        );
    }

    /// A rule that names a claim reaching no evidence match, in its own predicate or in any claim
    /// it tests, is refused as `inert-invalidation`, naming the rule and the claim: no record could
    /// ever be kept from that claim's evaluation. It is reported after every unresolved reference, so
    /// after the other rule refusals, and before claim cycles. A claim that reaches a match only
    /// through a claim it tests (`only_tests`) is not inert.
    #[test]
    fn a_rule_naming_a_claim_that_reaches_no_evidence_is_inert() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
            artifacts: {a: {}}\nevidence_kinds: {k: {}}\n\
            claims: {helper: {true_when: {evidence: {kind: k}}}, \
            only_tests: {true_when: {not: {claim: helper}}}, \
            bare: {true_when: {all: []}}, on_bare: {true_when: {claim: bare}}, \
            mixed: {true_when: {all: [{claim: helper}, {evidence: {kind: k}}]}}}\n\
            invalidation: {r: {upstream: a, invalidates: [mixed, only_tests, bare, gone]}, \
            s: {upstream: x, invalidates: [on_bare]}}\n";
        assert_eq!(
            problems(source),
            [
                "undeclared-claim: invalidation rule `r` references claim `gone`, which is not declared",
                "undeclared-artifact: invalidation rule `s` references artifact `x`, which is not declared",
                "inert-invalidation: invalidation rule `r` invalidates claim `bare`, which reaches no evidence match",
                "inert-invalidation: invalidation rule `s` invalidates claim `on_bare`, which reaches no evidence match",
            ]
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

    /// An order form names declared kinds and a declared subject, refused as an evidence match is,
    /// each kind once, `first` before `then`, then the subject; a `before` of one kind is refused,
    /// naming it. In a discharge or applicability predicate, an order form tests evidence and is
    /// refused there with its path. A protocol that fits is valid, and an order form makes no
    /// invalidation rule active.
    #[test]
    fn an_order_form_resolves_its_kinds_and_subject() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
            artifacts: {a: {}}\nevidence_kinds: {k: {}, m: {}}\n\
            claims: {c: {true_when: {all: [{first: {kind: z, result: r, subject: x}}, \
            {before: {first: y, then: z, subject: w}}, {before: {first: k, then: k}}]}}, \
            d: {true_when: {first: {kind: k, result: r}}}}\n\
            obligations: {o: {discharged_when: {not: {before: {first: k, then: m}}}, \
            applicable_when: {any: [{claim: c}, {first: {kind: m, result: r}}]}}}\n\
            invalidation: {i: {upstream: a, invalidates: [d]}}\n";
        assert_eq!(
            problems(source),
            [
                "undeclared-evidence-kind: claim `c` references evidence kind `z`, which is not declared",
                "undeclared-artifact: claim `c` references artifact `x`, which is not declared",
                "undeclared-evidence-kind: claim `c` references evidence kind `y`, which is not declared",
                "undeclared-evidence-kind: claim `c` references evidence kind `z`, which is not declared",
                "undeclared-artifact: claim `c` references artifact `w`, which is not declared",
                "self-ordered-evidence: claim `c` orders evidence kind `k` before itself",
                "evidence-in-discharge: obligation `o` tests evidence kind `k` at `discharged_when.not.before`; a discharge predicate tests only claim values",
                "evidence-in-discharge: obligation `o` tests evidence kind `m` at `discharged_when.not.before`; a discharge predicate tests only claim values",
                "evidence-in-applicability: obligation `o` tests evidence kind `m` at `applicable_when.any[1].first`; an applicability predicate tests only claim values and case inputs",
                "inert-invalidation: invalidation rule `i` invalidates claim `d`, which reaches no evidence match",
            ]
        );
        assert_eq!(
            problems(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
                 artifacts: {a: {}}\nevidence_kinds: {k: {}, m: {}}\n\
                 claims: {c: {true_when: {all: [{first: {kind: k, result: r, subject: a}}, \
                 {before: {first: m, then: k, subject: a}}, {before: {first: k, then: m}}]}}}\n"
            ),
            Vec::<String>::new()
        );
    }

    /// An `after` resolves its first kind, then its later kind, each once, then its subject; one kind
    /// as both is valid, and in a discharge or applicability predicate it is refused with its path,
    /// each kind once.
    #[test]
    fn an_after_form_resolves_its_kinds_and_subject() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
            artifacts: {a: {}}\nevidence_kinds: {k: {}, m: {}}\n\
            claims: {c: {true_when: {any: [\
            {after: {first: y, then: {kind: z, result: r}, subject: w}}, \
            {after: {first: z, then: {kind: z, result: r}}}, \
            {after: {first: k, then: {kind: k, result: r}, subject: a}}]}}}\n\
            obligations: {o: {discharged_when: {after: {first: k, then: {kind: m, result: r}}}, \
            applicable_when: {not: {after: {first: m, then: {kind: m, result: r}}}}}}\n";
        assert_eq!(
            problems(source),
            [
                "undeclared-evidence-kind: claim `c` references evidence kind `y`, which is not declared",
                "undeclared-evidence-kind: claim `c` references evidence kind `z`, which is not declared",
                "undeclared-artifact: claim `c` references artifact `w`, which is not declared",
                "undeclared-evidence-kind: claim `c` references evidence kind `z`, which is not declared",
                "evidence-in-discharge: obligation `o` tests evidence kind `k` at `discharged_when.after`; a discharge predicate tests only claim values",
                "evidence-in-discharge: obligation `o` tests evidence kind `m` at `discharged_when.after`; a discharge predicate tests only claim values",
                "evidence-in-applicability: obligation `o` tests evidence kind `m` at `applicable_when.not.after`; an applicability predicate tests only claim values and case inputs",
            ]
        );
    }

    /// `after` is a map with required keys `first` and `then`, its `then` a map with required keys
    /// `kind` and `result`; an unknown key, a null and a second form beside it do not parse.
    #[test]
    fn an_after_form_is_written_with_its_required_keys() {
        for written in [
            "{after: {first: k}}",
            "{after: {then: {kind: m, result: r}}}",
            "{after: {first: k, then: m}}",
            "{after: {first: k, then: {kind: m}}}",
            "{after: {first: k, then: {result: r}}}",
            "{after: {first: k, then: {kind: m, result: ~}}}",
            "{after: {first: k, then: {kind: m, result: r, subject: a}}}",
            "{after: {first: k, then: {kind: m, result: r}, result: r}}",
            "{after: {first: k, then: {kind: m, result: r}, subject: ~}}",
            "{after: {first: ~, then: {kind: m, result: r}}}",
            "{after: ~}",
            "{after: {first: k, then: {kind: m, result: r}}, before: {first: k, then: m}}",
        ] {
            let source = format!(
                "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nevidence_kinds: {{k: {{}}, m: {{}}}}\nclaims: {{c: {{true_when: {written}}}}}\n"
            );
            assert!(crate::model::parse(&source).is_err(), "{written} parses");
            let source = format!(
                "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nevidence_kinds: {{k: {{}}, m: {{}}}}\nclaims: {{c: {{true_when: {{all: []}}}}}}\noutcomes: {{o: {{requires: {written}}}}}\n"
            );
            assert!(
                crate::model::parse(&source).is_err(),
                "{written} parses as a requirement"
            );
        }
        let parsed = crate::model::parse(
            "format: protocol/1\nprotocol: {id: p, revision: 1}\nevidence_kinds: {k: {}, m: {}}\n\
             outcomes: {o: {requires: {after: {first: k, then: {kind: m, result: r}}}}}\n",
        )
        .expect("parses");
        let (_, outcome) = parsed.outcomes.iter().next().expect("o");
        assert_eq!(
            outcome.requires,
            Predicate::After(crate::model::EvidenceAfter {
                first: EvidenceKindId::new("k"),
                then: crate::model::LaterRecord {
                    kind: EvidenceKindId::new("m"),
                    result: "r".to_owned(),
                },
                subject: None,
            })
        );
    }

    /// A `since` resolves its last kind, then its later kind, each once, then its subject, as an
    /// `after` does (story:evidence-order-since); one kind as both is valid, and in a discharge or
    /// applicability predicate it is refused with its path, each kind once.
    #[test]
    fn a_since_form_resolves_its_kinds_and_subject() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
            artifacts: {a: {}}\nevidence_kinds: {k: {}, m: {}}\n\
            claims: {c: {true_when: {any: [\
            {since: {last: y, then: {kind: z, result: r}, subject: w}}, \
            {since: {last: z, then: {kind: z, result: r}}}, \
            {since: {last: k, then: {kind: k, result: r}, subject: a}}]}}}\n\
            obligations: {o: {discharged_when: {since: {last: k, then: {kind: m, result: r}}}, \
            applicable_when: {not: {since: {last: m, then: {kind: m, result: r}}}}}}\n";
        assert_eq!(
            problems(source),
            [
                "undeclared-evidence-kind: claim `c` references evidence kind `y`, which is not declared",
                "undeclared-evidence-kind: claim `c` references evidence kind `z`, which is not declared",
                "undeclared-artifact: claim `c` references artifact `w`, which is not declared",
                "undeclared-evidence-kind: claim `c` references evidence kind `z`, which is not declared",
                "evidence-in-discharge: obligation `o` tests evidence kind `k` at `discharged_when.since`; a discharge predicate tests only claim values",
                "evidence-in-discharge: obligation `o` tests evidence kind `m` at `discharged_when.since`; a discharge predicate tests only claim values",
                "evidence-in-applicability: obligation `o` tests evidence kind `m` at `applicable_when.not.since`; an applicability predicate tests only claim values and case inputs",
            ]
        );
    }

    /// `since` is a map with required keys `last` and `then`, its `then` a map with required keys
    /// `kind` and `result`; an unknown key, a null and a second form beside it do not parse.
    #[test]
    fn a_since_form_is_written_with_its_required_keys() {
        for written in [
            "{since: {last: k}}",
            "{since: {then: {kind: m, result: r}}}",
            "{since: {last: k, then: m}}",
            "{since: {last: k, then: {kind: m}}}",
            "{since: {last: k, then: {result: r}}}",
            "{since: {last: k, then: {kind: m, result: ~}}}",
            "{since: {last: k, then: {kind: m, result: r, subject: a}}}",
            "{since: {first: k, then: {kind: m, result: r}}}",
            "{since: {last: k, then: {kind: m, result: r}, result: r}}",
            "{since: {last: k, then: {kind: m, result: r}, subject: ~}}",
            "{since: {last: ~, then: {kind: m, result: r}}}",
            "{since: ~}",
            "{since: {last: k, then: {kind: m, result: r}}, after: {first: k, then: {kind: m, result: r}}}",
        ] {
            let source = format!(
                "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nevidence_kinds: {{k: {{}}, m: {{}}}}\nclaims: {{c: {{true_when: {written}}}}}\n"
            );
            assert!(crate::model::parse(&source).is_err(), "{written} parses");
            let source = format!(
                "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nevidence_kinds: {{k: {{}}, m: {{}}}}\nclaims: {{c: {{true_when: {{all: []}}}}}}\noutcomes: {{o: {{requires: {written}}}}}\n"
            );
            assert!(
                crate::model::parse(&source).is_err(),
                "{written} parses as a requirement"
            );
        }
        let parsed = crate::model::parse(
            "format: protocol/1\nprotocol: {id: p, revision: 1}\nevidence_kinds: {k: {}, m: {}}\n\
             outcomes: {o: {requires: {since: {last: k, then: {kind: m, result: r}}}}}\n",
        )
        .expect("parses");
        let (_, outcome) = parsed.outcomes.iter().next().expect("o");
        assert_eq!(
            outcome.requires,
            Predicate::Since(crate::model::EvidenceSince {
                last: EvidenceKindId::new("k"),
                then: crate::model::LaterRecord {
                    kind: EvidenceKindId::new("m"),
                    result: "r".to_owned(),
                },
                subject: None,
            })
        );
    }

    /// The order forms are maps with required keys: `kind` and `result`, or `first` and `then`;
    /// an unknown key, a null and a second form beside one do not parse.
    #[test]
    fn an_order_form_is_written_with_its_required_keys() {
        for written in [
            "{first: {kind: k}}",
            "{first: {result: r}}",
            "{first: {kind: k, result: r, then: m}}",
            "{first: {kind: k, result: ~}}",
            "{first: {kind: k, result: r, subject: ~}}",
            "{before: {first: k}}",
            "{before: {then: k}}",
            "{before: {first: k, then: m, result: r}}",
            "{before: ~}",
            "{first: {kind: k, result: r}, claim: c}",
            "{before: {first: k, then: m}, first: {kind: k, result: r}}",
        ] {
            let source = format!(
                "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nevidence_kinds: {{k: {{}}, m: {{}}}}\nclaims: {{c: {{true_when: {written}}}}}\n"
            );
            assert!(crate::model::parse(&source).is_err(), "{written} parses");
            let source = format!(
                "format: protocol/1\nprotocol: {{id: p, revision: 1}}\nevidence_kinds: {{k: {{}}, m: {{}}}}\nclaims: {{c: {{true_when: {{all: []}}}}}}\noutcomes: {{o: {{requires: {written}}}}}\n"
            );
            assert!(
                crate::model::parse(&source).is_err(),
                "{written} parses as a requirement"
            );
        }
        let parsed = crate::model::parse(
            "format: protocol/1\nprotocol: {id: p, revision: 1}\nevidence_kinds: {k: {}, m: {}}\n\
             claims: {c: {true_when: {first: {kind: k, result: r}}}}\n\
             outcomes: {o: {requires: {before: {first: k, then: m}}}}\n",
        )
        .expect("parses");
        let (_, claim) = parsed.claims.iter().next().expect("c");
        assert_eq!(
            claim.true_when,
            Predicate::First(crate::model::FirstResult {
                kind: EvidenceKindId::new("k"),
                result: "r".to_owned(),
                subject: None,
            })
        );
        let (_, outcome) = parsed.outcomes.iter().next().expect("o");
        assert_eq!(
            outcome.requires,
            Predicate::Before(crate::model::EvidenceOrder {
                first: EvidenceKindId::new("k"),
                then: EvidenceKindId::new("m"),
                subject: None,
            })
        );
    }
}
