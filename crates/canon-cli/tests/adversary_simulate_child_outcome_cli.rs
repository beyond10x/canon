//! Adversary pass 1 for story:simulate-child-outcome-step, from the command line.
//!
//! Claim 2 of the unit: "A protocol with a state a real case reaches and cannot leave to a
//! legitimate outcome gets at least one finding (exit 1)." `fixtures/child-outcome-adversary/
//! ended-once` opens one child once (the guard of `open` closes on its own record), publishes
//! under the capability `licence` once the child ended `accepted`, and declines when it ended
//! otherwise. The case `open`, child `accepted`, `licence` denied is a dead end the evaluator
//! confirms (`crates/canon/tests/adversary_simulate_child_outcome.rs`). The walk lets the one
//! child end a second time, with an unlisted outcome, so it counts that state as waiting and
//! exits 0.

use std::path::PathBuf;
use std::process::{Command, Output};

fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn simulate(path: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(repository_root())
        .args(["simulate", "--path", path])
        .output()
        .expect("the canon binary runs")
}

#[test]
fn a_dead_end_after_the_only_child_ended_exits_one() {
    let output = simulate("fixtures/child-outcome-adversary/ended-once/protocol.yaml");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let reported = stdout
        .lines()
        .any(|line| line.starts_with("stuck-state: ") || line.starts_with("cycle: "));
    assert!(
        output.status.code() == Some(1) && reported,
        "expected exit 1 and a stuck-state or cycle for the state after the only child ended \
         accepted and `licence` was denied; got exit {:?}:\n{stdout}{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}
