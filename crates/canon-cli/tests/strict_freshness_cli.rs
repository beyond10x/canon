//! Acceptance for story:freshness-needs-trusted-timestamps, against `canon evaluate`: without
//! `--strict-freshness` an undated record of a kind with `max_age`, a record observed after `--at`
//! and a run without `--at` are accepted as before (pinned here); with it each is refused, naming
//! its code and the file of the record it cites; `--strict-freshness <SKEW>` allows a record
//! observed up to that long after `--at`; `canon evaluate --help` names the mode.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const AT: &str = "2026-10-04T12:00:00Z";
const CASE: &str = "format: canon-case/1\nid: C-1\nprotocol: p\nartifacts: {a: {revision: r1}}\n";
const PROTOCOL: &str = "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
     evidence_kinds: {fresh: {max_age: 1h}}\n\
     claims: {f: {true_when: {evidence: {kind: fresh, result: pass}}}}\n";
/// The claim entry `canon evaluate` prints when the record applies.
const APPLIED: &str = "\"f\": {\n      \"value\": \"true\"\n    }";

fn canon(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .args(args)
        .output()
        .expect("canon runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn path(p: &Path) -> String {
    p.to_str().expect("utf-8 path").to_owned()
}

/// A scratch directory holding the compiled protocol, the case and an evidence directory with
/// one record `e1` of kind `fresh`, observed at `observed_at` when given.
struct Inputs {
    ir: PathBuf,
    case: PathBuf,
    evidence: PathBuf,
}

impl Inputs {
    fn new(name: &str, observed_at: Option<&str>) -> Self {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("strict-freshness-{name}"));
        if dir.exists() {
            std::fs::remove_dir_all(&dir).expect("old scratch directory removed");
        }
        std::fs::create_dir_all(dir.join("evidence")).expect("scratch directory created");
        let protocol = dir.join("protocol.yaml");
        std::fs::write(&protocol, PROTOCOL).expect("protocol written");
        let compiled = canon(&["compile", "--path", &path(&protocol)]);
        assert_eq!(
            compiled.status.code(),
            Some(0),
            "{}",
            text(&compiled.stderr)
        );
        let ir = dir.join("protocol.ir.json");
        std::fs::write(&ir, &compiled.stdout).expect("IR written");
        let case = dir.join("case.yaml");
        std::fs::write(&case, CASE).expect("case written");
        let observed = observed_at.map_or(String::new(), |at| format!("observed_at: {at}\n"));
        let evidence = dir.join("evidence");
        std::fs::write(
            evidence.join("e1.yaml"),
            format!(
                "format: canon-evidence/1\nid: e1\nkind: fresh\nresult: pass\nsubject: a\n\
                 subject_revision: r1\n{observed}"
            ),
        )
        .expect("record written");
        Self { ir, case, evidence }
    }

    fn record(&self) -> String {
        path(&self.evidence.join("e1.yaml"))
    }

    fn evaluate(&self, extra: &[&str]) -> Output {
        let (ir, case, evidence) = (path(&self.ir), path(&self.case), path(&self.evidence));
        let mut args = vec![
            "evaluate",
            "--ir",
            &ir,
            "--case",
            &case,
            "--evidence",
            &evidence,
        ];
        args.extend_from_slice(extra);
        canon(&args)
    }
}

fn accepted(run: &Output) -> String {
    assert_eq!(run.status.code(), Some(0), "{}", text(&run.stderr));
    let stdout = text(&run.stdout);
    assert!(stdout.contains(APPLIED), "{stdout}");
    stdout
}

fn refused(run: &Output) -> String {
    assert_eq!(run.status.code(), Some(1), "{}", text(&run.stdout));
    assert!(run.stdout.is_empty(), "{}", text(&run.stdout));
    text(&run.stderr)
}

#[test]
fn by_default_undated_future_dated_and_no_instant_are_accepted() {
    let undated = Inputs::new("default-undated", None);
    accepted(&undated.evaluate(&["--at", AT]));
    let future = Inputs::new("default-future", Some("2026-10-04T13:00:00Z"));
    accepted(&future.evaluate(&["--at", AT]));
    let old = Inputs::new("default-old", Some("2000-01-01T00:00:00Z"));
    accepted(&old.evaluate(&[]));
}

#[test]
fn strict_freshness_refuses_an_undated_record_naming_its_file() {
    let inputs = Inputs::new("strict-undated", None);
    assert_eq!(
        refused(&inputs.evaluate(&["--at", AT, "--strict-freshness"])),
        format!(
            "error[undated-evidence]: {}: evidence `e1` gives no observed_at, and its kind \
             `fresh` has max_age `1h`: strict freshness needs the time it was observed\n",
            inputs.record()
        )
    );
}

#[test]
fn strict_freshness_refuses_a_record_observed_after_the_instant_beyond_the_skew() {
    let inputs = Inputs::new("strict-future", Some("2026-10-04T12:00:30Z"));
    assert_eq!(
        refused(&inputs.evaluate(&["--strict-freshness", "--at", AT])),
        format!(
            "error[future-evidence]: {}: evidence `e1` observed_at `2026-10-04T12:00:30Z` is \
             later than the evaluation instant `2026-10-04T12:00:00Z` by 30 s, more than the \
             allowed clock skew `0s`\n",
            inputs.record()
        )
    );
    let by_default = accepted(&inputs.evaluate(&["--at", AT]));
    let within: [&[&str]; 2] = [
        &["--strict-freshness", "30s", "--at", AT],
        &["--at", AT, "--strict-freshness=1m"],
    ];
    for args in within {
        assert_eq!(accepted(&inputs.evaluate(args)), by_default, "{args:?}");
    }
    assert!(
        refused(&inputs.evaluate(&["--strict-freshness", "29s", "--at", AT]))
            .starts_with("error[future-evidence]: "),
    );
}

#[test]
fn strict_freshness_refuses_a_run_without_an_instant() {
    let inputs = Inputs::new("strict-no-at", Some("2026-10-04T11:30:00Z"));
    assert_eq!(
        refused(&inputs.evaluate(&["--strict-freshness"])),
        "error[missing-instant]: strict freshness needs the evaluation instant (`--at`): \
         evidence kind `fresh` has max_age `1h`\n"
    );
    assert_eq!(
        refused(&inputs.evaluate(&["--strict-freshness", "soon", "--at", AT])),
        "error[invalid-skew]: the allowed clock skew `soon` is not an age the evaluator can \
         read: a whole number of seconds, minutes, hours or days, written with its unit s, m, h \
         or d\n"
    );
}

#[test]
fn evaluate_help_names_the_strict_freshness_mode() {
    let run = canon(&["evaluate", "--help"]);
    assert_eq!(run.status.code(), Some(0), "{}", text(&run.stderr));
    let help = text(&run.stdout);
    for named in [
        "--strict-freshness [<SKEW>]",
        "undated-evidence",
        "future-evidence",
        "missing-instant",
        "invalid-skew",
    ] {
        assert!(
            help.contains(named),
            "`canon evaluate --help` names {named}:\n{help}"
        );
    }
}
