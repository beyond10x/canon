//! `canon trigger validate`: read one `canon-trigger/1` document and print its rule key, or each
//! error with the field it names. `canon trigger protocols`: compile a set of `protocol/1`
//! documents and print the `canon-trigger-protocols/1` map a trigger evaluation reads
//! (story:protocol-access-class). What a rule must hold, and what a protocol's class and guard
//! are, is the library's (`b10x_canon::trigger`).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use b10x_canon::trigger::DuplicateProtocolId;
use b10x_canon::{model, trigger};
use canon_cli::REJECTED;

use crate::check_set::{compiled, members};

/// Prints `valid: trigger rule `<id>@<version>`` and exits 0, or one `error[<code>]: <field>:
/// <why>` line per error and exits 1; a file that cannot be read exits 2.
pub fn validate(path: &Path) -> ExitCode {
    let source = match crate::read_text(path) {
        Ok(source) => source,
        Err(error) => return crate::unreadable(path, error),
    };
    match trigger::read(&source) {
        Ok(rule) => {
            println!("valid: trigger rule `{}`", model::one_line(&rule.key()));
            ExitCode::SUCCESS
        }
        Err(problems) => {
            for problem in &problems {
                eprintln!("error[{}]: {problem}", problem.code);
            }
            ExitCode::from(REJECTED)
        }
    }
}

/// Prints the `canon-trigger-protocols/1` document of the set and exits 0. The paths are read in
/// sorted order, a path resolving to a file already read is left out, as `canon check-set` reads
/// its set; the first file that cannot be read exits 2, the first that does not parse, has
/// imports or does not compile exits 1, naming its path, and two files of one protocol id exit 1
/// as `duplicate-protocol-id`.
pub fn protocols(paths: &[PathBuf]) -> ExitCode {
    let sorted = members(paths);
    let mut set = Vec::with_capacity(sorted.len());
    for path in sorted {
        match compiled(path, "canon trigger protocols") {
            Ok(compiled) => set.push((path.display().to_string(), compiled)),
            Err(code) => return code,
        }
    }
    match trigger::protocols(set.iter().map(|(source, ir)| (source.as_str(), ir))) {
        Ok(document) => {
            print!("{}", trigger::render_protocols(&document));
            ExitCode::SUCCESS
        }
        Err(duplicate) => {
            eprintln!("error[{}]: {duplicate}", DuplicateProtocolId::CODE);
            ExitCode::from(REJECTED)
        }
    }
}
