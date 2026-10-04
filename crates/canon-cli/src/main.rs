#![forbid(unsafe_code)]

//! `canon`: a thin command-line shell over the Canon library. It reads files and prints results;
//! every semantic decision is the library's. This file holds the subcommands and their dispatch,
//! and `validate` and `compile`; every other subcommand is in its own file.

mod conform;
mod diff;
mod evaluate;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use b10x_canon::{ir, model, validate};

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
        /// A `canon-authority/1` document of authority decisions. Read and passed through; not
        /// supported yet.
        #[arg(long)]
        authority: Option<PathBuf>,
        /// The evaluation instant. Passed through as written; not supported yet.
        #[arg(long)]
        at: Option<String>,
    },
    /// The semantic difference between two compiled protocol revisions. Not built yet.
    Diff {
        /// The earlier revision, as `canon compile` prints it.
        #[arg(long)]
        from: PathBuf,
        /// The later revision, as `canon compile` prints it.
        #[arg(long)]
        to: PathBuf,
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
        } => conform::run(&scenarios),
        Command::Evaluate {
            ir,
            case,
            evidence,
            authority,
            at,
        } => evaluate::run(&ir, &case, &evidence, authority.as_deref(), at.as_deref()),
        Command::Diff { from, to } => diff::run(&from, &to),
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

/// Reads a text file. One leading byte-order mark is not part of the document (YAML 1.2 § 5.2)
/// and is dropped.
fn read_text(path: &Path) -> std::io::Result<String> {
    let source = std::fs::read_to_string(path)?;
    Ok(match source.strip_prefix('\u{feff}') {
        Some(rest) => rest.to_owned(),
        None => source,
    })
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
