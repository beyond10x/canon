//! Adversary cases, second pass, for story:canon-ir (wave 2026-10-04-w2).
//!
//! `canon compile` prints `canon-ir/1` on stdout, and that output is meant to be redirected into a
//! file and hashed. Whenever it cannot compile, stdout must stay empty and the exit status must be
//! non-zero, so `canon compile --path x > x.ir.json` never leaves a plausible partial IR behind.
//! The unit's suite covers a missing file and three validator refusals; these cover the parse
//! refusal, a directory, bytes that are not UTF-8, and an empty file.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn scratch(name: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("adversary2-compile-{name}"))
}

fn compile(path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .arg("compile")
        .arg("--path")
        .arg(path)
        .output()
        .expect("canon runs")
}

fn refused(name: &str, output: &Output, code: i32, prefix: &str) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.stdout.is_empty(),
        "{name}: stdout {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(output.status.code(), Some(code), "{name}: {stderr}");
    assert!(stderr.starts_with(prefix), "{name}: {stderr}");
}

#[test]
fn a_document_that_does_not_parse_prints_nothing_and_exits_1() {
    let path = scratch("unparseable.yaml");
    std::fs::write(
        &path,
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nclaims:\n  c: {true_when: {all: [], any: []}}\n",
    )
    .expect("write");
    refused("unparseable", &compile(&path), 1, "error[parse]: ");
}

#[test]
fn an_empty_file_prints_nothing_and_exits_non_zero() {
    let path = scratch("empty.yaml");
    std::fs::write(&path, "").expect("write");
    let output = compile(&path);
    assert!(output.stdout.is_empty());
    assert_ne!(output.status.code(), Some(0));
}

#[test]
fn a_directory_prints_nothing_and_exits_2() {
    let path = scratch("directory");
    std::fs::create_dir_all(&path).expect("mkdir");
    refused("directory", &compile(&path), 2, "error[unreadable]: ");
}

#[test]
fn bytes_that_are_not_utf8_print_nothing_and_exit_non_zero() {
    let path = scratch("latin1.yaml");
    std::fs::write(
        &path,
        b"format: protocol/1\nprotocol: {id: p, revision: 1, description: caf\xe9}\n",
    )
    .expect("write");
    let output = compile(&path);
    assert!(output.stdout.is_empty());
    assert_ne!(output.status.code(), Some(0));
}

#[test]
fn a_wrong_format_prints_nothing_and_exits_1() {
    let path = scratch("wrong-format.yaml");
    std::fs::write(
        &path,
        "format: protocol/2\nprotocol: {id: p, revision: 1}\n",
    )
    .expect("write");
    refused(
        "wrong format",
        &compile(&path),
        1,
        "error[unsupported-format]: ",
    );
}
