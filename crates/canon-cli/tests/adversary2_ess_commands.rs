//! Adversary pass 2 on story:ess-command-surface (wave 2026-11-20-w1).
//!
//! As in `adversary_ess_commands.rs`: the parity test `ess_commands_match.rs` is compiled into this
//! binary unchanged, as the module `parity`, and its `every_canon_subcommand_is_declared_in_ess`
//! runs as a child process against a copy of `ess/` that the case edits (`CARGO_MANIFEST_DIR`
//! points the child at the copy). A case goes green when the parity test does what it states.

#[path = "ess_commands_match.rs"]
#[allow(dead_code)]
mod parity;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_yaml_ng::Value;

fn tree() -> PathBuf {
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR").expect("run through cargo");
    Path::new(&manifest)
        .join("../..")
        .canonicalize()
        .expect("repository root resolves")
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create directory");
    for entry in fs::read_dir(from).expect("read directory") {
        let path = entry.expect("directory entry").path();
        let target = to.join(path.file_name().expect("entry has a name"));
        if path.is_dir() {
            copy_dir(&path, &target);
        } else {
            fs::copy(&path, &target).expect("copy file");
        }
    }
}

/// Runs the unit's parity test against a copy of `ess/` in which the input `input` of the command
/// `command` has the type `ty`. Returns whether it passed and what it printed.
fn parity_with_type(case: &str, command: &str, input: &str, ty: &str) -> (bool, String) {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adversary2-ess-commands")
        .join(case);
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove the previous copy");
    }
    fs::create_dir_all(root.join("crates/canon-cli")).expect("create the copy");
    copy_dir(&tree().join("ess"), &root.join("ess"));
    let file = root.join("ess/domains/cli.yaml");
    let mut domain: Value =
        serde_yaml_ng::from_str(&fs::read_to_string(&file).expect("cli.yaml reads"))
            .expect("cli.yaml is YAML");
    let mut edited = false;
    for declared in domain["commands"]
        .as_sequence_mut()
        .expect("cli.yaml declares commands")
        .iter_mut()
        .filter(|declared| declared["name"].as_str() == Some(command))
    {
        for field in declared["input"]
            .as_sequence_mut()
            .expect("the command has inputs")
        {
            if field["name"].as_str() == Some(input) {
                field["type"] = Value::from(ty);
                edited = true;
            }
        }
    }
    assert!(edited, "cli.yaml declares no input {input} of {command}");
    fs::write(
        &file,
        serde_yaml_ng::to_string(&domain).expect("serializes"),
    )
    .expect("write copy");

    let output = Command::new(std::env::current_exe().expect("this test binary"))
        .args([
            "parity::every_canon_subcommand_is_declared_in_ess",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("CARGO_MANIFEST_DIR", root.join("crates/canon-cli"))
        .current_dir(&root)
        .output()
        .expect("run the parity test");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        text.contains("1 passed") || text.contains("1 failed"),
        "the child ran no parity test:\n{text}"
    );
    (output.status.success(), text)
}

/// AGENTS.md § ESS: "a flag clap takes more than once is a `List<…>` input, and a flag it takes
/// once is not". `--authority` is taken once; ESS 0.77.0 compiles and synthesizes it declared as a
/// `Map`, a collection of many values, and the gate must name it.
#[test]
fn a_single_valued_flag_declared_as_a_map_is_named() {
    let (passed, output) = parity_with_type(
        "single-as-map",
        "canon.cli.Evaluate",
        "authority",
        "Optional<Map<String, String>>",
    );
    assert!(
        !passed && output.contains("--authority"),
        "ESS declares `canon evaluate --authority`, which clap takes once, as \
         Optional<Map<String, String>>, and the parity test passed without naming it:\n{output}"
    );
}

/// Control for the harness: the same flag declared as a `List` fails the parity test, naming it.
#[test]
fn a_single_valued_flag_declared_as_a_list_fails_the_child() {
    let (passed, output) = parity_with_type(
        "single-as-list",
        "canon.cli.Evaluate",
        "authority",
        "Optional<List<String>>",
    );
    assert!(
        !passed && output.contains("--authority") && output.contains("a List"),
        "{output}"
    );
}
