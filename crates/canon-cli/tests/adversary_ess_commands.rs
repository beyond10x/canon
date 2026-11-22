//! Adversary pass 1 on story:ess-command-surface (wave 2026-11-20-w1).
//!
//! The parity test `ess_commands_match.rs` is compiled into this binary as the module `parity`,
//! unchanged, and its `every_canon_subcommand_is_declared_in_ess` is run as a child process of this
//! binary against a copy of `ess/` edited by each case (`CARGO_MANIFEST_DIR` points the child at
//! the copy; the clap side is the real `canon_cli::Cli`). Each case states what the gate must do on
//! that copy, so a case goes green when the parity test does it, not when this file changes.
//!
//! `long_flags_on_every_command_line_level_are_declared` walks the clap definition on its own, the
//! way clap parses it (globals propagated, flags of parent commands included), because a flag on
//! the root `canon` or on `canon conform` cannot be added from outside `crates/canon-cli/src`.

#[path = "ess_commands_match.rs"]
#[allow(dead_code, clippy::duplicate_mod)]
mod parity;

/// The parity test as the test harness names it: its path below the crate root of this binary.
fn parity_test() -> String {
    let test = "parity::every_canon_subcommand_is_declared_in_ess";
    match module_path!().split_once("::") {
        Some((_, module)) => format!("{module}::{test}"),
        None => test.to_owned(),
    }
}

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{ArgAction, CommandFactory};
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

/// Runs the unit's parity test against a copy of `ess/` whose `domains/cli.yaml` `edit` changed.
/// Returns whether it passed and what it printed.
fn parity_on(case: &str, edit: impl FnOnce(&mut Vec<Value>)) -> (bool, String) {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adversary-ess-commands")
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
    edit(
        domain["commands"]
            .as_sequence_mut()
            .expect("cli.yaml declares commands"),
    );
    fs::write(
        &file,
        serde_yaml_ng::to_string(&domain).expect("serializes"),
    )
    .expect("write copy");

    let output = Command::new(std::env::current_exe().expect("this test binary"))
        .args([
            parity_test().as_str(),
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

fn command(yaml: &str) -> Value {
    serde_yaml_ng::from_str(yaml).expect("command is YAML")
}

fn named<'a>(commands: &'a mut [Value], name: &str) -> &'a mut Value {
    commands
        .iter_mut()
        .find(|command| command["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("cli.yaml declares {name}"))
}

/// The spec-first commit of a new command (AGENTS.md § ESS: "declares the change in
/// `ess/domains/cli.yaml` in its first commit"), written by copying `canon.cli.Validate` and not
/// changing `naming.wire`. ESS 0.77.0 compiles two commands with one wire; the gate must name the
/// command that is no `canon` subcommand.
#[test]
fn an_ess_command_sharing_a_wire_is_named() {
    let (passed, output) = parity_on("shared-wire", |commands| {
        commands.push(command(
            "name: canon.cli.Unclaimed
naming:
  wire: validate
  display: Unclaimed
  summary: Not a canon subcommand.
input:
  - name: path
    type: String
  - name: budget
    type: Optional<String>
response:
  - name: output
    type: String
outcomes:
  - name: done
    returns: true
",
        ));
    });
    assert!(
        !passed && output.contains("canon.cli.Unclaimed"),
        "ESS declares canon.cli.Unclaimed (wire `validate`, input `budget`), which is no canon \
         subcommand, and the parity test passed without naming it:\n{output}"
    );
}

/// Control for the harness above: the same command with its own wire fails the parity test.
#[test]
fn an_ess_command_with_its_own_wire_is_named() {
    let (passed, output) = parity_on("own-wire", |commands| {
        commands.push(command(
            "name: canon.cli.Unclaimed
naming:
  wire: unclaimed
  display: Unclaimed
  summary: Not a canon subcommand.
input:
  - name: path
    type: String
response:
  - name: output
    type: String
outcomes:
  - name: done
    returns: true
",
        ));
    });
    assert!(
        !passed
            && output.contains("canon.cli.Unclaimed")
            && output.contains("not a canon subcommand"),
        "{output}"
    );
}

/// An input declared in ESS with no clap flag (the spec-first commit of a new flag) fails, naming
/// it. No control in the parity test reaches this branch.
#[test]
fn an_input_only_in_ess_is_named() {
    let (passed, output) = parity_on("input-only-in-ess", |commands| {
        named(commands, "canon.cli.Validate")["input"]
            .as_sequence_mut()
            .expect("Validate has inputs")
            .push(command("name: include\ntype: Optional<String>\n"));
    });
    assert!(
        !passed
            && output.contains("canon.cli.Validate")
            && output.contains("input `include` is not a flag"),
        "{output}"
    );
}

/// A subcommand with no ESS command fails, naming it. No control in the parity test reaches this
/// branch.
#[test]
fn a_command_only_in_clap_is_named() {
    let (passed, output) = parity_on("command-only-in-clap", |commands| {
        commands.retain(|command| command["name"].as_str() != Some("canon.cli.Diff"));
    });
    assert!(
        !passed && output.contains("`canon diff` has no ESS command"),
        "{output}"
    );
}

/// A flag clap does not require that ESS declares required fails, naming both sides. The parity
/// test's control covers only the other direction.
#[test]
fn an_optional_flag_required_in_ess_is_named() {
    let (passed, output) = parity_on("required-only-in-ess", |commands| {
        for input in named(commands, "canon.cli.Evaluate")["input"]
            .as_sequence_mut()
            .expect("Evaluate has inputs")
        {
            if input["name"].as_str() == Some("authority") {
                input["type"] = Value::from("String");
            }
        }
    });
    assert!(
        !passed && output.contains("--authority") && output.contains("clap says optional"),
        "{output}"
    );
}

/// Every long flag the `canon` command line accepts on a leaf command, wherever in the command
/// tree clap declares it (the root, a parent such as `conform`, a global flag), is an input of the
/// leaf's ESS command, and the reverse; no two ESS commands share a wire. `--help`, `--version`
/// and the `help` subcommand are clap's own. Green on d2c8e53; red when a global flag is added to
/// `Cli` or a flag to `Command::Conform`, which the parity test passes.
#[test]
fn long_flags_on_every_command_line_level_are_declared() {
    let mut cli = canon_cli::Cli::command();
    cli.build();
    let mut clap_side: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut problems = Vec::new();
    walk(&cli, &[], &BTreeSet::new(), &mut clap_side, &mut problems);

    let output = Command::new("ess")
        .args(["specify", "compile", "--path", "ess", "--format", "json"])
        .current_dir(tree())
        .output()
        .expect("`ess` 0.77.0 is on PATH");
    assert!(output.status.success(), "ess specify compile failed");
    let ir: Value = serde_yaml_ng::from_slice(&output.stdout).expect("IR is JSON");
    let mut ess_side: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (name, command) in ir["commands"].as_mapping().expect("commands map") {
        if command["domain"].as_str() != Some("canon.cli") {
            continue;
        }
        let wire = command["naming"]["wire"].as_str().expect("wire").to_owned();
        let inputs = command["input"]
            .as_sequence()
            .into_iter()
            .flatten()
            .map(|input| input["name"].as_str().expect("input name").to_owned())
            .collect();
        if ess_side.insert(wire.clone(), inputs).is_some() {
            problems.push(format!(
                "a second ESS command has wire `{wire}`: {}",
                name.as_str().unwrap_or("?")
            ));
        }
    }
    if clap_side != ess_side {
        problems.push(format!("clap {clap_side:?}\n  ESS  {ess_side:?}"));
    }
    assert!(problems.is_empty(), "{}", problems.join("\n  "));
}

fn walk(
    command: &clap::Command,
    prefix: &[&str],
    inherited: &BTreeSet<String>,
    out: &mut BTreeMap<String, BTreeSet<String>>,
    problems: &mut Vec<String>,
) {
    let mut flags = inherited.clone();
    for arg in command.get_arguments() {
        if matches!(
            arg.get_action(),
            ArgAction::Help | ArgAction::HelpShort | ArgAction::HelpLong | ArgAction::Version
        ) {
            continue;
        }
        match arg.get_long() {
            Some(long) => {
                flags.insert(long.to_owned());
            }
            None => problems.push(format!(
                "`canon {}` argument `{}` is not a long flag",
                prefix.join(" "),
                arg.get_id()
            )),
        }
    }
    if !command.has_subcommands() {
        out.insert(prefix.join("-"), flags);
        return;
    }
    // `build` adds clap's own `help` subcommand to every command that has subcommands.
    for sub in command
        .get_subcommands()
        .filter(|sub| sub.get_name() != "help")
    {
        let mut path = prefix.to_vec();
        path.push(sub.get_name());
        walk(sub, &path, &flags, out, problems);
    }
}
