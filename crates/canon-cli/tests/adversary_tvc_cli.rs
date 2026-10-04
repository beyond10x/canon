//! Adversary cases for story:three-valued-claims, against `canon evaluate`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("adversary-tvc-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("old scratch directory removed");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory created");
    dir
}

const CASE: &str = "format: canon-case/1\nid: INV-18\nprotocol: investigation\nartifacts:\n  explanation: {revision: r1}\n";

fn record(id: &str, kind: &str, result: Option<&str>) -> String {
    let result = result.map_or(String::new(), |r| format!("result: {r}\n"));
    format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\n{result}subject: explanation\nsubject_revision: r1\n"
    )
}

fn path(p: &Path) -> String {
    p.to_str().expect("utf-8 path").to_owned()
}

/// Compiles `protocol`, writes the IR and the case, and returns `(ir, case, evidence dir)`.
fn inputs(dir: &Path, protocol: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let compiled = canon(&["compile", "--path", &path(protocol)]);
    assert_eq!(
        compiled.status.code(),
        Some(0),
        "the protocol compiles: {}",
        text(&compiled.stderr)
    );
    let ir = dir.join("protocol.ir.json");
    std::fs::write(&ir, &compiled.stdout).expect("IR written");
    let case = dir.join("case.yaml");
    std::fs::write(&case, CASE).expect("case written");
    let evidence = dir.join("evidence");
    std::fs::create_dir(&evidence).expect("evidence directory created");
    (ir, case, evidence)
}

fn evaluate(ir: &Path, case: &Path, evidence: &Path) -> Output {
    canon(&[
        "evaluate",
        "--ir",
        &path(ir),
        "--case",
        &path(case),
        "--evidence",
        &path(evidence),
    ])
}

/// The scenario's `conflicting` step, with the refuting record written as JSON (`eval::read`:
/// "JSON is YAML, so either may be written as JSON"; design § 29 names inputs `case.json`). The
/// refutation is in the evidence directory the operator named, and the claim must not come out
/// TRUE as though it were not there: either the record is read (UNKNOWN), or the file is refused.
#[test]
fn a_refutation_in_the_evidence_directory_is_never_silently_dropped() {
    let dir = scratch("json-evidence");
    let fixture = repository_root().join("fixtures/investigation/three-valued-claims.yaml");
    let (ir, case, evidence) = inputs(&dir, &fixture);
    std::fs::write(
        evidence.join("observation-1.yaml"),
        record("observation-1", "supporting_observation", None),
    )
    .expect("written");
    std::fs::write(
        evidence.join("falsification-1.yaml"),
        record("falsification-1", "falsification_attempt", Some("survived")),
    )
    .expect("written");
    std::fs::write(
        evidence.join("falsification-2.json"),
        "{\"format\": \"canon-evidence/1\", \"id\": \"falsification-2\", \"kind\": \"falsification_attempt\", \"result\": \"refuted\", \"subject\": \"explanation\", \"subject_revision\": \"r1\"}\n",
    )
    .expect("written");
    let run = evaluate(&ir, &case, &evidence);
    let stdout = text(&run.stdout);
    assert!(
        run.status.code() != Some(0) || !stdout.contains("\"value\": \"true\""),
        "exit {:?}: a refuting record in the evidence directory was ignored and the claim decided \
         TRUE:\n{stdout}",
        run.status.code()
    );
}

/// `canon evaluate --help`: `--ir` is "`canon-ir/1` exactly as `canon compile` prints it". A
/// protocol whose description holds a character YAML does not take raw (DEL, written in the source
/// as the escape `\x7f`; NEL, `\x85`) compiles, and its IR must then evaluate.
#[test]
fn what_canon_compile_prints_canon_evaluate_accepts() {
    for (name, escape) in [("del", "\\x7f"), ("nel", "\\x85")] {
        let dir = scratch(&format!("compile-roundtrip-{name}"));
        let protocol = dir.join("protocol.yaml");
        let source = std::fs::read_to_string(
            repository_root().join("fixtures/investigation/three-valued-claims.yaml"),
        )
        .expect("fixture reads")
        .replace(
            "description: The explanation under investigation.",
            &format!("description: \"The explanation{escape}under investigation.\""),
        );
        std::fs::write(&protocol, source).expect("protocol written");
        let (ir, case, evidence) = inputs(&dir, &protocol);
        let run = evaluate(&ir, &case, &evidence);
        assert_eq!(
            run.status.code(),
            Some(0),
            "{name}: the IR canon compile printed is refused: {}",
            text(&run.stderr)
        );
        assert!(
            text(&run.stdout).contains("\"value\": \"unknown\""),
            "{name}"
        );
    }
}

/// `canon evaluate`: "A file that cannot be read exits 2". The unit's own test covers a missing
/// evidence directory only; a missing IR or case file is the same, and stdout stays empty.
#[test]
fn a_missing_ir_or_case_file_exits_2() {
    let dir = scratch("missing-inputs");
    let fixture = repository_root().join("fixtures/investigation/three-valued-claims.yaml");
    let (ir, case, evidence) = inputs(&dir, &fixture);
    let gone = dir.join("no-such-file");
    for (name, run) in [
        ("ir", evaluate(&gone, &case, &evidence)),
        ("case", evaluate(&ir, &gone, &evidence)),
    ] {
        assert_eq!(run.status.code(), Some(2), "{name}: {}", text(&run.stderr));
        assert!(
            text(&run.stderr).starts_with("error[unreadable]: "),
            "{name}: {}",
            text(&run.stderr)
        );
        assert_eq!(text(&run.stdout), "", "{name}");
    }
}
