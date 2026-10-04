//! Acceptance for story:conformance-runner: `canon conform run` runs every scenario of a registry
//! directory, reports each by id (or, when a file does not parse as a scenario, by path), exits 0
//! only when every scenario passed, and prints the same bytes on every run.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Read at run time, not compile time: a test binary reused from a shared build directory must
/// still read this tree's scenarios and fixtures.
fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

/// The all-passing registry: one compile scenario whose expected `canon-ir/1` matches.
const PASSING: &str = "crates/canon-cli/tests/conform/passing";
/// The mixed registry: that scenario, one whose expected `canon-ir/1` differs, and a file that
/// does not parse as a scenario.
const MIXED: &str = "crates/canon-cli/tests/conform/mixed";

/// Runs `canon conform run --scenarios <dir>` from the repository root, so that scenario fixture
/// paths and reported paths are relative to it.
fn conform_run(scenarios: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(repository_root())
        .arg("conform")
        .arg("run")
        .arg("--scenarios")
        .arg(scenarios)
        .output()
        .expect("canon runs")
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("utf-8 output")
}

/// The `*.yaml` file names of a registry directory, sorted, so a missing or renamed scenario file
/// fails here rather than leaving the run vacuous.
fn scenario_files(dir: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(repository_root().join(dir))
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
    names
}

#[test]
fn conform_run_reports_each_scenario() {
    assert_eq!(scenario_files(PASSING), ["compile-passes.yaml"]);
    assert_eq!(
        scenario_files(MIXED),
        [
            "compile-differs.yaml",
            "compile-passes.yaml",
            "not-a-scenario.yaml"
        ]
    );

    // 1. An all-passing registry reports its scenario passed and exits 0.
    let passing = conform_run(PASSING);
    assert_eq!(text(&passing.stderr), "", "passing registry: stderr");
    assert_eq!(
        text(&passing.stdout),
        "passed: scenario `RUNNER-COMPILE-PASSES`\n\
         conform: 1 passed, 0 failed, 0 unreadable\n",
        "passing registry: report"
    );
    assert_eq!(passing.status.code(), Some(0), "passing registry: exit");

    // The mixed registry, in sorted file-name order.
    let mixed = conform_run(MIXED);
    assert_eq!(text(&mixed.stderr), "", "mixed registry: stderr");
    let report = text(&mixed.stdout);
    let lines: Vec<&str> = report.lines().collect();
    assert_eq!(
        lines.len(),
        4,
        "mixed registry: one line per file and a summary: {report}"
    );

    // 2. The differing scenario is reported failed by its id and step.
    assert_eq!(
        lines[0],
        "failed: scenario `RUNNER-COMPILE-DIFFERS` step `compile`: canon-ir/1 differs from the expectation at line 80",
        "mixed registry: differing scenario"
    );
    assert_eq!(
        lines[1], "passed: scenario `RUNNER-COMPILE-PASSES`",
        "mixed registry: passing scenario"
    );

    // 3. The file that does not parse as a scenario is reported unreadable by its path.
    let unreadable = format!("unreadable: {MIXED}/not-a-scenario.yaml: ");
    assert!(
        lines[2].starts_with(&unreadable) && lines[2].len() > unreadable.len(),
        "mixed registry: unparseable file: {}",
        lines[2]
    );
    assert_eq!(
        lines[3], "conform: 1 passed, 1 failed, 1 unreadable",
        "mixed registry: summary"
    );

    // 4. A registry with a failing scenario or an unreadable file exits non-zero.
    assert_eq!(mixed.status.code(), Some(1), "mixed registry: exit");

    // 5. The same registry prints byte-identical output on a second run.
    let again = conform_run(MIXED);
    assert_eq!(
        again.stdout, mixed.stdout,
        "mixed registry: stdout across runs"
    );
    assert_eq!(
        again.stderr, mixed.stderr,
        "mixed registry: stderr across runs"
    );
    assert_eq!(
        again.status.code(),
        mixed.status.code(),
        "mixed registry: exit across runs"
    );
}
