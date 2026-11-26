//! `canon check-set`: reads each `protocol/1` document of a set, compiles it on its own and hands
//! the set to the library, which checks the documents against each other
//! (story:check-across-protocol-files): every protocol an action opens a case under and every
//! supplier is in the set, every child outcome a protocol names is declared by a protocol it opens
//! about that subject, every supplied kind is one an action of its supplier may produce, the
//! openings form no cycle, no two files declare one protocol id, and every action that opens a
//! case under a protocol that requires a kind at open holds a record of it to hand in
//! (story:supplied-evidence-required-at-open).

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use b10x_canon::{ir, model};

use crate::{REJECTED, read_text, unreadable};

/// Prints the report and exits 0 when the check found nothing, 1 when it found something. The
/// paths are read in sorted order, so the order of the flags changes nothing; the first file that
/// cannot be read exits 2, and the first that does not parse, has imports or does not compile
/// exits 1, naming its path on standard error.
///
/// A file is read once wherever its paths resolve to (`fs::canonicalize`): two spellings of it,
/// or a symbolic link and its target, are one member, named by the first of its paths in sorted
/// order. A path that does not resolve is a member of its own, and reading it fails, naming it as
/// it was typed.
pub(crate) fn run(paths: &[PathBuf]) -> ExitCode {
    let sorted = members(paths);
    let mut set = Vec::with_capacity(sorted.len());
    for path in sorted {
        match compiled(path, "canon check-set") {
            Ok(compiled) => set.push((path.display().to_string(), compiled)),
            Err(code) => return code,
        }
    }
    let found = ir::check_set(set.iter().map(|(source, ir)| (source.as_str(), ir)));
    print!("{found}");
    if found.is_clean() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(REJECTED)
    }
}

/// The members of a set given as paths: in sorted order, a path that resolves to a file an
/// earlier path resolves to left out (`fs::canonicalize`), and a path that does not resolve a
/// member of its own.
pub(crate) fn members(paths: &[PathBuf]) -> Vec<&PathBuf> {
    let mut sorted: Vec<&PathBuf> = paths.iter().collect();
    sorted.sort();
    let mut resolved: BTreeSet<PathBuf> = BTreeSet::new();
    sorted
        .retain(|path| resolved.insert(fs::canonicalize(path).unwrap_or_else(|_| (*path).clone())));
    sorted
}

/// Reads, parses and compiles one member of the set of `command` (`canon check-set`, `canon
/// trigger protocols`), refusing one with imports: a member is read as written, and a protocol
/// with imports compiles only with its imported documents.
pub(crate) fn compiled(path: &Path, command: &str) -> Result<ir::Ir, ExitCode> {
    let shown = model::one_line(&path.display().to_string());
    let source = read_text(path).map_err(|error| unreadable(path, error))?;
    let protocol = model::parse(&source).map_err(|error| {
        eprintln!("error[parse]: {shown}: {error}");
        ExitCode::from(REJECTED)
    })?;
    if let Some(first) = protocol.imports.ids().next() {
        eprintln!(
            "error[protocol-imports]: {shown}: the protocol imports `{}`: {command} reads each \
             protocol as written, and a protocol with imports compiles only with the documents \
             it imports",
            model::one_line(first.as_str())
        );
        return Err(ExitCode::from(REJECTED));
    }
    ir::compile(&protocol).map_err(|problems| {
        for problem in &problems {
            eprintln!("error[{}]: {shown}: {problem}", problem.code());
        }
        ExitCode::from(REJECTED)
    })
}
