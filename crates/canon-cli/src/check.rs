//! `canon check`: reads the `protocol/1` document and the optional `--properties` file, compiles
//! the protocol and hands both to the library, which enumerates and evaluates every state.

use std::path::Path;
use std::process::ExitCode;

use b10x_canon::{check, ir};

use crate::{REJECTED, read_protocol, read_text, report, unreadable};

/// Prints the report and exits 0 when the check found nothing, 1 when it found something. A file
/// that cannot be read exits 2; a protocol that does not compile, properties that are refused and a
/// state space above the bound exit 1, naming the problem on standard error.
pub(crate) fn run(path: &Path, properties_path: Option<&Path>) -> ExitCode {
    let protocol = match read_protocol(path) {
        Ok(protocol) => protocol,
        Err(code) => return code,
    };
    let properties = match properties_path.map(read_text).transpose() {
        Ok(text) => text,
        Err(error) => {
            return unreadable(properties_path.expect("only a given path is read"), error);
        }
    };
    let compiled = match ir::compile(&protocol) {
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
