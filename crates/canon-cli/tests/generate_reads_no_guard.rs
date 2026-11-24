//! Finding W14-2 of wave 2026-11-23-w14: `canon generate` never reads `runs_only_when`
//! (`website/docs/concepts/protocols.md`, "Ordering guards"; `ess/domains/protocol.yaml`), so its
//! output and exit status are the same with or without a guard, while `canon check` still decides
//! the guard and refuses one whose slice is above the bound.
//!
//! The protocol: ten evidence kinds, `done` and `k1` to `k9`, each produced by an action, so the
//! protocol's own space is 2^10 = 1024 states. The guard of `finish` names two results of each of
//! `k1` to `k9` that no other predicate names, so the space `canon check` decides the guard over
//! gives each of those kinds 8 values, and the guard's slice is above the bound of 65536.
//!
//! The case `generate_gives_the_same_output_with_a_guard_whose_slice_is_above_the_bound` in
//! `adversary_w14_wave.rs` uses a protocol of 18 kinds, whose own space of 262144 states is above
//! the bound without any guard, so `canon generate` refuses it with and without the guard; this
//! file holds the case with a protocol within the bound.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn canon_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("the canon binary runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// An empty directory of its own under the target's scratch directory.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("generate-reads-no-guard")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

/// The protocol above, with `guard` as the `runs_only_when` line of `finish` (empty for none).
fn source(guard: &str) -> String {
    let kinds: String = (1..=9).map(|n| format!("  k{n}: {{}}\n")).collect();
    let makers: String = (1..=9)
        .map(|n| format!("  make{n}: {{may_produce: [{{evidence: k{n}}}]}}\n"))
        .collect();
    format!(
        "format: protocol/1\nprotocol: {{id: gen.classes, revision: 1}}\n\
         artifacts: {{a: {{}}}}\nevidence_kinds:\n  done: {{}}\n{kinds}\
         claims:\n  done: {{true_when: {{evidence: {{kind: done}}}}}}\n\
         actions:\n  finish:\n    precondition: {{claim: done, is: unknown}}\n{guard}\
         \x20   may_produce: [{{evidence: done}}]\n{makers}\
         outcomes:\n  finished: {{requires: {{claim: done}}}}\n"
    )
}

/// The guard: `x` or `y` of any of `k1` to `k9`.
fn guard() -> String {
    let any: Vec<String> = (1..=9)
        .flat_map(|n| {
            ["x", "y"].map(|result| format!("{{evidence: {{kind: k{n}, result: {result}}}}}"))
        })
        .collect();
    format!("    runs_only_when: {{any: [{}]}}\n", any.join(", "))
}

/// Exit status, standard output and standard error of `canon generate` on `source`, and the
/// scenario files it wrote, by name.
fn generated(name: &str, source: &str) -> (Option<i32>, String, String, Vec<(String, String)>) {
    let dir = scratch(name);
    std::fs::write(dir.join("protocol.yaml"), source).expect("written");
    let output = canon_in(
        &dir,
        &["generate", "--path", "protocol.yaml", "--out", "out"],
    );
    let mut files: Vec<(String, String)> = std::fs::read_dir(dir.join("out"))
        .map(|entries| {
            entries
                .map(|entry| {
                    let entry = entry.expect("an entry");
                    (
                        entry.file_name().to_string_lossy().into_owned(),
                        std::fs::read_to_string(entry.path()).expect("a scenario reads"),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    (
        output.status.code(),
        text(&output.stdout),
        text(&output.stderr),
        files,
    )
}

/// `canon generate` gives the same bytes and exit status with and without a guard whose slice is
/// above the bound, for a protocol whose own space is within it.
#[test]
fn generate_gives_the_same_output_with_a_guard_whose_slice_alone_is_above_the_bound() {
    let without = generated("without", &source(""));
    assert_eq!(without.0, Some(0), "{without:?}");
    assert_eq!(without.3.len(), 2, "{without:?}");
    let with = generated("with", &source(&guard()));
    assert_eq!(with, without);
}

/// `canon check` still decides the guard: the guarded protocol is refused, naming the guard, and
/// the protocol without it checks clean over its 1024 states.
#[test]
fn check_still_refuses_the_guard_whose_slice_is_above_the_bound() {
    let dir = scratch("check");
    std::fs::write(dir.join("without.yaml"), source("")).expect("written");
    std::fs::write(dir.join("with.yaml"), source(&guard())).expect("written");
    let without = canon_in(&dir, &["check", "--path", "without.yaml"]);
    assert_eq!(without.status.code(), Some(0), "{}", text(&without.stderr));
    assert_eq!(
        text(&without.stdout),
        "checked: protocol `gen.classes` revision 1: 1024 states, 0 properties, 0 findings\n"
    );
    let with = canon_in(&dir, &["check", "--path", "with.yaml"]);
    assert_eq!(with.status.code(), Some(1));
    assert_eq!(text(&with.stdout), "");
    assert!(
        text(&with.stderr).starts_with(
            "error[state-space-bound]: protocol `gen.classes` revision 1: action `finish` \
             runs_only_when reads 268435456 states, more than the bound of 65536: "
        ),
        "{}",
        text(&with.stderr)
    );
}
