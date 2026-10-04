//! Acceptance for story:protocol-source-model: `canon validate --path` accepts the base
//! investigation fixture and rejects each invalid variant with an error naming the offending
//! identifier, byte-identical across repeated runs.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/investigation")
}

fn validate(path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .arg("validate")
        .arg("--path")
        .arg(path)
        .output()
        .expect("canon runs")
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("utf-8 output")
}

#[test]
fn validate_accepts_the_base_fixture() {
    let output = validate(&fixtures().join("protocol.yaml"));
    assert_eq!(text(&output.stderr), "");
    assert_eq!(
        text(&output.stdout),
        "valid: protocol `investigation` revision 1\n"
    );
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn validate_rejects_each_invalid_variant_naming_the_offending_identifier() {
    let cases = [
        (
            "undeclared-claim.yaml",
            "error[undeclared-claim]: outcome `supported` references claim `explanation.refuted`, which is not declared\n",
        ),
        (
            "duplicate-identifier.yaml",
            "error[duplicate-identifier]: action `inspect` is declared more than once\n",
        ),
        (
            "undeclared-evidence-kind.yaml",
            "error[undeclared-evidence-kind]: action `attempt_falsification` references evidence kind `expert_opinion`, which is not declared\n",
        ),
    ];
    for (file, expected) in cases {
        let output = validate(&fixtures().join("invalid").join(file));
        assert_eq!(text(&output.stderr), expected, "{file}");
        assert_eq!(text(&output.stdout), "", "{file}");
        assert_eq!(output.status.code(), Some(1), "{file}");
    }
}

#[test]
fn validate_rejects_every_file_under_invalid() {
    let mut files: Vec<PathBuf> = std::fs::read_dir(fixtures().join("invalid"))
        .expect("invalid fixtures exist")
        .map(|entry| entry.expect("directory entry").path())
        .collect();
    files.sort();
    assert!(files.len() >= 3, "found {files:?}");
    for file in files {
        let output = validate(&file);
        assert_eq!(output.status.code(), Some(1), "{}", file.display());
        assert!(!output.stderr.is_empty(), "{}", file.display());
    }
}

#[test]
fn validate_output_is_byte_identical_across_repeated_runs() {
    let mut paths = vec![fixtures().join("protocol.yaml")];
    for file in [
        "undeclared-claim.yaml",
        "duplicate-identifier.yaml",
        "undeclared-evidence-kind.yaml",
    ] {
        paths.push(fixtures().join("invalid").join(file));
    }
    for path in paths {
        let first = validate(&path);
        for _ in 0..4 {
            let again = validate(&path);
            assert_eq!(again.stdout, first.stdout, "{}", path.display());
            assert_eq!(again.stderr, first.stderr, "{}", path.display());
            assert_eq!(
                again.status.code(),
                first.status.code(),
                "{}",
                path.display()
            );
        }
    }
}

#[test]
fn validate_prints_one_line_per_problem_whatever_the_identifiers_contain() {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("forged-identifier.yaml");
    std::fs::write(
        &path,
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nclaims:\n  \"x\\nerror[duplicate-identifier]: action `inspect` is declared more than once\": {true_when: {all: []}}\n",
    )
    .expect("write fixture");
    let output = validate(&path);
    assert_eq!(output.status.code(), Some(1));
    let stderr = text(&output.stderr);
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(
        stderr.starts_with("error[invalid-identifier]: claim identifier `x\\nerror["),
        "{stderr}"
    );
}

#[test]
fn validate_prints_the_valid_line_through_the_same_escaping() {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("backslash-identifier.yaml");
    std::fs::write(
        &path,
        "format: protocol/1\nprotocol: {id: 'p\\q', revision: 1}\n",
    )
    .expect("write fixture");
    let output = validate(&path);
    assert_eq!(text(&output.stderr), "");
    assert_eq!(
        text(&output.stdout),
        "valid: protocol `p\\\\q` revision 1\n"
    );
    assert_eq!(output.status.code(), Some(0));
}
