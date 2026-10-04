//! Structured explanation of an evaluation: why each claim, obligation, action and outcome has the
//! value it has. Not built yet (story:explanation): no explanation.

use crate::ir::Ir;
use crate::model::{Decision, EvidenceRecord, Json};

/// An explanation, the payload of the decision's `explanation` slot: untyped, its shape fixed by
/// story:explanation's conformance scenarios.
pub type Explanation = Json;

/// The explanation of `decision`, or `None` to leave the slot empty.
pub fn explain(
    _ir: &Ir,
    _evidence: &[EvidenceRecord],
    _decision: &Decision,
) -> Option<Explanation> {
    None
}
