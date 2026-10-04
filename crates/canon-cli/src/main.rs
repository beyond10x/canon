#![forbid(unsafe_code)]

//! `canon`: a thin command-line shell over the Canon library. It reads files and prints results;
//! every semantic decision is the library's.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use b10x_canon::{conform, eval, ir, model, validate};

/// A document was read and rejected.
const REJECTED: u8 = 1;
/// A document could not be read.
const UNREADABLE: u8 = 2;

#[derive(Debug, Parser)]
#[command(
    name = "canon",
    version,
    about = "Evidence-governed protocol toolchain"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Parse and validate a `protocol/1` document.
    Validate {
        /// The protocol document to validate.
        #[arg(long)]
        path: PathBuf,
    },
    /// Compile a valid `protocol/1` document into `canon-ir/1` and print it.
    Compile {
        /// The protocol document to compile.
        #[arg(long)]
        path: PathBuf,
    },
    /// Evaluate every claim of a compiled protocol for a case from an evidence set, and print the
    /// `canon-decision/1` document.
    Evaluate {
        /// The compiled protocol: `canon-ir/1` exactly as `canon compile` prints it.
        #[arg(long)]
        ir: PathBuf,
        /// The `canon-case/1` case snapshot.
        #[arg(long)]
        case: PathBuf,
        /// A directory holding only `canon-evidence/1` records, one per `*.yaml` or `*.json` file.
        #[arg(long)]
        evidence: PathBuf,
    },
    /// Conformance scenarios.
    Conform {
        #[command(subcommand)]
        command: ConformCommand,
    },
}

#[derive(Debug, Subcommand)]
enum ConformCommand {
    /// Run every `canon-conformance/1` scenario of a registry directory and report each.
    Run {
        /// The registry directory; every `*.yaml` file in it is one scenario.
        #[arg(long, default_value = "conformance/scenarios")]
        scenarios: PathBuf,
    },
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Validate { path } => validate_command(&path),
        Command::Compile { path } => compile_command(&path),
        Command::Conform {
            command: ConformCommand::Run { scenarios },
        } => conform_run_command(&scenarios),
        Command::Evaluate { ir, case, evidence } => evaluate_command(&ir, &case, &evidence),
    }
}

/// Prints `error[unreadable]: <path>: <why>` and returns the exit status for an unreadable input.
fn unreadable(path: &Path, why: impl std::fmt::Display) -> ExitCode {
    eprintln!(
        "error[unreadable]: {}: {why}",
        model::one_line(&path.display().to_string())
    );
    ExitCode::from(UNREADABLE)
}

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
/// there is silently dropped. A file that cannot be read exits 2; a refusal exits 1 naming its
/// code.
fn evaluate_command(ir_path: &Path, case_path: &Path, evidence_dir: &Path) -> ExitCode {
    let read = |path: &Path| read_text(path).map_err(|error| unreadable(path, error));
    let inputs = (|| {
        let ir = read(ir_path)?;
        let case = read(case_path)?;
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
        Ok((ir, case, evidence))
    })();
    let (ir, case, evidence) = match inputs {
        Ok(inputs) => inputs,
        Err(code) => return code,
    };
    let decision = eval::read_ir(&ir).and_then(|ir| {
        let case = eval::read_case(&case)?;
        let evidence = evidence
            .iter()
            .map(|text| eval::read_evidence(text))
            .collect::<Result<Vec<_>, _>>()?;
        eval::evaluate(&ir, &case, &evidence)
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

/// Reads a text file. One leading byte-order mark is not part of the document (YAML 1.2 § 5.2)
/// and is dropped.
fn read_text(path: &Path) -> std::io::Result<String> {
    let source = std::fs::read_to_string(path)?;
    Ok(match source.strip_prefix('\u{feff}') {
        Some(rest) => rest.to_owned(),
        None => source,
    })
}

/// Reads every `*.yaml` file of the registry directory in sorted file-name order, and hands the
/// texts and a fixture reader to the library, which decides every verdict.
fn conform_run_command(scenarios: &Path) -> ExitCode {
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

/// Reads and parses a `protocol/1` document, reporting why when it cannot. One leading byte-order
/// mark is not part of the document (YAML 1.2 § 5.2) and is dropped before parsing.
fn read_protocol(path: &Path) -> Result<model::Protocol, ExitCode> {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("error[unreadable]: {path:?}: {error}");
            return Err(ExitCode::from(UNREADABLE));
        }
    };
    let source = source.strip_prefix('\u{feff}').unwrap_or(&source);
    model::parse(source).map_err(|error| {
        eprintln!("error[parse]: {error}");
        ExitCode::from(REJECTED)
    })
}

fn report(problems: &[validate::Problem]) -> ExitCode {
    for problem in problems {
        eprintln!("error[{}]: {problem}", problem.code());
    }
    ExitCode::from(REJECTED)
}

fn compile_command(path: &Path) -> ExitCode {
    let protocol = match read_protocol(path) {
        Ok(protocol) => protocol,
        Err(code) => return code,
    };
    match ir::compile(&protocol) {
        Ok(compiled) => {
            print!("{}", compiled.canonical_json());
            ExitCode::SUCCESS
        }
        Err(problems) => report(&problems),
    }
}

fn validate_command(path: &Path) -> ExitCode {
    let protocol = match read_protocol(path) {
        Ok(protocol) => protocol,
        Err(code) => return code,
    };
    match validate::validate(&protocol) {
        Ok(()) => {
            println!(
                "valid: protocol `{}` revision {}",
                model::one_line(protocol.protocol.id.as_str()),
                protocol.protocol.revision
            );
            ExitCode::SUCCESS
        }
        Err(problems) => report(&problems),
    }
}
