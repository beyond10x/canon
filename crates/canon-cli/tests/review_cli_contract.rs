//! Independent review of wave 2026-10-04-w7: `canon`'s exit statuses and standard error against
//! the contract it publishes.
//!
//! `canon_cli::EXIT_STATUSES` is the exit-status table `canon-docs` renders into
//! `website/docs/reference/cli.md`: 0 success, 1 rejected, 2 "Unreadable: an input file or
//! directory could not be read." A command line clap rejects (a required option missing) exits 2
//! as well, clap's own usage-error status, with clap's `error: …` text rather than an
//! `error[<code>]: …` line. A caller reading the published table takes that for an unreadable
//! file.
//!
//! `canon evaluate` reads the evidence directory file by file, and its documentation says a file
//! it cannot use is "refused as unreadable, naming it" (`crates/canon-cli/src/evaluate.rs`). A file
//! that is read but is not a `canon-evidence/1` document is refused by the library as
//! `malformed-input`, and the line names neither the file nor the record.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn canon(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .args(args)
        .output()
        .expect("canon runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("review-w7-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("old scratch directory removed");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory created");
    dir
}

fn path(p: &Path) -> &str {
    p.to_str().expect("utf-8 path")
}

/// The meaning the published table gives an exit status.
fn documented(code: i32) -> &'static str {
    canon_cli::EXIT_STATUSES
        .iter()
        .find(|(status, _)| i32::from(*status) == code)
        .map(|(_, meaning)| *meaning)
        .unwrap_or("(not in the table)")
}

#[test]
fn a_usage_error_exits_with_a_status_the_table_says_means_a_usage_error() {
    let dir = scratch("usage");
    let ir = dir.join("ir.json");
    std::fs::write(&ir, "{}\n").expect("written");
    // `--case` and `--evidence` are required and missing; nothing is unreadable.
    let run = canon(&["evaluate", "--ir", path(&ir)]);
    let code = run.status.code().expect("an exit status");
    let meaning = documented(code);
    assert!(
        meaning.to_lowercase().contains("usage")
            || meaning.to_lowercase().contains("argument")
            || meaning.to_lowercase().contains("command line"),
        "a missing required option exits {code}, which the published table describes as \
         {meaning:?}; stderr:\n{}",
        text(&run.stderr)
    );
}

#[test]
fn an_evidence_file_that_is_not_a_record_is_named_in_the_refusal() {
    let dir = scratch("malformed-evidence");
    let protocol = dir.join("protocol.yaml");
    std::fs::write(
        &protocol,
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {}}\nclaims: {c: {true_when: {evidence: {kind: k}}}}\n",
    )
    .expect("written");
    let compiled = canon(&["compile", "--path", path(&protocol)]);
    assert_eq!(
        compiled.status.code(),
        Some(0),
        "{}",
        text(&compiled.stderr)
    );
    let ir = dir.join("ir.json");
    std::fs::write(&ir, &compiled.stdout).expect("written");
    let case = dir.join("case.yaml");
    std::fs::write(
        &case,
        "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r1}}\n",
    )
    .expect("written");
    let evidence = dir.join("evidence");
    std::fs::create_dir_all(&evidence).expect("created");
    for name in ["a-ok.yaml", "c-ok.yaml"] {
        std::fs::write(
            evidence.join(name),
            format!(
                "format: canon-evidence/1\nid: {name}\nkind: k\nsubject: a\nsubject_revision: r1\n"
            ),
        )
        .expect("written");
    }
    // Read, and not a canon-evidence/1 document: `subject` is missing.
    std::fs::write(
        evidence.join("b-broken.yaml"),
        "format: canon-evidence/1\nid: e-b\nkind: k\nsubject_revision: r1\n",
    )
    .expect("written");

    let run = canon(&[
        "evaluate",
        "--ir",
        path(&ir),
        "--case",
        path(&case),
        "--evidence",
        path(&evidence),
    ]);
    let stderr = text(&run.stderr);
    assert_eq!(run.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("b-broken.yaml") || stderr.contains("e-b"),
        "the refusal of one evidence file among three names neither the file nor the record:\n\
         {stderr}"
    );
}
