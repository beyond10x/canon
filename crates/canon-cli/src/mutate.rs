//! `canon mutate`: reads the protocol, the scenario directory and the fixtures its scenarios name,
//! and hands them to the library, which makes, runs and classifies every mutant.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use b10x_canon::{conform, model, mutate};

use crate::{REJECTED, read_text, unreadable};

/// Prints the report and exits 0 when it holds no weak point, 1 when it holds one. The protocol,
/// the scenario directory or a scenario file that cannot be read exits 2; a refusal exits 1 with
/// one `error[<code>]: …` line per reason on standard error and nothing on standard output.
pub(crate) fn run(path: &Path, scenarios: &Path) -> ExitCode {
    let protocol = match read_text(path) {
        Ok(text) => text,
        Err(error) => return unreadable(path, error),
    };
    let target = match path.canonicalize() {
        Ok(target) => target,
        Err(error) => return unreadable(path, error),
    };
    let files = match scenario_files(scenarios) {
        Ok(files) => files,
        Err(code) => return code,
    };
    let working_directory = std::env::current_dir().and_then(|dir| dir.canonicalize());
    let names_protocol =
        |fixture: &str| Path::new(fixture).canonicalize().is_ok_and(|p| p == target);
    let read = |fixture: &str| read_fixture(&working_directory, fixture);
    match mutate::mutate(&protocol, &files, names_protocol, read) {
        Ok(report) => {
            print!("{}", report.render());
            if report.weak_points() == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(REJECTED)
            }
        }
        Err(mutate::Refusal::NoScenario) => {
            eprintln!(
                "error[no-scenario]: no scenario of {} names {} as its fixture",
                model::one_line(&scenarios.display().to_string()),
                model::one_line(&path.display().to_string())
            );
            ExitCode::from(REJECTED)
        }
        Err(refusal) => {
            for (code, message) in refusal.lines() {
                eprintln!("error[{code}]: {message}");
            }
            ExitCode::from(REJECTED)
        }
    }
}

/// Every `*.yaml` file of the directory, in sorted file-name order, each named as the directory as
/// given joined with the file name, as `canon conform run` reads a registry. A directory or a file
/// that cannot be read exits 2, naming it.
fn scenario_files(scenarios: &Path) -> Result<Vec<conform::ScenarioFile>, ExitCode> {
    let entries = std::fs::read_dir(scenarios).map_err(|error| unreadable(scenarios, error))?;
    let mut names = Vec::new();
    for entry in entries {
        names.push(
            entry
                .map_err(|error| unreadable(scenarios, error))?
                .file_name(),
        );
    }
    names.retain(|name| Path::new(name).extension().is_some_and(|ext| ext == "yaml"));
    names.sort();
    names
        .iter()
        .map(|name| {
            let path = scenarios.join(name);
            let text = read_text(&path).map_err(|error| unreadable(&path, error))?;
            Ok(conform::ScenarioFile {
                path: path.display().to_string(),
                text: Ok(text),
            })
        })
        .collect()
}

/// Reads a fixture, or an import, a scenario names, as `canon conform run` does
/// (`crates/canon-cli/src/conform.rs`, `read_fixture`): the library has already refused a path that
/// leaves the working directory as written; this resolves it (every symbolic link followed) and
/// refuses it, unread, unless the result lies below the resolved working directory.
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
