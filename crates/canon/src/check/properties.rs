//! Reading and validating the `canon-properties/1` document (`--properties`) against the compiled
//! protocol it is about.

use std::collections::BTreeSet;

use super::Refusal;
use crate::ir::Ir;
use crate::model::{
    PROPERTIES_FORMAT, Properties, PropertyDependency, PropertyId, PropertySubject, is_identifier,
    one_line,
};

/// Reads `canon-properties/1` text. YAML (or JSON) that is not such a document is refused as
/// `malformed-input`.
pub fn read_properties(source: &str) -> Result<Properties, Refusal> {
    serde_yaml_ng::from_str(source).map_err(|error| {
        Refusal::new(
            "malformed-input",
            format!(
                "`--properties` is not a canon-properties/1 document: {}",
                one_line(&error.to_string())
            ),
        )
    })
}

/// Checks `properties` against `ir`, the first problem found being the refusal, in this order:
/// the format (`unsupported-format`), the protocol id (`protocol-mismatch`), then property by
/// property in the order written, its id (`invalid-identifier`, `duplicate-identifier`), its
/// subject (`undeclared-action`, `undeclared-outcome`) and its claim (`undeclared-claim`).
pub(super) fn validate(ir: &Ir, properties: &Properties) -> Result<(), Refusal> {
    if properties.format != PROPERTIES_FORMAT {
        return Err(Refusal::new(
            "unsupported-format",
            format!(
                "format is `{}`, expected `{PROPERTIES_FORMAT}`",
                one_line(&properties.format)
            ),
        ));
    }
    if properties.protocol != ir.protocol.id {
        return Err(Refusal::new(
            "protocol-mismatch",
            format!(
                "the properties are about protocol `{}`, not `{}`",
                one_line(properties.protocol.as_str()),
                one_line(ir.protocol.id.as_str())
            ),
        ));
    }
    let mut seen: BTreeSet<&PropertyId> = BTreeSet::new();
    for (id, property) in properties.properties.iter() {
        let name = one_line(id.as_str());
        if !is_identifier(id.as_str()) {
            return Err(Refusal::new(
                "invalid-identifier",
                format!(
                    "property identifier `{name}` is empty or contains whitespace or a control \
                     character"
                ),
            ));
        }
        if !seen.insert(id) {
            return Err(Refusal::new(
                "duplicate-identifier",
                format!("property `{name}` is declared more than once"),
            ));
        }
        match &property.subject {
            PropertySubject::Action(action) if !ir.actions.contains_key(action) => {
                return Err(Refusal::new(
                    "undeclared-action",
                    format!(
                        "property `{name}` is about action `{}`, which is not declared",
                        one_line(action.as_str())
                    ),
                ));
            }
            PropertySubject::Outcome(outcome) if !ir.outcomes.contains_key(outcome) => {
                return Err(Refusal::new(
                    "undeclared-outcome",
                    format!(
                        "property `{name}` is about outcome `{}`, which is not declared",
                        one_line(outcome.as_str())
                    ),
                ));
            }
            _ => {}
        }
        let PropertyDependency::Claim(claim) = &property.independent_of;
        if !ir.claims.contains_key(claim) {
            return Err(Refusal::new(
                "undeclared-claim",
                format!(
                    "property `{name}` is independent of claim `{}`, which is not declared",
                    one_line(claim.as_str())
                ),
            ));
        }
    }
    Ok(())
}
