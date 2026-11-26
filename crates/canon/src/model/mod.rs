//! Canon's data model: the `protocol/1` source model, and the `canon-case/1`, `canon-evidence/1`
//! and `canon-decision/1` documents an evaluation reads and writes (`case.rs`, `evidence.rs`,
//! `decision.rs`), the `canon-authority/1` authority decisions (`authority.rs`) and
//! `canon-decisions/1` explicit decisions (`explicit.rs`) an evaluation may read, and the
//! `canon-properties/1` properties `canon check` reads (`properties.rs`), the
//! `canon-requirements/1` catalogue `canon conform run --requirements` reads (`catalogue.rs`), and
//! the `canon-trigger/1` rule `canon trigger validate` reads (`trigger.rs`).
//!
//! A protocol document declares a protocol id and revision, the case inputs a case gives values for
//! (`case: inputs:`), artifacts, evidence kinds, claims with their predicates, obligations with
//! when they apply, actions, outcomes, invalidation rules and the independence dimensions its
//! evidence matches may demand, and the protocols it imports, each pinned to a revision and a
//! digest. Every declaration section is a map keyed by identifier; [`Declarations`] keeps the
//! entries in source order and keeps repeated keys, so the validator can report a duplicate
//! identifier instead of the parser silently dropping one. [`write`] writes a protocol back out as
//! canonical JSON text that [`parse`] reads to an equal model.

// `ids` declares the `identifier!` macro, which a module can use only when declared after it.
#[macro_use]
mod ids;

mod authority;
mod case;
mod catalogue;
mod decision;
mod evidence;
mod explicit;
mod parse;
mod predicate;
mod present;
mod properties;
mod requirement;
mod time;
mod trigger;
mod write;

use std::fmt;
use std::marker::PhantomData;

use serde::Deserialize;
use serde::de::{MapAccess, Visitor};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

pub use authority::{
    AUTHORITY_FORMAT, AuthorityDecision, CitedEvidence, EvidenceGround, Grant, ScopeKey, ScopeValue,
};
pub use case::{CASE_FORMAT, Case, CaseArtifact};
pub use catalogue::{
    NormativeRequirement, REQUIREMENTS_FORMAT, RequirementCatalogue, RequirementId,
    RequirementStatus,
};
pub use decision::{
    ClaimDecision, DECISION_FORMAT, Decision, EvidenceExclusion, ExclusionReason, Json,
    ObligationStatus,
};
pub use evidence::{EVIDENCE_FORMAT, EvidenceRecord};
pub use explicit::{DECISIONS_FORMAT, ExplicitDecision};
pub use ids::{
    ActionId, Age, ArtifactId, CapabilityId, CaseId, ClaimId, DecisionName, EffectClass,
    EvidenceDigest, EvidenceId, EvidenceKindId, IndependenceDimensionId, InputId, InputValue,
    Instant, InvalidationRuleId, ItemId, ObligationId, OutcomeId, Principal, ProtocolId, Revision,
};
pub use parse::{FORMAT, ParseError, parse};
pub use predicate::{
    CASE_OUTCOME, ChildOutcomes, ChildRevision, ClaimTest, EvidenceAfter, EvidenceDescended,
    EvidenceMatch, EvidenceOrder, EvidenceSince, FirstResult, InputTest, LaterRecord, OPEN_OUTCOME,
    Predicate, Truth,
};
pub use properties::{
    PROPERTIES_FORMAT, Properties, Property, PropertyDependency, PropertyId, PropertySubject,
};
pub use requirement::{DecisionRequirement, OutcomeRequirement};
pub use trigger::{
    TRIGGER_FORMAT, TriggerAccess, TriggerField, TriggerFieldName, TriggerFieldType, TriggerInput,
    TriggerKind, TriggerMode, TriggerRefire, TriggerRule, TriggerRuleId, TriggerStart,
    TriggerState, TriggerSubject,
};
pub use write::write;

/// A `protocol/1` source document as written.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Protocol {
    /// The document format; the validator accepts only [`FORMAT`].
    #[serde(deserialize_with = "present::required")]
    pub format: String,
    pub protocol: ProtocolHeader,
    #[serde(default, deserialize_with = "present::defaulted")]
    pub artifacts: Declarations<ArtifactId, Artifact>,
    #[serde(default, deserialize_with = "present::defaulted")]
    pub evidence_kinds: Declarations<EvidenceKindId, EvidenceKind>,
    #[serde(default, deserialize_with = "present::defaulted")]
    pub claims: Declarations<ClaimId, Claim>,
    #[serde(default, deserialize_with = "present::defaulted")]
    pub obligations: Declarations<ObligationId, Obligation>,
    #[serde(default, deserialize_with = "present::defaulted")]
    pub actions: Declarations<ActionId, Action>,
    #[serde(default, deserialize_with = "present::defaulted")]
    pub outcomes: Declarations<OutcomeId, Outcome>,
    /// The invalidation rules, keyed by rule id; a protocol without the section declares none.
    #[serde(default, deserialize_with = "present::defaulted")]
    pub invalidation: Declarations<InvalidationRuleId, InvalidationRule>,
    /// What a case governed by the protocol gives when it opens: its case inputs. A protocol
    /// without the section declares no input.
    #[serde(default, deserialize_with = "present::optional")]
    pub case: Option<CaseSection>,
    /// The independence dimensions the protocol's evidence matches may demand, keyed by dimension
    /// id; a protocol without the section declares none.
    #[serde(default, deserialize_with = "present::defaulted")]
    pub independence_dimensions: Declarations<IndependenceDimensionId, IndependenceDimension>,
    /// The protocols this protocol imports, keyed by protocol id (story:protocol-imports), each
    /// pinned to a revision and a digest. A protocol without the section imports nothing; one with
    /// it compiles only composed with the documents it imports (`b10x_canon::ir::compose`).
    #[serde(default, deserialize_with = "present::defaulted")]
    pub imports: Declarations<ProtocolId, Import>,
}

/// One independence dimension: a relation between who produced a record and the case's work,
/// which the trusted context resolves for each producer and Canon reads from the case.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndependenceDimension {
    #[serde(default, deserialize_with = "present::optional")]
    pub description: Option<String>,
}

impl Protocol {
    /// The case inputs the protocol declares, in source order; none without a `case` section.
    pub fn inputs(&self) -> impl Iterator<Item = (&InputId, &CaseInput)> {
        self.case.iter().flat_map(|case| case.inputs.iter())
    }
}

/// The protocol's `case:` section.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseSection {
    /// The case inputs, keyed by input id.
    #[serde(default, deserialize_with = "present::defaulted")]
    pub inputs: Declarations<InputId, CaseInput>,
}

/// A case input: configuration a case fixes when it opens, one of a closed, non-empty list of
/// values. Not evidence, so no evaluation has an unknown input.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseInput {
    #[serde(default, deserialize_with = "present::optional")]
    pub description: Option<String>,
    /// The values the input may take.
    #[serde(deserialize_with = "present::required")]
    pub values: Vec<InputValue>,
}

/// The protocol's identity: id and revision.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolHeader {
    #[serde(deserialize_with = "present::required")]
    pub id: ProtocolId,
    pub revision: u64,
    #[serde(default, deserialize_with = "present::optional")]
    pub description: Option<String>,
}

/// One protocol a protocol imports: exactly the revision it reads and the digest of that
/// revision's compiled `canon-ir/1`, and the imported declarations the importing protocol may
/// redeclare.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Import {
    /// The revision of the imported protocol the import reads.
    pub revision: u64,
    /// `sha256:` and the 64 lowercase hexadecimal digits of the SHA-256 of the imported protocol's
    /// `canon-ir/1` bytes, compiled with its own imports (`b10x_canon::ir::digest`).
    #[serde(deserialize_with = "present::required")]
    pub digest: String,
    /// The identifiers, of any declaration section, of the imported declarations the importing
    /// protocol may redeclare; a redeclaration replaces the imported declaration.
    #[serde(default, deserialize_with = "present::defaulted")]
    pub refines: Vec<String>,
}

/// A thing whose revisions evidence is bound to.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    #[serde(default, deserialize_with = "present::optional")]
    pub description: Option<String>,
    /// The protocol whose cases revise this artifact (story:artifact-revised-by): any protocol
    /// but this one, named and never resolved. `canon evaluate`, `canon check` and `canon
    /// generate` read nothing from it; `canon simulate` offers a revise step for the artifact in
    /// every state. Absent: no protocol is said to revise it, or the protocol's own actions do,
    /// through `revises`.
    #[serde(default, deserialize_with = "present::optional")]
    pub revised_by: Option<ProtocolId>,
}

/// A kind of evidence the protocol admits.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceKind {
    #[serde(default, deserialize_with = "present::optional")]
    pub description: Option<String>,
    /// How old a record of this kind may be and still apply: at an evaluation instant later than
    /// its `observed_at` by more than this, it is excluded as expired (design § 8).
    #[serde(default, deserialize_with = "present::optional")]
    pub max_age: Option<Age>,
    /// The protocol whose cases produce records of this kind (story:supplied-evidence): any
    /// protocol but this one. No check of one protocol resolves it; `canon check-set`
    /// (`b10x_canon::ir::check_set`) reports, over a set of protocols, a supplier outside the set
    /// (`unknown-supplier`) and one no action of which may produce a kind of this id
    /// (`unproduced-supplied-kind`). A runner copies such a record unchanged
    /// from a case under that protocol; `canon check` counts the kind as produced. It counts as
    /// produced without authority and stays governed when an action that requires a capability
    /// may also produce it, so `canon check` reports `authority-bypass` on an outcome a supplied
    /// record alone makes legitimate. Absent: the protocol's own actions produce the kind, or
    /// nothing does.
    #[serde(default, deserialize_with = "present::optional")]
    pub supplied_by: Option<ProtocolId>,
    /// Whether the case's opener hands in a current record of this kind when it opens the case
    /// (story:supplied-evidence-required-at-open): `true`, a runner copies one into the case at
    /// open, or opens nothing; `false`, the default, written by leaving the key out, asks nothing.
    /// It needs `supplied_by` on the same kind (`at-open-without-supplier`). Canon opens nothing:
    /// `b10x_canon::eval::missing_at_open` names the required kinds a case lacks, and `canon
    /// evaluate` and `canon check` read nothing from it. `canon check-set` reports an action of a
    /// protocol of the set that opens a case under this one and whose precondition does not imply
    /// a record of the kind (`unsupplied-at-open`); `canon simulate` starts every walk with one
    /// record of it.
    #[serde(default, deserialize_with = "present::defaulted")]
    pub required_at_open: bool,
}

/// A proposition whose value is established from evidence by its predicate.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    #[serde(default, deserialize_with = "present::optional")]
    pub description: Option<String>,
    pub true_when: Predicate,
}

/// Something that must be done before the case can be complete. It is discharged only when its
/// discharge predicate, over claim values, evaluates true (design § 8). One with an applicability
/// predicate applies only when that predicate is not false: false, it is not applicable.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Obligation {
    #[serde(default, deserialize_with = "present::optional")]
    pub description: Option<String>,
    pub discharged_when: Predicate,
    /// When the obligation applies, over claim values and case inputs; always, when absent.
    #[serde(default, deserialize_with = "present::optional")]
    pub applicable_when: Option<Predicate>,
}

/// A semantically named possibility: what it needs, what authority it requires, what effect it
/// has and what evidence it may produce (design § 10).
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    #[serde(default, deserialize_with = "present::optional")]
    pub description: Option<String>,
    #[serde(default, deserialize_with = "present::optional")]
    pub precondition: Option<Predicate>,
    /// An assertion about when the action runs (story:ordering-guards-provable-by-check): it is
    /// `admissible` or `approval-required` only in a state where this predicate is `true`. Absent:
    /// no assertion. The grammar and reference checks of `precondition`; the literal `all: []`,
    /// true in every state, is refused (`vacuous-runs-only-when`), and only that literal: another
    /// guard true in every state (`{any: [{all: []}]}`, `{not: {any: []}}`) is not detected. No
    /// evaluation reads it: only `canon check` does, proving it in every state
    /// (`ordering-guard-failed`), per state and not along an order of actions, so a guard that
    /// holds only through the preconditions of earlier actions is proved only when the action's
    /// own precondition names it. A floor does not pin it: `canon floor` holds for a protocol that
    /// drops a guard its floor declares.
    #[serde(default, deserialize_with = "present::optional")]
    pub runs_only_when: Option<Predicate>,
    #[serde(default, deserialize_with = "present::defaulted")]
    pub requires: Vec<CapabilityRequirement>,
    #[serde(default, deserialize_with = "present::optional")]
    pub effect: Option<EffectClass>,
    #[serde(default, deserialize_with = "present::defaulted")]
    pub may_produce: Vec<EvidenceProduction>,
    /// The kinds one run of the action records together, all of them or none
    /// (story:simulate-record-groups). Absent: no group. Each group names two or more distinct
    /// kinds of `may_produce`, and no kind is in two groups of one action. It has no evaluation
    /// meaning: only `canon simulate` reads it, adding a group's records in one step and never
    /// one of its kinds alone.
    #[serde(default, deserialize_with = "present::defaulted")]
    pub records_together: Vec<RecordGroup>,
    /// The JSON Schema, in Canon's argument-schema dialect, that the action's arguments satisfy.
    /// Absent: the action takes no arguments; a runner accepts only `{}`. Canon evaluates no call:
    /// `canon validate` holds the schema to the dialect (`action-arguments`), and a runner checks
    /// a call's arguments with `validate::check_arguments` or any 2020-12 validator.
    #[serde(default, deserialize_with = "present::optional_json")]
    pub arguments: Option<Json>,
    /// The child case the action opens (story:case-composition): a case governed by another
    /// protocol, about a declared artifact. Absent: the action opens no case. Canon opens nothing
    /// itself; whoever takes the action opens the child case. A case has one commission, and a
    /// second party works through a child case: this is the one primitive by which a case hands
    /// work to another party (D207).
    #[serde(default, deserialize_with = "present::optional")]
    pub case_open: Option<CaseOpening>,
    /// Who writes the action's arguments and the content of the artifacts it revises
    /// (story:judgement-marking): `program`, a reproducible function of the action's inputs, or
    /// `judgement`, a person or a model. Absent: `program`. It has no evaluation meaning, names no
    /// executor, model or tool, and Canon cannot prove that a runner honours it.
    #[serde(default, deserialize_with = "present::optional")]
    pub author: Option<Author>,
    /// The declared artifacts the action may move to a new revision (story:judgement-marking).
    /// Absent: none. It has no evaluation meaning; Canon never refuses a runner that moves an
    /// artifact outside it.
    #[serde(default, deserialize_with = "present::defaulted")]
    pub revises: Vec<ArtifactId>,
}

/// Who decides (story:judgement-marking): a reproducible procedure, or a judgement by a person or
/// a model. On an action (`author`) it says who writes the action's arguments and the artifacts it
/// revises; on a production (`decided_by`) who decides the record's result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Author {
    /// A reproducible function of the action's inputs: the value when nothing is written.
    Program,
    /// A person or a model, whose result is not a function of the action's inputs.
    Judgement,
}

impl Author {
    /// The value as `protocol/1` and `canon-ir/1` write it.
    pub fn as_str(self) -> &'static str {
        match self {
            Author::Program => "program",
            Author::Judgement => "judgement",
        }
    }
}

/// What an action's `case_open` names: the protocol the child case is governed by and the
/// artifact it is about, never an existing case. No run-time check keeps a case from opening one
/// under its own protocol (D206): `canon validate` refuses an opening under the protocol's own id,
/// and the library function `b10x_canon::ir::check_openings` refuses a set of protocols whose
/// openings form a cycle, naming it in the order its openings go, from its least protocol id. No
/// check of one protocol resolves `protocol`; `canon check-set` (`b10x_canon::ir::check_set`)
/// reports, over a set of protocols, an opening under a protocol outside the set
/// (`unknown-opened-protocol`), an outcome of a child case named about `subject` that no protocol
/// opened about it declares (`undeclared-child-outcome`), and the cycle (`case-open-cycle`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseOpening {
    /// The protocol the child case is governed by.
    #[serde(deserialize_with = "present::required")]
    pub protocol: ProtocolId,
    /// The declared artifact the child case is about.
    #[serde(deserialize_with = "present::required")]
    pub subject: ArtifactId,
}

/// One capability an action requires authority for, at a scope (Atlas ADR 0083 § Decision 1).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityRequirement {
    #[serde(deserialize_with = "present::required")]
    pub capability: CapabilityId,
    /// The scope the capability is required at: one value for each key it names
    /// (`{cluster: prod-eu, namespace: foo}`), each key once. Absent or empty, the requirement is
    /// unscoped, and only an authority decision without a scope covers it.
    #[serde(default, deserialize_with = "present::defaulted")]
    pub scope: Declarations<ScopeKey, ScopeValue>,
}

/// One evidence kind an action may produce.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceProduction {
    #[serde(deserialize_with = "present::required")]
    pub evidence: EvidenceKindId,
    /// Who decides the result of a record of this kind the action produces
    /// (story:judgement-marking). Absent: the action's `author`. `program` on a judgement action
    /// covers a record the runner writes from the effect it applied, such as provenance. It has no
    /// evaluation meaning.
    #[serde(default, deserialize_with = "present::optional")]
    pub decided_by: Option<Author>,
}

/// Evidence kinds one run of an action records together, all of them or none
/// (story:simulate-record-groups).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordGroup {
    /// The kinds of the group: two or more distinct kinds the action may produce.
    #[serde(deserialize_with = "present::required")]
    pub kinds: Vec<EvidenceKindId>,
}

/// A legitimate terminal interpretation of a case and what it requires: a predicate, or an explicit
/// decision (`requires: decision: <name>`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    #[serde(default, deserialize_with = "present::optional")]
    pub description: Option<String>,
    pub requires: OutcomeRequirement,
}

/// An invalidation rule: a change of its upstream artifact's revision invalidates, for the claims
/// it names and every claim built on them, the evidence observed against an earlier revision of
/// that artifact (design § 4.1, CANON-INVALIDATION-001).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvalidationRule {
    #[serde(default, deserialize_with = "present::optional")]
    pub description: Option<String>,
    /// The artifact whose revision the rule watches.
    #[serde(deserialize_with = "present::required")]
    pub upstream: ArtifactId,
    /// The claims whose support a change of that revision invalidates.
    #[serde(deserialize_with = "present::required")]
    pub invalidates: Vec<ClaimId>,
}

/// The entries of one declaration section, in source order, repeated keys included.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declarations<K, V> {
    entries: Vec<(K, V)>,
}

impl<K, V> Default for Declarations<K, V> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<K: PartialEq, V> Declarations<K, V> {
    pub fn new(entries: Vec<(K, V)>) -> Self {
        Self { entries }
    }

    /// Entries in source order.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.entries.iter().map(|(id, value)| (id, value))
    }

    /// Identifiers in source order.
    pub fn ids(&self) -> impl Iterator<Item = &K> {
        self.entries.iter().map(|(id, _)| id)
    }

    /// The first declaration with this identifier.
    pub fn get(&self, id: &K) -> Option<&V> {
        self.entries
            .iter()
            .find(|(candidate, _)| candidate == id)
            .map(|(_, value)| value)
    }

    pub fn contains(&self, id: &K) -> bool {
        self.get(id).is_some()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<'de, K: Deserialize<'de>, V: Deserialize<'de>> Deserialize<'de> for Declarations<K, V> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct EntriesVisitor<K, V>(PhantomData<(K, V)>);

        impl<'de, K: Deserialize<'de>, V: Deserialize<'de>> Visitor<'de> for EntriesVisitor<K, V> {
            type Value = Declarations<K, V>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a map of declarations keyed by identifier")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut entries = Vec::new();
                while let Some((id, present::Required(value))) = map.next_entry()? {
                    entries.push((id, value));
                }
                Ok(Declarations { entries })
            }
        }

        deserializer.deserialize_map(EntriesVisitor(PhantomData))
    }
}

/// Whether a character could make rendered text read as something else: a backslash (the escape
/// character itself), a control character, whitespace other than a plain space, or a Unicode
/// format character (general category Cf: zero-width characters, bidirectional overrides, the
/// byte-order mark, the soft hyphen).
fn is_deceptive(c: char) -> bool {
    c == '\\'
        || c.is_control()
        || (c.is_whitespace() && c != ' ')
        || c.general_category() == GeneralCategory::Format
}

/// Renders text from a document so it stays on one line and cannot be mistaken for other text:
/// every character [`is_deceptive`] names is written as an escape. Every message that quotes
/// document text goes through this.
pub fn one_line(text: &str) -> String {
    let mut shown = String::with_capacity(text.len());
    for c in text.chars() {
        if is_deceptive(c) {
            shown.extend(c.escape_default());
        } else {
            shown.push(c);
        }
    }
    shown
}

/// Whether `id` is a well-formed identifier: not empty, and without whitespace, control characters
/// or Unicode format characters (general category Cf).
pub fn is_identifier(id: &str) -> bool {
    !id.is_empty()
        && !id.chars().any(|c| {
            c.is_whitespace() || c.is_control() || c.general_category() == GeneralCategory::Format
        })
}
