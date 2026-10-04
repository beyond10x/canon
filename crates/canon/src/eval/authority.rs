//! The authority decisions an evaluation reads (`--authority`, `canon-authority/1`). Not built
//! yet (story:action-admissibility): authority is refused when given, naming the flag.

use super::{Refusal, unsupported_input};

/// The authority decisions, read from the text given as `--authority`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Authority;

/// Reads the authority decisions. Refused when given, naming the flag, until they are read here.
pub(super) fn read(text: Option<&str>) -> Result<Option<Authority>, Refusal> {
    match text {
        None => Ok(None),
        Some(_) => Err(unsupported_input("--authority")),
    }
}
