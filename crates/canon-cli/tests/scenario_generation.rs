//! Acceptance for story:scenario-generation: `canon generate` writes `canon-conformance/1`
//! scenarios, one minimal witness evidence set per declared outcome and per action blocked in some
//! state, found over the state space `canon check` enumerates.
//!
//! Four expectations over the base investigation protocol
//! (`fixtures/investigation/check/base/protocol.yaml`):
//!
//! 1. `canon generate` writes exactly one scenario per declared outcome
//!    (`outcome.<outcome>.legitimate.yaml`) and per action blocked in some state
//!    (`action.<action>.blocked.yaml`), and nothing else;
//! 2. each matches the committed expectation under `fixtures/investigation/generate/` byte for
//!    byte;
//! 3. `canon conform run` over the output directory passes every scenario;
//! 4. for each scenario, removing any one evidence record makes `canon conform run` fail that
//!    scenario: the witness is minimal.
//!
//! The fourth is shown able to fail before it is trusted: the supported witness given a second
//! observation still passes with either observation removed, and the same check reports both and
//! nothing else.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use b10x_canon::{conform, model};

/// The protocol the scenarios are generated from, relative to the repository root.
const PROTOCOL: &str = "fixtures/investigation/check/base/protocol.yaml";

/// The committed expectations, relative to the repository root.
const EXPECTED: &str = "fixtures/investigation/generate";

/// The line an evidence list opens with, and the prefix each of its records opens with, in the
/// layout `canon generate` writes and the expectations hold.
const EVIDENCE: &str = "      evidence:";
const RECORD: &str = "        - ";

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

/// A fresh, empty directory under the test scratch area.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("scenario-generation-{name}"));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("an old scratch directory is removable");
    }
    std::fs::create_dir_all(&dir).expect("a scratch directory is creatable");
    dir
}

/// The `*.yaml` file names of a directory, sorted.
fn yaml_files(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", dir.display()))
        .map(|entry| {
            entry
                .expect("a directory entry is readable")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    names
}

/// The scenario files the protocol must yield: one per declared outcome, and one per action that
/// declares a precondition or requires a capability. In this protocol each such action is blocked
/// in some state (its precondition reads a claim that is UNKNOWN without evidence, and a required
/// capability can be denied), and every other action is admissible in every state.
fn wanted_files() -> Vec<String> {
    let source = std::fs::read_to_string(repository_root().join(PROTOCOL))
        .expect("the base protocol is readable");
    let protocol = model::parse(&source).expect("the base protocol parses");
    let mut names: Vec<String> = protocol
        .outcomes
        .iter()
        .map(|(id, _)| format!("outcome.{}.legitimate.yaml", id.as_str()))
        .chain(
            protocol
                .actions
                .iter()
                .filter(|(_, action)| action.precondition.is_some() || !action.requires.is_empty())
                .map(|(id, _)| format!("action.{}.blocked.yaml", id.as_str())),
        )
        .collect();
    names.sort();
    names
}

/// How many evidence records the scenario's one evaluate step carries, read by Canon's own
/// scenario parser.
fn record_count(scenario: &str) -> usize {
    let parsed = conform::parse(scenario).expect("the scenario parses");
    assert_eq!(parsed.steps.len(), 1, "a witness scenario has one step");
    match &parsed.steps[0].kind {
        conform::StepKind::Evaluate { inputs, .. } => inputs.evidence.len(),
        conform::StepKind::Compile { .. } => panic!("a witness scenario's step evaluates"),
    }
}

/// The scenario's id, read by Canon's own scenario parser.
fn scenario_id(scenario: &str) -> String {
    conform::parse(scenario).expect("the scenario parses").id
}

/// Splits the scenario text into the lines before its evidence records, each record's lines, and
/// the lines after them.
fn split_records(scenario: &str) -> (Vec<&str>, Vec<Vec<&str>>, Vec<&str>) {
    let lines: Vec<&str> = scenario.lines().collect();
    let open = lines
        .iter()
        .position(|line| *line == EVIDENCE)
        .expect("the scenario lists its evidence records in block style");
    let mut records: Vec<Vec<&str>> = Vec::new();
    let mut at = open + 1;
    while at < lines.len() && lines[at].starts_with("        ") {
        if lines[at].starts_with(RECORD) {
            records.push(Vec::new());
        }
        records
            .last_mut()
            .expect("a record's lines follow its opening line")
            .push(lines[at]);
        at += 1;
    }
    (lines[..=open].to_vec(), records, lines[at..].to_vec())
}

/// The scenario with its evidence records replaced by `records`; an empty list is written `[]`.
fn with_records(scenario: &str, records: &[Vec<&str>]) -> String {
    let (before, _, after) = split_records(scenario);
    let mut lines: Vec<String> = before.iter().map(|line| (*line).to_owned()).collect();
    if records.is_empty() {
        *lines.last_mut().expect("the evidence line") = format!("{EVIDENCE} []");
    }
    lines.extend(records.iter().flatten().map(|line| (*line).to_owned()));
    lines.extend(after.iter().map(|line| (*line).to_owned()));
    let mut text = lines.join("\n");
    text.push('\n');
    text
}

/// The scenario with its evidence record at `index` removed. Canon's parser confirms exactly one
/// record went.
fn without_record(scenario: &str, index: usize) -> String {
    let (_, mut records, _) = split_records(scenario);
    records.remove(index);
    let changed = with_records(scenario, &records);
    assert_eq!(
        record_count(&changed),
        record_count(scenario) - 1,
        "removing record {index} removed exactly one record"
    );
    changed
}

/// Runs `canon conform run` over a directory holding only `scenario`, from the repository root.
fn conform_one(name: &str, scenario: &str) -> Output {
    let dir = scratch(name);
    std::fs::write(dir.join("scenario.yaml"), scenario).expect("the scenario is writable");
    canon(&[
        "conform",
        "run",
        "--scenarios",
        dir.to_str().expect("a UTF-8 scratch path"),
    ])
}

/// The indices of the evidence records whose removal leaves the scenario passing: empty exactly
/// when the witness is minimal. Each removal that does make it fail must fail that scenario, not
/// leave it unreadable.
fn removable_records(label: &str, scenario: &str) -> Result<Vec<usize>, String> {
    let id = scenario_id(scenario);
    let mut removable = Vec::new();
    for index in 0..record_count(scenario) {
        let output = conform_one(
            &format!("{label}-without-{index}"),
            &without_record(scenario, index),
        );
        let stdout = text(&output.stdout);
        let summary = stdout.lines().rfind(|l| l.starts_with("conform: "));
        match (output.status.code(), summary) {
            (Some(0), Some("conform: 1 passed, 0 failed, 0 unreadable")) => removable.push(index),
            (Some(1), Some("conform: 0 passed, 1 failed, 0 unreadable"))
                if stdout.contains(&format!("failed: scenario `{id}` step `witness`: ")) => {}
            _ => {
                return Err(format!(
                    "{label} without record {index}: expected the scenario to pass or fail, got \
                     exit {:?}, stdout {stdout:?}, stderr {:?}",
                    output.status.code(),
                    text(&output.stderr),
                ));
            }
        }
    }
    Ok(removable)
}

#[test]
fn generated_scenarios_witness_every_outcome() {
    // The minimality check can fail: the supported witness with a redundant second observation
    // still passes, and so does it with one observation removed, which the check reports.
    let supported = std::fs::read_to_string(
        repository_root()
            .join(EXPECTED)
            .join("outcome.supported.legitimate.yaml"),
    )
    .expect("the supported expectation is readable");
    let (_, mut records, _) = split_records(&supported);
    let redundant: Vec<&str> = records[records.len() - 1]
        .iter()
        .map(|line| {
            if line.trim() == "id: \"e1\"" {
                "          id: \"e2\""
            } else {
                line
            }
        })
        .collect();
    records.push(redundant);
    let padded = with_records(&supported, &records);
    assert_eq!(record_count(&padded), record_count(&supported) + 1);
    let control = conform_one("control", &padded);
    assert_eq!(
        control.status.code(),
        Some(0),
        "the padded control passes: {}",
        text(&control.stdout)
    );
    // Either observation can go while its copy stays; the falsification attempt cannot.
    let caught = removable_records("control", &padded).expect("the control runs");
    assert_eq!(
        caught,
        vec![records.len() - 2, records.len() - 1],
        "the check reports both observations of a non-minimal witness, and nothing else"
    );

    let root = repository_root();
    let expected_dir = root.join(EXPECTED);
    let wanted = wanted_files();
    assert_eq!(
        yaml_files(&expected_dir),
        wanted,
        "the committed expectations are exactly one per outcome and per blocked action"
    );

    let out = scratch("out");
    let generated = canon(&[
        "generate",
        "--path",
        PROTOCOL,
        "--out",
        out.to_str().expect("a UTF-8 scratch path"),
    ]);
    assert_eq!(
        generated.status.code(),
        Some(0),
        "canon generate succeeds: stdout {:?}, stderr {:?}",
        text(&generated.stdout),
        text(&generated.stderr)
    );

    let mut failures = Vec::new();

    // 1. Exactly one scenario per declared outcome and per action blocked in some state.
    let written = yaml_files(&out);
    if written != wanted {
        failures.push(format!(
            "1: expected the files {wanted:?}, canon generate wrote {written:?}"
        ));
    }

    // 2. Each matches its committed expectation byte for byte.
    for name in &wanted {
        let got = std::fs::read(out.join(name));
        let want = std::fs::read(expected_dir.join(name)).expect("an expectation is readable");
        match got {
            Ok(got) if got == want => {}
            Ok(got) => failures.push(format!(
                "2: {name} differs from {EXPECTED}/{name}:\n--- expected\n{}--- got\n{}",
                text(&want),
                text(&got)
            )),
            Err(error) => failures.push(format!("2: {name} was not written: {error}")),
        }
    }

    // 3. `canon conform run` over the output directory passes every scenario.
    let run = canon(&[
        "conform",
        "run",
        "--scenarios",
        out.to_str().expect("a UTF-8 scratch path"),
    ]);
    let stdout = text(&run.stdout);
    let summary = stdout.lines().rfind(|l| l.starts_with("conform: "));
    let all_passed = format!("conform: {} passed, 0 failed, 0 unreadable", wanted.len());
    if run.status.code() != Some(0) || summary != Some(all_passed.as_str()) {
        failures.push(format!(
            "3: expected exit 0 and `{all_passed}`, got exit {:?}, stdout {stdout:?}, stderr {:?}",
            run.status.code(),
            text(&run.stderr)
        ));
    }

    // 4. Removing any one evidence record makes `canon conform run` fail that scenario.
    let mut removals = 0;
    for name in &written {
        let scenario = std::fs::read_to_string(out.join(name)).expect("a scenario is readable");
        removals += record_count(&scenario);
        match removable_records(name, &scenario) {
            Ok(removable) if removable.is_empty() => {}
            Ok(removable) => failures.push(format!(
                "4: {name} is not minimal: it still passes without record(s) {removable:?}"
            )),
            Err(why) => failures.push(format!("4: {why}")),
        }
    }
    if removals == 0 {
        failures.push("4: no scenario carries an evidence record, so none was removed".to_owned());
    }

    assert!(
        failures.is_empty(),
        "{} expectation failure(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}
