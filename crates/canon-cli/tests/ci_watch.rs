//! Acceptance for story:local-gate-equals-ci: `scripts/ci-watch` waits until no check is pending,
//! prints each check with its result and link, and exits 0 only when every check passed.
//!
//! The script runs with a stub `gh` first on `PATH` that prints a recorded answer of
//! `gh pr checks <target>` (tab separated: name, result, elapsed, link); no network request is made.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Instant;

const PASSED: &str = "canon / task check\tpass\t5m3s\thttps://github.com/beyond10x/canon/actions/runs/1/job/1\n\
Build documentation\tpass\t2m1s\thttps://github.com/beyond10x/canon/actions/runs/2/job/2\n\
Skipped job\tskipping\t0\thttps://github.com/beyond10x/canon/actions/runs/2/job/3\n";

const FAILED: &str = "canon / task check\tfail\t5m3s\thttps://github.com/beyond10x/canon/actions/runs/1/job/1\n\
Build documentation\tpass\t2m1s\thttps://github.com/beyond10x/canon/actions/runs/2/job/2\n";

const CANCELLED: &str = "canon / task check\tcancel\t1m0s\thttps://github.com/beyond10x/canon/actions/runs/1/job/1\n\
Build documentation\tpass\t2m1s\thttps://github.com/beyond10x/canon/actions/runs/2/job/2\n";

const PENDING: &str = "canon / task check\tpending\t0\thttps://github.com/beyond10x/canon/actions/runs/1/job/1\n\
Build documentation\tpass\t2m1s\thttps://github.com/beyond10x/canon/actions/runs/2/job/2\n";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

/// Runs the script against a stub `gh` that prints `answer` and records its arguments.
/// Returns the output, the recorded `gh` arguments and the wall time.
fn watch(case: &str, answer: &str, extra: &[&str]) -> (Output, String, f64) {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ci-watch")
        .join(case);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch");
    fs::write(dir.join("answer"), answer).expect("answer");
    let stub = dir.join("gh");
    fs::write(
        &stub,
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$(dirname \"$0\")/args\"\ncat \"$(dirname \"$0\")/answer\"\n",
    )
    .expect("stub");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&stub, fs::Permissions::from_mode(0o755)).expect("chmod");
    }
    let path = format!(
        "{}:{}",
        dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let started = Instant::now();
    let output = Command::new("sh")
        .arg(repo_root().join("scripts/ci-watch"))
        .args(["--timeout", "2", "--interval", "1"])
        .args(extra)
        .arg("28")
        .env("PATH", path)
        .env_remove("CI_WATCH_GH")
        .output()
        .expect("sh runs scripts/ci-watch");
    let args = fs::read_to_string(dir.join("args")).unwrap_or_default();
    (output, args, started.elapsed().as_secs_f64())
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn all_passed_exits_zero_and_prints_each_check_with_its_link() {
    let (out, args, _) = watch("passed", PASSED, &[]);
    let stdout = text(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{stdout}{}", text(&out.stderr));
    assert!(
        args.starts_with("pr checks 28"),
        "gh was called with: {args}"
    );
    for line in [
        "canon / task check",
        "https://github.com/beyond10x/canon/actions/runs/1/job/1",
        "Build documentation",
        "https://github.com/beyond10x/canon/actions/runs/2/job/2",
    ] {
        assert!(stdout.contains(line), "`{line}` missing from:\n{stdout}");
    }
}

#[test]
fn a_failed_check_exits_non_zero_and_names_it() {
    let (out, _, _) = watch("failed", FAILED, &[]);
    let stdout = text(&out.stdout);
    assert_eq!(out.status.code(), Some(1), "{stdout}");
    assert!(
        stdout.contains("fail") && stdout.contains("runs/1/job/1"),
        "{stdout}"
    );
}

#[test]
fn a_cancelled_check_exits_non_zero() {
    let (out, _, _) = watch("cancelled", CANCELLED, &[]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out.stdout));
}

#[test]
fn a_check_still_pending_past_the_timeout_exits_non_zero() {
    let (out, args, secs) = watch("pending", PENDING, &[]);
    assert_eq!(out.status.code(), Some(2), "{}", text(&out.stdout));
    assert!(
        text(&out.stderr).contains("timeout"),
        "stderr: {}",
        text(&out.stderr)
    );
    assert!(args.lines().count() >= 2, "it polled again: {args}");
    assert!(secs < 5.0, "the timeout of 2 s held for {secs} s");
}
