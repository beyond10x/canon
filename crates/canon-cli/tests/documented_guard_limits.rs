//! Two documented limits of `runs_only_when` (story:ordering-guards-provable-by-check), pinned
//! against the command line so that a change of either is a decision, not an accident:
//!
//! - `canon floor` does not pin a guard: `canon diff` classifies a removed guard as `NO SEMANTIC
//!   CHANGE`, so a protocol that drops a guard its floor declares stays above the floor.
//! - `canon validate` refuses only the literal `all: []` as `vacuous-runs-only-when`; another guard
//!   that is true in every state (`{any: [{all: []}]}`, `{all: [{all: []}]}`, `{not: {any: []}}`)
//!   is not detected and validates.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The fixture whose action `publish_finding` declares a guard, relative to the repository root.
const GUARDED: &str = "fixtures/investigation/check/ordering-guard-broken/protocol.yaml";

/// The guard as the fixture declares it.
const GUARD: &str = "    runs_only_when:\n      claim: explanation.survived\n";

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn canon_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("the canon binary runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// An empty directory of its own under the target's scratch directory.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("documented-guard-limits")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

/// `canon compile` of `source`, written in `dir` as `<name>.yaml`; the IR is written beside it as
/// `<name>.json`, whose file name is returned.
fn compiled(dir: &Path, name: &str, source: &str) -> String {
    let yaml = format!("{name}.yaml");
    std::fs::write(dir.join(&yaml), source).expect("written");
    let output = canon_in(dir, &["compile", "--path", &yaml]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    let json = format!("{name}.json");
    std::fs::write(dir.join(&json), &output.stdout).expect("written");
    json
}

/// A floor that guards `publish_finding` holds for a protocol that is the floor without that
/// guard: the floor does not pin a guard.
#[test]
fn a_floor_does_not_pin_a_guard() {
    let dir = scratch("floor");
    let guarded = std::fs::read_to_string(repository_root().join(GUARDED)).expect("readable");
    assert_eq!(guarded.matches(GUARD).count(), 1, "{guarded}");
    let floor = compiled(&dir, "floor", &guarded);
    let refined = compiled(&dir, "refined", &guarded.replace(GUARD, ""));
    let output = canon_in(&dir, &["floor", "--floor", &floor, "--ir", &refined]);
    assert_eq!(text(&output.stderr), "");
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        text(&output.stdout),
        "floor: holds\nTIGHTENING: 0\nEXPANSION: 0\nNO SEMANTIC CHANGE: 1\n"
    );
}

/// A protocol whose one action `go` declares `guard` as its `runs_only_when`.
fn guarded_by(guard: &str) -> String {
    format!(
        "format: protocol/1\nprotocol: {{id: vacuous, revision: 1}}\nartifacts: {{a: {{}}}}\n\
         evidence_kinds: {{done: {{}}}}\n\
         claims: {{done: {{true_when: {{evidence: {{kind: done}}}}}}}}\n\
         actions:\n  go:\n    precondition: {{claim: done, is: unknown}}\n\
         \x20   runs_only_when: {guard}\n    may_produce: [{{evidence: done}}]\n\
         outcomes: {{finished: {{requires: {{claim: done}}}}}}\n"
    )
}

/// `canon validate` of `source`, written in `dir` as `protocol.yaml`.
fn validated(dir: &Path, source: &str) -> Output {
    std::fs::write(dir.join("protocol.yaml"), source).expect("written");
    canon_in(dir, &["validate", "--path", "protocol.yaml"])
}

/// The literal `all: []` is refused; three other guards true in every state validate.
#[test]
fn only_the_literal_empty_all_is_refused_as_a_vacuous_guard() {
    let dir = scratch("vacuous");
    let refused = validated(&dir, &guarded_by("{all: []}"));
    assert_eq!(refused.status.code(), Some(1), "{}", text(&refused.stdout));
    assert!(
        text(&refused.stderr).starts_with("error[vacuous-runs-only-when]: "),
        "{}",
        text(&refused.stderr)
    );
    for guard in [
        "{any: [{all: []}]}",
        "{all: [{all: []}]}",
        "{not: {any: []}}",
    ] {
        let output = validated(&dir, &guarded_by(guard));
        assert_eq!(output.status.code(), Some(0), "{guard}: {}", text(&output.stderr));
        assert_eq!(text(&output.stderr), "", "{guard}");
    }
}
