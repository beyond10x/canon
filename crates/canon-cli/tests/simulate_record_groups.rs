//! Acceptance for story:simulate-record-groups: an action declares which kinds one run of it
//! records together (`records_together`), and `canon simulate` adds such a group in one step and
//! never offers one of its kinds alone. The declaration has no evaluation meaning.
//!
//! The fixtures are in `fixtures/investigation/record-group/`, each directory a `protocol.yaml`
//! and, where the walk's whole output is pinned, its standard output `report.txt`, written by hand
//! from the story before the group step existed (`pair/report.txt` is today's walk, which the
//! story records). The refused protocols are in its `invalid/`, which the tests over every
//! investigation fixture skip.
//!
//! `simulate_records_a_group_in_one_step` holds the story's expectations 1 to 6:
//!
//! 1. `pair`, two kinds an action records only while neither is recorded, without the
//!    declaration: exit 1, `3 reachable states, 0 terminable (outcome `done` 0), 4 findings (2
//!    stuck or cycle states)`.
//! 2. `grouped`, the same with `records_together: [{kinds: [fit, shape]}]`: exit 0 and only the
//!    summary, 2 reachable states, 1 terminable, 0 findings: no state holds one kind alone.
//! 3. `result`, a grouped protocol whose claims read a result: each stuck state's path is one
//!    step naming both records, in kind-id order, joined by ` and `.
//! 4. A group with fewer than two distinct kinds, one naming a kind outside `may_produce`, and a
//!    kind in two groups of one action are each refused as `invalid-record-group`, naming the
//!    action; so is a group naming `case_outcome` on an action that opens a child case.
//! 5. The `canon-ir/1` of `pair` is the bytes it had before the field existed; the grouped IR
//!    holds `"records_together"`, its groups and their kinds sorted, and reads back as `canon
//!    evaluate --ir` reads it; `canon diff` reports a group added as `NO SEMANTIC CHANGE`.
//! 6. Two runs on each fixture give identical bytes. (`simulate_reports_reachability_findings`,
//!    in `canon_simulate.rs`, still holds the walk without groups.)
//!
//! `canon evaluate`, `canon check` and `canon generate` give the same bytes for `pair` and
//! `grouped`, one test each.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use b10x_canon::eval;
use b10x_canon::ir;
use b10x_canon::model;

/// The fixture family, relative to the repository root.
const FIXTURES: &str = "fixtures/investigation/record-group";

/// Each fixture directory the walk is run on, with its exit status.
const WALKED: [(&str, i32); 3] = [("pair", 1), ("grouped", 0), ("result", 1)];

fn repository_root() -> PathBuf {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is unset; run this test through cargo");
    PathBuf::from(manifest_dir)
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

fn canon_in(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_canon"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("the canon binary runs")
}

fn canon(args: &[&str]) -> Output {
    canon_in(&repository_root(), args)
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

fn path(path: &Path) -> &str {
    path.to_str().expect("utf-8 path")
}

/// A scratch directory of this test binary, inside the build directory, emptied first.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("simulate-record-groups")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

/// Exit 0, nothing on standard error; standard output.
fn succeeded(output: &Output) -> String {
    assert_eq!(text(&output.stderr), "");
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stdout));
    text(&output.stdout)
}

/// `canon compile` of the fixture `case`, written to `dir`; its bytes and its path.
fn compiled(case: &str, dir: &Path) -> (String, PathBuf) {
    let bytes = succeeded(&canon(&["compile", "--path", &protocol(case)]));
    let at = dir.join(format!("{case}.ir.json"));
    std::fs::write(&at, &bytes).expect("IR written");
    (bytes, at)
}

/// `canon compile fixtures/investigation/record-group/pair/protocol.yaml` with the binary of
/// `7ef6baa1`, before `records_together` existed.
const PAIR_IR: &str = r#"{
  "actions": {
    "check": {
      "description": "records both kinds in one run, only while neither is recorded",
      "effect": "read",
      "may_produce": [
        {
          "evidence": "fit"
        },
        {
          "evidence": "shape"
        }
      ],
      "precondition": {
        "all": [
          {
            "claim": {
              "id": "fitted",
              "is": "unknown"
            }
          },
          {
            "claim": {
              "id": "shaped",
              "is": "unknown"
            }
          }
        ]
      },
      "requires": []
    }
  },
  "artifacts": {
    "doc": {
      "description": "the document checked"
    }
  },
  "claims": {
    "fitted": {
      "description": "a fit record exists",
      "true_when": {
        "evidence": {
          "kind": "fit",
          "result": null,
          "subject": "doc"
        }
      }
    },
    "shaped": {
      "description": "a shape record exists",
      "true_when": {
        "evidence": {
          "kind": "shape",
          "result": null,
          "subject": "doc"
        }
      }
    }
  },
  "evidence_kinds": {
    "fit": {
      "description": "fit check"
    },
    "shape": {
      "description": "shape check"
    }
  },
  "format": "canon-ir/1",
  "obligations": {},
  "outcomes": {
    "done": {
      "description": "both checked",
      "requires": {
        "all": [
          {
            "claim": {
              "id": "fitted",
              "is": "true"
            }
          },
          {
            "claim": {
              "id": "shaped",
              "is": "true"
            }
          }
        ]
      }
    }
  },
  "protocol": {
    "description": "one action records two kinds in one run",
    "id": "probe.pair",
    "revision": 1
  }
}
"#;

/// The group of `grouped` as `canon-ir/1` writes it, after the precondition.
const GROUPED_IR_GROUP: &str = "      \"precondition\": {\n        \"all\": [\n          {\n            \"claim\": {\n              \"id\": \"fitted\",\n              \"is\": \"unknown\"\n            }\n          },\n          {\n            \"claim\": {\n              \"id\": \"shaped\",\n              \"is\": \"unknown\"\n            }\n          }\n        ]\n      },\n      \"records_together\": [\n        {\n          \"kinds\": [\n            \"fit\",\n            \"shape\"\n          ]\n        }\n      ],\n      \"requires\": []\n";

/// Each refused fixture of `invalid/` and what `canon validate` prints on standard error.
const REFUSED: [(&str, &str); 4] = [
    (
        "one-kind.yaml",
        "error[invalid-record-group]: action `check` records_together group 1 names fewer than \
         two distinct evidence kinds\n",
    ),
    (
        "not-produced.yaml",
        "error[invalid-record-group]: action `check` records_together group 1 names evidence \
         kind `size`, which its may_produce does not list\n",
    ),
    (
        "two-groups.yaml",
        "error[invalid-record-group]: action `check` records_together group 2 names evidence \
         kind `shape`, which an earlier group names\n",
    ),
    (
        "child-outcome.yaml",
        "error[invalid-record-group]: action `check` records_together group 1 names evidence \
         kind `case_outcome`, which a child case the action opens records\n",
    ),
];

/// Runs `canon simulate` on each of `cases` twice and returns what differs from its `report.txt`
/// and exit status, or between the two runs.
fn walk_failures(cases: &[(&str, i32)]) -> Vec<String> {
    let mut failures = Vec::new();
    for (case, status) in cases {
        let expected = read(&format!("{FIXTURES}/{case}/report.txt"));
        let first = canon(&["simulate", "--path", &protocol(case)]);
        if first.status.code() != Some(*status)
            || text(&first.stdout) != expected
            || !first.stderr.is_empty()
        {
            failures.push(format!(
                "{case}: expected exit {status} and stdout {expected:?}; got exit {:?}, stdout \
                 {:?}, stderr {:?}",
                first.status.code(),
                text(&first.stdout),
                text(&first.stderr)
            ));
            continue;
        }
        let second = canon(&["simulate", "--path", &protocol(case)]);
        if (second.status.code(), &second.stdout, &second.stderr)
            != (first.status.code(), &first.stdout, &first.stderr)
        {
            failures.push(format!("{case}: a second run gave different bytes"));
        }
    }
    failures
}

#[test]
fn simulate_records_a_group_in_one_step() {
    // 1, 2, 3 and 6: the walk of each fixture, twice.
    let mut failures = walk_failures(&WALKED);
    let pair = read(&format!("{FIXTURES}/pair/report.txt"));
    assert!(pair.ends_with(
        "3 reachable states, 0 terminable (outcome `done` 0), 4 findings (2 stuck or cycle \
         states)\n"
    ));
    let grouped = read(&format!("{FIXTURES}/grouped/report.txt"));
    assert_eq!(grouped.lines().count(), 1);
    assert!(grouped.ends_with(
        "2 reachable states, 1 terminable (outcome `done` 1), 0 findings (0 stuck or cycle \
         states)\n"
    ));
    // 3: every stuck state of `result` is one step away, naming both records.
    let result = read(&format!("{FIXTURES}/result/report.txt"));
    for line in result
        .lines()
        .filter(|line| line.starts_with("stuck-state"))
    {
        let (_, path) = line.split_once("; path: ").expect("a path");
        assert!(!path.contains(", then "), "{line}");
        assert!(
            path.starts_with("action `check` produces evidence `fit` about `doc`")
                && path.contains(" and evidence `shape` about `doc`"),
            "{line}"
        );
    }

    // 4: each refusal, naming the action.
    for (file, stderr) in REFUSED {
        let output = canon(&["validate", "--path", &format!("{FIXTURES}/invalid/{file}")]);
        if output.status.code() != Some(1)
            || text(&output.stderr) != stderr
            || !output.stdout.is_empty()
        {
            failures.push(format!(
                "invalid/{file}: expected exit 1 and stderr {stderr:?}; got exit {:?}, stdout \
                 {:?}, stderr {:?}",
                output.status.code(),
                text(&output.stdout),
                text(&output.stderr)
            ));
        }
    }

    // 5: the IR without the field, the grouped IR, its read-back and the diff.
    let dir = scratch("acceptance");
    let (pair_ir, pair_at) = compiled("pair", &dir);
    if pair_ir != PAIR_IR {
        failures.push(format!("pair: the IR changed: {pair_ir}"));
    }
    let grouped = canon(&["compile", "--path", &protocol("grouped")]);
    if grouped.status.code() == Some(0) {
        let (grouped_ir, grouped_at) = compiled("grouped", &dir);
        if !grouped_ir.contains(GROUPED_IR_GROUP) {
            failures.push(format!(
                "grouped: the IR holds no sorted group: {grouped_ir}"
            ));
        }
        match eval::read_ir(&grouped_ir) {
            Ok(read) => {
                let source = model::parse(&read_source("grouped")).expect("parses");
                if read != ir::compile(&source).expect("compiles")
                    || read.canonical_json() != grouped_ir
                {
                    failures.push("grouped: the IR reads back to another IR".to_owned());
                }
            }
            Err(refusal) => failures.push(format!("grouped: the IR does not read back: {refusal}")),
        }
        let diff = canon(&["diff", "--from", path(&pair_at), "--to", path(&grouped_at)]);
        let expected = "diff: protocol `probe.pair` revision 1 to 1\n\
                        NO SEMANTIC CHANGE: action `check` now records together evidence `fit` \
                        and `shape`\n";
        if diff.status.code() != Some(0) || text(&diff.stdout) != expected {
            failures.push(format!(
                "diff: expected exit 0 and {expected:?}; got exit {:?}, stdout {:?}, stderr {:?}",
                diff.status.code(),
                text(&diff.stdout),
                text(&diff.stderr)
            ));
        }
    } else {
        failures.push(format!(
            "grouped: canon compile exits {:?}: {}",
            grouped.status.code(),
            text(&grouped.stderr)
        ));
    }

    assert!(
        failures.is_empty(),
        "{} expectations failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

fn read_source(case: &str) -> String {
    read(&protocol(case))
}

/// Groups are written sorted, and the kinds within each group sorted, each once; the IR reads
/// back to the same IR and the same bytes.
#[test]
fn the_ir_writes_groups_sorted_and_reads_them_back() {
    let source = "format: protocol/1\n\
        protocol: {id: p, revision: 1}\n\
        evidence_kinds: {a: {}, b: {}, c: {}, d: {}}\n\
        actions:\n  \
          act:\n    \
            may_produce: [{evidence: d}, {evidence: c}, {evidence: b}, {evidence: a}]\n    \
            records_together: [{kinds: [d, c]}, {kinds: [b, a, b]}]\n";
    let compiled = ir::compile(&model::parse(source).expect("parses")).expect("compiles");
    let text = compiled.canonical_json();
    assert!(
        text.contains(
            "      \"records_together\": [\n        {\n          \"kinds\": [\n            \"a\",\n            \"b\"\n          ]\n        },\n        {\n          \"kinds\": [\n            \"c\",\n            \"d\"\n          ]\n        }\n      ],\n"
        ),
        "{text}"
    );
    let read = eval::read_ir(&text).expect("reads back");
    assert_eq!(read, compiled);
    assert_eq!(read.canonical_json(), text);
    // An empty list writes no key, as no list does.
    let empty = source.replace(
        "records_together: [{kinds: [d, c]}, {kinds: [b, a, b]}]",
        "records_together: []",
    );
    let without = source.replace(
        "    records_together: [{kinds: [d, c]}, {kinds: [b, a, b]}]\n",
        "",
    );
    let bytes = |source: &str| {
        ir::compile(&model::parse(source).expect("parses"))
            .expect("compiles")
            .canonical_json()
    };
    assert!(!bytes(&without).contains("records_together"));
    assert_eq!(bytes(&empty), bytes(&without));
}

/// One case snapshot and one record of `shape`, in `dir`; the case's path and the evidence
/// directory's.
fn snapshot(dir: &Path) -> (PathBuf, PathBuf) {
    let case = dir.join("case.yaml");
    std::fs::write(
        &case,
        "format: canon-case/1\nid: C-1\nprotocol: probe.pair\nartifacts: {doc: {revision: r1}}\n",
    )
    .expect("case written");
    let evidence = dir.join("evidence");
    std::fs::create_dir_all(&evidence).expect("evidence directory");
    std::fs::write(
        evidence.join("shape.yaml"),
        "format: canon-evidence/1\nid: e1\nkind: shape\nsubject: doc\nsubject_revision: r1\n",
    )
    .expect("record written");
    (case, evidence)
}

/// `canon evaluate` gives the same bytes under `pair` and `grouped`.
#[test]
fn evaluate_reads_nothing_of_a_group() {
    let dir = scratch("evaluate");
    let (case, evidence) = snapshot(&dir);
    let (pair_ir, pair_at) = compiled("pair", &dir);
    let (grouped_ir, grouped_at) = compiled("grouped", &dir);
    assert_ne!(pair_ir, grouped_ir);
    let evaluate = |ir: &Path| {
        succeeded(&canon(&[
            "evaluate",
            "--ir",
            path(ir),
            "--case",
            path(&case),
            "--evidence",
            path(&evidence),
        ]))
    };
    assert_eq!(evaluate(&grouped_at), evaluate(&pair_at));
}

/// `canon check` gives the same bytes on `pair` and `grouped`.
#[test]
fn check_reads_nothing_of_a_group() {
    let check = |case: &str| canon(&["check", "--path", &protocol(case)]);
    let (pair, grouped) = (check("pair"), check("grouped"));
    assert_eq!(text(&grouped.stderr), text(&pair.stderr));
    assert_eq!(text(&grouped.stdout), text(&pair.stdout));
    assert_eq!(grouped.status.code(), pair.status.code());
    assert!(text(&pair.stdout).starts_with("checked: protocol `probe.pair` revision 1: "));
}

/// `canon generate` writes the same files with the same bytes for `pair` and `grouped`, each run
/// on its protocol copied to the same relative path, since a scenario names that path.
#[test]
fn generate_reads_nothing_of_a_group() {
    let written = |case: &str| -> Vec<(String, String)> {
        let dir = scratch(&format!("generate-{case}"));
        std::fs::write(dir.join("protocol.yaml"), read_source(case)).expect("copied");
        succeeded(&canon_in(
            &dir,
            &["generate", "--path", "protocol.yaml", "--out", "out"],
        ));
        let mut files: Vec<(String, String)> = std::fs::read_dir(dir.join("out"))
            .expect("scenarios written")
            .map(|entry| {
                let entry = entry.expect("an entry");
                (
                    entry.file_name().to_string_lossy().into_owned(),
                    std::fs::read_to_string(entry.path()).expect("a scenario reads"),
                )
            })
            .collect();
        files.sort();
        files
    };
    let pair = written("pair");
    assert!(!pair.is_empty());
    assert_eq!(written("grouped"), pair);
}

/// Every fixture directory and refused protocol is one a test above reads, so none is skipped
/// silently.
#[test]
fn every_record_group_fixture_is_read() {
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
    let root = repository_root().join(FIXTURES);
    let mut named: Vec<String> = WALKED.iter().map(|(case, _)| (*case).to_owned()).collect();
    named.extend(["chain".to_owned(), "invalid".to_owned()]);
    named.sort();
    assert_eq!(entries(root.clone()), named);
    for (case, _) in WALKED {
        assert_eq!(
            entries(root.join(case)),
            ["protocol.yaml", "report.txt"],
            "{case}"
        );
    }
    let mut refused: Vec<String> = REFUSED.iter().map(|(file, _)| (*file).to_owned()).collect();
    refused.sort();
    assert_eq!(entries(root.join("invalid")), refused);
}
