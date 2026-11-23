//! Acceptance for story:simulate-child-outcome-step: `canon simulate` lets a child case's
//! `case_outcome` record arrive at any step after the action that opened the child was taken,
//! and no longer prints the line naming the limit on child outcomes.
//!
//! Eight expectations, over the fixture directories in `fixtures/investigation/child-outcome/`
//! and `fixtures/child-outcome-large/`, each a `protocol.yaml` and the exact standard output,
//! `report.txt`, written by hand from the story before the step existed. The two large ones sit
//! outside `fixtures/investigation/`, whose every protocol `canon generate` is run over by
//! `adversary_scenario_generation.rs`: their spaces (4608 and 36864 states) add more than two
//! minutes to that test in a debug build.
//!
//! The figures are those of the story's rule as decided in review (D1: a mark counts the children
//! of an opening action that have not ended, none, one or two-or-more; D5: a state holding an
//! `open` record holds a mark above none), derived before the code by a walk in the unit's scratch
//! directory and checked against bounded real cases there.
//!
//! The figures below are those after story:simulate-opens-child-with-its-record, whose opening
//! step adds the child's `open` record: every state of an opened child with no record is gone.
//!
//! 1. `closed`, one child opened by an action whose guard closes once it was opened, exits 0 and
//!    prints only the summary line, with 1 state waiting on supplied evidence, a child outcome
//!    or expiry.
//! 2. `ends-open`, the same with only the accepted outcome: exactly one `stuck-state`, whose state
//!    holds `` evidence `case_outcome` `` without a result and no mark, since its only child
//!    ended; no printed state holds `` result `open` ``, since a child that ends replaces its open
//!    record.
//! 3. `repeat`, an opening action that records nothing: exit 0, 12 reachable states, 2 waiting.
//! 4. `denied`, an opening action under a capability: exactly one finding, the stuck state of the
//!    denial, and no path in which a child records its outcome before a step of `open`.
//! 5. `festival`, a parent case opening three kinds of child: its summary; no printed state holds
//!    the records of a state in which only a child's outcome was missing (an opening record and an
//!    accepted programme); the records of the stuck state with a programme acceptance without a
//!    result are printed only with the mark of `inquiry.open`, beside its child's `open`
//!    record.
//! 6. `event`, a parent case opening four kinds of child: its summary; the records of the state in
//!    which only a child's outcome was missing (an opening record, an accepted programme, a
//!    cleared inspection) are not printed; those of the stuck state with an inspection without a
//!    result are printed only with the mark of `script.open`, beside its child's `open`
//!    record.
//! 7. No output names the limit on child outcomes; the fixtures of `fixtures/investigation/simulate/`
//!    with no opening action give the bytes they give in `canon_simulate.rs`; two runs of each new
//!    fixture give identical bytes.
//! 8. No Python file is tracked, and no file under `crates/` names the prototype the two large
//!    fixtures were renamed from.
//!
//! Two more fixtures are pinned by their bytes. `twice`: the opening action is taken twice and
//! then closed, and both children's outcomes arrive after it closed (D2); exit 0. `two-records`:
//! its stuck state is true of a real case, one taking of `open` that records `booked` and
//! `briefed`, whose only child ends with an outcome the `children` form does not list. The
//! printed path takes `open` twice, which a runner can do too, since `open` stays admissible
//! until both records are there (D6: D3 is withdrawn).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// The fixture family, relative to the repository root.
const FIXTURES: &str = "fixtures/investigation/child-outcome";

/// The two large fixtures, relative to the repository root.
const LARGE: &str = "fixtures/child-outcome-large";

/// The fixture family of story:canon-simulate.
const SIMULATE: &str = "fixtures/investigation/simulate";

/// Every fixture directory, its family and the exit status `canon simulate` gives it.
const CASES: [(&str, &str, i32); 8] = [
    (FIXTURES, "closed", 0),
    (FIXTURES, "ends-open", 1),
    (FIXTURES, "repeat", 0),
    (FIXTURES, "denied", 1),
    (FIXTURES, "twice", 0),
    (FIXTURES, "two-records", 1),
    (LARGE, "festival", 1),
    (LARGE, "event", 1),
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

fn simulate(family: &str, case: &str) -> Output {
    canon(&[
        "simulate",
        "--path",
        &format!("{family}/{case}/protocol.yaml"),
    ])
}

/// The state of each printed `stuck-state` or `cycle`, as written between `state ` and `: `.
fn printed_states(report: &str) -> Vec<&str> {
    report
        .lines()
        .filter(|line| line.starts_with("stuck-state: ") || line.starts_with("cycle: "))
        .filter_map(|line| {
            let start = line.find("state {")? + "state ".len();
            let end = start + line[start..].find("}: ")? + 1;
            Some(&line[start..end])
        })
        .collect()
}

/// The path of each printed `stuck-state` or `cycle`, one string per step.
fn printed_paths(report: &str) -> Vec<Vec<&str>> {
    report
        .lines()
        .filter_map(|line| {
            let start = line.find("; path: ")? + "; path: ".len();
            let path = &line[start..];
            let path = path.split("; ").next().unwrap_or(path);
            Some(path.split(", then ").collect())
        })
        .collect()
}

/// The items of a printed state, `{a, b}` as `["a", "b"]`.
fn items(state: &str) -> Vec<&str> {
    let inner = &state[1..state.len() - 1];
    if inner.is_empty() {
        Vec::new()
    } else {
        inner.split(", ").collect()
    }
}

/// Whether the printed state holds exactly `records` as its records (its items that name
/// evidence), whatever its other items.
fn records_are(state: &str, records: &[&str]) -> bool {
    let held: Vec<&str> = items(state)
        .into_iter()
        .filter(|item| item.starts_with("evidence "))
        .collect();
    held == records
}

fn last_line(report: &str) -> &str {
    report.lines().last().unwrap_or("")
}

#[test]
fn simulate_lets_a_child_outcome_arrive_after_its_opening() {
    let mut failures = Vec::new();
    let mut reports = std::collections::BTreeMap::new();

    // The exact bytes, the exit status, an empty standard error and a second run's bytes (1 to 7).
    for (family, case, status) in CASES {
        let expected = read(&format!("{family}/{case}/report.txt"));
        let first = simulate(family, case);
        let stdout = text(&first.stdout);
        if first.status.code() != Some(status) || stdout != expected || !first.stderr.is_empty() {
            failures.push(format!(
                "{case}: expected exit {status}, stdout {expected:?}, empty stderr; got exit {:?}, \
                 stdout {stdout:?}, stderr {:?}",
                first.status.code(),
                text(&first.stderr),
            ));
        }
        let second = simulate(family, case);
        if (second.status.code(), &second.stdout, &second.stderr)
            != (first.status.code(), &first.stdout, &first.stderr)
        {
            failures.push(format!("{case}: a second run gave different bytes"));
        }
        reports.insert(case, stdout);
    }

    // 1: `closed` prints only its summary.
    let closed = &reports["closed"];
    let summary = "simulated: protocol `child.closed` revision 1: 4 reachable states, 2 terminable \
                   (outcome `declined` 1, outcome `done` 1), 1 waiting on supplied evidence, a \
                   child outcome or expiry, 0 findings (0 stuck or cycle states)\n";
    if closed != summary {
        failures.push(format!("closed: expected only {summary:?}, got {closed:?}"));
    }

    // 2: `ends-open` has one stuck state, the child that ended with an unlisted outcome.
    let ends_open = &reports["ends-open"];
    let stuck: Vec<&str> = ends_open
        .lines()
        .filter(|line| line.starts_with("stuck-state: "))
        .collect();
    let states = printed_states(ends_open);
    let holds = |state: &str, item: &str| items(state).contains(&item);
    if stuck.len() != 1
        || states.len() != 1
        || !holds(states[0], "evidence `case_outcome`")
        || items(states[0])
            .iter()
            .any(|item| item.starts_with("action "))
    {
        failures.push(format!(
            "ends-open: expected one stuck state holding `evidence `case_outcome`` and no mark, \
             got {ends_open:?}"
        ));
    }
    if states.iter().any(|state| state.contains("result `open`")) {
        failures.push(format!(
            "ends-open: a printed state holds result `open`: {ends_open:?}"
        ));
    }

    // 3: `repeat` is clean, with 12 reachable states and 2 waiting.
    let repeat = &reports["repeat"];
    let summary = last_line(repeat);
    if repeat.lines().count() != 1
        || !summary.contains(": 12 reachable states, ")
        || !summary.contains(", 2 waiting on supplied evidence, a child outcome or expiry, ")
    {
        failures.push(format!(
            "repeat: expected only a summary with 12 reachable states and 2 waiting, got {repeat:?}"
        ));
    }

    // 4: `denied` has one finding, the denial, and no child outcome before the opening.
    let denied = &reports["denied"];
    let findings: Vec<&str> = denied
        .lines()
        .filter(|line| !line.starts_with("simulated: "))
        .collect();
    if findings.len() != 1
        || !findings[0].starts_with("stuck-state: state {capability `delegate` denied}: ")
    {
        failures.push(format!(
            "denied: expected the one finding `stuck-state: state {{capability `delegate` \
             denied}}`, got {denied:?}"
        ));
    }
    for path in printed_paths(denied) {
        let child = path.iter().position(|step| step.starts_with("a child of "));
        let opening = path
            .iter()
            .position(|step| step.starts_with("action `open` "));
        if child.is_some_and(|child| opening.is_none_or(|opening| child < opening)) {
            failures.push(format!(
                "denied: a child outcome comes before a step of `open`: {path:?}"
            ));
        }
    }

    // 5: `festival`.
    let festival = &reports["festival"];
    let summary = "simulated: protocol `festival` revision 6: 48 reachable states, 25 terminable \
                   (outcome `cancelled` 24, outcome `running` 2), 3 waiting on supplied evidence, \
                   a child outcome or expiry, 4 findings (8 stuck or cycle states)";
    if last_line(festival) != summary {
        failures.push(format!(
            "festival: expected the summary {summary:?}, got {:?}",
            last_line(festival)
        ));
    }
    let opened = "evidence `inquiry_opened` about `concept`";
    let waited = [
        opened,
        "evidence `programme_approval` about `programme` result `accepted`",
    ];
    // The child of `inquiry.open` that has not ended holds its `open` record
    // (story:simulate-opens-child-with-its-record).
    let stuck_records = [
        "evidence `case_outcome` result `open`",
        opened,
        "evidence `programme_approval` about `programme`",
    ];
    let states = printed_states(festival);
    if states.iter().any(|state| records_are(state, &waited)) {
        failures.push(format!(
            "festival: a printed state holds only an opening record and an accepted programme, \
             which a child's outcome leads out of: {festival:?}"
        ));
    }
    let with_records: Vec<&&str> = states
        .iter()
        .filter(|state| records_are(state, &stuck_records))
        .collect();
    if with_records.is_empty()
        || with_records
            .iter()
            .any(|state| !items(state).contains(&"action `inquiry.open` opened a child"))
    {
        failures.push(format!(
            "festival: the stuck state {stuck_records:?} is not printed, or not only with the mark \
             of `inquiry.open`: {festival:?}"
        ));
    }

    // 6: `event`.
    let event = &reports["event"];
    let summary = "simulated: protocol `festival.event` revision 3: 46 reachable states, 24 \
                   terminable (outcome `cancelled` 23, outcome `held` 2), 4 waiting on supplied \
                   evidence, a child outcome or expiry, 5 findings (8 stuck or cycle states)";
    if last_line(event) != summary {
        failures.push(format!(
            "event: expected the summary {summary:?}, got {:?}",
            last_line(event)
        ));
    }
    let accepted = "evidence `programme_approval` about `programme` result `accepted`";
    let opened = "evidence `script_opened` about `event`";
    let waited = [
        "evidence `inspection` about `script` result `cleared`",
        accepted,
        opened,
    ];
    // The child of `script.open` that has not ended holds its `open` record
    // (story:simulate-opens-child-with-its-record).
    let stuck_records = [
        "evidence `case_outcome` result `open`",
        "evidence `inspection` about `script`",
        accepted,
        opened,
    ];
    let states = printed_states(event);
    if states.iter().any(|state| records_are(state, &waited)) {
        failures.push(format!(
            "event: a printed state holds only an opening record, an accepted programme and a \
             cleared inspection, which a child's outcome leads out of: {event:?}"
        ));
    }
    let with_records: Vec<&&str> = states
        .iter()
        .filter(|state| records_are(state, &stuck_records))
        .collect();
    if with_records.is_empty()
        || with_records
            .iter()
            .any(|state| !items(state).contains(&"action `script.open` opened a child"))
    {
        failures.push(format!(
            "event: the stuck state {stuck_records:?} is not printed, or not only with the mark of \
             `script.open`: {event:?}"
        ));
    }

    // 7: no line names the limit, here or in the fixtures of story:canon-simulate.
    for (case, report) in &reports {
        if report.lines().any(|line| line.starts_with("limit:")) {
            failures.push(format!("{case}: a line names the limit: {report:?}"));
        }
    }
    let mut unchanged = 0;
    let mut simulate_cases: Vec<String> = std::fs::read_dir(repository_root().join(SIMULATE))
        .expect("the simulate fixtures are readable")
        .map(|entry| {
            entry
                .expect("a fixture entry is readable")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    simulate_cases.sort();
    for case in &simulate_cases {
        let output = simulate(SIMULATE, case);
        if output.stdout.windows(7).any(|window| window == b"limit: ") {
            failures.push(format!("{SIMULATE}/{case}: a line names the limit"));
        }
        if read(&format!("{SIMULATE}/{case}/protocol.yaml")).contains("case_open:") {
            continue;
        }
        unchanged += 1;
        let directory = repository_root().join(SIMULATE).join(case);
        let (stdout, stderr) = if directory.join("refusal.txt").exists() {
            (
                String::new(),
                read(&format!("{SIMULATE}/{case}/refusal.txt")),
            )
        } else {
            (
                read(&format!("{SIMULATE}/{case}/report.txt")),
                String::new(),
            )
        };
        if text(&output.stdout) != stdout || text(&output.stderr) != stderr {
            failures.push(format!(
                "{SIMULATE}/{case}: expected stdout {stdout:?}, stderr {stderr:?}; got stdout \
                 {:?}, stderr {:?}",
                text(&output.stdout),
                text(&output.stderr)
            ));
        }
    }
    // The 15 fixtures of story:canon-simulate with no opening action, the 5 `revise*` fixtures of
    // story:simulate-revise-step with none, and the 2 `revised-by*` fixtures of
    // story:artifact-revised-by with none.
    if unchanged != 22 {
        failures.push(format!(
            "{SIMULATE}: expected 22 fixtures with no opening action, found {unchanged}"
        ));
    }

    // 8: no Python file is tracked, and no file under `crates/` names the prototype.
    let tracked = Command::new("git")
        .current_dir(repository_root())
        .args(["ls-files", "--", "*.py"])
        .output()
        .expect("git runs");
    if !tracked.status.success() || !tracked.stdout.is_empty() {
        failures.push(format!(
            "git ls-files '*.py': exit {:?}, {:?}",
            tracked.status.code(),
            text(&tracked.stdout)
        ));
    }
    // Spelled in pieces, so this file does not hold it.
    let name = ["s", "d", "c"].concat();
    let mut naming = Vec::new();
    let mut pending = vec![repository_root().join("crates")];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("a directory under crates/ is readable") {
            let path = entry.expect("an entry is readable").path();
            if path.is_dir() {
                pending.push(path);
            } else if let Ok(bytes) = std::fs::read(&path)
                && String::from_utf8_lossy(&bytes)
                    .to_lowercase()
                    .contains(name.as_str())
            {
                naming.push(relative(&path));
            }
        }
    }
    naming.sort();
    // Named before this story, in a test's account of where its fixture came from.
    let before = ["crates/canon-cli/tests/supplied_evidence.rs".to_owned()];
    if naming != before {
        failures.push(format!(
            "files under crates/ naming the prototype: {naming:?}, expected only {before:?}"
        ));
    }

    assert!(
        failures.is_empty(),
        "{} expectations failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

fn relative(path: &Path) -> String {
    path.strip_prefix(repository_root())
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

/// The red run must not come from a fixture typo: every fixture protocol is one `canon validate`
/// accepts, and every fixture directory is a case carrying exactly its two files.
#[test]
fn every_child_outcome_fixture_validates_and_is_a_case() {
    let entries = |directory: PathBuf| {
        let mut names: Vec<String> = std::fs::read_dir(directory)
            .expect("a fixture directory is readable")
            .map(|entry| {
                entry
                    .expect("a fixture entry is readable")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        names.sort();
        names
    };
    for family in [FIXTURES, LARGE] {
        let mut cases: Vec<String> = CASES
            .iter()
            .filter(|(of, ..)| *of == family)
            .map(|(_, case, _)| (*case).to_owned())
            .collect();
        cases.sort();
        assert_eq!(entries(repository_root().join(family)), cases, "{family}");
    }
    for (family, case, _) in CASES {
        let output = canon(&[
            "validate",
            "--path",
            &format!("{family}/{case}/protocol.yaml"),
        ]);
        assert!(output.status.success(), "{case}: {}", text(&output.stderr));
        assert_eq!(
            entries(repository_root().join(family).join(case)),
            ["protocol.yaml", "report.txt"],
            "{case}"
        );
    }
}
