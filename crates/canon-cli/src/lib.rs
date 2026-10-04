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
/// The command line was not understood: an unknown command or option, a missing required one, or
/// a value clap refuses (`EX_USAGE` of BSD `sysexits.h`).
pub const USAGE: u8 = 64;

/// Every exit status `canon` uses and what it means.
pub const EXIT_STATUSES: [(u8, &str); 4] = [
    (
        0,
        "Success: the document is valid or compiled, the case was evaluated, every conformance \
         scenario ran and passed, or a check found nothing.",
    ),
    (
        REJECTED,
        "Rejected: a document was read and rejected, an evaluation or a check was refused, a \
         conformance scenario failed or was unreadable, no scenario ran, or a check found \
         something.",
    ),
    (
        UNREADABLE,
        "Unreadable: an input file or directory could not be read.",
    ),
    (
        USAGE,
        "Usage error: the command line was not understood (an unknown command or option, a \
         missing required option, or a value that cannot be parsed); nothing was read.",
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
    /// Evaluate a case under a compiled protocol from an evidence set: every claim, and each
    /// declared obligation, action and outcome. Print the `canon-decision/1` document.
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
        /// A `canon-authority/1` document: which capabilities are granted or denied. An action is
        /// admissible only when every capability it requires is granted; without it, none is
        /// granted.
        #[arg(long)]
        authority: Option<PathBuf>,
        /// A `canon-decisions/1` document: the explicit decisions taken, each naming the decision,
        /// the outcome, the principal and the case revision it was taken at. An outcome that requires
        /// a decision is legitimate only with one taken at the case snapshot's revision.
        #[arg(long)]
        decisions: Option<PathBuf>,
        /// The evaluation instant, in UTC as `YYYY-MM-DDTHH:MM:SSZ`. Evidence older than its kind's
        /// `max_age` at this instant does not apply; without it, no evidence expires.
        #[arg(long)]
        at: Option<String>,
    },
    /// Check a `protocol/1` document over its whole finite state space: report outcomes no state
    /// reaches, actions whose precondition holds in no state, claims, obligations, preconditions and
    /// outcome requirements that read evidence no action produces, outcomes that rest on evidence an
    /// authority-requiring action may produce yet hold without any authority decision, and each
    /// declared property that fails, with a counterexample. A state is a set of evidence records,
    /// authority decisions and explicit decisions; a record an invalidation rule can keep from a
    /// claim is there in any combination of observed before and after each upstream artifact
    /// moved, one record each. A state space of more than 65536 states is refused.
    Check {
        /// The protocol document to check.
        #[arg(long)]
        path: PathBuf,
        /// A `canon-properties/1` document: properties declared beside the protocol, each saying an
        /// action's or outcome's status is independent of a claim. Each is checked in every state.
        #[arg(long)]
        properties: Option<PathBuf>,
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
