//! story:ordering-guards-provable-by-check against the command line: the optional action key
//! `runs_only_when` is an assertion that only `canon check` reads (its findings are pinned in
//! `canon_check.rs`, fixtures `ordering-guard-broken` and `ordering-guard-kept`).
//!
//! - `canon compile` writes `"runs_only_when"` only when the action declares it, normalized like
//!   every predicate; without it the IR is the bytes the protocol had before the key existed. The
//!   IR with it reads back as `canon evaluate --ir` reads it.
//! - `canon evaluate`, `canon simulate` and `canon generate` give the same bytes with and without
//!   the guard.
//! - `canon diff` reports the guard introduced, removed or changed as `NO SEMANTIC CHANGE`.
//! - `canon validate` refuses a guard that names an undeclared claim (`undeclared-claim`, naming
//!   `action `a` runs_only_when`) and the guard `all: []` (`vacuous-runs-only-when`).
//! - The protocol writer writes the key after `precondition`, and the text reads back as the model.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use b10x_canon::model;

/// The broken fixture, relative to the repository root.
const BROKEN: &str = "fixtures/investigation/check/ordering-guard-broken/protocol.yaml";

/// The guard as the fixture declares it.
const GUARD: &str = "    runs_only_when:\n      claim: explanation.survived\n";

/// The guard as `canon compile` writes it, with the comma that ends the key before it.
const COMPILED_GUARD: &str = ",\n      \"runs_only_when\": {\n        \"claim\": {\n          \
     \"id\": \"explanation.survived\",\n          \"is\": \"true\"\n        }\n      }";

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
        .join("ordering-guards")
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

/// The broken fixture's text, and the same text without its guard.
fn sources() -> (String, String) {
    let guarded = std::fs::read_to_string(repository_root().join(BROKEN)).expect("readable");
    assert_eq!(guarded.matches(GUARD).count(), 1);
    let unguarded = guarded.replace(GUARD, "");
    (guarded, unguarded)
}

/// `source` written as `protocol.yaml` in `dir`, and its path.
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

/// The IR holds the guard only when declared, normalized; without it, it is the IR with the guard
/// less its key. Both read back as `canon evaluate --ir` reads an IR, to the same decision.
#[test]
fn the_ir_writes_runs_only_when_only_when_declared() {
    let dir = scratch("ir");
    let (guarded, unguarded) = sources();
    let (with, with_at) = compiled(&dir, &written(&dir, "with.yaml", &guarded), "with");
    let (without, without_at) =
        compiled(&dir, &written(&dir, "without.yaml", &unguarded), "without");
    assert_eq!(with.matches(COMPILED_GUARD).count(), 1, "{with}");
    assert!(!without.contains("runs_only_when"), "{without}");
    assert_eq!(with.replace(COMPILED_GUARD, ""), without);

    let spelled = guarded.replace(
        GUARD,
        "    runs_only_when: {any: [{claim: explanation.survived, is: true}]}\n",
    );
    let (any, _) = compiled(&dir, &written(&dir, "any.yaml", &spelled), "any");
    assert!(
        any.contains(
            "\"runs_only_when\": {\n        \"any\": [\n          {\n            \"claim\""
        ),
        "{any}"
    );
    let (sugar, _) = compiled(
        &dir,
        &written(
            &dir,
            "sugar.yaml",
            &guarded.replace(
                GUARD,
                "    runs_only_when: {claim: explanation.survived, is: true}\n",
            ),
        ),
        "sugar",
    );
    assert_eq!(sugar, with);

    let case = written(
        &dir,
        "case.yaml",
        "format: canon-case/1\nid: C-1\nprotocol: investigation\n\
         artifacts: {explanation: {revision: r1}}\n",
    );
    let evidence = dir.join("evidence");
    std::fs::create_dir_all(&evidence).expect("evidence directory");
    written(
        &evidence,
        "attempt.yaml",
        "format: canon-evidence/1\nid: e1\nkind: falsification_attempt\nresult: refuted\n\
         subject: explanation\nsubject_revision: r1\n",
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

/// `canon simulate` and `canon generate` give the same bytes with and without the guard, each run
/// on its protocol at the same relative path, since a scenario names that path.
#[test]
fn simulate_and_generate_read_nothing_of_a_guard() {
    let (guarded, unguarded) = sources();
    let run = |name: &str, source: &str| {
        let dir = scratch(name);
        written(&dir, "protocol.yaml", source);
        let simulated = canon_in(&dir, &["simulate", "--path", "protocol.yaml"]);
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
            simulated.status.code(),
            text(&simulated.stdout),
            text(&simulated.stderr),
            files,
        )
    };
    let without = run("generate-without", &unguarded);
    assert!(!without.3.is_empty());
    assert_eq!(run("generate-with", &guarded), without);
}

/// Adding, removing or changing the guard is `NO SEMANTIC CHANGE`, naming the action.
#[test]
fn diff_reports_a_guard_as_no_semantic_change() {
    let dir = scratch("diff");
    let (guarded, unguarded) = sources();
    let changed = guarded.replace(
        GUARD,
        "    runs_only_when:\n      evidence: {kind: falsification_attempt, result: survived}\n",
    );
    let (_, with) = compiled(&dir, &written(&dir, "with.yaml", &guarded), "with");
    let (_, without) = compiled(&dir, &written(&dir, "without.yaml", &unguarded), "without");
    let (_, other) = compiled(&dir, &written(&dir, "other.yaml", &changed), "other");
    let diff = |from: &Path, to: &Path| {
        succeeded(&canon_in(
            &dir,
            &["diff", "--from", path(from), "--to", path(to)],
        ))
    };
    let head = "diff: protocol `investigation` revision 1 to 1\n";
    assert_eq!(
        diff(&without, &with),
        format!("{head}NO SEMANTIC CHANGE: action `publish_finding` runs_only_when introduced\n")
    );
    assert_eq!(
        diff(&with, &without),
        format!("{head}NO SEMANTIC CHANGE: action `publish_finding` runs_only_when removed\n")
    );
    assert_eq!(
        diff(&with, &other),
        format!("{head}NO SEMANTIC CHANGE: action `publish_finding` runs_only_when changed\n")
    );
}

/// The guard's references are checked as the precondition's are, naming the guard; `all: []`,
/// true in every state, is refused.
#[test]
fn validate_refuses_an_undeclared_claim_and_an_empty_all_in_a_guard() {
    let dir = scratch("validate");
    let (guarded, _) = sources();
    for (name, guard, refusal) in [
        (
            "undeclared.yaml",
            "    runs_only_when: {claim: nope}\n",
            "error[undeclared-claim]: action `publish_finding` runs_only_when references claim \
             `nope`, which is not declared\n",
        ),
        (
            "empty.yaml",
            "    runs_only_when: {all: []}\n",
            "error[vacuous-runs-only-when]: action `publish_finding` runs_only_when is `all: []`, \
             which is true in every state and asserts nothing\n",
        ),
    ] {
        let file = written(&dir, name, &guarded.replace(GUARD, guard));
        let output = canon_in(&dir, &["validate", "--path", path(&file)]);
        assert_eq!(
            (
                output.status.code(),
                text(&output.stdout),
                text(&output.stderr)
            ),
            (Some(1), String::new(), refusal.to_owned()),
            "{name}"
        );
    }
}

/// The protocol writer writes `runs_only_when` after `precondition` and before `requires`, and
/// the text reads back as the model; without the key it writes none.
#[test]
fn the_writer_writes_runs_only_when_after_the_precondition() {
    let (guarded, unguarded) = sources();
    let with = model::parse(&guarded).expect("parses");
    let written = model::write(&with);
    assert_eq!(model::parse(&written).expect("reads back"), with);
    let at = |key: &str| {
        written
            .find(&format!("\"{key}\": "))
            .unwrap_or_else(|| panic!("{key} in {written}"))
    };
    let publish = written
        .find("\"publish_finding\"")
        .expect("the action is written");
    assert!(
        publish < at("precondition")
            && at("precondition") < at("runs_only_when")
            && at("runs_only_when") < at("requires"),
        "{written}"
    );
    let without = model::parse(&unguarded).expect("parses");
    assert!(!model::write(&without).contains("runs_only_when"));
}
