//! Adversary cases, second pass, for story:canon-ir (wave 2026-10-04-w2).
//!
//! `canon-ir/1` bytes are for hashing (design § 30), so the same protocol saved on another platform
//! or by another editor must hash the same. Pass 1 checked CRLF on the base fixture, whose text is
//! all single-line plain scalars; these cases check the multi-line scalar styles, a classic-Mac CR
//! line ending and a missing final newline. The base fixture is read at run time.

use std::path::PathBuf;

use b10x_canon::{ir, model};

fn base_fixture() -> String {
    let path = PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo"),
    )
    .join("../../fixtures/investigation/protocol.yaml");
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{path:?}: {error}"))
}

fn ir(name: &str, source: &str) -> String {
    let parsed = model::parse(source).unwrap_or_else(|error| panic!("{name}: {error}"));
    ir::compile(&parsed)
        .unwrap_or_else(|problems| panic!("{name}: {problems:?}"))
        .canonical_json()
}

/// Descriptions written as a literal block, a folded block, a multi-line plain scalar and a
/// multi-line double-quoted scalar.
const MULTI_LINE: &str = "format: protocol/1\nprotocol:\n  id: p\n  revision: 2\n  description: |\n    first line\n    second line\nartifacts:\n  folded:\n    description: >\n      one\n      two\n\n      three\n  plain:\n    description: one\n      two\n  quoted:\n    description: \"one\n      two\\\n      three\"\n";

#[test]
fn multi_line_text_compiles_the_same_whatever_the_line_endings() {
    let lf = ir("lf", MULTI_LINE);
    assert!(
        lf.contains("\"description\": \"first line\\nsecond line\\n\""),
        "{lf}"
    );
    assert!(
        lf.contains("\"description\": \"one two\\nthree\\n\""),
        "{lf}"
    );
    assert!(lf.contains("\"description\": \"one two\""), "{lf}");
    assert!(lf.contains("\"description\": \"one twothree\""), "{lf}");
    assert_eq!(ir("crlf", &MULTI_LINE.replace('\n', "\r\n")), lf, "crlf");
    assert_eq!(ir("cr", &MULTI_LINE.replace('\n', "\r")), lf, "cr");
}

#[test]
fn the_base_fixture_compiles_the_same_without_its_final_newline_or_with_extra_blank_lines() {
    let base = base_fixture();
    let expected = ir("base", &base);
    let trimmed = base.trim_end_matches('\n');
    assert_ne!(trimmed.len(), base.len(), "the fixture ends with a newline");
    assert_eq!(ir("no final newline", trimmed), expected);
    assert_eq!(ir("blank lines", &format!("{base}\n\n\n")), expected);
    assert_eq!(
        ir("classic-mac line endings", &base.replace('\n', "\r")),
        expected
    );
}
