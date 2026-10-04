//! The `actions` section of `canon-decision/1`: which actions are admissible under the claim values
//! and the authority given. Not built yet (story:action-admissibility): no section.

use std::collections::BTreeMap;

use super::authority::Authority;
use crate::ir::Ir;
use crate::model::{ClaimId, EvidenceRecord, Json, Truth};

/// The section, or `None` to leave the slot empty. `evidence` is what the claims were evaluated
/// over (after the exclusion stages); a precondition is evaluated over it and `claims` with
/// `super::claims::predicate`.
pub(super) fn section(
    _ir: &Ir,
    _claims: &BTreeMap<ClaimId, Truth>,
    _evidence: &[EvidenceRecord],
    _authority: Option<&Authority>,
) -> Option<Json> {
    None
}
