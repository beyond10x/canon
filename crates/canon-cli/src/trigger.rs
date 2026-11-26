//! `canon trigger validate`: read one `canon-trigger/1` document and print its rule key, or each
//! error with the field it names. `canon trigger protocols`: compile a set of `protocol/1`
//! documents and print the `canon-trigger-protocols/1` map a trigger evaluation reads
//! (story:protocol-access-class). `canon trigger evaluate`: read a rule set, a snapshot, the
//! protocols and a history, and print the `canon-trigger-plan/1` plan (story:trigger-evaluate).
//! `canon trigger digest` and `canon trigger check-set`: the digest of a rule, and a rule set checked
//! before it is published (story:trigger-rule-publication).
//! What a rule must hold, what a protocol's class and guard are and what a plan lists is the
//! library's (`b10x_canon::trigger`).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use b10x_canon::trigger::{DuplicateProtocolId, Refusal, RuleEntry};
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

/// Prints the `canon-trigger-plan/1` plan of the rule set against the snapshot and exits 0. Reads
/// the set, then each rule file it names in sorted order of its path (a relative path read from
/// the set file's directory), then the snapshot, the protocols and the history: the first file
/// that cannot be read exits 2; a document that does not read as its format exits 1 as
/// `error[malformed-input]: <path>: <why>`, and a rule file that is not a valid rule with each
/// error as `error[<code>]: <path>: <field>: <why>`. A refused evaluation exits 1 with
/// `error[<code>]: <why>`.
pub fn evaluate(
    rules: &Path,
    snapshot: &Path,
    protocols: &Path,
    history: &Path,
    now: &str,
) -> ExitCode {
    let set = match document(rules, trigger::read_set) {
        Ok(set) => set,
        Err(code) => return code,
    };
    let directory = rules.parent().unwrap_or(Path::new(""));
    let mut files: Vec<(PathBuf, model::TriggerRuleState)> = set
        .rules
        .iter()
        .map(|entry| {
            let path = Path::new(&entry.path);
            let path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                directory.join(path)
            };
            (path, entry.state)
        })
        .collect();
    files.sort();
    let mut entries = Vec::with_capacity(files.len());
    for (path, state) in files {
        let source = match crate::read_text(&path) {
            Ok(source) => source,
            Err(error) => return crate::unreadable(&path, error),
        };
        match trigger::read(&source) {
            Ok(rule) => entries.push(RuleEntry { rule, state }),
            Err(problems) => {
                for problem in &problems {
                    eprintln!("error[{}]: {}: {problem}", problem.code, shown(&path));
                }
                return ExitCode::from(REJECTED);
            }
        }
    }
    let snapshot = match document(snapshot, trigger::read_snapshot) {
        Ok(snapshot) => snapshot,
        Err(code) => return code,
    };
    let protocols = match document(protocols, trigger::read_protocols) {
        Ok(protocols) => protocols,
        Err(code) => return code,
    };
    let history = match document(history, trigger::read_history) {
        Ok(history) => history,
        Err(code) => return code,
    };
    match trigger::evaluate(
        &entries,
        &snapshot,
        &protocols,
        &history,
        &model::Instant::new(now),
    ) {
        Ok(plan) => {
            print!("{}", trigger::render_plan(&plan));
            ExitCode::SUCCESS
        }
        Err(refusal) => {
            eprintln!("error[{}]: {refusal}", refusal.code);
            ExitCode::from(REJECTED)
        }
    }
}

/// Prints the digest of one `canon-trigger/1` rule and a newline and exits 0; a document that is
/// not a valid rule prints one `error[<code>]: <field>: <why>` line per error, as `validate` does,
/// and exits 1; a file that cannot be read exits 2.
pub fn digest(path: &Path) -> ExitCode {
    let source = match crate::read_text(path) {
        Ok(source) => source,
        Err(error) => return crate::unreadable(path, error),
    };
    match trigger::read(&source) {
        Ok(rule) => {
            println!("{}", trigger::digest(&rule));
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

/// Prints the `canon-trigger-index/1` index of a rule set that passes its checks against the
/// published index and the protocols, and exits 0. Reads each rule file in sorted order of its
/// path, a path resolving to a file already read left out, then the index and the protocols: the
/// first file that cannot be read exits 2; a rule file that is not a valid rule exits 1 with each
/// error as `error[<code>]: <path>: <field>: <why>`, and an index or protocols document that does
/// not read as its format as `error[malformed-input]: <path>: <why>`. A set that fails exits 1
/// with one `error[<code>]: <rule key>: <why>` line per failure.
pub fn check_set(paths: &[PathBuf], published: &Path, protocols: &Path) -> ExitCode {
    let mut rules = Vec::new();
    for path in members(paths) {
        let source = match crate::read_text(path) {
            Ok(source) => source,
            Err(error) => return crate::unreadable(path, error),
        };
        match trigger::read(&source) {
            Ok(rule) => rules.push((path.display().to_string(), rule)),
            Err(problems) => {
                for problem in &problems {
                    eprintln!("error[{}]: {}: {problem}", problem.code, shown(path));
                }
                return ExitCode::from(REJECTED);
            }
        }
    }
    let published = match document(published, trigger::read_index) {
        Ok(published) => published,
        Err(code) => return code,
    };
    let protocols = match document(protocols, trigger::read_protocols) {
        Ok(protocols) => protocols,
        Err(code) => return code,
    };
    match trigger::check_set(&rules, &published, &protocols) {
        Ok(index) => {
            print!("{}", trigger::render_index(&index));
            ExitCode::SUCCESS
        }
        Err(failures) => {
            for failure in &failures {
                eprintln!("error[{}]: {failure}", failure.code);
            }
            ExitCode::from(REJECTED)
        }
    }
}

/// Reads one document with `read`: exit 2 when the file cannot be read, 1 naming the file when the
/// document is refused.
fn document<T>(path: &Path, read: fn(&str) -> Result<T, Refusal>) -> Result<T, ExitCode> {
    let source = crate::read_text(path).map_err(|error| crate::unreadable(path, error))?;
    read(&source).map_err(|refusal| {
        eprintln!("error[{}]: {}: {refusal}", refusal.code, shown(path));
        ExitCode::from(REJECTED)
    })
}

fn shown(path: &Path) -> String {
    model::one_line(&path.display().to_string())
}
