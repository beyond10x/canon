//! `canon floor --floor --ir`: whether a protocol stays above a floor. Reads both compiled
//! protocols and hands them to the library, which classifies every change from the floor to the
//! protocol and prints what it found.

use std::path::Path;
use std::process::ExitCode;

use b10x_canon::{eval, floor, ir::Ir, model};

use crate::{REJECTED, read_text, unreadable};

/// Reads `--floor` and `--ir`, each `canon-ir/1` exactly as `canon compile` prints it, and prints
/// the library's verdict: `floor: holds` and the counts of `TIGHTENING`, `EXPANSION` and
/// `NO SEMANTIC CHANGE` changes with exit 0, or `floor: breached` and each `RELAXATION` and
/// `BREAKING` change with exit 1. A file that cannot be read exits 2; a file that is not
/// `canon-ir/1` exits 1, naming it (`error[malformed-input]: <file>: ...`).
pub(crate) fn run(floor_path: &Path, ir_path: &Path) -> ExitCode {
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
    let floor_ir = match compiled(floor_path) {
        Ok(ir) => ir,
        Err(code) => return code,
    };
    let protocol = match compiled(ir_path) {
        Ok(ir) => ir,
        Err(code) => return code,
    };
    match floor::check(&floor_ir, &protocol) {
        Ok(verdict) => {
            print!("{verdict}");
            if verdict.holds() {
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
