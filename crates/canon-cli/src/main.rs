#![forbid(unsafe_code)]

//! `canon`: a thin command-line shell over the Canon library. It reads files and prints results;
//! every semantic decision is the library's. This file holds the subcommands and their dispatch,
//! and `validate` and `compile`; every other subcommand is in its own file.

mod check;
mod conform;
mod diff;
mod evaluate;

use std::path::Path;
use std::process::ExitCode;

use clap::Parser;

use b10x_canon::{ir, model, validate};
use canon_cli::{Cli, Command, ConformCommand, REJECTED, UNREADABLE, USAGE};

fn main() -> ExitCode {
    // A usage error exits `USAGE`, not clap's own 2, which the exit-status table gives to an
    // unreadable input; `--help` and `--version` print to standard output and succeed.
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let _ = error.print();
            return if error.use_stderr() {
                ExitCode::from(USAGE)
            } else {
                ExitCode::SUCCESS
            };
        }
    };
    match cli.command {
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
            decisions,
            at,
        } => evaluate::run(
            &ir,
            &case,
            &evidence,
            authority.as_deref(),
            decisions.as_deref(),
            at.as_deref(),
        ),
        Command::Diff { from, to } => diff::run(&from, &to),
        Command::Check { path, properties } => check::run(&path, properties.as_deref()),
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
