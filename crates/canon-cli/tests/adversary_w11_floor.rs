//! Adversary pass on story:floor-guards-added-actions (review of wave 2026-11-22-w11).
//!
//! The rule (`crates/canon/src/floor/mod.rs`, `unguarded_added_actions`): an added action that
//! repeats an action the floor guards with a capability, "with the same precondition and at least
//! what it may produce and revise", and lacks the capability, is a `RELAXATION`. "Same" is
//! equality of the compiled predicate, and compile sorts and dedups the members of `all` and
//! `any` but does not unwrap a one-member `all`. The floor fixture's `publish_finding` produces
//! and revises nothing, so its precondition is all that identifies it.
//!
//! - An added action whose precondition is the floor's wrapped in `all: [...]` (the same truth
//!   value in every state) publishes the floor's finding without the capability: below the floor.
//! - An added action with no precondition at all publishes the finding in every state the floor's
//!   action does, and more, without the capability: below the floor. The classifier calls the same
//!   weakening of an existing action's precondition a `RELAXATION`.
//! - An added action at the floor's precondition that only inspects (it produces an observation
//!   and publishes nothing) is not the floor's finding: it stays an `EXPANSION`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FLOOR: &str = "fixtures/investigation/floor/floor.yaml";

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn canon(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(repository_root())
        .args(args)
        .output()
        .expect("canon runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adversary-w11-floor")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory creates");
    dir
}

/// Compiles `source` to `dir/<name>.json` and returns that path.
fn compiled(dir: &Path, name: &str, source: &str) -> String {
    let yaml = dir.join(format!("{name}.yaml"));
    std::fs::write(&yaml, source).expect("the source is written");
    let output = canon(&["compile", "--path", &yaml.display().to_string()]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{name} compiles: {}",
        text(&output.stderr)
    );
    let path = dir.join(format!("{name}.json"));
    std::fs::write(&path, &output.stdout).expect("the IR is written");
    path.display().to_string()
}

fn floor_source() -> String {
    std::fs::read_to_string(repository_root().join(FLOOR)).expect("the floor fixture reads")
}

/// The floor under its own id, with `publish_finding` unchanged and one added action `name`
/// whose declaration after its description is `body`.
fn with_added_action(name: &str, body: &str) -> String {
    let anchor = "    requires:\n      - capability: finding.publish\n";
    let source = floor_source();
    assert_eq!(
        source.matches(anchor).count(),
        1,
        "the floor names its capability once"
    );
    source
        .replacen("id: floor.investigation", "id: investigation.added", 1)
        .replacen(
            anchor,
            &format!("{anchor}\n  {name}:\n    description: An added action.\n{body}"),
            1,
        )
}

/// `canon floor` of the floor against the floor with the action `name` added: exit status and
/// standard output.
fn floor_of_added(dir: &Path, name: &str, body: &str) -> (Option<i32>, String) {
    let floor = compiled(dir, "floor", &floor_source());
    let added = compiled(dir, name, &with_added_action(name, body));
    let output = canon(&["floor", "--floor", &floor, "--ir", &added]);
    (output.status.code(), text(&output.stdout))
}

#[test]
fn an_added_action_whose_precondition_is_the_floors_inside_a_one_member_all_breaches_the_floor() {
    let dir = scratch("wrapped");
    let (status, stdout) = floor_of_added(
        &dir,
        "publish_finding_wrapped",
        "    precondition:\n      all:\n        - claim: explanation.supported\n          is: true\n",
    );
    assert_eq!(
        (status, stdout.lines().next().unwrap_or_default()),
        (Some(1), "floor: breached"),
        "an added action that publishes at the floor's precondition, written as a one-member \
         `all`, without `finding.publish`, passes the floor:\n{stdout}"
    );
}

#[test]
fn an_added_action_with_no_precondition_and_no_capability_breaches_the_floor() {
    let dir = scratch("unconditional");
    let (status, stdout) = floor_of_added(&dir, "publish_finding_now", "");
    assert_eq!(
        (status, stdout.lines().next().unwrap_or_default()),
        (Some(1), "floor: breached"),
        "an added action that publishes in every state, the floor's included, without \
         `finding.publish`, passes the floor:\n{stdout}"
    );
}

#[test]
fn an_added_action_at_the_floors_precondition_that_only_inspects_stays_an_expansion() {
    let dir = scratch("inspects");
    let (status, stdout) = floor_of_added(
        &dir,
        "inspect_again",
        "    precondition:\n      claim: explanation.supported\n      is: true\n    may_produce:\n      - evidence: supporting_observation\n",
    );
    assert_eq!(
        (status, stdout.as_str()),
        (
            Some(0),
            "floor: holds\nTIGHTENING: 0\nEXPANSION: 1\nNO SEMANTIC CHANGE: 0\n"
        ),
        "an added action that only records an observation once the explanation is supported is \
         refused as if it published the finding"
    );
}
