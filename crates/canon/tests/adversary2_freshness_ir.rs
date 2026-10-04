//! Adversary pass 2 for story:evidence-freshness: `max_age` in `canon-ir/1`, written by
//! `Ir::canonical_json` and read back by `eval::read_ir`.
//!
//! `ir/mod.rs`: every field is present, except an evidence kind's `max_age`, "written, as authored,
//! only when the kind declares one, so the IR of a protocol that declares none is the IR it was
//! before `max_age` existed". `eval/read.rs`: `read_ir` accepts exactly the bytes `canon compile`
//! prints.

use std::path::PathBuf;

use b10x_canon::eval;
use b10x_canon::ir;
use b10x_canon::model;

/// Read at run time: a test binary reused from a shared build directory must read this tree.
fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn compile(source: &str) -> ir::Ir {
    ir::compile(&model::parse(source).expect("parses")).expect("compiles")
}

/// Every protocol fixture of the tree (the `invalid/` variants aside) compiles to an IR that
/// `read_ir` reads back to the same IR and the same bytes.
#[test]
fn every_fixture_round_trips_through_read_ir() {
    let dir = repository_root().join("fixtures/investigation");
    let mut names: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("fixtures directory reads")
        .map(|entry| entry.expect("entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "yaml"))
        .collect();
    names.sort();
    let mut compiled = 0;
    for path in &names {
        let text = std::fs::read_to_string(path).expect("fixture reads");
        let Ok(protocol) = model::parse(&text) else {
            continue;
        };
        let Ok(ir) = ir::compile(&protocol) else {
            continue;
        };
        let bytes = ir.canonical_json();
        let read =
            eval::read_ir(&bytes).unwrap_or_else(|refusal| panic!("{}: {refusal}", path.display()));
        assert_eq!(read, ir, "{}", path.display());
        assert_eq!(read.canonical_json(), bytes, "{}", path.display());
        compiled += 1;
    }
    assert!(
        names
            .iter()
            .any(|path| path.ends_with("evidence-freshness.yaml")),
        "the freshness fixture is among them"
    );
    assert!(compiled >= 2, "{compiled} fixtures compiled");
}

/// The freshness fixture is the base plus one `max_age`; its IR is the base's IR plus exactly
/// one member, `"max_age": "1h"`, after `description` in that kind, and nothing else.
#[test]
fn the_freshness_ir_is_the_base_ir_plus_one_max_age_member() {
    let root = repository_root();
    let read = |name: &str| {
        std::fs::read_to_string(root.join("fixtures/investigation").join(name)).expect("reads")
    };
    let base = compile(&read("protocol.yaml")).canonical_json();
    let fresh = compile(&read("evidence-freshness.yaml")).canonical_json();
    let expected = base.replace(
        "    \"supporting_observation\": {\n      \"description\": \"An observation consistent with the explanation.\"\n    }",
        "    \"supporting_observation\": {\n      \"description\": \"An observation consistent with the explanation.\",\n      \"max_age\": \"1h\"\n    }",
    );
    assert_ne!(
        expected, base,
        "the base names supporting_observation as expected"
    );
    assert_eq!(fresh, expected);
}

/// `max_age` sorts after `description`; the same members in the other order, an explicit null,
/// an empty string, or a second spelling of the same length (`60m` for `1h`) are not the bytes
/// `canon compile` prints for this protocol, and each is refused or reads as a different IR.
#[test]
fn only_the_canonical_max_age_member_reads_back() {
    let ir = compile(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nevidence_kinds: {k: {max_age: 1h}}\n",
    );
    let text = ir.canonical_json();
    let canonical = "\"k\": {\n      \"description\": null,\n      \"max_age\": \"1h\"\n    }";
    assert!(text.contains(canonical), "{text}");
    let reordered = text.replace(
        canonical,
        "\"k\": {\n      \"max_age\": \"1h\",\n      \"description\": null\n    }",
    );
    assert_eq!(
        eval::read_ir(&reordered)
            .expect_err("reordered members are refused")
            .code(),
        "malformed-input"
    );
    let empty = text.replace("\"max_age\": \"1h\"", "\"max_age\": \"\"");
    assert_eq!(
        eval::read_ir(&empty)
            .expect_err("an empty max_age is refused")
            .code(),
        "malformed-input"
    );
    let minutes = eval::read_ir(&text.replace("\"1h\"", "\"60m\"")).expect("60m is canonical");
    assert_ne!(minutes, ir, "60m and 1h are different IRs, as the docs say");
}
