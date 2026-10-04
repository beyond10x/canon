//! The invalidation exclusion stage: evidence an invalidation rule invalidates no longer applies.
//! Not built yet: this stage excludes nothing (story:invalidation-rules).

use super::Refusal;
use crate::ir::Ir;
use crate::model::{Case, EvidenceExclusion, EvidenceRecord};

/// The records of `evidence` this stage keeps from claim evaluation, each with the reason
/// `invalidated`. `evidence` is what earlier stages left.
pub(super) fn exclude(
    _ir: &Ir,
    _case: &Case,
    _evidence: &[&EvidenceRecord],
) -> Result<Vec<EvidenceExclusion>, Refusal> {
    Ok(Vec::new())
}
