//! `canon evaluate`: reads the compiled protocol, the case snapshot, the evidence directory and the
//! optional `--authority` and `--decisions` files and `--at` instant, and hands them to the library
//! unparsed.

use std::path::Path;
use std::process::ExitCode;

use b10x_canon::{eval, model};

use crate::{REJECTED, read_text, unreadable};

/// Whether a file name is an evidence record's: `*.yaml` or `*.json`.
fn is_record_name(name: &std::ffi::OsStr) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|ext| ext == "yaml" || ext == "json")
}

/// Reads the compiled protocol, the case snapshot and every evidence record of the evidence
/// directory, and hands them to the library, which decides everything else. The directory holds
/// only records: every entry must be a regular file (a symbolic link to one counts) named `*.yaml`
/// or `*.json`, read in sorted file-name order. Any other entry — another file, a subdirectory —
/// is refused as unreadable, naming it, rather than skipped, so no evidence the operator put
/// there is silently dropped. The `--authority` and `--decisions` files, when given, are read as
/// text; they and the `--at` instant are passed through unparsed. A file that cannot be read
/// exits 2; a refusal exits 1 naming its code. Every evidence refusal names the file of each record
/// it cites: an evidence file the library refuses to read as a record (`` error[malformed-input]:
/// <file>: evidence `e1` is not a canon-evidence/1 document: missing field `subject` ``), and a
/// record refused after reading, by every file holding a record of its id, so a repeated id names
/// both files (`` error[duplicate-identifier]: <file>, <file>: evidence `e1` is given more than
/// once ``).
pub(crate) fn run(
    ir_path: &Path,
    case_path: &Path,
    evidence_dir: &Path,
    authority_path: Option<&Path>,
    decisions_path: Option<&Path>,
    at: Option<&str>,
) -> ExitCode {
    let read = |path: &Path| read_text(path).map_err(|error| unreadable(path, error));
    let inputs = (|| {
        let ir = read(ir_path)?;
        let case = read(case_path)?;
        let authority = authority_path.map(read).transpose()?;
        let decisions = decisions_path.map(read).transpose()?;
        let mut names = Vec::new();
        for entry in std::fs::read_dir(evidence_dir).map_err(|e| unreadable(evidence_dir, e))? {
            names.push(entry.map_err(|e| unreadable(evidence_dir, e))?.file_name());
        }
        names.sort();
        let mut evidence = Vec::with_capacity(names.len());
        for name in &names {
            let path = evidence_dir.join(name);
            let metadata = std::fs::metadata(&path).map_err(|e| unreadable(&path, e))?;
            if !metadata.is_file() || !is_record_name(name) {
                return Err(unreadable(
                    &path,
                    "not an evidence record: the evidence directory holds only `*.yaml` and \
                     `*.json` files",
                ));
            }
            let text = read(&path)?;
            evidence.push((path, text));
        }
        Ok((ir, case, evidence, authority, decisions))
    })();
    let (ir, case, evidence, authority, decisions) = match inputs {
        Ok(inputs) => inputs,
        Err(code) => return code,
    };
    // A refusal that cites evidence records names the file of each: `error[<code>]: <file>[, <file>…]:
    // <why>`. A record that cannot be read is cited by its file alone; a record refused after reading
    // by its id, which names every file holding a record of that id (a repeated id names both).
    let decision = eval::read_ir(&ir)
        .map_err(|r| (Vec::new(), r))
        .and_then(|ir| {
            let case = eval::read_case(&case).map_err(|r| (Vec::new(), r))?;
            let records = evidence
                .iter()
                .map(|(path, text)| eval::read_evidence(text).map_err(|r| (vec![path], r)))
                .collect::<Result<Vec<_>, _>>()?;
            let supplied = eval::Supplied {
                authority: authority.as_deref(),
                at,
                decisions: decisions.as_deref(),
            };
            eval::evaluate_with(&ir, &case, &records, supplied).map_err(|refusal| {
                let files = refusal
                    .evidence()
                    .iter()
                    .flat_map(|cited| {
                        records
                            .iter()
                            .zip(&evidence)
                            .filter(move |(record, _)| record.id == *cited)
                            .map(|(_, (path, _))| path)
                    })
                    .collect();
                (files, refusal)
            })
        });
    match decision {
        Ok(decision) => {
            print!("{}", eval::render(&decision));
            ExitCode::SUCCESS
        }
        Err((files, refusal)) => {
            if files.is_empty() {
                eprintln!("error[{}]: {refusal}", refusal.code());
            } else {
                let files: Vec<String> = files
                    .iter()
                    .map(|path| model::one_line(&path.display().to_string()))
                    .collect();
                eprintln!("error[{}]: {}: {refusal}", refusal.code(), files.join(", "));
            }
            ExitCode::from(REJECTED)
        }
    }
}
