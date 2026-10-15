//! Adversary cases for story:conformance-suite through the `canon` binary: exit statuses of
//! `canon conform run --requirements`, the report without the flag, and the real catalogue.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn canon(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(dir)
        .args(["conform", "run"])
        .args(args)
        .output()
        .expect("canon runs")
}

fn out(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("utf-8")
}

fn err(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("utf-8")
}

/// A fresh directory under this test target's own scratch area (inside the tree's `target/`).
fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("adversary_conformance_suite").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

#[test]
fn a_missing_catalogue_exits_2_with_nothing_on_stdout() {
    let root = repository_root();
    let output = canon(&root, &["--requirements", "does/not/exist.yaml"]);
    assert_eq!(output.status.code(), Some(2), "{}", err(&output));
    assert_eq!(out(&output), "");
    assert!(
        err(&output).starts_with("error[unreadable]: does/not/exist.yaml: "),
        "{}",
        err(&output)
    );
}

#[test]
fn a_directory_given_as_the_catalogue_exits_2() {
    let root = repository_root();
    let output = canon(&root, &["--requirements", "conformance"]);
    assert_eq!(output.status.code(), Some(2), "{}", err(&output));
    assert_eq!(out(&output), "");
}

#[test]
fn a_file_that_is_not_a_catalogue_exits_1_by_kind_and_position() {
    let dir = scratch("not_a_catalogue");
    std::fs::write(dir.join("c.yaml"), "format: canon-requirements/1\nrequirements: {}\nSECRET: 1\n")
        .unwrap();
    std::fs::create_dir_all(dir.join("s")).unwrap();
    let output = canon(&dir, &["--scenarios", "s", "--requirements", "c.yaml"]);
    assert_eq!(output.status.code(), Some(1), "{}", err(&output));
    assert_eq!(out(&output), "");
    assert!(err(&output).starts_with("error[catalogue]: c.yaml: "), "{}", err(&output));
    assert!(!err(&output).contains("SECRET"), "{}", err(&output));
}

/// With the flag over the real registry, the report is the plain report and one more line,
/// byte for byte.
#[test]
fn the_real_registry_with_the_flag_is_the_plain_report_and_one_line() {
    let root = repository_root();
    let plain = canon(&root, &[]);
    assert_eq!(plain.status.code(), Some(0), "{}", out(&plain));
    let checked = canon(&root, &["--requirements", "conformance/requirements.yaml"]);
    assert_eq!(checked.status.code(), Some(0), "{}", out(&checked));
    assert_eq!(
        out(&checked),
        format!("{}requirements: 15 implemented, 15 covered, 0 planned\n", out(&plain))
    );
    assert_eq!(err(&checked), "");
}

/// Without the flag, the determinism check does not run: the unit's own nondeterministic fixture
/// registry is the runner's report alone and passes.
#[test]
fn without_the_flag_no_check_runs() {
    let root = repository_root();
    let output = canon(
        &root,
        &["--scenarios", "fixtures/conformance/requirements/nondeterministic"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", out(&output));
    assert_eq!(
        out(&output),
        "passed: scenario `FIXTURE-NONDETERMINISTIC`\nconform: 1 passed, 0 failed, 0 unreadable\n"
    );
}

/// The catalogue lists exactly the 15 requirements the story names, all implemented, and every
/// one is covered by a scenario of the registry.
#[test]
fn the_real_catalogue_is_the_fifteen_the_story_names() {
    let root = repository_root();
    let catalogue: serde_yaml_ng::Value = serde_yaml_ng::from_str(
        &std::fs::read_to_string(root.join("conformance/requirements.yaml")).unwrap(),
    )
    .unwrap();
    let mut ids: Vec<String> = catalogue["requirements"]
        .as_mapping()
        .unwrap()
        .iter()
        .map(|(id, requirement)| {
            assert_eq!(requirement["status"].as_str(), Some("implemented"));
            id.as_str().unwrap().to_owned()
        })
        .collect();
    ids.sort();
    let mut story = vec![
        "CANON-CLAIM-001",
        "CANON-CLAIM-002",
        "CANON-EVIDENCE-001",
        "CANON-EVIDENCE-002",
        "CANON-EVIDENCE-003",
        "CANON-AUTHORITY-001",
        "CANON-INDEPENDENCE-001",
        "CANON-OUTCOME-001",
        "CANON-OUTCOME-002",
        "CANON-INVALIDATION-001",
        "CANON-DETERMINISM-001",
        "CANON-EXPLAIN-001",
        "CANON-OBLIGATION-001",
        "CANON-INPUT-001",
        "CANON-ORDER-001",
    ];
    story.sort();
    assert_eq!(ids, story);

    let mut covered = std::collections::BTreeSet::new();
    for entry in std::fs::read_dir(root.join("conformance/scenarios")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|ext| ext == "yaml") {
            let scenario: serde_yaml_ng::Value =
                serde_yaml_ng::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            for id in scenario["covers"].as_sequence().unwrap() {
                covered.insert(id.as_str().unwrap().to_owned());
            }
        }
    }
    let covered: Vec<String> = covered.into_iter().collect();
    assert_eq!(covered, ids);
}
