//! The `obligations` section of `canon-decision/1`: each obligation open or discharged by its
//! discharge predicate over the claim values. Not built yet (story:obligations): no section.

use std::collections::BTreeMap;

use crate::ir::Ir;
use crate::model::{ClaimId, Json, Truth};

/// The section, or `None` to leave the slot empty. A discharge predicate tests only claim values
/// (the validator refuses evidence in one), so it is evaluated over `claims` alone with
/// `super::claims::predicate`.
pub(super) fn section(_ir: &Ir, _claims: &BTreeMap<ClaimId, Truth>) -> Option<Json> {
    None
}
