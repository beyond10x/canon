//! The `canon-decision/1` document as the evaluator writes it. The type is
//! [`crate::model::Decision`]; this file is its one serialization, canonical JSON (`json.rs`).
//!
//! The document's top-level keys are its sections: `format`, `case`, `protocol`,
//! `protocol_revision`, `claims`, and the slots `obligations`, `actions`, `outcomes` and
//! `explanation`. A slot that is absent is not written, and neither is a claim's
//! `excluded_evidence` list when it is empty, so a decision that fills no slot is written exactly
//! as `canon-decision/1` was before the slots existed.

use std::collections::{BTreeMap, BTreeSet};

use super::json::Value;
use crate::model::{ClaimDecision, Decision};

/// Every section the decision carries, keyed by its name.
fn sections(decision: &Decision) -> BTreeMap<&'static str, Value> {
    let claims = decision
        .claims
        .iter()
        .map(|(id, entry)| (id.as_str().to_owned(), claim(entry)))
        .collect();
    let mut sections = BTreeMap::from([
        ("format", Value::string(decision.format.as_str())),
        ("case", Value::string(decision.case.as_str())),
        ("protocol", Value::string(decision.protocol.as_str())),
        (
            "protocol_revision",
            Value::Integer(decision.protocol_revision),
        ),
        ("claims", Value::Object(claims)),
    ]);
    for (name, slot) in [
        ("obligations", &decision.obligations),
        ("actions", &decision.actions),
        ("outcomes", &decision.outcomes),
        ("explanation", &decision.explanation),
    ] {
        if let Some(value) = slot {
            sections.insert(name, Value::json(value));
        }
    }
    sections
}

fn claim(entry: &ClaimDecision) -> Value {
    let mut fields = vec![("value", Value::string(entry.value.to_string()))];
    if !entry.excluded_evidence.is_empty() {
        let excluded = entry
            .excluded_evidence
            .iter()
            .map(|exclusion| {
                Value::object([
                    ("evidence", Value::string(exclusion.evidence.as_str())),
                    ("reason", Value::string(exclusion.reason.as_str())),
                ])
            })
            .collect();
        fields.push(("excluded_evidence", Value::Array(excluded)));
    }
    Value::object(fields)
}

fn document(sections: impl IntoIterator<Item = (&'static str, Value)>) -> String {
    Value::object(sections).render()
}

/// The decision's canonical JSON, ending with a newline: object keys in code-point order, two-space
/// indentation, the string escaping `canon-ir/1` uses.
pub fn render(decision: &Decision) -> String {
    document(sections(decision))
}

/// The decision's canonical JSON holding only the `listed` sections, each written exactly as
/// [`render`] writes it. A listed section the decision does not carry is left out, so it shows up
/// as a difference from an expectation that lists it.
pub fn render_sections(decision: &Decision, listed: &BTreeSet<String>) -> String {
    document(
        sections(decision)
            .into_iter()
            .filter(|(name, _)| listed.contains(*name)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        CaseId, ClaimId, Declarations, EvidenceExclusion, EvidenceId, ExclusionReason, ProtocolId,
        Truth,
    };

    fn decision() -> Decision {
        Decision {
            format: "canon-decision/1".to_owned(),
            case: CaseId::new("C"),
            protocol: ProtocolId::new("p"),
            protocol_revision: 2,
            claims: Declarations::new(vec![(
                ClaimId::new("c"),
                ClaimDecision {
                    value: Truth::Unknown,
                    excluded_evidence: Vec::new(),
                },
            )]),
            obligations: None,
            actions: None,
            outcomes: None,
            explanation: None,
        }
    }

    const BARE: &str = "{\n  \"case\": \"C\",\n  \"claims\": {\n    \"c\": {\n      \"value\": \"unknown\"\n    }\n  },\n  \"format\": \"canon-decision/1\",\n  \"protocol\": \"p\",\n  \"protocol_revision\": 2\n}\n";

    #[test]
    fn empty_slots_are_not_written() {
        assert_eq!(render(&decision()), BARE);
    }

    #[test]
    fn filled_slots_and_exclusions_are_written() {
        let mut decision = decision();
        decision.outcomes = Some(serde_json::json!({"supported": "blocked"}));
        decision.explanation = Some(serde_json::json!([]));
        decision.claims = Declarations::new(vec![(
            ClaimId::new("c"),
            ClaimDecision {
                value: Truth::Unknown,
                excluded_evidence: vec![EvidenceExclusion {
                    evidence: EvidenceId::new("e1"),
                    reason: ExclusionReason::RevisionMismatch,
                }],
            },
        )]);
        assert_eq!(
            render(&decision),
            "{\n  \"case\": \"C\",\n  \"claims\": {\n    \"c\": {\n      \"excluded_evidence\": [\n        {\n          \"evidence\": \"e1\",\n          \"reason\": \"revision_mismatch\"\n        }\n      ],\n      \"value\": \"unknown\"\n    }\n  },\n  \"explanation\": [],\n  \"format\": \"canon-decision/1\",\n  \"outcomes\": {\n    \"supported\": \"blocked\"\n  },\n  \"protocol\": \"p\",\n  \"protocol_revision\": 2\n}\n"
        );
    }

    #[test]
    fn only_listed_sections_are_written() {
        let listed = |names: &[&str]| names.iter().map(|name| (*name).to_owned()).collect();
        assert_eq!(
            render_sections(&decision(), &listed(&["claims", "obligations"])),
            "{\n  \"claims\": {\n    \"c\": {\n      \"value\": \"unknown\"\n    }\n  }\n}\n"
        );
        assert_eq!(
            render_sections(
                &decision(),
                &listed(&["case", "claims", "format", "protocol", "protocol_revision"])
            ),
            BARE
        );
        assert_eq!(render_sections(&decision(), &listed(&[])), "{}\n");
    }
}
