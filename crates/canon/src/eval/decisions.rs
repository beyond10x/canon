//! The explicit decisions an evaluation reads (`canon-decisions/1`), which outcome requirements may
//! name. Not built yet (story:decision-outcomes): explicit decisions are refused when given.

use super::{Refusal, unsupported_input};

/// The explicit decisions, read from the text given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Decisions;

/// Reads the explicit decisions. Refused when given until they are read here.
pub(super) fn read(text: Option<&str>) -> Result<Option<Decisions>, Refusal> {
    match text {
        None => Ok(None),
        Some(_) => Err(unsupported_input("decisions")),
    }
}
