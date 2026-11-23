//! story:approvals-bound-to-evidence-content, documented form: `canon evaluate --help` says, at
//! both flags that take an approval (`--authority` and `--decisions`), that Canon checks a cited
//! evidence record by id only and that evidence IDs must be immutable once an approval cites them.

use clap::CommandFactory;

/// The words the help uses, compared without case and with runs of whitespace as one space.
const REQUIREMENT: &str = "evidence ids must be immutable once an approval cites them";

/// The help of `--authority` and `--decisions`, lower case, whitespace runs as one space.
fn approval_help() -> Vec<(&'static str, String)> {
    let command = canon_cli::Cli::command();
    let evaluate = command
        .find_subcommand("evaluate")
        .expect("canon evaluate exists");
    ["authority", "decisions"]
        .into_iter()
        .map(|flag| {
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
            (flag, help.split_whitespace().collect::<Vec<_>>().join(" "))
        })
        .collect()
}

#[test]
fn evaluate_help_states_that_cited_evidence_ids_must_be_immutable() {
    for (flag, help) in approval_help() {
        assert!(
            help.contains(REQUIREMENT),
            "--{flag} help does not state `{REQUIREMENT}`: {help}"
        );
    }
}

/// The help says what decides an evidence ground (review of wave 2026-11-22-w12): the id and the
/// fields that decide whether the record under it is applicable, not the rest of its contents;
/// with other contents the entry still applies, and only a claim that reads them changes.
#[test]
fn evaluate_help_states_what_decides_an_evidence_ground() {
    for (flag, help) in approval_help() {
        for half in [
            "`subject_revision`, `observed_at`, `upstream_revisions`",
            "`producer`",
            "not by the rest of its contents",
            "the entry still applies",
            "the decision changes only where a claim reads them",
        ] {
            assert!(help.contains(half), "--{flag} help does not say `{half}`: {help}");
        }
        assert!(
            !help.contains("not by the record's contents"),
            "--{flag} help still says a ground is not checked by the record's contents: {help}"
        );
    }
}
