//! story:protocol-access-class, `canon trigger protocols --path <file> ...` and `canon validate`
//! against the command line: the set's `canon-trigger-protocols/1` map is printed and exits 0, the
//! order of the flags changing no byte; a `read_only` action that writes is refused by `canon
//! validate` and by `canon trigger protocols` as `read-only-action-writes`; two files with one
//! protocol id are refused; a file that cannot be read exits 2. What the class and the guard mean
//! is the conformance scenario `conformance/scenarios/protocol-access-class.yaml`.

use std::path::PathBuf;
use std::process::{Command, Output};

/// Read at run time: a test binary reused from a shared build directory must read this tree.
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

fn protocols(paths: &[&str]) -> Output {
    let mut args = vec!["trigger", "protocols"];
    for path in paths {
        args.extend(["--path", path]);
    }
    canon(&args)
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

const LOOK_UP: &str = "fixtures/trigger/access/look-up.yaml";
const PUBLISH: &str = "fixtures/trigger/access/publish.yaml";
const REVISES: &str = "fixtures/trigger/access/read-only-revises.yaml";
const SEND_REMINDER: &str = "fixtures/trigger/send-reminder.yaml";

const TWO: &str = "{\n  \"format\": \"canon-trigger-protocols/1\",\n  \"protocols\": {\n    \
                   \"look-up\": {\n      \"class\": \"read_only\",\n      \"guarded\": false,\n      \
                   \"inputs\": {\n        \"region\": [\n          \"east\",\n          \"west\"\n        \
                   ]\n      }\n    },\n    \"publish\": {\n      \"class\": \"write\",\n      \
                   \"guarded\": true,\n      \"inputs\": {}\n    }\n  }\n}\n";

#[test]
fn the_set_prints_each_protocol_class_guard_and_inputs_in_any_flag_order() {
    for paths in [[LOOK_UP, PUBLISH], [PUBLISH, LOOK_UP]] {
        let output = protocols(&paths);
        assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
        assert_eq!(text(&output.stdout), TWO);
        assert_eq!(text(&output.stderr), "");
    }
}

#[test]
fn a_protocol_written_before_access_existed_is_write_and_lists_its_inputs() {
    let output = protocols(&[SEND_REMINDER]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_eq!(
        text(&output.stdout),
        "{\n  \"format\": \"canon-trigger-protocols/1\",\n  \"protocols\": {\n    \
         \"send-reminder\": {\n      \"class\": \"write\",\n      \"guarded\": false,\n      \
         \"inputs\": {\n        \"channel\": [\n          \"email\",\n          \"letter\"\n        \
         ]\n      }\n    }\n  }\n}\n"
    );
}

#[test]
fn a_file_given_twice_is_one_member() {
    let output = protocols(&[LOOK_UP, PUBLISH, LOOK_UP]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_eq!(text(&output.stdout), TWO);
}

#[test]
fn a_read_only_action_that_revises_is_refused_by_validate_and_by_protocols() {
    let validated = canon(&["validate", "--path", REVISES]);
    assert_eq!(validated.status.code(), Some(1));
    assert_eq!(text(&validated.stdout), "");
    assert_eq!(
        text(&validated.stderr),
        "error[read-only-action-writes]: action `edit` declares access read_only but revises \
         artifact `page`\n"
    );

    let listed = protocols(&[LOOK_UP, REVISES]);
    assert_eq!(listed.status.code(), Some(1));
    assert_eq!(text(&listed.stdout), "");
    assert_eq!(
        text(&listed.stderr),
        format!(
            "error[read-only-action-writes]: {REVISES}: action `edit` declares access read_only \
             but revises artifact `page`\n"
        )
    );
}

#[test]
fn two_files_with_one_protocol_id_are_refused() {
    let output = protocols(&[LOOK_UP, "fixtures/trigger/access/look-up-again.yaml"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(text(&output.stdout), "");
    assert_eq!(
        text(&output.stderr),
        format!(
            "error[duplicate-protocol-id]: protocol `look-up` is declared by \
             fixtures/trigger/access/look-up-again.yaml and {LOOK_UP}\n"
        )
    );
}

#[test]
fn a_file_that_cannot_be_read_exits_2() {
    let output = protocols(&[LOOK_UP, "fixtures/trigger/access/no-such-file.yaml"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(text(&output.stdout), "");
    assert!(
        text(&output.stderr).starts_with("error[unreadable]: "),
        "{}",
        text(&output.stderr)
    );
}
