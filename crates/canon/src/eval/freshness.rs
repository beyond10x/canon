//! The freshness exclusion stage and the evaluation instant it reads (`--at`): evidence older than
//! its kind allows at that instant no longer applies. Not built yet (story:evidence-freshness): the
//! instant is refused when given, and the stage excludes nothing.

use super::{Refusal, unsupported_input};
use crate::ir::Ir;
use crate::model::{Case, EvidenceExclusion, EvidenceRecord};

/// The evaluation instant, read from the text given as `--at`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Instant;

/// Reads the evaluation instant. Refused when given, naming the flag, until it is read here.
pub(super) fn instant(text: Option<&str>) -> Result<Option<Instant>, Refusal> {
    match text {
        None => Ok(None),
        Some(_) => Err(unsupported_input("--at")),
    }
}

/// The records of `evidence` this stage keeps from claim evaluation, each with the reason
/// `expired`. `evidence` is what earlier stages left.
pub(super) fn exclude(
    _ir: &Ir,
    _case: &Case,
    _evidence: &[&EvidenceRecord],
    _at: Option<&Instant>,
) -> Result<Vec<EvidenceExclusion>, Refusal> {
    Ok(Vec::new())
}
