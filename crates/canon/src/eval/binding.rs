//! The revision-binding exclusion stage (design § 9): evidence applies only to the revision of its
//! subject it is bound to. Not built yet: this stage excludes nothing (story:evidence-revision-binding).

use super::Refusal;
use crate::ir::Ir;
use crate::model::{Case, EvidenceExclusion, EvidenceRecord};

/// The records of `evidence` this stage keeps from claim evaluation, each with the reason
/// `revision_mismatch`. `evidence` is what earlier stages left.
pub(super) fn exclude(
    _ir: &Ir,
    _case: &Case,
    _evidence: &[&EvidenceRecord],
) -> Result<Vec<EvidenceExclusion>, Refusal> {
    Ok(Vec::new())
}
