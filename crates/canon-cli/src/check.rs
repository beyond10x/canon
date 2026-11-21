//! `canon check`: reads the `protocol/1` document, the documents it imports and the optional
//! `--properties` file, composes and compiles the protocol and hands both to the library, which
//! decides each outcome, action and property over the states of the dimensions it reads, its
//! slice, and so covers every state of the protocol while the bound holds each slice.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use b10x_canon::{check, ir};

use crate::{REJECTED, read_imports, read_protocol, read_text, report, unreadable};

/// Prints the report and exits 0 when the check found nothing, 1 when it found something. A file
/// that cannot be read exits 2; a protocol that does not compose or compile, properties that are
/// refused and a subject whose slice is above the bound exit 1, naming the problem on standard
/// error.
pub(crate) fn run(path: &Path, properties_path: Option<&Path>, imports: &[PathBuf]) -> ExitCode {
    let protocol = match read_protocol(path) {
        Ok(protocol) => protocol,
        Err(code) => return code,
    };
    let imported = match read_imports(imports) {
        Ok(imported) => imported,
        Err(code) => return code,
    };
    let properties = match properties_path.map(read_text).transpose() {
        Ok(text) => text,
        Err(error) => {
            return unreadable(properties_path.expect("only a given path is read"), error);
        }
    };
    let compiled =
        match ir::compose(&protocol, &imported).and_then(|composed| ir::compile(&composed)) {
            Ok(compiled) => compiled,
            Err(problems) => return report(&problems),
        };
    let result = properties
        .as_deref()
        .map(check::read_properties)
        .transpose()
        .and_then(|properties| check::check(&compiled, properties.as_ref()));
    match result {
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
