//! Adversary pass 2 on story:review-hardening-w7, at the command line.
//!
//! `canon check` holds a protocol to the evaluator's effective-depth bound and refuses one beyond
//! it as `predicate-too-deep` (`check/mod.rs`, "Refusals"); `canon evaluate` names the file of
//! every record a refusal cites (`evaluate.rs`).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn canon(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("canon runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("adversary2-rh-w7-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("old scratch directory removed");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory created");
    dir
}

/// A protocol whose claims form one chain of `links` claims, each testing the next, the last
/// reading evidence. Every predicate is one or two levels deep; the chain's effective depth is
/// `links`.
fn chain_protocol(links: usize) -> String {
    let mut source = String::from(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {}}\nclaims:\n",
    );
    for at in 0..links - 1 {
        source.push_str(&format!(
            "  c{at:06}: {{true_when: {{claim: c{:06}}}}}\n",
            at + 1
        ));
    }
    source.push_str(&format!(
        "  c{:06}: {{true_when: {{evidence: {{kind: k}}}}}}\n",
        links - 1
    ));
    source
}

/// `canon check` on a protocol of 40 000 chained claims (1.6 MB): far beyond the effective-depth bound,
/// which `canon check` refuses as `predicate-too-deep` (exit 1). The claims are compiled before
/// the bound is checked, and compiling them must not abort the process either.
#[test]
fn check_refuses_a_long_chain_of_claims_without_aborting() {
    let dir = scratch("long-chain");
    std::fs::write(dir.join("p.yaml"), chain_protocol(40_000)).expect("protocol written");
    let checked = canon(&dir, &["check", "--path", "p.yaml"]);
    let stderr = text(&checked.stderr);
    assert!(
        checked.status.code() == Some(1)
            && stderr.starts_with("error[predicate-too-deep]: claim `c"),
        "canon check of 40 000 chained claims must be refused as predicate-too-deep; it ended \
         with {:?}, stderr: {}",
        checked.status,
        stderr.lines().take(3).collect::<Vec<_>>().join("\n")
    );
    std::fs::remove_dir_all(&dir).expect("scratch removed");
}

const IR_SOURCE: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
    evidence_kinds: {k: {}}\nclaims: {c: {true_when: {evidence: {kind: k}}}}\n";

/// One record id in three files, written in an order that is not file-name order: the refusal
/// names all three, in file-name order, on every run.
#[test]
fn an_id_in_three_files_names_the_three_in_file_name_order() {
    let dir = scratch("three-files");
    std::fs::write(dir.join("p.yaml"), IR_SOURCE).expect("protocol written");
    let compiled = canon(&dir, &["compile", "--path", "p.yaml"]);
    assert_eq!(
        compiled.status.code(),
        Some(0),
        "{}",
        text(&compiled.stderr)
    );
    std::fs::write(dir.join("p.json"), &compiled.stdout).expect("ir written");
    std::fs::write(
        dir.join("case.yaml"),
        "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r}}\n",
    )
    .expect("case written");
    let evidence = dir.join("evidence");
    std::fs::create_dir(&evidence).expect("evidence dir");
    for name in ["m.yaml", "z.json", "a.yaml"] {
        std::fs::write(
            evidence.join(name),
            "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r\n",
        )
        .expect("record written");
    }
    let expected = "error[duplicate-identifier]: evidence/a.yaml, evidence/m.yaml, \
                    evidence/z.json: evidence `e1` is given more than once\n";
    for _ in 0..3 {
        let run = canon(
            &dir,
            &[
                "evaluate",
                "--ir",
                "p.json",
                "--case",
                "case.yaml",
                "--evidence",
                "evidence",
            ],
        );
        assert_eq!(run.status.code(), Some(1));
        assert_eq!(text(&run.stdout), "");
        assert_eq!(text(&run.stderr), expected);
    }
    std::fs::remove_dir_all(&dir).expect("scratch removed");
}
