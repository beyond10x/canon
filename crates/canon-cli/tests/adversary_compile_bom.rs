//! Adversary case for story:canon-ir (wave 2026-10-04-w2).
//!
//! A protocol file saved with a UTF-8 byte-order mark is the same document (YAML 1.2 § 5.2 allows a
//! byte-order mark at the start of a stream), so `canon compile` gives it the same `canon-ir/1` as
//! the file without one. Today the mark reaches `model::parse` unchanged and the document is
//! refused as a parse error.

use std::path::Path;
use std::process::Command;

fn compile(path: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .arg("compile")
        .arg("--path")
        .arg(path)
        .output()
        .expect("canon runs")
}

#[test]
fn a_byte_order_mark_does_not_change_the_ir() {
    let base = Path::new(
        &std::env::var_os("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo"),
    )
    .join("../../fixtures/investigation/protocol.yaml");
    let source = std::fs::read(&base).expect("read base fixture");
    let mut marked = vec![0xEF, 0xBB, 0xBF];
    marked.extend_from_slice(&source);
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join("adversary-bom-protocol.yaml");
    std::fs::write(&path, marked).expect("write marked copy");

    let plain = compile(&base);
    let with_mark = compile(&path);
    assert_eq!(plain.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&with_mark.stderr),
        "",
        "exit {:?}",
        with_mark.status.code()
    );
    assert_eq!(with_mark.stdout, plain.stdout);
}
