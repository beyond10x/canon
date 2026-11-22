//! story:approvals-bound-to-evidence-content, documented form: `canon evaluate --help` says, at
//! both flags that take an approval (`--authority` and `--decisions`), that Canon checks a cited
//! evidence record by id only and that evidence IDs must be immutable once an approval cites them.

use clap::CommandFactory;

/// The words the help uses, compared without case and with runs of whitespace as one space.
const REQUIREMENT: &str = "evidence ids must be immutable once an approval cites them";

#[test]
fn evaluate_help_states_that_cited_evidence_ids_must_be_immutable() {
    let command = canon_cli::Cli::command();
    let evaluate = command
        .find_subcommand("evaluate")
        .expect("canon evaluate exists");
    for flag in ["authority", "decisions"] {
        let arg = evaluate
            .get_arguments()
            .find(|arg| arg.get_long() == Some(flag))
            .unwrap_or_else(|| panic!("--{flag} exists"));
        let help = arg
            .get_long_help()
            .or_else(|| arg.get_help())
            .map(ToString::to_string)
            .unwrap_or_default()
            .to_lowercase();
        let help = help.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            help.contains(REQUIREMENT),
            "--{flag} help does not state `{REQUIREMENT}`: {help}"
        );
    }
}
