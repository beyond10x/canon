//! Acceptance for story:canon-check: `canon check` enumerates a protocol's finite state space,
//! evaluates every state with Canon's own evaluator and reports what no single evaluation shows.
//!
//! Seven expectations, each over one fixture directory in `fixtures/investigation/check/`: the
//! base investigation protocol reports nothing; a variant with an outcome whose requirement
//! contradicts itself reports that outcome unreachable; a variant with an unsatisfiable
//! precondition reports that action; a variant with a claim over an evidence kind no action
//! produces reports that claim and kind; a variant whose outcome is reachable without any
//! authority-requiring action reports the bypass; a variant with a declared property that fails
//! reports it with a counterexample; and a protocol whose state space exceeds the bound is refused
//! naming the bound. Two runs give identical bytes.
//!
//! Each fixture directory carries `protocol.yaml`, a `properties.yaml` (`canon-properties/1`) when
//! the case declares properties, and the exact output `canon check` must give: `report.txt` on
//! standard output with nothing on standard error, or `refusal.txt` on standard error with nothing
//! on standard output.

use std::path::PathBuf;
use std::process::{Command, Output};

/// The fixture family, relative to the repository root.
const FIXTURES: &str = "fixtures/investigation/check";

/// Where `canon check` writes the expected text.
#[derive(Clone, Copy, Debug)]
enum Expected {
    /// `report.txt` on standard output; standard error empty.
    Report,
    /// `refusal.txt` on standard error; standard output empty.
    Refusal,
}

/// The seven expectations: fixture directory, whether it declares properties, the exit status and
/// where the expected text goes. A clean check exits 0; findings and a refusal exit 1 (`REJECTED`).
const CASES: [(&str, bool, i32, Expected); 7] = [
    ("base", true, 0, Expected::Report),
    ("unreachable-outcome", false, 1, Expected::Report),
    ("unsatisfiable-precondition", false, 1, Expected::Report),
    ("unproduced-evidence", false, 1, Expected::Report),
    ("authority-bypass", false, 1, Expected::Report),
    ("failing-property", true, 1, Expected::Report),
    ("state-space-bound", false, 1, Expected::Refusal),
];

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
        .expect("the canon binary runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn read(path: &str) -> String {
    std::fs::read_to_string(repository_root().join(path))
        .unwrap_or_else(|error| panic!("{path} is readable: {error}"))
}

/// Runs `canon check` over one fixture directory.
fn check(case: &str, properties: bool) -> Output {
    let protocol = format!("{FIXTURES}/{case}/protocol.yaml");
    let declared = format!("{FIXTURES}/{case}/properties.yaml");
    let mut args = vec!["check", "--path", protocol.as_str()];
    if properties {
        args.extend(["--properties", declared.as_str()]);
    }
    canon(&args)
}

#[test]
fn canon_check_reports_investigation_defects() {
    let mut failures = Vec::new();
    for (case, properties, status, expected) in CASES {
        let first = check(case, properties);
        let (stdout, stderr) = match expected {
            Expected::Report => (
                read(&format!("{FIXTURES}/{case}/report.txt")),
                String::new(),
            ),
            Expected::Refusal => (
                String::new(),
                read(&format!("{FIXTURES}/{case}/refusal.txt")),
            ),
        };
        if first.status.code() != Some(status)
            || text(&first.stdout) != stdout
            || text(&first.stderr) != stderr
        {
            failures.push(format!(
                "{case}: expected exit {status}, stdout {stdout:?}, stderr {stderr:?}; \
                 got exit {:?}, stdout {:?}, stderr {:?}",
                first.status.code(),
                text(&first.stdout),
                text(&first.stderr),
            ));
            continue;
        }
        let second = check(case, properties);
        if (second.status.code(), &second.stdout, &second.stderr)
            != (first.status.code(), &first.stdout, &first.stderr)
        {
            failures.push(format!("{case}: a second run gave different bytes"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} expectations failed:\n{}",
        failures.len(),
        CASES.len(),
        failures.join("\n")
    );
}

/// The red run must not come from a fixture typo: every fixture protocol is one `canon validate`
/// accepts, so `canon check` is the only thing the acceptance waits on.
#[test]
fn every_check_fixture_protocol_validates() {
    for (case, ..) in CASES {
        let output = canon(&[
            "validate",
            "--path",
            &format!("{FIXTURES}/{case}/protocol.yaml"),
        ]);
        assert!(output.status.success(), "{case}: {}", text(&output.stderr));
    }
}

/// Every fixture directory is one of the seven expectations, so none is skipped silently, and each
/// carries exactly the files its expectation reads.
#[test]
fn every_check_fixture_is_an_expectation() {
    let mut found: Vec<String> = std::fs::read_dir(repository_root().join(FIXTURES))
        .expect("the fixture family is readable")
        .map(|entry| {
            entry
                .expect("a fixture entry is readable")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    found.sort();
    let mut named: Vec<String> = CASES.iter().map(|(case, ..)| (*case).to_owned()).collect();
    named.sort();
    assert_eq!(found, named);
    for (case, properties, _, expected) in CASES {
        let mut files: Vec<String> = std::fs::read_dir(repository_root().join(FIXTURES).join(case))
            .expect("a fixture directory is readable")
            .map(|entry| {
                entry
                    .expect("a fixture file is readable")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        files.sort();
        let mut wanted = vec!["protocol.yaml".to_owned()];
        if properties {
            wanted.push("properties.yaml".to_owned());
        }
        wanted.push(
            match expected {
                Expected::Report => "report.txt",
                Expected::Refusal => "refusal.txt",
            }
            .to_owned(),
        );
        wanted.sort();
        assert_eq!(files, wanted, "{case}");
    }
}
