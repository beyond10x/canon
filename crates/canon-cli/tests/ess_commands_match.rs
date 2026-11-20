//! The `canon` command surface and its ESS declaration stay equal.
//!
//! `canon_cli::Cli` is the clap definition of the `canon` command line; `ess/domains/cli.yaml`
//! declares the same surface as the commands of the domain `canon.cli` (Atlas ADR 0076). This test
//! walks the clap definition through `clap::CommandFactory`, compiles the specification
//! (`ess specify compile --format json`) and compares the two in both directions. A subcommand or
//! flag added to one and not the other fails here, naming what differs.
//!
//! How the clap definition maps onto the specification:
//!
//! * Every leaf subcommand is one ESS command of `canon.cli` whose `naming.wire` is the clap path
//!   joined by `-`: `canon validate` is `validate`, `canon conform run` is `conform-run`. A
//!   subcommand that has subcommands of its own (`conform`) is not a command.
//! * Every long flag `--name` of a leaf is the input `name` of its command. A flag clap requires is
//!   a required input; one it does not (an `Option<_>`, or one with a default value) is an
//!   `Optional<…>` input. An argument with no long flag matches nothing and is reported.
//!
//! It also synthesizes the specification's conformance suite (`ess verify conform synthesize`)
//! and counts its scenarios per command: every `canon.cli` command has at least one, which
//! `crates/canon/tests/ess_gate.rs` does not check.

use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{Arg, CommandFactory};
use serde_yaml_ng::Value;

/// The ESS domain whose commands are the `canon` command line.
const DOMAIN: &str = "canon.cli";

/// The tree under test, read at run time: a test binary reused from a shared target directory
/// would otherwise check the tree it was built from.
fn repo_root() -> PathBuf {
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR").expect(
        "CARGO_MANIFEST_DIR is unset: run this test through cargo, which sets it to the \
         crates/canon-cli directory of the tree under test",
    );
    Path::new(&manifest)
        .join("../..")
        .canonicalize()
        .expect("repository root resolves")
}

/// A fresh directory under `CARGO_TARGET_TMPDIR` for one case, distinct per tree under test.
fn scratch(case: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    repo_root().hash(&mut hasher);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("canon-ess-commands")
        .join(format!("{:016x}", hasher.finish()))
        .join(case);
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("remove the previous scratch copy");
    }
    fs::create_dir_all(&dir).expect("create scratch");
    dir
}

/// Runs `ess <args>` in `dir` and returns its stdout. Fails, rather than skips, when `ess` is not
/// on `PATH` or exits non-zero.
fn ess(dir: &Path, args: &[&str]) -> Vec<u8> {
    let output = match Command::new("ess").args(args).current_dir(dir).output() {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => panic!(
            "`ess` is not on PATH: comparing the command surface with ess/ needs ess 0.77.0, the \
             release ess/ess-inputs.yaml pins"
        ),
        Err(error) => panic!("cannot run `ess {}`: {error}", args.join(" ")),
    };
    assert!(
        output.status.success(),
        "`ess {}` in {} exited {}:\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        dir.display(),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

// ---------------------------------------------------------------------------------------------
// One shape for both sides: wire name -> (flag name -> required).

/// A command's inputs: flag (input) name to whether it is required.
type Inputs = BTreeMap<String, bool>;

#[derive(Debug, Default)]
struct Surface {
    /// Wire name (`conform-run`) to the command as each side names it and its inputs.
    commands: BTreeMap<String, (String, Inputs)>,
    /// Arguments that are not long flags, as `canon <path> <id>`; they match nothing.
    unmapped: Vec<String>,
}

/// The clap side: every leaf subcommand of `command`, below the path `prefix`.
fn clap_surface(command: &clap::Command) -> Surface {
    let mut surface = Surface::default();
    collect_leaves(command, &[], &mut surface);
    surface
}

fn collect_leaves(command: &clap::Command, prefix: &[&str], surface: &mut Surface) {
    for sub in command.get_subcommands() {
        let mut path = prefix.to_vec();
        path.push(sub.get_name());
        if sub.has_subcommands() {
            collect_leaves(sub, &path, surface);
            continue;
        }
        let display = format!("canon {}", path.join(" "));
        let mut inputs = Inputs::new();
        for arg in sub.get_arguments() {
            match arg.get_long() {
                Some(long) => {
                    inputs.insert(long.to_owned(), arg.is_required_set());
                }
                None => surface
                    .unmapped
                    .push(format!("{display} {}", arg.get_id().as_str())),
            }
        }
        surface.commands.insert(path.join("-"), (display, inputs));
    }
}

/// The specification side: every command of [`DOMAIN`] in the compiled IR of `root`/ess.
fn ess_surface(root: &Path) -> Surface {
    let ir: Value = serde_yaml_ng::from_slice(&ess(
        root,
        &["specify", "compile", "--path", "ess", "--format", "json"],
    ))
    .expect("compiled IR is JSON");
    let mut surface = Surface::default();
    let commands = ir["commands"]
        .as_mapping()
        .expect("compiled IR has a commands map");
    for (name, command) in commands {
        if command["domain"].as_str() != Some(DOMAIN) {
            continue;
        }
        let name = name.as_str().expect("command name").to_owned();
        let wire = command["naming"]["wire"]
            .as_str()
            .unwrap_or_else(|| panic!("ESS command `{name}` declares no naming.wire"))
            .to_owned();
        let mut inputs = Inputs::new();
        for input in command["input"].as_sequence().into_iter().flatten() {
            let field = input["name"].as_str().expect("input name").to_owned();
            let optional = input["type_ref"]["kind"].as_str() == Some("optional");
            inputs.insert(field, !optional);
        }
        surface.commands.insert(wire, (name, inputs));
    }
    surface
}

/// Every difference between the clap and ESS surfaces, in a stable order.
fn differences(clap: &Surface, spec: &Surface) -> Vec<String> {
    let mut out = Vec::new();
    for argument in &clap.unmapped {
        out.push(format!(
            "`{argument}` is not a long flag, so no ESS input can declare it"
        ));
    }
    for (wire, (display, clap_inputs)) in &clap.commands {
        let Some((name, spec_inputs)) = spec.commands.get(wire) else {
            out.push(format!(
                "`{display}` has no ESS command with wire `{wire}` in domain {DOMAIN}"
            ));
            continue;
        };
        for (flag, required) in clap_inputs {
            match spec_inputs.get(flag) {
                None => out.push(format!(
                    "`{display} --{flag}` has no input `{flag}` in ESS command `{name}`"
                )),
                Some(spec_required) if spec_required != required => out.push(format!(
                    "`{display} --{flag}`: clap says {}, ESS command `{name}` says {}",
                    requiredness(*required),
                    requiredness(*spec_required)
                )),
                Some(_) => {}
            }
        }
        for input in spec_inputs.keys() {
            if !clap_inputs.contains_key(input) {
                out.push(format!(
                    "ESS command `{name}` input `{input}` is not a flag of `{display}`"
                ));
            }
        }
    }
    for (wire, (name, _)) in &spec.commands {
        if !clap.commands.contains_key(wire) {
            out.push(format!(
                "ESS command `{name}` (wire `{wire}`) is not a canon subcommand"
            ));
        }
    }
    out
}

fn requiredness(required: bool) -> &'static str {
    if required { "required" } else { "optional" }
}

fn assert_equal(clap: &Surface, spec: &Surface) {
    let found = differences(clap, spec);
    assert!(
        found.is_empty(),
        "canon_cli::Cli and the {DOMAIN} commands of ess/ differ; the first difference is:\n  {}\n\
         all {}:\n  {}",
        found[0],
        found.len(),
        found.join("\n  ")
    );
}

fn assert_named(clap: &Surface, spec: &Surface, needles: &[&str]) {
    let found = differences(clap, spec);
    assert!(
        found
            .first()
            .is_some_and(|first| needles.iter().all(|needle| first.contains(needle))),
        "expected the first difference to name {needles:?}, found {found:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// The scenarios `ess verify conform synthesize` writes, per command.

/// Scenarios per command of [`DOMAIN`] in the suite synthesized from `ess/`.
fn scenarios_per_command() -> BTreeMap<String, usize> {
    let suite = scratch("suite").join("suite.json");
    let out = suite.to_str().expect("suite path is UTF-8");
    ess(
        &repo_root(),
        &[
            "verify",
            "conform",
            "synthesize",
            "--path",
            "ess",
            "--out",
            out,
        ],
    );
    let suite: Value =
        serde_yaml_ng::from_str(&fs::read_to_string(&suite).expect("the synthesized suite reads"))
            .expect("the synthesized suite is JSON");
    let mut counts = BTreeMap::new();
    let scenarios = suite["scenarios"]
        .as_mapping()
        .expect("suite has a scenarios map");
    for scenario in scenarios.values() {
        for source in scenario["source"].as_sequence().into_iter().flatten() {
            if source["kind"].as_str() == Some("command")
                && let Some(name) = source["name"].as_str()
                && name.starts_with(&format!("{DOMAIN}."))
            {
                *counts.entry(name.to_owned()).or_default() += 1;
            }
        }
    }
    counts
}

// ---------------------------------------------------------------------------------------------

/// The story's acceptance: every leaf subcommand and long flag of `canon_cli::Cli` is a command
/// and an input of the compiled `canon.cli` domain, and the reverse, and every command has at
/// least one synthesized scenario.
#[test]
fn every_canon_subcommand_is_declared_in_ess() {
    let clap = clap_surface(&canon_cli::Cli::command());
    let spec = ess_surface(&repo_root());
    assert!(
        !clap.commands.is_empty(),
        "canon_cli::Cli has no leaf subcommand, so nothing was compared"
    );
    assert_equal(&clap, &spec);

    let counts = scenarios_per_command();
    let rows: Vec<String> = spec
        .commands
        .values()
        .map(|(name, _)| format!("{name}: {}", counts.get(name).copied().unwrap_or(0)))
        .collect();
    println!(
        "synthesized scenarios per command:\n  {}",
        rows.join("\n  ")
    );
    let without: Vec<&String> = spec
        .commands
        .values()
        .map(|(name, _)| name)
        .filter(|name| counts.get(*name).copied().unwrap_or(0) == 0)
        .collect();
    assert!(
        without.is_empty(),
        "`ess verify conform synthesize --path ess` writes no scenario for {without:?}; \
         scenarios per command:\n  {}",
        rows.join("\n  ")
    );
}

/// A flag added to a subcommand in clap and not declared in ESS fails, naming both.
#[test]
fn a_flag_only_in_clap_is_named() {
    let spec = ess_surface(&repo_root());
    let command = canon_cli::Cli::command().mut_subcommand("validate", |validate| {
        validate.arg(Arg::new("probe_flag").long("probe-flag"))
    });
    assert_named(
        &clap_surface(&command),
        &spec,
        &["canon validate --probe-flag", "canon.cli.Validate"],
    );
}

/// A command declared in ESS that is no `canon` subcommand fails, naming it. It is added to a copy
/// of `ess/` and compiled, so the reading of the compiled IR is what is checked.
#[test]
fn a_command_only_in_ess_is_named() {
    let root = scratch("command-only-in-ess");
    copy_dir(&repo_root().join("ess"), &root.join("ess"));
    let file = root.join("ess/domains/cli.yaml");
    let mut domain: Value = serde_yaml_ng::from_str(
        &fs::read_to_string(&file)
            .unwrap_or_else(|error| panic!("{} reads: {error}", file.display())),
    )
    .expect("ess/domains/cli.yaml is YAML");
    let probe: Value = serde_yaml_ng::from_str(
        "name: canon.cli.Probe
naming:
  wire: probe
  display: Probe
input:
  - name: probe_input
    type: String
response:
  - name: output
    type: String
outcomes:
  - name: done
    returns: true
",
    )
    .expect("the probe command is YAML");
    domain["commands"]
        .as_sequence_mut()
        .expect("ess/domains/cli.yaml declares commands")
        .push(probe);
    fs::write(
        &file,
        serde_yaml_ng::to_string(&domain).expect("the domain serializes"),
    )
    .expect("the edited copy is written");

    assert_named(
        &clap_surface(&canon_cli::Cli::command()),
        &ess_surface(&root),
        &["canon.cli.Probe", "`probe`", "not a canon subcommand"],
    );
}

/// A flag clap requires that ESS declares optional fails, naming the flag and both sides.
#[test]
fn a_changed_requiredness_is_named() {
    let spec = ess_surface(&repo_root());
    let command = canon_cli::Cli::command().mut_subcommand("evaluate", |evaluate| {
        evaluate.mut_arg("authority", |authority| authority.required(true))
    });
    assert_named(
        &clap_surface(&command),
        &spec,
        &[
            "canon evaluate --authority",
            "clap says required",
            "canon.cli.Evaluate",
            "says optional",
        ],
    );
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap_or_else(|error| panic!("create {}: {error}", to.display()));
    for entry in
        fs::read_dir(from).unwrap_or_else(|error| panic!("read {}: {error}", from.display()))
    {
        let path = entry.expect("directory entry").path();
        let target = to.join(path.file_name().expect("entry has a name"));
        if path.is_dir() {
            copy_dir(&path, &target);
        } else {
            fs::copy(&path, &target)
                .unwrap_or_else(|error| panic!("copy {}: {error}", path.display()));
        }
    }
}
