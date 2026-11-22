//! Acceptance for story:canon-check: `canon check` enumerates a protocol's finite state space,
//! evaluates every state with Canon's own evaluator and reports what no single evaluation shows.
//!
//! Nine expectations, each over one fixture directory in `fixtures/investigation/check/`: the
//! base investigation protocol reports nothing; a variant with an outcome whose requirement
//! contradicts itself reports that outcome unreachable; a variant with an unsatisfiable
//! precondition reports that action; a variant with a claim over an evidence kind no action
//! produces reports that claim and kind; a variant whose outcome is reachable without any
//! authority-requiring action reports the bypass; a variant with a declared property that fails
//! reports it with a counterexample; a protocol whose whole product exceeds the bound while every
//! subject's slice fits reports one finding of each code, exactly as an enumeration of the whole
//! product does (story:check-subject-slices); a protocol whose only large dimension no subject
//! reads is checked; and a protocol in which one subject's slice exceeds the bound is refused
//! naming that subject. Two runs give identical bytes.
//!
//! Each fixture directory carries `protocol.yaml`, a `properties.yaml` (`canon-properties/1`) when
//! the case declares properties, and the exact output `canon check` must give: `report.txt` on
//! standard output with nothing on standard error, or `refusal.txt` on standard error with nothing
//! on standard output.
//!
//! `sliced/report.txt` and `unread-dimension/report.txt` are what `canon check` printed at
//! 557e75b with `STATE_BOUND` raised to `1 << 24` locally, enumerating the whole product: the
//! sliced check must print those bytes.

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

/// The nine expectations: fixture directory, whether it declares properties, the exit status and
/// where the expected text goes. A clean check exits 0; findings and a refusal exit 1 (`REJECTED`).
const CASES: [(&str, bool, i32, Expected); 9] = [
    ("base", true, 0, Expected::Report),
    ("unreachable-outcome", false, 1, Expected::Report),
    ("unsatisfiable-precondition", false, 1, Expected::Report),
    ("unproduced-evidence", false, 1, Expected::Report),
    ("authority-bypass", false, 1, Expected::Report),
    ("failing-property", true, 1, Expected::Report),
    ("sliced", true, 1, Expected::Report),
    ("unread-dimension", false, 0, Expected::Report),
    ("state-space-bound", false, 1, Expected::Refusal),
];

/// The whole-product state count the `sliced` summary gives, above the bound of 65536.
const SLICED_STATES: &str = ": 196608 states (input `track` 2 values), 2 properties, ";

/// The `unread-dimension` summary: its whole product, 98304 states, is above the bound, and the
/// dimension that makes it so is read by no action and no outcome.
const UNREAD_SUMMARY: &str =
    "checked: protocol `investigation` revision 1: 98304 states, 0 properties, 0 findings\n";

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

/// The `sliced` summary gives the size of the whole product, 196608 states, above the bound, and
/// the `unread-dimension` summary its 98304 states.
#[test]
fn a_sliced_summary_gives_the_whole_product() {
    let sliced = check("sliced", true);
    let stdout = text(&sliced.stdout);
    let summary = stdout.lines().last().unwrap_or_default();
    assert!(
        summary.starts_with("checked: protocol `remedy` revision 1")
            && summary.contains(SLICED_STATES),
        "{stdout}{}",
        text(&sliced.stderr)
    );
    let unread = check("unread-dimension", false);
    assert_eq!(
        (
            unread.status.code(),
            text(&unread.stdout),
            text(&unread.stderr)
        ),
        (Some(0), UNREAD_SUMMARY.to_owned(), String::new())
    );
}

/// A slice above the bound refuses the whole check, naming one subject, the states its slice
/// holds and the dimensions of that slice, whose numbers multiply to that count.
#[test]
fn a_slice_above_the_bound_is_refused_naming_its_subject() {
    let refused = check("state-space-bound", false);
    let stderr = text(&refused.stderr);
    let message = stderr
        .strip_prefix("error[state-space-bound]: protocol `investigation` revision 1: ")
        .and_then(|rest| rest.strip_suffix('\n'))
        .unwrap_or_else(|| panic!("not a subject's bound refusal: {stderr:?}"));
    let (subject, rest) = message
        .split_once(" reads ")
        .unwrap_or_else(|| panic!("names no subject: {message}"));
    assert!(
        ["action `", "outcome `", "property `"]
            .iter()
            .any(|section| subject.starts_with(section)),
        "{subject}"
    );
    let (count, rest) = rest
        .split_once(" states, more than the bound of 65536: ")
        .expect("count");
    let count: u128 = count.parse().expect("a number of states");
    assert!(count > 65536, "{message}");
    let product: u128 = rest
        .split(", ")
        .map(|dimension| {
            let (_, values) = dimension
                .rsplit_once(' ')
                .expect("a dimension and its values");
            values.parse::<u128>().expect("a number of values")
        })
        .product();
    assert_eq!(product, count, "{message}");
}

/// `canon generate` enumerates the whole product, so it keeps the whole-product bound: over
/// `sliced`, which `canon check` decides, it is refused as `state-space-bound`, naming the whole
/// product, and writes nothing.
#[test]
fn generate_keeps_the_whole_product_bound() {
    let out = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("canon-check-sliced-generate");
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("a stale output directory is removable");
    }
    let protocol = format!("{FIXTURES}/sliced/protocol.yaml");
    let output = canon(&[
        "generate",
        "--path",
        protocol.as_str(),
        "--out",
        out.to_str().expect("UTF-8"),
    ]);
    assert_eq!(
        (
            output.status.code(),
            text(&output.stdout),
            text(&output.stderr)
        ),
        (
            Some(1),
            String::new(),
            "error[state-space-bound]: protocol `remedy` revision 1 has 196608 states, more than \
             the bound of 65536: evidence kind `approval` 2, evidence kind `audit_report` 2, \
             evidence kind `survey_reading` 256, evidence kind `trial_run` 16, capability \
             `release.approve` 3, decision `explicitly_abandoned` 2, input `track` 2\n"
                .to_owned()
        )
    );
    assert!(!out.exists(), "a refused generate wrote {}", out.display());
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

/// Every fixture directory is one of the nine expectations, so none is skipped silently, and each
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

/// `canon check --help` names the history modes a `descended` form and another order form read
/// one kind in (story:check-descended-two-result-history): records that name no item and keep
/// their instants, earlier records observed after the current ones, and earlier records of two
/// results observed at two instants. The help is wrapped, so words are compared, not lines.
#[test]
fn check_help_names_the_history_modes_of_descended() {
    let help = text(&canon(&["check", "--help"]).stdout)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for mode in [
        "records that name no item and keep their instants",
        "earlier records observed after the current ones",
        "earlier records of two results observed at two instants",
    ] {
        assert!(
            help.contains(mode),
            "`canon check --help` lacks `{mode}`:\n{help}"
        );
    }
}
