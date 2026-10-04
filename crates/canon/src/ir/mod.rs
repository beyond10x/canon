//! `canon-ir/1`, the normalized form evaluators consume (design § 30).
//!
//! [`compile`] is a pure function of a `protocol/1` document: it validates the document and, when it
//! is valid, returns its [`Ir`]. The IR differs from the source in exactly these ways:
//!
//! - **Canonical ordering.** Declarations are keyed and ordered by identifier, not by the order they
//!   are written in. The members of an `all` or `any` predicate are sorted by [`canonical_order`],
//!   and an action's capability requirements and the evidence kinds it may produce by identifier.
//! - **No repetition.** Each of those four lists holds every member once: in three-valued logic
//!   `all` and `any` are idempotent, and the two action lists are sets.
//! - **Explicit defaults.** Every field is present. An absent description or effect is `null`; an
//!   evidence match without a result has `"result": null`; a claim test without `is` has
//!   `"is": "true"`; an action without a precondition has the precondition `{"all": []}`, which is
//!   true; absent lists and sections are empty. The one exception is an evidence kind's
//!   `max_age`: written, as authored, only when the kind declares one, so the IR of a protocol
//!   that declares none is the IR it was before `max_age` existed.
//! - **No authoring sugar.** A claim test is always `{"claim": {"id": …, "is": …}}`, whichever way
//!   it was written. An outcome that requires an explicit decision has the requirement
//!   `{"decision": <name>}`, as written; any other requirement is its predicate.
//! - **Resolved references.** Only a document the validator accepts compiles, so every claim and
//!   evidence kind the IR references is declared in it.
//!
//! The IR carries the protocol id and revision (design § 37) and nothing about where the document
//! came from: no path, no working directory, no time. [`Ir::canonical_json`] is its one
//! serialization, and the bytes are suitable for hashing: documents that differ only in the ways
//! listed above give the same bytes. Other logical equivalences are not normalized, so they give
//! different bytes: a one-member `all` or `any` and its member, a nested `all` inside an `all` and
//! the flattened list, `not` of `not` and what it negates, and maximum ages of the same length
//! written in different units (`60m` and `1h`).

mod json;

use std::cmp::Ordering;
use std::collections::BTreeMap;

use crate::model::{
    self, ActionId, Age, ArtifactId, CapabilityId, ClaimId, ClaimTest, EffectClass, EvidenceKindId,
    EvidenceMatch, ObligationId, OutcomeId, OutcomeRequirement, Predicate, Protocol, ProtocolId,
    Truth,
};
use crate::validate::{self, Problem};

use json::Value;

/// The format this module produces.
pub const FORMAT: &str = "canon-ir/1";

/// A compiled protocol: every declaration keyed by identifier, every field explicit, every
/// predicate in canonical order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ir {
    pub protocol: Header,
    pub artifacts: BTreeMap<ArtifactId, Described>,
    pub evidence_kinds: BTreeMap<EvidenceKindId, EvidenceKind>,
    pub claims: BTreeMap<ClaimId, Claim>,
    pub obligations: BTreeMap<ObligationId, Obligation>,
    pub actions: BTreeMap<ActionId, Action>,
    pub outcomes: BTreeMap<OutcomeId, Outcome>,
}

/// The protocol's identity, which an evaluation records to say which protocol applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub id: ProtocolId,
    pub revision: u64,
    pub description: Option<String>,
}

/// A declaration that carries nothing but its description: an artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Described {
    pub description: Option<String>,
}

/// An evidence kind: its description, and how old a record of it may be and still apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceKind {
    pub description: Option<String>,
    /// Readable when compiled: the validator refuses a maximum age [`Age::seconds`] does not read.
    /// A caller can build an IR with one; the evaluator then refuses it as `invalid-max-age`.
    pub max_age: Option<Age>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    pub description: Option<String>,
    pub true_when: Predicate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Obligation {
    pub description: Option<String>,
    /// Discharges the obligation when it evaluates true.
    pub discharged_when: Predicate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    pub description: Option<String>,
    /// `{"all": []}`, which is true, when the source declares none.
    pub precondition: Predicate,
    /// Sorted by capability, each once.
    pub requires: Vec<CapabilityId>,
    pub effect: Option<EffectClass>,
    /// Sorted by evidence kind, each once.
    pub may_produce: Vec<EvidenceKindId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub description: Option<String>,
    /// A predicate, normalized like every predicate, or the explicit decision the outcome requires.
    pub requires: OutcomeRequirement,
}

/// Compiles a `protocol/1` document into `canon-ir/1`. A document the validator rejects does not
/// compile: the result is the validator's problems, unchanged.
pub fn compile(protocol: &Protocol) -> Result<Ir, Vec<Problem>> {
    validate::validate(protocol)?;

    let described = |description: &Option<String>| Described {
        description: description.clone(),
    };
    Ok(Ir {
        protocol: Header {
            id: protocol.protocol.id.clone(),
            revision: protocol.protocol.revision,
            description: protocol.protocol.description.clone(),
        },
        artifacts: keyed(&protocol.artifacts, |artifact| {
            described(&artifact.description)
        }),
        evidence_kinds: keyed(&protocol.evidence_kinds, |kind| EvidenceKind {
            description: kind.description.clone(),
            max_age: kind.max_age.clone(),
        }),
        claims: keyed(&protocol.claims, |claim| Claim {
            description: claim.description.clone(),
            true_when: normalize(&claim.true_when),
        }),
        obligations: keyed(&protocol.obligations, |obligation| Obligation {
            description: obligation.description.clone(),
            discharged_when: normalize(&obligation.discharged_when),
        }),
        actions: keyed(&protocol.actions, |action| Action {
            description: action.description.clone(),
            precondition: action
                .precondition
                .as_ref()
                .map_or(Predicate::All(Vec::new()), normalize),
            requires: unique(
                action
                    .requires
                    .iter()
                    .map(|requirement| requirement.capability.clone())
                    .collect(),
            ),
            effect: action.effect.clone(),
            may_produce: unique(
                action
                    .may_produce
                    .iter()
                    .map(|production| production.evidence.clone())
                    .collect(),
            ),
        }),
        outcomes: keyed(&protocol.outcomes, |outcome| Outcome {
            description: outcome.description.clone(),
            requires: match &outcome.requires {
                OutcomeRequirement::Predicate(predicate) => {
                    OutcomeRequirement::Predicate(normalize(predicate))
                }
                OutcomeRequirement::Decision(name) => OutcomeRequirement::Decision(name.clone()),
            },
        }),
    })
}

/// Keys a validated declaration section by identifier. Validation has refused duplicates, so no
/// entry is lost.
fn keyed<K: Ord + Clone, V, T>(
    section: &model::Declarations<K, V>,
    convert: impl Fn(&V) -> T,
) -> BTreeMap<K, T> {
    section
        .iter()
        .map(|(id, value)| (id.clone(), convert(value)))
        .collect()
}

/// Puts every `all` and `any` inside the predicate into canonical member order.
fn normalize(predicate: &Predicate) -> Predicate {
    match predicate {
        Predicate::All(members) => Predicate::All(sorted(members)),
        Predicate::Any(members) => Predicate::Any(sorted(members)),
        Predicate::Not(inner) => Predicate::Not(Box::new(normalize(inner))),
        Predicate::Evidence(_) | Predicate::Claim(_) => predicate.clone(),
    }
}

/// Normalizes each member, sorts them by [`canonical_order`] and keeps each member once.
fn sorted(members: &[Predicate]) -> Vec<Predicate> {
    let mut normalized: Vec<Predicate> = members.iter().map(normalize).collect();
    normalized.sort_by(canonical_order);
    normalized.dedup_by(|later, earlier| canonical_order(later, earlier) == Ordering::Equal);
    normalized
}

/// Sorts identifiers and keeps each once.
fn unique<T: Ord>(mut ids: Vec<T>) -> Vec<T> {
    ids.sort();
    ids.dedup();
    ids
}

/// The canonical order of predicates, which is the order of the members of every `all` and `any`
/// in `canon-ir/1`. Text (identifiers and results) compares by Unicode code point.
///
/// 1. Different forms compare by form, in the order `all`, `any`, `claim`, `evidence`, `not`.
/// 2. Two `all`s, or two `any`s, compare their members pairwise in order; the first pair that
///    differs decides, and when one list runs out first, the shorter list comes first.
/// 3. Two `not`s compare what they negate.
/// 4. Two evidence matches compare by kind, then by result: no result comes before any result, and
///    results compare as text.
/// 5. Two claim tests compare by claim, then by the value tested: `false`, `true`, `unknown`.
fn canonical_order(left: &Predicate, right: &Predicate) -> Ordering {
    fn form(predicate: &Predicate) -> u8 {
        match predicate {
            Predicate::All(_) => 0,
            Predicate::Any(_) => 1,
            Predicate::Claim(_) => 2,
            Predicate::Evidence(_) => 3,
            Predicate::Not(_) => 4,
        }
    }
    fn tested(value: Truth) -> u8 {
        match value {
            Truth::False => 0,
            Truth::True => 1,
            Truth::Unknown => 2,
        }
    }
    fn text(left: &str, right: &str) -> Ordering {
        left.chars().cmp(right.chars())
    }
    fn members(left: &[Predicate], right: &[Predicate]) -> Ordering {
        for (left, right) in left.iter().zip(right) {
            let order = canonical_order(left, right);
            if order != Ordering::Equal {
                return order;
            }
        }
        left.len().cmp(&right.len())
    }

    match (left, right) {
        (Predicate::All(left), Predicate::All(right))
        | (Predicate::Any(left), Predicate::Any(right)) => members(left, right),
        (Predicate::Not(left), Predicate::Not(right)) => canonical_order(left, right),
        (Predicate::Evidence(left), Predicate::Evidence(right)) => {
            text(left.kind.as_str(), right.kind.as_str()).then_with(|| {
                match (&left.result, &right.result) {
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => Ordering::Less,
                    (Some(_), None) => Ordering::Greater,
                    (Some(left), Some(right)) => text(left, right),
                }
            })
        }
        (Predicate::Claim(left), Predicate::Claim(right)) => {
            text(left.claim.as_str(), right.claim.as_str())
                .then_with(|| tested(left.is).cmp(&tested(right.is)))
        }
        _ => form(left).cmp(&form(right)),
    }
}

impl Ir {
    /// The canonical `canon-ir/1` serialization: the same IR always gives the same bytes.
    pub fn canonical_json(&self) -> String {
        self.value().render()
    }

    fn value(&self) -> Value {
        fn section<K: AsRef<str>, T>(map: &BTreeMap<K, T>, f: impl Fn(&T) -> Value) -> Value {
            Value::Object(
                map.iter()
                    .map(|(id, entry)| (id.as_ref().to_owned(), f(entry)))
                    .collect(),
            )
        }
        let described =
            |entry: &Described| Value::object([("description", description(&entry.description))]);

        Value::object([
            ("format", Value::string(FORMAT)),
            (
                "protocol",
                Value::object([
                    ("id", Value::string(self.protocol.id.as_str())),
                    ("revision", Value::Integer(self.protocol.revision)),
                    ("description", description(&self.protocol.description)),
                ]),
            ),
            ("artifacts", section(&self.artifacts, described)),
            (
                "evidence_kinds",
                section(&self.evidence_kinds, |kind| {
                    let mut fields = vec![("description", description(&kind.description))];
                    if let Some(max_age) = &kind.max_age {
                        fields.push(("max_age", Value::string(max_age.as_str())));
                    }
                    Value::object(fields)
                }),
            ),
            (
                "claims",
                section(&self.claims, |claim| {
                    Value::object([
                        ("description", description(&claim.description)),
                        ("true_when", predicate_value(&claim.true_when)),
                    ])
                }),
            ),
            (
                "obligations",
                section(&self.obligations, |obligation| {
                    Value::object([
                        ("description", description(&obligation.description)),
                        (
                            "discharged_when",
                            predicate_value(&obligation.discharged_when),
                        ),
                    ])
                }),
            ),
            (
                "actions",
                section(&self.actions, |action| {
                    Value::object([
                        ("description", description(&action.description)),
                        ("precondition", predicate_value(&action.precondition)),
                        (
                            "requires",
                            Value::Array(
                                action
                                    .requires
                                    .iter()
                                    .map(|capability| {
                                        Value::object([(
                                            "capability",
                                            Value::string(capability.as_str()),
                                        )])
                                    })
                                    .collect(),
                            ),
                        ),
                        (
                            "effect",
                            Value::optional(action.effect.as_ref().map(EffectClass::as_str)),
                        ),
                        (
                            "may_produce",
                            Value::Array(
                                action
                                    .may_produce
                                    .iter()
                                    .map(|kind| {
                                        Value::object([("evidence", Value::string(kind.as_str()))])
                                    })
                                    .collect(),
                            ),
                        ),
                    ])
                }),
            ),
            (
                "outcomes",
                section(&self.outcomes, |outcome| {
                    Value::object([
                        ("description", description(&outcome.description)),
                        ("requires", requirement_value(&outcome.requires)),
                    ])
                }),
            ),
        ])
    }
}

fn description(text: &Option<String>) -> Value {
    Value::optional(text.as_deref())
}

/// A predicate requirement as its predicate; a decision requirement as `{"decision": <name>}`.
fn requirement_value(requirement: &OutcomeRequirement) -> Value {
    match requirement {
        OutcomeRequirement::Predicate(predicate) => predicate_value(predicate),
        OutcomeRequirement::Decision(name) => {
            Value::object([("decision", Value::string(name.as_str()))])
        }
    }
}

fn predicate_value(predicate: &Predicate) -> Value {
    match predicate {
        Predicate::All(members) => Value::object([(
            "all",
            Value::Array(members.iter().map(predicate_value).collect()),
        )]),
        Predicate::Any(members) => Value::object([(
            "any",
            Value::Array(members.iter().map(predicate_value).collect()),
        )]),
        Predicate::Not(inner) => Value::object([("not", predicate_value(inner))]),
        Predicate::Evidence(EvidenceMatch { kind, result }) => Value::object([(
            "evidence",
            Value::object([
                ("kind", Value::string(kind.as_str())),
                ("result", Value::optional(result.as_deref())),
            ]),
        )]),
        Predicate::Claim(ClaimTest { claim, is }) => Value::object([(
            "claim",
            Value::object([
                ("id", Value::string(claim.as_str())),
                ("is", Value::string(truth(*is))),
            ]),
        )]),
    }
}

fn truth(value: Truth) -> &'static str {
    match value {
        Truth::True => "true",
        Truth::False => "false",
        Truth::Unknown => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::{FORMAT, compile, sorted};
    use crate::model::{self, ClaimId, ClaimTest, EvidenceKindId, EvidenceMatch, Predicate, Truth};

    fn claim(id: &str, is: Truth) -> Predicate {
        Predicate::Claim(ClaimTest {
            claim: ClaimId::new(id),
            is,
        })
    }

    fn evidence(kind: &str, result: Option<&str>) -> Predicate {
        Predicate::Evidence(EvidenceMatch {
            kind: EvidenceKindId::new(kind),
            result: result.map(str::to_owned),
        })
    }

    #[test]
    fn canonical_order_is_the_documented_order() {
        let ordered = vec![
            Predicate::All(Vec::new()),
            Predicate::All(vec![claim("c", Truth::True)]),
            Predicate::All(vec![claim("c", Truth::True), evidence("a", None)]),
            Predicate::Any(Vec::new()),
            claim("c", Truth::False),
            claim("c", Truth::True),
            claim("c", Truth::Unknown),
            claim("d", Truth::False),
            evidence("a", None),
            evidence("a", Some("x")),
            evidence("a", Some("y")),
            evidence("a", Some("\u{FF61}")),
            evidence("a", Some("\u{1F600}")),
            evidence("b", None),
            Predicate::Not(Box::new(claim("c", Truth::True))),
            Predicate::Not(Box::new(evidence("a", None))),
        ];
        let mut reversed = ordered.clone();
        reversed.reverse();
        assert_eq!(sorted(&reversed), ordered);
        let mut rotated = ordered.clone();
        rotated.rotate_left(7);
        assert_eq!(sorted(&rotated), ordered);
    }

    #[test]
    fn repeated_members_compile_to_the_same_bytes_as_the_deduplicated_copy() {
        let declarations = "evidence_kinds: {a: {}, b: {}}\nclaims:\n  c: {true_when: {all: []}}\n";
        let repeated = format!(
            "{HEADER}{declarations}outcomes:\n  o:\n    requires:\n      all:\n        - {{claim: c}}\n        - {{evidence: {{kind: a}}}}\n        - {{claim: c, is: true}}\n        - not: {{any: [{{evidence: {{kind: b}}}}, {{evidence: {{kind: b}}}}]}}\n        - any: [{{claim: c}}, {{evidence: {{kind: a}}}}]\n        - any: [{{evidence: {{kind: a}}}}, {{claim: c}}]\nactions:\n  act:\n    requires: [{{capability: k}}, {{capability: k}}]\n    may_produce: [{{evidence: a}}, {{evidence: a}}]\n"
        );
        let once = format!(
            "{HEADER}{declarations}outcomes:\n  o:\n    requires:\n      all:\n        - {{claim: c}}\n        - {{evidence: {{kind: a}}}}\n        - not: {{any: [{{evidence: {{kind: b}}}}]}}\n        - any: [{{claim: c}}, {{evidence: {{kind: a}}}}]\nactions:\n  act:\n    requires: [{{capability: k}}]\n    may_produce: [{{evidence: a}}]\n"
        );
        assert_eq!(ir(&repeated), ir(&once));
    }

    #[test]
    fn each_tested_value_compiles_to_its_name() {
        let compiled = ir(&format!(
            "{HEADER}claims:\n  c: {{true_when: {{all: []}}}}\noutcomes:\n  o:\n    requires:\n      any: [{{claim: c, is: unknown}}, {{claim: c, is: false}}, {{claim: c}}]\n"
        ));
        let names: Vec<&str> = compiled
            .lines()
            .filter_map(|line| line.trim().strip_prefix("\"is\": "))
            .collect();
        assert_eq!(
            names,
            ["\"false\"", "\"true\"", "\"unknown\""],
            "{compiled}"
        );
    }

    #[test]
    fn obligations_compile_keyed_by_identifier_with_their_descriptions() {
        let compiled = ir(&format!(
            "{HEADER}claims: {{c: {{true_when: {{all: []}}}}}}\nobligations:\n  z: {{description: Last., discharged_when: {{claim: c}}}}\n  a: {{discharged_when: {{any: [{{claim: c, is: unknown}}, {{claim: c, is: false}}]}}}}\n"
        ));
        assert!(
            compiled.contains(
                "  \"obligations\": {\n    \"a\": {\n      \"description\": null,\n      \"discharged_when\": {\n        \"any\": [\n          {\n            \"claim\": {\n              \"id\": \"c\",\n              \"is\": \"false\"\n            }\n          },\n          {\n            \"claim\": {\n              \"id\": \"c\",\n              \"is\": \"unknown\"\n            }\n          }\n        ]\n      }\n    },\n    \"z\": {\n      \"description\": \"Last.\",\n      \"discharged_when\": {\n        \"claim\": {\n          \"id\": \"c\",\n          \"is\": \"true\"\n        }\n      }\n    }\n  },\n"
            ),
            "{compiled}"
        );
    }

    #[test]
    fn an_obligation_without_a_discharge_predicate_does_not_parse() {
        assert!(
            model::parse(&format!("{HEADER}obligations:\n  o: {{description: D.}}\n")).is_err()
        );
    }

    #[test]
    fn a_written_precondition_is_normalized_like_every_predicate() {
        let declarations = "evidence_kinds: {a: {}}\nclaims:\n  c: {true_when: {all: []}}\n";
        let compiled = ir(&format!(
            "{HEADER}{declarations}actions:\n  act:\n    precondition:\n      any: [{{evidence: {{kind: a}}}}, {{claim: c}}, {{evidence: {{kind: a}}}}]\n"
        ));
        assert!(
            compiled.contains(
                "      \"precondition\": {\n        \"any\": [\n          {\n            \"claim\": {\n              \"id\": \"c\",\n              \"is\": \"true\"\n            }\n          },\n          {\n            \"evidence\": {\n              \"kind\": \"a\",\n              \"result\": null\n            }\n          }\n        ]\n      },\n"
            ),
            "{compiled}"
        );
    }

    fn ir(source: &str) -> String {
        compile(&model::parse(source).expect("parses"))
            .expect("compiles")
            .canonical_json()
    }

    const HEADER: &str = "format: protocol/1\nprotocol: {id: p, revision: 3}\n";

    #[test]
    fn predicate_members_compile_in_one_order_whatever_order_they_are_written_in() {
        let declarations = "evidence_kinds: {a: {}, b: {}}\nclaims:\n  c: {true_when: {all: [{evidence: {kind: a}}]}}\n  d: {true_when: {all: [{evidence: {kind: b}}]}}\n";
        let written = format!(
            "{HEADER}{declarations}outcomes:\n  o:\n    requires:\n      any:\n        - not: {{any: [{{claim: d}}, {{evidence: {{kind: b, result: x}}}}, {{claim: c, is: unknown}}]}}\n        - all: [{{evidence: {{kind: a}}}}, {{claim: c}}]\n"
        );
        let reordered = format!(
            "{HEADER}{declarations}outcomes:\n  o:\n    requires:\n      any:\n        - all: [{{claim: c, is: true}}, {{evidence: {{kind: a}}}}]\n        - not: {{any: [{{claim: c, is: unknown}}, {{evidence: {{result: x, kind: b}}}}, {{claim: d}}]}}\n"
        );
        assert_eq!(ir(&written), ir(&reordered));
    }

    #[test]
    fn action_lists_are_sorted_each_once_and_an_absent_precondition_is_an_explicit_true() {
        let source = format!(
            "{HEADER}evidence_kinds: {{a: {{}}, b: {{}}}}\nactions:\n  act:\n    effect: write\n    requires: [{{capability: z}}, {{capability: m}}, {{capability: z}}]\n    may_produce: [{{evidence: b}}, {{evidence: a}}, {{evidence: b}}]\n"
        );
        let compiled = ir(&source);
        assert!(
            compiled.contains(
                "  \"actions\": {\n    \"act\": {\n      \"description\": null,\n      \"effect\": \"write\",\n      \"may_produce\": [\n        {\n          \"evidence\": \"a\"\n        },\n        {\n          \"evidence\": \"b\"\n        }\n      ],\n      \"precondition\": {\n        \"all\": []\n      },\n      \"requires\": [\n        {\n          \"capability\": \"m\"\n        },\n        {\n          \"capability\": \"z\"\n        }\n      ]\n    }\n  },\n"
            ),
            "{compiled}"
        );
    }

    #[test]
    fn the_ir_names_its_format_and_the_protocol_id_and_revision() {
        let compiled = ir(HEADER);
        assert_eq!(
            compiled,
            format!(
                "{{\n  \"actions\": {{}},\n  \"artifacts\": {{}},\n  \"claims\": {{}},\n  \"evidence_kinds\": {{}},\n  \"format\": \"{FORMAT}\",\n  \"obligations\": {{}},\n  \"outcomes\": {{}},\n  \"protocol\": {{\n    \"description\": null,\n    \"id\": \"p\",\n    \"revision\": 3\n  }}\n}}\n"
            )
        );
    }

    #[test]
    fn a_document_the_validator_rejects_does_not_compile() {
        let protocol =
            model::parse("format: protocol/1\nprotocol: {id: p, revision: 1}\noutcomes: {o: {requires: {claim: missing}}}\n")
                .expect("parses");
        let problems = compile(&protocol).expect_err("refused");
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].code(), "undeclared-claim");
    }

    /// An outcome that requires an explicit decision compiles to `{"decision": <name>}`.
    #[test]
    fn a_decision_requirement_compiles_to_its_name() {
        let compiled = ir(&format!(
            "{HEADER}outcomes:\n  inconclusive: {{requires: {{decision: explicitly_inconclusive}}}}\n"
        ));
        assert!(
            compiled.contains(
                "  \"outcomes\": {\n    \"inconclusive\": {\n      \"description\": null,\n      \"requires\": {\n        \"decision\": \"explicitly_inconclusive\"\n      }\n    }\n  },\n"
            ),
            "{compiled}"
        );
    }

    /// A maximum age is written as authored, and only on the kind that declares one.
    #[test]
    fn a_maximum_age_is_written_only_where_declared() {
        let compiled = ir(&format!(
            "{HEADER}evidence_kinds: {{fresh: {{max_age: 90m}}, plain: {{description: d}}}}\n"
        ));
        assert!(
            compiled.contains(
                "  \"evidence_kinds\": {\n    \"fresh\": {\n      \"description\": null,\n      \"max_age\": \"90m\"\n    },\n    \"plain\": {\n      \"description\": \"d\"\n    }\n  },\n"
            ),
            "{compiled}"
        );
    }
}
