//! Canon's data model: the `protocol/1` source model, and the `canon-case/1`, `canon-evidence/1`
//! and `canon-decision/1` documents an evaluation reads and writes (`case.rs`, `evidence.rs`,
//! `decision.rs`), the `canon-decisions/1` explicit decisions an evaluation may read
//! (`explicit.rs`), and the `canon-properties/1` properties `canon check` reads (`properties.rs`).
//!
//! A protocol document declares a protocol id and revision, artifacts, evidence kinds, claims with
//! their predicates, obligations, actions, outcomes and invalidation rules. Every declaration
//! section is a map keyed by identifier; [`Declarations`] keeps the entries in source order and
//! keeps repeated keys, so the validator can report a duplicate identifier instead of the parser
//! silently dropping one.

mod case;
mod decision;
mod evidence;
mod explicit;
#[macro_use]
mod ids;
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

pub use case::{CASE_FORMAT, Case, CaseArtifact};
pub use decision::{
    ClaimDecision, DECISION_FORMAT, Decision, EvidenceExclusion, ExclusionReason, Json,
};
pub use evidence::{EVIDENCE_FORMAT, EvidenceRecord};
pub use explicit::{DECISIONS_FORMAT, ExplicitDecision};
pub use ids::{
    ActionId, Age, ArtifactId, CapabilityId, CaseId, ClaimId, DecisionName, EffectClass,
    EvidenceId, EvidenceKindId, Instant, InvalidationRuleId, ObligationId, OutcomeId, Principal,
    ProtocolId, Revision,
};
pub use parse::{FORMAT, ParseError, parse};
pub use predicate::{ClaimTest, EvidenceMatch, Predicate, Truth};
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
    #[serde(default, deserialize_with = "present::required")]
    pub artifacts: Declarations<ArtifactId, Artifact>,
    #[serde(default, deserialize_with = "present::required")]
    pub evidence_kinds: Declarations<EvidenceKindId, EvidenceKind>,
    #[serde(default, deserialize_with = "present::required")]
    pub claims: Declarations<ClaimId, Claim>,
    #[serde(default, deserialize_with = "present::required")]
    pub obligations: Declarations<ObligationId, Obligation>,
    #[serde(default, deserialize_with = "present::required")]
    pub actions: Declarations<ActionId, Action>,
    #[serde(default, deserialize_with = "present::required")]
    pub outcomes: Declarations<OutcomeId, Outcome>,
    /// The invalidation rules, keyed by rule id; a protocol without the section declares none.
    #[serde(default, deserialize_with = "present::required")]
    pub invalidation: Declarations<InvalidationRuleId, InvalidationRule>,
}

/// The protocol's identity: id and revision.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolHeader {
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
/// discharge predicate, over claim values, evaluates true (design § 8).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Obligation {
    #[serde(default, deserialize_with = "present::optional")]
    pub description: Option<String>,
    pub discharged_when: Predicate,
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
    #[serde(default, deserialize_with = "present::required")]
    pub requires: Vec<CapabilityRequirement>,
    #[serde(default, deserialize_with = "present::optional")]
    pub effect: Option<EffectClass>,
    #[serde(default, deserialize_with = "present::required")]
    pub may_produce: Vec<EvidenceProduction>,
}

/// One capability an action requires authority for.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityRequirement {
    pub capability: CapabilityId,
}

/// One evidence kind an action may produce.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceProduction {
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
