//! `canon-evidence/1` records as the evaluator reads and checks them. The type is
//! [`crate::model::EvidenceRecord`]; this file reads a record from text or a YAML value and checks
//! the set against the compiled protocol. Which records apply is decided afterwards, by the
//! exclusion stages (`binding.rs`, `freshness.rs`, `invalidation.rs`).

use std::collections::BTreeSet;

use serde_yaml_ng::Value;

use super::freshness::invalid_instant;
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
        if let Some(observed_at) = &record.observed_at
            && observed_at.seconds().is_none()
        {
            return Err(invalid_instant(
                &format!("evidence `{}` observed_at", one_line(record.id.as_str())),
                observed_at.as_str(),
            ));
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ir() -> Ir {
        crate::ir::compile(
            &crate::model::parse(
                "format: protocol/1\nprotocol: {id: p, revision: 1}\nevidence_kinds: {k: {}}\n",
            )
            .expect("parses"),
        )
        .expect("compiles")
    }

    fn checked(observed_at: &str) -> Result<(), Refusal> {
        let record = read_evidence(&format!(
            "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r1\n\
             observed_at: {observed_at}\n"
        ))?;
        check(&ir(), &[record])
    }

    /// `observed_at` is optional, and when given it is an instant; an explicit null or any other
    /// text is refused.
    #[test]
    fn observed_at_is_an_instant_when_given() {
        assert_eq!(checked("2026-10-04T12:00:00Z"), Ok(()));
        let refusal = checked("2026-10-04T12:00:00+02:00").expect_err("refused");
        assert_eq!(refusal.code(), "invalid-instant");
        assert_eq!(
            refusal.to_string(),
            "evidence `e1` observed_at `2026-10-04T12:00:00+02:00` is not an instant in UTC written YYYY-MM-DDTHH:MM:SSZ"
        );
        assert_eq!(
            checked("~").expect_err("null refused").code(),
            "malformed-input"
        );
    }
}
