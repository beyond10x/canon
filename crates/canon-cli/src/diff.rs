//! `canon diff --from --to`: the semantic difference between two protocol revisions. Declared, not
//! built yet (story:semantic-diff): it refuses without reading either file.

use std::path::Path;
use std::process::ExitCode;

use crate::REJECTED;

/// Refuses as not built: `error[not-built]: ...`, exit 1.
pub(crate) fn run(_from: &Path, _to: &Path) -> ExitCode {
    eprintln!("error[not-built]: `canon diff` is not built yet");
    ExitCode::from(REJECTED)
}
