//! Adversary pass 2 on story:ess-hard-gate: the gate's documents read against what runs.
//!
//! Every path is read at run time from the tree under test.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(&std::env::var_os("CARGO_MANIFEST_DIR").expect("set by cargo"))
        .join("../..")
        .canonicalize()
        .expect("repository root resolves")
}

fn ess(args: &[&str]) -> String {
    let output = Command::new("ess")
        .args(args)
        .current_dir(repo_root())
        .output()
        .expect("ess runs");
    assert!(
        output.status.success(),
        "`ess {}` failed:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Expectation 6: "`ess specify toolchain which`, run from the repository root, names ess 0.52.0,
/// which is the release that `requires:` pins." The ess dispatcher looks for `ess-inputs.yaml` in
/// the working directory and above it, never below, so from the root it does not find
/// `ess/ess-inputs.yaml` and names whichever `ess` is on `PATH` (reason `self: no ess-inputs.yaml`).
/// The unit's `toolchain_is_the_pinned_release` (ess_gate.rs) reads only the first line, so it
/// passes for any `requires:` while `PATH` holds 0.52.0.
#[test]
fn toolchain_which_from_the_root_names_the_pin() {
    // Coordinator decision (F2): expectation 6 runs from ess/, where ess-inputs.yaml is.
    let output = Command::new("ess")
        .args(["specify", "toolchain", "which"])
        .current_dir(repo_root().join("ess"))
        .output()
        .expect("ess runs");
    assert!(
        output.status.success(),
        "`ess specify toolchain which` failed"
    );
    let printed = String::from_utf8_lossy(&output.stdout).into_owned();
    let reason = printed
        .lines()
        .find_map(|line| line.strip_prefix("reason: "))
        .unwrap_or_default();
    assert!(
        reason.starts_with("pin: `requires: ess 0.53.0` in")
            && reason.ends_with("ess/ess-inputs.yaml"),
        "`ess specify toolchain which` from the repository root does not take its release from \
         ess/ess-inputs.yaml:\n{printed}"
    );
}

/// AGENTS.md § ESS says ESS specifies Canon's command surface. Compiled, `ess/` declares no
/// command, while `canon` (crates/canon-cli) has one. ADR 0076 § Canon requires the command
/// surface; the story put it out of scope, so the document states as done what is owed.
#[test]
fn agents_md_command_surface_claim_is_backed_by_ess() {
    let agents = fs::read_to_string(repo_root().join("AGENTS.md")).expect("AGENTS.md reads");
    let section = agents
        .split("\n## ")
        .find(|section| section.starts_with("ESS\n"))
        .expect("AGENTS.md has § ESS");
    let claims_commands = section.split_whitespace().collect::<Vec<_>>().join(" ");
    if !claims_commands.contains("command surface") {
        return;
    }
    let ir: serde_yaml_ng::Value = serde_yaml_ng::from_str(&ess(&[
        "specify", "compile", "--path", "ess", "--format", "json",
    ]))
    .expect("compiled IR is JSON");
    let commands = ir["commands"]
        .as_mapping()
        .map_or(0, |commands| commands.len());
    let cli = fs::read_to_string(repo_root().join("crates/canon-cli/src/main.rs"))
        .expect("canon-cli reads");
    assert!(
        commands > 0,
        "AGENTS.md § ESS says ESS specifies Canon's command surface; the compiled specification \
         declares {commands} command(s) while crates/canon-cli/src/main.rs defines a subcommand \
         enum: {}",
        cli.contains("enum Command")
    );
}
