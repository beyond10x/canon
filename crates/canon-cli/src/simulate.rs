//! `canon simulate`: reads the `protocol/1` document, composes and compiles it and hands it to the
//! library, which walks every state a case can reach from the empty case (story:canon-simulate,
//! with the rules refined after adversary pass 1).

use std::path::Path;
use std::process::ExitCode;

use b10x_canon::{ir, simulate};

use crate::{REJECTED, read_protocol, report};

/// Prints the report and exits 0 when the walk found nothing, 1 when it found something. A file
/// that cannot be read exits 2; a protocol that does not compose or compile, and a refused walk,
/// exit 1, naming the problem on standard error.
pub(crate) fn run(path: &Path) -> ExitCode {
    let protocol = match read_protocol(path) {
        Ok(protocol) => protocol,
        Err(code) => return code,
    };
    let compiled = match ir::compose(&protocol, &[]).and_then(|composed| ir::compile(&composed)) {
        Ok(compiled) => compiled,
        Err(problems) => return report(&problems),
    };
    match simulate::simulate(&compiled) {
        Ok(found) => {
            print!("{found}");
            if found.is_clean() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(REJECTED)
            }
        }
        Err(refusal) => {
            eprintln!("error[{}]: {refusal}", refusal.code());
            ExitCode::from(REJECTED)
        }
    }
}
