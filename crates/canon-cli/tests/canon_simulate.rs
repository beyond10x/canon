//! Acceptance for story:canon-simulate: `canon simulate` walks every state a case can reach from
//! the empty case, one step at a time, and reports stuck states with the shortest path to each,
//! cycles, dead actions and unreached outcomes.
//!
//! Seven expectations, over the fixture directories in `fixtures/investigation/simulate/`:
//!
//! 1. `clean` exits 0 and prints only the summary line, with its reachable state count and the
//!    terminable count per outcome.
//! 2. `mutual`, two actions each needing evidence only the other produces and an outcome needing
//!    both: `canon check` exits 0; `canon simulate` exits 1 with `stuck-state` for `{}` and an empty
//!    path, `dead-action` for both actions, `unreached-outcome` and `no-outcome-reachable`.
//! 3. `denied`, one action that requires capability `sign`: `stuck-state` for
//!    `` {capability `sign` denied} `` after `` capability `sign` denied ``, and none for `{}`,
//!    where the action is `approval-required`.
//! 4. `cycle`: after a denial the one admissible action may produce nothing, so `cycle` names
//!    that state, its shortest path and the action.
//! 5. `ordered`, a protocol with an order form, is refused as `ordered-evidence`, exit 1.
//! 6. `reachable-state-space-bound`, more than 65536 reachable states, is refused as
//!    `reachable-state-bound`, exit 1; `state-space-bound`, whose upstream variants put its space
//!    above the bound so that `canon check` refuses it as `state-space-bound`, is simulated and
//!    exits 0.
//! 7. Two runs give identical bytes. (`every_canon_subcommand_is_declared_in_ess`, in
//!    `ess_commands_match.rs`, holds the declaration of the command in `ess/domains/cli.yaml`.)
//!
//! A kind that declares `supplied_by` is produced outside the protocol: a record of it may arrive
//! at any step, so a state a supplied record leads out of is not stuck, and the summary counts it
//! as waiting (`supplied` against `unsupplied`, the same protocol without the declaration).
//!
//! The story's rules refined after adversary pass 1 (R1 to R6) are held by
//! `simulate_follows_the_refined_rules`: an expiry step, the open-state rule for `stuck-state`
//! and `cycle`, a decision named on a finding, and one finding per smallest state with a count.
//! Those refined after adversary pass 2 (R2a, an expired child outcome) by
//! `simulate_follows_the_rules_refined_after_pass_2`; R7, the line naming the limit on child
//! outcomes, was removed by story:simulate-child-outcome-step, whose child-outcome step those two
//! fixtures also take.
//!
//! The revise step of story:simulate-revise-step, which moves an artifact an admissible action
//! declares in `revises` to a new revision, is held by `simulate_revises_declared_artifacts` over
//! the six `revise*` fixtures.
//!
//! Each fixture directory carries `protocol.yaml` and the exact output: `report.txt` on standard
//! output with nothing on standard error, or `refusal.txt` on standard error with nothing on
//! standard output. Both were written by hand from the story before `canon simulate` existed.

use std::path::PathBuf;
use std::process::{Command, Output};

/// The fixture family, relative to the repository root.
const FIXTURES: &str = "fixtures/investigation/simulate";

/// Where `canon simulate` writes the expected text.
#[derive(Clone, Copy, Debug)]
enum Expected {
    /// `report.txt` on standard output; standard error empty.
    Report,
    /// `refusal.txt` on standard error; standard output empty.
    Refusal,
}

/// Every fixture directory, the exit status and where the expected text goes. A walk that finds
/// nothing exits 0; findings and a refusal exit 1 (`REJECTED`).
const CASES: [(&str, i32, Expected); 29] = [
    ("clean", 0, Expected::Report),
    ("mutual", 1, Expected::Report),
    ("denied", 1, Expected::Report),
    ("cycle", 1, Expected::Report),
    ("ordered", 1, Expected::Refusal),
    ("reachable-state-space-bound", 1, Expected::Refusal),
    ("state-space-bound", 0, Expected::Report),
    ("supplied", 0, Expected::Report),
    ("unsupplied", 1, Expected::Report),
    ("expiry", 0, Expected::Report),
    ("renew-loop", 1, Expected::Report),
    ("decision-exit", 1, Expected::Report),
    ("contained", 1, Expected::Report),
    ("unread-supplied", 1, Expected::Report),
    ("clean-result", 0, Expected::Report),
    ("child-expiry", 1, Expected::Report),
    ("child-expiry-held", 1, Expected::Report),
    ("revise", 0, Expected::Report),
    ("revise-undeclared", 1, Expected::Report),
    ("revise-upstream", 0, Expected::Report),
    ("revise-denied", 1, Expected::Report),
    ("revise-mixed", 0, Expected::Report),
    ("revise-child", 0, Expected::Report),
    ("revised-by", 0, Expected::Report),
    ("revised-by-undeclared", 1, Expected::Report),
    ("revise-child-current", 0, Expected::Report),
    ("revise-child-other-subject", 1, Expected::Report),
    ("revise-child-two-subjects", 1, Expected::Report),
    ("revise-child-late", 1, Expected::Report),
];

/// The cases of story:child-outcome-recovery, expectations 1 to 4.
const RECOVERY: [&str; 4] = [
    "revise-child-current",
    "revise-child-other-subject",
    "revise-child-two-subjects",
    "revise-child-late",
];

/// The cases of story:simulate-revise-step, expectations 1 to 6.
const REVISE: [&str; 6] = [
    "revise",
    "revise-undeclared",
    "revise-upstream",
    "revise-denied",
    "revise-mixed",
    "revise-child",
];

/// The cases of the rules refined after adversary pass 2 (R2a and R7).
const CHILDREN: [&str; 2] = ["child-expiry", "child-expiry-held"];

/// The cases of the rules refined after adversary pass 1 (the story's last section, R1 to R6).
const REFINED: [&str; 6] = [
    "expiry",
    "renew-loop",
    "decision-exit",
    "contained",
    "unread-supplied",
    "clean-result",
];

/// The cases of the named test, expectations 1 to 7.
const ACCEPTANCE: [&str; 7] = [
    "clean",
    "mutual",
    "denied",
    "cycle",
    "ordered",
    "reachable-state-space-bound",
    "state-space-bound",
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

fn protocol(case: &str) -> String {
    format!("{FIXTURES}/{case}/protocol.yaml")
}

/// Runs `canon simulate` over one fixture directory.
fn simulate(case: &str) -> Output {
    canon(&["simulate", "--path", &protocol(case)])
}

/// The largest fixture (3160 states, about 4 s in a debug build): `failures` walks it once, and
/// `simulate_walks_the_largest_recovery_fixture_the_same_twice` holds its second run to the first.
const WALKED_ONCE: [&str; 1] = ["revise-child-two-subjects"];

/// Runs each of `cases` twice, except those of `WALKED_ONCE`, and returns what differs from its
/// expectation.
fn failures(cases: &[&str]) -> Vec<String> {
    let mut failures = Vec::new();
    for case in cases {
        let (_, status, expected) = CASES
            .iter()
            .find(|(name, ..)| name == case)
            .expect("a named case");
        let first = simulate(case);
        let (stdout, stderr) = match expected {
            Expected::Report => (
                read(&format!("{FIXTURES}/{case}/report.txt")),
                String::new(),
            ),
            Expected::Refusal => (
                String::new(),
                read(&format!("{FIXTURES}/{case}/refusal.txt")),
            ),
        };
        if first.status.code() != Some(*status)
            || text(&first.stdout) != stdout
            || text(&first.stderr) != stderr
        {
            failures.push(format!(
                "{case}: expected exit {status}, stdout {stdout:?}, stderr {stderr:?}; \
                 got exit {:?}, stdout {:?}, stderr {:?}",
                first.status.code(),
                text(&first.stdout),
                text(&first.stderr),
            ));
            continue;
        }
        if WALKED_ONCE.contains(case) {
            continue;
        }
        let second = simulate(case);
        if (second.status.code(), &second.stdout, &second.stderr)
            != (first.status.code(), &first.stdout, &first.stderr)
        {
            failures.push(format!("{case}: a second run gave different bytes"));
        }
    }
    failures
}

#[test]
fn simulate_reports_reachability_findings() {
    let mut failures = failures(&ACCEPTANCE);

    // 2: the static check finds nothing in `mutual`; only the walk does.
    let checked = canon(&["check", "--path", &protocol("mutual")]);
    if checked.status.code() != Some(0) {
        failures.push(format!(
            "mutual: canon check exits {:?}: {}{}",
            checked.status.code(),
            text(&checked.stdout),
            text(&checked.stderr)
        ));
    }

    // 6: `canon check` refuses `state-space-bound`, which the walk simulates.
    let checked = canon(&["check", "--path", &protocol("state-space-bound")]);
    let stderr = text(&checked.stderr);
    if checked.status.code() != Some(1)
        || !stderr
            .starts_with("error[state-space-bound]: protocol `simulate.upstream` revision 1: ")
    {
        failures.push(format!(
            "state-space-bound: canon check exits {:?}, stderr {stderr:?}",
            checked.status.code()
        ));
    }

    assert!(
        failures.is_empty(),
        "{} expectations failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// A record of a kind that declares `supplied_by` may arrive at any step, so a state that waits
/// only on one is counted as waiting, not reported as stuck; without the declaration nothing
/// produces the kind and the same protocol is stuck at `{}`.
#[test]
fn simulate_lets_a_supplied_record_arrive_at_any_step() {
    let failures = failures(&["supplied", "unsupplied"]);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The rules refined after adversary pass 1:
///
/// - `expiry`: an action admissible only after a `max_age` record expires is reached by the
///   expiry step (R2), and the state that waits for the expiry is counted as waiting (R3).
/// - `renew-loop`: a record renewed and expiring forever, while the outcome needs evidence nothing
///   produces, is a `cycle` at the state in which renewing returns the same state (R2, R3, R6).
/// - `decision-exit`: a stuck state a decision makes terminable is still stuck, and the finding
///   names the decision and its outcome (R4).
/// - `contained`: one denial strands the case beside any of three memo states; one finding is
///   printed, at the smallest state, with the count of the others (R5).
/// - `unread-supplied`: a supplied kind nothing reads leaves `{}` stuck, with an empty path, and
///   no state waits (R3, R5).
/// - `clean-result`: a protocol that reads a `result:` is clean when no action may record a
///   second, different result beside a fresh one.
#[test]
fn simulate_follows_the_refined_rules() {
    let failures = failures(&REFINED);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The rules refined after adversary pass 2, on a `case_outcome` kind that declares `max_age`:
///
/// - `child-expiry`: only a `children` form reads `case_outcome`, which an expiry does not undo,
///   so there is no expiry step for it (R2a). Without one, a child that ended with an outcome the
///   form does not list keeps the outcome illegitimate: the `cycle` and the `stuck-state` are
///   real. A child's outcome arrives by the child-outcome step of
///   story:simulate-child-outcome-step, which counts the children of `open_change` not ended:
///   the `stuck-state` holds none, after two children ended, and the `cycle` two or more, where a
///   step of `open_change` leaves the count where it is.
/// - `child-expiry-held`: an evidence match reads `case_outcome` too, so an expired record is
///   held as expired: absent to the match, present to the `children` form. `{lock}` with an
///   expired accepted outcome is terminable, as the evaluator says past the `max_age` (R2a).
#[test]
fn simulate_follows_the_rules_refined_after_pass_2() {
    let failures = failures(&CHILDREN);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// story:simulate-revise-step: an admissible action may move each artifact it declares in
/// `revises` to a new revision, alone or with one record or one group it may produce, in one step.
///
/// 1. `revise`: a test of `doc` that did not pass is set aside by `fix`, which revises `doc` and
///    records nothing, and `doc` is tested again: exit 0, the summary line only.
/// 2. `revise-undeclared`, the same protocol without `revises`: the `cycle` naming `fix`, the
///    bytes the walk gave before the revise step.
/// 3. `revise-upstream`: a test of `b` under an invalidation rule with upstream `a`; `fix` revises
///    `a`, which keeps the failed test from the claim, so `b` is tested again: exit 0. Without the
///    change to "observed before upstream `a` moved" the revise step would change nothing and the
///    state would be a `cycle`.
/// 4. `revise-denied`: approving needs a denied capability; the case revision does not move, so
///    the denial lasts, and exactly one `cycle` is printed, ending with the revision suffix.
/// 5. `revise-mixed`: two different results of one approval leave it unknown; revising the
///    document sets both aside: no finding.
/// 6. `revise-child`: a child's `accepted` outcome stays read by the `children` form after its
///    subject is revised, as an earlier record: exit 0, where a revise step that dropped the
///    record would leave a `cycle`.
///
/// Two runs of each give identical bytes (`failures`); the fixtures without `revises` are held
/// unchanged by the other tests of this file (expectation 7).
#[test]
fn simulate_revises_declared_artifacts() {
    let failures = failures(&REVISE);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// story:child-outcome-recovery: the revise step sets aside only the child outcomes about the
/// artifact it revises, and a child that ends after its subject was revised may end against the
/// earlier revision. Every fixture declares `a` and `b`; action `open` opens a child case about
/// `b`; claim `done.current` is `{children: {in: [done], revision: current}}`; action `fix` is
/// admissible while it is false; outcome `built` requires it.
///
/// 1. `revise-child-current`, `fix` revises `b`: exit 0, the summary line only. The failed child's
///    outcome, about `b`, becomes an earlier one, the claim is unknown again, and a new child ends
///    `done`.
/// 2. `revise-child-other-subject`, `fix` revises `a`: exit 1, one finding, whose state holds the
///    failed outcome as a current one.
/// 3. `revise-child-two-subjects`, `open_a` opens a child about `a` too, `fix` revises `b`: a
///    failed child of `open` is in no printed state, a failed child of `open_a` is in exactly one.
/// 4. `revise-child-late`: a path holds a child's outcome written `earlier` after
///    `` action `fix` revises `b` ``.
///
/// The text of each is held byte for byte; two runs give the same bytes (`failures`, and for the
/// largest fixture `simulate_walks_the_largest_recovery_fixture_the_same_twice`).
#[test]
fn simulate_sets_aside_child_outcomes_of_the_revised_subject() {
    let mut failures = failures(&RECOVERY);

    // `failures` walked each case and held its output to `report.txt`; the properties
    // below read that file, so no walk is repeated for them.
    let report = |case: &str| read(&format!("{FIXTURES}/{case}/report.txt"));
    let current = report("revise-child-current");
    let findings = |report: &str| {
        report
            .lines()
            .filter(|line| !line.starts_with("simulated:"))
            .count()
    };
    if findings(&current) != 0 {
        failures.push(format!("revise-child-current: findings in {current:?}"));
    }
    let other = report("revise-child-other-subject");
    if findings(&other) != 1 || !other.contains("`case_outcome` about `b`") {
        failures.push(format!(
            "revise-child-other-subject: expected one finding holding a current record about b: {other:?}"
        ));
    }
    let two = report("revise-child-two-subjects");
    let printed: Vec<&str> = two
        .lines()
        .filter(|line| line.contains("state {"))
        .collect();
    // A failed child's record is the one without a result.
    let about = |subject: &str| {
        let failed = format!("evidence `case_outcome` about `{subject}`");
        printed
            .iter()
            .filter(|line| {
                let state = line.split_once("state {").map_or("", |(_, rest)| rest);
                let state = state.split_once("}:").map_or(state, |(state, _)| state);
                state.split(", ").any(|item| item == failed)
            })
            .count()
    };
    if about("b") != 0 || about("a") != 1 {
        failures.push(format!(
            "revise-child-two-subjects: a failed child of open in {} printed states, of open_a in {}: {two:?}",
            about("b"),
            about("a")
        ));
    }
    let late = report("revise-child-late");
    let revise = "action `fix` revises `b`";
    let path_has_earlier = late.lines().any(|line| {
        line.split_once("path: ").is_some_and(|(_, path)| {
            path.find(revise)
                .is_some_and(|at| path[at..].contains(" earlier"))
        })
    });
    if !path_has_earlier {
        failures.push(format!(
            "revise-child-late: no path with an earlier record after `fix`: {late:?}"
        ));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A second, independent run of the largest recovery fixture gives the bytes of `report.txt`, to
/// which the one walk of `simulate_sets_aside_child_outcomes_of_the_revised_subject` holds the
/// first: the two runs give the same bytes.
#[test]
fn simulate_walks_the_largest_recovery_fixture_the_same_twice() {
    for case in WALKED_ONCE {
        let output = simulate(case);
        assert_eq!(
            text(&output.stdout),
            read(&format!("{FIXTURES}/{case}/report.txt")),
            "{case}: a second run gave different bytes"
        );
        assert_eq!(text(&output.stderr), "", "{case}");
    }
}

/// The red run must not come from a fixture typo: every fixture protocol is one `canon validate`
/// accepts.
#[test]
fn every_simulate_fixture_protocol_validates() {
    for (case, ..) in CASES {
        let output = canon(&["validate", "--path", &protocol(case)]);
        assert!(output.status.success(), "{case}: {}", text(&output.stderr));
    }
}

/// Every fixture directory is a case, so none is skipped silently, and each carries exactly the
/// files its expectation reads.
#[test]
fn every_simulate_fixture_is_a_case() {
    let entries = |dir: PathBuf| {
        let mut names: Vec<String> = std::fs::read_dir(dir)
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
    let mut named: Vec<String> = CASES.iter().map(|(case, ..)| (*case).to_owned()).collect();
    named.sort();
    assert_eq!(entries(repository_root().join(FIXTURES)), named);
    for (case, _, expected) in CASES {
        let mut wanted = vec![
            "protocol.yaml".to_owned(),
            match expected {
                Expected::Report => "report.txt",
                Expected::Refusal => "refusal.txt",
            }
            .to_owned(),
        ];
        wanted.sort();
        assert_eq!(
            entries(repository_root().join(FIXTURES).join(case)),
            wanted,
            "{case}"
        );
    }
}
