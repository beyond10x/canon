//! Acceptance for story:artifact-revised-by against the `canon` command line: an artifact of
//! `protocol/1` names the protocol whose cases revise it, `revised_by`, and `canon simulate` counts
//! a state only that protocol's revision can open as waiting.
//!
//! Over `fixtures/investigation/revised-by/`: `revised.yaml` is the protocol
//! `revised.by.example` whose artifact `doc` declares `revised_by: revised.by.owner`; `plain.yaml`
//! is the same document without that line and `replaced.yaml` the same with another protocol
//! named. `invalid/revised-by-self.yaml` names the protocol's own id. `simulate/revised-by` and
//! `simulate/revised-by-undeclared` are the pair for the revise step, with `report.txt` written by
//! hand from the story before the step existed.
//!
//! `artifact_revised_by_another_protocol_is_waited_on` holds the story's acceptance, numbered as it
//! numbers them. Acceptance 7 is `simulate::tests::an_external_revision_is_written_as_protocol_
//! revises` in `crates/canon/src/simulate/tests.rs`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use b10x_canon::{ir, model};

const DIR: &str = "fixtures/investigation/revised-by";
const SIMULATE: &str = "fixtures/investigation/simulate";
/// The line `revised.yaml` holds and `plain.yaml` does not.
const REVISED_LINE: &str = "    revised_by: revised.by.owner\n";
/// The digest of `plain.yaml`'s `canon-ir/1` as canon compiled it before the field existed.
const PLAIN_DIGEST_BEFORE: &str =
    "sha256:5ca016df810d6a77dca9e8e2ca7083738766470d8e11183e7ec62a12d99a618c";

/// Read at run time, not compile time: a test binary reused from a shared build directory must
/// still read this tree's fixtures.
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

fn text(bytes: &[u8]) -> &str {
    std::str::from_utf8(bytes).expect("utf-8 output")
}

fn fixture(name: &str) -> String {
    format!("{DIR}/{name}")
}

fn read(path: &str) -> String {
    std::fs::read_to_string(repository_root().join(path))
        .unwrap_or_else(|error| panic!("{path}: {error}"))
}

/// A scratch directory of this test binary, inside the build directory, emptied first.
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("revised-by")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

fn path(path: &Path) -> &str {
    path.to_str().expect("utf-8 path")
}

/// Exit 0, nothing on standard error; standard output.
fn succeeded(output: &Output) -> String {
    assert_eq!(text(&output.stderr), "");
    assert_eq!(output.status.code(), Some(0));
    text(&output.stdout).to_owned()
}

/// A refusal: exit 1, nothing on standard output, and exactly `stderr`.
fn assert_refused(output: &Output, stderr: &str) {
    assert_eq!(text(&output.stderr), stderr);
    assert_eq!(text(&output.stdout), "");
    assert_eq!(output.status.code(), Some(1));
}

/// `canon compile` of the fixture `name`, written to `dir`; its bytes and its path.
fn compiled(name: &str, dir: &Path) -> (String, PathBuf) {
    let bytes = succeeded(&canon(&["compile", "--path", &fixture(name)]));
    let at = dir.join(name.replace(".yaml", ".ir.json"));
    std::fs::write(&at, &bytes).expect("IR written");
    (bytes, at)
}

/// The one record the evaluation below reads: a passing test of `doc`.
const TEST_RECORD: &str = "format: canon-evidence/1\nid: test-1\nkind: test\nresult: pass\n\
    subject: doc\nsubject_revision: r1\n";

fn evaluate(ir: &Path, evidence: &Path) -> Output {
    canon(&[
        "evaluate",
        "--ir",
        path(ir),
        "--case",
        &fixture("case.yaml"),
        "--evidence",
        path(evidence),
    ])
}

/// Every file under `dir`, by name, with its bytes.
fn files(dir: &Path) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = std::fs::read_dir(dir)
        .expect("a directory")
        .map(|entry| {
            let entry = entry.expect("an entry");
            (
                entry.file_name().to_string_lossy().into_owned(),
                std::fs::read_to_string(entry.path()).expect("a text file"),
            )
        })
        .collect();
    found.sort();
    found
}

#[test]
fn artifact_revised_by_another_protocol_is_waited_on() {
    let dir = scratch("acceptance");

    // 1. The declaration validates; the protocol's own id is refused, naming the artifact; a
    // name that is not an identifier is refused as one.
    assert_eq!(
        succeeded(&canon(&["validate", "--path", &fixture("revised.yaml")])),
        "valid: protocol `revised.by.example` revision 1\n"
    );
    assert_refused(
        &canon(&[
            "validate",
            "--path",
            "fixtures/investigation/invalid/revised-by-self.yaml",
        ]),
        "error[self-revised-artifact]: artifact `doc` is revised by protocol \
         `revised.by.example`, the protocol's own id\n",
    );
    let spaced = dir.join("spaced.yaml");
    std::fs::write(
        &spaced,
        read(&fixture("revised.yaml")).replace("revised_by: revised.by.owner", "revised_by: 'q q'"),
    )
    .expect("written");
    assert_refused(
        &canon(&["validate", "--path", path(&spaced)]),
        "error[invalid-identifier]: protocol identifier `q q` is empty or contains whitespace or \
         a control character\n",
    );

    // 2. The artifact compiles with its reviser, after its description, and a protocol without
    // one compiles to the bytes it did before the field existed.
    let (revised_ir, revised_at) = compiled("revised.yaml", &dir);
    let (plain_ir, plain_at) = compiled("plain.yaml", &dir);
    let (_, replaced_at) = compiled("replaced.yaml", &dir);
    assert!(
        revised_ir.contains(
            "    \"doc\": {\n      \"description\": \"The document under test.\",\n      \
             \"revised_by\": \"revised.by.owner\"\n    }\n"
        ),
        "{revised_ir}"
    );
    assert!(!plain_ir.contains("revised_by"), "{plain_ir}");
    assert_eq!(
        revised_ir.replace(",\n      \"revised_by\": \"revised.by.owner\"\n", "\n"),
        plain_ir
    );
    assert_eq!(
        read(&fixture("revised.yaml")).replace(REVISED_LINE, ""),
        read(&fixture("plain.yaml"))
    );
    let plain = ir::compile(&model::parse(&read(&fixture("plain.yaml"))).expect("parses"))
        .expect("compiles");
    assert_eq!(plain.canonical_json(), plain_ir);
    assert_eq!(ir::digest(&plain), PLAIN_DIGEST_BEFORE);
    // The IR names its own protocol as the reviser: refused on read-back.
    let own_at = dir.join("own.ir.json");
    std::fs::write(
        &own_at,
        revised_ir.replace(
            "\"revised_by\": \"revised.by.owner\"",
            "\"revised_by\": \"revised.by.example\"",
        ),
    )
    .expect("written");
    let evidence = dir.join("evidence");
    std::fs::create_dir_all(&evidence).expect("evidence directory");
    std::fs::write(evidence.join("test.yaml"), TEST_RECORD).expect("written");
    let own = evaluate(&own_at, &evidence);
    assert_eq!(text(&own.stdout), "");
    assert_eq!(own.status.code(), Some(1));
    assert!(
        text(&own.stderr).contains("self-revised-artifact"),
        "{}",
        text(&own.stderr)
    );
    // A reviser that is not a string is refused on read-back.
    let number_at = dir.join("number.ir.json");
    std::fs::write(
        &number_at,
        revised_ir.replace("\"revised_by\": \"revised.by.owner\"", "\"revised_by\": 7"),
    )
    .expect("written");
    let number = evaluate(&number_at, &evidence);
    assert_eq!(number.status.code(), Some(1));
    assert!(
        text(&number.stderr).contains("`revised_by` is not a string"),
        "{}",
        text(&number.stderr)
    );

    // 3. Evaluate, check and generate give the same bytes and exit codes with and without it.
    let with = succeeded(&evaluate(&revised_at, &evidence));
    assert_eq!(with, succeeded(&evaluate(&plain_at, &evidence)));
    assert!(with.contains("\"status\": \"legitimate\""), "{with}");
    let check_with = canon(&["check", "--path", &fixture("revised.yaml")]);
    let check_without = canon(&["check", "--path", &fixture("plain.yaml")]);
    assert_eq!(check_with.status.code(), check_without.status.code());
    assert_eq!(text(&check_with.stderr), text(&check_without.stderr));
    assert_eq!(text(&check_with.stdout), text(&check_without.stdout));
    let generated = |name: &str| {
        let out = dir.join(format!("generated-{name}"));
        let output = canon(&["generate", "--path", &fixture(name), "--out", path(&out)]);
        let stdout = succeeded(&output).replace(path(&out), "OUT");
        // Each scenario names the document it was generated from.
        let written: Vec<(String, String)> = files(&out)
            .into_iter()
            .map(|(file, bytes)| (file, bytes.replace(&fixture(name), "FIXTURE")))
            .collect();
        (stdout, written)
    };
    assert_eq!(generated("revised.yaml"), generated("plain.yaml"));

    // 4. A diff between them, in each direction and to another reviser, is no semantic change.
    let diff = |from: &Path, to: &Path| {
        succeeded(&canon(&["diff", "--from", path(from), "--to", path(to)]))
    };
    assert_eq!(
        diff(&plain_at, &revised_at),
        "diff: protocol `revised.by.example` revision 1 to 1\n\
         NO SEMANTIC CHANGE: artifact `doc` revised_by introduced: protocol `revised.by.owner`\n"
    );
    assert_eq!(
        diff(&revised_at, &plain_at),
        "diff: protocol `revised.by.example` revision 1 to 1\n\
         NO SEMANTIC CHANGE: artifact `doc` revised_by removed: was protocol \
         `revised.by.owner`\n"
    );
    assert_eq!(
        diff(&revised_at, &replaced_at),
        "diff: protocol `revised.by.example` revision 1 to 1\n\
         NO SEMANTIC CHANGE: artifact `doc` revised_by changed from protocol `revised.by.owner` \
         to protocol `revised.by.successor`\n"
    );

    // 5. Without the declaration the state holding the test is stuck; with it, it waits.
    let undeclared = canon(&[
        "simulate",
        "--path",
        &format!("{SIMULATE}/revised-by-undeclared/protocol.yaml"),
    ]);
    assert_eq!(text(&undeclared.stderr), "");
    assert_eq!(
        text(&undeclared.stdout),
        read(&format!("{SIMULATE}/revised-by-undeclared/report.txt"))
    );
    assert_eq!(undeclared.status.code(), Some(1));
    let declared = succeeded(&canon(&[
        "simulate",
        "--path",
        &format!("{SIMULATE}/revised-by/protocol.yaml"),
    ]));
    assert_eq!(declared, read(&format!("{SIMULATE}/revised-by/report.txt")));
    assert_eq!(
        declared,
        "simulated: protocol `simulate.revised.by` revision 1: 3 reachable states, 1 terminable \
         (outcome `done` 1), 1 waiting on supplied evidence, expiry or a revision by another \
         protocol, 0 findings (0 stuck or cycle states)\n"
    );

    // 6. A protocol that declares `supplied_by` and no `revised_by` keeps today's phrase.
    let supplied = succeeded(&canon(&[
        "simulate",
        "--path",
        &format!("{SIMULATE}/supplied/protocol.yaml"),
    ]));
    assert!(
        supplied.contains("1 waiting on supplied evidence or expiry, 0 findings"),
        "{supplied}"
    );
    assert!(
        !supplied.contains("revision by another protocol"),
        "{supplied}"
    );

    // 8. Two runs of each new fixture give identical bytes.
    for case in ["revised-by", "revised-by-undeclared"] {
        let again = |_| {
            canon(&[
                "simulate",
                "--path",
                &format!("{SIMULATE}/{case}/protocol.yaml"),
            ])
        };
        let (first, second) = (again(1), again(2));
        assert_eq!(
            (first.status.code(), &first.stdout, &first.stderr),
            (second.status.code(), &second.stdout, &second.stderr)
        );
    }
    assert_eq!(compiled("revised.yaml", &dir).0, revised_ir);
}
