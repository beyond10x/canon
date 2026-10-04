#![forbid(unsafe_code)]

//! The `canon` command line's definition: its commands, their arguments and its exit statuses.
//!
//! The binary in `main.rs` parses and dispatches it; `canon-docs` walks it to generate the CLI
//! reference, so the reference cannot drift from the commands that exist.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// A document was read and rejected.
pub const REJECTED: u8 = 1;
/// A document could not be read.
pub const UNREADABLE: u8 = 2;

/// Every exit status `canon` uses and what it means.
pub const EXIT_STATUSES: [(u8, &str); 3] = [
    (
        0,
        "Success: the document is valid or compiled, the case was evaluated, or every conformance \
         scenario ran and passed.",
    ),
    (
        REJECTED,
        "Rejected: a document was read and rejected, an evaluation was refused, a conformance \
         scenario failed or was unreadable, or no scenario ran.",
    ),
    (
        UNREADABLE,
        "Unreadable: an input file or directory could not be read.",
    ),
];

#[derive(Debug, Parser)]
#[command(
    name = "canon",
    version,
    about = "Evidence-governed protocol toolchain"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
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
pub enum ConformCommand {
    /// Run every `canon-conformance/1` scenario of a registry directory and report each.
    Run {
        /// The registry directory; every `*.yaml` file in it is one scenario.
        #[arg(long, default_value = "conformance/scenarios")]
        scenarios: PathBuf,
    },
}
