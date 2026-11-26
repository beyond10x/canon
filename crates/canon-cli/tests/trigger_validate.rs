//! story:trigger-rule-document, `canon trigger validate --path <file>` against the command line:
//! a valid `canon-trigger/1` document prints its rule key and exits 0; a refused one prints one
//! `error[<code>]: <field>: <why>` line per error on standard error, naming the field, and exits 1;
//! a file that cannot be read exits 2. What each refusal means is the conformance scenario
//! `conformance/scenarios/trigger-rule-document.yaml`; this file holds the command's output.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Read at run time: a test binary reused from a shared build directory must read this tree.
fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn validate(path: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(repository_root())
        .args(["trigger", "validate", "--path", path])
        .output()
        .expect("canon runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// The `(code, field)` of each `error[<code>]: <field>: <why>` line, in order.
fn named(stderr: &str) -> Vec<(String, String)> {
    stderr
        .lines()
        .map(|line| {
            let rest = line
                .strip_prefix("error[")
                .unwrap_or_else(|| panic!("not an error line: {line}"));
            let (code, rest) = rest.split_once("]: ").expect("a code");
            let (field, _) = rest.split_once(": ").expect("a field");
            (code.to_owned(), field.to_owned())
        })
        .collect()
}

fn pairs(expected: &[(&str, &str)]) -> Vec<(String, String)> {
    expected
        .iter()
        .map(|(code, field)| ((*code).to_owned(), (*field).to_owned()))
        .collect()
}

#[test]
fn a_document_with_every_field_prints_its_rule_key() {
    let output = validate("fixtures/trigger/stale-invoice-reminder.yaml");
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_eq!(
        text(&output.stdout),
        "valid: trigger rule `stale-invoice-reminder@3`\n"
    );
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn a_mode_outside_its_values_is_refused_naming_mode() {
    let output = validate("fixtures/trigger/invalid/mode-sometimes.yaml");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(text(&output.stdout), "");
    assert_eq!(
        named(&text(&output.stderr)),
        pairs(&[("invalid-value", "mode")])
    );
}

#[test]
fn every_error_of_a_well_shaped_document_is_named_in_field_order() {
    let output = validate("fixtures/trigger/invalid/several.yaml");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(text(&output.stdout), "");
    assert_eq!(
        named(&text(&output.stderr)),
        pairs(&[
            ("invalid-value", "version"),
            ("invalid-value", "owner"),
            ("undeclared-field", "subject.state.field"),
            ("invalid-where", "where"),
            ("undeclared-field", "start.input.invoice"),
            ("invalid-value", "cap"),
        ])
    );
}

#[test]
fn text_that_is_not_yaml_is_a_parse_error_and_a_missing_file_is_unreadable() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("trigger-validate");
    std::fs::create_dir_all(&dir).expect("scratch directory created");
    let broken = dir.join("broken.yaml");
    std::fs::write(&broken, "format: [canon-trigger/1\n").expect("scratch file written");
    let output = validate(broken.to_str().expect("utf-8 path"));
    assert_eq!(output.status.code(), Some(1));
    let stderr = text(&output.stderr);
    assert!(
        stderr.starts_with("error[parse]: not well-formed YAML at line "),
        "{stderr}"
    );
    assert_eq!(stderr.lines().count(), 1, "{stderr}");

    let missing = dir.join("missing.yaml");
    let output = validate(missing.to_str().expect("utf-8 path"));
    assert_eq!(output.status.code(), Some(2));
    assert!(
        text(&output.stderr).starts_with("error[unreadable]: "),
        "{}",
        text(&output.stderr)
    );
}
