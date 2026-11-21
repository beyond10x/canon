//! Adversary pass 1 for story:simulate-record-groups, from the command line.
//!
//! § Outcome 4 of the story fixes the order of an action's steps: "single-kind steps come first
//! in today's order. Group steps follow, in order of each group's least kind id", and within a
//! group "the members in kind-id order and the last member varying fastest"; "Breadth-first
//! search ... still reaches each state first along a shortest path". § Outcome 3: "A group step
//! has the product of its members' successor counts, one successor per combination."
//!
//! The order of an action's successors shows only in which of two equally short paths the walk
//! prints, so each fixture in `fixtures/record-group-adversary/` has a printed state that two
//! paths of equal length reach, and the test pins the one the order makes first. None of the
//! unit's fixtures has an action with both a grouped and an ungrouped kind, with two groups, or
//! with such a state.

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

fn simulate(case: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(repository_root())
        .args([
            "simulate",
            "--path",
            &format!("fixtures/record-group-adversary/{case}/protocol.yaml"),
        ])
        .output()
        .expect("the canon binary runs")
}

/// Exit 1, nothing on standard error; standard output.
fn walked(case: &str) -> String {
    let output = simulate(case);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert_eq!(String::from_utf8_lossy(&output.stderr), "", "{case}");
    assert_eq!(output.status.code(), Some(1), "{case}: {stdout}");
    stdout
}

/// `act` may produce `a`, `b` and `c` and groups `[a, c]`; its guard is open while `a` or `b` has
/// no record. `{a, b, c}` is reached from `{b}` by the group step and from `{a, c}` by the
/// single step of `b`, both at depth 2. Single-kind steps come first, so `{b}` is reached first,
/// although the group's least kind `a` sorts before `b`.
#[test]
fn single_kind_steps_come_before_group_steps() {
    assert_eq!(
        walked("single-first"),
        "stuck-state: state {evidence `a` about `doc`, evidence `b` about `doc`, evidence `c` \
         about `doc`}: no outcome is legitimate and no action is admissible or \
         approval-required; path: action `act` produces evidence `b` about `doc`, then action \
         `act` produces evidence `a` about `doc` and evidence `c` about `doc`\n\
         unreached-outcome: outcome `done` is legitimate in no reachable state\n\
         no-outcome-reachable: protocol `adversary.single` revision 1 reaches no state in which \
         an outcome is legitimate\n\
         simulated: protocol `adversary.single` revision 1: 4 reachable states, 0 terminable \
         (outcome `done` 0), 3 findings (1 stuck or cycle state)\n"
    );
}

/// `act` declares the groups `[c, b]` and `[d, a]`, in that order; its guard is open while `a` or
/// `b` has no record. `{a, b, c, d}` is reached from `{a, d}` and from `{b, c}`, both at depth
/// 2. Groups step in order of their least kind, so `[a, d]` comes first.
#[test]
fn groups_step_in_order_of_their_least_kind() {
    assert_eq!(
        walked("group-order"),
        "stuck-state: state {evidence `a` about `doc`, evidence `b` about `doc`, evidence `c` \
         about `doc`, evidence `d` about `doc`}: no outcome is legitimate and no action is \
         admissible or approval-required; path: action `act` produces evidence `a` about `doc` \
         and evidence `d` about `doc`, then action `act` produces evidence `b` about `doc` and \
         evidence `c` about `doc`\n\
         unreached-outcome: outcome `done` is legitimate in no reachable state\n\
         no-outcome-reachable: protocol `adversary.order` revision 1 reaches no state in which \
         an outcome is legitimate\n\
         simulated: protocol `adversary.order` revision 1: 4 reachable states, 0 terminable \
         (outcome `done` 0), 3 findings (1 stuck or cycle state)\n"
    );
}

/// The group `[a, b]`, `a` of the classes `x` and another, `b` of `y` and another. The
/// combinations from the empty state, the last member varying fastest: (x, y), (x, other),
/// (other, y), (other, other). The guard closes in (x, y) only. The state holding all four
/// classes, a `cycle`, is reached at depth 2 from (x, other), (other, y) and (other, other);
/// (x, other) is first, so its path is (x, other) then (other, y). With the first member
/// varying fastest it would be (other, y) then (x, other).
#[test]
fn the_last_member_of_a_group_varies_fastest() {
    let stdout = walked("fastest");
    let cycle = stdout
        .lines()
        .find(|line| line.starts_with("cycle: "))
        .unwrap_or_else(|| panic!("a cycle is printed:\n{stdout}"));
    assert_eq!(
        cycle.split_once("; path: ").map(|(_, path)| path),
        Some(
            "action `act` produces evidence `a` about `doc` result `x` and evidence `b` about \
             `doc`, then action `act` produces evidence `a` about `doc` and evidence `b` about \
             `doc` result `y`"
        ),
        "{stdout}"
    );
    assert!(
        stdout.ends_with(
            "simulated: protocol `adversary.fastest` revision 1: 10 reachable states, 0 \
             terminable (outcome `done` 0), 4 findings (2 stuck or cycle states)\n"
        ),
        "{stdout}"
    );
}

/// `act`: `f` in no group (2 classes), the group `[a, b, c]` (2 x 2 x 1 combinations) and the
/// group `[d, e]` (2 x 2); its guard closes on its first run. From the empty state: 2 + 4 + 4
/// successors, each a stuck state, so 11 reachable states and 10 stuck states; every
/// combination of `[a, b, c]` is one path line naming the three records in kind-id order.
#[test]
fn a_group_step_has_the_product_of_its_members_successors() {
    let stdout = walked("count");
    assert!(
        stdout.ends_with(
            "simulated: protocol `adversary.count` revision 1: 11 reachable states, 0 terminable \
             (outcome `done` 0), 12 findings (10 stuck or cycle states)\n"
        ),
        "{stdout}"
    );
    let paths: Vec<&str> = stdout
        .lines()
        .filter_map(|line| line.split_once("; path: ").map(|(_, path)| path))
        .collect();
    for path in [
        "action `act` produces evidence `a` about `doc` result `ok` and evidence `b` about `doc` \
         result `ok` and evidence `c` about `doc`",
        "action `act` produces evidence `a` about `doc` result `ok` and evidence `b` about `doc` \
         and evidence `c` about `doc`",
        "action `act` produces evidence `a` about `doc` and evidence `b` about `doc` result `ok` \
         and evidence `c` about `doc`",
        "action `act` produces evidence `a` about `doc` and evidence `b` about `doc` and \
         evidence `c` about `doc`",
        "action `act` produces evidence `d` about `doc` result `ok` and evidence `e` about `doc` \
         result `ok`",
        "action `act` produces evidence `d` about `doc` and evidence `e` about `doc`",
        "action `act` produces evidence `f` about `doc` result `ok`",
        "action `act` produces evidence `f` about `doc`",
    ] {
        assert!(paths.contains(&path), "{path}\n{stdout}");
    }
    assert_eq!(paths.len(), 10, "{stdout}");
}
