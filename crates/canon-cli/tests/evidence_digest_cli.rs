//! story:evidence-digest-binds-approvals, through the command line. `canon digest --evidence
//! <file>` prints the digest an evidence ground gives, and `canon evaluate` holds an approval
//! resting on it while the record file under that id is the one hashed, and lapses it as
//! `content_changed` once the file holds other contents.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use clap::CommandFactory;

/// Read at run time: a test binary reused from a shared build directory must read this tree.
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
        .expect("canon runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("evidence-digest-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("old scratch directory removed");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory created");
    dir
}

fn path(p: &Path) -> String {
    p.to_str().expect("utf-8 path").to_owned()
}

/// The record `e1` of CANON-GROUND-002 (`conformance/scenarios/evidence-digest.yaml`).
const E1: &str = "format: canon-evidence/1\nid: e1\nkind: k\nsubject: spec\nsubject_revision: r2\n\
                  observed_at: 2026-11-21T10:00:00Z\nupstream_revisions: {up: u2}\n";

/// [`E1`] as JSON, its keys in another order.
const E1_AS_JSON: &str = "{\"upstream_revisions\": {\"up\": \"u2\"}, \"observed_at\": \
                          \"2026-11-21T10:00:00Z\", \"subject_revision\": \"r2\", \"subject\": \
                          \"spec\", \"kind\": \"k\", \"id\": \"e1\", \"format\": \
                          \"canon-evidence/1\"}";

/// The SHA-256 of [`E1`]'s canonical form, computed outside Canon (`sha256sum` over the text the
/// scenario's comment writes out).
const E1_DIGEST: &str = "sha256:b5971ff0209e762046a75b9465832b0b24892d22b49fbb2cb05684472c4d6dea";

/// Other contents stored under `e1` later, still applicable.
const E1_REPLACED: &str = "format: canon-evidence/1\nid: e1\nkind: k\nresult: fail\nsubject: spec\n\
                           subject_revision: r2\nobserved_at: 2026-11-21T10:00:00Z\n\
                           upstream_revisions: {up: u2}\nproducer: mallory\n";

/// `canon digest` prints the digest and a newline, exit 0, whatever the file's key order.
#[test]
fn digest_prints_the_digest_of_the_record() {
    let dir = scratch("prints");
    for (name, record) in [("e1.yaml", E1), ("e1.json", E1_AS_JSON)] {
        let file = dir.join(name);
        std::fs::write(&file, record).expect("record written");
        let out = canon(&["digest", "--evidence", &path(&file)]);
        assert_eq!(out.status.code(), Some(0), "{name}: {}", text(&out.stderr));
        assert_eq!(text(&out.stdout), format!("{E1_DIGEST}\n"), "{name}");
        assert_eq!(text(&out.stderr), "", "{name}");
    }
}

/// A file that does not read as a `canon-evidence/1` record exits 1 naming the code; a file that
/// cannot be read exits 2.
#[test]
fn digest_refuses_what_is_not_a_record() {
    let dir = scratch("refuses");
    let not_a_record = dir.join("case.yaml");
    std::fs::write(
        &not_a_record,
        "format: canon-case/1\nid: C-1\nprotocol: p\n",
    )
    .expect("written");
    let out = canon(&["digest", "--evidence", &path(&not_a_record)]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out.stderr));
    assert!(
        text(&out.stderr).starts_with("error[malformed-input]: "),
        "{}",
        text(&out.stderr)
    );
    assert_eq!(text(&out.stdout), "");

    let other_format = dir.join("other.yaml");
    std::fs::write(
        &other_format,
        E1.replace("canon-evidence/1", "canon-evidence/2"),
    )
    .expect("written");
    let out = canon(&["digest", "--evidence", &path(&other_format)]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out.stderr));
    assert!(
        text(&out.stderr).starts_with("error[unsupported-format]: "),
        "{}",
        text(&out.stderr)
    );

    let missing = dir.join("missing.yaml");
    let out = canon(&["digest", "--evidence", &path(&missing)]);
    assert_eq!(out.status.code(), Some(2), "{}", text(&out.stderr));
    assert!(
        text(&out.stderr).starts_with("error[unreadable]: "),
        "{}",
        text(&out.stderr)
    );
}

/// Acceptance 2 through `canon evaluate`: the grant carrying the digest `canon digest` printed
/// makes `act` admissible; after other contents are written to the same file under the same id,
/// the grant lapses as `content_changed`.
#[test]
fn evaluate_lapses_a_grant_when_the_record_file_is_rewritten() {
    let dir = scratch("evaluate");
    let compiled = canon(&[
        "compile",
        "--path",
        "fixtures/investigation/decision-grounds.yaml",
    ]);
    assert_eq!(
        compiled.status.code(),
        Some(0),
        "{}",
        text(&compiled.stderr)
    );
    let ir = dir.join("grounds.ir.json");
    std::fs::write(&ir, &compiled.stdout).expect("IR written");
    let case = dir.join("case.yaml");
    std::fs::write(
        &case,
        "format: canon-case/1\nid: G-1\nprotocol: grounds\nrevision: c2\n\
         artifacts: {spec: {revision: r2}, up: {revision: u2}}\n",
    )
    .expect("case written");
    let evidence = dir.join("evidence");
    std::fs::create_dir(&evidence).expect("evidence directory created");
    let record = evidence.join("e1.yaml");
    std::fs::write(&record, E1).expect("record written");

    let printed = canon(&["digest", "--evidence", &path(&record)]);
    assert_eq!(printed.status.code(), Some(0), "{}", text(&printed.stderr));
    let digest = text(&printed.stdout).trim_end().to_owned();
    let authority = dir.join("authority.yaml");
    std::fs::write(
        &authority,
        format!(
            "- capability: x\n  decision: granted\n  ground_evidence:\n    \
             - {{id: e1, digest: '{digest}'}}\n"
        ),
    )
    .expect("authority written");

    let evaluate = || {
        let out = canon(&[
            "evaluate",
            "--ir",
            &path(&ir),
            "--case",
            &path(&case),
            "--evidence",
            &path(&evidence),
            "--authority",
            &path(&authority),
            "--at",
            "2026-11-21T10:30:00Z",
        ]);
        assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
        let decision: serde_yaml_ng::Value =
            serde_yaml_ng::from_slice(&out.stdout).expect("the decision is JSON");
        decision["actions"]["act"].clone()
    };
    let holds = evaluate();
    assert_eq!(holds["status"].as_str(), Some("admissible"), "{holds:?}");

    std::fs::write(&record, E1_REPLACED).expect("record rewritten");
    let lapsed = evaluate();
    assert_eq!(
        lapsed["status"].as_str(),
        Some("approval-required"),
        "{lapsed:?}"
    );
    let stale = &lapsed["reasons"][0]["stale_grounds"][0];
    assert_eq!(stale["evidence"].as_str(), Some("e1"), "{lapsed:?}");
    assert_eq!(
        stale["reason"].as_str(),
        Some("content_changed"),
        "{lapsed:?}"
    );
}

/// The help of `--authority` and `--decisions` names the map form, `canon digest` and
/// `content_changed`, lower case, whitespace runs as one space.
#[test]
fn evaluate_help_names_the_digest_and_how_to_compute_it() {
    let command = canon_cli::Cli::command();
    let evaluate = command
        .find_subcommand("evaluate")
        .expect("canon evaluate exists");
    for flag in ["authority", "decisions"] {
        let arg = evaluate
            .get_arguments()
            .find(|arg| arg.get_long() == Some(flag))
            .unwrap_or_else(|| panic!("--{flag} exists"));
        let help = arg
            .get_long_help()
            .or_else(|| arg.get_help())
            .map(ToString::to_string)
            .unwrap_or_default()
            .to_lowercase();
        let help = help.split_whitespace().collect::<Vec<_>>().join(" ");
        for words in [
            "`{id: <id>, digest: <digest>}`",
            "`canon digest`",
            "`content_changed`",
        ] {
            assert!(
                help.contains(words),
                "--{flag} help does not say {words}: {help}"
            );
        }
    }
}
