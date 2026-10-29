//! Acceptance for story:conformance-suite: `canon conform run --requirements <catalogue>` checks the
//! registry against the `canon-requirements/1` catalogue (coverage, unlisted requirements and
//! determinism), and without the flag the runner's report is unchanged.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Read at run time, not compile time: a test binary reused from a shared build directory must
/// still read this tree's registry, catalogue and fixtures.
fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

const REGISTRY: &str = "conformance/scenarios";
const CATALOGUE: &str = "conformance/requirements.yaml";
const FIXTURES: &str = "fixtures/conformance/requirements";

/// Runs `canon conform run` with `args` from the repository root.
fn conform_run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(repository_root())
        .args(["conform", "run"])
        .args(args)
        .output()
        .expect("canon runs")
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("utf-8 output")
}

/// The runner's report for the real registry, built from the files themselves: one `passed:` line
/// per `*.yaml` file in sorted file-name order, naming the scenario id the file declares, then the
/// summary.
fn runner_report() -> String {
    let dir = repository_root().join(REGISTRY);
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .expect("registry directory exists")
        .map(|entry| {
            entry
                .expect("readable entry")
                .file_name()
                .into_string()
                .expect("utf-8 file name")
        })
        .filter(|name| Path::new(name).extension().is_some_and(|ext| ext == "yaml"))
        .collect();
    names.sort();
    assert!(!names.is_empty(), "the registry holds scenarios");
    let mut report = String::new();
    for name in &names {
        let source = std::fs::read_to_string(dir.join(name)).expect("scenario reads");
        let id = source
            .lines()
            .find_map(|line| line.strip_prefix("id: "))
            .unwrap_or_else(|| panic!("{name} declares an id"));
        report.push_str(&format!("passed: scenario `{id}`\n"));
    }
    report.push_str(&format!(
        "conform: {} passed, 0 failed, 0 unreadable\n",
        names.len()
    ));
    report
}

fn assert_exit(output: &Output, code: i32, what: &str) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "{what}: exit status\nstdout:\n{}\nstderr:\n{}",
        text(&output.stdout),
        text(&output.stderr)
    );
}

fn assert_line(output: &Output, line: &str, what: &str) {
    let stdout = text(&output.stdout);
    assert!(
        stdout.lines().any(|printed| printed == line),
        "{what}: expected the line\n{line}\nin:\n{stdout}"
    );
}

#[test]
fn conform_run_checks_requirements() {
    // 1. Without --requirements: exactly the runner's lines and summary, no requirements line.
    let plain = conform_run(&["--scenarios", REGISTRY]);
    assert_exit(&plain, 0, "real registry without --requirements");
    assert_eq!(
        text(&plain.stdout),
        runner_report(),
        "real registry without --requirements: report"
    );
    assert!(
        !text(&plain.stdout).contains("requirements:"),
        "no requirements line without the flag"
    );

    // 2. With the real catalogue: every implemented requirement covered.
    let checked = conform_run(&["--scenarios", REGISTRY, "--requirements", CATALOGUE]);
    assert_exit(
        &checked,
        0,
        "real registry with conformance/requirements.yaml",
    );
    assert_line(
        &checked,
        "requirements: 17 implemented, 17 covered, 0 planned",
        "real registry with conformance/requirements.yaml",
    );

    // 3. An implemented requirement no scenario covers.
    let uncovered = conform_run(&[
        "--scenarios",
        &format!("{FIXTURES}/covered"),
        "--requirements",
        &format!("{FIXTURES}/uncovered.yaml"),
    ]);
    assert_exit(&uncovered, 1, "uncovered requirement");
    assert_line(
        &uncovered,
        "uncovered: requirement `CANON-FIXTURE-002`",
        "uncovered requirement",
    );

    // 4. A scenario covering a requirement the catalogue does not list.
    let unlisted = conform_run(&[
        "--scenarios",
        &format!("{FIXTURES}/unlisted"),
        "--requirements",
        &format!("{FIXTURES}/catalogue.yaml"),
    ]);
    assert_exit(&unlisted, 1, "unlisted requirement");
    assert_line(
        &unlisted,
        "unlisted: requirement `CANON-FIXTURE-099` covered by scenario `FIXTURE-UNLISTED`",
        "unlisted requirement",
    );

    // 5. A scenario whose evaluation differs when its evidence list is reversed.
    let nondeterministic = conform_run(&[
        "--scenarios",
        &format!("{FIXTURES}/nondeterministic"),
        "--requirements",
        &format!("{FIXTURES}/catalogue.yaml"),
    ]);
    assert_exit(&nondeterministic, 1, "nondeterministic scenario");
    let prefix =
        "nondeterministic: scenario `FIXTURE-NONDETERMINISTIC` step `two-faults`: permuted ";
    assert!(
        text(&nondeterministic.stdout)
            .lines()
            .any(|line| line.starts_with(prefix)),
        "nondeterministic scenario: expected a line starting\n{prefix}\nin:\n{}",
        text(&nondeterministic.stdout)
    );

    // 6. `task check` runs the registry with the catalogue.
    let taskfile = std::fs::read_to_string(repository_root().join("Taskfile.yml"))
        .expect("Taskfile.yml reads");
    let taskfile: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(&taskfile).expect("Taskfile.yml is YAML");
    let commands = taskfile["tasks"]["check"]["cmds"]
        .as_sequence()
        .expect("the check task lists its commands");
    assert!(
        commands.iter().any(|command| {
            let command = command
                .as_str()
                .or_else(|| command["cmd"].as_str())
                .unwrap_or_default();
            command.contains("conform run --requirements conformance/requirements.yaml")
        }),
        "the check task runs `conform run --requirements conformance/requirements.yaml`: {commands:?}"
    );
}
