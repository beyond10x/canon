//! The `canon-case/1` case snapshot as the evaluator reads and checks it. The type is
//! [`crate::model::Case`]; this file reads it from text or a YAML value and checks it against the
//! compiled protocol. Whether a termination names a declared outcome is checked by `outcomes.rs`.

use std::collections::BTreeSet;

use serde_yaml_ng::Value;

use super::read::{malformed, yaml};
use super::{Refusal, identifier, unsupported_format};
use crate::ir::Ir;
use crate::model::{CASE_FORMAT, Case, one_line};

/// Reads one `canon-case/1` document: parsed as YAML, then read as [`case_from_value`] reads it.
pub fn read_case(text: &str) -> Result<Case, Refusal> {
    case_from_value(&yaml(text, "case is not a canon-case/1 document")?)
}

/// Reads one `canon-case/1` document already parsed as YAML.
pub fn case_from_value(value: &Value) -> Result<Case, Refusal> {
    serde_yaml_ng::from_value(value.clone())
        .map_err(|error| malformed("case is not a canon-case/1 document", error))
}

/// Checks the snapshot against the protocol, in the order the module docs of [`super`] give.
pub(super) fn check(ir: &Ir, case: &Case) -> Result<(), Refusal> {
    if case.format != CASE_FORMAT {
        return Err(unsupported_format(&case.format, CASE_FORMAT));
    }
    identifier("case identifier", case.id.as_str())?;
    identifier("case protocol identifier", case.protocol.as_str())?;
    for (artifact, entry) in case.artifacts.iter() {
        identifier("case artifact identifier", artifact.as_str())?;
        identifier("artifact revision", entry.revision.as_str())?;
    }
    if case.protocol != ir.protocol.id {
        return Err(Refusal::new(
            "protocol-mismatch",
            format!(
                "case `{}` is governed by protocol `{}`, not by `{}`",
                one_line(case.id.as_str()),
                one_line(case.protocol.as_str()),
                one_line(ir.protocol.id.as_str())
            ),
        ));
    }
    let mut seen = BTreeSet::new();
    for artifact in case.artifacts.ids() {
        if !ir.artifacts.contains_key(artifact) {
            return Err(Refusal::new(
                "undeclared-artifact",
                format!(
                    "case `{}` lists artifact `{}`, which the protocol does not declare",
                    one_line(case.id.as_str()),
                    one_line(artifact.as_str())
                ),
            ));
        }
        if !seen.insert(artifact) {
            return Err(Refusal::new(
                "duplicate-identifier",
                format!(
                    "case `{}` lists artifact `{}` more than once",
                    one_line(case.id.as_str()),
                    one_line(artifact.as_str())
                ),
            ));
        }
    }
    if let Some(missing) = ir.artifacts.keys().find(|id| !seen.contains(id)) {
        return Err(Refusal::new(
            "missing-artifact",
            format!(
                "case `{}` does not give the current revision of artifact `{}`",
                one_line(case.id.as_str()),
                one_line(missing.as_str())
            ),
        ));
    }
    Ok(())
}
