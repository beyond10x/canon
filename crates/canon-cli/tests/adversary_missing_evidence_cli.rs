//! Adversary case for story:missing-evidence (wave 2026-11-20-w4, pass 1), through the `canon`
//! binary: `canon evaluate` writes the `missing_evidence` section CANON-FRONTIER-001 expects for a
//! record invalidated by an upstream change, and the names and formats of the evidence files
//! change no byte of the decision.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FIXTURE: &str = "fixtures/investigation/missing-evidence.yaml";

const CASE: &str = "format: canon-case/1\nid: INV-20\nprotocol: investigation\n\
    artifacts: {explanation: {revision: r1}, dataset: {revision: d2}}\n\
    independence: {author-1: [], reviewer-2: [different_principal]}\n";

const OBSERVATION_YAML: &str = "format: canon-evidence/1\nid: observation-1\n\
    kind: supporting_observation\nsubject: explanation\nsubject_revision: r1\n\
    upstream_revisions: {dataset: d1}\n";

const FALSIFICATION_JSON: &str = "{\"format\": \"canon-evidence/1\", \"id\": \"falsification-1\", \
    \"kind\": \"falsification_attempt\", \"result\": \"survived\", \"producer\": \"reviewer-2\", \
    \"subject\": \"explanation\", \"subject_revision\": \"r1\"}\n";

const SECTION: &str = "  \"missing_evidence\": {\n    \"attempt_falsification\": [],\n    \
    \"inspect\": [\n      \"supporting_observation\"\n    ]\n  },\n";

fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("run through cargo"))
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn canon(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(root())
        .args(args)
        .output()
        .expect("canon runs")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adversary_missing_evidence_cli")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("stale scratch removes");
    }
    std::fs::create_dir_all(&dir).expect("scratch creates");
    dir
}

fn path(p: &Path) -> &str {
    p.to_str().expect("utf-8 path")
}

/// Evaluates the fixture with the two records written under the given file names.
fn evaluate(name: &str, observation: &str, falsification: &str) -> Output {
    let dir = scratch(name);
    let compiled = canon(&["compile", "--path", FIXTURE]);
    assert_eq!(compiled.status.code(), Some(0), "the fixture compiles");
    let ir = dir.join("ir.json");
    std::fs::write(&ir, &compiled.stdout).expect("IR writes");
    let case = dir.join("case.yaml");
    std::fs::write(&case, CASE).expect("case writes");
    let evidence = dir.join("evidence");
    std::fs::create_dir(&evidence).expect("evidence directory creates");
    std::fs::write(evidence.join(observation), OBSERVATION_YAML).expect("record writes");
    std::fs::write(evidence.join(falsification), FALSIFICATION_JSON).expect("record writes");
    canon(&[
        "evaluate",
        "--ir",
        path(&ir),
        "--case",
        path(&case),
        "--evidence",
        path(&evidence),
    ])
}

#[test]
fn an_invalidated_record_lists_its_kind_and_file_names_change_no_byte() {
    let first = evaluate("first", "a.yaml", "b.json");
    let second = evaluate("second", "z.yaml", "0.json");
    assert_eq!(
        first.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let stdout = String::from_utf8(first.stdout).expect("utf-8");
    assert!(
        stdout.contains(&format!("  \"format\": \"canon-decision/1\",\n{SECTION}")),
        "the section follows `format` with the expected lists:\n{stdout}"
    );
    assert_eq!(stdout.matches("\"missing_evidence\"").count(), 1);
    assert_eq!(stdout, String::from_utf8(second.stdout).expect("utf-8"));
}
