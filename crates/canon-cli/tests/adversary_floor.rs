//! Adversary pass 1 on story:protocol-floor (wave 2026-11-20-w9).
//!
//! The unit's own suite (`floor.rs`) never prints a `NO SEMANTIC CHANGE` count other than 0 and
//! never prints more than one line below the floor, so it stays green when the floor refuses a
//! `NO SEMANTIC CHANGE` change, counts one as an `EXPANSION`, or reorders the lines it prints.
//! These cases pin what story § Outcome says of both: the command exits 0 when every change is
//! `TIGHTENING`, `EXPANSION` or `NO SEMANTIC CHANGE`, printing `floor: holds` and the three counts,
//! and otherwise prints each `RELAXATION` and `BREAKING` change "in the classifier's deterministic
//! order". Each protocol is the unit's floor fixture with a few declarations edited, an id and a
//! revision of its own, and each verdict is compared with what `canon diff` reports for the same
//! two documents once they share an id.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FLOOR: &str = "fixtures/investigation/floor/floor.yaml";
const FLOOR_ID: &str = "floor.investigation";

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
        .expect("canon runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("utf-8 output")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adversary_floor")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("stale scratch directory removes");
    }
    std::fs::create_dir_all(&dir).expect("scratch directory creates");
    dir
}

/// The floor fixture's source.
fn floor_source() -> String {
    std::fs::read_to_string(repository_root().join(FLOOR)).expect("the floor fixture reads")
}

/// The floor with each `(old, new)` edit applied once, under the id `probe.variant` revision 9.
fn variant(edits: &[(&str, &str)]) -> String {
    let mut source = floor_source();
    for (old, new) in edits {
        assert_eq!(
            source.matches(old).count(),
            1,
            "`{old}` is in the floor once"
        );
        source = source.replacen(old, new, 1);
    }
    source
        .replacen("id: floor.investigation", "id: probe.variant", 1)
        .replacen("revision: 1\n", "revision: 9\n", 1)
}

/// Compiles `source` into `dir/<name>.json` and returns that path and the IR.
fn compiled(dir: &Path, name: &str, source: &str) -> (String, String) {
    let yaml = dir.join(format!("{name}.yaml"));
    std::fs::write(&yaml, source).expect("the source is written");
    let output = canon(&["compile", "--path", &yaml.display().to_string()]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{name} compiles: {}",
        text(&output.stderr)
    );
    let ir = text(&output.stdout);
    let path = dir.join(format!("{name}.json"));
    std::fs::write(&path, &ir).expect("the IR is written");
    (path.display().to_string(), ir)
}

/// `canon floor --floor <floor> --ir <ir>`: exit status and standard output, standard error empty.
fn floor(floor: &str, ir: &str) -> (Option<i32>, String) {
    let output = canon(&["floor", "--floor", floor, "--ir", ir]);
    assert_eq!(text(&output.stderr), "", "nothing on standard error");
    (output.status.code(), text(&output.stdout))
}

/// The change lines of `canon diff` from the floor to the protocol once the protocol carries the
/// floor's id, without the line about the protocol header (its description differs by design).
fn diff_lines(dir: &Path, floor: &str, ir: &str, protocol_id: &str) -> Vec<String> {
    let renamed = ir.replacen(&format!("\"{protocol_id}\""), &format!("\"{FLOOR_ID}\""), 1);
    let path = dir.join("same-id.json");
    std::fs::write(&path, renamed).expect("the renamed IR is written");
    let output = canon(&["diff", "--from", floor, "--to", &path.display().to_string()]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    text(&output.stdout)
        .lines()
        .skip(1)
        .filter(|line| !line.contains(&format!(": protocol `{FLOOR_ID}`")))
        .map(str::to_owned)
        .collect()
}

/// A protocol that only rewrites descriptions makes two `NO SEMANTIC CHANGE` changes: it stays
/// above the floor, exit 0, and the count says 2 (story § Outcome: "exits 0 when every change is
/// `TIGHTENING`, `EXPANSION` or `NO SEMANTIC CHANGE`", then "the counts of `TIGHTENING`,
/// `EXPANSION` and `NO SEMANTIC CHANGE` changes").
#[test]
fn a_protocol_that_only_rewrites_descriptions_holds_and_counts_them() {
    let dir = scratch("rewrites");
    let (floor_path, _) = compiled(&dir, "floor", &floor_source());
    let (path, ir) = compiled(
        &dir,
        "rewrites",
        &variant(&[
            (
                "description: Look for observations bearing on the explanation.",
                "description: Search for observations that bear on the explanation.",
            ),
            (
                "description: The explanation is supported.\n",
                "description: The explanation holds.\n",
            ),
        ]),
    );
    assert_eq!(
        diff_lines(&dir, &floor_path, &ir, "probe.variant"),
        [
            "NO SEMANTIC CHANGE: action `inspect` description changed",
            "NO SEMANTIC CHANGE: outcome `supported` description changed",
        ],
        "the classifier reports two description changes"
    );
    assert_eq!(
        floor(&floor_path, &path),
        (
            Some(0),
            "floor: holds\n\
             TIGHTENING: 0\n\
             EXPANSION: 0\n\
             NO SEMANTIC CHANGE: 2\n"
                .to_owned()
        )
    );
}

/// Two tightenings, one expansion and one rewrite: the protocol holds, and each count is the
/// number of the classifier's changes of that category.
#[test]
fn each_count_is_the_classifiers_number_of_that_category() {
    let dir = scratch("mixed");
    let (floor_path, _) = compiled(&dir, "floor", &floor_source());
    let (path, ir) = compiled(
        &dir,
        "mixed",
        &variant(&[
            (
                "  falsification_attempt:\n    description: An attempt",
                "  peer_review:\n    description: A peer's review.\n  falsification_attempt:\n    description: An attempt",
            ),
            (
                "      - capability: finding.publish\n",
                "      - capability: finding.publish\n      - capability: finding.sign\n",
            ),
            (
                "\nactions:\n",
                "\nobligations:\n  peer.reviewed:\n    discharged_when:\n      claim: explanation.supported\n\nactions:\n",
            ),
            (
                "description: Try to refute the explanation.",
                "description: Try hard to refute the explanation.",
            ),
        ]),
    );
    assert_eq!(
        diff_lines(&dir, &floor_path, &ir, "probe.variant"),
        [
            "EXPANSION: evidence kind `peer_review` added",
            "TIGHTENING: obligation `peer.reviewed` added",
            "NO SEMANTIC CHANGE: action `attempt_falsification` description changed",
            "TIGHTENING: action `publish_finding` now requires capability `finding.sign`",
        ],
        "the classifier's changes"
    );
    assert_eq!(
        floor(&floor_path, &path),
        (
            Some(0),
            "floor: holds\n\
             TIGHTENING: 2\n\
             EXPANSION: 1\n\
             NO SEMANTIC CHANGE: 1\n"
                .to_owned()
        )
    );
}

/// Three changes below the floor in three sections, among changes that are not: the floor prints
/// exactly the `RELAXATION` and `BREAKING` lines of the classifier, in its order (claims, then
/// actions by identifier, then outcomes), and exits 1. Swapped, a relaxation of the floor is a
/// tightening of the protocol and the breaking change stays breaking.
#[test]
fn several_changes_below_the_floor_print_in_the_classifiers_order() {
    let dir = scratch("several");
    let (floor_path, floor_ir) = compiled(&dir, "floor", &floor_source());
    let (path, ir) = compiled(
        &dir,
        "several",
        &variant(&[
            (
                "    true_when:\n      all:\n",
                "    true_when:\n      any:\n",
            ),
            (
                "    description: Try to refute the explanation.\n",
                "    description: Try to refute the explanation.\n    revises: [explanation]\n",
            ),
            ("    requires:\n      - capability: finding.publish\n", ""),
            (
                "    requires:\n      claim: explanation.supported\n",
                "    requires:\n      all:\n        - claim: explanation.supported\n        - evidence:\n            kind: supporting_observation\n",
            ),
            (
                "  inspect:\n    description: Look",
                "  inspect:\n    description: Now look",
            ),
        ]),
    );
    let below = [
        "RELAXATION: claim `explanation.supported` predicate weakened",
        "RELAXATION: action `attempt_falsification` now revises artifact `explanation`",
        "RELAXATION: action `publish_finding` no longer requires capability `finding.publish`",
        "BREAKING: outcome `supported` requirement changed",
    ];
    let classified: Vec<String> = diff_lines(&dir, &floor_path, &ir, "probe.variant")
        .into_iter()
        .filter(|line| line.starts_with("RELAXATION: ") || line.starts_with("BREAKING: "))
        .collect();
    assert_eq!(classified, below, "the classifier's lines below the floor");
    let mut expected = String::from("floor: breached\n");
    for line in below {
        expected.push_str(line);
        expected.push('\n');
    }
    assert_eq!(floor(&floor_path, &path), (Some(1), expected));

    // The floor is the protocol now, and the protocol the floor.
    let swapped = dir.join("swapped-floor.json");
    std::fs::write(&swapped, &ir).expect("written");
    let protocol = dir.join("swapped-protocol.json");
    std::fs::write(&protocol, &floor_ir).expect("written");
    assert_eq!(
        floor(
            &swapped.display().to_string(),
            &protocol.display().to_string()
        ),
        (
            Some(1),
            "floor: breached\n\
             BREAKING: outcome `supported` requirement changed\n"
                .to_owned()
        )
    );
}
