//! story:supplied-evidence-required-at-open against the command line: the optional evidence-kind
//! key `required_at_open: true` says the case's opener hands in a current record of the kind when
//! it opens the case. `canon check-set` holds the opener to it (`unsupplied-at-open`, pinned in
//! `check_across_protocol_files_cli.rs`), `canon simulate` starts with the record (`at-open`,
//! pinned in `canon_simulate.rs`); this file holds the rest:
//!
//! - `canon compile` writes `"required_at_open": true` only when declared true, in key order
//!   before `supplied_by`; `false` and an absent key compile to the bytes the protocol had before
//!   the key existed. The IR with it reads back as `canon evaluate --ir` reads it.
//! - `canon evaluate`, `canon check` and `canon generate` give the same bytes with and without it.
//! - `canon diff` reports it introduced as `TIGHTENING` and removed as `RELAXATION`.
//! - `canon validate` refuses it on a kind without `supplied_by` (`at-open-without-supplier`,
//!   naming the kind), and a value that is not a boolean does not parse.
//! - The protocol writer writes the key after `supplied_by`, and the text reads back as the model.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use b10x_canon::model;

/// The fixture that declares the key, relative to the repository root.
const AT_OPEN: &str = "fixtures/investigation/simulate/at-open/protocol.yaml";

/// The invalid fixture: the key on a kind without a supplier.
const WITHOUT_SUPPLIER: &str = "fixtures/investigation/invalid/at-open-without-supplier.yaml";

/// The key as the fixture declares it.
const KEY: &str = "    required_at_open: true\n";

/// The key as `canon compile` writes it, with the comma that ends the key before it.
const COMPILED_KEY: &str = ",\n      \"required_at_open\": true";

fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn canon_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("the canon binary runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn path(path: &Path) -> &str {
    path.to_str().expect("a UTF-8 path")
}

/// An empty directory of its own under the target's scratch directory.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("supplied-at-open")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

/// Exit 0, nothing on standard error; standard output.
fn succeeded(output: &Output) -> String {
    assert_eq!(text(&output.stderr), "");
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stdout));
    text(&output.stdout)
}

/// The fixture's text, and the same text without the key.
fn sources() -> (String, String) {
    let with = std::fs::read_to_string(repository_root().join(AT_OPEN)).expect("readable");
    assert_eq!(with.matches(KEY).count(), 1);
    let without = with.replace(KEY, "");
    (with, without)
}

/// `source` written as `name` in `dir`, and its path.
fn written(dir: &Path, name: &str, source: &str) -> PathBuf {
    let file = dir.join(name);
    std::fs::write(&file, source).expect("written");
    file
}

/// `canon compile` of `protocol`, written beside it as `<name>.json`; the IR and its path.
fn compiled(dir: &Path, protocol: &Path, name: &str) -> (String, PathBuf) {
    let ir = succeeded(&canon_in(dir, &["compile", "--path", path(protocol)]));
    let file = written(dir, &format!("{name}.json"), &ir);
    (ir, file)
}

/// The IR holds the key only when it is declared true, in key order; `false` and no key
/// compile to the same bytes, which are the IR with the key less it. Both read back as `canon
/// evaluate --ir` reads an IR, to the same decision.
#[test]
fn the_ir_writes_required_at_open_only_when_true() {
    let dir = scratch("ir");
    let (with, without) = sources();
    let (with_ir, with_at) = compiled(&dir, &written(&dir, "with.yaml", &with), "with");
    let (without_ir, without_at) =
        compiled(&dir, &written(&dir, "without.yaml", &without), "without");
    let (false_ir, _) = compiled(
        &dir,
        &written(
            &dir,
            "false.yaml",
            &with.replace(KEY, "    required_at_open: false\n"),
        ),
        "false",
    );
    assert_eq!(with_ir.matches(COMPILED_KEY).count(), 1, "{with_ir}");
    assert!(
        with_ir
            .contains("\"required_at_open\": true,\n      \"supplied_by\": \"simulate.review\"\n"),
        "{with_ir}"
    );
    assert!(!without_ir.contains("required_at_open"), "{without_ir}");
    assert_eq!(with_ir.replace(COMPILED_KEY, ""), without_ir);
    assert_eq!(false_ir, without_ir);

    let case = written(
        &dir,
        "case.yaml",
        "format: canon-case/1\nid: C-1\nprotocol: simulate.at-open\n\
         artifacts: {report: {revision: r1}}\n",
    );
    let evidence = dir.join("evidence");
    std::fs::create_dir_all(&evidence).expect("evidence directory");
    written(
        &evidence,
        "review.yaml",
        "format: canon-evidence/1\nid: e1\nkind: peer_review\nresult: approved\n\
         subject: report\nsubject_revision: r1\n",
    );
    let evaluate = |ir: &Path| {
        succeeded(&canon_in(
            &dir,
            &[
                "evaluate",
                "--ir",
                path(ir),
                "--case",
                path(&case),
                "--evidence",
                path(&evidence),
            ],
        ))
    };
    assert_eq!(evaluate(&with_at), evaluate(&without_at));
}

/// `canon check` and `canon generate` read nothing of the key: the same bytes with and without
/// it, each run on its protocol at the same relative path, since a scenario names that path.
#[test]
fn check_and_generate_read_nothing_of_the_key() {
    let (with, without) = sources();
    let run = |name: &str, source: &str| {
        let dir = scratch(name);
        written(&dir, "protocol.yaml", source);
        let checked = canon_in(&dir, &["check", "--path", "protocol.yaml"]);
        succeeded(&canon_in(
            &dir,
            &["generate", "--path", "protocol.yaml", "--out", "out"],
        ));
        let mut files: Vec<(String, String)> = std::fs::read_dir(dir.join("out"))
            .expect("scenarios written")
            .map(|entry| {
                let entry = entry.expect("an entry");
                (
                    entry.file_name().to_string_lossy().into_owned(),
                    std::fs::read_to_string(entry.path()).expect("a scenario reads"),
                )
            })
            .collect();
        files.sort();
        (
            checked.status.code(),
            text(&checked.stdout),
            text(&checked.stderr),
            files,
        )
    };
    let plain = run("plain", &without);
    assert!(!plain.3.is_empty());
    assert_eq!(run("at-open", &with), plain);
}

/// Requiring the record at open asks more of the opener (`TIGHTENING`); no longer requiring it
/// asks less (`RELAXATION`); each names the kind.
#[test]
fn diff_tightens_when_the_key_is_introduced_and_relaxes_when_removed() {
    let dir = scratch("diff");
    let (with, without) = sources();
    let (_, with_at) = compiled(&dir, &written(&dir, "with.yaml", &with), "with");
    let (_, without_at) = compiled(&dir, &written(&dir, "without.yaml", &without), "without");
    let diff = |from: &Path, to: &Path| {
        succeeded(&canon_in(
            &dir,
            &["diff", "--from", path(from), "--to", path(to)],
        ))
    };
    let head = "diff: protocol `simulate.at-open` revision 1 to 1\n";
    assert_eq!(
        diff(&without_at, &with_at),
        format!("{head}TIGHTENING: evidence kind `peer_review` required at open introduced\n")
    );
    assert_eq!(
        diff(&with_at, &without_at),
        format!("{head}RELAXATION: evidence kind `peer_review` required at open removed\n")
    );
    assert_eq!(
        diff(&with_at, &with_at),
        format!("{head}NO SEMANTIC CHANGE: no declaration differs\n")
    );
}

/// The key needs `supplied_by` on the same kind; a value that is not a boolean does not parse.
#[test]
fn validate_refuses_the_key_without_a_supplier() {
    let root = repository_root();
    let output = canon_in(&root, &["validate", "--path", WITHOUT_SUPPLIER]);
    assert_eq!(
        (
            output.status.code(),
            text(&output.stdout),
            text(&output.stderr)
        ),
        (
            Some(1),
            String::new(),
            "error[at-open-without-supplier]: evidence kind `peer_review` is required at open \
             and names no protocol that supplies it (`supplied_by`)\n"
                .to_owned()
        )
    );
    let dir = scratch("validate");
    let (with, _) = sources();
    let quoted = written(
        &dir,
        "quoted.yaml",
        &with.replace(KEY, "    required_at_open: \"true\"\n"),
    );
    let output = canon_in(&dir, &["validate", "--path", path(&quoted)]);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.starts_with("error[parse]: "), "{stderr}");
}

/// The protocol writer writes `required_at_open` after `supplied_by`, and the text reads back as
/// the model; without the key, or with `false`, it writes none.
#[test]
fn the_writer_writes_required_at_open_after_the_supplier() {
    let (with, without) = sources();
    let model = model::parse(&with).expect("parses");
    let written = model::write(&model);
    assert_eq!(model::parse(&written).expect("reads back"), model);
    let at = |key: &str| {
        written
            .find(&format!("\"{key}\": "))
            .unwrap_or_else(|| panic!("{key} in {written}"))
    };
    assert!(at("supplied_by") < at("required_at_open"), "{written}");
    assert!(written.contains("\"required_at_open\": true"), "{written}");
    let plain = model::parse(&without).expect("parses");
    assert!(!model::write(&plain).contains("required_at_open"));
    let off = model::parse(&with.replace(KEY, "    required_at_open: false\n")).expect("parses");
    assert_eq!(off, plain);
    assert!(!model::write(&off).contains("required_at_open"));
}
