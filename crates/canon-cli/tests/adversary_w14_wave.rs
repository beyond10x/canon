//! Adversary review of wave 2026-11-23-w14 (stories ordering-guards-provable-by-check,
//! supplied-evidence-required-at-open and children-form-counts-unopened-children), against the
//! command line. Each case asserts what the wave's own documents say.

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
        .join("adversary-w14-wave")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

/// Exit status, standard output and standard error of `canon generate` run on `source`, written
/// as `protocol.yaml` in a directory of its own, and the scenario files it wrote, by name.
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

/// Three readers about `sources`, each opened once (each opener needs its own record about
/// `marks` unknown, which no revision sets aside), an action that revises `sources`, and two
/// counting forms over the readers' records: `ended.now` (`revision: current`, count 3) and
/// `ended.any` (count 3). Outcome `early` is the probe of the story's acceptance: `ended.any` true
/// while reader `c` is unopened, which no case reaches, since two children write two records.
const EARLY: &str = "format: protocol/1\n\
protocol: {id: early.min, revision: 1}\n\
artifacts: {sources: {}, marks: {}}\n\
evidence_kinds:\n  case_outcome: {}\n  a_opened: {}\n  b_opened: {}\n  c_opened: {}\n\
claims:\n\
\x20 a.opened: {true_when: {evidence: {kind: a_opened, subject: marks}}}\n\
\x20 b.opened: {true_when: {evidence: {kind: b_opened, subject: marks}}}\n\
\x20 c.opened: {true_when: {evidence: {kind: c_opened, subject: marks}}}\n\
\x20 ended.now: {true_when: {children: {in: [ok], subject: sources, revision: current, count: 3}}}\n\
\x20 ended.any: {true_when: {children: {in: [ok], subject: sources, count: 3}}}\n\
actions:\n\
\x20 a.open: {precondition: {claim: a.opened, is: unknown}, case_open: {protocol: reader, subject: sources}, may_produce: [{evidence: a_opened}]}\n\
\x20 b.open: {precondition: {claim: b.opened, is: unknown}, case_open: {protocol: reader, subject: sources}, may_produce: [{evidence: b_opened}]}\n\
\x20 c.open: {precondition: {claim: c.opened, is: unknown}, case_open: {protocol: reader, subject: sources}, may_produce: [{evidence: c_opened}]}\n\
\x20 redo: {precondition: {claim: a.opened}, revises: [sources]}\n\
outcomes:\n\
\x20 fresh: {requires: {claim: ended.now}}\n\
\x20 early: {requires: {all: [{claim: ended.any}, {claim: c.opened, is: unknown}]}}\n";

/// story:children-form-counts-unopened-children, acceptance: "the claim is not true while any
/// reader is unopened"; `simulate/mod.rs` module docs: "Each count stays at or below the case's,
/// never above it, so a counting form is never true in the walk where it is not in the case".
///
/// The walk reaches `early`: reader `a` opens, `redo` revises `sources`, reader `b` opens (two
/// children, one counted at the current revision), `a`'s child ends against the earlier revision,
/// and `b`'s child ends replacing both `open` records, the current one and the one held as
/// earlier. `Layout::ended` raises the count at the current revision for the earlier `open`
/// record replaced, though `b` was counted there when it opened, so the walk counts 2 children at
/// the current revision of 2 in all, and `Layout::with_children` evaluates one current copy beside
/// the current and the earlier record: three records for two children, and `ended.any` is true
/// while `c` is unopened.
#[test]
fn simulate_never_counts_more_children_than_were_opened() {
    let dir = scratch("early");
    std::fs::write(dir.join("protocol.yaml"), EARLY).expect("written");
    let output = canon_in(&dir, &["simulate", "--path", "protocol.yaml"]);
    let report = text(&output.stdout);
    assert!(
        report.contains("unreached-outcome: outcome `early` is legitimate in no reachable state\n"),
        "{}",
        report
            .lines()
            .filter(|line| line.starts_with("simulated: ") || line.starts_with("unreached"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

/// story:children-form-counts-unopened-children: `canon validate` refuses `count: 0`
/// (`zero-child-count`), and the published `protocol/1` schema, which an editor validates a
/// document with, says the same: its `count` is an integer of 1 or more. The schema says
/// `"minimum": 0`, and the generated reference (`website/docs/reference/protocol.md`) types the
/// field "integer, 0 or more" beside a description that says 0 is refused.
#[test]
fn the_schema_admits_no_count_that_validate_refuses() {
    let dir = scratch("zero-count");
    std::fs::write(
        dir.join("protocol.yaml"),
        "format: protocol/1\nprotocol: {id: p, revision: 1}\nartifacts: {a: {}}\n\
         evidence_kinds: {case_outcome: {}}\n\
         claims: {c: {true_when: {children: {in: [done], count: 0}}}}\n",
    )
    .expect("written");
    let output = canon_in(&dir, &["validate", "--path", "protocol.yaml"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        text(&output.stderr).starts_with("error[zero-child-count]: "),
        "{}",
        text(&output.stderr)
    );
    let schema = std::fs::read_to_string(
        repository_root().join("website/static/schemas/protocol-1.schema.json"),
    )
    .expect("the schema reads");
    let schema: serde_yaml_ng::Value = serde_yaml_ng::from_str(&schema).expect("JSON is YAML");
    let count = &schema["$defs"]["ChildOutcomes"]["properties"]["count"];
    assert_eq!(count["type"].as_str(), Some("integer"), "{count:?}");
    assert_eq!(count["minimum"].as_u64(), Some(1), "{count:?}");
}

/// A parent that opens one reader about `sources` and merges once every reader ended `done`.
/// `GUARD` stands for the merge's `runs_only_when` line.
const CHILDREN_PARENT: &str = "format: protocol/1\n\
protocol: {id: gen.guard, revision: 1}\n\
artifacts:\n  sources: {}\n\
evidence_kinds:\n  case_outcome: {}\n  merged: {}\n  opened: {}\n\
claims:\n\
\x20 readers.done: {true_when: {children: {in: [done], subject: sources}}}\n\
\x20 merged: {true_when: {evidence: {kind: merged}}}\n\
\x20 opened: {true_when: {evidence: {kind: opened}}}\n\
actions:\n\
\x20 open.reader:\n\
\x20   precondition: {claim: opened, is: unknown}\n\
\x20   case_open: {protocol: reader, subject: sources}\n\
\x20   may_produce: [{evidence: opened}]\n\
\x20 merge:\n\
\x20   precondition: {all: [{claim: readers.done}, {claim: merged, is: unknown}]}\n\
GUARD\
\x20   may_produce: [{evidence: merged}]\n\
outcomes:\n  done: {requires: {claim: merged}}\n";

/// story:ordering-guards-provable-by-check. `website/docs/concepts/protocols.md` ("Ordering
/// guards") says `canon evaluate`, `canon simulate` and `canon generate` never read
/// `runs_only_when` "and give the same output with or without it"; `ess/domains/protocol.yaml`
/// says the same of `canon generate`. A guard that reads `case_outcome` with an order form, beside
/// the protocol's own `children` form, makes `canon generate` refuse `ordered-child-outcomes`,
/// where the same protocol without the guard generates its scenarios.
#[test]
fn generate_gives_the_same_output_with_a_guard_that_reads_child_outcomes_in_order() {
    let without = generated("children-without", &CHILDREN_PARENT.replace("GUARD", ""));
    assert_eq!(without.0, Some(0), "{without:?}");
    assert!(!without.3.is_empty(), "{without:?}");
    let with = generated(
        "children-with",
        &CHILDREN_PARENT.replace(
            "GUARD",
            "    runs_only_when: {first: {kind: case_outcome, result: done}}\n",
        ),
    );
    assert_eq!(with, without);
}

/// The same promise, for a guard whose slice is above the state bound while every slice of the
/// protocol without it is within it: `canon generate` refuses `state-space-bound`, naming the
/// guard, where without the guard it writes its scenarios.
#[test]
fn generate_gives_the_same_output_with_a_guard_whose_slice_is_above_the_bound() {
    let kinds: String = (1..=17).map(|n| format!("  k{n}: {{}}\n")).collect();
    let makers: String = (1..=17)
        .map(|n| format!("  make{n}: {{may_produce: [{{evidence: k{n}}}]}}\n"))
        .collect();
    let any: Vec<String> = (1..=17)
        .map(|n| format!("{{evidence: {{kind: k{n}}}}}"))
        .collect();
    let source = |guard: &str| {
        format!(
            "format: protocol/1\nprotocol: {{id: gen.bound, revision: 1}}\n\
             artifacts: {{a: {{}}}}\nevidence_kinds:\n  done: {{}}\n{kinds}\
             claims:\n  done: {{true_when: {{evidence: {{kind: done}}}}}}\n\
             actions:\n  finish:\n    precondition: {{claim: done, is: unknown}}\n{guard}\
             \x20   may_produce: [{{evidence: done}}]\n{makers}\
             outcomes:\n  finished: {{requires: {{claim: done}}}}\n"
        )
    };
    let without = generated("bound-without", &source(""));
    assert_eq!(without.0, Some(0), "{without:?}");
    assert!(!without.3.is_empty(), "{without:?}");
    let with = generated(
        "bound-with",
        &source(&format!(
            "    runs_only_when: {{any: [{}]}}\n",
            any.join(", ")
        )),
    );
    assert_eq!(with, without);
}
