//! Adversary pass 2 for story:conformance-runner (wave 2026-10-04-w3): the error paths of
//! `canon conform run` against the report and exit-status contract its module documentation
//! (`crates/canon/src/conform/mod.rs`, "# The report") states.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

/// A fresh working directory with an empty registry `scenarios/` and the base fixture at
/// `p.yaml`.
fn workspace(name: &str) -> PathBuf {
    let cwd = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("adversary2-exit-{name}"));
    if cwd.exists() {
        std::fs::remove_dir_all(&cwd).expect("clear old workspace");
    }
    std::fs::create_dir_all(cwd.join("scenarios")).expect("create registry");
    let fixture =
        std::fs::read_to_string(repository_root().join("fixtures/investigation/protocol.yaml"))
            .expect("base fixture");
    std::fs::write(cwd.join("p.yaml"), fixture).expect("write fixture");
    cwd
}

/// The unit's passing scenario under a given id, naming `p.yaml`.
fn passing_scenario(id: &str) -> String {
    std::fs::read_to_string(
        repository_root().join("crates/canon-cli/tests/conform/passing/compile-passes.yaml"),
    )
    .expect("passing scenario")
    .replace("id: RUNNER-COMPILE-PASSES\n", &format!("id: {id}\n"))
    .replace(
        "fixture: fixtures/investigation/protocol.yaml\n",
        "fixture: p.yaml\n",
    )
}

fn conform_run(cwd: &Path, scenarios: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(cwd)
        .args(["conform", "run", "--scenarios", scenarios])
        .output()
        .expect("canon runs")
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("utf-8 output")
}

/// The documentation: "2 when the registry directory itself cannot be read, with
/// `error[unreadable]: <dir>: <why>` on standard error". `<dir>` is the directory as given, the
/// same rendering `<path>` has on the report's `unreadable:` lines ("the scenario directory as
/// given"), not a Rust debug-quoted string.
#[test]
fn a_missing_registry_is_reported_as_the_documentation_shows() {
    let cwd = workspace("missing");
    let output = conform_run(&cwd, "no-such-registry");
    assert_eq!(text(&output.stdout), "", "stdout");
    assert_eq!(
        text(&output.stderr),
        "error[unreadable]: no-such-registry: No such file or directory (os error 2)\n",
        "stderr"
    );
    assert_eq!(output.status.code(), Some(2), "exit");
}

/// A registry path that is a file, not a directory, is the registry being unreadable: exit 2,
/// nothing on standard output.
#[test]
fn a_registry_path_naming_a_file_exits_two_with_nothing_on_stdout() {
    let cwd = workspace("file");
    let output = conform_run(&cwd, "p.yaml");
    assert_eq!(text(&output.stdout), "", "stdout");
    assert!(
        text(&output.stderr).starts_with("error[unreadable]: "),
        "{}",
        text(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(2), "exit");
}

/// A `*.yaml` entry that cannot be read as text — not UTF-8, or a directory — is one
/// `unreadable:` line by its path, and the run exits 1, not 2: the registry itself was read.
#[test]
fn an_unreadable_scenario_file_is_a_report_line_and_exit_one() {
    let cwd = workspace("entries");
    std::fs::write(cwd.join("scenarios/a.yaml"), passing_scenario("A")).expect("write");
    std::fs::write(cwd.join("scenarios/b.yaml"), [0xff, 0xfe, 0x00, 0x80]).expect("write");
    std::fs::create_dir_all(cwd.join("scenarios/c.yaml")).expect("create dir");

    let output = conform_run(&cwd, "scenarios");
    assert_eq!(text(&output.stderr), "");
    assert_eq!(
        text(&output.stdout),
        "passed: scenario `A`\n\
         unreadable: scenarios/b.yaml: stream did not contain valid UTF-8\n\
         unreadable: scenarios/c.yaml: Is a directory (os error 21)\n\
         conform: 1 passed, 0 failed, 2 unreadable\n"
    );
    assert_eq!(output.status.code(), Some(1));
}

/// Scenario ids that differ only in case are different scenarios: both run, each is reported on
/// its own line in file-name order, and the report is byte-identical across runs.
#[test]
fn scenario_ids_differing_only_in_case_are_reported_separately_and_stably() {
    let cwd = workspace("case");
    std::fs::write(cwd.join("scenarios/a.yaml"), passing_scenario("CANON-X")).expect("write");
    std::fs::write(cwd.join("scenarios/b.yaml"), passing_scenario("canon-x")).expect("write");
    std::fs::write(cwd.join("scenarios/c.yaml"), passing_scenario("CANON-X")).expect("write");

    let output = conform_run(&cwd, "scenarios");
    assert_eq!(text(&output.stderr), "");
    assert_eq!(
        text(&output.stdout),
        "passed: scenario `CANON-X`\n\
         passed: scenario `canon-x`\n\
         unreadable: scenarios/c.yaml: scenario id `CANON-X` is already used by scenarios/a.yaml\n\
         conform: 2 passed, 0 failed, 1 unreadable\n"
    );
    assert_eq!(output.status.code(), Some(1));
    let again = conform_run(&cwd, "scenarios");
    assert_eq!(again.stdout, output.stdout, "stdout across runs");
}

/// Pass 1 made a fixture that does not parse report kind and position only, because a file named
/// by path "may be anything". A registry file is any `*.yaml` file of the directory `--scenarios`
/// names, and one that does not parse as a scenario still has its content quoted on its
/// `unreadable:` line.
#[test]
fn a_registry_file_that_is_not_a_scenario_is_not_echoed() {
    let cwd = workspace("echo");
    std::fs::write(cwd.join("scenarios/a.yaml"), "SECRET-SCALAR-VALUE\n").expect("write");

    let output = conform_run(&cwd, "scenarios");
    let report = text(&output.stdout);
    assert!(
        report.starts_with("unreadable: scenarios/a.yaml: "),
        "{report}"
    );
    assert!(!report.contains("SECRET"), "{report}");
    assert_eq!(output.status.code(), Some(1));
}
