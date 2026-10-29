//! Adversary pass 1 on story:case-composition against the `canon` command line.
//!
//! - `canon evaluate` refuses two `case_outcome` records for one child, naming the file of each.
//! - `canon generate` over protocols beyond the fixture (a `children` form beside a subject-bound
//!   match demanding independence; a form listing `other`, which the check space uses for a record
//!   without a result) writes witnesses that `canon conform run` passes.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn canon(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("the canon binary runs")
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("utf-8 output")
}

/// A fresh scratch directory under the test target's own tmpdir.
fn scratch(name: &str) -> PathBuf {
    let work = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    if work.exists() {
        std::fs::remove_dir_all(&work).expect("an old scratch directory is removable");
    }
    std::fs::create_dir_all(&work).expect("a scratch directory is creatable");
    work
}

fn record(id: &str, child: &str, result: &str) -> String {
    format!(
        "format: canon-evidence/1\nid: {id}\nkind: case_outcome\nresult: {result}\n\
         subject: findings\nsubject_revision: f1\nchild_case: {child}\n"
    )
}

#[test]
fn evaluate_names_the_file_of_each_record_for_one_child() {
    let work = scratch("adversary-case-composition-evaluate");
    let compiled = canon(
        &repository_root(),
        &[
            "compile",
            "--path",
            "fixtures/investigation/case-composition.yaml",
        ],
    );
    assert_eq!(
        compiled.status.code(),
        Some(0),
        "{}",
        text(&compiled.stderr)
    );
    std::fs::write(work.join("ir.json"), &compiled.stdout).expect("the IR writes");
    std::fs::write(
        work.join("case.yaml"),
        "format: canon-case/1\nid: AUD-7\nprotocol: audit\nartifacts:\n  findings: {revision: f1}\n",
    )
    .expect("the case writes");
    std::fs::create_dir_all(work.join("evidence")).expect("the evidence directory");
    // File names in the other order than the record ids.
    std::fs::write(
        work.join("evidence/a.yaml"),
        record("o2", "CHG-1", "accepted"),
    )
    .expect("a record writes");
    std::fs::write(work.join("evidence/b.yaml"), record("o1", "CHG-1", "open"))
        .expect("a record writes");
    std::fs::write(work.join("evidence/c.yaml"), record("o3", "CHG-2", "open"))
        .expect("a record writes");
    let run = canon(
        &work,
        &[
            "evaluate",
            "--ir",
            "ir.json",
            "--case",
            "case.yaml",
            "--evidence",
            "evidence",
        ],
    );
    assert_eq!(text(&run.stdout), "");
    assert_eq!(
        text(&run.stderr),
        "error[duplicate-child-case]: evidence/b.yaml, evidence/a.yaml: child case `CHG-1` has \
         more than one case_outcome record: `o1`, `o2`\n"
    );
    assert_eq!(run.status.code(), Some(1));
}

/// `canon check` accepts `source`, and every witness `canon generate` writes for it passes under
/// `canon conform run`.
fn generated_witnesses_pass(name: &str, source: &str) {
    let work = scratch(name);
    std::fs::write(work.join("p.yaml"), source).expect("the protocol writes");
    let checked = canon(&work, &["check", "--path", "p.yaml"]);
    assert_eq!(
        checked.status.code(),
        Some(0),
        "{}{}",
        text(&checked.stdout),
        text(&checked.stderr)
    );
    let generated = canon(&work, &["generate", "--path", "p.yaml", "--out", "out"]);
    assert_eq!(
        generated.status.code(),
        Some(0),
        "{}{}",
        text(&generated.stdout),
        text(&generated.stderr)
    );
    let written = std::fs::read_dir(work.join("out"))
        .expect("the witnesses are written")
        .count();
    assert!(written > 0, "no witness written");
    let run = canon(&work, &["conform", "run", "--scenarios", "out"]);
    assert_eq!(
        run.status.code(),
        Some(0),
        "{}{}",
        text(&run.stdout),
        text(&run.stderr)
    );
}

#[test]
fn generated_witnesses_pass_beside_a_bound_match_demanding_independence() {
    generated_witnesses_pass(
        "adversary-case-composition-generate-independence",
        "format: protocol/1\n\
         protocol: {id: p, revision: 1}\n\
         artifacts: {findings: {}}\n\
         evidence_kinds: {case_outcome: {}}\n\
         independence_dimensions: {team: {}}\n\
         actions:\n\
         \x20\x20open: {requires: [{capability: case.open}], case_open: {protocol: q, subject: findings}}\n\
         \x20\x20close: {precondition: {children: {in: [accepted, declined]}}}\n\
         claims:\n\
         \x20\x20done: {true_when: {children: {in: [accepted]}}}\n\
         \x20\x20acc_f: {true_when: {evidence: {kind: case_outcome, subject: findings, result: accepted, independent_on: [team]}}}\n\
         outcomes:\n\
         \x20\x20finished: {requires: {all: [{claim: done}, {claim: acc_f}]}}\n\
         \x20\x20stalled: {requires: {claim: done, is: unknown}}\n\
         \x20\x20failed: {requires: {claim: done, is: false}}\n",
    );
}

#[test]
fn generated_witnesses_pass_for_a_form_that_lists_other() {
    generated_witnesses_pass(
        "adversary-case-composition-generate-other",
        "format: protocol/1\n\
         protocol: {id: p, revision: 1}\n\
         artifacts: {findings: {}}\n\
         evidence_kinds: {case_outcome: {}}\n\
         actions:\n\
         \x20\x20open: {case_open: {protocol: q, subject: findings}}\n\
         \x20\x20close: {precondition: {not: {children: {in: [other, \"other'\"]}}}}\n\
         claims:\n\
         \x20\x20done: {true_when: {children: {in: [other, \"other'\"]}}}\n\
         \x20\x20m: {true_when: {evidence: {kind: case_outcome, result: \"other''\"}}}\n\
         outcomes:\n\
         \x20\x20finished: {requires: {claim: done}}\n\
         \x20\x20failed: {requires: {all: [{claim: done, is: false}, {claim: m, is: false}]}}\n",
    );
}
