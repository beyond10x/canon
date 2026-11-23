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
/// symbolic link to one) is written into as it is, keeping its mode, owner and inode. The files
/// are written into a private directory of the run inside `--out` and moved into place only when
/// every one is written. On failure, nothing this run created is left, and nothing else is removed:
/// a file or directory is removed only while it is still the one this run made there.
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

/// Writes the scenarios to `out`, creating it with any missing parents when `absent`. Each file is
/// written first into a private directory of this run inside `out` (so on the same filesystem,
/// mode 0700, a name no other process uses) and moved into place, never replacing a file that is
/// there, only when every one is written. On failure removes what this run made, each only while it
/// is still the one this run made, and returns the path that failed with the error.
/// `before_rollback` runs after a failure and before anything is removed: a test puts another
/// writer's file there.
fn write_out(
    out: &Path,
    absent: bool,
    scenarios: &[generate::Scenario],
    before_rollback: &mut dyn FnMut(),
) -> Result<(), (PathBuf, std::io::Error)> {
    write_out_with(
        out,
        absent,
        scenarios,
        &mut Hooks {
            staging_made: &mut |_| {},
            staging_open: &mut |_| {},
            link: &|staged, path| std::fs::hard_link(staged, path),
            before_rollback,
        },
    )
}

/// Where a test steps into a run of [`write_out`]: another process that writes `--out` acts there.
struct Hooks<'a> {
    /// Runs with the staging path once the staging directory is made and its identity read, before
    /// it is opened.
    staging_made: &'a mut dyn FnMut(&Path),
    /// Runs with the staging path once the staging directory is open and checked, before any file
    /// is written in it.
    staging_open: &'a mut dyn FnMut(&Path),
    /// Places a staged file (the first path) in `--out` (the second): a hard link.
    link: &'a dyn Fn(&Path, &Path) -> std::io::Result<()>,
    /// Runs after a failure and before anything is removed.
    before_rollback: &'a mut dyn FnMut(),
}

/// [`write_out`], with the steps a test takes.
fn write_out_with(
    out: &Path,
    absent: bool,
    scenarios: &[generate::Scenario],
    hooks: &mut Hooks<'_>,
) -> Result<(), (PathBuf, std::io::Error)> {
    let mut made = Made::default();
    let written = write_all(out, absent, scenarios, &mut made, hooks);
    if let Err(failed) = written {
        (hooks.before_rollback)();
        made.roll_back();
        return Err(failed);
    }
    Ok(())
}

/// What a file or directory this run made is: compared with what is at its path before removing it.
/// A directory is the same while its device and inode are; a file, while its device, inode, length
/// and modification time are.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Identity {
    directory: bool,
    #[cfg(unix)]
    device: u64,
    #[cfg(unix)]
    inode: u64,
    len: u64,
    modified: Option<std::time::SystemTime>,
}

impl Identity {
    fn of(metadata: &std::fs::Metadata) -> Self {
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt as _;
        let directory = metadata.is_dir();
        Identity {
            directory,
            #[cfg(unix)]
            device: metadata.dev(),
            #[cfg(unix)]
            inode: metadata.ino(),
            // A directory's length and time change with its entries.
            len: if directory { 0 } else { metadata.len() },
            modified: if directory {
                None
            } else {
                metadata.modified().ok()
            },
        }
    }

    /// Whether `path`, not followed when a symbolic link, is still this.
    fn is_at(&self, path: &Path) -> bool {
        std::fs::symlink_metadata(path).is_ok_and(|now| {
            let kind = if self.directory {
                now.is_dir()
            } else {
                now.is_file()
            };
            kind && Identity::of(&now) == *self
        })
    }
}

/// What this run made, each with its identity, and an open handle on each directory, so no other
/// directory takes its inode while the run holds it.
#[derive(Default)]
struct Made {
    /// The directories created for `--out`, first first.
    directories: Vec<(PathBuf, Identity)>,
    /// The private staging directory inside `--out`.
    staging: Option<(PathBuf, Identity)>,
    /// Each file written into the staging directory. Its name there keeps its inode while the run
    /// lasts.
    staged: Vec<(PathBuf, Identity)>,
    /// Each file moved into `--out`, with the identity of the staged file it is a link to.
    placed: Vec<(PathBuf, Identity)>,
    handles: Vec<std::fs::File>,
}

impl Made {
    /// Records the directory just made at `path`, as the staging directory or as one made for
    /// `--out`, and opens it. When it cannot be opened, it is still recorded when its identity can
    /// be read, and the error is returned.
    fn directory(
        &mut self,
        path: &Path,
        staging: bool,
        made: &mut dyn FnMut(&Path),
    ) -> std::io::Result<()> {
        made(path);
        let opened = std::fs::File::open(path).and_then(|handle| Ok((handle.metadata()?, handle)));
        let (identity, failed) = match opened {
            Ok((metadata, handle)) => {
                self.handles.push(handle);
                (Some(Identity::of(&metadata)), None)
            }
            Err(error) => (
                std::fs::symlink_metadata(path)
                    .ok()
                    .map(|metadata| Identity::of(&metadata)),
                Some(error),
            ),
        };
        if let Some(identity) = identity {
            let made = (path.to_path_buf(), identity);
            if staging {
                self.staging = Some(made);
            } else {
                self.directories.push(made);
            }
        }
        failed.map_or(Ok(()), Err)
    }

    /// Removes the staged names, last first, and then the staging directory, each only while it is
    /// the one this run made; each is forgotten once removed.
    fn remove_staging(&mut self) -> std::io::Result<()> {
        while let Some((path, identity)) = self.staged.last() {
            if identity.is_at(path) {
                std::fs::remove_file(path)?;
            }
            self.staged.pop();
        }
        if let Some((path, identity)) = &self.staging {
            if identity.is_at(path) {
                std::fs::remove_dir(path)?;
            }
            self.staging = None;
        }
        Ok(())
    }

    /// Removes everything this run made, last first, each only while it is still the one this run
    /// made, and nothing else. A directory is removed only when empty.
    fn roll_back(self) {
        // The files in `--out` first: while the staged names are there, no other file can have
        // the inode of one of them.
        let files = self.placed.iter().rev().chain(self.staged.iter().rev());
        for (path, identity) in files {
            if identity.is_at(path) {
                let _ = std::fs::remove_file(path);
            }
        }
        let directories = self.directories.iter().chain(&self.staging).rev();
        for (path, identity) in directories {
            if identity.is_at(path) {
                let _ = std::fs::remove_dir(path);
            }
        }
    }
}

/// Creates `out` and its missing parents when it is `absent`, a private staging directory in it,
/// each scenario file in that, and then links each into `out`, never replacing a file that is
/// there, and removes the staging directory. Records what it makes in `made`; on failure returns
/// the path that failed with the error.
fn write_all(
    out: &Path,
    absent: bool,
    scenarios: &[generate::Scenario],
    made: &mut Made,
    hooks: &mut Hooks<'_>,
) -> Result<(), (PathBuf, std::io::Error)> {
    if absent {
        let missing: Vec<&Path> = out
            .ancestors()
            .take_while(|dir| !dir.as_os_str().is_empty() && !dir.exists())
            .collect();
        for dir in missing.into_iter().rev() {
            match std::fs::create_dir(dir) {
                Ok(()) => made
                    .directory(dir, false, &mut |_| {})
                    .map_err(|error| (dir.to_path_buf(), error))?,
                // Made by someone else meanwhile, or a `..` component: not ours to remove.
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && dir.is_dir() => {
                }
                Err(error) => return Err((dir.to_path_buf(), error)),
            }
        }
    }
    // A failure of the staging directory names `--out`: the staging name is the run's own.
    let staging = create_staging(out).map_err(|error| (out.to_path_buf(), error))?;
    made.directory(&staging, true, hooks.staging_made)
        .map_err(|error| (out.to_path_buf(), error))?;
    (hooks.staging_open)(&staging);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        // The mode asked for at creation is narrowed by the umask; this is the one asked for.
        let handle = made.handles.last().expect("the staging directory is open");
        handle
            .set_permissions(std::fs::Permissions::from_mode(0o700))
            .map_err(|error| (out.to_path_buf(), error))?;
    }
    for scenario in scenarios {
        // A failure names the file as it would be in `--out`.
        let failed = |error: std::io::Error| (out.join(&scenario.file), error);
        let path = staging.join(&scenario.file);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(failed)?;
        file.write_all(scenario.text.as_bytes()).map_err(failed)?;
        let identity = file
            .metadata()
            .map(|metadata| Identity::of(&metadata))
            .map_err(failed)?;
        made.staged.push((path, identity));
    }
    for (scenario, (staged, identity)) in scenarios.iter().zip(&made.staged) {
        let path = out.join(&scenario.file);
        // A hard link fails when the name is taken: nothing another writer put there is replaced.
        (hooks.link)(staged, &path).map_err(|error| (path.clone(), error))?;
        made.placed.push((path, identity.clone()));
    }
    made.remove_staging()
        .map_err(|error| (out.to_path_buf(), error))
}

/// Creates the private staging directory of this run in `out`: `.canon-generate-<pid>-<n>`, with
/// the first `n` no other process has taken, mode 0700.
fn create_staging(out: &Path) -> std::io::Result<PathBuf> {
    let mut builder = std::fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt as _;
        builder.mode(0o700);
    }
    let pid = std::process::id();
    let mut attempt = 0u32;
    loop {
        let path = out.join(format!(".canon-generate-{pid}-{attempt}"));
        match builder.create(&path) {
            Ok(()) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists && attempt < 1000 => {
                attempt += 1;
            }
            Err(error) => return Err(error),
        }
    }
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

    /// What another writer of `--out` does to the run's staging directory: moves it to `moved` and
    /// puts `with` in its place, a symbolic link to a directory of the user's or a directory of
    /// its own.
    #[cfg(unix)]
    fn swap(staging: &Path, moved: &Path, with: &Swap) {
        std::fs::rename(staging, moved).expect("the other writer moves the staging directory");
        match with {
            Swap::Link(victim) => {
                std::os::unix::fs::symlink(victim, staging).expect("the other writer links");
            }
            Swap::Directory => std::fs::create_dir(staging).expect("the other writer creates"),
        }
    }

    #[cfg(unix)]
    enum Swap {
        Link(PathBuf),
        Directory,
    }

    #[cfg(unix)]
    fn mode(path: &Path) -> u32 {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::symlink_metadata(path)
            .expect("there")
            .permissions()
            .mode()
            & 0o7777
    }

    /// `--out` with another writer's directory `victim` beside it, mode 0755.
    #[cfg(unix)]
    fn shared_out(name: &str) -> (PathBuf, PathBuf, PathBuf) {
        use std::os::unix::fs::PermissionsExt as _;
        let work = scratch(name);
        let out = work.join("out");
        std::fs::create_dir(&out).expect("creatable");
        let victim = work.join("victim");
        std::fs::create_dir(&victim).expect("creatable");
        std::fs::set_permissions(&victim, std::fs::Permissions::from_mode(0o755)).expect("chmod");
        (work, out, victim)
    }

    /// A hard link that first notes what `dir` holds: a run that wrote into `dir` and removes it
    /// again before it ends is seen there.
    #[cfg(unix)]
    fn watching<'a>(
        dir: &'a Path,
        seen: &'a std::cell::RefCell<Vec<String>>,
    ) -> impl Fn(&Path, &Path) -> std::io::Result<()> + 'a {
        move |staged, path| {
            if dir.is_dir() {
                seen.borrow_mut().extend(entries(dir));
            }
            std::fs::hard_link(staged, path)
        }
    }

    /// The victim kept its mode and holds nothing, now and whenever the run placed a file.
    #[cfg(unix)]
    fn untouched(victim: &Path, seen: &std::cell::RefCell<Vec<String>>) {
        assert_eq!(mode(victim), 0o755, "the run changed the victim's mode");
        assert!(
            entries(victim).is_empty() && seen.borrow().is_empty(),
            "the run wrote through the swapped staging directory into {}: {:?}, during the run {:?}",
            victim.display(),
            entries(victim),
            seen.borrow()
        );
    }

    /// F2: the staging directory is swapped for a symbolic link to another directory between its
    /// `mkdir` and its open. The run must not open, change or write through the link.
    #[cfg(unix)]
    #[test]
    fn a_staging_directory_swapped_for_a_symbolic_link_is_not_written_through() {
        let (work, out, victim) = shared_out("staging-swapped-for-link");
        let scenarios = [scenario("a.yaml"), scenario("b.yaml")];
        let mut name = None;
        let seen = std::cell::RefCell::new(Vec::new());
        let result = write_out_with(
            &out,
            false,
            &scenarios,
            &mut Hooks {
                staging_made: &mut |staging| {
                    name = staging.file_name().map(|name| name.to_owned());
                    swap(staging, &work.join("moved"), &Swap::Link(victim.clone()));
                },
                staging_open: &mut |_| {},
                link: &watching(&victim, &seen),
                before_rollback: &mut || {},
            },
        );
        untouched(&victim, &seen);
        assert!(result.is_err(), "the run went on through the swapped link");
        let name = name.expect("the staging directory was made");
        assert_eq!(
            entries(&out),
            [name.to_string_lossy().into_owned()],
            "--out holds what the run made"
        );
        assert!(
            std::fs::symlink_metadata(out.join(&name))
                .expect("there")
                .file_type()
                .is_symlink(),
            "the other writer's link is gone"
        );
    }

    /// F2: the staging directory is swapped for another directory between its `mkdir` and its
    /// open. The run must notice it is not the one it made and write nothing into it.
    #[cfg(unix)]
    #[test]
    fn a_staging_directory_swapped_for_another_directory_is_refused() {
        let (work, out, _) = shared_out("staging-swapped-for-directory");
        let scenarios = [scenario("a.yaml"), scenario("b.yaml")];
        let staging = std::cell::RefCell::new(PathBuf::new());
        let seen = std::cell::RefCell::new(Vec::new());
        let link = |staged: &Path, path: &Path| {
            let theirs = staging.borrow();
            if theirs.is_dir() {
                seen.borrow_mut().extend(entries(&theirs));
            }
            std::fs::hard_link(staged, path)
        };
        let result = write_out_with(
            &out,
            false,
            &scenarios,
            &mut Hooks {
                staging_made: &mut |made| {
                    *staging.borrow_mut() = made.to_path_buf();
                    swap(made, &work.join("moved"), &Swap::Directory);
                },
                staging_open: &mut |_| {},
                link: &link,
                before_rollback: &mut || {},
            },
        );
        let theirs = staging.borrow().clone();
        let name = theirs.file_name().expect("the staging directory was made");
        assert!(
            theirs.is_dir(),
            "the run removed the other writer's directory"
        );
        assert!(
            entries(&theirs).is_empty() && seen.borrow().is_empty(),
            "the run wrote into the other writer's directory: {:?}, during the run {:?}",
            entries(&theirs),
            seen.borrow()
        );
        assert!(
            result.is_err(),
            "the run went on in another writer's directory"
        );
        assert_eq!(
            entries(&out),
            [name.to_string_lossy().into_owned()],
            "--out holds what the run made"
        );
    }

    /// F2: the staging directory is swapped for a symbolic link after the run opened it. Every
    /// later step must still address the directory the run opened, not the path.
    #[cfg(unix)]
    #[test]
    fn a_staging_directory_swapped_after_it_is_opened_is_not_written_through() {
        let (work, out, victim) = shared_out("staging-swapped-after-open");
        let scenarios = [scenario("a.yaml"), scenario("b.yaml")];
        let seen = std::cell::RefCell::new(Vec::new());
        let result = write_out_with(
            &out,
            false,
            &scenarios,
            &mut Hooks {
                staging_made: &mut |_| {},
                staging_open: &mut |staging| {
                    swap(staging, &work.join("moved"), &Swap::Link(victim.clone()));
                },
                link: &watching(&victim, &seen),
                before_rollback: &mut || {},
            },
        );
        untouched(&victim, &seen);
        for name in entries(&out) {
            if let Some(scenario) = scenarios.iter().find(|scenario| scenario.file == name) {
                assert!(result.is_ok(), "{name} is in --out after a failed run");
                assert_eq!(
                    std::fs::read_to_string(out.join(&name)).expect("readable"),
                    scenario.text
                );
            } else {
                assert!(name.starts_with(".canon-generate-"), "{name} in --out");
            }
        }
    }

    /// F7: where the filesystem has no hard links (vfat, exFAT, some FUSE mounts: `EPERM`,
    /// `EOPNOTSUPP`, `EXDEV` from the link call), each file is written in place, never over one
    /// that is there, and the run succeeds as 0.12.0 did.
    #[cfg(unix)]
    #[test]
    fn a_file_system_without_hard_links_gets_the_files_written_in_place() {
        for code in [libc::EPERM, libc::EOPNOTSUPP, libc::ENOTSUP, libc::EXDEV] {
            let work = scratch(&format!("no-hard-links-{code}"));
            let out = work.join("out");
            let scenarios = [scenario("a.yaml"), scenario("b.yaml")];
            let result = write_out_with(
                &out,
                true,
                &scenarios,
                &mut Hooks {
                    staging_made: &mut |_| {},
                    staging_open: &mut |_| {},
                    link: &|_, _| Err(std::io::Error::from_raw_os_error(code)),
                    before_rollback: &mut || panic!("no rollback"),
                },
            );
            assert!(result.is_ok(), "link error {code}: {result:?}");
            assert_eq!(entries(&out), ["a.yaml", "b.yaml"], "link error {code}");
            for scenario in &scenarios {
                assert_eq!(
                    std::fs::read_to_string(out.join(&scenario.file)).expect("readable"),
                    scenario.text
                );
            }
        }
    }

    /// F7: written in place, a file another writer put there first is not replaced, and the
    /// rollback removes only the files the run wrote that are still the ones it wrote.
    #[cfg(unix)]
    #[test]
    fn a_file_system_without_hard_links_keeps_the_rollback_rule() {
        let work = scratch("no-hard-links-rollback");
        let out = work.join("out");
        std::fs::create_dir(&out).expect("creatable");
        std::fs::write(out.join("c.yaml"), "another writer's c.yaml\n").expect("writable");
        let a = out.join("a.yaml");
        let mut placed_a = false;
        let scenarios = [scenario("a.yaml"), scenario("b.yaml"), scenario("c.yaml")];
        let result = write_out_with(
            &out,
            false,
            &scenarios,
            &mut Hooks {
                staging_made: &mut |_| {},
                staging_open: &mut |_| {},
                link: &|_, _| Err(std::io::Error::from_raw_os_error(libc::EXDEV)),
                before_rollback: &mut || {
                    placed_a =
                        std::fs::read_to_string(&a).ok().as_deref() == Some("generated a.yaml\n");
                    replace(&a, "another writer's a.yaml\n");
                },
            },
        );
        assert!(result.is_err(), "c.yaml is another writer's");
        assert!(placed_a, "a.yaml was not written in place");
        assert_eq!(
            std::fs::read_to_string(out.join("c.yaml")).ok().as_deref(),
            Some("another writer's c.yaml\n")
        );
        assert_eq!(
            std::fs::read_to_string(&a).ok().as_deref(),
            Some("another writer's a.yaml\n")
        );
        assert_eq!(
            entries(&out),
            ["a.yaml", "c.yaml"],
            "b.yaml, the run's, is removed"
        );
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
