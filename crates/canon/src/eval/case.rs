//! The `canon-case/1` case snapshot as the evaluator reads and checks it. The type is
//! [`crate::model::Case`]; this file reads it from text or a YAML value and checks it against the
//! compiled protocol, its case inputs and its independence map (`independence.rs`) included. Whether a termination names a declared outcome is
//! checked by `outcomes.rs`.

use std::collections::BTreeSet;

use serde_yaml_ng::Value;

use super::read::{malformed, yaml};
use super::{Refusal, identifier, unsupported_format};
use crate::ir::Ir;
use crate::model::{ArtifactId, CASE_FORMAT, Case, CaseArtifact, InputId, one_line};

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
    if let Some(revision) = &case.revision {
        identifier("case revision", revision.as_str())?;
    }
    for (artifact, entry) in case.artifacts.iter() {
        identifier("case artifact identifier", artifact.as_str())?;
        identifier("artifact revision", entry.revision.as_str())?;
        descent(case, artifact, entry)?;
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
    inputs(ir, case)?;
    super::independence::check(ir, case)
}

/// Checks the revisions an artifact's current revision descends from (story:revision-ordering),
/// in the order written: each an identifier (`invalid-identifier`), listed once
/// (`duplicate-identifier`), and not the current revision itself, which does not descend from
/// itself (`self-descent`), each refusal naming the case, the artifact and the revision.
fn descent(case: &Case, artifact: &ArtifactId, entry: &CaseArtifact) -> Result<(), Refusal> {
    let mut seen = BTreeSet::new();
    for revision in entry.descends_from.iter().flatten() {
        identifier("artifact ancestor revision", revision.as_str())?;
        let named = || {
            format!(
                "case `{}` lists revision `{}` among those artifact `{}` descends from",
                one_line(case.id.as_str()),
                one_line(revision.as_str()),
                one_line(artifact.as_str())
            )
        };
        if !seen.insert(revision) {
            return Err(Refusal::new(
                "duplicate-identifier",
                format!("{} more than once", named()),
            ));
        }
        if *revision == entry.revision {
            return Err(Refusal::new(
                "self-descent",
                format!("{}, which is its current revision", named()),
            ));
        }
    }
    Ok(())
}

/// Checks the case's inputs against those the protocol declares: each input the case gives, in the
/// order written, is an identifier with an identifier for its value, is declared by the protocol
/// (`undeclared-input`), is given once (`duplicate-identifier`) and has a value its input declares
/// (`undeclared-input-value`, naming the value too); then each input the protocol declares, in
/// identifier order, is given by the case (`missing-input`). Each refusal names the input.
fn inputs(ir: &Ir, case: &Case) -> Result<(), Refusal> {
    let named = |input: &InputId| (one_line(case.id.as_str()), one_line(input.as_str()));
    let mut seen = BTreeSet::new();
    for (input, value) in case.inputs.iter() {
        identifier("case input identifier", input.as_str())?;
        identifier("input value", value.as_str())?;
        let (case_id, shown) = named(input);
        let Some(declared) = ir.inputs.get(input) else {
            return Err(Refusal::new(
                "undeclared-input",
                format!(
                    "case `{case_id}` gives input `{shown}`, which the protocol does not declare"
                ),
            ));
        };
        if !seen.insert(input) {
            return Err(Refusal::new(
                "duplicate-identifier",
                format!("case `{case_id}` gives input `{shown}` more than once"),
            ));
        }
        if !declared.values.contains(value) {
            return Err(Refusal::new(
                "undeclared-input-value",
                format!(
                    "case `{case_id}` gives input `{shown}` the value `{}`, which the input does not declare",
                    one_line(value.as_str())
                ),
            ));
        }
    }
    if let Some(missing) = ir.inputs.keys().find(|id| !seen.contains(id)) {
        let (case_id, shown) = named(missing);
        return Err(Refusal::new(
            "missing-input",
            format!("case `{case_id}` does not give a value for input `{shown}`"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROTOCOL: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\n\
        artifacts: {a: {}}\nevidence_kinds: {k: {}}\n";

    fn ir() -> Ir {
        crate::ir::compile(&crate::model::parse(PROTOCOL).expect("parses")).expect("compiles")
    }

    fn case(artifact: &str) -> Case {
        read_case(&format!(
            "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {{a: {artifact}}}\n"
        ))
        .expect("reads")
    }

    /// `descends_from` (story:revision-ordering): absent, empty and listing other revisions is
    /// accepted; an entry that is not an identifier, one listed twice and the current revision
    /// itself are refused, naming the case, the revision and the artifact.
    #[test]
    fn descends_from_is_refused_when_it_lists_a_revision_twice_or_the_current_one() {
        for accepted in [
            "{revision: r2}",
            "{revision: r2, descends_from: []}",
            "{revision: r2, descends_from: [r1, r0]}",
        ] {
            if let Err(refusal) = check(&ir(), &case(accepted)) {
                panic!("{accepted}: {refusal}");
            }
        }
        for (artifact, code, message) in [
            (
                "{revision: r2, descends_from: ['r 1']}",
                "invalid-identifier",
                "artifact ancestor revision `r 1` is empty or contains whitespace or a control character",
            ),
            (
                "{revision: r2, descends_from: [r1, r0, r1]}",
                "duplicate-identifier",
                "case `C-1` lists revision `r1` among those artifact `a` descends from more than once",
            ),
            (
                "{revision: r2, descends_from: [r1, r2]}",
                "self-descent",
                "case `C-1` lists revision `r2` among those artifact `a` descends from, which is its current revision",
            ),
        ] {
            let refusal = check(&ir(), &case(artifact)).expect_err(artifact);
            assert_eq!(refusal.code(), code, "{artifact}");
            assert_eq!(refusal.to_string(), message, "{artifact}");
        }
    }
}
