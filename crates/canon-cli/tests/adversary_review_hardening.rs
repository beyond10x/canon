//! Adversary pass 1 on story:review-hardening-w7, through the `canon` binary.
//!
//! `MAX_IR_DEPTH` bounds one predicate. A protocol author can chain claims, each testing the next
//! a hundred levels deep (the YAML reader stops a document at 128 levels), and `canon compile`
//! compiles it, `read_ir` reads it and `check_depth` passes it. Claim evaluation recurses through
//! each claim reference as well as through each level, so the depth it needs is the sum along the
//! chain.
//!
//! The story also says every evidence refusal names the record and `canon evaluate` names the
//! file; these cases check the refusals that come after reading (a repeated id, an undeclared
//! kind) as well as the one at reading.

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
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("adversary-rh-w7-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("old scratch directory removed");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory created");
    dir
}

fn path(p: &Path) -> &str {
    p.to_str().expect("utf-8 path")
}

const CHAIN: usize = 1500;
const NOTS: usize = 100;

/// A protocol whose claims form one chain, `c0000` testing `c0001` inside `NOTS` `not`s, and so on,
/// the last one reading evidence `k`. `NOTS` is even, so a `k` record makes every claim `true`.
fn chained_protocol() -> String {
    let mut source = String::from(
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {}}\nclaims:\n",
    );
    for at in 0..CHAIN {
        source.push_str(&format!(
            "  c{at:04}: {{true_when: {}{{claim: c{:04}}}{}}}\n",
            "{not: ".repeat(NOTS),
            at + 1,
            "}".repeat(NOTS)
        ));
    }
    source.push_str(&format!(
        "  c{CHAIN:04}: {{true_when: {{evidence: {{kind: k}}}}}}\n"
    ));
    source
}

/// `canon compile` accepts the chained protocol and `canon evaluate` must then exit with a status
/// the exit-status table lists (0 or 1), not be killed by a signal.
#[test]
fn a_compiled_chain_of_claims_evaluates_without_aborting() {
    let dir = scratch("chain");
    let protocol = dir.join("protocol.yaml");
    std::fs::write(&protocol, chained_protocol()).expect("written");
    let compiled = canon(&["compile", "--path", path(&protocol)]);
    assert_eq!(
        compiled.status.code(),
        Some(0),
        "the chained protocol compiles: {}",
        text(&compiled.stderr)
    );
    let ir = dir.join("ir.json");
    std::fs::write(&ir, &compiled.stdout).expect("written");
    let case = dir.join("case.yaml");
    std::fs::write(
        &case,
        "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r}}\n",
    )
    .expect("written");
    let evidence = dir.join("evidence");
    std::fs::create_dir_all(&evidence).expect("created");
    std::fs::write(
        evidence.join("e1.yaml"),
        "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r\n",
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
    assert!(
        matches!(run.status.code(), Some(0 | 1)),
        "canon evaluate of a compiled protocol of {CHAIN} chained claims, each {NOTS} levels \
         deep, ended with {:?}; stderr: {}",
        run.status,
        stderr.lines().take(3).collect::<Vec<_>>().join(" | ")
    );
}

/// Each evidence refusal `canon evaluate` gives names the file the refused record came from, so an
/// operator with many files can find it: the story's "every evidence refusal names the record, and
/// `canon evaluate` names the file".
#[test]
fn an_evidence_refusal_after_reading_names_the_file() {
    let protocol = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
                    evidence_kinds: {k: {}}\nclaims: {c: {true_when: {evidence: {kind: k}}}}\n";
    let record = |id: &str, kind: &str| {
        format!(
            "format: canon-evidence/1\nid: {id}\nkind: {kind}\nsubject: a\nsubject_revision: r\n"
        )
    };
    let cases = [
        ("repeated-id", record("e1", "k"), "duplicate-identifier"),
        (
            "undeclared-kind",
            record("e2", "z"),
            "undeclared-evidence-kind",
        ),
        ("bad-kind", record("e2", "'k k'"), "invalid-identifier"),
    ];
    for (name, second, code) in cases {
        let dir = scratch(name);
        let source = dir.join("protocol.yaml");
        std::fs::write(&source, protocol).expect("written");
        let compiled = canon(&["compile", "--path", path(&source)]);
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
            "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r}}\n",
        )
        .expect("written");
        let evidence = dir.join("evidence");
        std::fs::create_dir_all(&evidence).expect("created");
        std::fs::write(evidence.join("first.yaml"), record("e1", "k")).expect("written");
        let culprit = format!("second-{name}.yaml");
        std::fs::write(evidence.join(&culprit), &second).expect("written");

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
        assert_eq!(run.status.code(), Some(1), "{name}: {stderr}");
        assert!(
            stderr.contains(&format!("error[{code}]")),
            "{name}: refused as {code}: {stderr}"
        );
        assert!(
            stderr.contains(&culprit),
            "{name}: the refusal names the file `{culprit}` the refused record came from: {stderr}"
        );
    }
}
