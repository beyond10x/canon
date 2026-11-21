//! Acceptance for story:canon-mutate: `canon mutate --path <protocol> --scenarios <dir>` applies
//! five mutation operators to the protocol, runs each mutant against the scenarios that name the
//! protocol file as their fixture, classifies each with the semantic diff and reports a weakening
//! survivor as a weak point of the scenario set.
//!
//! Five expectations, over `fixtures/investigation/mutate/`:
//!
//! 1. Every operator (`drop-conjunct`, `all-to-any`, `remove-precondition`, `remove-capability`,
//!    `change-result`) yields at least one mutant, and each mutant carries its semantic-diff
//!    categories.
//! 2. With `lacking/`, the set without the witness that `supported` needs a survived falsification
//!    attempt (`observation-only.yaml`), the mutant that drops that conjunct from
//!    `explanation.supported`, which weakens the outcome's requirement, survives, is reported as a
//!    weak point, and the command exits 1.
//! 3. With `complete/`, that mutant is killed and the command exits 0.
//! 4. A scenario set in which no scenario names the `--path` file as its fixture is refused with
//!    exit 1.
//! 5. Two runs give identical bytes.
//!
//! `every_canon_subcommand_is_declared_in_ess` (`ess_commands_match.rs`) holds the declaration of
//! the command in `ess/domains/cli.yaml`.
//!
//! The expected texts, `complete.txt`, `lacking.txt` and `unnamed.txt`, were written by hand from
//! the story before `canon mutate` existed.

use std::path::PathBuf;
use std::process::{Command, Output};

/// The fixture family, relative to the repository root.
const FIXTURES: &str = "fixtures/investigation/mutate";

/// The five operators, in the order the report lists them.
const OPERATORS: [&str; 5] = [
    "drop-conjunct",
    "all-to-any",
    "remove-precondition",
    "remove-capability",
    "change-result",
];

/// The categories of the semantic diff (`b10x_canon::diff::Category`).
const CATEGORIES: [&str; 5] = [
    "TIGHTENING",
    "RELAXATION",
    "BREAKING",
    "EXPANSION",
    "NO SEMANTIC CHANGE",
];

fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn canon(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(repository_root())
        .args(args)
        .output()
        .expect("the canon binary runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn read(path: &str) -> String {
    std::fs::read_to_string(repository_root().join(path))
        .unwrap_or_else(|error| panic!("{path} is readable: {error}"))
}

fn mutate(path: &str, scenarios: &str) -> Output {
    canon(&["mutate", "--path", path, "--scenarios", scenarios])
}

fn protocol() -> String {
    format!("{FIXTURES}/protocol.yaml")
}

/// Asserts the exit status and both streams of one run.
fn assert_run(output: &Output, status: i32, stdout: &str, stderr: &str, case: &str) {
    assert_eq!(
        (
            output.status.code(),
            text(&output.stdout),
            text(&output.stderr)
        ),
        (Some(status), stdout.to_owned(), stderr.to_owned()),
        "{case}: exit status, standard output and standard error"
    );
}

#[test]
fn mutate_reports_weakening_survivors() {
    let lacking_dir = format!("{FIXTURES}/lacking");
    let complete_dir = format!("{FIXTURES}/complete");

    // 1. Every operator yields a mutant, and every mutant line carries its categories.
    let complete = mutate(&protocol(), &complete_dir);
    let report = text(&complete.stdout);
    let mutants: Vec<&str> = report
        .lines()
        .filter(|line| !line.starts_with("mutate: "))
        .collect();
    for operator in OPERATORS {
        assert!(
            mutants.iter().any(|line| line
                .split(": ")
                .nth(1)
                .is_some_and(|rest| rest.starts_with(&format!("{operator} ")))),
            "operator `{operator}` yields no mutant in:\n{report}"
        );
    }
    for line in &mutants {
        assert!(
            line.starts_with("invalid: ")
                || CATEGORIES
                    .iter()
                    .any(|category| line.contains(&format!(": {category}"))),
            "mutant without its categories: {line}"
        );
    }

    // 2. Without the witness, the weakening mutant survives as a weak point and the run exits 1.
    let lacking = mutate(&protocol(), &lacking_dir);
    assert_run(
        &lacking,
        1,
        &read(&format!("{FIXTURES}/lacking.txt")),
        "",
        "lacking",
    );
    assert!(text(&lacking.stdout).contains(
        "weak point: drop-conjunct claim `explanation.supported` true_when all[1]: RELAXATION \
         (weakening); no scenario fails\n"
    ));

    // 3. With the witness, that mutant is killed and the run exits 0.
    assert_run(
        &complete,
        0,
        &read(&format!("{FIXTURES}/complete.txt")),
        "",
        "complete",
    );

    // 4. A scenario set in which no scenario names the `--path` file is refused.
    let unnamed = mutate("fixtures/investigation/protocol.yaml", &complete_dir);
    assert_run(
        &unnamed,
        1,
        "",
        &read(&format!("{FIXTURES}/unnamed.txt")),
        "unnamed",
    );

    // 5. Two runs give identical bytes.
    for (first, scenarios) in [(&lacking, &lacking_dir), (&complete, &complete_dir)] {
        let second = mutate(&protocol(), scenarios);
        assert_eq!(
            (second.status.code(), &second.stdout, &second.stderr),
            (first.status.code(), &first.stdout, &first.stderr),
            "{scenarios}: a second run gave different bytes"
        );
    }
    let again = mutate("fixtures/investigation/protocol.yaml", &complete_dir);
    assert_eq!(
        (again.status.code(), &again.stdout, &again.stderr),
        (unnamed.status.code(), &unnamed.stdout, &unnamed.stderr),
        "unnamed: a second run gave different bytes"
    );
}
