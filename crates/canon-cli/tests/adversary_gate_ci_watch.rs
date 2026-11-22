//! Adversary, wave 10 review, area `gate`: `scripts/ci-watch` against story:local-gate-equals-ci
//! ("a red forge run makes the watch script exit non-zero") and `AGENTS.md` § Releases ("wait for
//! CI with it"; the bot merges through the API with its bypass of the ruleset, so the ruleset's
//! required check does not hold the merge and this script is the only reader of the result).
//!
//! The script counts the checks `gh pr checks` reports and never asks which checks must be there.
//! A pull request on which the job `canon / task check` never reported (a `check.yml` GitHub
//! refuses to start, a trigger changed, the run not created yet) shows only the other workflow's
//! check, and the script exits 0 with "all 1 check(s) passed".
//!
//! A stub `gh` first on `PATH` prints a recorded answer; no network request is made.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// `gh pr checks 28` of a pull request on which only `pages.yml` reported: its job passed, the
/// job of `check.yml` is not in the answer.
const ONLY_THE_DOCUMENTATION_JOB: &str =
    "Build documentation\tpass\t2m1s\thttps://github.com/beyond10x/canon/actions/runs/2/job/2\n";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

fn watch(case: &str, answer: &str) -> Output {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adversary-gate-ci-watch")
        .join(case);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch");
    fs::write(dir.join("answer"), answer).expect("answer");
    let stub = dir.join("gh");
    fs::write(&stub, "#!/bin/sh\ncat \"$(dirname \"$0\")/answer\"\n").expect("stub");
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
    Command::new("sh")
        .arg(repo_root().join("scripts/ci-watch"))
        .args(["--timeout", "2", "--interval", "1", "28"])
        .env("PATH", path)
        .env_remove("CI_WATCH_GH")
        .output()
        .expect("sh runs scripts/ci-watch")
}

#[test]
fn a_pull_request_whose_task_check_job_never_reported_is_not_reported_green() {
    let out = watch("check-job-missing", ONLY_THE_DOCUMENTATION_JOB);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_ne!(
        out.status.code(),
        Some(0),
        "the job `canon / task check` (check.yml) is not among the reported checks, and \
         scripts/ci-watch still exits 0, so a merge or release waiting on it proceeds without \
         the gate having run. stdout:\n{stdout}stderr:\n{stderr}"
    );
}
