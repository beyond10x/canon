#![forbid(unsafe_code)]

//! `canon`: a thin command-line shell over the Canon library. It reads files and prints results;
//! every semantic decision is the library's.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use b10x_canon::{model, validate};

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
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Validate { path } => validate_command(&path),
    }
}

fn validate_command(path: &std::path::Path) -> ExitCode {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("error[unreadable]: {path:?}: {error}");
            return ExitCode::from(UNREADABLE);
        }
    };
    let protocol = match model::parse(&source) {
        Ok(protocol) => protocol,
        Err(error) => {
            eprintln!("error[parse]: {error}");
            return ExitCode::from(REJECTED);
        }
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
        Err(problems) => {
            for problem in &problems {
                eprintln!("error[{}]: {problem}", problem.code());
            }
            ExitCode::from(REJECTED)
        }
    }
}
