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
/// A protocol path that resolves outside the working directory, a protocol that does not parse, has
/// imports (naming its first) or does not compile, a refused generation and an output directory
/// that is not empty exit 1, naming the
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
    if let Some(first) = protocol.imports.ids().next() {
        eprintln!(
            "error[protocol-imports]: the protocol imports `{}`: a generated scenario names one \
             fixture, and a protocol with imports compiles only with its imported documents",
            model::one_line(first.as_str())
        );
        return ExitCode::from(REJECTED);
    }
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
    if let Err((failed, error)) = write_out(out, absent, &scenarios, &mut || {}) {
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

/// Writes the scenarios to `out`, creating it when `absent`; on failure removes what this run
/// created and returns the path that failed with the error. `before_rollback` runs after a failure
/// and before anything is removed: a test puts another writer's file there.
fn write_out(
    out: &Path,
    absent: bool,
    scenarios: &[generate::Scenario],
    before_rollback: &mut dyn FnMut(),
) -> Result<(), (PathBuf, std::io::Error)> {
    let mut created = Created::default();
    if let Err(failed) = write_all(out, absent, scenarios, &mut created) {
        before_rollback();
        created.remove();
        return Err(failed);
    }
    Ok(())
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

/// The rollback of `canon generate` in a shared output directory: another writer's file, put where
/// this run wrote one, stays.
#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh, empty directory under the crate's test scratch area (`target/tmp`).
    fn scratch(name: &str) -> PathBuf {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/tmp")
            .join(format!("generate-rollback-{name}"));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).expect("an old scratch directory is removable");
        }
        std::fs::create_dir_all(&dir).expect("a scratch directory is creatable");
        dir
    }

    fn scenario(file: &str) -> generate::Scenario {
        generate::Scenario {
            file: file.to_owned(),
            id: file.to_owned(),
            text: format!("generated {file}\n"),
        }
    }

    /// Scenario `a.yaml`, written first, then one whose name is over 255 bytes, which no common
    /// filesystem creates.
    fn second_unwritable() -> Vec<generate::Scenario> {
        vec![
            scenario("a.yaml"),
            scenario(&format!("{}.yaml", "b".repeat(260))),
        ]
    }

    fn entries(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .expect("readable")
            .map(|entry| {
                entry
                    .expect("an entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        names.sort();
        names
    }

    /// Another writer replaces `path`: writes its own file beside it and renames it over it.
    fn replace(path: &Path, text: &str) {
        let theirs = path.with_file_name(".another-writer");
        std::fs::write(&theirs, text).expect("the other writer writes");
        std::fs::rename(&theirs, path).expect("the other writer renames");
    }

    #[test]
    fn a_file_replaced_before_a_failed_write_is_rolled_back_stays() {
        let work = scratch("replaced-before-staging-failure");
        let out = work.join("out");
        std::fs::create_dir(&out).expect("creatable");
        let a = out.join("a.yaml");
        let result = write_out(&out, false, &second_unwritable(), &mut || {
            replace(&a, "another writer's a.yaml\n")
        });
        assert!(result.is_err(), "the second scenario cannot be written");
        assert_eq!(
            std::fs::read_to_string(&a).ok().as_deref(),
            Some("another writer's a.yaml\n"),
            "the rollback removed the file another writer put at {}",
            a.display()
        );
        assert_eq!(
            entries(&out),
            ["a.yaml"],
            "only the other writer's file is left"
        );
    }

    #[test]
    fn a_file_replaced_before_a_failed_move_is_rolled_back_stays() {
        let work = scratch("replaced-before-move-failure");
        let out = work.join("out");
        std::fs::create_dir(&out).expect("creatable");
        // Another writer made `b.yaml` after the emptiness check: the run cannot put its own there.
        std::fs::write(out.join("b.yaml"), "another writer's b.yaml\n").expect("writable");
        let a = out.join("a.yaml");
        let scenarios = [scenario("a.yaml"), scenario("b.yaml"), scenario("c.yaml")];
        let result = write_out(&out, false, &scenarios, &mut || {
            replace(&a, "another writer's a.yaml\n")
        });
        assert!(result.is_err(), "b.yaml is another writer's");
        assert_eq!(
            std::fs::read_to_string(&a).ok().as_deref(),
            Some("another writer's a.yaml\n"),
            "the rollback removed the file another writer put at {}",
            a.display()
        );
        assert_eq!(
            std::fs::read_to_string(out.join("b.yaml")).ok().as_deref(),
            Some("another writer's b.yaml\n")
        );
        assert_eq!(entries(&out), ["a.yaml", "b.yaml"]);
    }

    #[test]
    fn a_failed_run_leaves_the_output_directory_as_it_was() {
        let work = scratch("left-as-it-was");
        let out = work.join("out");
        std::fs::create_dir(&out).expect("creatable");
        std::fs::write(out.join("b.yaml"), "another writer's b.yaml\n").expect("writable");
        let scenarios = [scenario("a.yaml"), scenario("b.yaml"), scenario("c.yaml")];
        let result = write_out(&out, false, &scenarios, &mut || {});
        assert!(result.is_err(), "b.yaml is another writer's");
        assert_eq!(entries(&out), ["b.yaml"]);
        assert_eq!(
            std::fs::read_to_string(out.join("b.yaml")).ok().as_deref(),
            Some("another writer's b.yaml\n")
        );

        let result = write_out(&out, false, &second_unwritable(), &mut || {});
        assert!(result.is_err(), "the second scenario cannot be written");
        assert_eq!(entries(&out), ["b.yaml"]);
    }

    #[test]
    fn a_directory_replaced_before_rollback_stays() {
        let work = scratch("replaced-directory");
        let created = work.join("new");
        let out = created.join("out");
        let result = write_out(&out, true, &second_unwritable(), &mut || {
            // Another writer moves the directory this run created away and makes its own.
            std::fs::rename(&created, work.join("moved")).expect("the other writer renames");
            std::fs::create_dir(&created).expect("the other writer creates");
        });
        assert!(result.is_err(), "the second scenario cannot be written");
        assert!(
            created.is_dir(),
            "the rollback removed the directory another writer put at {}",
            created.display()
        );
    }

    /// Nothing reaches `--out` before every scenario is written: until then the run's files are in
    /// a directory of its own inside `--out`, mode 0700.
    #[cfg(unix)]
    #[test]
    fn a_run_stages_in_a_private_directory_until_every_file_is_written() {
        use std::os::unix::fs::PermissionsExt;
        let work = scratch("private-staging");
        let out = work.join("out");
        std::fs::create_dir(&out).expect("creatable");
        let mut seen = Vec::new();
        let result = write_out(&out, false, &second_unwritable(), &mut || {
            for name in entries(&out) {
                let mode = std::fs::symlink_metadata(out.join(&name))
                    .expect("an entry")
                    .permissions()
                    .mode();
                seen.push((name, mode & 0o7777, out.join("a.yaml").exists()));
            }
        });
        assert!(result.is_err(), "the second scenario cannot be written");
        assert_eq!(seen.len(), 1, "at the failure --out holds {seen:?}");
        let (name, mode, placed) = &seen[0];
        assert!(name.starts_with(".canon-generate-"), "{name}");
        assert_eq!(*mode, 0o700, "{name} has mode {mode:o}");
        assert!(
            !placed,
            "a.yaml was put in --out before the run had written every file"
        );
        assert!(entries(&out).is_empty(), "left {:?}", entries(&out));
    }

    #[test]
    fn a_run_that_succeeds_leaves_exactly_its_scenarios() {
        let work = scratch("success");
        let out = work.join("nested/out");
        let scenarios = [scenario("a.yaml"), scenario("b.yaml")];
        write_out(&out, true, &scenarios, &mut || panic!("no rollback")).expect("written");
        assert_eq!(entries(&out), ["a.yaml", "b.yaml"]);
        for scenario in &scenarios {
            assert_eq!(
                std::fs::read_to_string(out.join(&scenario.file)).expect("readable"),
                scenario.text
            );
        }
    }
}
