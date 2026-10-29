//! Acceptance for story:case-composition against the `canon` command line. The fixture
//! `fixtures/investigation/case-composition.yaml` is the protocol `audit`, whose action
//! `open_change` opens a child `software.change` case about its findings and whose claim
//! `findings.resolved` and action `close` hold when every child case ended accepted or declined.
//!
//! - `canon validate` accepts the fixture and refuses `invalid/case-open-self.yaml`, whose action
//!   `reaudit` opens a case under the protocol's own id, naming the action (D206).
//! - `canon check` checks the fixture with no finding.
//! - `canon generate` writes a witness scenario per outcome and blocked action, its `case_outcome`
//!   records naming their child cases, and `canon conform run` passes every one.
//!
//! The values are CANON-COMPOSE-001 (`conformance/scenarios/case-composition.yaml`), which
//! `canon conform run` over the registry runs (`conform_requirements.rs`).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn canon(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("the canon binary runs")
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("utf-8 output")
}

#[test]
fn validate_accepts_the_fixture_and_refuses_a_self_opened_case_naming_the_action() {
    let root = repository_root();
    let valid = canon(
        &root,
        &[
            "validate",
            "--path",
            "fixtures/investigation/case-composition.yaml",
        ],
    );
    assert_eq!(text(&valid.stderr), "");
    assert_eq!(text(&valid.stdout), "valid: protocol `audit` revision 1\n");
    assert_eq!(valid.status.code(), Some(0));

    let refused = canon(
        &root,
        &[
            "validate",
            "--path",
            "fixtures/investigation/invalid/case-open-self.yaml",
        ],
    );
    assert_eq!(
        text(&refused.stderr),
        "error[self-opened-case]: action `reaudit` opens a case under protocol `audit`, the protocol's own id\n"
    );
    assert_eq!(text(&refused.stdout), "");
    assert_eq!(refused.status.code(), Some(1));
}

#[test]
fn check_finds_nothing_and_generated_witnesses_pass() {
    let root = repository_root();
    let checked = canon(
        &root,
        &[
            "check",
            "--path",
            "fixtures/investigation/case-composition.yaml",
        ],
    );
    assert_eq!(text(&checked.stderr), "");
    assert_eq!(
        text(&checked.stdout),
        "checked: protocol `audit` revision 1: 816 states, 0 properties, 0 findings\n"
    );
    assert_eq!(checked.status.code(), Some(0));

    let work = Path::new(env!("CARGO_TARGET_TMPDIR")).join("case-composition-generate");
    if work.exists() {
        std::fs::remove_dir_all(&work).expect("an old scratch directory is removable");
    }
    std::fs::create_dir_all(&work).expect("a scratch directory is creatable");
    std::fs::copy(
        root.join("fixtures/investigation/case-composition.yaml"),
        work.join("audit.yaml"),
    )
    .expect("the fixture copies");
    let generated = canon(&work, &["generate", "--path", "audit.yaml", "--out", "out"]);
    assert_eq!(
        generated.status.code(),
        Some(0),
        "{}",
        text(&generated.stderr)
    );
    let resolved = std::fs::read_to_string(work.join("out/outcome.resolved.legitimate.yaml"))
        .expect("the outcome's witness is written");
    assert!(resolved.contains("child_case: "), "{resolved}");
    let run = canon(&work, &["conform", "run", "--scenarios", "out"]);
    assert_eq!(run.status.code(), Some(0), "{}", text(&run.stdout));
}
