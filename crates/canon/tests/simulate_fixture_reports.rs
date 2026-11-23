//! The `canon simulate` reports of the existing fixtures whose output
//! story:simulate-opens-child-with-its-record and story:refused-approval-is-not-stuck changed and
//! that no `report.txt` of a simulate test holds, byte for byte (found by the review of wave
//! 2026-11-23-w13). The expected text of each is `tests/simulate_fixture_reports/<name>.txt`, the
//! report the CLI prints, with counts below as before the two stories and after.
//!
//! story:refused-approval-is-not-stuck, option B (a state that ends by a decision after a denial
//! is no finding) drops findings from three: `decision-grounds.yaml` (2 findings to 1),
//! `pending-decisions.yaml` (4 to 2) and `check/authority-bypass/protocol.yaml` (2 to 1; its
//! `report.txt` is the `canon check` report). Option A (a printed finding with a denial names the
//! forms that end the case after it) adds the clause to four, whose counts stay:
//! `capability-scope.yaml`, `supplied-evidence/bypass.yaml`,
//! `supplied-evidence/bypass-unsupplied.yaml` and `record-group-large/chain`.
//!
//! story:simulate-opens-child-with-its-record reaches fewer states in six, with findings of the
//! same kinds and number: `child-outcome-adversary/either-once` and
//! `child-outcome-subject.yaml`, and, with option A's clause too, `child-outcome-adversary/ended-once`,
//! `case-composition.yaml`, `child-outcome-revision.yaml` and `evidence-redelivery.yaml`.

use b10x_canon::{ir, model, simulate};

/// Each fixture's report name, its path from the repository root, and its protocol source.
const FIXTURES: [(&str, &str, &str); 13] = [
    (
        "decision-grounds",
        "fixtures/investigation/decision-grounds.yaml",
        include_str!("../../../fixtures/investigation/decision-grounds.yaml"),
    ),
    (
        "pending-decisions",
        "fixtures/investigation/pending-decisions.yaml",
        include_str!("../../../fixtures/investigation/pending-decisions.yaml"),
    ),
    (
        "authority-bypass",
        "fixtures/investigation/check/authority-bypass/protocol.yaml",
        include_str!("../../../fixtures/investigation/check/authority-bypass/protocol.yaml"),
    ),
    (
        "capability-scope",
        "fixtures/investigation/capability-scope.yaml",
        include_str!("../../../fixtures/investigation/capability-scope.yaml"),
    ),
    (
        "supplied-evidence-bypass",
        "fixtures/investigation/supplied-evidence/bypass.yaml",
        include_str!("../../../fixtures/investigation/supplied-evidence/bypass.yaml"),
    ),
    (
        "supplied-evidence-bypass-unsupplied",
        "fixtures/investigation/supplied-evidence/bypass-unsupplied.yaml",
        include_str!("../../../fixtures/investigation/supplied-evidence/bypass-unsupplied.yaml"),
    ),
    (
        "record-group-large-chain",
        "fixtures/record-group-large/chain/protocol.yaml",
        include_str!("../../../fixtures/record-group-large/chain/protocol.yaml"),
    ),
    (
        "child-outcome-adversary-either-once",
        "fixtures/child-outcome-adversary/either-once/protocol.yaml",
        include_str!("../../../fixtures/child-outcome-adversary/either-once/protocol.yaml"),
    ),
    (
        "child-outcome-adversary-ended-once",
        "fixtures/child-outcome-adversary/ended-once/protocol.yaml",
        include_str!("../../../fixtures/child-outcome-adversary/ended-once/protocol.yaml"),
    ),
    (
        "case-composition",
        "fixtures/investigation/case-composition.yaml",
        include_str!("../../../fixtures/investigation/case-composition.yaml"),
    ),
    (
        "child-outcome-revision",
        "fixtures/investigation/child-outcome-revision.yaml",
        include_str!("../../../fixtures/investigation/child-outcome-revision.yaml"),
    ),
    (
        "child-outcome-subject",
        "fixtures/investigation/child-outcome-subject.yaml",
        include_str!("../../../fixtures/investigation/child-outcome-subject.yaml"),
    ),
    (
        "evidence-redelivery",
        "fixtures/investigation/evidence-redelivery.yaml",
        include_str!("../../../fixtures/investigation/evidence-redelivery.yaml"),
    ),
];

/// The report `canon simulate` prints for `source`, composed with no imports as the CLI does.
fn simulated(source: &str) -> String {
    let protocol = model::parse(source).expect("parses");
    let composed = ir::compose(&protocol, &[]).expect("composes");
    simulate::simulate(&ir::compile(&composed).expect("compiles"))
        .expect("walked")
        .to_string()
}

fn reports() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/simulate_fixture_reports")
}

fn expected(name: &str) -> String {
    std::fs::read_to_string(reports().join(format!("{name}.txt")))
        .expect("the expected report is readable")
}

fn summary(name: &str) -> String {
    expected(name)
        .lines()
        .find(|line| line.starts_with("simulated: "))
        .expect("a summary line")
        .to_owned()
}

/// Each report, byte for byte.
#[test]
fn each_report_the_two_stories_changed_is_held() {
    let mut failures = Vec::new();
    for (name, path, source) in FIXTURES {
        let report = simulated(source);
        if report != expected(name) {
            failures.push(format!("{path}: got\n{report}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The lines the release notes name: the three that end by a decision after a denial, with
/// their finding counts, and those that carry option A's clause, with their counts.
#[test]
fn the_changed_lines_are_the_ones_named() {
    for (name, ending) in [
        (
            "decision-grounds",
            ", 1 ending by a decision after a denial, 1 finding (1 stuck or cycle state)",
        ),
        (
            "pending-decisions",
            ", 12 ending by a decision after a denial, 2 findings (3 stuck or cycle states)",
        ),
        (
            "authority-bypass",
            ", 3 ending by a decision after a denial, 1 finding (3 stuck or cycle states)",
        ),
    ] {
        assert!(summary(name).ends_with(ending), "{name}: {}", summary(name));
    }
    let clause = "to end the case after the denial, declare an outcome with `requires: {decision: \
                  ..., when: ...}` or an action without capability";
    for (name, clauses, counts) in [
        (
            "capability-scope",
            1,
            ", 2 findings (2 stuck or cycle states)",
        ),
        (
            "supplied-evidence-bypass",
            1,
            ", 2 findings (8 stuck or cycle states)",
        ),
        (
            "supplied-evidence-bypass-unsupplied",
            1,
            ", 4 findings (2 stuck or cycle states)",
        ),
        (
            "record-group-large-chain",
            4,
            ", 35 findings (89 stuck or cycle states)",
        ),
        (
            "child-outcome-adversary-ended-once",
            1,
            ", 1 finding (1 stuck or cycle state)",
        ),
        (
            "case-composition",
            1,
            ", 2 findings (5 stuck or cycle states)",
        ),
        (
            "child-outcome-revision",
            1,
            ", 2 findings (3 stuck or cycle states)",
        ),
        (
            "evidence-redelivery",
            1,
            ", 2 findings (5 stuck or cycle states)",
        ),
    ] {
        let report = expected(name);
        assert_eq!(
            report.lines().filter(|line| line.contains(clause)).count(),
            clauses,
            "{name}: {report}"
        );
        assert!(!report.contains("ending by a decision"), "{name}: {report}");
        assert!(summary(name).ends_with(counts), "{name}: {}", summary(name));
    }
}

/// Every expected report belongs to a fixture of the list, so none is held by nothing.
#[test]
fn every_expected_report_is_a_fixture_of_the_list() {
    let mut found: Vec<String> = std::fs::read_dir(reports())
        .expect("the reports directory is readable")
        .map(|entry| {
            entry
                .expect("an entry is readable")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    found.sort();
    let mut named: Vec<String> = FIXTURES
        .iter()
        .map(|(name, ..)| format!("{name}.txt"))
        .collect();
    named.sort();
    assert_eq!(found, named);
}
