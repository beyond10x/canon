//! Adversary pass 1 on story:canon-mutate (wave 2026-11-20-w9).
//!
//! The story (§ Outcome, "Which scenarios run against a mutant"): `canon mutate` hands the
//! mutant's text in for the fixture whose path is the `--path` file, and every other fixture
//! unchanged; a mutant no scenario fails is a survivor, and a weakening survivor is a weak point.

use std::path::PathBuf;
use std::process::{Command, Output};

fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn canon(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(repository_root())
        .args(args)
        .output()
        .expect("the canon binary runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// `shared.yaml` is the fixture of one scenario and an import of the other, whose fixture
/// `consumer.yaml` pins it by the digest of its compiled IR. Both scenarios record both checks, so
/// no scenario's behaviour tells `item.ready` from either of its conjuncts: the three weakening
/// mutants survive. Handing the mutant in for the import as well makes the consumer's scenario
/// fail on `import-digest-mismatch`, a failure every mutant that changes the IR has, and reports
/// the three as killed, the run exiting 0.
#[test]
fn a_mutant_handed_in_for_a_pinned_import_is_not_killed_by_the_digest() {
    let output = canon(&[
        "mutate",
        "--path",
        "fixtures/mutate-adversary/import/shared.yaml",
        "--scenarios",
        "fixtures/mutate-adversary/import/scenarios",
    ]);
    assert_eq!(
        (output.status.code(), text(&output.stdout), text(&output.stderr)),
        (
            Some(1),
            "weak point: drop-conjunct claim `item.ready` true_when all[0]: RELAXATION (weakening); \
             no scenario fails\n\
             weak point: drop-conjunct claim `item.ready` true_when all[1]: RELAXATION (weakening); \
             no scenario fails\n\
             weak point: all-to-any claim `item.ready` true_when: RELAXATION (weakening); no \
             scenario fails\n\
             mutate: 3 mutants, 0 killed, 3 survived (3 weak points), 0 invalid\n"
                .to_owned(),
            String::new()
        ),
        "exit status, standard output and standard error"
    );
}

/// Story claim 1 (`crates/canon/src/mutate/mod.rs`, § Classification): a mutant whose written text
/// does not read back as the mutant is `invalid` with the code `unwritable`. An artifact identifier
/// of 1023 bytes parses as a plain key; `model::write` quotes it, 1025 bytes, which the reader
/// drops, so no mutant of this protocol reads back. Without the guard each mutant's text fails to
/// parse in the scenario run and is reported `killed`. The scenario passes on any protocol `p`:
/// its case names protocol `q`, and the evaluation refuses it.
#[test]
fn mutate_reports_a_mutant_that_does_not_read_back_as_unwritable() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("adversary-mutate-unwritable");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("scenarios")).expect("create the scenario directory");
    let artifact = "a".repeat(1023);
    std::fs::write(
        dir.join("p.yaml"),
        format!(
            "format: protocol/1\nprotocol: {{id: p, revision: 1}}\n\
             artifacts:\n  {artifact}: {{}}\n\
             evidence_kinds: {{k: {{}}, l: {{}}}}\n\
             claims:\n  c:\n    true_when:\n      all:\n        - evidence: {{kind: k}}\n        \
             - evidence: {{kind: l}}\n"
        ),
    )
    .expect("write the protocol");
    std::fs::write(
        dir.join("scenarios/a.yaml"),
        "format: canon-conformance/1\nid: A\ncovers: []\nfixture: p.yaml\nsteps:\n  - id: s\n    \
         evaluate:\n      case: {format: canon-case/1, id: c, protocol: q, artifacts: {}}\n      \
         evidence: []\n    expect:\n      refusal: protocol-mismatch\n",
    )
    .expect("write the scenario");
    let output = Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(&dir)
        .args(["mutate", "--path", "p.yaml", "--scenarios", "scenarios"])
        .output()
        .expect("the canon binary runs");
    assert_eq!(
        (
            output.status.code(),
            text(&output.stdout),
            text(&output.stderr)
        ),
        (
            Some(0),
            "invalid: drop-conjunct claim `c` true_when all[0]: unwritable\n\
             invalid: drop-conjunct claim `c` true_when all[1]: unwritable\n\
             invalid: all-to-any claim `c` true_when: unwritable\n\
             mutate: 3 mutants, 0 killed, 0 survived (0 weak points), 3 invalid\n"
                .to_owned(),
            String::new()
        ),
        "exit status, standard output and standard error"
    );
}
