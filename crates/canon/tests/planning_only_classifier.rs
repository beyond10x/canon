//! story:planning-only-checks: the changed-file classifier `scripts/classify-changes.sh` and the two
//! workflows that call it. One path list lives in the script; this test reads the script, never a
//! copy of the list.

use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("run this test through cargo"))
        .join("../..")
}

fn classify(files: &[&str]) -> String {
    let mut child = Command::new("sh")
        .arg(root().join("scripts/classify-changes.sh"))
        .current_dir(root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("sh runs scripts/classify-changes.sh");
    let mut input = files.join("\n");
    input.push('\n');
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(input.as_bytes())
        .expect("write the list");
    let out = child.wait_with_output().expect("the script ends");
    assert!(out.status.success(), "the classifier exits 0 on {files:?}");
    String::from_utf8(out.stdout).expect("utf-8").trim().to_owned()
}

#[test]
fn planning_only_list() {
    assert_eq!(
        classify(&["docs/plan/wave-2026-11-20-w10.md", "README.md", "CHANGELOG.md"]),
        "planning-only"
    );
    assert_eq!(
        classify(&[
            ".engineering/planning/epic/x.md",
            ".engineering/evidence/story/y.yaml",
            "docs/plan/y.md"
        ]),
        "planning-only"
    );
    assert_eq!(classify(&["CLAUDE.md"]), "planning-only");
}

#[test]
fn one_code_file_among_planning_files_is_code() {
    assert_eq!(
        classify(&["docs/plan/a.md", "crates/canon/src/lib.rs", ".engineering/project.yaml"]),
        "code"
    );
}

#[test]
fn website_list_is_docs_site_only() {
    assert_eq!(
        classify(&["website/docs/concepts/protocols.md", "website/status.yaml"]),
        "docs-site-only"
    );
    assert_eq!(classify(&["website/data/status.json"]), "docs-site-only");
    assert_eq!(
        classify(&["website/docs/reference/cli.md", "docs/plan/a.md"]),
        "docs-site-only"
    );
}

#[test]
fn empty_list_is_code() {
    assert_eq!(classify(&[]), "code");
}

#[test]
fn rename_from_code_to_planning_is_code() {
    // `git diff --no-renames --name-only` lists both paths of a rename.
    assert_eq!(classify(&["crates/a.rs", "docs/a.md"]), "code");
    assert_eq!(classify(&["docs/a.md", "crates/a.rs"]), "code");
}

#[test]
fn agents_md_is_code() {
    assert_eq!(classify(&["AGENTS.md"]), "code");
    assert_eq!(classify(&["AGENTS.md", "docs/plan/a.md"]), "code");
}

#[test]
fn story_files_are_docs_site_only() {
    assert_eq!(
        classify(&[".engineering/planning/story/x.md"]),
        "docs-site-only"
    );
    assert_eq!(
        classify(&[".engineering/planning/story/x.md", "docs/plan/y.md"]),
        "docs-site-only"
    );
}

#[test]
fn other_files_are_code() {
    for path in [
        "Taskfile.yml",
        ".github/workflows/check.yml",
        "scripts/classify-changes.sh",
        "ess/domains/cli.yaml",
        "crates/canon/src/lib.rs",
        "conformance/requirements.yaml",
        "LICENSE",
        "crates/canon/README.md",
        "docs",
        "website",
    ] {
        assert_eq!(classify(&[path]), "code", "{path}");
    }
}

#[test]
fn a_diff_that_cannot_be_computed_is_code() {
    let out = Command::new("sh")
        .arg(root().join("scripts/classify-changes.sh"))
        .args(["--diff", "no-such-ref-for-the-classifier-test"])
        .current_dir(root())
        .stdin(Stdio::null())
        .output()
        .expect("sh runs the script");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "code");
}

#[test]
fn both_workflows_call_the_one_script() {
    for name in ["check.yml", "pages.yml"] {
        let text = fs::read_to_string(root().join(".github/workflows").join(name))
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        assert!(
            text.contains("scripts/classify-changes.sh"),
            "{name} runs scripts/classify-changes.sh"
        );
        assert!(
            text.contains("fetch-depth: 0"),
            "{name} fetches the base with fetch-depth: 0"
        );
        assert!(
            text.contains("steps.classify.outputs.class"),
            "{name} conditions its steps on the classifier's answer"
        );
        assert!(
            !text.contains(".engineering") && !text.contains("README") && !text.contains("docs/plan"),
            "{name} holds no path list of its own"
        );
        assert!(
            !text.lines().any(|line| line.trim_start().starts_with("paths")),
            "{name}: no `paths:` filter on a trigger"
        );
    }
}

#[test]
fn job_names_stay() {
    let check = fs::read_to_string(root().join(".github/workflows/check.yml")).expect("check.yml");
    assert!(check.contains("name: canon / task check"));
    let pages = fs::read_to_string(root().join(".github/workflows/pages.yml")).expect("pages.yml");
    assert!(pages.contains("name: Build documentation"));
}
