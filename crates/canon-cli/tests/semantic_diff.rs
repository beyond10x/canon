//! Acceptance for story:semantic-diff: `canon diff --from --to` compares two revisions of one
//! protocol on their compiled `canon-ir/1` and classifies every change as `TIGHTENING`,
//! `RELAXATION`, `BREAKING`, `EXPANSION` or `NO SEMANTIC CHANGE` (design § 31), one line per change
//! naming its category and the identifiers it touches, after a line naming the protocol and the two
//! revisions.
//!
//! The specification is the revision pairs under `fixtures/investigation/semantic-diff/`: one
//! directory per pair, holding the earlier revision `from.yaml` and the later one `to.yaml`, each a
//! `protocol/1` document derived from the investigation fixture with one change between them.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The revision pairs, one directory each.
const PAIRS: &str = "fixtures/investigation/semantic-diff";

/// Every pair and the report `canon diff` prints for it, byte for byte.
const EXPECTATIONS: [(&str, &str); 8] = [
    (
        "added-obligation",
        "diff: protocol `investigation` revision 1 to 2\n\
         TIGHTENING: obligation `establish.explanation` added\n",
    ),
    (
        "shortened-max-age",
        "diff: protocol `investigation` revision 1 to 2\n\
         TIGHTENING: evidence kind `supporting_observation` maximum age shortened from 1h to 30m\n",
    ),
    (
        "removed-capability",
        "diff: protocol `investigation` revision 1 to 2\n\
         RELAXATION: action `publish_finding` no longer requires capability `finding.publish`\n",
    ),
    (
        "added-action",
        "diff: protocol `investigation` revision 1 to 2\n\
         EXPANSION: action `request_review` added\n",
    ),
    (
        "changed-outcome-requirement",
        "diff: protocol `investigation` revision 1 to 2\n\
         BREAKING: outcome `supported` requirement changed\n",
    ),
    (
        "description-only",
        "diff: protocol `investigation` revision 1 to 2\n\
         NO SEMANTIC CHANGE: protocol `investigation` description changed\n\
         NO SEMANTIC CHANGE: artifact `explanation` description changed\n\
         NO SEMANTIC CHANGE: claim `explanation.supported` description changed\n",
    ),
    (
        "claim-all-conjunct",
        "diff: protocol `investigation` revision 1 to 2\n\
         TIGHTENING: claim `explanation.supported` predicate strengthened\n",
    ),
    (
        "claim-any-alternative",
        "diff: protocol `investigation` revision 1 to 2\n\
         RELAXATION: claim `explanation.supported` predicate weakened\n",
    ),
];

/// Read at run time, not compile time: a test binary reused from a shared build directory must
/// still read this tree's fixtures.
fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

/// Runs `canon <args>` from the repository root, so that fixture paths resolve against it.
fn canon(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(repository_root())
        .args(args)
        .output()
        .expect("canon runs")
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("utf-8 output")
}

/// A fresh, empty directory under this test's scratch space; each pair uses its own name.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("semantic_diff")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("stale scratch directory removes");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory creates");
    dir
}

/// Compiles one revision of a pair with `canon compile` and writes its `canon-ir/1` to `dir`.
fn compiled(pair: &str, revision: &str, dir: &Path) -> String {
    let source = format!("{PAIRS}/{pair}/{revision}.yaml");
    let run = canon(&["compile", "--path", &source]);
    assert_eq!(text(&run.stderr), "", "{source}: compile stderr");
    assert_eq!(run.status.code(), Some(0), "{source}: compiles");
    let ir = dir.join(format!("{revision}.ir.json"));
    std::fs::write(&ir, &run.stdout).expect("IR writes");
    ir.to_str().expect("utf-8 path").to_owned()
}

fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
        .map(|entry| {
            entry
                .expect("a directory entry is readable")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

#[test]
fn diff_classifies_investigation_revisions() {
    // Every pair directory is one of the expectations, so none is skipped silently, and each holds
    // exactly its two revisions.
    let pairs = repository_root().join(PAIRS);
    let mut named: Vec<String> = EXPECTATIONS
        .iter()
        .map(|(pair, _)| (*pair).to_owned())
        .collect();
    named.sort();
    assert_eq!(
        entries(&pairs),
        named,
        "{PAIRS}: one directory per expectation"
    );
    for (pair, _) in EXPECTATIONS {
        assert_eq!(
            entries(&pairs.join(pair)),
            ["from.yaml", "to.yaml"],
            "{PAIRS}/{pair}: the earlier and the later revision"
        );
    }

    for (pair, expected) in EXPECTATIONS {
        let dir = scratch(pair);
        let from = compiled(pair, "from", &dir);
        let to = compiled(pair, "to", &dir);
        let run = canon(&["diff", "--from", &from, "--to", &to]);
        assert_eq!(text(&run.stderr), "", "{pair}: stderr");
        assert_eq!(text(&run.stdout), expected, "{pair}: classification");
        assert_eq!(run.status.code(), Some(0), "{pair}: exit");
    }
}
