//! `canon conform run`: reads a registry directory and the fixtures its scenarios name, and hands
//! them to the library, which decides every verdict.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use b10x_canon::{conform, model};

use crate::{REJECTED, UNREADABLE, read_text};

/// Reads every `*.yaml` file of the registry directory in sorted file-name order, and hands the
/// texts and a fixture reader to the library, which decides every verdict.
pub(crate) fn run(scenarios: &Path) -> ExitCode {
    // The directory as given, rendered as the report renders every path.
    let shown = model::one_line(&scenarios.display().to_string());
    let entries = match std::fs::read_dir(scenarios) {
        Ok(entries) => entries,
        Err(error) => {
            eprintln!("error[unreadable]: {shown}: {error}");
            return ExitCode::from(UNREADABLE);
        }
    };
    let mut names = Vec::new();
    for entry in entries {
        match entry {
            Ok(entry) => names.push(entry.file_name()),
            Err(error) => {
                eprintln!("error[unreadable]: {shown}: {error}");
                return ExitCode::from(UNREADABLE);
            }
        }
    }
    names.retain(|name| Path::new(name).extension().is_some_and(|ext| ext == "yaml"));
    names.sort();

    let files: Vec<conform::ScenarioFile> = names
        .iter()
        .map(|name| {
            let path = scenarios.join(name);
            conform::ScenarioFile {
                path: path.display().to_string(),
                text: read_text(&path).map_err(|error| error.to_string()),
            }
        })
        .collect();
    let working_directory = std::env::current_dir().and_then(|dir| dir.canonicalize());
    let report = conform::run_registry(&files, |fixture| read_fixture(&working_directory, fixture));
    print!("{}", report.render());
    if report.all_passed() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(REJECTED)
    }
}

/// Reads a fixture a scenario names. The library has already refused a path that leaves the
/// working directory as written; this is the second check, against the filesystem: the path is
/// resolved (every symbolic link followed) and refused, unread, unless the result lies below the
/// resolved working directory. The resolved path is the one read.
fn read_fixture(
    working_directory: &std::io::Result<PathBuf>,
    fixture: &str,
) -> Result<String, conform::FixtureError> {
    let unreadable = |error: std::io::Error| conform::FixtureError::Unreadable(error.to_string());
    let root = working_directory.as_ref().map_err(|error| {
        conform::FixtureError::Unreadable(format!(
            "the working directory cannot be resolved: {error}"
        ))
    })?;
    let resolved = Path::new(fixture).canonicalize().map_err(unreadable)?;
    if !resolved.starts_with(root) {
        return Err(conform::FixtureError::Outside);
    }
    read_text(&resolved).map_err(unreadable)
}
