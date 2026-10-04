//! `canon conform run` over registries the acceptance does not build: an empty registry, a
//! scenario whose fixture path leaves the working directory, and a fixture that does not parse.
//! Each runs in its own scratch working directory, so fixture paths resolve there.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A fresh working directory holding an empty registry directory `scenarios/`.
fn workspace(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("conform-registry-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clear old workspace");
    }
    std::fs::create_dir_all(dir.join("scenarios")).expect("create registry");
    dir
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

fn scenario(fixture: &str) -> String {
    format!(
        "format: canon-conformance/1\nid: S\ncovers: []\nfixture: '{fixture}'\nsteps:\n  - id: compile\n    compile: {{}}\n    expect: {{canon-ir: x}}\n"
    )
}

#[test]
fn an_empty_registry_is_refused() {
    let cwd = workspace("empty");
    std::fs::write(cwd.join("scenarios/notes.txt"), "not a scenario file").expect("write");
    let output = conform_run(&cwd);
    assert_eq!(text(&output.stderr), "");
    assert_eq!(
        text(&output.stdout),
        "no scenario ran: the registry holds no scenario file\n\
         conform: 0 passed, 0 failed, 0 unreadable\n"
    );
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn a_fixture_outside_the_working_directory_is_refused_unread() {
    let cwd = workspace("escaping");
    let outside = cwd
        .parent()
        .expect("scratch parent")
        .join("conform-registry-outside.yaml");
    std::fs::write(&outside, "SECRET-OUTSIDE: 1\n").expect("write outside file");
    let absolute = outside.to_str().expect("utf-8 path").to_owned();
    std::fs::write(cwd.join("scenarios/a.yaml"), scenario(&absolute)).expect("write");
    std::fs::write(
        cwd.join("scenarios/b.yaml"),
        scenario("../conform-registry-outside.yaml").replace("id: S", "id: T"),
    )
    .expect("write");

    let output = conform_run(&cwd);
    assert_eq!(text(&output.stderr), "");
    assert_eq!(
        text(&output.stdout),
        format!(
            "unreadable: scenarios/a.yaml: fixture `{absolute}` is not a relative path without `..` components\n\
             unreadable: scenarios/b.yaml: fixture `../conform-registry-outside.yaml` is not a relative path without `..` components\n\
             conform: 0 passed, 0 failed, 2 unreadable\n"
        )
    );
    assert!(!text(&output.stdout).contains("SECRET"));
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn a_fixture_that_does_not_parse_is_not_echoed() {
    let cwd = workspace("unparseable-fixture");
    std::fs::write(
        cwd.join("credentials.yaml"),
        "token: SECRET-TOKEN-VALUE\nformat: protocol/1\n",
    )
    .expect("write fixture");
    std::fs::write(cwd.join("scenarios/a.yaml"), scenario("credentials.yaml")).expect("write");

    let output = conform_run(&cwd);
    assert_eq!(text(&output.stderr), "");
    assert_eq!(
        text(&output.stdout),
        "failed: scenario `S` step `compile`: fixture does not parse: not a protocol/1 document at line 1 column 1\n\
         conform: 0 passed, 1 failed, 0 unreadable\n"
    );
    assert!(!text(&output.stdout).contains("SECRET"));
    assert_eq!(output.status.code(), Some(1));
}

/// A symbolic link is followed before the fixture is read: one leading outside the working
/// directory reports its scenario unreadable, naming the path, and the target is never read; one
/// that stays inside is read like the file it names.
#[cfg(unix)]
#[test]
fn a_fixture_symlink_is_judged_by_where_it_resolves() {
    let cwd = workspace("symlink");
    let outside = cwd
        .parent()
        .expect("scratch parent")
        .join("conform-registry-symlink-outside.yaml");
    std::fs::write(&outside, "SECRET-THROUGH-LINK: 1\n").expect("write outside file");
    std::os::unix::fs::symlink(&outside, cwd.join("escape.yaml")).expect("symlink");
    std::fs::write(
        cwd.join("inside.yaml"),
        "format: protocol/1\nprotocol: {id: p, revision: 1}\n",
    )
    .expect("write inside fixture");
    std::os::unix::fs::symlink(cwd.join("inside.yaml"), cwd.join("alias.yaml")).expect("symlink");
    std::fs::write(cwd.join("scenarios/a.yaml"), scenario("escape.yaml")).expect("write");
    std::fs::write(
        cwd.join("scenarios/b.yaml"),
        scenario("alias.yaml").replace("id: S", "id: T"),
    )
    .expect("write");

    let output = conform_run(&cwd);
    assert_eq!(text(&output.stderr), "");
    assert_eq!(
        text(&output.stdout),
        "unreadable: scenarios/a.yaml: fixture `escape.yaml` resolves outside the working directory\n\
         failed: scenario `T` step `compile`: canon-ir/1 differs from the expectation at line 1\n\
         conform: 0 passed, 1 failed, 1 unreadable\n"
    );
    assert!(!text(&output.stdout).contains("SECRET"));
    assert_eq!(output.status.code(), Some(1));
}
