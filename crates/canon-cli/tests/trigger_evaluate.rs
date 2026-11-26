//! story:trigger-evaluate, `canon trigger evaluate --rules <set> --snapshot <file> --protocols
//! <file> --history <file> --now <instant>` against the command line: over 3 matching and 4
//! non-matching rows the active auto rule lists 3 `would-start` and the active offer rule 3
//! `offer`; two runs, and a run over the rows and rules in reverse order, print the same bytes; the
//! command exits 0 with the same plan in a directory it cannot write, inside a network namespace
//! with no interface up (`unshare --net --map-current-user`); a rule file that is not a valid rule,
//! an instant that is not one and a file that cannot be read are refused. What a verdict means is
//! the conformance scenario `conformance/scenarios/trigger-evaluate.yaml`.

use std::fs;
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

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

const DIR: &str = "fixtures/trigger/evaluate";
const NOW: &str = "2026-11-26T12:00:00Z";

/// The arguments of one evaluation, each file named relative to `dir`.
fn arguments(dir: &str, set: &str, snapshot: &str, history: &str, now: &str) -> Vec<String> {
    let at = |file: &str| {
        if dir.is_empty() {
            file.to_owned()
        } else {
            format!("{dir}/{file}")
        }
    };
    vec![
        "trigger".to_owned(),
        "evaluate".to_owned(),
        "--rules".to_owned(),
        at(set),
        "--snapshot".to_owned(),
        at(snapshot),
        "--protocols".to_owned(),
        at("protocols.json"),
        "--history".to_owned(),
        at(history),
        "--now".to_owned(),
        now.to_owned(),
    ]
}

fn evaluate(set: &str, snapshot: &str, history: &str, now: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(repository_root())
        .args(arguments(DIR, set, snapshot, history, now))
        .output()
        .expect("canon runs")
}

/// One entry of the plan of `fixtures/trigger/evaluate/set.yaml` over `snapshot.yaml`.
fn entry(rule: &str, row: &str, entered_at: &str, channel: &str, verdict: &str) -> String {
    format!(
        "    {{\n      \"firing_id\": \"{rule}/invoice:{row}/{entered_at}\",\n      \"input\": {{\n        \
         \"channel\": \"{channel}\",\n        \"invoice\": \"{row}\"\n      }},\n      \
         \"kind\": \"invoice\",\n      \"protocol\": \"send-reminder\",\n      \"row\": \"{row}\",\n      \
         \"rule\": \"{rule}\",\n      \"rule_state\": \"active\",\n      \"verdict\": \"{verdict}\"\n    }}"
    )
}

/// The plan: 3 `offer` of `overdue-offer@2`, then 3 `would-start` of `overdue-reminder@1`, each
/// in row-id order.
fn plan() -> String {
    let rows = [
        ("inv-1", "2026-11-20T09:00:00Z"),
        ("inv-2", "2026-11-20T10:00:00Z"),
        ("inv-3", "2026-11-20T11:00:00Z"),
    ];
    let mut entries = Vec::new();
    for (rule, channel, verdict) in [
        ("overdue-offer@2", "letter", "offer"),
        ("overdue-reminder@1", "email", "would-start"),
    ] {
        for (row, entered_at) in rows {
            entries.push(entry(rule, row, entered_at, channel, verdict));
        }
    }
    format!(
        "{{\n  \"entries\": [\n{}\n  ],\n  \"format\": \"canon-trigger-plan/1\"\n}}\n",
        entries.join(",\n")
    )
}

#[test]
fn three_matching_rows_give_three_would_start_and_three_offer() {
    let output = evaluate("set.yaml", "snapshot.yaml", "history.yaml", NOW);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_eq!(text(&output.stdout), plan());
    assert_eq!(text(&output.stderr), "");
}

#[test]
fn two_runs_and_a_reversed_input_print_the_same_bytes() {
    let first = evaluate("set.yaml", "snapshot.yaml", "history.yaml", NOW);
    let second = evaluate("set.yaml", "snapshot.yaml", "history.yaml", NOW);
    let reversed = evaluate(
        "set-reversed.yaml",
        "snapshot-reversed.yaml",
        "history.yaml",
        NOW,
    );
    for output in [&first, &second, &reversed] {
        assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    }
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stdout, reversed.stdout);
    assert_eq!(text(&first.stdout), plan());
}

/// A directory below the build's temporary directory, holding a copy of the evaluation's files,
/// that nobody may write while the test runs; its permissions are restored and it is removed when
/// dropped.
struct ReadOnlyCopy(PathBuf);

impl ReadOnlyCopy {
    fn new(files: &[&str]) -> Self {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("trigger-evaluate-read-only-{}", std::process::id()));
        if dir.exists() {
            set_mode(&dir, 0o755);
            fs::remove_dir_all(&dir).expect("an earlier copy is removed");
        }
        fs::create_dir_all(&dir).expect("the copy's directory is created");
        for file in files {
            fs::copy(repository_root().join(DIR).join(file), dir.join(file))
                .expect("a fixture is copied");
        }
        set_mode(&dir, 0o555);
        ReadOnlyCopy(dir)
    }

    fn entries(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(&self.0)
            .expect("the copy reads")
            .map(|entry| {
                entry
                    .expect("an entry reads")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        names.sort();
        names
    }
}

impl Drop for ReadOnlyCopy {
    fn drop(&mut self) {
        set_mode(&self.0, 0o755);
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn set_mode(dir: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(dir, fs::Permissions::from_mode(mode)).expect("permissions are set");
}

/// `program` run with `args` in `dir`, inside a new network namespace and a user namespace that
/// maps only the current user, so the process has no interface up and no privilege over the
/// directory it could not write before.
fn isolated(dir: &Path, program: &str, args: &[String]) -> Output {
    Command::new("unshare")
        .current_dir(dir)
        .args(["--net", "--map-current-user", "--", program])
        .args(args)
        .output()
        .expect("unshare runs: the test needs util-linux's unshare and unprivileged user namespaces")
}

#[test]
fn evaluate_exits_0_in_a_read_only_directory_without_network() {
    let files = [
        "set.yaml",
        "overdue-offer.yaml",
        "overdue-reminder.yaml",
        "snapshot.yaml",
        "protocols.json",
        "history.yaml",
    ];
    let copy = ReadOnlyCopy::new(&files);
    let before = copy.entries();

    // Controls: in that namespace the directory cannot be written and no interface but `lo` exists.
    let touched = isolated(&copy.0, "touch", &["probe".to_owned()]);
    assert_ne!(
        touched.status.code(),
        Some(0),
        "the directory must not be writable inside the namespace"
    );
    let devices = isolated(&copy.0, "cat", &["/proc/net/dev".to_owned()]);
    assert_eq!(devices.status.code(), Some(0), "{}", text(&devices.stderr));
    let interfaces: Vec<String> = text(&devices.stdout)
        .lines()
        .filter_map(|line| line.split_once(':').map(|(name, _)| name.trim().to_owned()))
        .collect();
    assert_eq!(interfaces, ["lo"], "{}", text(&devices.stdout));

    let output = isolated(
        &copy.0,
        env!("CARGO_BIN_EXE_canon"),
        &arguments("", "set.yaml", "snapshot.yaml", "history.yaml", NOW),
    );
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_eq!(text(&output.stdout), plan());
    assert_eq!(text(&output.stderr), "");
    assert_eq!(copy.entries(), before);
}

#[test]
fn a_rule_file_that_is_not_a_valid_rule_is_refused_naming_the_file() {
    let output = evaluate("set-invalid.yaml", "snapshot.yaml", "history.yaml", NOW);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(text(&output.stdout), "");
    let stderr = text(&output.stderr);
    assert!(
        stderr.starts_with("error[invalid-value]: ")
            && stderr.contains("mode-sometimes.yaml: mode: ")
            && stderr.lines().count() == 1,
        "{stderr}"
    );
}

#[test]
fn now_that_is_not_an_instant_is_refused() {
    let output = evaluate("set.yaml", "snapshot.yaml", "history.yaml", "2026-11-26");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(text(&output.stdout), "");
    let stderr = text(&output.stderr);
    assert!(
        stderr.starts_with("error[invalid-instant]: now: `2026-11-26`") && stderr.lines().count() == 1,
        "{stderr}"
    );
}

#[test]
fn a_file_that_cannot_be_read_exits_2() {
    let output = evaluate("set.yaml", "snapshot.yaml", "no-such-history.yaml", NOW);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(text(&output.stdout), "");
    assert!(
        text(&output.stderr).starts_with("error[unreadable]: "),
        "{}",
        text(&output.stderr)
    );
}
