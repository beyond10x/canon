//! `canon-evidence/1` records as the evaluator reads and checks them. The type is
//! [`crate::model::EvidenceRecord`]; this file reads a record from text or a YAML value and checks
//! the set against the compiled protocol. Which records apply is decided afterwards, by the
//! exclusion stages (`binding.rs`, `freshness.rs`, `invalidation.rs`).

use std::collections::BTreeSet;

use serde_yaml_ng::Value;

use super::read::{malformed, yaml};
use super::{Refusal, identifier, unsupported_format};
use crate::ir::Ir;
use crate::model::{EVIDENCE_FORMAT, EvidenceRecord, one_line};

/// Reads one `canon-evidence/1` document: parsed as YAML, then read as [`evidence_from_value`]
/// reads it.
pub fn read_evidence(text: &str) -> Result<EvidenceRecord, Refusal> {
    evidence_from_value(&yaml(text, "evidence is not a canon-evidence/1 document")?)
}

/// Reads one `canon-evidence/1` document already parsed as YAML.
pub fn evidence_from_value(value: &Value) -> Result<EvidenceRecord, Refusal> {
    serde_yaml_ng::from_value(value.clone())
        .map_err(|error| malformed("evidence is not a canon-evidence/1 document", error))
}

/// Checks each record in the order given, as the module docs of [`super`] describe.
pub(super) fn check(ir: &Ir, evidence: &[EvidenceRecord]) -> Result<(), Refusal> {
    let mut seen = BTreeSet::new();
    for record in evidence {
        if record.format != EVIDENCE_FORMAT {
            return Err(unsupported_format(&record.format, EVIDENCE_FORMAT));
        }
        identifier("evidence identifier", record.id.as_str())?;
        identifier("evidence kind identifier", record.kind.as_str())?;
        identifier("evidence subject identifier", record.subject.as_str())?;
        identifier(
            "evidence subject revision",
            record.subject_revision.as_str(),
        )?;
        if !seen.insert(&record.id) {
            return Err(Refusal::new(
                "duplicate-identifier",
                format!(
                    "evidence `{}` is given more than once",
                    one_line(record.id.as_str())
                ),
            ));
        }
        if !ir.evidence_kinds.contains_key(&record.kind) {
            return Err(Refusal::new(
                "undeclared-evidence-kind",
                format!(
                    "evidence `{}` is of kind `{}`, which the protocol does not declare",
                    one_line(record.id.as_str()),
                    one_line(record.kind.as_str())
                ),
            ));
        }
    }
    Ok(())
}
