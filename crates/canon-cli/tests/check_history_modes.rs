//! story:check-descended-two-result-history, correction of finding F6 of the review of wave
//! 2026-11-22-w11: the history modes of a kind a `descended` form and another order form both read
//! are counted for the earlier sets they change only.
//!
//! The two modes `LOW_FIRST` and `HIGH_FIRST` of `crates/canon/src/check/space.rs` observe the
//! earlier records of two or more classes at two instants. For a history without an earlier
//! record, or with the records of one class, they hold the records of the `NAMED` value, so the
//! values are not repeated: a kind of three classes has 35 history values, not 45.
//!
//! The protocol below was checked by canon 0.11.1: 41472 states, 0 findings, exit 0. With every
//! value repeated in the two modes the check refused it (`state-space-bound`, 69120 states); with
//! none repeated it has 53760 states, under the bound of 65536.

use std::path::{Path, PathBuf};
use std::process::Command;

const PROTOCOL: &str = "format: protocol/1\n\
    protocol: {id: p, revision: 1}\n\
    case: {inputs: {track: {values: [a, b, c]}}}\n\
    artifacts: {a: {}}\n\
    evidence_kinds: {k: {}, x0: {}, x1: {}, x2: {}, x3: {}, x4: {}, x5: {}}\n\
    claims:\n\
    \x20 on_track: {true_when: {input: track, in: [a]}}\n\
    \x20 red_then_green: {true_when: {descended: {first: {kind: k, result: failed}, \
    then: {kind: k, result: passed}}}}\n\
    \x20 first_pass: {true_when: {first: {kind: k, result: passed}}}\n\
    \x20 c0: {true_when: {evidence: {kind: x0}}}\n\
    \x20 c1: {true_when: {evidence: {kind: x1}}}\n\
    \x20 c2: {true_when: {evidence: {kind: x2}}}\n\
    \x20 c3: {true_when: {evidence: {kind: x3}}}\n\
    \x20 c4: {true_when: {evidence: {kind: x4}}}\n\
    \x20 c5: {true_when: {evidence: {kind: x5}}}\n\
    actions: {run: {may_produce: [{evidence: k}, {evidence: x0}, {evidence: x1}, {evidence: x2}, \
    {evidence: x3}, {evidence: x4}, {evidence: x5}]}}\n\
    outcomes: {done: {requires: {all: [{claim: red_then_green}, {claim: first_pass}, \
    {claim: on_track}, {claim: c0}, {claim: c1}, {claim: c2}, {claim: c3}, {claim: c4}, \
    {claim: c5}]}}}\n";

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("check-history-modes")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory creates");
    dir
}

/// A protocol canon 0.11.1 checks is checked, over the 53760 states that remain once the two
/// modes hold only the earlier sets of two or more classes. Red: `error[state-space-bound]`
/// naming 69120 states.
#[test]
fn a_protocol_0_11_1_checks_is_not_refused_for_history_values_that_repeat_a_state() {
    let path = scratch("repeated-histories").join("protocol.yaml");
    std::fs::write(&path, PROTOCOL).expect("the protocol is written");
    let output = Command::new(env!("CARGO_BIN_EXE_canon"))
        .args(["check", "--path", &path.display().to_string()])
        .output()
        .expect("canon runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success() && stderr.is_empty(),
        "canon 0.11.1 checks this protocol (41472 states, exit 0), its history dimension 27 \
         values; it is refused with the values of LOW_FIRST and HIGH_FIRST that hold the records \
         of a NAMED one counted:\n{stderr}"
    );
    assert_eq!(
        stdout,
        "checked: protocol `p` revision 1: 53760 states, 0 properties, 0 findings\n"
    );
}
