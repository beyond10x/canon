//! `canon evaluate` refuses an evidence file it cannot read as a record naming the file, the
//! record and the key left out (story:review-hardening-w7).

use std::path::{Path, PathBuf};
use std::process::Command;

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("evidence-file-named-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("old scratch directory removed");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory created");
    dir
}

fn canon(args: &[&Path]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .args(args)
        .output()
        .expect("canon runs")
}

/// The protocol, case and IR of these cases, compiled into `dir`; returns the IR and case paths.
fn inputs(dir: &Path) -> (PathBuf, PathBuf) {
    let protocol = dir.join("protocol.yaml");
    std::fs::write(
        &protocol,
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {}}\nclaims: {c: {true_when: {evidence: {kind: k}}}}\n",
    )
    .expect("written");
    let compiled = canon(&[Path::new("compile"), Path::new("--path"), &protocol]);
    assert_eq!(compiled.status.code(), Some(0));
    let ir = dir.join("ir.json");
    std::fs::write(&ir, &compiled.stdout).expect("written");
    let case = dir.join("case.yaml");
    std::fs::write(
        &case,
        "format: canon-case/1\nid: C\nprotocol: p\nartifacts: {a: {revision: r1}}\n",
    )
    .expect("written");
    (ir, case)
}

/// A record refused after reading names every file holding a record of its id: an id repeated with
/// different content names both files, in file-name order; a record about an undeclared artifact
/// names its own.
#[test]
fn a_refusal_after_reading_names_the_file_of_each_record_it_cites() {
    for (name, second, expected) in [
        (
            "repeated",
            "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r2\n",
            "error[duplicate-identifier]: {first}, {second}: evidence `e1` is given more than once \
             with different content",
        ),
        (
            "undeclared-subject",
            "format: canon-evidence/1\nid: e2\nkind: k\nsubject: z\nsubject_revision: r1\n",
            "error[undeclared-artifact]: {second}: evidence `e2` is about artifact `z`, which the \
             protocol does not declare",
        ),
    ] {
        let dir = scratch(name);
        let (ir, case) = inputs(&dir);
        let evidence = dir.join("evidence");
        std::fs::create_dir_all(&evidence).expect("created");
        let first = evidence.join("a.yaml");
        std::fs::write(
            &first,
            "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r1\n",
        )
        .expect("written");
        let second_path = evidence.join("b.yaml");
        std::fs::write(&second_path, second).expect("written");
        let run = canon(&[
            Path::new("evaluate"),
            Path::new("--ir"),
            &ir,
            Path::new("--case"),
            &case,
            Path::new("--evidence"),
            &evidence,
        ]);
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert_eq!(run.status.code(), Some(1), "{name}: {stderr}");
        assert_eq!(
            stderr.trim_end(),
            expected
                .replace("{first}", &first.display().to_string())
                .replace("{second}", &second_path.display().to_string()),
            "{name}"
        );
        std::fs::remove_dir_all(&dir).expect("scratch removed");
    }
}

#[test]
fn a_malformed_evidence_file_is_refused_naming_the_file_the_record_and_the_key() {
    let dir = scratch("missing-subject");
    let protocol = dir.join("protocol.yaml");
    std::fs::write(
        &protocol,
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {k: {}}\nclaims: {c: {true_when: {evidence: {kind: k}}}}\n",
    )
    .expect("written");
    let compiled = canon(&[Path::new("compile"), Path::new("--path"), &protocol]);
    assert_eq!(compiled.status.code(), Some(0));
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
    std::fs::write(
        evidence.join("a-ok.yaml"),
        "format: canon-evidence/1\nid: e-a\nkind: k\nsubject: a\nsubject_revision: r1\n",
    )
    .expect("written");
    std::fs::write(
        evidence.join("b-broken.yaml"),
        "format: canon-evidence/1\nid: e-b\nkind: k\nsubject_revision: r1\n",
    )
    .expect("written");
    let run = canon(&[
        Path::new("evaluate"),
        Path::new("--ir"),
        &ir,
        Path::new("--case"),
        &case,
        Path::new("--evidence"),
        &evidence,
    ]);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert_eq!(run.status.code(), Some(1), "{stderr}");
    let file = evidence.join("b-broken.yaml").display().to_string();
    assert_eq!(
        stderr.trim_end(),
        format!(
            "error[malformed-input]: {file}: evidence `e-b` is not a canon-evidence/1 document: \
             missing field `subject`"
        )
    );
}

/// Two evidence files holding equal records of one id are one record: the evaluation exits 0 with
/// the decision the one file gives, byte for byte. Two holding records that differ exit 1 naming
/// both files (story:evidence-redelivery).
#[test]
fn equal_records_in_two_files_are_one_record_and_differing_ones_are_refused() {
    let dir = scratch("redelivered");
    let (ir, case) = inputs(&dir);
    let record = "format: canon-evidence/1\nid: e1\nkind: k\nsubject: a\nsubject_revision: r1\n";
    let evaluated = |name: &str, files: &[(&str, &str)]| {
        let evidence = dir.join(name);
        std::fs::create_dir_all(&evidence).expect("created");
        for (file, text) in files {
            std::fs::write(evidence.join(file), text).expect("written");
        }
        canon(&[
            Path::new("evaluate"),
            Path::new("--ir"),
            &ir,
            Path::new("--case"),
            &case,
            Path::new("--evidence"),
            &evidence,
        ])
    };
    let one = evaluated("one", &[("a.yaml", record)]);
    assert_eq!(one.status.code(), Some(0));
    assert!(!one.stdout.is_empty());
    let two = evaluated("two", &[("a.yaml", record), ("b.yaml", record)]);
    assert_eq!(
        String::from_utf8_lossy(&two.stderr),
        "",
        "equal records in two files"
    );
    assert_eq!(two.status.code(), Some(0));
    assert_eq!(two.stdout, one.stdout);
    let differing = record.replace("r1", "r2");
    let refused = evaluated("differing", &[("a.yaml", record), ("b.yaml", &differing)]);
    assert_eq!(refused.status.code(), Some(1));
    let evidence = dir.join("differing");
    assert_eq!(
        String::from_utf8_lossy(&refused.stderr).trim_end(),
        format!(
            "error[duplicate-identifier]: {}, {}: evidence `e1` is given more than once with \
             different content",
            evidence.join("a.yaml").display(),
            evidence.join("b.yaml").display()
        )
    );
    std::fs::remove_dir_all(&dir).expect("scratch removed");
}
