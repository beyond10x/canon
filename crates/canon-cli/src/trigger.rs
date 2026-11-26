//! `canon trigger validate`: read one `canon-trigger/1` document and print its rule key, or each
//! error with the field it names. What a rule must hold is the library's (`b10x_canon::trigger`).

use std::path::Path;
use std::process::ExitCode;

use b10x_canon::{model, trigger};
use canon_cli::REJECTED;

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
