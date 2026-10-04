//! Adversary pass 2 for story:conformance-runner (wave 2026-10-04-w3): fixture paths the textual
//! confinement check (`conform::is_confined`) lets through. `conform_registry.rs` states the
//! guarantee these cases test: a fixture outside the working directory is refused, unread. The
//! check is decided on the text alone, so a path that is textually below the working directory but
//! resolves outside it through a symbolic link is read like any other.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Read at run time, not compile time: a shared build directory reuses this binary across trees.
fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

/// A fresh scratch area holding a working directory `cwd/` with an empty registry
/// `cwd/scenarios/`, and a sibling directory `outside/` that is not below `cwd/`.
fn workspace(name: &str) -> (PathBuf, PathBuf) {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("adversary2-paths-{name}"));
    if root.exists() {
        std::fs::remove_dir_all(&root).expect("clear old workspace");
    }
    let cwd = root.join("cwd");
    let outside = root.join("outside");
    std::fs::create_dir_all(cwd.join("scenarios")).expect("create registry");
    std::fs::create_dir_all(&outside).expect("create outside");
    (cwd, outside)
}

/// The base investigation fixture, as this tree holds it.
fn base_fixture() -> String {
    std::fs::read_to_string(repository_root().join("fixtures/investigation/protocol.yaml"))
        .expect("base fixture")
}

/// The unit's own passing scenario, whose expected `canon-ir/1` is that of the base fixture, with
/// its fixture path replaced.
fn passing_scenario(fixture: &str) -> String {
    let text = std::fs::read_to_string(
        repository_root().join("crates/canon-cli/tests/conform/passing/compile-passes.yaml"),
    )
    .expect("passing scenario");
    let original = "fixture: fixtures/investigation/protocol.yaml\n";
    assert!(
        text.contains(original),
        "the passing scenario names the base fixture"
    );
    text.replace(original, &format!("fixture: '{fixture}'\n"))
}

fn conform_run(cwd: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(cwd)
        .args(["conform", "run", "--scenarios", "scenarios"])
        .output()
        .expect("canon runs")
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("utf-8 output")
}

/// A fixture path that is a symbolic link inside the working directory, pointing at a file outside
/// it. The outside file is a valid protocol whose IR the scenario expects, so a scenario that
/// passes proves the outside file was read.
#[cfg(unix)]
#[test]
fn a_fixture_symlink_leading_outside_the_working_directory_is_not_read() {
    let (cwd, outside) = workspace("file-link");
    std::fs::write(outside.join("protocol.yaml"), base_fixture()).expect("write outside fixture");
    std::os::unix::fs::symlink(outside.join("protocol.yaml"), cwd.join("link.yaml"))
        .expect("symlink");
    std::fs::write(cwd.join("scenarios/a.yaml"), passing_scenario("link.yaml")).expect("write");

    let output = conform_run(&cwd);
    let report = text(&output.stdout);
    assert!(
        !report.contains("passed: scenario `RUNNER-COMPILE-PASSES`"),
        "a fixture outside the working directory was read and the scenario passed:\n{report}"
    );
    assert_ne!(output.status.code(), Some(0), "exit: {report}");
}

/// The same escape through a symbolic link to a directory: every path component is a plain name.
#[cfg(unix)]
#[test]
fn a_fixture_under_a_symlinked_directory_leading_outside_is_not_read() {
    let (cwd, outside) = workspace("dir-link");
    std::fs::write(outside.join("protocol.yaml"), base_fixture()).expect("write outside fixture");
    std::os::unix::fs::symlink(&outside, cwd.join("fixtures")).expect("symlink");
    std::fs::write(
        cwd.join("scenarios/a.yaml"),
        passing_scenario("fixtures/protocol.yaml"),
    )
    .expect("write");

    let output = conform_run(&cwd);
    let report = text(&output.stdout);
    assert!(
        !report.contains("passed: scenario `RUNNER-COMPILE-PASSES`"),
        "a fixture outside the working directory was read and the scenario passed:\n{report}"
    );
    assert_ne!(output.status.code(), Some(0), "exit: {report}");
}

/// `.` segments and empty components stay below the working directory and name the same file.
#[test]
fn dot_segments_and_empty_components_name_the_fixture_below_the_working_directory() {
    let (cwd, _) = workspace("dots");
    std::fs::create_dir_all(cwd.join("fixtures")).expect("create fixtures");
    std::fs::write(cwd.join("fixtures/protocol.yaml"), base_fixture()).expect("write fixture");
    std::fs::write(
        cwd.join("scenarios/a.yaml"),
        passing_scenario("./fixtures//./protocol.yaml"),
    )
    .expect("write");

    let output = conform_run(&cwd);
    assert_eq!(text(&output.stderr), "");
    assert_eq!(
        text(&output.stdout),
        "passed: scenario `RUNNER-COMPILE-PASSES`\n\
         conform: 1 passed, 0 failed, 0 unreadable\n"
    );
    assert_eq!(output.status.code(), Some(0));
}

/// A fixture path naming a directory fails the step as unreadable, on one line, exit 1.
#[test]
fn a_fixture_path_naming_a_directory_fails_the_step() {
    let (cwd, _) = workspace("directory");
    std::fs::create_dir_all(cwd.join("fixtures")).expect("create fixtures");
    std::fs::write(cwd.join("scenarios/a.yaml"), passing_scenario("fixtures")).expect("write");

    let output = conform_run(&cwd);
    assert_eq!(text(&output.stderr), "");
    assert_eq!(
        text(&output.stdout),
        "failed: scenario `RUNNER-COMPILE-PASSES` step `compile`: fixture `fixtures` is unreadable: Is a directory (os error 21)\n\
         conform: 0 passed, 1 failed, 0 unreadable\n"
    );
    assert_eq!(output.status.code(), Some(1));
}

/// A fixture path longer than the platform allows fails the step on one line, exit 1, and the run
/// is byte-identical when repeated.
#[test]
fn a_fixture_path_longer_than_the_platform_allows_fails_the_step_on_one_line() {
    let (cwd, _) = workspace("long");
    let long = "a/".repeat(3000) + "p.yaml";
    std::fs::write(cwd.join("scenarios/a.yaml"), passing_scenario(&long)).expect("write");

    let output = conform_run(&cwd);
    assert_eq!(text(&output.stderr), "");
    let report = text(&output.stdout);
    let lines: Vec<&str> = report.lines().collect();
    assert_eq!(lines.len(), 2, "{report}");
    assert!(
        lines[0]
            .starts_with("failed: scenario `RUNNER-COMPILE-PASSES` step `compile`: fixture `a/a/"),
        "{}",
        lines[0]
    );
    assert_eq!(lines[1], "conform: 0 passed, 1 failed, 0 unreadable");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(conform_run(&cwd).stdout, output.stdout, "repeat run");
}
