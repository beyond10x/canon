//! `canon generate`: reads and compiles the `protocol/1` document, hands it to the library, which
//! finds every witness and writes each scenario's text, and writes the scenarios to the output
//! directory.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use b10x_canon::{conform, generate, ir, model};

use crate::{REJECTED, read_text, report, unreadable};

/// Prints `wrote: <file>` for each scenario written, in order, then `generated: protocol `<id>`
/// revision <n>: <N> scenarios`, and exits 0.
///
/// Every refusal comes before anything is written: the scenarios are built in memory first. Then
/// `--out` is created with any missing parents when absent, or an existing empty one (or a
/// symbolic link to one) is written into as it is, keeping its mode, owner and inode. On failure,
/// nothing this run created is left: each file and directory it created is removed, last first,
/// and nothing else.
/// A protocol path that resolves outside the working directory, a protocol that does not parse or
/// compile, a refused generation and an output directory that is not empty exit 1, naming the
/// problem on standard error. A protocol that cannot be read, or an output directory or file that
/// cannot be read, created or written, exits 2.
pub(crate) fn run(path: &Path, out: &Path) -> ExitCode {
    let Some(fixture) = path.to_str() else {
        eprintln!(
            "error[fixture-not-confined]: the protocol path `{}` is not valid UTF-8, so no \
             scenario can name it as its fixture",
            model::one_line(&path.display().to_string())
        );
        return ExitCode::from(REJECTED);
    };
    // The library refuses a path that leaves the working directory as written; this is the second
    // check, against the filesystem, as `canon conform run` makes it when it reads the fixture.
    if !fixture.is_empty() && conform::is_confined(fixture) {
        let resolved = match path.canonicalize() {
            Ok(resolved) => resolved,
            Err(error) => return unreadable(path, error),
        };
        let working_directory = match std::env::current_dir().and_then(|dir| dir.canonicalize()) {
            Ok(dir) => dir,
            Err(error) => return unreadable(Path::new("."), error),
        };
        if !resolved.starts_with(&working_directory) {
            eprintln!(
                "error[fixture-not-confined]: the protocol path `{}` resolves outside the working \
                 directory, so no scenario can name it as its fixture",
                model::one_line(fixture)
            );
            return ExitCode::from(REJECTED);
        }
    }
    let source = match read_text(path) {
        Ok(source) => source,
        Err(error) => return unreadable(path, error),
    };
    let protocol = match model::parse(&source) {
        Ok(protocol) => protocol,
        Err(error) => {
            eprintln!("error[parse]: {error}");
            return ExitCode::from(REJECTED);
        }
    };
    let compiled = match ir::compile(&protocol) {
        Ok(compiled) => compiled,
        Err(problems) => return report(&problems),
    };
    let scenarios = match generate::generate(&compiled, fixture, &source) {
        Ok(scenarios) => scenarios,
        Err(refusal) => {
            eprintln!("error[{}]: {refusal}", refusal.code());
            return ExitCode::from(REJECTED);
        }
    };

    let absent = match std::fs::read_dir(out) {
        Ok(mut entries) => {
            if entries.next().is_some() {
                eprintln!(
                    "error[out-not-empty]: {}: the output directory is not empty",
                    model::one_line(&out.display().to_string())
                );
                return ExitCode::from(REJECTED);
            }
            false
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => true,
        Err(error) => return unreadable(out, error),
    };
    let mut created = Created::default();
    if let Err((failed, error)) = write_all(out, absent, &scenarios, &mut created) {
        created.remove();
        return unreadable(&failed, error);
    }
    for scenario in &scenarios {
        let file = out.join(&scenario.file);
        println!("wrote: {}", model::one_line(&file.display().to_string()));
    }
    println!(
        "generated: protocol `{}` revision {}: {} {}",
        model::one_line(compiled.protocol.id.as_str()),
        compiled.protocol.revision,
        scenarios.len(),
        if scenarios.len() == 1 {
            "scenario"
        } else {
            "scenarios"
        }
    );
    ExitCode::SUCCESS
}

/// What this run created, in the order it was created.
#[derive(Default)]
struct Created(Vec<(PathBuf, bool)>);

impl Created {
    fn directory(&mut self, path: &Path) {
        self.0.push((path.to_path_buf(), true));
    }

    fn file(&mut self, path: &Path) {
        self.0.push((path.to_path_buf(), false));
    }

    /// Removes everything this run created, last first, and nothing else. A directory is removed
    /// only when empty.
    fn remove(self) {
        for (path, is_directory) in self.0.into_iter().rev() {
            let _ = if is_directory {
                std::fs::remove_dir(&path)
            } else {
                std::fs::remove_file(&path)
            };
        }
    }
}

/// Creates `out` and its missing parents when it is `absent`, then each scenario file in it, never
/// replacing a file that is there. Records each directory and file it creates in `created`; on
/// failure returns the path that failed with the error.
fn write_all(
    out: &Path,
    absent: bool,
    scenarios: &[generate::Scenario],
    created: &mut Created,
) -> Result<(), (PathBuf, std::io::Error)> {
    if absent {
        let missing: Vec<&Path> = out
            .ancestors()
            .take_while(|dir| !dir.as_os_str().is_empty() && !dir.exists())
            .collect();
        for dir in missing.into_iter().rev() {
            match std::fs::create_dir(dir) {
                Ok(()) => created.directory(dir),
                // Made by someone else meanwhile, or a `..` component: not ours to remove.
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && dir.is_dir() => {
                }
                Err(error) => return Err((dir.to_path_buf(), error)),
            }
        }
    }
    for scenario in scenarios {
        let path = out.join(&scenario.file);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| (path.clone(), error))?;
        created.file(&path);
        file.write_all(scenario.text.as_bytes())
            .map_err(|error| (path.clone(), error))?;
    }
    Ok(())
}
