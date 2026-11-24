//! `canon digest`: reads one `canon-evidence/1` record and prints its digest, the one a
//! `ground_evidence` entry written `{id, digest}` gives (story:evidence-digest-binds-approvals).
//! The library reads the record and computes the digest; this file reads the file and prints.

use std::path::Path;
use std::process::ExitCode;

use b10x_canon::{eval, model};

use crate::{REJECTED, read_text, unreadable};

/// Prints `sha256:<64 lowercase hex>` and a newline and exits 0; a file that cannot be read exits
/// 2, and a file that is not a `canon-evidence/1` record exits 1, naming the code and the file
/// (`` error[malformed-input]: <file>: evidence is not a canon-evidence/1 document: … ``).
pub(crate) fn run(path: &Path) -> ExitCode {
    let text = match read_text(path) {
        Ok(text) => text,
        Err(error) => return unreadable(path, error),
    };
    match eval::digest_evidence(&text) {
        Ok(digest) => {
            println!("{digest}");
            ExitCode::SUCCESS
        }
        Err(refusal) => {
            eprintln!(
                "error[{}]: {}: {refusal}",
                refusal.code(),
                model::one_line(&path.display().to_string())
            );
            ExitCode::from(REJECTED)
        }
    }
}
