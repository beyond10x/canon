//! Adversary cases for story:protocol-source-model (wave 2026-10-04-w1).
//!
//! Every fixture under `fixtures/investigation/invalid/` parses and is rejected by validation, so
//! no existing case drives `canon validate` through its parse-error branch. A mutant that makes
//! that branch exit 0 keeps the suite green; this case catches it.

use std::path::Path;
use std::process::Command;

#[test]
fn validate_rejects_a_document_that_does_not_parse() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/adversary-unparseable.yaml");
    let output = Command::new(env!("CARGO_BIN_EXE_canon"))
        .arg("validate")
        .arg("--path")
        .arg(&path)
        .output()
        .expect("canon runs");
    let stderr = String::from_utf8(output.stderr).expect("utf-8 stderr");
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(output.stdout.is_empty());
    assert!(
        stderr.starts_with("error[parse]: ") && stderr.contains("owner"),
        "{stderr}"
    );
}
