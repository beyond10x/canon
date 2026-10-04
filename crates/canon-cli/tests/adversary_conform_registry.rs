//! Adversary cases for story:conformance-runner (wave 2026-10-04-w3): `canon conform run` over
//! registries the unit's acceptance does not build. The unit's two registries hold only `*.yaml`
//! files and always exist, so the CLI's file filter and its missing-directory path are unobserved.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR").expect("run through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

/// A fresh registry directory under the target's scratch area.
fn registry(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("adversary-conform-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clear old registry");
    }
    std::fs::create_dir_all(&dir).expect("create registry");
    dir
}

fn passing_scenario() -> String {
    std::fs::read_to_string(
        repository_root().join("crates/canon-cli/tests/conform/passing/compile-passes.yaml"),
    )
    .expect("the unit's passing scenario")
}

fn conform_run(scenarios: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(repository_root())
        .args(["conform", "run", "--scenarios"])
        .arg(scenarios)
        .output()
        .expect("canon runs")
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("utf-8 output")
}

/// "`canon conform run` reads every `*.yaml` file of the directory": other files and a
/// subdirectory's scenarios are not part of the registry. A failing scenario sits in each place a
/// wider reader would find it.
#[test]
fn only_yaml_files_of_the_directory_itself_are_read() {
    let dir = registry("filter");
    std::fs::write(dir.join("a.yaml"), passing_scenario()).expect("write");
    let failing = "format: canon-conformance/1\nid: NOT-IN-REGISTRY\ncovers: []\nfixture: fixtures/investigation/protocol.yaml\nsteps:\n  - id: c\n    compile: {}\n    expect: {canon-ir: x}\n";
    for name in ["notes.txt", "b.yml", "c.YAML", "README", "d.yaml.bak"] {
        std::fs::write(dir.join(name), failing).expect("write");
    }
    std::fs::create_dir(dir.join("nested")).expect("mkdir");
    std::fs::write(dir.join("nested/inner.yaml"), failing).expect("write");

    let output = conform_run(&dir);
    assert_eq!(text(&output.stderr), "");
    assert_eq!(
        text(&output.stdout),
        "passed: scenario `RUNNER-COMPILE-PASSES`\nconform: 1 passed, 0 failed, 0 unreadable\n"
    );
    assert_eq!(output.status.code(), Some(0));
}

/// A registry directory that does not exist is not a passing registry: nothing is reported on
/// standard output and the exit is the CLI's unreadable status, 2.
#[test]
fn a_missing_registry_directory_exits_unreadable() {
    let dir = registry("missing").join("does-not-exist");
    let output = conform_run(&dir);
    assert_eq!(text(&output.stdout), "", "stdout");
    assert!(
        text(&output.stderr).starts_with("error[unreadable]: "),
        "stderr: {}",
        text(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(2), "exit");
}

/// A scenario file starting with a byte-order mark is the same scenario.
#[test]
fn a_scenario_file_with_a_byte_order_mark_passes() {
    let dir = registry("bom");
    std::fs::write(
        dir.join("a.yaml"),
        format!("\u{feff}{}", passing_scenario()),
    )
    .expect("write");
    let output = conform_run(&dir);
    assert_eq!(
        text(&output.stdout),
        "passed: scenario `RUNNER-COMPILE-PASSES`\nconform: 1 passed, 0 failed, 0 unreadable\n"
    );
    assert_eq!(output.status.code(), Some(0));
}
