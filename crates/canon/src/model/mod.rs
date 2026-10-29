//! Canon's data model: the `protocol/1` source model, and the `canon-case/1`, `canon-evidence/1`
//! and `canon-decision/1` documents an evaluation reads and writes (`case.rs`, `evidence.rs`,
//! `decision.rs`), the `canon-authority/1` authority decisions (`authority.rs`) and
//! `canon-decisions/1` explicit decisions (`explicit.rs`) an evaluation may read, and the
//! `canon-properties/1` properties `canon check` reads (`properties.rs`), and the
//! `canon-requirements/1` catalogue `canon conform run --requirements` reads (`catalogue.rs`).
//!
//! A protocol document declares a protocol id and revision, the case inputs a case gives values for
//! (`case: inputs:`), artifacts, evidence kinds, claims with their predicates, obligations with
//! when they apply, actions, outcomes, invalidation rules and the independence dimensions its
//! evidence matches may demand. Every declaration
//! section is a map keyed by identifier; [`Declarations`] keeps the entries in source order and
//! keeps repeated keys, so the validator can report a duplicate identifier instead of the parser
//! silently dropping one.

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

use std::fmt;
use std::marker::PhantomData;

use serde::Deserialize;
use serde::de::{MapAccess, Visitor};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

pub use authority::{AUTHORITY_FORMAT, AuthorityDecision, Grant};
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
    EvidenceId, EvidenceKindId, IndependenceDimensionId, InputId, InputValue, Instant,
    InvalidationRuleId, ObligationId, OutcomeId, Principal, ProtocolId, Revision,
};
pub use parse::{FORMAT, ParseError, parse};
pub use predicate::{
    ClaimTest, EvidenceAfter, EvidenceMatch, EvidenceOrder, EvidenceSince, FirstResult, InputTest,
    LaterRecord, Predicate, Truth,
};
pub use properties::{
    PROPERTIES_FORMAT, Properties, Property, PropertyDependency, PropertyId, PropertySubject,
};
pub use requirement::OutcomeRequirement;

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

/// A thing whose revisions evidence is bound to.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    #[serde(default, deserialize_with = "present::optional")]
    pub description: Option<String>,
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
    #[serde(default, deserialize_with = "present::defaulted")]
    pub requires: Vec<CapabilityRequirement>,
    #[serde(default, deserialize_with = "present::optional")]
    pub effect: Option<EffectClass>,
    #[serde(default, deserialize_with = "present::defaulted")]
    pub may_produce: Vec<EvidenceProduction>,
    /// The JSON Schema, in Canon's argument-schema dialect, that the action's arguments satisfy.
    /// Absent: the action takes no arguments; a runner accepts only `{}`. Canon never validates
    /// arguments at run time: the runner that calls the action validates them against the
    /// schema, and `canon validate` holds the schema to the dialect (`action-arguments`).
    #[serde(default, deserialize_with = "present::optional_json")]
    pub arguments: Option<Json>,
}

/// One capability an action requires authority for.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityRequirement {
    #[serde(deserialize_with = "present::required")]
    pub capability: CapabilityId,
}

/// One evidence kind an action may produce.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceProduction {
    #[serde(deserialize_with = "present::required")]
    pub evidence: EvidenceKindId,
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
