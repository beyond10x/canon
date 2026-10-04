//! Adversary cases for story:evaluator-skeleton (wave 2026-10-04-w6, pass 2), driving `canon
//! conform run`: a step's `authority` is passed to the evaluator as YAML text, re-serialized from
//! the YAML values the scenario holds (conform/mod.rs module docs).

use std::path::{Path, PathBuf};
use std::process::Command;

fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adversary2_skel_conform_cli")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("stale scratch directory removes");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory creates");
    dir
}

/// One evaluate step over the three-valued-claims fixture, giving `authority`, expecting the
/// refusal the skeleton gives any authority.
fn scenario(id: &str, authority: &str) -> String {
    format!(
        "format: canon-conformance/1\nid: {id}\ncovers: []\nfixture: fixtures/investigation/three-valued-claims.yaml\nsteps:\n  - id: e\n    evaluate:\n      case: {{format: canon-case/1, id: INV-18, protocol: investigation, artifacts: {{explanation: {{revision: r1}}}}}}\n      evidence: []\n      authority: {authority}\n    expect: {{refusal: unsupported-input}}\n"
    )
}

/// A registry whose scenarios all meet their expectations exits 0 and reports each of them. An
/// authority entry whose mapping key is itself a mapping is valid YAML the scenario parser
/// accepts; the runner's re-serialization panics on it, so `canon conform run` aborts with 101 and
/// prints no report, losing the verdict of every other scenario in the registry as well.
#[test]
fn an_authority_with_a_mapping_key_does_not_abort_the_registry() {
    let dir = scratch("mapping-key");
    std::fs::write(dir.join("a.yaml"), scenario("A-OK", "[]")).expect("writes");
    std::fs::write(dir.join("b.yaml"), scenario("B-KEY", "[{{x: 1}: c}]")).expect("writes");
    let output = Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(repository_root())
        .args([
            "conform",
            "run",
            "--scenarios",
            dir.to_str().expect("utf-8 path"),
        ])
        .output()
        .expect("canon runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("A-OK") && stdout.contains("B-KEY"),
        "{stdout}"
    );
}
