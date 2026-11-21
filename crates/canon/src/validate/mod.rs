//! Deterministic validation of a `protocol/1` document.
//!
//! Validation resolves every reference and reports every problem it finds, in a stable order that
//! does not depend on the order the sections are written in:
//!
//! 1. the format;
//! 2. the imports section (story:protocol-imports): each import id that is not an identifier,
//!    followed by each of its `refines` entries that is not one (`invalid-identifier`), then each
//!    import id listed again (`duplicate-identifier`), then each digest not written `sha256:` and
//!    64 lowercase hexadecimal digits (`invalid-import-digest`), each naming the import, then each
//!    import, once, as `missing-import`: a document with imports is valid only composed with the
//!    documents it imports (`b10x_canon::ir::compose`), which leaves it without imports. Every
//!    other problem is the composed document's, whose declarations may come from any document of
//!    the composition;
//! 3. malformed identifiers: the protocol id, then the ids declared in artifacts, evidence kinds,
//!    claims, obligations, actions, outcomes, invalidation rules, case inputs and independence
//!    dimensions, in that section order, then the capabilities actions name, each followed by the
//!    keys and values of its scope in the order written, the effect classes actions name and the
//!    protocol each action's `case_open` names, in action order, then the protocol each evidence
//!    kind's `supplied_by` names, in evidence kind order, then the decisions outcomes require, in
//!    outcome order, then the values case inputs declare, in input order;
//! 4. duplicate identifiers, in the same section order;
//! 5. capability requirements whose scope names a key more than once (`duplicate-scope-key`,
//!    naming the action, the capability and the key, once for every naming after its first), in
//!    the order the actions and their requirements are written;
//! 6. maximum ages that are not a whole number without leading zeros, followed by `s`, `m`, `h`
//!    or `d`, or that are but are too long to count in seconds (each with its own message), and
//!    evidence kinds whose `supplied_by` names the protocol's own id (`self-supplied-evidence`,
//!    naming the kind; story:supplied-evidence), each kind's in that order, in the order the
//!    evidence kinds are written. Any other protocol may supply a kind, and nothing resolves the
//!    name;
//! 7. case inputs that declare no value (`empty-input-values`, naming the input) or list a value
//!    more than once (`duplicate-input-value`, naming the input and the value, once for every
//!    listing after its first), in the order the inputs are written;
//! 8. action argument schemas outside the argument-schema dialect (`action-arguments`; the dialect,
//!    and the order within one schema, are stated with it), each naming the action, the JSON
//!    Pointer of the keyword inside the schema and why it is refused, one per offending keyword,
//!    in the order the actions are written;
//! 9. unresolved references and malformed input tests and order forms (a claim, an evidence kind,
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
//!    naming the declaration the test is written in and the input; for a `children` form, first
//!    an empty `in` list, `empty-child-outcomes`, or an outcome it lists more than once,
//!    `duplicate-child-outcome`, once for every listing after its first, then `open` listed,
//!    `open-child-outcome`, once, then the evidence kind `case_outcome` when it is not declared,
//!    `undeclared-evidence-kind`, every one naming the declaration the form is written in): those
//!    in claims first, then those in obligations (each obligation's discharge predicate, followed
//!    by every evidence match, order form and `children` form it holds, which a discharge
//!    predicate may not: it tests only claim values, each kind an order form names once, the kind
//!    `case_outcome` for a `children` form; then its applicability predicate, followed likewise by
//!    every evidence match, order form and `children` form it holds, `evidence-in-applicability`:
//!    it tests only claim values and case inputs), then those in actions (each action's
//!    precondition, then each evidence kind it may produce, then the subject of the case it opens,
//!    `undeclared-artifact`, then an opening under the protocol's own id, `self-opened-case`, both
//!    naming the action), then those in outcomes, then those in invalidation rules (the upstream
//!    artifact, then each claim the rule invalidates); within a section, in the order its
//!    declarations are written, and within a declaration, in the order its references are
//!    written;
//! 10. invalidation rules that name a claim reaching no evidence match, in its own predicate or
//!     in a claim it tests through any number of claim references (`inert-invalidation`), in the
//!     order the rules and their claims are written. An order form or a `children` form is not an
//!     evidence match here: it reads the history, which an invalidation rule does not change;
//! 11. cycles between claims.
//!
//! Within a section, entries come in the order they are written. Every message renders document
//! text on one line. The same document always yields the same problems in the same order.

pub(crate) mod arguments;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::ir::DIGEST_PREFIX;
use crate::model::{
    ActionId, ArtifactId, CASE_OUTCOME, CapabilityId, ClaimId, EvidenceKindId, FORMAT,
    IndependenceDimensionId, InputId, InputValue, InvalidationRuleId, OPEN_OUTCOME, ObligationId,
    OutcomeId, Predicate, Protocol, ProtocolId, ScopeKey, is_identifier, one_line,
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
    ScopeKey,
    ScopeValue,
    /// An imported protocol, by the id the `imports` section keys it by.
    Import,
    /// An identifier an import lists under `refines`.
    Refinement,
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
            Section::ScopeKey => "scope key",
            Section::ScopeValue => "scope value",
            Section::Import => "import",
            Section::Refinement => "refinement",
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
    /// A capability requirement's scope names a key more than once.
    DuplicateScopeKey {
        action: ActionId,
        capability: CapabilityId,
        key: ScopeKey,
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
    /// A `children` form lists no outcome, so no child case could end with one of them.
    EmptyChildOutcomes { referrer: Referrer },
    /// A `children` form lists an outcome more than once.
    DuplicateChildOutcome { referrer: Referrer, outcome: String },
    /// A `children` form lists `open`, which is a running child case's outcome, never a terminal
    /// one: the form is unknown while a child is open.
    OpenChildOutcome { referrer: Referrer },
    /// An action opens a child case under the protocol's own id: a case could then open cases
    /// under its own protocol without end (D206).
    SelfOpenedCase {
        action: ActionId,
        protocol: ProtocolId,
    },
    /// An evidence kind names the protocol's own id as the protocol that supplies its records
    /// (story:supplied-evidence): records of it come from outside the protocol, or from its own
    /// actions, never from a case of the same protocol through `supplied_by`.
    SelfSuppliedEvidence {
        kind: EvidenceKindId,
        protocol: ProtocolId,
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
    /// An import pins a digest not written `sha256:` followed by 64 lowercase hexadecimal digits.
    InvalidImportDigest { import: ProtocolId, digest: String },
    /// A protocol imports one that no supplied document is: a protocol with imports compiles only
    /// with every document it imports, directly or through another import.
    MissingImport {
        importer: ProtocolId,
        import: ProtocolId,
    },
    /// A document supplied to a composition that neither the root nor any document it imports
    /// imports.
    UnusedImport { protocol: ProtocolId },
    /// Two documents supplied to one composition, the root included, are of one protocol.
    DuplicateImportDocument { protocol: ProtocolId },
    /// Protocols that import each other in a cycle, from the first a walk from the root meets; the
    /// first protocol is repeated at the end.
    ImportCycle { protocols: Vec<ProtocolId> },
    /// The supplied document of an import is another revision than the import pins.
    ImportRevisionMismatch {
        importer: ProtocolId,
        import: ProtocolId,
        pinned: u64,
        supplied: u64,
    },
    /// The supplied document of an import, compiled with its own imports, has another digest than
    /// the import pins.
    ImportDigestMismatch {
        importer: ProtocolId,
        import: ProtocolId,
        pinned: String,
        actual: String,
    },
    /// An import lists under `refines` an identifier it does not declare, itself or through its
    /// own imports.
    UndeclaredRefinement {
        importer: ProtocolId,
        import: ProtocolId,
        id: String,
    },
    /// Two imports declare one identifier in one section differently, and the importing document
    /// does not redeclare it. Two declarations with the same `canon-ir/1` entry are one.
    ConflictingImports {
        importer: ProtocolId,
        section: Section,
        id: String,
        first: ProtocolId,
        second: ProtocolId,
    },
    /// A document redeclares an identifier an import declares in the same section, and that
    /// import does not list it under `refines`.
    UnrefinedRedeclaration {
        importer: ProtocolId,
        section: Section,
        id: String,
        import: ProtocolId,
    },
    /// An imported document is not valid itself: `problem` says why.
    InvalidImport {
        import: ProtocolId,
        problem: Box<Problem>,
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
            Problem::DuplicateScopeKey { .. } => "duplicate-scope-key",
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
            Problem::EmptyChildOutcomes { .. } => "empty-child-outcomes",
            Problem::DuplicateChildOutcome { .. } => "duplicate-child-outcome",
            Problem::OpenChildOutcome { .. } => "open-child-outcome",
            Problem::SelfOpenedCase { .. } => "self-opened-case",
            Problem::SelfSuppliedEvidence { .. } => "self-supplied-evidence",
            Problem::ClaimCycle { .. } => "claim-cycle",
            Problem::ActionArguments { .. } => "action-arguments",
            Problem::InvalidImportDigest { .. } => "invalid-import-digest",
            Problem::MissingImport { .. } => "missing-import",
            Problem::UnusedImport { .. } => "unused-import",
            Problem::DuplicateImportDocument { .. } => "duplicate-import-document",
            Problem::ImportCycle { .. } => "import-cycle",
            Problem::ImportRevisionMismatch { .. } => "import-revision-mismatch",
            Problem::ImportDigestMismatch { .. } => "import-digest-mismatch",
            Problem::UndeclaredRefinement { .. } => "undeclared-refinement",
            Problem::ConflictingImports { .. } => "conflicting-imports",
            Problem::UnrefinedRedeclaration { .. } => "unrefined-redeclaration",
            Problem::InvalidImport { .. } => "invalid-import",
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
            Problem::DuplicateScopeKey {
                action,
                capability,
                key,
            } => write!(
                f,
                "action `{}` requires capability `{}` with scope key `{}` more than once",
                one_line(action.as_str()),
                one_line(capability.as_str()),
                one_line(key.as_str())
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
            Problem::EmptyChildOutcomes { referrer } => {
                write!(f, "{referrer} tests the child cases for no outcome")
            }
            Problem::DuplicateChildOutcome { referrer, outcome } => write!(
                f,
                "{referrer} tests the child cases for outcome `{}` more than once",
                one_line(outcome)
            ),
            Problem::OpenChildOutcome { referrer } => write!(
                f,
                "{referrer} tests the child cases for outcome `{OPEN_OUTCOME}`, which a running child case has, never a terminal one"
            ),
            Problem::SelfOpenedCase { action, protocol } => write!(
                f,
                "action `{}` opens a case under protocol `{}`, the protocol's own id",
                one_line(action.as_str()),
                one_line(protocol.as_str())
            ),
            Problem::SelfSuppliedEvidence { kind, protocol } => write!(
                f,
                "evidence kind `{}` is supplied by protocol `{}`, the protocol's own id",
                one_line(kind.as_str()),
                one_line(protocol.as_str())
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
            Problem::InvalidImportDigest { import, digest } => write!(
                f,
                "import `{}` pins digest `{}`, which is not `{DIGEST_PREFIX}` followed by 64 lowercase hexadecimal digits",
                one_line(import.as_str()),
                one_line(digest)
            ),
            Problem::MissingImport { importer, import } => write!(
                f,
                "protocol `{}` imports `{}`, and no supplied document is that protocol",
                one_line(importer.as_str()),
                one_line(import.as_str())
            ),
            Problem::UnusedImport { protocol } => write!(
                f,
                "the supplied protocol `{}` is imported neither by the root nor by any document it imports",
                one_line(protocol.as_str())
            ),
            Problem::DuplicateImportDocument { protocol } => write!(
                f,
                "protocol `{}` is supplied more than once",
                one_line(protocol.as_str())
            ),
            Problem::ImportCycle { protocols } => {
                let path: Vec<String> = protocols.iter().map(|id| one_line(id.as_str())).collect();
                write!(
                    f,
                    "protocols import each other in a cycle: {}",
                    path.join(" -> ")
                )
            }
            Problem::ImportRevisionMismatch {
                importer,
                import,
                pinned,
                supplied,
            } => write!(
                f,
                "protocol `{}` imports `{}` pinned to revision {pinned}, and the supplied document is revision {supplied}",
                one_line(importer.as_str()),
                one_line(import.as_str())
            ),
            Problem::ImportDigestMismatch {
                importer,
                import,
                pinned,
                actual,
            } => write!(
                f,
                "protocol `{}` imports `{}` pinned to digest {}, and the supplied document compiles to {actual}",
                one_line(importer.as_str()),
                one_line(import.as_str()),
                one_line(pinned)
            ),
            Problem::UndeclaredRefinement {
                importer,
                import,
                id,
            } => write!(
                f,
                "protocol `{}` refines `{}` of its import `{}`, which declares no such identifier",
                one_line(importer.as_str()),
                one_line(id),
                one_line(import.as_str())
            ),
            Problem::ConflictingImports {
                importer,
                section,
                id,
                first,
                second,
            } => write!(
                f,
                "protocol `{}` imports {section} `{}` from `{}` and from `{}`, declared differently",
                one_line(importer.as_str()),
                one_line(id),
                one_line(first.as_str()),
                one_line(second.as_str())
            ),
            Problem::UnrefinedRedeclaration {
                importer,
                section,
                id,
                import,
            } => write!(
                f,
                "protocol `{}` declares {section} `{}`, which its import `{}` declares and does not list under `refines`",
                one_line(importer.as_str()),
                one_line(id),
                one_line(import.as_str())
            ),
            Problem::InvalidImport { import, problem } => write!(
                f,
                "imported protocol `{}`: {}: {problem}",
                one_line(import.as_str()),
                problem.code()
            ),
        }
    }
}

/// The problems of a document's own `imports` section, in the order written: each import id that
/// is not an identifier, followed by each of its `refines` entries that is not one, then each
/// import id listed again, then each digest not written [`DIGEST_PREFIX`] and 64 lowercase
/// hexadecimal digits.
pub(crate) fn import_section(protocol: &Protocol) -> Vec<Problem> {
    let mut problems = Vec::new();
    for (id, import) in protocol.imports.iter() {
        if !is_identifier(id.as_str()) {
            problems.push(Problem::InvalidIdentifier {
                section: Section::Import,
                id: id.as_str().to_owned(),
            });
        }
        for refined in &import.refines {
            if !is_identifier(refined) {
                problems.push(Problem::InvalidIdentifier {
                    section: Section::Refinement,
                    id: refined.clone(),
                });
            }
        }
    }
    duplicates(
        Section::Import,
        &strs(protocol.imports.ids()),
        &mut problems,
    );
    for (id, import) in protocol.imports.iter() {
        let well_formed = import
            .digest
            .strip_prefix(DIGEST_PREFIX)
            .is_some_and(|hex| {
                hex.len() == 64 && hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
            });
        if !well_formed {
            problems.push(Problem::InvalidImportDigest {
                import: id.clone(),
                digest: import.digest.clone(),
            });
        }
    }
    problems
}

/// Validates a parsed protocol. `Ok` when it is valid; otherwise every problem, in a stable order.
pub fn validate(protocol: &Protocol) -> Result<(), Vec<Problem>> {
    let mut problems = Vec::new();

    if protocol.format != FORMAT {
        problems.push(Problem::UnsupportedFormat {
            found: protocol.format.clone(),
        });
    }

    problems.extend(import_section(protocol));
    let mut uncomposed = BTreeSet::new();
    for import in protocol.imports.ids() {
        if uncomposed.insert(import) {
            problems.push(Problem::MissingImport {
                importer: protocol.protocol.id.clone(),
                import: import.clone(),
            });
        }
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
            for (key, value) in requirement.scope.iter() {
                named.push((Section::ScopeKey, key.as_str()));
                named.push((Section::ScopeValue, value.as_str()));
            }
        }
        if let Some(effect) = &action.effect {
            named.push((Section::EffectClass, effect.as_str()));
        }
        if let Some(opening) = &action.case_open {
            named.push((Section::Protocol, opening.protocol.as_str()));
        }
    }
    for (_, kind) in protocol.evidence_kinds.iter() {
        if let Some(supplier) = &kind.supplied_by {
            named.push((Section::Protocol, supplier.as_str()));
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

    for (id, action) in protocol.actions.iter() {
        for requirement in &action.requires {
            let mut seen = BTreeSet::new();
            for key in requirement.scope.ids() {
                if !seen.insert(key) {
                    problems.push(Problem::DuplicateScopeKey {
                        action: id.clone(),
                        capability: requirement.capability.clone(),
                        key: key.clone(),
                    });
                }
            }
        }
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
        if kind.supplied_by.as_ref() == Some(&protocol.protocol.id) {
            problems.push(Problem::SelfSuppliedEvidence {
                kind: id.clone(),
                protocol: protocol.protocol.id.clone(),
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
        if let Some(opening) = &action.case_open {
            if !protocol.artifacts.contains(&opening.subject) {
                problems.push(Problem::UndeclaredArtifact {
                    referrer: referrer.clone(),
                    artifact: opening.subject.clone(),
                });
            }
            if opening.protocol == protocol.protocol.id {
                problems.push(Problem::SelfOpenedCase {
                    action: id.clone(),
                    protocol: opening.protocol.clone(),
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

    let reaching = reaching_evidence(protocol);
    for (id, rule) in protocol.invalidation.iter() {
        for claim in &rule.invalidates {
            if protocol.claims.contains(claim) && !reaching.contains(claim) {
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

/// The declared claims that reach an evidence match: in their own predicate, or in a claim they test
/// through any number of claim references. A claim declared twice is read in its first
/// declaration, as `Declarations::get` reads it. The claims whose predicate holds a match are found
/// first, then every claim testing one found, each once, so a cycle ends and the work grows with
/// the size of the predicates, not with the number of rules times the length of a chain.
fn reaching_evidence(protocol: &Protocol) -> BTreeSet<&ClaimId> {
    let mut declared: BTreeMap<&ClaimId, &Predicate> = BTreeMap::new();
    for (id, claim) in protocol.claims.iter() {
        declared.entry(id).or_insert(&claim.true_when);
    }
    let mut testers: BTreeMap<&ClaimId, Vec<&ClaimId>> = BTreeMap::new();
    let mut reaching = BTreeSet::new();
    let mut pending = Vec::new();
    for (&id, &predicate) in &declared {
        let mut found = false;
        predicate.visit(&mut |node| match node {
            Predicate::Evidence(_) => found = true,
            Predicate::Claim(test) => testers.entry(&test.claim).or_default().push(id),
            _ => {}
        });
        if found && reaching.insert(id) {
            pending.push(id);
        }
    }
    while let Some(next) = pending.pop() {
        for &tester in testers.get(next).into_iter().flatten() {
            if reaching.insert(tester) {
                pending.push(tester);
            }
        }
    }
    reaching
}

/// Every evidence match, order form and `children` form in an obligation's discharge or
/// applicability predicate, in the order written, each with its kind and its path from `root`
/// (`discharged_when`, `applicable_when`); a `before`, an `after` or a `since` gives each kind it
/// names once, `first` (or `last`) before `then`, and a `children` form the kind `case_outcome`.
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
            Predicate::Children(_) => {
                found.push((
                    EvidenceKindId::new(CASE_OUTCOME),
                    format!("{path}.children"),
                ));
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
/// value its input does not declare, every `before` that orders a kind before itself, and every
/// `children` form that lists no outcome, an outcome twice or `open`, or is written in a protocol
/// that does not declare `case_outcome`, in the order the predicate is written.
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
        Predicate::Children(form) => {
            if form.r#in.is_empty() {
                unresolved.push(Problem::EmptyChildOutcomes {
                    referrer: referrer.clone(),
                });
            }
            let mut listed = Vec::new();
            for outcome in &form.r#in {
                if listed.contains(&outcome) {
                    unresolved.push(Problem::DuplicateChildOutcome {
                        referrer: referrer.clone(),
                        outcome: outcome.clone(),
                    });
                } else {
                    listed.push(outcome);
                }
            }
            if listed
                .iter()
                .any(|outcome| outcome.as_str() == OPEN_OUTCOME)
            {
                unresolved.push(Problem::OpenChildOutcome {
                    referrer: referrer.clone(),
                });
            }
            let kind = EvidenceKindId::new(CASE_OUTCOME);
            if !protocol.evidence_kinds.contains(&kind) {
                unresolved.push(Problem::UndeclaredEvidenceKind {
                    referrer: referrer.clone(),
                    kind,
                });
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

    /// A capability requirement's scope keys and values are identifiers, checked right after its
    /// capability; a key is named once per requirement, reported after the duplicate identifiers
    /// and before the maximum ages. One capability at several scopes, or with and without one, is
    /// valid.
    #[test]
    fn a_requirement_scope_names_each_key_once_with_identifiers() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
            evidence_kinds: {k: {max_age: 5x}}\n\
            actions:\n  b: {}\n  b: {}\n  a:\n    effect: 'bad effect'\n    requires:\n      \
            - {capability: m, scope: {'clu ster': prod-eu, namespace: ''}}\n      \
            - {capability: m, scope: {cluster: a, namespace: foo, cluster: b, cluster: c}}\n      \
            - {capability: n, scope: {cluster: a}}\n";
        assert_eq!(
            problems(source),
            [
                "invalid-identifier: scope key identifier `clu ster` is empty or contains whitespace or a control character",
                "invalid-identifier: scope value identifier `` is empty or contains whitespace or a control character",
                "invalid-identifier: effect class identifier `bad effect` is empty or contains whitespace or a control character",
                "duplicate-identifier: action `b` is declared more than once",
                "duplicate-scope-key: action `a` requires capability `m` with scope key `cluster` more than once",
                "duplicate-scope-key: action `a` requires capability `m` with scope key `cluster` more than once",
                "invalid-max-age: evidence kind `k` has max_age `5x`, which is not a whole number without leading zeros, followed by s, m, h or d",
            ]
        );
        assert_eq!(
            problems(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
                 actions: {a: {requires: [{capability: m}, {capability: m, scope: {cluster: a}}, \
                 {capability: m, scope: {cluster: b, namespace: foo}}, {capability: m, scope: {}}]}}\n"
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

    /// A supplier is a protocol identifier, checked after every name actions give and before the
    /// decisions outcomes require; the protocol's own id is refused as `self-supplied-evidence`,
    /// naming the kind, after its maximum age and before the unresolved references. Any other id
    /// is valid, resolved by nothing: the parent's, a child's, one no document declares.
    #[test]
    fn a_supplier_is_another_protocol_named_by_an_identifier() {
        let source = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
            evidence_kinds: {z: {supplied_by: p, max_age: 5x}, a: {supplied_by: 'q q'}, \
            k: {supplied_by: p}}\n\
            actions: {act: {effect: 'bad effect'}}\n\
            outcomes: {o: {requires: {decision: 'bad decision'}}}\n\
            claims: {c: {true_when: {evidence: {kind: x}}}}\n";
        assert_eq!(
            problems(source),
            [
                "invalid-identifier: effect class identifier `bad effect` is empty or contains whitespace or a control character",
                "invalid-identifier: protocol identifier `q q` is empty or contains whitespace or a control character",
                "invalid-identifier: decision identifier `bad decision` is empty or contains whitespace or a control character",
                "invalid-max-age: evidence kind `z` has max_age `5x`, which is not a whole number without leading zeros, followed by s, m, h or d",
                "self-supplied-evidence: evidence kind `z` is supplied by protocol `p`, the protocol's own id",
                "self-supplied-evidence: evidence kind `k` is supplied by protocol `p`, the protocol's own id",
                "undeclared-evidence-kind: claim `c` references evidence kind `x`, which is not declared",
            ]
        );
        assert_eq!(
            problems(
                "format: protocol/1\nprotocol: {id: p.child, revision: 1}\n\
                 evidence_kinds: {up: {supplied_by: p}, down: {supplied_by: p.child.grandchild}, \
                 side: {supplied_by: p.sibling}, nowhere: {supplied_by: never.declared}}\n"
            ),
            Vec::<String>::new()
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

    /// The oracle: whether `claim` reaches an evidence match, found by walking from it alone, as
    /// the inert-invalidation check did for each claim a rule names before it found the claims
    /// that reach evidence once for every rule. Each claim is visited once, so a cycle ends.
    fn reaches_evidence_by_walking(protocol: &Protocol, claim: &ClaimId) -> bool {
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

    /// Each rule and claim the oracle finds inert, in rule order and then the order the rule names
    /// its claims.
    fn inert_by_walking(protocol: &Protocol) -> Vec<(InvalidationRuleId, ClaimId)> {
        let mut inert = Vec::new();
        for (id, rule) in protocol.invalidation.iter() {
            for claim in &rule.invalidates {
                if protocol.claims.contains(claim) && !reaches_evidence_by_walking(protocol, claim)
                {
                    inert.push((id.clone(), claim.clone()));
                }
            }
        }
        inert
    }

    /// Each `inert-invalidation` problem `validate` reports, in its order.
    fn inert_reported(protocol: &Protocol) -> Vec<(InvalidationRuleId, ClaimId)> {
        validate(protocol)
            .err()
            .into_iter()
            .flatten()
            .filter_map(|problem| match problem {
                Problem::InertInvalidation { rule, claim } => Some((rule, claim)),
                _ => None,
            })
            .collect()
    }

    /// Every protocol under `fixtures/` that parses, with its path.
    fn fixture_protocols() -> Vec<(std::path::PathBuf, Protocol)> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
        let mut pending = vec![root];
        let mut found = Vec::new();
        while let Some(directory) = pending.pop() {
            let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(&directory)
                .expect("fixtures directory reads")
                .map(|entry| entry.expect("fixture entry reads").path())
                .collect();
            entries.sort();
            for path in entries {
                if path.is_dir() {
                    pending.push(path);
                } else if path
                    .extension()
                    .is_some_and(|ext| ext == "yaml" || ext == "yml")
                {
                    let text = std::fs::read_to_string(&path).expect("fixture reads");
                    if let Ok(protocol) = crate::model::parse(&text) {
                        found.push((path, protocol));
                    }
                }
            }
        }
        found
    }

    /// A fixed-seed xorshift generator: the same protocols on every run.
    struct Rng(u64);

    impl Rng {
        fn below(&mut self, n: usize) -> usize {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 % n as u64) as usize
        }
    }

    /// A protocol of `claims` claims, each testing up to two others (itself and later claims
    /// included, so cycles occur) through `all`, `any` or `not`, some reading evidence, with rules
    /// naming declared claims and a claim no section declares.
    fn generated(rng: &mut Rng, claims: usize) -> String {
        let mut source = String::from(
            "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {up: {}}\n\
             evidence_kinds: {k: {}}\nclaims:\n",
        );
        for at in 0..claims {
            let mut members: Vec<String> = (0..rng.below(3))
                .map(|_| format!("{{claim: c{}}}", rng.below(claims)))
                .collect();
            if rng.below(4) == 0 {
                members.push("{evidence: {kind: k}}".to_owned());
            }
            let predicate = match (rng.below(3), members.len()) {
                (_, 0) => "{all: []}".to_owned(),
                (0, 1) => format!("{{not: {}}}", members[0]),
                (1, _) => format!("{{any: [{}]}}", members.join(", ")),
                _ => format!("{{all: [{}]}}", members.join(", ")),
            };
            source.push_str(&format!("  c{at}: {{true_when: {predicate}}}\n"));
        }
        source.push_str("invalidation:\n");
        for rule in 0..claims / 2 {
            let named: Vec<String> = (0..1 + rng.below(3))
                .map(|_| match rng.below(8) {
                    0 => "undeclared".to_owned(),
                    _ => format!("c{}", rng.below(claims)),
                })
                .collect();
            source.push_str(&format!(
                "  r{rule}: {{upstream: up, invalidates: [{}]}}\n",
                named.join(", ")
            ));
        }
        source
    }

    /// The claims that reach evidence, found once for every rule, give every `inert-invalidation`
    /// problem the walk from each named claim gave, in the same order: over every protocol fixture,
    /// cycles, a claim declared twice (its first declaration counts), a rule naming an undeclared
    /// claim, evidence reached only through `not` or `any`, and generated protocols.
    #[test]
    fn inert_invalidation_is_reported_as_the_walk_from_each_named_claim_reports_it() {
        let mut protocols: Vec<(String, Protocol)> = fixture_protocols()
            .into_iter()
            .map(|(path, protocol)| (path.display().to_string(), protocol))
            .collect();
        assert!(protocols.len() >= 20, "fixtures found: {}", protocols.len());
        let written = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
            artifacts: {up: {}}\nevidence_kinds: {k: {}}\n\
            claims:\n\
            \x20 e: {true_when: {evidence: {kind: k}}}\n\
            \x20 through_not: {true_when: {not: {claim: e}}}\n\
            \x20 through_any: {true_when: {any: [{all: []}, {not: {claim: through_not}}]}}\n\
            \x20 bare: {true_when: {all: []}}\n\
            \x20 loop_a: {true_when: {claim: loop_b}}\n\
            \x20 loop_b: {true_when: {not: {claim: loop_a}}}\n\
            \x20 ring_a: {true_when: {claim: ring_b}}\n\
            \x20 ring_b: {true_when: {any: [{claim: ring_a}, {evidence: {kind: k}}]}}\n\
            \x20 self: {true_when: {all: [{claim: self}]}}\n\
            \x20 twice: {true_when: {all: []}}\n\
            invalidation:\n\
            \x20 r1: {upstream: up, invalidates: [through_not, bare, nowhere, through_any]}\n\
            \x20 r2: {upstream: up, invalidates: [loop_a, ring_a, self, loop_b, ring_b]}\n\
            \x20 r3: {upstream: up, invalidates: [twice, e, bare]}\n";
        let protocol = crate::model::parse(written).expect("parses");
        // `twice` declared again, reaching evidence: the first declaration, which does not, counts.
        let mut declared_twice = protocol.clone();
        declared_twice.claims = crate::model::Declarations::new(
            protocol
                .claims
                .iter()
                .map(|(id, claim)| (id.clone(), claim.clone()))
                .chain([(
                    ClaimId::new("twice"),
                    crate::model::Claim {
                        description: None,
                        true_when: Predicate::Claim(crate::model::ClaimTest {
                            claim: ClaimId::new("e"),
                            is: crate::model::Truth::True,
                        }),
                    },
                )])
                .collect(),
        );
        // And the other way round: the first declaration reaches evidence.
        let mut reversed = declared_twice.clone();
        let mut entries: Vec<(ClaimId, crate::model::Claim)> = declared_twice
            .claims
            .iter()
            .map(|(id, claim)| (id.clone(), claim.clone()))
            .collect();
        entries.reverse();
        reversed.claims = crate::model::Declarations::new(entries);
        protocols.push(("written".to_owned(), protocol));
        protocols.push(("declared twice".to_owned(), declared_twice));
        protocols.push(("declared twice, reversed".to_owned(), reversed));
        let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
        for at in 0..200 {
            let source = generated(&mut rng, 2 + at % 30);
            let protocol = crate::model::parse(&source).expect("generated protocol parses");
            protocols.push((source, protocol));
        }
        let mut inert = 0;
        for (name, protocol) in &protocols {
            let expected = inert_by_walking(protocol);
            inert += expected.len();
            assert_eq!(inert_reported(protocol), expected, "{name}");
        }
        assert_eq!(
            inert_reported(&protocols[protocols.len() - 203].1),
            [
                ("r1", "bare"),
                ("r2", "loop_a"),
                ("r2", "self"),
                ("r2", "loop_b"),
                ("r3", "twice"),
                ("r3", "bare"),
            ]
            .map(|(rule, claim)| (InvalidationRuleId::new(rule), ClaimId::new(claim)))
        );
        assert!(inert > 200, "inert problems compared: {inert}");
    }

    /// A chain of 4000 claims, each testing the next and named by a rule of its own, validates in
    /// under 10 s: the claims that reach evidence are found once, not once for each rule.
    #[test]
    fn a_chain_of_4000_claims_with_a_rule_each_validates_within_budget() {
        let claims = 4000;
        let mut source = String::from(
            "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {up: {}}\n\
             evidence_kinds: {k: {}}\nclaims:\n",
        );
        for at in 0..claims - 1 {
            source.push_str(&format!(
                "  c{at:05}: {{true_when: {{claim: c{:05}}}}}\n",
                at + 1
            ));
        }
        source.push_str(&format!(
            "  c{:05}: {{true_when: {{evidence: {{kind: k}}}}}}\ninvalidation:\n",
            claims - 1
        ));
        for at in 0..claims {
            source.push_str(&format!(
                "  r{at:05}: {{upstream: up, invalidates: [c{at:05}]}}\n"
            ));
        }
        let protocol = crate::model::parse(&source).expect("parses");
        let budget = std::time::Duration::from_secs(10);
        let (sent, received) = std::sync::mpsc::channel();
        std::thread::Builder::new()
            .name("validate-chain".to_owned())
            .stack_size(64 << 20)
            .spawn(move || {
                let started = std::time::Instant::now();
                let valid = validate(&protocol).is_ok();
                let _ = sent.send((started.elapsed(), valid));
            })
            .expect("a worker thread starts");
        let (took, valid) = received.recv_timeout(budget).unwrap_or_else(|_| {
            panic!("validating a chain of {claims} claims took more than {budget:?}")
        });
        eprintln!("validated a chain of {claims} claims in {took:?}");
        assert!(valid, "the chain is valid");
    }
}
