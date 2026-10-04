//! Adversary cases, second pass, for story:protocol-source-model (wave 2026-10-04-w1).
//!
//! `canon validate` prints the protocol id of a valid document unescaped. The identifier grammar
//! admits Unicode format characters, so a valid document can make that line display a different
//! protocol id from the one it declares.

use std::process::Command;

/// No other case gives `canon validate` a path it cannot read, so a mutant that exits 0 from the
/// unreadable branch (`main.rs`, `UNREADABLE`: "A document could not be read.") stays green.
#[test]
fn validate_exits_2_on_a_path_it_cannot_read() {
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("adversary2-absent.yaml");
    let _ = std::fs::remove_file(&path);
    let output = Command::new(env!("CARGO_BIN_EXE_canon"))
        .arg("validate")
        .arg("--path")
        .arg(&path)
        .output()
        .expect("canon runs");
    let stderr = String::from_utf8(output.stderr).expect("utf-8 stderr");
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(output.stdout.is_empty());
    assert!(stderr.starts_with("error[unreadable]: "), "{stderr}");
}

#[test]
fn the_valid_line_carries_no_reordering_characters() {
    let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("adversary2-bidi-id.yaml");
    std::fs::write(
        &path,
        "format: protocol/1\nprotocol: {id: \"p\\u202Elanif.noitagitsevni\", revision: 1}\n",
    )
    .expect("write fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_canon"))
        .arg("validate")
        .arg("--path")
        .arg(&path)
        .output()
        .expect("canon runs");
    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    assert!(
        !stdout.contains('\u{202e}'),
        "exit {:?}, stdout {stdout:?} displays as: {stdout}",
        output.status.code()
    );
}
