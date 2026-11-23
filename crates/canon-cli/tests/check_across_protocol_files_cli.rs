//! Acceptance for story:check-across-protocol-files against the `canon` command line:
//! `canon check-set --path <file> ...` checks a set of `protocol/1` documents against each other.
//!
//! Each directory of `fixtures/check-across-protocol-files/` is one set, every `*.yaml` file in it
//! a member, and `report.txt` the exact standard output, written from the story before the command
//! existed.
//!
//! 1. Each defect set exits 1 and prints exactly one finding line, with a stable code naming the
//!    protocol, the action or kind and the missing id: `missing-protocol` opens a protocol not in
//!    the set, `undeclared-children` lists a child outcome the child lacks in a `children` form,
//!    `undeclared-evidence` reads one in an evidence match, `missing-supplier` names a supplier not
//!    in the set and `unproduced-supplied` one no action of which may produce the kind.
//! 2. `clean` exits 0 with no finding: it names `open`, a form without a subject and a supplied
//!    kind its supplier produces. `other-subject` names an outcome about `work` that only the child
//!    opened about `notes` declares, and that is a finding.
//! 3. `cycle` prints `case-open-cycle`. `duplicate-id` holds two files with the protocol id
//!    `change` and prints `duplicate-protocol-id`, naming both paths, instead of pooling them.
//! 4. A member with `imports:` is refused, naming it (`protocol-imports`), and an unreadable file
//!    exits 2; the order of the `--path` flags changes no byte.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The fixture family, relative to the repository root.
const FIXTURES: &str = "fixtures/check-across-protocol-files";

/// Every set with a `report.txt`, the exit status `canon check-set` gives it and the code of its
/// one finding, `None` for a clean set.
const SETS: [(&str, i32, Option<&str>); 9] = [
    ("clean", 0, None),
    ("missing-protocol", 1, Some("unknown-opened-protocol")),
    ("undeclared-children", 1, Some("undeclared-child-outcome")),
    ("undeclared-evidence", 1, Some("undeclared-child-outcome")),
    ("other-subject", 1, Some("undeclared-child-outcome")),
    ("missing-supplier", 1, Some("unknown-supplier")),
    ("unproduced-supplied", 1, Some("unproduced-supplied-kind")),
    ("cycle", 1, Some("case-open-cycle")),
    ("duplicate-id", 1, Some("duplicate-protocol-id")),
];

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn canon(dir: &Path, args: &[String]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("the canon binary runs")
}

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("utf-8 output")
}

/// The members of a set, as paths relative to the repository root, sorted.
fn members(root: &Path, set: &str) -> Vec<String> {
    let dir = root.join(FIXTURES).join(set);
    let mut found: Vec<String> = std::fs::read_dir(&dir)
        .expect("the set directory is read")
        .map(|entry| {
            entry
                .expect("an entry")
                .file_name()
                .into_string()
                .expect("utf-8")
        })
        .filter(|name| name.ends_with(".yaml"))
        .map(|name| format!("{FIXTURES}/{set}/{name}"))
        .collect();
    found.sort();
    found
}

fn check_set(root: &Path, paths: &[String]) -> Output {
    let mut args = vec!["check-set".to_owned()];
    for path in paths {
        args.push("--path".to_owned());
        args.push(path.clone());
    }
    canon(root, &args)
}

#[test]
fn every_set_prints_its_report_and_exits_with_its_status() {
    let root = repository_root();
    for (set, status, code) in SETS {
        let output = check_set(&root, &members(&root, set));
        let expected = std::fs::read_to_string(root.join(FIXTURES).join(set).join("report.txt"))
            .expect("the report is read");
        assert_eq!(text(&output.stderr), "", "{set}: standard error");
        assert_eq!(text(&output.stdout), expected, "{set}: standard output");
        assert_eq!(output.status.code(), Some(status), "{set}: exit status");
        let lines: Vec<&str> = expected.lines().collect();
        match code {
            None => assert_eq!(lines.len(), 1, "{set}: only the summary line"),
            Some(code) => {
                assert_eq!(lines.len(), 2, "{set}: one finding and the summary line");
                assert!(
                    lines[0].starts_with(&format!("{code}: ")),
                    "{set}: the finding has the code {code}: {}",
                    lines[0]
                );
            }
        }
    }
}

#[test]
fn the_order_of_the_paths_changes_no_byte() {
    let root = repository_root();
    for (set, _, _) in SETS {
        let sorted = members(&root, set);
        let mut reversed = sorted.clone();
        reversed.reverse();
        let first = check_set(&root, &sorted);
        let second = check_set(&root, &reversed);
        assert_eq!(first.stdout, second.stdout, "{set}");
        assert_eq!(first.status.code(), second.status.code(), "{set}");
    }
}

#[test]
fn a_member_with_imports_is_refused_naming_it() {
    let root = repository_root();
    let output = check_set(&root, &members(&root, "imports"));
    let expected = std::fs::read_to_string(root.join(FIXTURES).join("imports/stderr.txt"))
        .expect("the expected error is read");
    assert_eq!(text(&output.stderr), expected);
    assert_eq!(text(&output.stdout), "");
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn an_unreadable_member_exits_2() {
    let root = repository_root();
    let mut paths = members(&root, "clean");
    paths.push(format!("{FIXTURES}/clean/absent.yaml"));
    let output = check_set(&root, &paths);
    let stderr = text(&output.stderr);
    assert!(
        stderr.starts_with(&format!(
            "error[unreadable]: {FIXTURES}/clean/absent.yaml: "
        )),
        "{stderr}"
    );
    assert_eq!(text(&output.stdout), "");
    assert_eq!(output.status.code(), Some(2));
}
