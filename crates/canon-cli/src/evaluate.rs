//! `canon evaluate`: reads the compiled protocol, the case snapshot, the evidence directory and the
//! optional `--authority` and `--decisions` files and `--at` instant, and hands them to the library
//! unparsed.

use std::path::Path;
use std::process::ExitCode;

use b10x_canon::eval;

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
/// text; they and the `--at` instant are passed through unparsed. A file that cannot be read exits 2; a refusal exits
/// 1 naming its code.
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
            evidence.push(read(&path)?);
        }
        Ok((ir, case, evidence, authority, decisions))
    })();
    let (ir, case, evidence, authority, decisions) = match inputs {
        Ok(inputs) => inputs,
        Err(code) => return code,
    };
    let decision = eval::read_ir(&ir).and_then(|ir| {
        let case = eval::read_case(&case)?;
        let evidence = evidence
            .iter()
            .map(|text| eval::read_evidence(text))
            .collect::<Result<Vec<_>, _>>()?;
        let supplied = eval::Supplied {
            authority: authority.as_deref(),
            at,
            decisions: decisions.as_deref(),
        };
        eval::evaluate_with(&ir, &case, &evidence, supplied)
    });
    match decision {
        Ok(decision) => {
            print!("{}", eval::render(&decision));
            ExitCode::SUCCESS
        }
        Err(refusal) => {
            eprintln!("error[{}]: {refusal}", refusal.code());
            ExitCode::from(REJECTED)
        }
    }
}
