//! Adversary pass 2 for story:three-valued-claims, against `canon evaluate`: what `canon compile`
//! prints for a deep predicate, and the order in which evidence files are judged.

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
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("adversary2-tvc-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("old scratch directory removed");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory created");
    dir
}

fn path(p: &Path) -> String {
    p.to_str().expect("utf-8 path").to_owned()
}

const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n";

fn record(id: &str, kind: &str) -> String {
    format!(
        "format: canon-evidence/1\nid: {id}\nkind: {kind}\nresult: pass\nsubject: a\nsubject_revision: r1\n"
    )
}

/// Writes `source`, compiles it with `canon compile` (which must accept it), and returns the IR,
/// case and empty evidence directory paths.
fn inputs(dir: &Path, source: &str) -> (PathBuf, PathBuf, PathBuf) {
    let protocol = dir.join("protocol.yaml");
    std::fs::write(&protocol, source).expect("protocol written");
    let compiled = canon(&["compile", "--path", &path(&protocol)]);
    assert_eq!(
        compiled.status.code(),
        Some(0),
        "canon compile accepts the protocol: {}",
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

const HEADER: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\nevidence_kinds: {k: {}}\nclaims:\n";

/// `canon evaluate --help`: `--ir` is "`canon-ir/1` exactly as `canon compile` prints it". A claim
/// nested 123 `not` deep is accepted by `canon compile`; `canon evaluate` refuses the bytes it
/// printed as "not well-formed JSON". `canon conform run` compiles the fixture itself and never
/// reads IR text, so a scenario over the same protocol passes while the command it stands for
/// refuses.
#[test]
fn what_canon_compile_prints_for_a_deep_predicate_canon_evaluate_accepts() {
    let depth = 123;
    let dir = scratch("deep-not");
    let source = format!(
        "{HEADER}  c: {{true_when: {}{{evidence: {{kind: k, result: pass}}}}{}}}\n",
        "{not: ".repeat(depth),
        "}".repeat(depth)
    );
    let (ir, case, evidence) = inputs(&dir, &source);
    let run = evaluate(&ir, &case, &evidence);
    assert_eq!(
        run.status.code(),
        Some(0),
        "the IR canon compile printed is refused: {}",
        text(&run.stderr)
    );
    assert!(text(&run.stdout).contains("\"value\": \"unknown\""));
}

/// `canon evaluate`: records are "read in sorted file-name order", and the library refuses "the
/// first problem found ... each evidence record in the order given". With twenty-six records of
/// undeclared kinds, the one refused is the first by file name, whatever order the filesystem lists
/// them in. Nothing else in the suite gives two refusable records at once.
#[test]
fn the_first_refusable_record_by_file_name_is_the_one_named() {
    let dir = scratch("refusal-order");
    let (ir, case, evidence) = inputs(
        &dir,
        &format!("{HEADER}  c: {{true_when: {{evidence: {{kind: k, result: pass}}}}}}\n"),
    );
    for letter in ('a'..='z').rev() {
        std::fs::write(
            evidence.join(format!("{letter}.yaml")),
            record(&format!("e-{letter}"), &format!("undeclared-{letter}")),
        )
        .expect("written");
    }
    let run = evaluate(&ir, &case, &evidence);
    assert_eq!(run.status.code(), Some(1), "{}", text(&run.stderr));
    // Story review-hardening-w7: a refusal names the file of each record it cites.
    assert_eq!(
        text(&run.stderr),
        format!(
            "error[undeclared-evidence-kind]: {}: evidence `e-a` is of kind `undeclared-a`, which \
             the protocol does not declare\n",
            evidence.join("a.yaml").display()
        )
    );
}
