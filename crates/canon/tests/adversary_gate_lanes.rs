//! Adversary, wave 10 review, area `gate`: the lanes of `Taskfile.yml` against
//! `~/beyond10x/AGENTS.md` § Tests stay fast ("The runner enforces the budget: a test over 60 s
//! is killed and counts as failed (cargo-nextest `slow-timeout` with `terminate-after`)") and
//! story:fast-test-step ("the runner kills at 60 s").
//!
//! `task docs-site-check` is the whole test step of a `docs-site-only` pull request
//! (`check.yml`, `if: steps.classify.outputs.class == 'docs-site-only'`). It runs the tests of
//! `canon-docs` with `cargo test`, which has no per-test kill; `.config/nextest.toml`'s 60 s
//! `slow-timeout` holds only under `cargo nextest run`. A `canon-docs` test that hangs holds the
//! job until the job's `timeout-minutes: 60`.

use std::fs;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("run this test through cargo"))
        .join("../..")
}

#[test]
fn the_docs_site_lane_runs_every_test_under_the_runner_that_kills_at_60_s() {
    let text = fs::read_to_string(root().join("Taskfile.yml")).expect("Taskfile.yml");
    let tasks: serde_yaml_ng::Value = serde_yaml_ng::from_str(&text).expect("Taskfile.yml is YAML");
    let cmds = tasks["tasks"]["docs-site-check"]["cmds"]
        .as_sequence()
        .expect("task docs-site-check has cmds");
    let unbounded: Vec<&str> = cmds
        .iter()
        .filter_map(|cmd| cmd.as_str())
        .filter(|cmd| cmd.trim_start().starts_with("cargo test"))
        .collect();
    assert!(
        unbounded.is_empty(),
        "task docs-site-check runs tests outside cargo-nextest, where no test is killed at 60 s: \
         {unbounded:?}"
    );
}
