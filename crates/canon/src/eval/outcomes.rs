//! The `outcomes` section of `canon-decision/1`: each declared outcome legitimate or blocked, and
//! the check that a case terminates only through a declared outcome. Not built yet
//! (story:outcomes): no section, and any termination is accepted.

use std::collections::BTreeMap;

use super::Refusal;
use super::decisions::Decisions;
use crate::ir::Ir;
use crate::model::{Case, ClaimId, EvidenceRecord, Json, Truth};

/// The section, or `None` to leave the slot empty; or a refusal of the case snapshot. `evidence`
/// is what the claims were evaluated over (after the exclusion stages); a requirement is evaluated
/// over it and `claims` with `super::claims::predicate`.
pub(super) fn section(
    _ir: &Ir,
    _case: &Case,
    _claims: &BTreeMap<ClaimId, Truth>,
    _evidence: &[EvidenceRecord],
    _decisions: Option<&Decisions>,
) -> Result<Option<Json>, Refusal> {
    Ok(None)
}
