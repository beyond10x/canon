//! `canon diff --from --to`: the semantic difference between two revisions of one protocol. Reads
//! both compiled protocols and hands them to the library, which lists and classifies every change.

use std::path::Path;
use std::process::ExitCode;

use b10x_canon::{diff, eval, ir::Ir, model};

use crate::{REJECTED, read_text, unreadable};

/// Reads `--from` and `--to`, each `canon-ir/1` exactly as `canon compile` prints it, and prints the
/// library's report: a line naming the protocol and the two revisions, then one line per change,
/// ``<CATEGORY>: <section> `<id>` <what>``. A file that cannot be read exits 2; a file that is not
/// `canon-ir/1` exits 1, naming it (`error[malformed-input]: <file>: ...`), and so do two revisions
/// of different protocols (`error[different-protocol]: ...`).
pub(crate) fn run(from: &Path, to: &Path) -> ExitCode {
    let compiled = |path: &Path| -> Result<Ir, ExitCode> {
        let text = read_text(path).map_err(|error| unreadable(path, error))?;
        eval::read_ir(&text).map_err(|refusal| {
            eprintln!(
                "error[{}]: {}: {refusal}",
                refusal.code(),
                model::one_line(&path.display().to_string())
            );
            ExitCode::from(REJECTED)
        })
    };
    let from = match compiled(from) {
        Ok(ir) => ir,
        Err(code) => return code,
    };
    let to = match compiled(to) {
        Ok(ir) => ir,
        Err(code) => return code,
    };
    match diff::diff(&from, &to) {
        Ok(report) => {
            print!("{}", report.render());
            ExitCode::SUCCESS
        }
        Err(refusal) => {
            eprintln!("error[{}]: {refusal}", refusal.code());
            ExitCode::from(REJECTED)
        }
    }
}
